#!/usr/bin/env bash
# Usage: ./scripts/package-deb.sh <version> <arch>
# Requires: target/release/vidly and out/bin/{ffmpeg,ffprobe} already built.
set -euo pipefail

VER="${1:?usage: package-deb.sh <version> <arch>}"
ARCH="${2:-amd64}"
ROOT="package/vidly"
rm -rf "$ROOT"
mkdir -p "$ROOT"/{DEBIAN,usr/bin,usr/lib/vidly,usr/share/applications,usr/share/icons/hicolor/256x256/apps,usr/share/doc/vidly}

# ── main binary + ffmpeg ────────────────────────────────────────────────────
cp target/release/vidly          "$ROOT/usr/bin/"
cp out/bin/ffmpeg out/bin/ffprobe "$ROOT/usr/lib/vidly/"
chmod 755 "$ROOT/usr/bin/vidly" "$ROOT/usr/lib/vidly/"*

# ── control ─────────────────────────────────────────────────────────────────
# Dependencies can be auto-checked with:
#   ldd target/release/vidly | awk '/=> \//{print $3}' | xargs -n1 basename | sort -u
cat > "$ROOT/DEBIAN/control" <<EOF
Package: vidly
Version: ${VER}
Section: video
Priority: optional
Architecture: ${ARCH}
Depends: libxkbcommon0, libwayland-client0, libxkbcommon-x11-0, libx11-6, libglib2.0-0, libvulkan1, libegl1, libgl1, xdg-desktop-portal
Maintainer: indmak <indmak@users.noreply.github.com>
Homepage: https://github.com/indmak/vidly
Description: Lossless MP4/MOV container converter
 GUI tool that converts between MP4 and MOV without re-encoding
 (lossless remux), powered by FFmpeg (LGPL build).
EOF

# ── desktop entry ───────────────────────────────────────────────────────────
cat > "$ROOT/usr/share/applications/vidly.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Vidly
Comment=Lossless MP4/MOV remuxer
Exec=vidly
Icon=vidly
Categories=AudioVideo;Utility;
Terminal=false
EOF
cp assets/vidly-256.png "$ROOT/usr/share/icons/hicolor/256x256/apps/vidly.png"

# ── docs & licenses (LGPL compliance) ───────────────────────────────────────
# Prefer the license text shipped with the self-built (LGPL v2.1+) FFmpeg.
if [ -f out/bin/LICENSE-ffmpeg.txt ]; then
    cp out/bin/LICENSE-ffmpeg.txt "$ROOT/usr/share/doc/vidly/LICENSE-ffmpeg.txt"
else
    cp LICENSE-ffmpeg.txt "$ROOT/usr/share/doc/vidly/LICENSE-ffmpeg.txt"
fi
cat > "$ROOT/usr/share/doc/vidly/copyright" <<EOF
vidly ${VER}
Copyright (c) 2026 indmak
License: MIT (see https://github.com/indmak/vidly)
This package bundles FFmpeg (in /usr/lib/vidly/), licensed under the GNU LGPL
(see LICENSE-ffmpeg.txt). FFmpeg source: https://ffmpeg.org/releases/
Users may replace the bundled ffmpeg binaries at that path.
EOF

# ── package ─────────────────────────────────────────────────────────────────
mkdir -p dist
dpkg-deb --build --root-owner-group "$ROOT" "dist/vidly_${VER}_${ARCH}.deb"
echo "✅ dist/vidly_${VER}_${ARCH}.deb"
