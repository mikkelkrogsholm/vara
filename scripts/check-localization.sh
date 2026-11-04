#!/bin/sh
# Repeat the localization checks with an existing Rust target and Playwright installation.
set -eu
cd "$(dirname "$0")/.."
: "${CARGO_TARGET_DIR:?Set CARGO_TARGET_DIR to an existing Cargo target directory}"
: "${PLAYWRIGHT_MODULE:?Set PLAYWRIGHT_MODULE to the installed Playwright module}"
: "${PLAYWRIGHT_CHROMIUM:?Set PLAYWRIGHT_CHROMIUM to the browser executable}"
CARGO_BIN="${CARGO_BIN:-cargo}"
"$CARGO_BIN" test -p vara-core
"$CARGO_BIN" test -p vara-core --test ui_locale --features test-extra-locale
"$CARGO_BIN" test -p vara-desktop --lib
"$CARGO_BIN" check --manifest-path desktop/src-tauri/Cargo.toml
node desktop/tests/i18n.test.cjs
node desktop/tests/runtime-messages.test.cjs
node desktop/tests/ui-locale.playwright.cjs
node desktop/tests/ui-main-i18n.playwright.cjs
node desktop/tests/transcription-i18n.playwright.cjs
node desktop/tests/runtime-i18n.playwright.cjs
node desktop/tests/legacy-runner.cjs
