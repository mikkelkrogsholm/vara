# macOS insertion diagnostics: bounded acceptance evidence

Date: 2026-11-14. Scope: core/store, macOS native adapter and Rust bridge,
Service orchestration, UI recovery, deterministic tests, and the final bounded
TextEdit acceptance below. No claim is made that this establishes or fixes Peter's
reported root cause or verifies compatibility with other editors. The initial
backend and browser phases did not control personal applications.

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

## Acceptance remaining after the initial backend phase

At the end of that phase, no real target-field insertion, TCC grant/revocation,
editor-specific behavior, clipboard observation, or user-facing diagnostic recovery
had been manually exercised. No release, signing, version change, push or GitHub
update was part of that phase. Windows and Linux were not run locally; their
adapter continues to return manual-ready with `platform_unsupported` and nullable
post-event capability. Later native and packaging evidence is recorded below.


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

Native computer control was unavailable to the UI worker during that phase.
Actual cross-app insertion, permission grant/revocation, keyboard event delivery,
and editor-specific behavior therefore remained separate native acceptance at that
point. No backend, version, release, signing, workflow, push or publication change
was part of that UI delivery. The subsequent bounded native run is recorded below.


MAIN visual review requested one bounded follow-up: put the primary Copy action
before secondary diagnostic details and use the existing compact original-text
disclosure presentation instead of Settings disclosure spacing. Technical details
now follow Copy. The original-text toggle/focus handler explicitly excludes this
secondary disclosure so opening diagnostics cannot relabel or open the original
transcript. No styles or backend behavior changed. The focused browser regression
asserts ordering, compact presentation and independent disclosure behavior; its
existing screenshot paths above were refreshed for the corrected layout.

Missing/partial access recovery names the concrete Accessibility pane in both
languages and suggests quitting and reopening Vara if its permission is already
enabled. This is troubleshooting guidance, not a promise to fix unsupported fields
or changed focus; no permission toggles, reset or retry are automatic.


## Final bounded native acceptance and release preparation

The tested runtime code is frozen at
`303455926cca3a20dfa64bd5315dae345f8ae6d2`; the subsequent report/date commit
changes documentation only. MAIN exercised the current-source, Developer ID signed
Tauri test-harness app through CUA with the real UI assets. Its executable SHA256 is
`202a4fa98f5775a1229953cca643187c5e09cac80c3dbd7ca52f367f81218774`.
This test app is separate from the normal production artifacts and must not be
published.

The existing guarded GUI fixture used a 15-second start delay and six-second
recording interval. Recognition used real local Parakeet Redux on the public
`Samples/acceptance/audio/da-000.wav` fixture (12.84 seconds of audio), not the
microphone. The fresh `/tmp/vara-native-acceptance-20261114/profile` used strict
local settings, no AI, no app rules and no cloud requests. Its models symlink reused
the existing models without downloading or copying them; all four Parakeet hashes
matched the source manifest. No personal history, recordings or credentials were
copied into the profile.

The first CUA background-focus attempt was rejected by the fixture guard, showing
a generic unknown message and producing no result. For the second trial, after an
asynchronous question, the user physically foregrounded TextEdit. CUA observed an
initially empty TextEdit document receive the public transcript once, and observed
the same transcript in Vara history. Success is not attributed to CUA coordinate
clicking alone. The native receipt at
`/tmp/vara-native-acceptance-20261114/native-receipt.json` records:

- Result `e5ff119f-7c43-4e96-8826-d3829a2da27d`, origin `com.apple.TextEdit`.
- Delivery `inserted`; diagnostic reason `confirmed`, method `accessibility`,
  `os_error: null`; no serialized `target_token`.
- Zero recovery records, staging records and retained audio files after completion.

The live native Settings snapshot reported Microphone, Input Monitoring,
Accessibility and keyboard access granted. This observes existing permission state;
it does not test granting or revoking permissions. MAIN quit the test app through
Vara → Quit in CUA and verified zero remaining test-app processes.

This run verifies one stable empty TextEdit field using direct Accessibility
insertion. It does not verify focus-change safety in an actual native trial,
preservation of a nonempty clipboard, keyboard-event OS delivery, permission
recovery, an upgrade, or the requested four-app editor coverage. Word, Outlook and
Claude were not installed; Mail was not exercised. These limits remain even though
the deterministic and browser suites cover their corresponding policy and UI
branches.

Normal production 1.1.0 artifacts from the same runtime revision were Developer ID
signed, strictly verified and checked to exclude test-harness markers. App
notarization initially failed while the Mac was locked. After unlock, the same
existing `vara-notary` profile was accessible without credential changes; Apple
accepted submission `cf9837e7-b1ad-497f-a89e-c655388ee1a2`. MAIN stapled and validated
the extracted normal app, and Gatekeeper accepted it as Notarized Developer ID.
Apple also accepted DMG submission `14f47204-ba0f-416a-a85a-a1e6e513585f`.
DMG stapling was initially pending; at 22:17:23 UTC MAIN completed `xcrun stapler
staple` and `validate`, and Gatekeeper assessed the DMG as accepted with source
Notarized Developer ID (`spctl --assess --type open --context
context:primary-signature --verbose=2`). Logs:
`/tmp/vara-insertion-notary-unlocked.json` and
`/tmp/vara-insertion-dmg-notary.json`. Normal artifacts are under
`~/.local/share/vara-release/artifacts/1.1.0-insertion-review`.

MAIN previously verified 67 normal release workspace library tests. Native
Windows/Linux build run `43620194390` passed all jobs; MAIN verified artifact
checksums, source revision and packaged CLI version. Those packages remain
experimental previews: no native GUI installation, recording, upgrade or desktop
integration acceptance is claimed. No release had been published when this report
was updated; MAIN retains final integration and publication review.
