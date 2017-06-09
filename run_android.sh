#!/usr/bin/env sh
APP=abstract_art
PACKAGE=org.kinloch.colin.abstract_art
set -e
for target in "aarch64-linux-android" ; do
  CARGO_INCREMENTAL=1 cargo build --target=${target}
done;
cd ./android
./gradlew clean installArm64Debug
cd ..
adb shell am start -n ${PACKAGE}/${PACKAGE}.MainActivity
