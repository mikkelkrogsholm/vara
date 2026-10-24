# Verification without native Mac UI — 2026-10-23

Requested scope: run the checks possible without unlocking or controlling the
native Mac session. No native application, microphone, clipboard or OS secret
store was controlled. No live cloud key or cloud inference request was used.

## Results

| Check | Result |
| --- | --- |
| `cargo test --workspace` | PASS: 30 tests (11 core/storage, 12 HTTP protocol, 7 model integrity) |
| Real Redux release runtime | PASS: 1 additional opt-in test, 1.53 s runtime |
| Production UI browser fixtures | PASS: 13 groups, zero JavaScript errors |
| `cargo clippy --workspace --all-targets` | Completed successfully; existing style warnings remain |
| `cargo fmt --all --check` | PASS |
| `git diff --check` | PASS |
| GDD visualization/model validation | PASS against the current generated graph |
| Existing Mac release bundle | Info.plist valid, arm64 Mach-O executable and model notices present |

Added three HTTP tests for concrete compatibility/failure boundaries:

- Existing legacy OpenAI model configurations send the singular language hint;
  automatic language detection omits language hints.
- Missing credentials and already-cancelled text requests never connect to the
  local test server.
- Malformed provider JSON returns a safe error without echoing response data.

Existing HTTP tests also verify GPT-Transcribe multipart language arrays,
Responses prompt preservation and typed output aggregation, no response storage,
refusal/incomplete/empty-result errors, Groq request contracts, redacted 401/429,
catalogue checks without user content and cancellation of an in-flight request.
All HTTP tests use a local server and fictional credentials.

The opt-in Redux test uses verified model files already present under
`target/redux-acceptance/models` and `Samples/da_test.wav`. It performs actual
recognition, accepts cancellation while processing a longer repeated recording,
and checks that the following session returns the same transcript as the first.
No new model downloads were needed. This checks runtime reuse/cancellation,
not transcription accuracy against a reviewed reference transcript.

The browser exercises production views/components/controller using mocked IPC.
This verifies presentation and interactions, not native host integration or
real provider permissions. It includes OpenAI/Groq key dialogs and error states,
explicit cloud choice, local readiness, app prompts, history, recovery, setup,
responsive layouts and a stalled microphone-enumeration fixture.

Only test code changed during this pass. The previously successful production
Mac release build remains applicable; no production behavior was changed.
Clippy warnings concern collapsible conditionals, default struct initialization,
needless return and an unnecessary vector allocation; no compiler errors.

## Remaining evidence requiring another environment

- Unlocked Mac: actual microphone capture, global shortcut, native HUD, permissions,
  exact focus detection and insertion/clipboard preservation in another app.
- Real provider account/key: OpenAI/Groq inference permission, quota, quality and latency.
- Windows and Linux: native builds and OS integration acceptance.

These results do not imply release acceptance or full end-to-end desktop parity.

## Subsequent live test

After the user identified legacy credentials, live Groq tests revealed and fixed
a duplicate multipart field. Both speech models and text now pass. The earlier
local-only results above should be read with this later finding:
[LIVE-CLOUD-VERIFICATION-2026-10-23.md](LIVE-CLOUD-VERIFICATION-2026-10-23.md).
