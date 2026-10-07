# Vidly — Tech Stack & Implementation

> A cross-platform desktop tool that converts between **MP4 and MOV** (and MKV/WebM)
> **losslessly** by remuxing — swapping the container without re-encoding.

## Overview

MP4 and MOV are both ISO Base Media (QuickTime) containers, so converting between
them is a **container swap, not a codec change**. Vidly invokes FFmpeg's stream
copy (`-c copy`) as a subprocess, moving the audio/video bitstreams untouched:
lossless and near file-copy speed.

Vidly is a GUI "shell" around FFmpeg. It is **not** linked against libav*, so the
application's own license is unaffected by FFmpeg's.

## Tech stack

| Layer | Choice | Version | Responsibility |
|---|---|---|---|
| GUI | [`iced`](https://iced.rs/) (`tokio` feature) | 0.13 | Elm-architecture UI, subscriptions (drag & drop / shortcuts), async tasks |
| Async runtime | [`tokio`](https://tokio.rs/) | 1.x | Subprocesses, file I/O — never blocking the UI |
| Cancellation | `tokio-util` → `CancellationToken` | 0.7 | Cancel one item / all; kills the ffmpeg child |
| File dialogs | [`rfd`](https://github.com/PolyMeilex/rfd) (`xdg-portal` on Linux) | 0.15 | Add files / choose output dir |
| Conversion engine | [`ffmpeg`](https://ffmpeg.org/) / `ffprobe` (subprocess) | 7.x | `-c copy` remux + progress probing |
| Open files | [`opener`](https://github.com/Seeker14491/opener) (`reveal`) | 0.7 | Reveal output in the file manager |
| Config / history | [`serde`](https://serde.rs/) + `serde_json` + `dirs` | 1 / 1 / 6 | Output dir, overwrite policy, concurrency, faststart, theme, language, window size, history |
| Errors | [`thiserror`](https://github.com/dtolnay/thiserror) | 2 | Typed errors (cancel / fail / probe separated) |
| Logging | [`tracing`](https://github.com/tokio-rs/tracing) + `tracing-appender` | 0.1 / 0.2 | Rotating log files (`RUST_LOG=debug`) |
| System theme | [`dark-light`](https://github.com/frewsxcv/rust-dark-light) | 1 | "Follow system" theme |
| System locale | [`sys-locale`](https://github.com/1Password/sys-locale) | 0.3 | Detect UI language |
| UI font | [Noto Sans CJK SC](https://github.com/notofonts/noto-cjk) (embedded, OFL) | — | Deterministic Latin/Cyrillic/Han/Kana/Hangul rendering |
| Icons | [Lucide](https://lucide.dev/) (embedded SVG) | — | Monochrome line icons, tinted per state |

## Project structure

```
src/
├── main.rs      # entry: logging, theme/font/icon setup, window
├── app.rs       # state machine: queue, scheduling, cancel, views
├── convert.rs   # ffmpeg/ffprobe wrapper: progress, cancel, errors
├── config.rs    # settings persistence + Container/OverwritePolicy
├── history.rs   # persisted conversion history (capped)
├── i18n.rs      # 8-language tables + system-locale detection
├── theme.rs     # design tokens, light/dark themes, component styles
├── icons.rs     # embedded, cached SVG handles
└── util.rs      # extension flip, output planning, unique naming, size formatting
```

## How conversion works

1. **Probe** duration with `ffprobe` (for progress/ETA).
2. **Remux** with `ffmpeg -y -hide_banner -loglevel error -nostdin -i <in>
   -c copy -map 0:v? -map 0:a? [-movflags +faststart] <out>`.
   - Subtitles are dropped by default; "keep subtitles" adds `-map 0:s?`.
   - "First audio only" maps `0:a:0?` instead of `0:a?`.
   - `+faststart` is applied only for MP4/MOV/M4V outputs.
3. **Progress** is parsed from `-progress pipe:1` (`out_time_ms` is actually in
   microseconds — a legacy naming quirk). `stderr` is drained on a separate task
   to avoid pipe deadlocks.

The output path is finalized **at launch** according to the overwrite policy
(auto-rename / overwrite / skip). Vidly never writes over the source file.

### FFmpeg discovery order

`find_binary` looks for `ffmpeg`/`ffprobe` in:
1. next to the executable (portable/installer layout),
2. `/usr/lib/vidly` (deb layout),
3. `PATH`,
4. well-known dirs (`/opt/homebrew/bin`, `/usr/local/bin`, …) — important on
   macOS, where a GUI launched from Finder has no shell `PATH`.

Users may replace the bundled FFmpeg at any time.

## Local data

| Item | Location |
|---|---|
| Config | `<config_dir>/vidly/config.json` |
| History | `<config_dir>/vidly/history.json` (max 200 entries) |
| Logs | `<config_dir>/vidly/logs/vidly-*.log` (daily, 7 kept) |

`<config_dir>` is `%APPDATA%` (Windows), `~/.config` (Linux),
`~/Library/Application Support` (macOS). The app makes **no network requests**.

## Internationalization

8 languages: English, Français, Deutsch, Español, Русский, 日本語, 한국어, 简体中文.
Strings live in `src/i18n/*.json`, embedded at compile time. The active language
follows the system locale, falling back to English; a language picker overrides it.
Lookups return `&'static str` (zero allocation). A unit test asserts every table
has the same key set as English.

## Theming

Two palettes (dark/light) with a single accent color; "follow system" resolves via
`dark-light` (cached). The active theme and resolved tokens are cached so rendering
never rebuilds them per frame. See [`UI_DESIGN.md`](UI_DESIGN.md).

## Performance

- The queue list is **not virtualized**, so the item count is capped (2000). Rows
  are wrapped in iced `lazy`, so unchanged rows are **not rebuilt each frame**.
- File scanning, history stats, and opening paths run **off the UI thread**
  (`spawn_blocking`); i18n/theme/token lookups are allocation-free.
- ffmpeg runs with `CREATE_NO_WINDOW` on Windows (no console flash).

## Build from source

```bash
cargo build --release
cargo run
cargo test
```

Requires a stable Rust toolchain and FFmpeg on `PATH` for local runs.

## Packaging & release

Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds all three
platforms on GitHub-hosted runners and publishes to GitHub Releases.

| Platform | Artifact | Tooling |
|---|---|---|
| Windows | `Vidly-Setup-<ver>.0.exe` | Inno Setup; bundles FFmpeg, skips it if already on `PATH` |
| macOS | `Vidly-<ver>.dmg` | `.app` bundle, Developer ID sign + notarize (when secrets are set) |
| Linux | `vidly_<ver>_amd64.deb` | `dpkg-deb`, FFmpeg in `/usr/lib/vidly` |

- **Versioning**: `Cargo.toml`'s `version` (3-part semver) is the single source.
  Windows/Microsoft Store use a **4-part** version derived as `<version>.0`.
- **FFmpeg**: a minimal, self-compiled **LGPL v2.1+** build
  (`scripts/build-ffmpeg-min.sh`, `--disable-gpl`, remux-only), checksum-verified.
  Windows builds it under MSYS2/UCRT64.
- **macOS signing secrets** (optional): `MACOS_SIGN_IDENTITY`, `MACOS_P12_B64`,
  `MACOS_P12_PASSWORD`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER`,
  `APPLE_API_KEY_B64`. Without them the DMG is unsigned.
- **Windows signing**: SignPath (open-source program) is optional and wired but
  disabled until the secrets exist.

## Licensing

- Application: **MIT**.
- FFmpeg: bundled as an independent **LGPL v2.1+** binary; license text ships in
  every package.
- UI font: Noto Sans CJK SC under the **SIL OFL 1.1**.
- See [`../THIRD_PARTY.md`](../THIRD_PARTY.md) and [`../LICENSE`](../LICENSE).
