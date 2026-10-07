use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::i18n::{self, LangPref};
use crate::theme::ThemeMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OverwritePolicy {
    #[default]
    Rename, // auto-rename name (1).mov
    Overwrite,
    Skip,
}

impl std::fmt::Display for OverwritePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(i18n::t(match self {
            OverwritePolicy::Rename => "ow_rename",
            OverwritePolicy::Overwrite => "ow_overwrite",
            OverwritePolicy::Skip => "ow_skip",
        }))
    }
}

/// Target container. `Auto` flips mp4/mov and sends mkv/webm to mp4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Container {
    #[default]
    Auto,
    Mp4,
    Mov,
    Mkv,
    Webm,
}

impl Container {
    /// File extension; empty for `Auto` (resolved per input).
    pub fn ext(self) -> &'static str {
        match self {
            Container::Auto => "",
            Container::Mp4 => "mp4",
            Container::Mov => "mov",
            Container::Mkv => "mkv",
            Container::Webm => "webm",
        }
    }

    /// Default target for an input: mp4/m4v -> mov; mov/mkv/webm -> mp4.
    pub fn default_for(path: &Path) -> Container {
        match crate::util::auto_target(path) {
            "mp4" => Container::Mp4,
            _ => Container::Mov,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Container::Auto => "cont_auto",
            Container::Mp4 => "cont_mp4",
            Container::Mov => "cont_mov",
            Container::Mkv => "cont_mkv",
            Container::Webm => "cont_webm",
        }
    }
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(i18n::t(self.key()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub output_dir: Option<PathBuf>, // None = same directory as the source file
    pub overwrite: OverwritePolicy,
    pub concurrency: usize,
    pub faststart: bool,
    #[serde(default)]
    pub first_audio_only: bool,
    #[serde(default)]
    pub keep_subtitles: bool,
    #[serde(default)]
    pub window_size: Option<(f32, f32)>, // remembered across launches
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default)]
    pub language: LangPref,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: None,
            overwrite: OverwritePolicy::default(),
            concurrency: 2,
            faststart: true,
            first_audio_only: false,
            keep_subtitles: false,
            window_size: None,
            theme: ThemeMode::default(),
            language: LangPref::default(),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("vidly").join("config.json"))
}

/// Directory holding rotating log files.
pub fn log_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("vidly").join("logs"))
}

/// Synchronous load, used before the window is created (to restore its size).
pub fn load_blocking() -> Config {
    let Some(path) = config_path() else { return Config::default() };
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// Synchronous save, used when the window is closing (async tasks may not run).
pub fn save_blocking(cfg: &Config) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(s) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&path, s);
    }
}

pub async fn load() -> Config {
    let Some(path) = config_path() else { return Config::default() };
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            tracing::warn!("配置解析失败，使用默认值: {e}");
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

pub async fn save(cfg: &Config) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    match serde_json::to_string_pretty(cfg) {
        Ok(s) => {
            if let Err(e) = tokio::fs::write(&path, s).await {
                tracing::warn!("保存配置失败: {e}");
            }
        }
        Err(e) => tracing::warn!("序列化配置失败: {e}"),
    }
}
