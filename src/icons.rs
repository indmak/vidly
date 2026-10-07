//! Embedded monochrome SVG icons (Lucide-style), tinted per use.
//!
//! The SVGs live in `assets/icons/` and are embedded at compile time, so no
//! runtime asset packaging is needed.
use iced::{widget::svg, Color};

pub fn handle(name: &str) -> svg::Handle {
    let bytes: &'static [u8] = match name {
        "plus" => include_bytes!("../assets/icons/plus.svg"),
        "x" => include_bytes!("../assets/icons/x.svg"),
        "folder" => include_bytes!("../assets/icons/folder.svg"),
        "folder-open" => include_bytes!("../assets/icons/folder-open.svg"),
        "rotate-ccw" => include_bytes!("../assets/icons/rotate-ccw.svg"),
        "rotate-cw" => include_bytes!("../assets/icons/rotate-cw.svg"),
        "history" => include_bytes!("../assets/icons/history.svg"),
        "file-text" => include_bytes!("../assets/icons/file-text.svg"),
        "info" => include_bytes!("../assets/icons/info.svg"),
        "trash" => include_bytes!("../assets/icons/trash.svg"),
        "play" => include_bytes!("../assets/icons/play.svg"),
        "clock" => include_bytes!("../assets/icons/clock.svg"),
        "loader" => include_bytes!("../assets/icons/loader.svg"),
        "check-circle" => include_bytes!("../assets/icons/check-circle.svg"),
        "x-circle" => include_bytes!("../assets/icons/x-circle.svg"),
        "ban" => include_bytes!("../assets/icons/ban.svg"),
        "skip-forward" => include_bytes!("../assets/icons/skip-forward.svg"),
        "download" => include_bytes!("../assets/icons/download.svg"),
        "alert-triangle" => include_bytes!("../assets/icons/alert-triangle.svg"),
        "check" => include_bytes!("../assets/icons/check.svg"),
        _ => include_bytes!("../assets/icons/x.svg"),
    };
    svg::Handle::from_memory(bytes)
}

/// A tinted icon of the given pixel size.
pub fn icon(name: &str, size: f32, color: Color) -> iced::widget::Svg<'static> {
    svg(handle(name))
        .width(size)
        .height(size)
        .style(move |_theme, _status| svg::Style { color: Some(color) })
}
