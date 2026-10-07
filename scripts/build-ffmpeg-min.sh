#!/usr/bin/env bash
# Builds a minimal, LGPL-only ffmpeg + ffprobe — just enough for lossless remux.
# Works on Linux/macOS and in MSYS2 (MINGW64) on Windows.
# Output: out/bin/ffmpeg[.exe], out/bin/ffprobe[.exe], out/bin/LICENSE-ffmpeg.txt
set -euo pipefail

VER="${FFMPEG_VER:-7.1}"
PREFIX="$(pwd)/out"
JOBS="${JOBS:-$( (command -v nproc >/dev/null && nproc) || (command -v getconf >/dev/null && getconf _NPROCESSORS_ONLN) || echo 4 )}"

mkdir -p out/bin build
cd build

if [ ! -d "ffmpeg-${VER}" ]; then
    curl -fL -o "ffmpeg-${VER}.tar.xz" "https://ffmpeg.org/releases/ffmpeg-${VER}.tar.xz"
    tar xf "ffmpeg-${VER}.tar.xz"
fi

cd "ffmpeg-${VER}"

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
    --enable-parser=aac,ac3,mpegaudio,h264,hevc,vp8,vp9,av1,opus,vorbis

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
