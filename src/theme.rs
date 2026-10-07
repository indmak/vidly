//! Custom theme (dark / light / follow-system) + component styles.
//! See docs/UI_DESIGN.md.
// The theme module is a design-token library: not every variant is wired into the
// UI yet (e.g. hover/pressed states, success progress bar), so unused items are fine.
#![allow(dead_code)]

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::{
    overlay::menu,
    theme,
    widget::{button, container, pick_list, progress_bar, text},
    Border, Color,
};
use serde::{Deserialize, Serialize};

// ───────────────────────── Theme mode ─────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Dark => "theme_dark",
            ThemeMode::Light => "theme_light",
            ThemeMode::System => "theme_system",
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(crate::i18n::t(self.label()))
    }
}

static MODE: AtomicU8 = AtomicU8::new(0); // 0=Dark 1=Light 2=System
// Cached result of OS theme detection (avoid querying the OS on every frame).
static RESOLVED: AtomicU8 = AtomicU8::new(0); // 0=Dark 1=Light
static RESOLVED_AT: AtomicU64 = AtomicU64::new(0); // unix secs of last detection
const RESOLVE_TTL_SECS: u64 = 3;

fn encode(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Dark => 0,
        ThemeMode::Light => 1,
        ThemeMode::System => 2,
    }
}

pub fn set_mode(mode: ThemeMode) {
    MODE.store(encode(mode), Ordering::Relaxed);
    RESOLVED_AT.store(0, Ordering::Relaxed); // force re-detection next time
}

pub fn mode() -> ThemeMode {
    match MODE.load(Ordering::Relaxed) {
        1 => ThemeMode::Light,
        2 => ThemeMode::System,
        _ => ThemeMode::Dark,
    }
}

/// Resolves `System` to the actual OS preference (cached for `RESOLVE_TTL_SECS`,
/// so rendering never hits the OS on every frame).
pub fn resolved() -> ThemeMode {
    match mode() {
        ThemeMode::System => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let last = RESOLVED_AT.load(Ordering::Relaxed);
            if last != 0 && now.saturating_sub(last) < RESOLVE_TTL_SECS {
                return if RESOLVED.load(Ordering::Relaxed) == 1 {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                };
            }
            let detected = match dark_light::detect() {
                dark_light::Mode::Light => ThemeMode::Light,
                _ => ThemeMode::Dark,
            };
            RESOLVED.store(if detected == ThemeMode::Light { 1 } else { 0 }, Ordering::Relaxed);
            RESOLVED_AT.store(now, Ordering::Relaxed);
            detected
        }
        other => other,
    }
}

// ───────────────────────── Design tokens ─────────────────────────
pub struct Tokens {
    pub bg_base: Color,
    pub bg_surface: Color,
    pub bg_hover: Color,
    pub bg_input: Color,
    pub border_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
}

const fn rgb(r: u32, g: u32, b: u32) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

const DARK: Tokens = Tokens {
    bg_base: rgb(0x16, 0x18, 0x1D),
    bg_surface: rgb(0x1E, 0x21, 0x28),
    bg_hover: rgb(0x26, 0x2A, 0x33),
    bg_input: rgb(0x11, 0x13, 0x18),
    border_subtle: rgb(0x2A, 0x2E, 0x38),
    text_primary: rgb(0xE8, 0xEA, 0xED),
    text_secondary: rgb(0x9A, 0xA0, 0xA8),
    text_disabled: rgb(0x5A, 0x5F, 0x68),
    accent: rgb(0x6C, 0x8C, 0xFF),
    accent_hover: rgb(0x8A, 0xA4, 0xFF),
    success: rgb(0x3F, 0xB9, 0x50),
    error: rgb(0xF8, 0x71, 0x71),
    warning: rgb(0xFB, 0xBF, 0x24),
};

// Light palette (docs/UI_DESIGN.md); accent darkened for contrast on white.
const LIGHT: Tokens = Tokens {
    bg_base: rgb(0xF7, 0xF8, 0xFA),
    bg_surface: rgb(0xFF, 0xFF, 0xFF),
    bg_hover: rgb(0xEF, 0xF1, 0xF4),
    bg_input: rgb(0xEC, 0xEE, 0xF2),
    border_subtle: rgb(0xE3, 0xE6, 0xEB),
    text_primary: rgb(0x1A, 0x1D, 0x23),
    text_secondary: rgb(0x5F, 0x66, 0x72),
    text_disabled: rgb(0xA0, 0xA6, 0xB0),
    accent: rgb(0x4A, 0x68, 0xE0),
    accent_hover: rgb(0x63, 0x80, 0xE8),
    success: rgb(0x22, 0xA0, 0x6B),
    error: rgb(0xD1, 0x43, 0x43),
    warning: rgb(0xB7, 0x79, 0x1F),
};

pub fn tokens() -> &'static Tokens {
    if resolved() == ThemeMode::Light {
        &LIGHT
    } else {
        &DARK
    }
}

// ───────────────────────── Custom theme ─────────────────────────
pub fn theme() -> theme::Theme {
    use iced::theme::Palette;
    static DARK: OnceLock<theme::Theme> = OnceLock::new();
    static LIGHT: OnceLock<theme::Theme> = OnceLock::new();
    let light = resolved() == ThemeMode::Light;
    let slot = if light { &LIGHT } else { &DARK };
    slot.get_or_init(|| {
        let k = tokens();
        theme::Theme::custom(
            (if light { "Vidly Light" } else { "Vidly Dark" }).into(),
            Palette {
                background: k.bg_base,
                text: k.text_primary,
                primary: k.accent,
                success: k.success,
                danger: k.error,
            },
        )
    })
    .clone()
}

// ───────────────────────── Style functions ─────────────────────────
pub fn page(_t: &theme::Theme) -> container::Style {
    let k = tokens();
    container::Style {
        background: Some(k.bg_base.into()),
        text_color: Some(k.text_primary),
        ..Default::default()
    }
}

pub fn card(_t: &theme::Theme) -> container::Style {
    let k = tokens();
    container::Style {
        background: Some(k.bg_surface.into()),
        text_color: Some(k.text_primary),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: k.border_subtle,
        },
        ..Default::default()
    }
}

pub fn card_hovered(_t: &theme::Theme) -> container::Style {
    let k = tokens();
    container::Style {
        background: Some(k.bg_hover.into()),
        text_color: Some(k.text_primary),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: k.border_subtle,
        },
        ..Default::default()
    }
}

pub fn primary_button(_t: &theme::Theme) -> button::Style {
    let k = tokens();
    button::Style {
        background: Some(k.accent.into()),
        text_color: Color::WHITE,
        border: Border {
            radius: 8.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

pub fn primary_button_hovered(_t: &theme::Theme) -> button::Style {
    let k = tokens();
    button::Style {
        background: Some(k.accent_hover.into()),
        text_color: Color::WHITE,
        ..primary_button(_t)
    }
}

pub fn secondary_button(_t: &theme::Theme) -> button::Style {
    let k = tokens();
    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: k.text_primary,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: k.border_subtle,
        },
        ..Default::default()
    }
}

pub fn secondary_button_hovered(_t: &theme::Theme) -> button::Style {
    let k = tokens();
    button::Style {
        background: Some(k.bg_hover.into()),
        text_color: k.text_primary,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: k.border_subtle,
        },
        ..Default::default()
    }
}

pub fn progress_accent(_t: &theme::Theme) -> progress_bar::Style {
    let k = tokens();
    progress_bar::Style {
        background: k.bg_input.into(),
        bar: k.accent.into(),
        border: Border {
            radius: 3.0.into(),
            ..Default::default()
        },
    }
}

pub fn progress_success(_t: &theme::Theme) -> progress_bar::Style {
    let k = tokens();
    progress_bar::Style {
        background: k.bg_input.into(),
        bar: k.success.into(),
        border: Border {
            radius: 3.0.into(),
            ..Default::default()
        },
    }
}

/// Dropdown (pick list): input-colored surface, subtle border, accent when open.
pub fn pick_list_style(_t: &theme::Theme, status: pick_list::Status) -> pick_list::Style {
    let k = tokens();
    let (background, border_color) = match status {
        pick_list::Status::Hovered => (k.bg_hover, k.border_subtle),
        pick_list::Status::Opened => (k.bg_hover, k.accent),
        pick_list::Status::Active => (k.bg_input, k.border_subtle),
    };
    pick_list::Style {
        text_color: k.text_primary,
        placeholder_color: k.text_secondary,
        handle_color: k.text_secondary,
        background: background.into(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
    }
}

/// Opened dropdown menu: surface background, accent for the selected row.
pub fn menu_style(_t: &theme::Theme) -> menu::Style {
    let k = tokens();
    menu::Style {
        background: k.bg_surface.into(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: k.border_subtle,
        },
        text_color: k.text_primary,
        selected_text_color: Color::WHITE,
        selected_background: k.accent.into(),
    }
}

/// Muted caption text used for paths / stats.
pub fn secondary_text(_t: &theme::Theme) -> text::Style {
    text::Style { color: Some(tokens().text_secondary) }
}
