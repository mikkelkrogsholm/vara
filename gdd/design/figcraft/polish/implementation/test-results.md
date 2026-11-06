# Verification results — 2026-11-06

All final verification commands exited successfully. No dependency install or
browser download was performed. `regressions.log` retains the suite summaries.

| Check | Result |
| --- | --- |
| Production WebKit captures vs approved browser references | 12/12 pixel-identical, 0 differing pixels |
| Reused geometry/focus/responsive/stress checks | 38/38 pass |
| Existing UI regression | 23 checks pass |
| Existing transcription regression | 17 checks pass |
| Updater | DA/EN notices, state retention, preference and install guards pass |
| Main UI i18n | Both locales, history, app rule, settings, dialogs, drafts pass |
| Transcription i18n | Import, jobs, editor/player, AI, Who, speaker provenance, export pass |
| UI locale | Save, dirty settings, selection, retained bindings, HTML safety pass |
| Runtime i18n | Catalog, reports, causes, live locale, editor, Who, startup pass |
| i18n and runtime message unit checks | 2 test files pass; 453 runtime codes checked |
| Git whitespace validation | `git diff --check` passes |

Production source was packaged for self-serving regression fixtures with the
existing `python3 scripts/sync-desktop-design.py`; generated copies remain ignored.
The preserved source server was owned by this verification task and stopped after
WebKit captures. The legacy runner then owned/closed its own server on port18770.
All verification/browser jobs are terminal. The native packaging job is managed
separately by MAIN.

Initial Chromium attempts failed before executing tests because Playwright's old
default cache1208 was absent. Existing cached Chromium1248 was explicitly selected
for the successful runs; this is an environment selection, not a test alteration.

From repository root, use the installed environment:

```sh
export PLAYWRIGHT_MODULE=/Users/mikkelfreltoftkrogsholm/.bun/install/global/node_modules/playwright
export PLAYWRIGHT_CHROMIUM=/Users/mikkelfreltoftkrogsholm/Library/Caches/ms-playwright/chromium_headless_shell-1248/chrome-headless-shell-mac-arm64/chrome-headless-shell
python3 scripts/sync-desktop-design.py
VARA_TEST_OUTPUT_ROOT=/tmp/vara-polish-legacy node desktop/tests/legacy-runner.cjs
node desktop/tests/updater.playwright.cjs
VARA_I18N_SHOTS=/tmp/vara-polish-i18n node desktop/tests/ui-main-i18n.playwright.cjs
VARA_I18N_SHOTS=/tmp/vara-polish-tx-i18n node desktop/tests/transcription-i18n.playwright.cjs
node desktop/tests/ui-locale.playwright.cjs
VARA_I18N_SHOTS=/tmp/vara-polish-runtime node desktop/tests/runtime-i18n.playwright.cjs
node --test desktop/tests/i18n.test.cjs desktop/tests/runtime-messages.test.cjs
git diff --check
```

WebKit capture/geometry reproduction commands are in `README.md`. Existing UI
suites use sanitized in-memory host fixtures, never a live profile or model. This
bounded verification does not establish full accessibility compliance, inference
quality, native OS keyboard/permissions or native-app screenshot equality.
