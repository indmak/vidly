# Privacy Policy — Vidly

**Last updated:** October 12, 2026
**Applies to:** the Vidly desktop application for Windows, macOS and Linux
(including the Microsoft Store distribution) and the page
`https://www.indmak.com/products/vidly`.

**Publisher:** Indmaksoft
**Contact:** https://www.indmak.com/contact

---

## 1. Summary

Vidly is a small, offline desktop tool that converts between MP4 and MOV (and
MKV/WebM) by **remuxing** — copying the audio and video streams into a new
container without re-encoding.

**Vidly does not collect, transmit, sell or share your personal data.** It has no
analytics, no telemetry, no advertising, no accounts and no sign-in. It makes **no
network requests** while running. Everything happens locally on your own computer.

## 2. Information we collect

**We collect no personal information.** We do not operate servers that receive
your data, and the application does not send anything anywhere.

## 3. What the application accesses on your device

Vidly only touches what it needs to do its job, and only on your machine:

- **Files and folders you choose.** When you add files (via the file picker or by
  dragging them onto the window), Vidly reads those video files and writes the
  converted output to the location you select (or next to the source file). It
  does not scan your disk or access any file you did not explicitly add.
- **A local FFmpeg process.** Conversion is performed by FFmpeg running as a
  **local subprocess** on your computer. No media is uploaded; FFmpeg reads the
  local input file and writes the local output file.
- **Local settings, history and logs.** Vidly stores a few small files in your
  operating system's per-user configuration directory (see section 4). These
  files stay on your device.

Vidly does not request access to your camera, microphone, contacts, location,
calendar, or any other personal data, and it does not read your clipboard.

## 4. Data stored locally on your device

Vidly keeps its own data under your user configuration directory:

| Platform | Location |
|---|---|
| Windows | `%APPDATA%\vidly` |
| macOS | `~/Library/Application Support/vidly` |
| Linux | `~/.config/vidly` |

This directory may contain:

- `config.json` — your preferences (output folder, overwrite policy,
  concurrency, faststart, theme, language, window size).
- `history.json` — the recent conversion history (up to 200 entries). Each entry
  records the **local file paths** of the input and output, the result status,
  duration and file size, and a timestamp.
- `logs\` — rotating diagnostic log files (kept for a few days). Logs may include
  local file paths and technical details about the FFmpeg run.

**This data never leaves your computer.** It is not sent to us or anyone else.
You can delete it at any time by removing the folder above (or by uninstalling
the app).

## 5. Network activity

The Vidly application makes **no outbound network connections**. It does not
phone home, check for updates, fetch ads, or contact any server.

The only network-related actions are ones **you** trigger by clicking a link (for
example, "Project · GitHub" on the About screen or the website link). In that
case your default web browser opens the URL — the request is made by your browser,
not by Vidly.

## 6. Third-party components

Vidly is built with open-source software and bundles some of it. None of these
components collect or transmit data on our behalf:

- **FFmpeg** — the conversion engine, shipped as a separate LGPL binary and
  invoked as a local subprocess. FFmpeg does not send your media anywhere.
- **Noto Sans CJK SC** — an embedded UI font (SIL OFL 1.1).
- **Lucide** — embedded icon set (ISC License).

For a full list, see `THIRD_PARTY.md` in the project repository:
https://github.com/indmak/vidly

## 7. Microsoft Store

If you install Vidly from the **Microsoft Store**, Microsoft may collect
information related to the Store itself (for example, download, install and crash
telemetry) under **Microsoft's** privacy statement. That data is collected by
Microsoft, not by Vidly, and is governed by Microsoft's policy. The Vidly
application still collects nothing.

## 8. Data retention and deletion

We retain nothing, because we receive nothing. Data created by the app stays on
your device until **you** delete it:

- Remove the folder listed in section 4, or
- Uninstall the application (on Windows via Settings → Apps; on macOS by moving
  the app to the Trash; on Linux via `sudo apt remove vidly`).

## 9. Children's privacy

Vidly is a general-purpose utility that is not directed to children under 13.
Because the application collects no personal information, we do not knowingly
collect any data from children.

## 10. Security

Vidly processes your files locally and does not transmit them. As with any
software, keep your operating system and the app up to date. You can always
verify the app's behavior in its source code, which is public under the MIT
license: https://github.com/indmak/vidly

## 11. Changes to this policy

If we change this policy, we will update the "Last updated" date above and, for
material changes, note them on the product page. Because the app collects no data,
we do not expect this policy to change in substance.

## 12. Contact

Questions about this policy or about privacy in Vidly?

- Contact page: https://www.indmak.com/contact
- GitHub Issues: https://github.com/indmak/vidly/issues

**Indmaksoft** — Flat 226, Chun Tip House, Fu Tip Estate, Tai Po, Hong Kong.
