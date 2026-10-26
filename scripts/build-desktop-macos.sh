#!/bin/sh
# Stable bundle/signing identity prevents each development rebuild from invalidating TCC access.
set -eu
cd "$(dirname "$0")/.."
if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
  APPLE_SIGNING_IDENTITY=$(security find-identity -v -p codesigning | sed -n 's/.*"\(Developer ID Application:.*\)"/\1/p' | head -1)
fi
if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
  echo 'Set APPLE_SIGNING_IDENTITY to a trusted Apple signing identity before building.' >&2
  exit 1
fi
export APPLE_SIGNING_IDENTITY
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_RELEASE_DEBUG=0
sh scripts/build-who-runtime.sh
cargo tauri build --bundles app
codesign --verify --deep --strict target/release/bundle/macos/Vara.app

# Hardened runtime also needs the audio-input entitlement, independently of TCC consent.
python3 - <<'CHECK'
import plistlib, subprocess
app = 'target/release/bundle/macos/Vara.app'
result = subprocess.run(['codesign', '-d', '--entitlements', ':-', app], capture_output=True, check=True)
assert plistlib.loads(result.stdout).get('com.apple.security.device.audio-input') is True, 'Signed app lacks microphone entitlement'
CHECK
