# Support — Vidly

**Vidly** is a free, open-source desktop tool that converts between MP4 and MOV
(and MKV/WebM) **losslessly** by remuxing — it repackages the streams into a new
container without re-encoding.

- Website: https://www.indmak.com/products/vidly
- Downloads: https://github.com/indmak/vidly/releases
- Source & issues: https://github.com/indmak/vidly
- Contact: https://www.indmak.com/contact

---

## 1. System requirements

| Platform | Requirement |
|---|---|
| Windows | Windows 10 (1809) or later, 64-bit |
| macOS | macOS 11 (Big Sur) or later (Apple Silicon or Intel) |
| Linux | 64-bit distribution (Ubuntu 22.04 / Debian or similar) |

Vidly uses **FFmpeg** as its conversion engine. Depending on the build, FFmpeg is
either **bundled** with the installer or detected on your system. You do not need
to install FFmpeg yourself if the package includes it.

## 2. Download and install

All builds are published on **GitHub Releases**:
https://github.com/indmak/vidly/releases

> A **Microsoft Store** listing is coming soon. Until it is live, please use the
> GitHub Releases above.

### Windows

1. Download `Vidly-Setup-<version>.exe`.
2. Run it. If Windows SmartScreen shows a warning (the build may not yet be
   code-signed), click **More info → Run anyway**.
3. The installer installs to your user folder (no admin prompt). If you don't
   already have FFmpeg, it also installs the bundled copy and skips it if you do.
4. Launch Vidly from the Start menu.

### macOS

1. Download `Vidly-<version>.dmg`.
2. Open the DMG and drag **Vidly** onto the **Applications** shortcut.
3. The app is signed and notarized. If you get a Gatekeeper prompt, right-click
   the app → **Open** → **Open**.
4. FFmpeg is bundled inside the app.

### Linux (Debian / Ubuntu)

```bash
sudo apt install ./vidly_<version>_amd64.deb
vidly
```

FFmpeg is bundled at `/usr/lib/vidly`. To remove:

```bash
sudo apt remove vidly
```

## 3. First run

- The header shows an **FFmpeg** status badge. If it reads *FFmpeg ready*, you're
  good to go.
- If it shows *FFmpeg not found*, either install FFmpeg, or place `ffmpeg` and
  `ffprobe` **next to the Vidly executable** and restart. Vidly looks for FFmpeg
  next to its own executable first, then in your system `PATH`.

## 4. Using Vidly

1. **Add files** with the *Add files* button (`Ctrl+O`), or drag files (or whole
   folders) onto the window. Folders are expanded automatically.
2. Each item has its own **target container** dropdown (Auto / MP4 / MOV / MKV /
   WebM). Defaults: MP4 → MOV, MOV → MP4, MKV/WebM → MP4. Change it per file.
3. Choose an **output folder**, or leave it to write next to the source file.
4. Set the **overwrite policy** (auto-rename / overwrite / skip) and
   **concurrency** (how many files convert at once).
5. Optional: **MP4 faststart**, **Only first audio track**, **Keep subtitles**.
6. Press **Convert all** (`Ctrl+Enter`). Use **Cancel** to stop an item, or
   **Open** to reveal a finished file.

Other shortcuts: `Ctrl+O` add files, `Ctrl+Enter` convert all, `Esc` close the
About/History panel.

The **History** button lists recent conversions; the **Logs** button opens the log
folder (see section 6).

## 5. Troubleshooting

### "FFmpeg not found"

- Install FFmpeg, or copy `ffmpeg` / `ffprobe` next to the Vidly executable, then
  restart the app.
- On Windows, the installer bundles FFmpeg unless you already have it on `PATH`.

### A conversion fails

Remuxing copies streams as-is, so the streams must be valid for the target
container. Common cases:

- **H.264/HEVC into WebM** fails — WebM only allows VP8/VP9/AV1. Choose MP4/MOV
  for such sources.
- **Keeping subtitles into MP4** may fail — subtitles need conversion
  (`mov_text`), not a copy. Leave *Keep subtitles* off, or output MKV.
- A corrupted or unusual source can make FFmpeg exit with an error; the item shows
  the message.

### The converted file won't play in QuickTime

Some codecs (for example AV1 or VP9) can be stored in a MOV container but are not
played by Apple's QuickTime Player. The conversion is correct — it's a player
limitation, not a broken file.

### Where is the output?

Next to the source file by default, or in the folder you selected. Use the
**Open** button on a finished item to reveal it in your file manager.

### Windows shows a SmartScreen warning

Early builds may not be code-signed yet. Click **More info → Run anyway**. Signed
builds remove this prompt over time.

### macOS won't open the app

Right-click the app → **Open** → **Open** (this is the standard Gatekeeper flow
for the first launch).

### Linux: the file dialog or app won't start

Ensure `xdg-desktop-portal` is installed (Vidly uses it for native file dialogs).
On Ubuntu/Debian: `sudo apt install xdg-desktop-portal`.

### The app is slow or seems stuck

Remuxing is fast (seconds even for large files). If a very large batch is queued,
lower the **concurrency** in the settings row. Vidly keeps the interface
responsive; heavy work runs in the background.

### Language or theme

Both are in the settings row. The language follows your system by default (8
languages: English, French, German, Spanish, Russian, Japanese, Korean, Simplified
Chinese); the theme can be Dark, Light, or follow the system.

## 6. Logs and diagnostics

If you need to report a problem, the log files help a lot. Open them with the
**Logs** button, or navigate to:

| Platform | Log folder |
|---|---|
| Windows | `%APPDATA%\vidly\logs` |
| macOS | `~/Library/Application Support/vidly/logs` |
| Linux | `~/.config/vidly/logs` |

Logs are rotated daily and kept for about a week. They may contain local file
paths — review before sharing.

## 7. Reporting a bug or requesting help

Please open an issue at **https://github.com/indmak/vidly/issues** (or use the
contact page: https://www.indmak.com/contact). Include:

- Your OS and version (e.g., Windows 11 23H2, macOS 14.5, Ubuntu 22.04).
- The Vidly version (see the **About** screen).
- What you did, what you expected, and what happened.
- The input file's container/codec if a conversion failed (e.g., "MP4, H.264 +
  AAC").
- The relevant log file from section 6, if possible.

## 8. Feature requests and contributing

Vidly is open source under the **MIT license**. Feature requests, bug reports and
pull requests are welcome on GitHub:
https://github.com/indmak/vidly

## 9. Contact

- Support & contact: https://www.indmak.com/contact
- GitHub Issues: https://github.com/indmak/vidly/issues
- Downloads: https://github.com/indmak/vidly/releases

**Indmaksoft** — Flat 226, Chun Tip House, Fu Tip Estate, Tai Po, Hong Kong.
