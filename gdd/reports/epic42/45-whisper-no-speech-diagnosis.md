# Issue 45 — Whisper no-speech diagnosis

2026-11-07, Apple M3 Max, release CPU, whisper-rs 0.16.0 / whisper-rs-sys 0.15.0.
This is diagnostic evidence; no production inference parameters changed.
The frozen `45-local-runtime` reports remain unchanged.
The bounded implementation and its remaining mixed-window failure are recorded
in [45-whisper-fix.md](45-whisper-fix.md); issue 45 remains open.

The pinned native implementation reads its no-speech probability from an unwritten
logits row. `whisper_batch_prep_legacy` requests logits only for the final prompt
token, but `whisper_full_with_state` computes no-speech softmax from row zero.
With an explicit Danish multilingual prompt these are different rows. The existing
application segment probability filter (`< 0.85`) therefore receives the wrong signal.
This matches the independently documented upstream defect and proposed repair:
[whisper.cpp PR 4068](https://github.com/ggml-org/whisper.cpp/pull/4068),
[SOT-row fix 65dacc8](https://github.com/ggml-org/whisper.cpp/commit/65dacc8).

The opt-in example `whisper_no_speech_probe` measured the shipped full decoder and,
independently, public `pcm_to_mel`, `encode`, `decode([token_sot()])`, and a stable
softmax of `get_logits()[token_nosp()]`. Single-token decode actually writes row
zero. Both full-decoder timestamp modes used explicit `da`, no context, CPU, eight
threads, and Greedy best-of-one, matching the application.

| Fixture | SOT no-speech probability | Native direct probability | Native timed probability |
|---|---:|---:|---:|
| silence | 0.931494 | 0.0000001295 | 0.0000000127 |
| seeded noise | 0.884881 | 0.0000009692 | 0.0000000306 |
| da-000 | 0.017670 | 0.0000192681 | 0.0000039536 |
| da-000, amplitude ×0.02 | 0.214342 | 0.0000192726 | 0.0000192768 |
| da-001 | 0.002095 | 0.0000192740 | 0.0000192865 |
| da-001, amplitude ×0.02 | 0.008319 | 0.0000192759 | 0.0000192830 |
| da-002 | 0.020971 | 0.0000192698 | 0.0000192759 |
| da-002, amplitude ×0.02 | 0.173101 | 0.0000192790 | 0.0000192919 |

All six ordinary/quiet speech cases produced nonempty text in both decoder modes.
Quiet scaling is applied in memory, preserving the public fixture files. This is
bounded retention evidence, not a claim about all quiet speakers or ASR accuracy.
The independent encoder/decode pass cost approximately 0.19 seconds per fixture.

The native default rejection also requires decoder average log probability below
−1. Silence emitted token means −0.7203 direct / −0.6127 timed, and noise −0.3266 /
−0.3657. Thus fixing the native probability alone does not imply native suppression
of these confident hallucinations: the application's existing independent 0.85
segment filter remains necessary. Corrected SOT probabilities place both negatives
above 0.85 and all tested positives below it. Token means here are over emitted
segment tokens; native scoring averages its selected sequence's `result_len`
token `plog` values, so multi-segment output need not have identical segment means.

## Recommended bounded fix

Vendor the exact pinned sys crate reproducibly and apply only the upstream SOT-row
repair, preserving its license, existing backend and cancellation behavior, and
the application's existing 0.85 postfilter. Do not modify the local Cargo registry.
This preserves native per-decoding-window semantics and avoids a new VAD model,
phrase filter, amplitude cutoff, or threshold tuning.

The public low-level API is useful as this independent probe, but using it as a
whole-file gate is inadequate: direct input can span 600 seconds and file chunks
span 300 seconds, while one encoder window sees approximately 30 seconds. An
any-speech gate would allow phantom text in other silent windows; a first-window
gate could discard later speech. A full public-API workaround would need to change
the transcription windowing/context/timing behavior and duplicate encoding.

Implementation acceptance still requires actual fixed-runtime direct and timed
negative/positive tests, mixed silent/speech windows, cancellation and Service
publication verification. Empty local timed chunks must remain skippable and final
empty file aggregation must preserve the existing `speech.file_empty` contract;
direct dictation maps empty text to `engine.no_speech`.

## Reproduction and retained artifacts

Model installed into task-owned `/tmp/vara-whisper45-diagnostic-models` using the
existing explicit installer. It contains 147,951,465-byte `ggml-base.bin`, SHA256
`60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`, exactly the
catalogue artifact. Personal model files were only listed, never modified.

```sh
export PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
export CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_BUILD_JOBS=4
cargo build --release -p vara-engine --example whisper_no_speech_probe --example install_whisper_acceptance
target/release/examples/install_whisper_acceptance /tmp/vara-whisper45-diagnostic-models
VARA_TEST_WHISPER_MODEL=/tmp/vara-whisper45-diagnostic-models/ggml-base.bin \
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
target/release/examples/whisper_no_speech_probe
cargo test --release -p vara-engine
```

Inference ran under process-local network deny, with no provider calls. Normal
tests passed 55 tests (39 unit, 5 media, 10 integrity, 1 runtime messages), with one
explicit real-weight test ignored; normal tests require neither weights nor download.
Build and diagnostic probe completed successfully. Logs retained in `/tmp`:
`vara-whisper45-probe-build.log`, `vara-whisper45-tests-build.log`,
`vara-whisper45-install.log`, `vara-whisper45-normal.log`,
`vara-whisper45-probe-before.jsonl`, `vara-whisper45-probe-before.stderr`.
