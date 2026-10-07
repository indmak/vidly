#!/usr/bin/env bash
# Builds assets/vidly.icns from assets/icon.png (run on macOS; needs sips + iconutil).
set -euo pipefail
cd "$(dirname "$0")/.."
SRC="assets/icon.png"
OUT="assets/vidly.icns"
SET="$(mktemp -d)/icon.iconset"
mkdir -p "$SET"
for s in 16 32 128 256 512; do
    sips -z "$s" "$s" "$SRC" --out "$SET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) "$SRC" --out "$SET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$SET" -o "$OUT"
echo "✅ $OUT"
