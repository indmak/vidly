# Third-party notices

Vidly is distributed under the [MIT License](LICENSE). It depends on the
following third-party software.

| Dependency | Version | License | Source |
|---|---|---|---|
| iced | 0.13 | MIT | https://github.com/iced-rs/iced |
| tokio | 1.x | MIT | https://github.com/tokio-rs/tokio |
| tokio-util | 0.7 | MIT | https://github.com/tokio-rs/tokio |
| futures | 0.3 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| **rfd** | 0.15 | **MPL-2.0** | https://github.com/PolyMeilex/rfd |
| serde / serde_json | 1.x | MIT OR Apache-2.0 | https://github.com/serde-rs |
| thiserror | 2.x | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| dirs | 6.x | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-rs |
| opener | 0.7 | MIT | https://github.com/Seeker14491/opener |
| tracing / tracing-subscriber | 0.1 / 0.3 | MIT | https://github.com/tokio-rs/tracing |
| winresource (build) | 0.1 | MIT | https://github.com/mxre/winres |
| **Noto Sans CJK SC** (embedded UI font) | 2.004 | **SIL OFL 1.1** | https://github.com/notofonts/noto-cjk |
| **FFmpeg** (independent binary, bundled) | 7.x (minimal) | **LGPL-2.1-or-later** | https://ffmpeg.org/releases/ |

> Regenerate the Rust dependency list with:
> `cargo install cargo-license && cargo license --all-features --avoid-dev-deps`.

## Fonts

The UI embeds **Noto Sans CJK SC** (`assets/fonts/NotoSansCJKsc-Regular.otf`) to
render Latin, Cyrillic, Greek, Han, Kana and Hangul deterministically across all
supported languages. It is licensed under the SIL Open Font License 1.1
(see `assets/fonts/OFL.txt`).

## FFmpeg

Vidly invokes FFmpeg as a **separate process** (it does not link libav*), so the
application itself is not covered by FFmpeg's license (mere aggregation).

All platforms bundle a **minimal, self-compiled LGPL v2.1+** build produced by
[`scripts/build-ffmpeg-min.sh`](scripts/build-ffmpeg-min.sh) (`--disable-gpl
--disable-nonfree`, no `--enable-version3`, remux-only components). The matching
`COPYING.LGPLv2.1` ships inside every package (`LICENSE-ffmpeg.txt`).

Users may replace the bundled `ffmpeg` / `ffprobe` binaries at any time (Vidly
looks next to its own executable first). FFmpeg source: https://ffmpeg.org/releases/.
