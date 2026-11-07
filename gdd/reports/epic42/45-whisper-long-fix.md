# Issue 45 — Whisper speech regions and long silent windows

2026-11-07, Apple M3 Max, release CPU. The previously failing 78.84-second
public-audio acceptance now passes through the actual shared `Engines` direct
and timed APIs, including quiet speech and automatic Danish detection. Native
CLI file jobs and original TXT/SRT/JSON exports retain the original timeline.
This is bounded regression evidence, not a claim of general Whisper Base accuracy.
The frozen `45-whisper-fix.*` and original runtime reports remain unchanged.

## Runtime change

Whisper Base is paired with an explicitly downloaded, pinned Silero VAD 5.1.2
model. The existing Download action completes older Base-only installations;
there is no download during recognition. The same `local.whisper` engine ID,
saved settings, CPU backend and ASR parameters remain. Both managed hashes are
verified before caching the runtime pair; readiness additionally verifies the
885 KB helper. Missing and same-size corrupt helpers fail explicitly instead of
falling back. Cached recognition remains usable after removal of both disposable
model links. Cancellation is checked before/after detection and each ASR region;
VAD's native recurrent buffers are cleared at the start of every detection call.

The detector sees a peak-normalized copy of the audio, with its existing 0.5
probability threshold, 100-ms minimum speech (matching the existing input
minimum rather than adding the detector's default 250-ms gate), 1-second
minimum silence and 200-ms context padding.
Recognition receives the original PCM, independently for each detected speech
region. Native ASR times are checked against that region before adding its
original offset. IDs include the region and the outer file chunk. The validator
uses the existing core 100-ms timestamp quantization tolerance, without rewriting
times: public da-002 reports a native 5.00-second end within a 4.97-second region
(30 ms difference). A unit check rejects 110-ms excess, negative/nonfinite times
and empty intervals. Empty speech regions remain skippable, and an entirely
empty file retains `speech.file_empty`; direct dictation retains `engine.no_speech`.
The upstream SOT-logits repair and native build refresh fix remain unchanged.

The public Rust `FullParams.enable_vad` flag cannot provide this preprocessing:
`WhisperState::full` calls `whisper_full_with_state`, whereas the pinned native
VAD wrapper runs in `whisper_full`. The adapter therefore invokes the supported
`WhisperVadContext` API explicitly, without a native patch or new dependency.

## Diagnostic comparison

The first real VAD run removed repeated phantom text and invalid long-window
times. With 200-ms padding but unnormalized detector input, however, quiet speech
lost its first 1.63 seconds relative to the ordinary speech detection boundary.
Two bounded alternatives were measured, with unchanged recognition PCM:

| Detector input and context | Actual result |
| --- | --- |
| Original PCM, 200 ms | Ordinary mixed text equals short control with exact +33-second times; quiet opening is lost. |
| Original PCM, 2 seconds | Quiet opening restored; short/mixed quiet recognition differs because available context differs. |
| Peak-normalized detector only, 200 ms | Ordinary and quiet mixed text each equal their full corresponding short control; every native segment shifts exactly +33 seconds. Silence/noise still produce no regions. |

The last variant was selected. These initial diagnostic comparisons used the
native 250-ms minimum-speech default; final production and the rerun acceptance
use 100 ms to preserve the existing accepted audio minimum. On quiet mixed audio the detector peak probability
is approximately 0.999925 after normalization (versus 0.816384 before); actual
Whisper no-speech probability is 0.037318. Normalized deterministic noise receives
54.61× detector gain but still has no speech regions (maximum probability
0.054634). Silence has maximum probability 0.012030 and no regions. Diagnostic
raw probabilities, texts, timings and all 14-case control comparisons are retained
in `45-whisper-long-vad-probe.jsonl`. Raw timestamps on diagnostic untimed decoder
segments are native sentinel values and are not recognition timing evidence.

## Actual acceptance

Normal engine tests: **58 passed, 4 ignored**. The three opt-in real runtime tests
pass in **30.92 seconds**, with process networking denied. They cover:

- Original 33-second silence + da-000 + 33-second silence fixture, ordinary and
  PCM16 ×0.02 quiet speech, each with explicit Danish and automatic language.
  Both direct and timed full text equal their corresponding short controls;
  timed speech retains `Europa` and negation `ikke`, with each timestamp shifted
  by 33 seconds. Ordinary timed segments are 34.78–40.42 and 40.42–43.10 seconds;
  quiet segments are 34.78–40.42 and 40.42–42.94. No repeated phantom segments.
- Two distinct public phrases separated by 33 seconds: the second remains at
  79.26–84.70 seconds, in order after the first. Region IDs remain unique.
- A 345.84-second fixture with an entirely silent first 300-second file chunk:
  speech remains at 301.78–307.42 and 307.42–310.10 seconds, with `c300` IDs.
- Existing 321-second `long-boundaries.wav`: all four legitimate da-000 utterances
  at source offsets 28, 58, 148 and 298 seconds retain `Europa` in their original
  locations. Legitimate repeated speech is retained. The final hard-cut phrase
  crosses the unchanged 300-second chunk boundary and is reported at 300–308
  seconds; its wording differs from the complete short control.
- Silence/noise rejection for both direct/file APIs and da/automatic language;
  da-000/001/002 ordinary and quiet; automatic short speech; sticky in-flight
  cancellation and identical subsequent cached output after removing both links.
- Base-only and corrupt VAD failure/readiness, plus fully verified installation
  reuse with denied networking (no model fetch).

After the final 100-ms minimum-speech compatibility adjustment, all three real
tests were rerun and the source-matched final normal desktop build passed in **36.38 seconds** after the 100-ms region
validation adjustment. A fresh backend from that exact binary (PID 24234) produced
actual CLI da-002 `ready`, with its unmodified native 1.05–6.05-second segment.
The previous owned backend had already exited before this run. CLI version is
`vara 1.0.2`, with no test-harness marker. Every desktop invocation explicitly
sets `VARA_DATA_DIR=/tmp/vara-whisper45-long-profile`; no normal GUI was started.

Actual shared CLI jobs also reached `ready` for mixed explicit-Danish audio
(`c9641576-fdac-4e20-88e6-d9c743e002c4`) and delayed automatic-language audio
(`72dd4826-aa21-4806-ac44-e05648af1353`). Six original exports were checked:
TXT contains the speech/negation, JSON exactly preserves original transcript and
IDs, and SRT starts at `00:00:34,780` / `00:05:01,780`. Exports and receipts remain
under `/tmp/vara-whisper45-long-exports` and `/tmp/vara-whisper45-long-export-receipts.json`.
CLI/backend processes denied all IP networking while permitting their required
local Unix socket IPC; a separate control verified IP socket binding was denied.

Existing Base-only installation was completed using the existing installer in
`/tmp/vara-whisper45-long-migration`: only the 885,098-byte helper was added;
Base size, SHA256 and nanosecond modification time were unchanged. A subsequent
network-denied installer run verifies `ready=true`, proving both verified files
are reused. All models, profiles and generated audio are disposable task-owned
files. No personal models, credentials, installed app or frozen reports changed.

## Pinned models and limits

Whisper Base: revision `5359861c739e955e79d9a303bcbc70fb988958b1`, 147,951,465 bytes,
SHA256 `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`.
Silero VAD: MIT, official
[ggml-org/whisper-vad](https://huggingface.co/ggml-org/whisper-vad/tree/9ffd54a1e1ee413ddf265af9913beaf518d1639b),
revision `9ffd54a1e1ee413ddf265af9913beaf518d1639b`, 885,098 bytes,
SHA256 `29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf`.
Exact revision URLs were used; metadata/license cards and downloaded hashes
were checked, with the VAD metadata verified before its download.
Public FLEURS fixtures and their reference/rights are recorded in the unchanged
acceptance manifest. No private audio, human listening or cloud ASR was used.

Whisper Base's Danish wording remains approximate. In da-001 it still recognizes
`4 %` where the reference says `firs procent` (80%); this change does not repair
that numerical error. Hard chunk boundaries can change wording. The detector
and tested fixtures do not establish universal hallucination avoidance.

## Reproduction and artifacts

```sh
export PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
export CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_BUILD_JOBS=4
cargo test --release -p vara-engine --locked
VARA_TEST_WHISPER_MODELS=/tmp/vara-whisper45-long-models \
  sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  target/release/deps/whisper_runtime-c15ae88f56cd91b0 \
  --ignored --nocapture --test-threads=1
cargo build --release -p vara-desktop --locked
VARA_DATA_DIR=/tmp/vara-whisper45-long-profile \
  sandbox-exec -p '(version 1)(allow default)(deny network*)(allow network* (local unix-socket))(allow network* (remote unix-socket))' \
  target/release/vara-desktop cli transcribe \
  /tmp/vara-whisper45-long-fixtures/delayed300.wav \
  --model local.whisper --timeout 120
```

The test executable suffix is build-specific. Normal tests do not require models
or downloads. Full commands/results, actual output and artifact hashes are in
`45-whisper-long-fix.jsonl`; build/runtime/CLI logs use `/tmp/vara-whisper45-long-*`.
The prior normal standalone binary is retained at
`/tmp/vara-whisper45-long-desktop-before` (SHA256 `6e4456c47b5201e4d08de0df189ad1b1a66f85bd61a7ef2998c63d8fb3e39407`).
Final standalone SHA256 is `a9f5d8c8f9ba5823679511549c2aba1bdbe8857d433c83600a08c4e6a1cfcf7a`.
The packaged app binary remains unchanged at SHA256
`97854ca756d4b19fbb68eecd572b6fafb795ab85c09f5bbf74fa218793c8f7b1`.
No packaging, installation, push or merge occurred.

All associated build, test, installer and CLI jobs are terminal. The final owned
backend PID 24234 exited through its normal idle timeout; it is no longer running.
