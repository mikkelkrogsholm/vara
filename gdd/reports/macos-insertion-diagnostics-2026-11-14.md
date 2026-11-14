# macOS insertion diagnostics: bounded backend evidence

Date: 2026-11-14. Scope: core/store, macOS native adapter and Rust bridge,
Service orchestration, deterministic tests. UI recovery presentation is separate.
No claim is made about the cause on Peter's computer or compatibility with a
specific editor; no personal application was controlled for this delivery.

The prior backend collapsed capture, focus, permission and unsupported-target
refusals into a missing token or `manual_ready`. This delivery retains a closed
reason, insertion method and numeric AX error where available. It preserves the
original transcript and existing clipboard-free insertion behavior. Runtime target
handles are omitted from newly serialized durable state. Existing history loads
without new metadata. The full contract is in [Rust bindings](../RUST-BINDINGS.md#macos-insertion-diagnostics-2026-11-14).

## Automated evidence

Environment for Cargo commands: `PATH="$HOME/.cargo/bin:$PATH"`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_RELEASE_DEBUG=0`, `CARGO_BUILD_JOBS=4`.

- `cargo test -p vara-core -p vara-platform -p vara-app`: 53 tests passed.
  Log: `/tmp/vara-insertion-core-platform.log`.
- `cargo build --release -p vara-desktop --features test-harness`: native Objective-C
  and Rust FFI build passed. Log: `/tmp/vara-insertion-desktop-build.log`.
- `target/release/vara-desktop --vara-test-service NEW_ISOLATED_PROFILE Samples/acceptance/audio/da-000.wav`:
  passed with `VARA_TEST_BACKEND_NO_SECRET=1`, `VARA_TEST_BACKEND_IDLE_MS=500`,
  `VARA_TEST_ASR_FIXTURE=TASK_OWNED_TRANSCRIPT_JSON`, `VARA_TEST_ASR_DELAY_MS=600`.
  Log: `/tmp/vara-insertion-service-final.log`. Real Service/store/audio lifecycle;
  inference and OS insertion explicitly mocked. Checks the persisted Unknown marker
  at dispatch, metadata, original/processed text, repeat completion without replay,
  existing no-speech, fallback, recovery, cancellation and concurrent-file behavior.
- `cargo check --release -p vara-desktop`: production configuration passed.
  Log: `/tmp/vara-insertion-desktop-default-check.log`.
- `git diff --check`: passed.

Core tests verify missing metadata and old token deserialization, omission of tokens
from new serialization, numeric diagnostics after restart, idempotent commit, and an
injected SQLite final-update failure leaving durable Unknown intact. A missing
result or a zero-row UPDATE (injected RAISE(IGNORE) trigger) cannot authorize a pre-dispatch write.

The native harness compiles and executes the same pure C policy header used by
`macos.m`: Accessibility denial, missing/dead target, app switch, unavailable field,
focus switch, keyboard role rejection, post-event denial, event allocation failure,
and before/after-send interruption status. This validates deterministic branch
policy, not real Accessibility APIs, permission grants, CG event delivery or editor
behavior. Native wire tests also verify closed diagnostic values and token handling.

The first Service-fixture invocation omitted its scripted ASR delay and failed the
existing concurrent-file "must still be in flight" assertion; this is retained in
`/tmp/vara-insertion-service.log`. Supplying the fixture's documented 600ms delay
made the bounded acceptance pass. No product workaround was added for that setup
mistake. Existing Security-framework deprecation warnings and a desktop dead-code
warning remain in build output.

## Remaining acceptance

No real target-field insertion, TCC grant/revocation, editor-specific behavior,
clipboard observation, or user-facing diagnostic recovery was manually exercised.
No release, signing, version change, push or GitHub update was performed. Windows
and Linux were not run locally; their adapter continues to return manual-ready
with `platform_unsupported` and nullable post-event capability.


## Agreed UI delivery and evidence

The bounded UI plan preserves the current reading layout, history selection, Copy,
original text, microphone and global shortcut flows. One shared interpreter uses
`delivery` as the primary status in history and detail; diagnostic reasons explain
manual-ready results without downgrading Unknown. Unknown warns that text may
already have been sent and asks the user to check the target before pasting again.
Inserted results receive no error notice. Pending and historical records without
metadata remain readable. Unrecognized diagnostics use the generic manual-copy
explanation. There is no automatic retry or clipboard fallback.

Manual-ready explanations cover missing Accessibility, failed capture, vanished
target, unavailable field, changed app/focus, unsupported field, denied keyboard
insertion, failed dispatch, no attempt, and unsupported platform. Historical access
refusals describe that attempt; Settings and onboarding use live permission values.
Only permission-related reasons offer the existing explicit permission action.
Collapsed secondary details contain only whitelisted reason/method codes and
signed 32-bit numeric AX errors. Arbitrary diagnostic properties never enter the DOM.
No support export or new dependency was introduced.

The shared Settings/onboarding insertion row distinguishes missing Accessibility
from direct Accessibility insertion being available while keyboard insertion is
blocked. The latter offers `permission {kind: "post_event"}`; missing Accessibility
offers `permission {kind: "accessibility"}`. Both are invoked only on click. Null or
missing post-event data is explained as unavailable information from this version,
not denied access. Unsupported platforms retain manual insertion.

Browser verification uses a local HTTP fixture server, an in-memory IPC host,
headless Chrome, and isolated browser contexts. It does not use personal profiles,
credentials, cloud requests, real target apps, or OS permission controls. This is UI
and IPC contract evidence, not proof of native insertion or TCC recovery.

Commands run from the repository root:

```sh
node desktop/tests/i18n.test.cjs
node desktop/tests/runtime-messages.test.cjs
```

Browser commands use:

```sh
export PLAYWRIGHT_MODULE=/opt/homebrew/lib/node_modules/@playwright/cli/node_modules/playwright
export PLAYWRIGHT_CHROMIUM='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
node desktop/tests/insertion-diagnostics.playwright.cjs
node desktop/tests/runtime-i18n.playwright.cjs
node desktop/tests/ui-main-i18n.playwright.cjs
node desktop/tests/legacy-runner.cjs
```

Results: catalog parity and 453 runtime codes passed; the focused insertion suite
passed in both languages; runtime localization and main UI localization passed;
legacy UI passed 23 checks and transcription passed 17 checks. Logs:
`/tmp/vara-insertion-i18n-unit.log`, `/tmp/vara-insertion-runtime-unit.log`,
`/tmp/vara-insertion-ui.log`, `/tmp/vara-insertion-runtime-i18n.log`,
`/tmp/vara-insertion-main-i18n.log`, `/tmp/vara-insertion-legacy-ui.log`.

The focused suite exercises every closed reason family, Unknown combined with
changed focus or revoked access, inserted/pending states, old history without
metadata, malformed/future diagnostic fields, numeric-detail whitelisting, live
partial permissions, old-host missing/null post-event data, explicit correct
permission calls, no retry, retained text selection and row selection while copying,
and the onboarding Grant access path. Both locale catalogs resolve all new labels.

Screenshots for MAIN visual review (1440 × 950):

- `/tmp/vara-insertion-ui-screens/en-manual.png`
- `/tmp/vara-insertion-ui-screens/da-manual.png`
- `/tmp/vara-insertion-ui-screens/en-unknown.png`
- `/tmp/vara-insertion-ui-screens/da-unknown.png`
- `/tmp/vara-insertion-ui-screens/en-partial-permissions.png`
- `/tmp/vara-insertion-ui-screens/da-partial-permissions.png`

Native computer control was unavailable for this worker. Actual cross-app insertion,
permission grant/revocation, keyboard event delivery, and editor-specific behavior
remain separate native acceptance. No backend, version, release, signing, workflow,
push or publication change is part of this UI delivery.
