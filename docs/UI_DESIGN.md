# Vidly — UI Design

> Single-window desktop utility (file queue + progress). Framework: **iced 0.13**.
> This document describes the design language, tokens, components and states that
> are actually implemented in `src/theme.rs` and `src/app.rs`.

## 1. Design language

A restrained, modern desktop-tool aesthetic: **flat surfaces + hairline borders +
a single accent color + generous spacing + light, state-driven feedback**.
No glassmorphism, no layered shadows, no gradients, no animated icons — iced has
no shadow/animation system and they do not fit a "quiet, reliable" tool.

| Keyword | In Vidly |
|---|---|
| Flat + hairline borders | Panels are solid fills; 1px `border_subtle` separates layers |
| One accent color | A single accent (`#6C8CFF`) for interactive/in-progress states |
| Generous spacing | 8pt grid; page padding 16px; list item gap 10px |
| State-driven feedback | Progress bars fill with the accent; buttons use hover/pressed states |
| Monospace numerals | Percent / ETA / sizes use `Font::MONOSPACE` to avoid jitter |
| Low-noise icons | Single-color Lucide SVGs; color only for status |

## 2. iced capability constraints

| Capability | iced 0.13 | How Vidly handles it |
|---|---|---|
| Dark/light theme | ✅ custom `Theme` + `Palette` | Custom dark & light themes (not the built-in blue) |
| Monochrome SVG icons | ✅ `svg` feature | Embedded Lucide SVGs, tinted per state |
| Rounded corners | ✅ `border::rounded(n)` | 8px (buttons/cards/progress), 6px (small controls) |
| Borders | ✅ `Border` | 1px hairline dividers |
| Four button states | ✅ `style(&Theme, Status)` | active / hovered / pressed / disabled |
| Progress bar | ✅ custom style | Accent fill; success fill when done |
| Text styling | ✅ `size/font/style` | Two text levels + monospace numerals |
| Scrolling list | ✅ `scrollable` | Queue area |
| File drop | ✅ `window::Event::FileDropped` | Empty-state guidance |
| Shadows / glass / animations | ❌ | Avoided; borders + background layers instead |
| Custom title bar | ⚠️ high complexity | Avoided — system title bar |

## 3. Design tokens

### Colors (dark theme)

| Token | Value | Use |
|---|---|---|
| `bg_base` | `#16181D` | Window background |
| `bg_surface` | `#1E2128` | Cards / list items / menus |
| `bg_hover` | `#262A33` | Hover background |
| `bg_input` | `#111318` | Inputs, dropdowns, slider track |
| `border_subtle` | `#2A2E38` | 1px dividers / borders |
| `text_primary` | `#E8EAED` | Primary text |
| `text_secondary` | `#9AA0A8` | Secondary text (paths, stats) |
| `text_disabled` | `#5A5F68` | Disabled text |
| `accent` | `#6C8CFF` | The only accent: primary buttons, progress, selection |
| `accent_hover` | `#8AA4FF` | Accent hover |
| `success` | `#3FB950` | "Done" only |
| `error` | `#F87171` | "Failed" only |
| `warning` | `#FBBF24` | "Skipped" / "FFmpeg missing" |

Light theme tokens (used when the light theme is active): `bg_base #F7F8FA`,
`bg_surface #FFFFFF`, `bg_hover #EFF1F4`, `bg_input #ECEEF2`,
`border_subtle #E3E6EB`, `text_primary #1A1D23`, `text_secondary #5F6672`,
`accent #4A68E0`.

> **Single-accent rule**: colors other than `accent` express **state only**, never
> decoration. e.g. "Convert all" is accent-filled white text, "Cancel" is an
> outlined gray button.

### Typography

| Level | Size | Color | Use |
|---|---|---|---|
| H1 | 24 | `text_primary` | App title "Vidly" |
| Body | 13 | `text_primary` | File names, buttons |
| Caption | 11–12 | `text_secondary` | Paths, stats, descriptions |
| Mono | 11 | `text_secondary` | Percent / ETA (monospace) |

CJK/Arabic scripts render via the embedded **Noto Sans CJK SC** font, so text is
deterministic across all supported languages.

### Spacing & radius

`space_xs 4 · space_s 8 · space_m 12 · space_l 16 · space_xl 24`
`radius_s 6 (small controls) · radius_m 8 (buttons/cards/progress)`

## 4. Components

### Buttons

| Variant | Default | Hovered | Pressed | Disabled |
|---|---|---|---|---|
| **Primary** (Convert all / Start) | accent fill, white text | `accent_hover` | accent at 85% alpha | `bg_surface` + `text_disabled` |
| **Secondary** (Cancel / Remove) | transparent, `border_subtle`, `text_primary` | `bg_hover` | `bg_input` + accent border | disabled style |
| **Ghost** (icon-only ✕) | transparent, `text_secondary` | `bg_hover` + `text_primary` | — | — |

Buttons pair a tinted SVG icon with a label (icon-only buttons get a `tooltip`).

### Progress bar

- Track `bg_input`, fill `accent` while running, `success` when the whole queue is done.
- Per-item bars are thin; the footer total-progress bar is lighter.

### Queue item (card)

```
┌───────────────────────────────────────────────────────────┐
│ ⏳  name.mp4 → name.mov                        [启动] [MOV ▾] [✕] │
│     /path/to/name.mp4 · 43.8 MB                            │
│     ▓▓▓▓▓▓▓░░░░░░░░░░░░░  42% · 3.2s left                  │
└───────────────────────────────────────────────────────────┘
```

- Container `bg_surface`, `radius_m`, 1px border, 12px padding; hover → `bg_hover`.
- Status icon is a tinted SVG per state (clock/loader/check-circle/x-circle/ban/skip-forward).
- Each row has a **per-file target container** dropdown (Auto / MP4 / MOV / MKV / WebM).

### Inputs

- Dropdowns: `bg_input`, hairline border, accent border when open; menu uses
  `bg_surface` with an accent-selected row.
- Slider (concurrency): `bg_input` track, accent handle.
- Checkbox: accent fill when checked.

### Empty state

Centered: a download icon + "Drop MP4 / MOV files here" + "or click Add files (Ctrl+O)".

## 5. Layout

Single window (default 760×580, remembered). Vertical sections:

```
① Header      Vidly · MP4 / MOV · Lossless remux     FFmpeg ✓ · About
② Toolbar     Add files · Output folder      History · Logs
              output dir label               Next to source · Clear list
③ Settings    Overwrite · Parallel · faststart
              Theme · Language
              Only first audio track · Keep subtitles
④ Queue       (scrollable list of cards, or the empty state)
⑤ Footer      total progress bar · summary · Convert all · Cancel all · Clear finished
```

- Settings are split across short rows so the UI never overflows / wraps in
  narrow windows or longer languages.
- The footer is fixed; the queue list takes the remaining height.
- A warning banner (e.g. "FFmpeg not found") appears under the header with a
  15%-alpha warning background and a dismiss button.

## 6. States & feedback

| State | Color | Icon | Text |
|---|---|---|---|
| Queued | `text_secondary` | clock | Waiting |
| Running | `accent` | loader | Converting… |
| Done | `success` | check-circle | Done in 3.2s |
| Failed | `error` | x-circle | error message |
| Canceled | `text_disabled` | ban | Canceled |
| Skipped | `warning` | skip-forward | Output exists, skipped |

Other feedback: the footer summary counts each state; a missing FFmpeg shows a
top warning banner and disables conversion; ignored unsupported files show a
status message.

## 7. Interactions

| Interaction | Implementation |
|---|---|
| Drag files/folders | `window::Event::FileDropped` → expand folders (bounded) → filter → de-dupe |
| `Ctrl+O` | Add files |
| `Ctrl+Enter` | Convert all |
| `Esc` | Close the About/History overlay |
| Hover a row | Card background → `bg_hover` |
| Click a status item | Start / Cancel / Open / Retry; ✕ removes (with tooltip) |

## 8. Theming & i18n

- Themes: dark / light / follow-system; "follow system" detection is cached.
- The theme object and design tokens are cached so rendering never rebuilds them
  per frame.
- 8 languages (see [`TECH_STACK.md`](TECH_STACK.md)); a language picker is in the
  settings row.

## 9. Accessibility

- Color contrast targets WCAG AA (`text_primary` on `bg_base` ≈ 13:1,
  `text_secondary` on `bg_surface` ≈ 5.8:1).
- Icon-only buttons carry `tooltip` text.
- Not yet done: keyboard focus rings, full UI mirroring for RTL languages.
