#!/usr/bin/env bash
# Usage: ./scripts/package-macos.sh <version>
# Requires: target/release/vidly and out/bin/{ffmpeg,ffprobe} already built.
set -euo pipefail

VER="${1:?usage: package-macos.sh <version>}"
# Optional: without an identity we still build an (unsigned) DMG, like Windows.
SIGN_IDENTITY="${MACOS_SIGN_IDENTITY:-}"
APP="dist/Vidly.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" dist
cp target/release/vidly out/bin/ffmpeg out/bin/ffprobe "$APP/Contents/MacOS/"
cp assets/vidly.icns "$APP/Contents/Resources/"
# LGPL: ship the license text matching this self-built (LGPL v2.1+) FFmpeg.
if [ -f out/bin/LICENSE-ffmpeg.txt ]; then
    cp out/bin/LICENSE-ffmpeg.txt "$APP/Contents/Resources/"
else
    cp LICENSE-ffmpeg.txt "$APP/Contents/Resources/"
fi

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Vidly</string>
  <key>CFBundleDisplayName</key><string>Vidly</string>
  <key>CFBundleIdentifier</key><string>com.indmak.vidly</string>
  <key>CFBundleExecutable</key><string>vidly</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleIconFile</key><string>vidly</string>
  <key>CFBundleShortVersionString</key><string>${VER}</string>
  <key>CFBundleVersion</key><string>${VER}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

# ── 1. Sign (order matters: nested binaries first, then the .app as a whole) ──
if [[ -n "$SIGN_IDENTITY" ]]; then
    for bin in ffmpeg ffprobe; do
        codesign --force --timestamp --options runtime \
            --sign "$SIGN_IDENTITY" "$APP/Contents/MacOS/$bin"
    done
    codesign --force --timestamp --options runtime \
        --sign "$SIGN_IDENTITY" "$APP"
    codesign --verify --deep --strict "$APP"
else
    echo "⚠️ 未提供 MACOS_SIGN_IDENTITY，跳过签名（产物未签名，用户需右键打开）"
fi

# ── 2. DMG (with an /Applications shortcut for drag-to-install) ─────────────
STAGE="dist/dmg-stage"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "Vidly" -srcfolder "$STAGE" -ov -format UDZO \
    "dist/Vidly-${VER}.dmg"
rm -rf "$STAGE"

# ── 3. Notarize + staple (p8 is prepared by the caller in CI) ───────────────
if [[ -n "${APPLE_API_KEY_ID:-}" && -n "${APPLE_API_KEY_B64:-}" ]]; then
    KEY_PATH="$HOME/private_keys/AuthKey_${APPLE_API_KEY_ID}.p8"
    mkdir -p "$HOME/private_keys"
    echo -n "$APPLE_API_KEY_B64" | base64 --decode > "$KEY_PATH"
    xcrun notarytool submit "dist/Vidly-${VER}.dmg" \
        --key "$KEY_PATH" \
        --key-id "$APPLE_API_KEY_ID" \
        --issuer "$APPLE_API_ISSUER" \
        --wait
    xcrun stapler staple "dist/Vidly-${VER}.dmg"
    xcrun stapler validate "dist/Vidly-${VER}.dmg"
fi

echo "✅ dist/Vidly-${VER}.dmg"
