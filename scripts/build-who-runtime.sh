#!/bin/sh
# Build the pinned SDK with the local lane-export patch. No model weights are bundled. Swift is a
# build-time SDK dependency; the application and integration remain Rust.
set -eu
cd "$(dirname "$0")/.."
revision=1a5efed67f44731b5eb5433aa9be3eebae05e269
patch="$PWD/scripts/patches/who-lanes.patch"
if [ ! -d target/who-sdk/.git ]; then
  mkdir -p target/who-sdk
  git -C target/who-sdk init -q
  git -C target/who-sdk remote add origin https://github.com/Desert-Ant-Labs/desert-ant-core.git
fi
if [ "$(git -C target/who-sdk rev-parse HEAD 2>/dev/null || true)" != "$revision" ]; then
  git -C target/who-sdk fetch --depth 1 origin "$revision"
  git -C target/who-sdk checkout --detach -q FETCH_HEAD
fi
if git -C target/who-sdk apply --reverse --check "$patch" 2>/dev/null; then
  : # Already patched.
else
  git -C target/who-sdk apply --check "$patch"
  git -C target/who-sdk apply "$patch"
fi
swift build --package-path target/who-sdk --scratch-path target/who-build -c release --product WhoAndroid
mkdir -p desktop/src-tauri/runtime/who
cp target/who-build/release/libWhoAndroid.dylib desktop/src-tauri/runtime/who/
# Swift's privacy bundle is a resource, not inference code.
for bundle in target/who-build/release/*.bundle; do
  [ -d "$bundle" ] && ditto "$bundle" "desktop/src-tauri/runtime/who/$(basename "$bundle")"
done
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" desktop/src-tauri/runtime/who/libWhoAndroid.dylib
fi
