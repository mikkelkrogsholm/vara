# Issue 45 — bounded Whisper no-speech repair

2026-11-07, Apple M3 Max, release CPU. **Pure silence/noise rejection is repaired;
Whisper mixed-window quality remains unsatisfied and issue 45 remains open.**
The original frozen runtime reports are unchanged.

The final change vendors published whisper-rs-sys 0.15.0, retains whisper-rs
0.16.0, and applies upstream SOT-logits fix
[`65dacc8082ac4c7c3c48e998bbcb9dbbc6751148`](https://github.com/ggml-org/whisper.cpp/commit/65dacc8082ac4c7c3c48e998bbcb9dbbc6751148).
The pinned decoder previously calculated no-speech probability from an unwritten
prompt logits row. Its reported silence probability was about 0.00000013;
the correctly decoded SOT probability is 0.931494. Noise changes from about
0.00000097 to 0.884881. The application's existing independent 0.85 segment
filter now receives the actual model signal. No threshold, amplitude cutoff,
phrase filter, model, provider, native fallback or sampling setting was changed.

The other final patch repairs the vendored native build script: watch native
sources and refresh changed files in its existing OUT_DIR source copy before
CMake runs, retaining unchanged-file timestamps and compiler intermediates.
Upstream watched only wrapper.h and copied native sources only if its destination
did not already exist; this could silently compile an old local patch. An
incremental harmless-comment edit and restoration both refreshed the copied
source and recompiled its C++ object. Copied-source hashes match the repository.
Source archive SHA256, every vendored file hash, both patch hashes and preserved
license notices are documented in [vendor provenance](../../../vendor/README.md).
The crate archive SHA256 is
`6986c0fe081241d391f09b9a071fbcbb59720c3563628c3c829057cf69f2a56f`.

## Acceptance boundary

The focused real engine test checks explicit Danish and default automatic
language selection. Silence and deterministic noise return `engine.no_speech`
for direct dictation and `speech.file_empty` for aggregate file transcription;
there are no successful timed segments to publish. Empty file chunks remain
skippable under the existing file contract. Ordinary Danish da-000/001/002 and
their PCM16 amplitude ×0.02 variants retain nonempty direct and timed output;
timed outputs validate against actual durations. Automatic-language da-000 also
retains nonempty direct and timed output. In-flight cancellation remains sticky,
and a subsequent cached request returns identical output after removal of only
the disposable model link. All real engine inference runs deny process networking.

The actual Service/store acceptance fixture additionally injects the real
engine's `engine.no_speech` outcome with AI disabled and enabled: both retain
existing failed recovery/audio policy, publish zero history records, and make
zero delivery calls. Its ASR and OS insertion are scripted; this is composition
evidence, not an actual OS insertion test. The unchanged Service error branch
stops before text processing, history and delivery.

## Remaining mixed-window failure

A generated 78.84-second fixture contains 33 seconds of silence, public da-000
(12.84 seconds), then another 33 seconds of silence. With the original native
runtime, timed recognition emitted 22 repeated phantom segments from 0 to
83.52 seconds. The upstream SOT repair removes the first eight silent-window
segments, but the remaining 14 segments still repeat unrelated text from 29.08
to 83.52 seconds. `Transcript::validate(78.84)` therefore still rejects the
result. Direct recognition retains approximate speech; timed speech content is
not preserved. The opt-in mixed test checks strict timing bounds and the known
public speech content anchor `Europa`; that assertion is only a test, never
recognition filtering. This test remains an explicit failing quality acceptance.

A bounded experiment applied the existing 0.85 policy earlier in native window
processing. It improved bounds to 30–64 seconds but still produced unrelated
repeated text, which could turn an invalid result into valid-looking bad output.
That experiment was reverted completely. Neither its policy nor its extra native
patch is part of the final source. No further tuning or speculative VAD was added.

## Reproduction and evidence

Model: task-owned `/tmp/vara-whisper45-diagnostic-models/ggml-base.bin`,
147,951,465 bytes, catalogue SHA256
`60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`.
It was installed once using the existing explicit installer (under 150 MB);
inference did not download or contact a provider. Personal model/key files were
not modified. The normal packaged app bundle was not modified. A mistaken
`vara-desktop --version` invocation omitted the required `cli` argument and
started the normal app for approximately 16 seconds; owned GUI process 81385 was
terminated. It also spawned internal file backend 81392, which outlived the GUI
and required separate MAIN reconciliation. MAIN checked read-only stage/recovery
metadata: four file documents ready, zero recoveries, and no new dictation rows
since 06:15 UTC, then confirmed backend 81392 terminated. No personal document
contents were queried. No UI interaction, microphone or insertion action occurred, but normal
startup profile effects were not audited. The correct `cli --version` invocation
then returned `vara 1.0.2`.

```sh
export PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
export CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_BUILD_JOBS=4
cargo test --release -p vara-engine --locked
VARA_TEST_WHISPER_MODELS=/tmp/vara-whisper45-diagnostic-models \
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
target/release/deps/whisper_runtime-c15ae88f56cd91b0 --ignored --nocapture \
  whisper_rejects_non_speech_preserves_quiet_and_cached_cancellation
# Separate known failing mixed-quality acceptance:
VARA_TEST_WHISPER_MODELS=/tmp/vara-whisper45-diagnostic-models \
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
target/release/deps/whisper_runtime-c15ae88f56cd91b0 --ignored --nocapture \
  whisper_mixed_silent_windows_have_valid_speech_timings
```

Executable suffixes are build-specific; obtain the current path with
`cargo test --release -p vara-engine --test whisper_runtime --no-run --locked`.
Normal tests require neither weights nor downloads. The structured companion
`45-whisper-fix.jsonl` retains observations, timings, hashes and verification
results, including the known mixed failure and reverted experiment.

Relevant task-owned `/tmp/vara-whisper45-*` logs: `probe-build.log`, `install.log`,
`probe-before.jsonl`, `probe-after.jsonl` (upstream-only raw probabilities),
`mixed-baseline.log`, `mixed-diagnostic.log`, `service-build.log`, `service.log`,
`incremental-comment.log`, `incremental-restored.log`, `incremental.json`, and
final normal/core/mixed/normal-desktop verification logs recorded in the JSONL.

Final verification: normal engine tests **56 passed, 3 ignored**; focused real
core regression **1 passed in 14.40 s**; separate mixed quality acceptance
**1 failed in 9.46 s**, as documented above. The Service composition fixture
passed. The final normal desktop build passed in 37.63 s and its CLI version has
no test-harness marker. Task-owned harness binary is retained separately at
`/tmp/vara-whisper45-service-harness`; no packaging or installation occurred.

Final runtime-test SHA256:
`5ef1e770b430c14f3bda253f3c99271aedca5174bc35ebec46f2b8885f09f483`.
Normal standalone desktop SHA256:
`6e4456c47b5201e4d08de0df189ad1b1a66f85bd61a7ef2998c63d8fb3e39407`.
Unchanged packaged desktop binary SHA256:
`97854ca756d4b19fbb68eecd572b6fafb795ab85c09f5bbf74fa218793c8f7b1`.
Final native source and active copied source both hash to
`14b8bfd7959a2db0d41053f157518abc03d51c2159d530c74d819f8b989021d3`.
All planned build/test/fixture processes and the accidental startup's backend
are terminal. MAIN confirmed the backend idle before stopping it.
Task-owned model, logs, profiles and compiler intermediates remain for MAIN review
and cleanup.
Both patch artifacts apply cleanly to the original published source and reproduce
the final native/build-script hashes. The own-file whitespace check passes;
vendored unchanged upstream EOF whitespace and unified-patch context spaces are
preserved, so a whole-vendor `git diff --check` reports those expected exceptions.
