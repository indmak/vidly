#!/usr/bin/env bash
# Builds a minimal, LGPL-only ffmpeg + ffprobe — just enough for lossless remux.
# Works on Linux/macOS and in MSYS2 (MINGW64) on Windows.
# Output: out/bin/ffmpeg[.exe], out/bin/ffprobe[.exe], out/bin/LICENSE-ffmpeg.txt
set -euo pipefail

VER="${FFMPEG_VER:-7.1}"
# SHA-256 of ffmpeg-${VER}.tar.xz. If you override FFMPEG_VER, also set FFMPEG_SHA256.
SHA256="${FFMPEG_SHA256:-40973d44970dbc83ef302b0609f2e74982be2d85916dd2ee7472d30678a7abe6}"
PREFIX="$(pwd)/out"
JOBS="${JOBS:-$( (command -v nproc >/dev/null && nproc) || (command -v getconf >/dev/null && getconf _NPROCESSORS_ONLN) || echo 4 )}"

verify_sha256() {
    local file="$1" expected="$2" actual
    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$file" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        echo "❌ no sha256 tool (sha256sum/shasum) available" >&2
        return 1
    fi
    [ "$actual" = "$expected" ]
}

mkdir -p out/bin build
cd build

if [ ! -d "ffmpeg-${VER}" ]; then
    curl -fL -o "ffmpeg-${VER}.tar.xz" "https://ffmpeg.org/releases/ffmpeg-${VER}.tar.xz"
    if ! verify_sha256 "ffmpeg-${VER}.tar.xz" "$SHA256"; then
        echo "❌ checksum mismatch for ffmpeg-${VER}.tar.xz (expected $SHA256)" >&2
        exit 1
    fi
    tar xf "ffmpeg-${VER}.tar.xz"
fi

cd "ffmpeg-${VER}"

# On Windows (MSYS2/MinGW) statically link the MinGW runtime, otherwise the
# binaries need libwinpthread-1.dll / libgcc_s on the target machine.
CONFIGURE_EXTRA=()
case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) CONFIGURE_EXTRA+=(--extra-ldexeflags=-static) ;;
esac

# Remux only: container demuxers/muxers + the bitstream filters ffmpeg may need
# when copying streams between mp4/mov/mkv/webm. No encoders, no network, no GPL.
./configure \
    --prefix="$PREFIX" \
    --disable-gpl \
    --disable-nonfree \
    --disable-doc \
    --disable-debug \
    --disable-network \
    --disable-autodetect \
    --disable-everything \
    --enable-ffmpeg \
    --enable-ffprobe \
    --enable-protocol=file,pipe \
    --enable-demuxer=mov,matroska,mpegts,avi \
    --enable-muxer=mov,mp4,matroska,webm,ipod \
    --enable-bsf=null,extract_extradata,aac_adtstoasc,h264_mp4toannexb,hevc_mp4toannexb \
    --enable-parser=aac,ac3,mpegaudio,h264,hevc,vp8,vp9,av1,opus,vorbis \
    ${CONFIGURE_EXTRA[@]+"${CONFIGURE_EXTRA[@]}"}

make -j"$JOBS"

if [ -f ffmpeg.exe ]; then
    cp ffmpeg.exe ffprobe.exe "$PREFIX/bin/"
    FFMPEG_BIN="$PREFIX/bin/ffmpeg.exe"
else
    cp ffmpeg ffprobe "$PREFIX/bin/"
    FFMPEG_BIN="$PREFIX/bin/ffmpeg"
fi
# This build is LGPL v2.1+ (no --enable-version3): ship the matching license text.
cp COPYING.LGPLv2.1 "$PREFIX/bin/LICENSE-ffmpeg.txt"

"$FFMPEG_BIN" -version | head -1
echo "✅ ffmpeg + ffprobe built into $PREFIX/bin ($(du -h "$FFMPEG_BIN" | cut -f1) for ffmpeg)"
