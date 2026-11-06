# Issue 48 — automated product acceptance

The two authorized Gemini rich-file production requests passed through the real
CLI, shared backend and engine, using the saved OS Gemini credential. Their exact
IDs and transcripts also appeared through the production desktop `Files.file_list`
adapter with a real, windowless Tauri AppHandle. This is native adapter evidence;
it does not prove WebView rendering, keyboard activation or insertion into another
application. Native OS acceptance remains incomplete while the Mac is locked.

## Evidence reused before filling gaps

Unchanged core `Plan`/session/store suites cover frozen general/app-specific and
one-shot routing, independent speech/text selection, strict-local speech/text
rejection, sticky cancellation, atomic publication and original recovery expiry.
The #44 HTTP matrices cover provider timeout/refusal/empty/malformed output and
cancellation; #45 records real local runtime cancellation/reuse. #46 and the
existing browser suites cover Gemini pickers, shared-key labels, settings,
EN/DA, retained drafts/selection, playback and supplied timestamp seeking.
Existing CLI/backend tests cover request identity, cancellation, interrupted
clients, restart, sanitized exports and unavailable timing. None of this evidence
is relabeled as native OS insertion.

## New bounded coverage

- `service_acceptance.rs` runs real Service pipeline/store/audio/retry/maintenance
  paths under a real windowless Tauri host. Only recognizer/text results and OS
  insertion are scripted. Timeout, incomplete/refused output and empty-output
  errors at that boundary retain original text and explain AI fallback. Success
  displays/delivers processed text first, preserves original under the originating
  app and removes audio. Failed retry preserves the first creation/expiry;
  successful retry produces one history row and removes audio. Restart/manual
  deletion and post-ASR staged-original recovery are exercised. The expiry clock
  is injected at the exact 24-hour boundary; no real-time TTL wait. Accepted
  cancellation rejects a late successful recognizer and publishes nothing.
  An actual separate shared file owner and Service dictation run simultaneously
  under one task-owned profile (ASR scripted), both finishing successfully.
- `file_product_acceptance.py` sends public WAV, task-created MP3/MP4 and the
  public 321-second corpus file through production CLI/queue/decoding. ASR is
  explicitly injected after normalization. Source hashes stay unchanged. Real
  no-audio MP4 and invalid media fail safely. Accepted cancellation remains
  cancelled after late work; backend retry uses the same ID, creates no duplicate,
  and survives an observed owner/socket exit and restart. The existing media
  suite separately verifies bounded five-minute chunks and cancellation before
  the next chunk; the 321-second scripted-ASR run is not a long inference claim.
- `provider_file_acceptance.py` is explicitly opt-in and issues at most two
  Gemini production requests, no retry or AI/Who call. Full public transcripts,
  real timing, exports and provenance are retained in `48-gemini-file.json`.

| Public case | Duration | Real word timing | Segments | Provider speaker IDs | Production exports |
| --- | ---: | ---: | ---: | --- | --- |
| da-000 | 12.84 s | 19 words | 1 | `0:spk:0` | TXT/JSON/SRT/VTT passed |
| four-speakers | 23.215 s | 49 words | 4 | `0:spk:0` through `0:spk:3` | TXT/JSON/SRT/VTT passed |

Both returned `timing:word`, `timing_source:asr`; the production backend validates
all intervals against actual decoded duration, and the acceptance script checks
interval bounds again. Speaker labels are measured output, not a human quality
grade. Original public files retain their recorded SHA256. Matching document IDs,
exact adapter transcript equality, export bytes/content and signed execution hash
are in the JSON evidence. No timed text was fabricated or manually corrected.

## Acceptance matrix

| #48 requirement | Status and evidence |
| --- | --- |
| Independent settings, app/general/one-session routing, strict-local before network | Automated pass: existing core contracts + refreshed settings/browser suites; host pipeline consumes the frozen plan. Physical key activation not tested. |
| Hold/toggle one session, stable/changed focus, clipboard restoration, TextEdit and Codex insertion | **not_tested** for native OS path: locked Mac; no bypass, microphone recording or permission prompts. Mocked delivery only proves one host delivery call and manual-ready persistence. |
| Origin history, processed-first/original, AI failure recovery | Automated pass: actual Service/store with injected boundary results and mocked insertion; existing provider failure matrices and browser history. |
| Audio cleanup, retry/manual delete, original 24h expiry | Automated pass: actual Service/filesystem/retry/maintenance with injected boundary clock; restarted staged originals are manual-ready without replay. |
| WAV/compressed/video/>5m/invalid, queue cancel/retry/restart, simultaneous dictation | Automated pass for real import/decoding/queue/lifecycle with scripted ASR; real Gemini WAV inference separately passes. Long/compressed/video real inference remains **not_tested** in this run. |
| CLI/backend/desktop same IDs/results; cancellation/duplicate protection | Pass: two real Gemini documents match real desktop adapter; offline owner restart/retry/cancel and existing CLI/core suites. Native WebView rendering of these live jobs **not_tested**. |
| Playback/word/segment seeking, original/edit/AI text, genuine exports/fallback | Browser HTMLAudio playback, supplied timestamp seeking and text modes pass; actual live Gemini exports pass. Existing core/engine validators reject malformed/range-invalid timing; existing CLI untimed SRT rejection remains passing. Native macOS audio output **not_tested**. |
| Gemini selectable/discoverable/shared saved key/native tab | Real CLI discovery/rich inference and desktop adapter visibility pass; EN/DA picker/shared-key UI suite passes. Installed app's actual Transcription WebView **not_tested** while locked. |
| EN/DA/drafts/selection; isolated automation and bounded cloud | Refreshed suites pass. New profiles only, no private data/model/credential mutation, two authorized public requests only. Windows/Linux platform integration **not_tested**. |

The prior #47 reference-0.2.1 result (50 pass / 14 fail / 6 not_tested) remains
unchanged, including genuine Whisper Danish/silence/noise quality failures.
This delivery makes no broad production quality claim and does not close the
remaining native OS acceptance items.

## Execution and checks

Base: `4ac7860c1be038be40de102f4f4ab74d79560eb3`. No install/replacement,
push/merge, model download, personal profile mutation, secret export or ACL
change. Unrelated installed Vara processes and other projects' jobs were untouched.
Live executable was a task-owned standalone copy, signed with the existing
Developer ID identity, identifier `dk.vara.desktop`, team `3B7KHK6C9K`, hardened
runtime and the existing desktop entitlements. `codesign --verify --strict` passed;
the designated requirement was recorded. It was built from the base plus the
windowless snapshot fixture, before the later Service-only fixture additions.

Exactly **2** additional production Gemini requests were issued; no retries.
Additional reserve $0.20, prior epic reserve $0.60, combined reserve $0.80 of the
$2 cap; $1.20 unreserved. These are conservative reservations, not billing
measurements. No billing usage/invoice was retrieved. Earlier epic calls: 37
including preflight; this run brings the request count to 39.

The first broad Rust run found an inherited test-only invalid message literal,
`credential.unavailable`, in the prior acceptance-runner test. MAIN authorized
replacing it with the existing canonical `credential.read`; no new catalog or
product behavior was introduced. Targeted catalog/key tests and the full suite
were rerun after that correction.

Release environment for Rust commands:
`PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH`,
`DEVELOPER_DIR=/Library/Developer/CommandLineTools`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_RELEASE_DEBUG=0`, `CARGO_BUILD_JOBS=4`.
Browser runs use the existing Playwright module and cached Chromium via
`PLAYWRIGHT_MODULE` / `PLAYWRIGHT_CHROMIUM`, without installing dependencies.

| Command | Outcome | Log |
| --- | --- | --- |
| `cargo test --release -p vara-core origin_codes_exist_in_catalog --locked` | Pass | `/tmp/vara-epic42-48-catalog.log` |
| `cargo test --release -p vara-engine --features test-harness acceptance::tests::inaccessible_keys_do_not_abort_independent_rows --locked` | 1 test passed | `/tmp/vara-epic42-48-key-test-final.log` |
| `cargo test --release -p vara-core -p vara-engine -p vara-app --locked` | 98 passed; 1 intentionally ignored real Redux fixture | `/tmp/vara-epic42-48-rust-final.log` |
| `python3 scripts/tests/agent_cli.py target/release/vara-desktop` | Pass, real CLI/backend with explicit synthetic fixtures | `/tmp/vara-epic42-48-agent-cli.log` |
| `node desktop/tests/legacy-runner.cjs` | 23 main UI + 17 transcription checks passed | `/tmp/vara-epic42-48-browser.log` |
| `node desktop/tests/ui-main-i18n.playwright.cjs` | EN/DA settings/history/Gemini/drafts passed | `/tmp/vara-epic42-48-ui-i18n.log` |
| `node desktop/tests/transcription-i18n.playwright.cjs` | EN/DA import/player/export/drafts passed | `/tmp/vara-epic42-48-transcription-i18n.log` |
| `node desktop/tests/runtime-i18n.playwright.cjs` | Runtime messages/retained editor passed | `/tmp/vara-epic42-48-runtime-i18n.log` |
| `python3 scripts/tests/provider_file_acceptance.py /tmp/vara-epic42-48-live/vara-desktop /tmp/vara-epic42-48-live/results --allow-two-cloud-requests` | Both real rich-file requests and adapter/export checks passed | `/tmp/vara-epic42-48-live.log` |
| `cargo build --release -p vara-desktop --features test-harness --locked` | Pass | `/tmp/vara-epic42-48-build-final.log` |
| `VARA_TEST_ASR_FIXTURE=/tmp/vara-epic42-48-file-product/transcript.json VARA_TEST_ASR_DELAY_MS=2000 VARA_TEST_BACKEND_NO_SECRET=1 VARA_TEST_BACKEND_IDLE_MS=500 /tmp/vara-epic42-48-offline/vara-desktop --vara-test-service /tmp/vara-epic42-48-service-verified Samples/acceptance/audio/da-000.wav` | All Service scenarios passed; inference and insertion scripted | `/tmp/vara-epic42-48-service-verified.log` |
| `python3 scripts/tests/file_product_acceptance.py /tmp/vara-epic42-48-offline/vara-desktop /tmp/vara-epic42-48-file-verified` | 7 cases passed; actual decoding/queue, scripted ASR | `/tmp/vara-epic42-48-file-verified.log` |
| `cargo test --release -p vara-desktop --lib --locked` | 6 passed in normal feature configuration | `/tmp/vara-epic42-48-desktop-lib.log` |
| `cargo build --release -p vara-desktop --locked` | Pass; CLI version has no test-harness marker | `/tmp/vara-epic42-48-normal-build.log` |
| `python3 -m unittest discover -s scripts/tests -p test_acceptance_provider_file_preflight.py -v` | 6 offline guards passed; same 6 also passed with `python3 -O` | `/tmp/vara-epic42-48-preflight.log`, `/tmp/vara-epic42-48-preflight-optimized.log` |
| `python3 -m unittest discover -s scripts/tests -p 'test_acceptance*.py'` | 34 offline tests passed, including the 6 new guards | `/tmp/vara-epic42-48-python-ci.log` |

The first targeted key-test invocation omitted `--features test-harness` and
executed zero tests. The corrected command above executed and passed exactly one.
The 98-test broad run uses normal features; its count does not include that guard.

The reusable live script now checks **both** fixed public files before any
process, credential access or paid call: manifest upload rights, compiled/pinned
SHA256 equality, actual WAV duration and manifest duration at most 30 seconds
(one chunk/request). Offline negative tests reject changed bytes, changed pins,
revoked rights and oversized duration, including a second-file failure before
any subprocess or profile is created. The guard uses explicit exceptions so
Python optimization cannot disable it.
The new test filename matches normal CI's existing `test_acceptance*.py` discovery.
These checks were added after live execution; both already-used file hashes were revalidated offline. No cloud
request was repeated to test the guard.

Final offline harness SHA256:
`ab41326c47c086a747e032e789ec48c1b72baaa50691feb2a004da309247ca4b`.
Normal release executable SHA256:
`a9f8fd114281d8660e918a66d91bff5c6f22fa2486a3c1801f09f87ea612fde1`.
The live signed binary hash is in `48-gemini-file.json`; its unsigned pre-signing
hash was not recorded. Existing Keychain deprecation and `root_gate` warnings
remain; no warning was treated as a functional pass/failure.

All owned build/test/native/script jobs are terminal. The live, offline-media
and concurrent-Service backend socket endpoints were confirmed absent after
idle exit; no unrelated process was stopped. Task-owned `/tmp` profiles and
outputs remain for MAIN's review and authorized cleanup. Durable public
observations are checked in here.

## Native continuation after unlocking the Mac

The later [native continuation report](48-native-continuation.md) records actual
CUA launch, public-fixture local ASR/history and the remaining tool limitations.
The Mac was unlocked for that continuation. Its final 2026-11-06 update verifies
stable-focus TextEdit insertion and empty-clipboard preservation. The user
explicitly skipped the final changed-focus test. Global hotkey activation and
Codex input remain unverified; Codex access was explicitly blocked by CUA.
The original evidence above remains historical and unchanged.
