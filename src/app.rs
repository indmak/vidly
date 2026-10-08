use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use futures::SinkExt;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, container, horizontal_space, mouse_area, pick_list, progress_bar,
        row, scrollable, slider, text, tooltip, Column,
    },
    Border, Color, Element, Font, Length, Subscription, Task, Theme,
};
use tokio_util::sync::CancellationToken;

use crate::config::{self, Config, Container, OverwritePolicy};
use crate::convert::{self, ConvertError, FfmpegPaths, Request};
use crate::history;
use crate::i18n::{self, LangPref};
use crate::icons;
use crate::theme::{self, ThemeMode};
use crate::util;

pub type ItemId = u64;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub enum Phase {
    Queued,
    Running { progress: f32 },
    Done { secs: f64 },
    Failed(String),
    Canceled,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub input: PathBuf,
    pub output: PathBuf, // finalized only at launch, according to the overwrite policy
    pub target: Container, // per-item target container (editable in the queue)
    pub size: u64,       // input file size in bytes (0 when unknown)
    // Cached display strings (recomputed only when input/output change, not per frame).
    pub name: String,
    pub out_name: String,
    pub path_label: String,
    pub started: Option<Instant>,
    pub phase: Phase,
}

#[derive(Debug, Clone)]
pub enum Message {
    Init(Option<FfmpegPaths>, Config, Vec<history::Entry>, Vec<bool>),
    PickFiles,
    FilesPicked(Vec<PathBuf>),
    FilesDropped(Vec<PathBuf>),
    FilesScanned(Vec<(PathBuf, u64)>, usize),
    PickOutputDir,
    OutputDirPicked(Option<PathBuf>),
    ResetOutputDir,
    OverwriteChanged(OverwritePolicy),
    ConcurrencyChanged(usize),
    SaveSettings,
    FaststartToggled(bool),
    ItemTargetChanged(ItemId, Container),
    FirstAudioToggled(bool),
    SubtitlesToggled(bool),
    ThemeChanged(ThemeMode),
    LanguageChanged(LangPref),
    StartAll,
    StartItem(ItemId),
    Cancel(ItemId),
    CancelAll,
    Remove(ItemId),
    ClearFinished,
    ClearAll,
    Progress(ItemId, f32),
    Finished(ItemId, Result<f64, ConvertError>),
    OpenOutput(ItemId),
    HoverItem(ItemId),
    UnhoverItem,
    ShowAbout,
    ShowHistory,
    HistoryExists(Vec<bool>),
    HideOverlays,
    ClearHistory,
    OpenUrl(String),
    OpenPath(PathBuf),
    OpenResult(Result<(), String>),
    DismissStatus,
    OpenLogs,
    WindowResized(f32, f32),
    WindowCloseRequested,
    Ignore,
}

#[derive(Default)]
pub struct App {
    items: Vec<Item>,
    tokens: HashMap<ItemId, CancellationToken>, // cancel handles for running items
    ffmpeg: Option<FfmpegPaths>,
    config: Config,
    status: String,
    hovered: Option<ItemId>,
    about_open: bool,
    history_open: bool,
    history: Vec<history::Entry>,
    history_exists: Vec<bool>, // cached output-existence, computed off the render path
}

impl App {
    pub fn init() -> Task<Message> {
        Task::perform(
            async {
                let hist = history::load().await;
                // Compute output existence off the UI thread.
                let exists = hist.iter().map(|e| e.output.is_file()).collect::<Vec<_>>();
                (convert::find_ffmpeg(), config::load().await, hist, exists)
            },
            |(ffmpeg, cfg, hist, exists)| Message::Init(ffmpeg, cfg, hist, exists),
        )
    }

    // ---------- update ----------
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Init(paths, mut cfg, hist, exists) => {
                cfg.concurrency = cfg.concurrency.clamp(1, 8);
                theme::set_mode(cfg.theme);
                i18n::set_lang(cfg.language.resolve());
                self.config = cfg;
                self.ffmpeg = paths;
                self.history = hist;
                self.history_exists = exists;
                match &self.ffmpeg {
                    Some(p) => tracing::info!(ffmpeg = %p.ffmpeg.display(), "FFmpeg 就绪"),
                    None => {
                        tracing::warn!("未检测到 ffmpeg / ffprobe");
                        self.status = i18n::t("no_ffmpeg_status").to_string();
                    }
                }
                Task::none()
            }
            Message::PickFiles => Task::perform(pick_files(), Message::FilesPicked),
            Message::FilesPicked(paths) | Message::FilesDropped(paths) => Task::perform(
                scan_files(paths),
                |(files, ignored)| Message::FilesScanned(files, ignored),
            ),
            Message::FilesScanned(files, ignored) => {
                self.enqueue_scanned(files, ignored);
                Task::none()
            }
            Message::PickOutputDir => Task::perform(pick_dir(), Message::OutputDirPicked),
            Message::OutputDirPicked(Some(dir)) => {
                self.config.output_dir = Some(dir);
                self.relocate_outputs();
                self.save_config()
            }
            Message::OutputDirPicked(None) => Task::none(),
            Message::ResetOutputDir => {
                self.config.output_dir = None;
                self.relocate_outputs();
                self.save_config()
            }
            Message::OverwriteChanged(p) => {
                self.config.overwrite = p;
                self.save_config()
            }
            Message::ConcurrencyChanged(n) => {
                self.config.concurrency = n;
                self.pump() // raising concurrency immediately fills the new slots
            }
            Message::SaveSettings => self.save_config(), // persisted only after the slider is released
            Message::FaststartToggled(b) => {
                self.config.faststart = b;
                self.save_config()
            }
            Message::ItemTargetChanged(id, c) => {
                let dir = self.config.output_dir.clone();
                if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                    item.target = c;
                    if matches!(
                        item.phase,
                        Phase::Queued | Phase::Failed(_) | Phase::Canceled | Phase::Skipped
                    ) {
                        let ext = resolve_ext(c, &item.input);
                        set_output(item, util::planned_output(&item.input, dir.as_deref(), ext));
                    }
                }
                Task::none()
            }
            Message::FirstAudioToggled(b) => {
                self.config.first_audio_only = b;
                self.save_config()
            }
            Message::SubtitlesToggled(b) => {
                self.config.keep_subtitles = b;
                self.save_config()
            }
            Message::ThemeChanged(m) => {
                self.config.theme = m;
                theme::set_mode(m);
                self.save_config()
            }
            Message::LanguageChanged(pref) => {
                self.config.language = pref;
                i18n::set_lang(pref.resolve());
                tracing::info!(lang = i18n::lang().code(), "语言已切换");
                self.save_config()
            }
            Message::StartAll => {
                // Reset failed/canceled/skipped items and start everything together.
                for item in &mut self.items {
                    if matches!(item.phase, Phase::Failed(_) | Phase::Canceled | Phase::Skipped) {
                        item.phase = Phase::Queued;
                        item.started = None;
                    }
                }
                self.start()
            }
            Message::StartItem(id) => {
                let dir = self.config.output_dir.clone();
                if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                    item.phase = Phase::Queued;
                    item.started = None;
                    let ext = resolve_ext(item.target, &item.input);
                    set_output(item, util::planned_output(&item.input, dir.as_deref(), ext));
                }
                self.start()
            }
            Message::Cancel(id) => {
                if let Some(t) = self.tokens.get(&id) {
                    t.cancel();
                }
                Task::none()
            }
            Message::CancelAll => {
                for t in self.tokens.values() {
                    t.cancel();
                }
                Task::none()
            }
            Message::Remove(id) => {
                self.cancel_token(&id);
                self.items.retain(|i| i.id != id);
                if self.hovered == Some(id) {
                    self.hovered = None;
                }
                Task::none()
            }
            Message::ClearFinished => {
                self.items.retain(|i| {
                    !matches!(
                        i.phase,
                        Phase::Done { .. } | Phase::Failed(_) | Phase::Canceled | Phase::Skipped
                    )
                });
                Task::none()
            }
            Message::ClearAll => {
                let ids: Vec<_> = self.items.iter().map(|i| i.id).collect();
                for id in ids {
                    self.cancel_token(&id);
                }
                self.items.clear();
                self.hovered = None;
                Task::none()
            }
            Message::Progress(id, p) => {
                if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                    if let Phase::Running { progress } = &mut item.phase {
                        *progress = p;
                    }
                }
                Task::none()
            }
            Message::Finished(id, result) => {
                self.tokens.remove(&id);
                let (status, secs) = match &result {
                    Ok(secs) => {
                        tracing::info!(id, secs = *secs, "转换完成");
                        (history::Status::Done, *secs)
                    }
                    Err(ConvertError::Canceled) => {
                        tracing::info!(id, "转换已取消");
                        (history::Status::Canceled, 0.0)
                    }
                    Err(e) => {
                        tracing::warn!(id, error = %e, "转换失败");
                        (history::Status::Failed, 0.0)
                    }
                };
                if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                    item.started = None;
                    item.phase = match result {
                        Ok(secs) => Phase::Done { secs },
                        Err(ConvertError::Canceled) => Phase::Canceled,
                        Err(e) => Phase::Failed(e.to_string()),
                    };
                    history::push(
                        &mut self.history,
                        history::Entry {
                            input: item.input.clone(),
                            output: item.output.clone(),
                            status,
                            secs,
                            size: item.size,
                            at: history::now_secs(),
                        },
                    );
                    // Cache the new row's existence (one stat, not per frame).
                    let exists = self
                        .history
                        .first()
                        .map(|e| e.output.is_file())
                        .unwrap_or(false);
                    self.history_exists.insert(0, exists);
                    self.history_exists.truncate(self.history.len());
                }
                Task::batch([self.pump(), self.save_history()]) // one slot freed; persist history
            }
            Message::OpenOutput(id) => match self.items.iter().find(|i| i.id == id) {
                Some(item) => Task::perform(reveal_path(item.output.clone()), Message::OpenResult),
                None => Task::none(),
            },
            Message::HoverItem(id) => {
                self.hovered = Some(id);
                Task::none()
            }
            Message::UnhoverItem => {
                self.hovered = None;
                Task::none()
            }
            Message::ShowAbout => {
                self.about_open = true;
                self.history_open = false;
                Task::none()
            }
            Message::ShowHistory => {
                self.history_open = true;
                self.about_open = false;
                // Compute output existence off the UI thread.
                let hist = self.history.clone();
                Task::perform(
                    async move { hist.iter().map(|e| e.output.is_file()).collect::<Vec<_>>() },
                    Message::HistoryExists,
                )
            }
            Message::HistoryExists(exists) => {
                self.history_exists = exists;
                Task::none()
            }
            Message::HideOverlays => {
                self.about_open = false;
                self.history_open = false;
                Task::none()
            }
            Message::ClearHistory => {
                self.history.clear();
                self.history_exists.clear();
                self.save_history()
            }
            Message::OpenPath(path) => Task::perform(reveal_path(path), Message::OpenResult),
            Message::OpenResult(Ok(())) => Task::none(),
            Message::OpenResult(Err(e)) => {
                self.status = i18n::t("open_failed").replace("{e}", &e);
                Task::none()
            }
            Message::OpenUrl(url) => {
                // Only ever open web links; never hand arbitrary schemes to the OS opener.
                if url.starts_with("https://") || url.starts_with("http://") {
                    Task::perform(open_url(url), Message::OpenResult)
                } else {
                    tracing::warn!(%url, "拒绝打开非 http(s) 链接");
                    Task::none()
                }
            }
            Message::DismissStatus => {
                self.status.clear();
                Task::none()
            }
            Message::OpenLogs => match config::log_dir() {
                Some(dir) => Task::perform(reveal_path(dir), Message::OpenResult),
                None => {
                    self.status = i18n::t("logs_dir_missing").to_string();
                    Task::none()
                }
            },
            Message::WindowResized(w, h) => {
                self.config.window_size = Some((w, h)); // persisted when the window closes
                Task::none()
            }
            Message::WindowCloseRequested => {
                // Synchronous save: async tasks may not get a chance to run before exit.
                config::save_blocking(&self.config);
                Task::none()
            }
            Message::Ignore => Task::none(),
        }
    }

    // ---------- scheduling ----------
    fn start(&mut self) -> Task<Message> {
        if self.ffmpeg.is_none() {
            self.status = i18n::t("cannot_convert").to_string();
            return Task::none();
        }
        if !self.items.iter().any(|i| matches!(i.phase, Phase::Queued)) {
            return Task::none();
        }
        self.pump()
    }

    /// Feed queued items into free slots, bounded by the configured concurrency.
    fn pump(&mut self) -> Task<Message> {
        let Some(paths) = self.ffmpeg.clone() else { return Task::none() };
        let mut tasks = Vec::new();
        loop {
            if self.running_count() >= self.config.concurrency.max(1) {
                break;
            }
            let Some(id) = self
                .items
                .iter()
                .find(|i| matches!(i.phase, Phase::Queued))
                .map(|i| i.id)
            else {
                break;
            };
            let item = self.items.iter_mut().find(|i| i.id == id).unwrap();
            // Never write over the source file itself (that would corrupt / destroy
            // the original) — force a unique name regardless of the overwrite policy.
            if item.output == item.input {
                let renamed = util::unique_path(item.output.clone());
                set_output(item, renamed);
            }
            // Apply the overwrite policy only now.
            match self.config.overwrite {
                OverwritePolicy::Skip if item.output.exists() => {
                    item.phase = Phase::Skipped;
                    item.started = None;
                    continue;
                }
                OverwritePolicy::Rename => {
                    let renamed = util::unique_path(item.output.clone());
                    set_output(item, renamed);
                }
                _ => {}
            }
            let req = Request {
                input: item.input.clone(),
                output: item.output.clone(),
                faststart: self.config.faststart,
                first_audio_only: self.config.first_audio_only,
                keep_subtitles: self.config.keep_subtitles,
            };
            item.phase = Phase::Running { progress: 0.0 };
            item.started = Some(Instant::now());
            let token = CancellationToken::new();
            self.tokens.insert(id, token.clone());
            tracing::info!(
                id,
                input = %req.input.display(),
                output = %req.output.display(),
                faststart = req.faststart,
                "开始转换"
            );
            tasks.push(spawn_conversion(paths.clone(), req, id, token));
        }
        Task::batch(tasks)
    }

    fn running_count(&self) -> usize {
        self.items.iter().filter(|i| matches!(i.phase, Phase::Running { .. })).count()
    }

    fn cancel_token(&mut self, id: &ItemId) {
        if let Some(t) = self.tokens.remove(id) {
            t.cancel();
        }
    }

    /// Enqueues files already scanned off the UI thread (pure in-memory work).
    fn enqueue_scanned(&mut self, files: Vec<(PathBuf, u64)>, ignored: usize) {
        const MAX_ITEMS: usize = 2000; // keep the (non-virtualized) list responsive
        let mut existing: HashSet<PathBuf> =
            self.items.iter().map(|i| i.input.clone()).collect();
        let mut added = 0usize;
        for (p, size) in files {
            if self.items.len() >= MAX_ITEMS {
                tracing::warn!(MAX_ITEMS, "队列达到上限，忽略其余文件");
                break;
            }
            if !existing.insert(p.clone()) {
                continue; // already queued
            }
            let target = Container::default_for(&p);
            let output =
                util::planned_output(&p, self.config.output_dir.as_deref(), resolve_ext(target, &p));
            let name = p
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let out_name = output
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let path_label = if size > 0 {
                format!("{} · {}", p.display(), util::human_size(size))
            } else {
                p.display().to_string()
            };
            self.items.push(Item {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                input: p,
                output,
                target,
                size,
                name,
                out_name,
                path_label,
                started: None,
                phase: Phase::Queued,
            });
            added += 1;
        }
        if added > 0 || ignored > 0 {
            tracing::info!(added, ignored, "文件入队");
        }
        if ignored > 0 {
            self.status = i18n::t("ignored_files").replace("{n}", &ignored.to_string());
        }
    }

    fn relocate_outputs(&mut self) {
        let dir = self.config.output_dir.clone();
        for item in &mut self.items {
            if matches!(
                item.phase,
                Phase::Queued | Phase::Failed(_) | Phase::Canceled | Phase::Skipped
            ) {
                let ext = resolve_ext(item.target, &item.input);
                set_output(item, util::planned_output(&item.input, dir.as_deref(), ext));
            }
        }
    }

    fn save_config(&self) -> Task<Message> {
        let cfg = self.config.clone();
        Task::perform(async move { config::save(&cfg).await }, |_| Message::Ignore)
    }

    fn save_history(&self) -> Task<Message> {
        let entries = self.history.clone();
        Task::perform(async move { history::save(&entries).await }, |_| Message::Ignore)
    }

    // ---------- subscription ----------
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            // Drag & drop (iced 0.13 listen_with takes a window::Id as the third arg).
            iced::event::listen_with(|event, _status, _id| match event {
                iced::event::Event::Window(iced::window::Event::FileDropped(path)) => {
                    Some(Message::FilesDropped(vec![path]))
                }
                iced::event::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(Message::WindowResized(size.width, size.height))
                }
                iced::event::Event::Window(iced::window::Event::CloseRequested) => {
                    Some(Message::WindowCloseRequested)
                }
                _ => None,
            }),
            // Shortcuts: Ctrl+O add files, Ctrl+Enter convert all, Esc close About.
            iced::keyboard::on_key_press(|key, modifiers| {
                use iced::keyboard::{key::Named, Key, Modifiers};
                let ctrl = modifiers.contains(Modifiers::CTRL);
                match key {
                    Key::Character(c) if ctrl && c.as_ref().eq_ignore_ascii_case("o") => {
                        Some(Message::PickFiles)
                    }
                    Key::Named(Named::Enter) if ctrl => Some(Message::StartAll),
                    Key::Named(Named::Escape) => Some(Message::HideOverlays),
                    _ => None,
                }
            }),
        ])
    }

    // ---------- view ----------
    pub fn view(&self) -> Element<'_, Message> {
        if self.about_open {
            return self.view_about();
        }
        if self.history_open {
            return self.view_history();
        }

        let k = theme::tokens();
        let (badge_icon, badge_color) = if self.ffmpeg.is_some() {
            ("check", k.success)
        } else {
            ("alert-triangle", k.warning)
        };
        let header = row![
            column![text("Vidly").size(24), text(i18n::t("subtitle")).size(12)]
                .spacing(2),
            horizontal_space(),
            row![
                icons::icon(badge_icon, 14.0, badge_color),
                text(self.ffmpeg_badge()).size(12).style(theme::secondary_text),
            ]
            .spacing(6)
            .align_y(Vertical::Center),
            button(icon_label("info", i18n::t("about"), k.text_primary))
                .on_press(Message::ShowAbout)
                .style(secondary),
        ]
        .spacing(8)
        .align_y(Vertical::Center);

        // Split into several short rows so the UI never overflows / wraps in
        // narrower windows or in languages with longer labels.
        let toolbar = row![
            button(icon_label("plus", i18n::t("add_files"), k.text_primary))
                .on_press(Message::PickFiles)
                .style(secondary),
            button(icon_label("folder", i18n::t("pick_output"), k.text_primary))
                .on_press(Message::PickOutputDir)
                .style(secondary),
            horizontal_space(),
            button(icon_label("history", i18n::t("history"), k.text_primary))
                .on_press(Message::ShowHistory)
                .style(secondary),
            button(icon_label("file-text", i18n::t("logs"), k.text_primary))
                .on_press(Message::OpenLogs)
                .style(secondary),
        ]
        .spacing(8)
        .align_y(Vertical::Center);

        let toolbar2 = row![
            text(self.output_dir_label()).size(12).style(theme::secondary_text),
            horizontal_space(),
            button(icon_label("rotate-ccw", i18n::t("reset_output"), k.text_primary))
                .on_press_maybe(self.config.output_dir.is_some().then_some(Message::ResetOutputDir))
                .style(secondary),
            button(icon_label("trash", i18n::t("clear_list"), k.text_primary))
                .on_press_maybe((!self.items.is_empty()).then_some(Message::ClearAll))
                .style(secondary),
        ]
        .spacing(8)
        .align_y(Vertical::Center);

        let settings = row![
            text(i18n::t("overwrite")).size(12),
            pick_list(
                vec![OverwritePolicy::Rename, OverwritePolicy::Overwrite, OverwritePolicy::Skip],
                Some(self.config.overwrite),
                Message::OverwriteChanged,
            )
            .style(theme::pick_list_style)
            .menu_style(theme::menu_style),
            text(i18n::t("concurrency")).size(12),
            slider(
                1.0..=8.0,
                self.config.concurrency as f64,
                |v| Message::ConcurrencyChanged(v.round() as usize),
            )
            .step(1.0)
            .on_release(Message::SaveSettings) // persist only after the drag ends
            .width(110),
            checkbox(i18n::t("faststart"), self.config.faststart)
                .on_toggle(Message::FaststartToggled),
            horizontal_space(),
        ]
        .spacing(10)
        .align_y(Vertical::Center);

        let settings2 = row![
            text(i18n::t("theme")).size(12),
            pick_list(
                vec![ThemeMode::Dark, ThemeMode::Light, ThemeMode::System],
                Some(self.config.theme),
                Message::ThemeChanged,
            )
            .style(theme::pick_list_style)
            .menu_style(theme::menu_style),
            text(i18n::t("language")).size(12),
            pick_list(LangPref::choices(), Some(self.config.language), Message::LanguageChanged)
                .style(theme::pick_list_style)
            .menu_style(theme::menu_style),
            horizontal_space(),
        ]
        .spacing(10)
        .align_y(Vertical::Center);

        let settings3 = row![
            checkbox(i18n::t("first_audio_only"), self.config.first_audio_only)
                .on_toggle(Message::FirstAudioToggled),
            checkbox(i18n::t("keep_subtitles"), self.config.keep_subtitles)
                .on_toggle(Message::SubtitlesToggled),
            horizontal_space(),
        ]
        .spacing(10)
        .align_y(Vertical::Center);

        let list: Element<'_, Message> = if self.items.is_empty() {
            container(
                column![
                    icons::icon("download", 40.0, k.text_secondary),
                    text(i18n::t("empty_title")).size(16),
                    text(i18n::t("empty_hint")).size(12).style(theme::secondary_text),
                ]
                .spacing(6)
                .align_x(Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
        } else {
            scrollable(
                Column::with_children(
                    self.items
                        .iter()
                        .map(|i| item_view(i, self.hovered == Some(i.id)))
                        .collect::<Vec<_>>(),
                )
                .spacing(10),
            )
            .height(Length::Fill)
            .into()
        };

        let all_done = !self.items.is_empty()
            && self.items.iter().all(|i| matches!(i.phase, Phase::Done { .. }));
        let progress_style: fn(&Theme) -> iced::widget::progress_bar::Style = if all_done {
            theme::progress_success
        } else {
            theme::progress_accent
        };

        let footer = column![
            progress_bar(0.0..=100.0, self.total_progress()).style(progress_style),
            row![
                text(self.summary()).size(12).style(theme::secondary_text),
                horizontal_space(),
                button(icon_label("play", i18n::t("start_all"), Color::WHITE))
                    .on_press_maybe(self.has_pending().then_some(Message::StartAll))
                    .style(primary),
                button(icon_label("x", i18n::t("cancel_all"), k.text_primary))
                    .on_press_maybe((!self.tokens.is_empty()).then_some(Message::CancelAll))
                    .style(secondary),
                button(icon_label("trash", i18n::t("clear_finished"), k.text_primary))
                    .on_press_maybe(self.has_finished().then_some(Message::ClearFinished))
                    .style(secondary),
            ]
            .spacing(8),
        ]
        .spacing(8);

        let mut root: Column<'_, Message> = Column::new().spacing(12).padding(16);
        root = root.push(header);
        if !self.status.is_empty() {
            let warning = self.ffmpeg.is_none();
            let banner = container(
                row![
                    text(self.status.as_str()).size(12),
                    horizontal_space(),
                    button(icons::icon("x", 12.0, theme::tokens().text_secondary))
                        .on_press(Message::DismissStatus)
                        .style(ghost),
                ]
                .align_y(Vertical::Center),
            )
            .padding([8, 12])
            .style(move |t| status_banner(t, warning));
            root = root.push(banner);
        }
        root = root
            .push(toolbar)
            .push(toolbar2)
            .push(settings)
            .push(settings2)
            .push(settings3)
            .push(list)
            .push(footer);

        container(root).style(theme::page).into()
    }

    fn view_about(&self) -> Element<'_, Message> {
        let version = env!("CARGO_PKG_VERSION");
        let repo = env!("CARGO_PKG_REPOSITORY");

        let card = container(
            column![
                text("Vidly").size(30),
                text(i18n::t("about_version").replace("{v}", version))
                    .size(13)
                    .style(theme::secondary_text),
                text(i18n::t("about_tagline")).size(13),
                text(i18n::t("about_copyright")).size(12).style(theme::secondary_text),
                button(text(i18n::t("about_repo")))
                    .on_press(Message::OpenUrl(repo.into()))
                    .style(link),
                text(i18n::t("about_ffmpeg")).size(11).style(theme::secondary_text),
                text(i18n::t("about_source")).size(11).style(theme::secondary_text),
                text(i18n::t("about_thirdparty")).size(11).style(theme::secondary_text),
                button(text(i18n::t("about_back"))).on_press(Message::HideOverlays).style(primary),
            ]
            .spacing(10)
            .max_width(460),
        )
        .padding(24)
        .style(theme::card);

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(theme::page)
            .into()
    }

    fn view_history(&self) -> Element<'_, Message> {
        let header = row![
            text(i18n::t("history")).size(22),
            horizontal_space(),
            button(icon_label("trash", i18n::t("history_clear"), theme::tokens().text_primary))
                .on_press_maybe((!self.history.is_empty()).then_some(Message::ClearHistory))
                .style(secondary),
            button(icon_label("x", i18n::t("history_close"), Color::WHITE))
                .on_press(Message::HideOverlays)
                .style(primary),
        ]
        .spacing(8)
        .align_y(Vertical::Center);

        let body: Element<'_, Message> = if self.history.is_empty() {
            container(text(i18n::t("history_empty")).size(14).style(theme::secondary_text))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
        } else {
            scrollable(
                Column::with_children(
                    self.history
                        .iter()
                        .enumerate()
                        .map(|(i, e)| {
                            history_row(e, self.history_exists.get(i).copied().unwrap_or(false))
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(10),
            )
            .height(Length::Fill)
            .into()
        };

        container(column![header, body].spacing(12).padding(16))
            .style(theme::page)
            .into()
    }

    fn ffmpeg_badge(&self) -> String {
        if self.ffmpeg.is_some() {
            i18n::t("ffmpeg_ready").to_string()
        } else {
            i18n::t("ffmpeg_missing").to_string()
        }
    }

    fn output_dir_label(&self) -> String {
        match &self.config.output_dir {
            Some(d) => d.display().to_string(),
            None => i18n::t("output_same_dir").to_string(),
        }
    }

    fn total_progress(&self) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let sum: f32 = self
            .items
            .iter()
            .map(|i| match i.phase {
                Phase::Running { progress } => progress,
                Phase::Done { .. } => 100.0,
                _ => 0.0,
            })
            .sum();
        sum / self.items.len() as f32
    }

    fn summary(&self) -> String {
        if self.items.is_empty() {
            return i18n::t("ready").to_string();
        }
        let done = self.items.iter().filter(|i| matches!(i.phase, Phase::Done { .. })).count();
        let failed = self.items.iter().filter(|i| matches!(i.phase, Phase::Failed(_))).count();
        let running = self.running_count();
        let queued = self
            .items
            .iter()
            .filter(|i| matches!(i.phase, Phase::Queued))
            .count();
        i18n::t("summary")
            .replace("{total}", &self.items.len().to_string())
            .replace("{done}", &done.to_string())
            .replace("{failed}", &failed.to_string())
            .replace("{running}", &running.to_string())
            .replace("{queued}", &queued.to_string())
            .replace("{pct}", &format!("{:.0}", self.total_progress()))
    }

    fn has_pending(&self) -> bool {
        self.items.iter().any(|i| {
            matches!(i.phase, Phase::Queued | Phase::Failed(_) | Phase::Canceled | Phase::Skipped)
        })
    }

    fn has_finished(&self) -> bool {
        self.items.iter().any(|i| {
            matches!(
                i.phase,
                Phase::Done { .. } | Phase::Failed(_) | Phase::Canceled | Phase::Skipped
            )
        })
    }
}

// ---------- component styles ----------
fn primary(t: &Theme, status: button::Status) -> button::Style {
    let k = theme::tokens();
    match status {
        button::Status::Hovered => theme::primary_button_hovered(t),
        button::Status::Pressed => {
            let mut style = theme::primary_button(t);
            style.background = Some(Color { a: 0.85, ..k.accent }.into());
            style
        }
        button::Status::Disabled => disabled_button(),
        _ => theme::primary_button(t),
    }
}

fn secondary(t: &Theme, status: button::Status) -> button::Style {
    let k = theme::tokens();
    match status {
        button::Status::Hovered => theme::secondary_button_hovered(t),
        button::Status::Pressed => {
            let mut style = theme::secondary_button(t);
            style.background = Some(k.bg_input.into());
            style.border.color = k.accent;
            style
        }
        button::Status::Disabled => disabled_button(),
        _ => theme::secondary_button(t),
    }
}

fn ghost(t: &Theme, status: button::Status) -> button::Style {
    let k = theme::tokens();
    let (background, text_color) = match status {
        button::Status::Hovered => (k.bg_hover, k.text_primary),
        _ => (Color::TRANSPARENT, k.text_secondary),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..theme::secondary_button(t)
    }
}

fn link(_t: &Theme, status: button::Status) -> button::Style {
    let k = theme::tokens();
    let color = match status {
        button::Status::Hovered => k.accent_hover,
        _ => k.accent,
    };
    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: color,
        border: Border::default(),
        ..Default::default()
    }
}

fn disabled_button() -> button::Style {
    let k = theme::tokens();
    button::Style {
        background: Some(k.bg_surface.into()),
        text_color: k.text_disabled,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: k.border_subtle,
        },
        ..Default::default()
    }
}

fn status_banner(_t: &Theme, warning: bool) -> container::Style {
    let k = theme::tokens();
    let color = if warning { k.warning } else { k.text_secondary };
    container::Style {
        background: Some(Color { a: 0.15, ..color }.into()),
        text_color: Some(color),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color,
        },
        ..Default::default()
    }
}

/// Per-item conversion task: streams progress back and reports completion.
fn spawn_conversion(
    paths: FfmpegPaths,
    req: Request,
    id: ItemId,
    cancel: CancellationToken,
) -> Task<Message> {
    Task::stream(iced::stream::channel(16, move |tx: futures::channel::mpsc::Sender<Message>| {
        let mut tx_done = tx.clone();
        let progress = move |pct: f32| {
            // Dropping a frame or two of progress is fine -> non-blocking try_send
            let mut tx = tx.clone();
            let _ = tx.try_send(Message::Progress(id, pct));
        };
        async move {
            let result = convert::run(&paths, req, cancel, progress).await;
            let _ = tx_done
                .send(Message::Finished(id, result.map(|d| d.as_secs_f64())))
                .await;
        }
    }))
}

/// Icon + label used as button content.
fn icon_label(name: &str, label: &'static str, color: Color) -> Element<'static, Message> {
    row![icons::icon(name, 15.0, color), text(label).size(13)]
        .spacing(6)
        .align_y(Vertical::Center)
        .into()
}

fn history_row(entry: &history::Entry, output_exists: bool) -> Element<'static, Message> {
    let k = theme::tokens();
    let (icon_name, icon_color, status) = match entry.status {
        history::Status::Done => ("check-circle", k.success, i18n::t("hist_done").to_string()),
        history::Status::Failed => ("x-circle", k.error, i18n::t("hist_failed").to_string()),
        history::Status::Skipped => ("skip-forward", k.warning, i18n::t("st_skipped").to_string()),
        history::Status::Canceled => ("ban", k.text_disabled, i18n::t("st_canceled").to_string()),
    };
    let name = entry
        .input
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let out_name = entry
        .output
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let meta = format!("{} · {}", rel_time(entry.at), util::human_size(entry.size));

    let mut actions = row![].spacing(6);
    if output_exists {
        actions = actions.push(
            button(icon_label("folder-open", i18n::t("act_open"), k.text_primary))
                .on_press(Message::OpenPath(entry.output.clone()))
                .style(secondary),
        );
    }

    container(
        row![
            icons::icon(icon_name, 18.0, icon_color),
            column![
                row![
                    text(format!("{name} → {out_name}"))
                        .size(13)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::Word),
                    text(status).size(12).wrapping(text::Wrapping::None),
                ]
                .spacing(8)
                .align_y(Vertical::Center),
                text(meta).size(11).style(theme::secondary_text),
            ]
            .spacing(4)
            .width(Length::Fill),
            actions,
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .padding(12)
    .style(theme::card)
    .into()
}

fn rel_time(at: u64) -> String {
    let delta = history::now_secs().saturating_sub(at);
    if delta < 60 {
        i18n::t("time_now").to_string()
    } else if delta < 3600 {
        i18n::t("time_min").replace("{n}", &(delta / 60).to_string())
    } else if delta < 86_400 {
        i18n::t("time_hour").replace("{n}", &(delta / 3600).to_string())
    } else {
        i18n::t("time_day").replace("{n}", &(delta / 86_400).to_string())
    }
}

fn resolve_ext(target: Container, input: &Path) -> &'static str {
    match target {
        Container::Auto => util::auto_target(input),
        c => c.ext(),
    }
}

/// Sets an item's output path and keeps the cached display name in sync.
fn set_output(item: &mut Item, output: PathBuf) {
    item.out_name = output
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    item.output = output;
}

fn item_view(item: &Item, hovered: bool) -> Element<'static, Message> {
    let k = theme::tokens();
    let (icon_name, status) = match &item.phase {
        Phase::Queued => ("clock", i18n::t("st_waiting").to_string()),
        Phase::Running { .. } => ("loader", i18n::t("st_converting").to_string()),
        Phase::Done { secs } => {
            ("check-circle", i18n::t("st_done").replace("{secs}", &format!("{secs:.1}")))
        }
        Phase::Failed(e) => ("x-circle", e.clone()),
        Phase::Canceled => ("ban", i18n::t("st_canceled").to_string()),
        Phase::Skipped => ("skip-forward", i18n::t("st_skipped").to_string()),
    };
    let icon_color = match &item.phase {
        Phase::Queued => k.text_secondary,
        Phase::Running { .. } => k.accent,
        Phase::Done { .. } => k.success,
        Phase::Failed(_) => k.error,
        Phase::Canceled => k.text_disabled,
        Phase::Skipped => k.warning,
    };
    let info: Element<'static, Message> = match &item.phase {
        Phase::Running { progress } => {
            let p = *progress;
            let eta = item.started.map(|s| {
                let elapsed = s.elapsed().as_secs_f64();
                if p > 1.0 {
                    elapsed * (100.0 - p as f64) / p as f64
                } else {
                    f64::NAN
                }
            });
            let eta_txt = match eta {
                Some(v) if v.is_finite() && v >= 0.0 => format!(
                    " · {}",
                    i18n::t("remaining").replace("{secs}", &format!("{v:.1}"))
                ),
                _ => String::new(),
            };
            column![
                progress_bar(0.0..=100.0, p).style(theme::progress_accent),
                text(format!("{p:.0}%{eta_txt}"))
                    .size(11)
                    .font(Font::MONOSPACE)
                    .style(theme::secondary_text),
            ]
            .spacing(2)
            .into()
        }
        _ => text(item.path_label.clone())
            .size(11)
            .width(Length::Fill)
            .wrapping(text::Wrapping::Word)
            .style(theme::secondary_text)
            .into(),
    };

    let body = column![
        row![
            text(format!("{} → {}", item.name, item.out_name))
                .size(13)
                .width(Length::Fill)
                .wrapping(text::Wrapping::Word),
            text(status).size(12).wrapping(text::Wrapping::None),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
        info,
    ]
    .spacing(4)
    .width(Length::Fill);

    let mut actions = row![].spacing(6);
    actions = match &item.phase {
        Phase::Queued => actions.push(
            button(icon_label("play", i18n::t("act_start"), k.text_primary))
                .on_press(Message::StartItem(item.id))
                .style(secondary),
        ),
        Phase::Running { .. } => actions.push(
            button(icon_label("x", i18n::t("act_cancel"), k.text_primary))
                .on_press(Message::Cancel(item.id))
                .style(secondary),
        ),
        Phase::Done { .. } => actions.push(
            button(icon_label("folder-open", i18n::t("act_open"), k.text_primary))
                .on_press(Message::OpenOutput(item.id))
                .style(secondary),
        ),
        _ => actions.push(
            button(icon_label("rotate-cw", i18n::t("act_retry"), k.text_primary))
                .on_press(Message::StartItem(item.id))
                .style(secondary),
        ),
    };
    if !matches!(item.phase, Phase::Running { .. }) {
        let remove = button(icons::icon("x", 14.0, k.text_secondary))
            .on_press(Message::Remove(item.id))
            .style(ghost);
        actions = actions.push(tooltip(
            remove,
            text(i18n::t("act_remove")).size(12),
            tooltip::Position::Top,
        ));
    }

    let item_id = item.id;
    let target_pick = pick_list(
        vec![Container::Auto, Container::Mp4, Container::Mov, Container::Mkv, Container::Webm],
        Some(item.target),
        move |c| Message::ItemTargetChanged(item_id, c),
    )
    .style(theme::pick_list_style)
    .menu_style(theme::menu_style);

    let card = container(
        row![
            icons::icon(icon_name, 18.0, icon_color),
            body,
            target_pick,
            actions
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .padding(12)
    .style(if hovered { theme::card_hovered } else { theme::card });

    mouse_area(card)
        .on_enter(Message::HoverItem(item.id))
        .on_exit(Message::UnhoverItem)
        .into()
}

/// Scans dropped paths on a blocking thread (never on the UI thread).
async fn scan_files(paths: Vec<PathBuf>) -> (Vec<(PathBuf, u64)>, usize) {
    tokio::task::spawn_blocking(move || scan_files_blocking(paths))
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "扫描文件失败");
            (Vec::new(), 0)
        })
}

fn scan_files_blocking(paths: Vec<PathBuf>) -> (Vec<(PathBuf, u64)>, usize) {
    // Bounds for folder expansion: guard against symlink cycles and huge trees.
    const MAX_DIR_DEPTH: u32 = 32;
    // The list is not virtualized (the whole queue is rebuilt each frame), so cap
    // the count to keep rendering responsive.
    const MAX_FILES: usize = 2000;

    let mut files = Vec::new();
    let mut ignored = 0usize;
    let mut visited_dirs: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<(PathBuf, u32)> = paths.into_iter().map(|p| (p, 0)).collect();
    while let Some((p, depth)) = queue.pop_front() {
        if files.len() >= MAX_FILES {
            tracing::warn!(MAX_FILES, "文件数量达到上限，停止展开");
            break;
        }
        if p.is_dir() {
            if depth >= MAX_DIR_DEPTH {
                continue;
            }
            // Canonicalize to detect symlink cycles (same dir reached twice).
            if let Ok(real) = p.canonicalize() {
                if !visited_dirs.insert(real) {
                    continue;
                }
            }
            if let Ok(entries) = std::fs::read_dir(&p) {
                for e in entries.flatten() {
                    queue.push_back((e.path(), depth + 1));
                }
            }
        } else if util::is_supported(&p) {
            let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            files.push((p, size));
        } else {
            ignored += 1;
        }
    }
    (files, ignored)
}

/// Reveals a path in the OS file manager without blocking the UI thread.
async fn reveal_path(path: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || opener::reveal(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Opens a URL without blocking the UI thread.
async fn open_url(url: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || opener::open(&url))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

async fn pick_files() -> Vec<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(i18n::t("dlg_pick_files"))
        .add_filter(i18n::t("dlg_filter"), &["mp4", "mov", "m4v", "mkv", "webm"])
        .pick_files()
        .await
        .map(|hs| hs.into_iter().map(|h| h.path().to_path_buf()).collect())
        .unwrap_or_default()
}

async fn pick_dir() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(i18n::t("dlg_pick_dir"))
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}
