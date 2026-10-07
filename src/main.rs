#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // no console window in release

mod app;
mod config;
mod convert;
mod history;
mod i18n;
mod icons;
mod theme;
mod util;

use app::App;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Pan-CJK UI font (Noto Sans CJK SC, SIL OFL): covers Latin, Cyrillic, Greek,
/// Han, Kana and Hangul. Embedding it makes text rendering deterministic — no
/// reliance on flaky system-font fallback, so runtime language switching works.
const UI_FONT: &[u8] = include_bytes!("../assets/fonts/NotoSansCJKsc-Regular.otf");
const UI_FONT_FAMILY: &str = "Noto Sans CJK SC";

/// Decodes the bundled PNG into a window icon.
fn load_icon() -> Option<iced::window::Icon> {
    let bytes = include_bytes!("../assets/icon.png");
    let mut decoder = png::Decoder::new(&bytes[..]);
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;

    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(info.buffer_size() / 3 * 4);
            for px in buf[..info.buffer_size()].chunks_exact(3) {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        _ => return None,
    };

    iced::window::icon::from_rgba(rgba, info.width, info.height).ok()
}

fn main() -> iced::Result {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Rotating file log (kept for the whole run; `_log_guard` must not be dropped early).
    let _log_guard = config::log_dir().and_then(|dir| {
        std::fs::create_dir_all(&dir).ok()?;
        let appender = tracing_appender::rolling::Builder::new()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix("vidly")
            .filename_suffix("log")
            .max_log_files(7)
            .build(&dir)
            .ok()?;
        Some(tracing_appender::non_blocking(appender))
    });

    let file_layer = _log_guard.as_ref().map(|(writer, _)| {
        tracing_subscriber::fmt::layer().with_ansi(false).with_writer(writer.clone())
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer()) // stderr (dev / RUST_LOG=debug)
        .with(file_layer)
        .init();

    tracing::info!("Vidly {} starting", env!("CARGO_PKG_VERSION"));

    // Restore the remembered language, theme and window size before the window is created.
    let cfg = config::load_blocking();
    let lang = cfg.language.resolve();
    i18n::set_lang(lang);
    theme::set_mode(cfg.theme);
    let size = cfg.window_size.unwrap_or((760.0, 580.0));

    let window = iced::window::Settings {
        size: size.into(),
        icon: load_icon(),
        ..Default::default()
    };

    let mut application = iced::application("Vidly", App::update, App::view)
        .theme(|_| theme::theme())
        .window(window)
        .subscription(App::subscription);

    application = application
        .font(UI_FONT)
        .default_font(iced::Font::with_name(UI_FONT_FAMILY));

    application.run_with(|| (App::default(), App::init()))
}
