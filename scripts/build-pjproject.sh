#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0-or-later
#
# Build the pinned pjproject (D-10) with Ferrophone's patches and config_site.h,
# and install it into third_party/prefix. pjsua-rust finds it through
# PKG_CONFIG_PATH:
#
#   scripts/build-pjproject.sh
#   export PKG_CONFIG_PATH="$PWD/third_party/prefix/lib/pkgconfig"
#
# Environment:
#   PJPROJECT_GIT   repository to clone from (default: upstream on GitHub;
#                   a local checkout path works and avoids the network)
#   PJPROJECT_REF   tag to build (default: 2.17)
#   PREFIX          install prefix (default: third_party/prefix)
#   JOBS            parallel make jobs (default: nproc)
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
git_url="${PJPROJECT_GIT:-https://github.com/pjsip/pjproject.git}"
ref="${PJPROJECT_REF:-2.17}"
prefix="${PREFIX:-$root/third_party/prefix}"
jobs="${JOBS:-$(nproc 2>/dev/null || echo 4)}"
patches="$root/third_party/pjproject/patches"
src="$root/third_party/build/pjproject-$ref"

# The stamp covers the ref, the patches, config_site.h and this script, so any
# change to them triggers a rebuild. CI caches $prefix keyed on the same hash.
stamp_hash="$(cat "$0" "$root/third_party/pjproject/config_site.h" "$patches"/*.patch \
    | { echo "$ref"; cat; } | sha256sum | cut -c1-16)"
stamp="$prefix/.ferrophone-pjproject-$stamp_hash"
if [[ -f "$stamp" ]]; then
    echo "pjproject $ref already built in $prefix ($stamp_hash)"
    exit 0
fi

rm -rf "$src"
mkdir -p "$(dirname "$src")"
git clone --quiet --depth 1 --branch "$ref" "$git_url" "$src"

for p in "$patches"/*.patch; do
    echo "applying $(basename "$p")"
    git -C "$src" apply --whitespace=nowarn "$p"
done
cp "$root/third_party/pjproject/config_site.h" "$src/pjlib/include/pj/config_site.h"

cd "$src"
# Audio only (D-09); no pjsua2 (we use the C API); AEC3 bundled (FR-20);
# Opus, G.722, G.711 and iLBC kept (FR-18, FR-19); OpenSSL for TLS (FR-23).
CFLAGS="${CFLAGS:-} -O2 -fPIC" CXXFLAGS="${CXXFLAGS:-} -O2 -fPIC" \
./configure --quiet \
    --prefix="$prefix" \
    --enable-shared=no \
    --disable-pjsua2 \
    --disable-video --disable-sdl --disable-ffmpeg --disable-v4l2 \
    --disable-openh264 --disable-vpx --disable-libyuv \
    --disable-upnp --disable-bcg729 --disable-silk --disable-gsm-codec \
    --enable-libwebrtc-aec3

make -s dep
make -s -j"$jobs"
make -s install

rm -f "$prefix"/.ferrophone-pjproject-*
touch "$stamp"
echo "pjproject $ref installed in $prefix"
echo "export PKG_CONFIG_PATH=\"$prefix/lib/pkgconfig\""
