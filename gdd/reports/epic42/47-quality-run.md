# Issue 47 — local baseline and prepared cloud inventory

2026-11-06, Apple M3 Max / arm64 / macOS 26.6.2. Automatic source-reference checks only.
This is a partial issue delivery: **38 local observations, 20 fixed-check passes,
12 failures, 38 not-tested rows in the 70-row inventory**. Thirty-two cloud rows
have never been executed in this run. Six executed local rows have descriptive
WER/segment timing but no applicable accepted broad-quality/word-anchor check.
The generated table labels its status **Fixed checks**; a high-WER row passing a
single negation check is not broad ASR acceptance. Broader WER thresholds remain
unset for MAIN's baseline review. No model ranking or robust latency percentiles.

Evidence: [scored JSONL](47-quality-local.jsonl), [table](47-quality-local.md),
[raw observations](47-quality-local.observations.jsonl),
[local inventory/config](47-quality-local.plan.json),
[proposed cloud inventory](47-cloud-proposed.plan.json).
The report records the base app revision `141d30f` and dirty worktree explicitly:
inference ran during implementation at approximately 17:24–17:28 UTC; the
report timestamp and report-source hashes describe subsequent offline scoring.
The initial developer example's executable hash was not captured before rebuilding;
this provenance limitation is retained. Future runner executions capture executable
SHA256 and each harness row has a Unix timestamp. This delivery's final Git commit
identifies the reviewed source, not a claim that all observations ran that commit.

## Actual findings

- Parakeet Danish `da-000`: normalized WER 0, critical negation retained. Cold
  total 1.101 s; two warm totals 0.100–0.120 s, median 0.110 s.
- Whisper Danish `da-000`: WER 0.895, while retaining the narrow negation
  expectation. Cold 0.338 s; two warm totals 0.268–0.269 s, median 0.268 s.
  This is poor source-reference accuracy, not a quality pass.
- Parakeet→embedded CPU Qwen on actual `da-000` preserved the raw word sequence.
  Separate authored cleanup cases expose number/name/negation, instruction-data
  and email-format failures. General/email rewriting permits lexical variation;
  only punctuation-only uses exact normalized word-order acceptance. WER for all
  profiles remains descriptive. Language adherence has no validated automatic
  detector and is explicitly not tested.
- Explicit Danish Whisper silence/noise reproduces issue 45 exactly: silence
  `Det er ikke en del af det, vi kan ikke tage den.`; noise `(tronk)`.
  Both are failures. Auto-language silence/noise returned `engine.no_speech`
  instead. These are separate configuration rows, with `da` versus `auto` in
  the table; no filtering, tuning, new VAD or production defect fix occurred.
  Parakeet rejects all four non-speech checks.
- The 321-second Parakeet composite has valid transcript bounds. Two anchors
  match exactly; two have `moltider` versus reference `måltider`, so strict full
  source-reference anchor matching fails. That lexical mismatch does not establish
  missing speech. The four-speaker composite matches all four placed references
  within ±2-second clip windows. Raw Parakeet speaker IDs are absent, and source
  speaker identity is not inferred. Whisper supplies segment timing: exact word
  anchors remain not tested. Every actual rich transcript and offsets is retained.
- Independent real HTML audio fixture: word `w3` seeking compares the actual
  `HTMLAudioElement.currentTime` against the original supplied fixture word's
  start (2.4 s), independently of rendered `data-time`. It also asserts the DOM
  coordinate equals that source value. The existing selection test checks stopping
  at the selected supplied word boundary. This establishes player-coordinate
  behavior, not acoustic alignment of any model-generated word time.

Models/backends and exact read-only artifact hashes reuse the still-valid
[issue 45 evidence](45-local-runtime.jsonl): pinned voxrt Redux revision
`4fab626c00708d9bc97f837d309e0d362094282b`, Int8 Metal encoder; Whisper Base CPU,
whisper-rs 0.16.0; Qwen3-0.6B Q8_0, Candle 0.11.0 CPU, no Ollama. No repeated
integrity/cancellation/download acceptance. The task-owned Whisper file remains
in `/tmp/vara-epic42-45-whisper-models` under MAIN's retained ownership.

## Runner and secure boundary

`scripts/evaluate-acceptance.py` validates the versioned corpus offline, chooses
bounded existing paths, writes the exact plan, and produces JSONL plus Markdown.
Without `--binary` it makes no inference calls. Without `--cloud-opt-in` it selects
local rows only. Cloud execution additionally requires explicit `--providers`,
`--fixtures`, and `--desktop`; ordinary PR CI only runs deterministic tests.
`--observations FILE` re-scores saved public observations without inference.

The feature-gated `acceptance` example refuses all cloud credentials. The desktop
`--acceptance PLAN` entry exists only with `test-harness`, follows credential-worker
entry before GUI startup, and invokes existing `service::engine_secret` directly.
No secret is exported through args, environment, files, stdout or stderr. The
harness can upload only baked-in, SHA256-checked public corpus IDs and authored
cleanup text, using default allowlisted engine configurations. It cannot accept
arbitrary audio paths, caller-supplied text, custom endpoints, models or prompts.
Missing/inaccessible credentials produce a failed row and independent rows continue.
Malformed inventory/ceilings fail before reading credentials. Production builds
have no acceptance entry. This run did not invoke the signed credential path.

Hard per-plan guards: at most 100 rows, 40 cloud requests, 180 audio seconds,
30 seconds per uploaded clip, 120 seconds per HTTP request, no redirects/retries,
2048 UTF-8 bytes per text input. The Python controller adds a 600-second process
ceiling (blocking native inference also terminates). Existing OpenAI text output
cap is 8192; Gemini text cap is 4096. Acceptance-only Gemini speech explicitly
sends `maxOutputTokens: 4096`; unchanged production request tests still assert
its original configuration. Speech API charges for OpenAI/ElevenLabs are audio-based.

## Cloud proposal — prepared, not executed

The supplied proposed plan is exactly **35 requests**:

| Provider/path | Calls | Audio or output ceiling |
|---|---:|---|
| OpenAI speech | 4 | 51.36 audio seconds |
| ElevenLabs speech | 4 | 51.36 audio seconds |
| Gemini speech | 4 | 51.36 audio seconds; 16,384 output tokens total |
| OpenAI text | 12 | 98,304 output tokens total |
| Gemini text | 11 | 45,056 output tokens total |

Total uploaded audio: **154.08 seconds**, public `da-000` only. Text profiles use
three fixed authored cases; selected mixed paths reuse bounded Parakeet/public
or cloud raw output. Input ceiling is 2048 UTF-8 bytes plus the fixed system/user
wrapper per call (conservative reserve allows 3000 input tokens per text call).
The model aliases are exactly the default configuration in each observation:
`gpt-transcribe`, `gpt-6-luna`, `scribe_v2`, `gemini-3.5-transcribe`,
`gemini-2.5-flash`. Availability is not assumed. No automatic model substitution.

MAIN's current official price review, 2026-11-06:

- [OpenAI pricing](https://developers.openai.com/api/docs/pricing): Luna
  $0.10/$0.50 per million input/output tokens; transcription $0.0045/minute.
  Configured maximum text output: `12*8192*0.50/1e6 = $0.049152`.
- [ElevenLabs API pricing](https://elevenlabs.io/pricing/api): Scribe v2
  $0.22/hour; `51.36/3600*0.22 = $0.003139`.
- [Gemini pricing](https://ai.google.dev/gemini-api/docs/pricing): 2.5 Flash
  $0.30/$2.50 per million input/output tokens, including thinking output;
  configured maximum text output `11*4096*2.50/1e6 = $0.112640`.
  Gemini speech: $2/M audio tokens, $12/M text output tokens; approximate ordinary
  blended $0.005/minute is not the worst-case ceiling. Maximum speech output
  `4*4096*12/1e6 = $0.196608` is included separately.

Maximum configured outputs sum to **$0.358400**, plus bounded inputs/audio.
**Proposed conservative reserve: $0.50**, subject to MAIN's actual diff/plan review
before invocation. This is a ceiling reserve, not actual billing or a spend claim.
Earlier epic reserve $0.10 plus this proposed reserve would be $0.60, leaving $1.40
of the $2 epic budget. **No additional cloud spend occurred in this delivery.**
No retries, additional fixtures or requests are implicitly approved by this plan.

## Checks and reproduction

Build environment (all Rust commands):

```sh
export PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
export CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_BUILD_JOBS=4
cargo build --release -p vara-engine --features test-harness --example acceptance
cargo test --release -p vara-engine --features test-harness
cargo check --release -p vara-desktop --features test-harness
cargo build --release -p vara-desktop --features test-harness
python3 scripts/validate-acceptance-corpus.py
python3 -m unittest discover -s scripts/tests -p 'test_acceptance*.py'
```

Rust: **67 passed, 1 ignored** real-weight acceptance test; no live test in normal
CI. Python: **23 passed** (14 corpus and 9 scorer/report checks). Mock HTTP asserts
Gemini's actual acceptance token ceiling. Guard tests reject missing opt-in,
unknown/private fixture IDs, unknown engines/prompts, wrong engine kind, excessive
request/audio/timeout ceilings and long cloud audio before credential access;
a two-row inaccessible-key test confirms the inventory continues.

The first local replay used a disposable directory containing only symlinks to
already-verified installed Parakeet/Qwen and the retained task Whisper file:

```sh
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  python3 scripts/evaluate-acceptance.py \
  --binary target/release/examples/acceptance --models TASK_MODEL_SYMLINK_ROOT \
  --output gdd/reports/epic42/47-quality-local
```

Only four supplementary explicit-Danish non-speech rows were then run via the
same example under that network-deny profile. Their observations were appended;
no blind whole-matrix replay. Offline re-scoring:

```sh
python3 scripts/evaluate-acceptance.py \
  --observations gdd/reports/epic42/47-quality-local.observations.jsonl \
  --output gdd/reports/epic42/47-quality-local
```

UI: `node desktop/tests/legacy-runner.cjs`, with existing cached Playwright module
and Chromium selected through `PLAYWRIGHT_MODULE` / `PLAYWRIGHT_CHROMIUM`, and
`VARA_TEST_OUTPUT_ROOT=/tmp/vara-epic42-47-browser`: **23 main UI + 17 transcription
checks passed**. No dependencies installed. The player used synthetic fixture
`desktop/tests/fixtures/player.wav`, not personal audio or model speech alignment.

Logs: `/tmp/vara-epic42-47-build.log`, `-rust-tests.log`, `-python-tests.log`,
`-desktop-check.log`, `-desktop-build.log`, `-local.log`, `-supplement.log`,
`-browser.log`. Final release build/signing evidence is for MAIN to review; no
bundle, install, push, merge or issue closure by this worker. Scratch model links
and browser screenshot intermediates are removed after checks; retained text logs
and committed public result artifacts provide evidence. The shared serial target
cache is retained per MAIN instruction. Local quality failures, broad threshold
review, live cloud results, actual secure-key invocation and optional Who labeling
remain outstanding; unavailable/skipped rows are not passes.
