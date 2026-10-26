#!/bin/sh
# Build the unmodified official SDK. No model weights are bundled. Swift is a
# build-time SDK dependency; the application and integration remain Rust.
set -eu
cd "$(dirname "$0")/.."
revision=86b9b55fe8c61ec1c27c5b6383f545c6df1d6ceb
if [ ! -d target/who-sdk/.git ]; then
  mkdir -p target/who-sdk
  git -C target/who-sdk init -q
  git -C target/who-sdk remote add origin https://github.com/Desert-Ant-Labs/desert-ant-core.git
  git -C target/who-sdk fetch --depth 1 origin "$revision"
  git -C target/who-sdk checkout --detach -q FETCH_HEAD
fi
[ "$(git -C target/who-sdk rev-parse HEAD)" = "$revision" ]
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
