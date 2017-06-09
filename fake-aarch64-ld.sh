#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# android abis:
#  armeabi     => arch-arm    => arm-linux-androideabi   => android-9
#  armeabi-v7a => arch-arm?   => armv7-linux-androideabi => android-9
#  arm64-v8a   => arch-arm64  => aarch64-linux-android   => android-21
#  x86         => arch-x86    => i686-linux-android      => android-9
#  x86_64      => arch-x86_64 => x86_64-linux-android    => android-21
#  mips        => arch-mips   => _                       => android-9
#  mips64      => arch-mips64 => _                       => android-21

set -o errexit
set -o nounset
set -o pipefail

TARGET_DIR="${OUT_DIR}/../../../libs/arm64-v8a"
mkdir -p $TARGET_DIR

export _ANDROID_ARCH=arch-arm64
export ANDROID_SYSROOT="${ANDROID_NDK}/platforms/android-21/${_ANDROID_ARCH}"
export _ANDROID_EABI=aarch64-linux-android-4.9
ANDROID_TOOLCHAIN=""
for host in "linux-x86_64" "linux-x86" "darwin-x86_64" "darwin-x86"; do
  if [[ -d "${ANDROID_NDK}/toolchains/${_ANDROID_EABI}/prebuilt/${host}/bin" ]]; then
    ANDROID_TOOLCHAIN="${ANDROID_NDK}/toolchains/${_ANDROID_EABI}/prebuilt/${host}/bin"
    break
  fi
done

ANDROID_CPU_ARCH_DIR=arm64-v8a
ANDROID_CXX_LIBS="${ANDROID_NDK}/sources/cxx-stl/llvm-libc++/libs/${ANDROID_CPU_ARCH_DIR}"

echo "toolchain: ${ANDROID_TOOLCHAIN}"
echo "libs dir: ${ANDROID_CXX_LIBS}"
echo "sysroot: ${ANDROID_SYSROOT}"

"${ANDROID_TOOLCHAIN}/aarch64-linux-android-gcc" \
  --sysroot="${ANDROID_SYSROOT}" -L "${ANDROID_CXX_LIBS}" "${@}" -lc++ \
  -o "${TARGET_DIR}/libabstract_art.so" -shared && touch "${TARGET_DIR}/abstract_art"
