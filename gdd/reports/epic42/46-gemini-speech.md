# Issue 46 — Gemini Developer API speech

Implemented 2026-11-06 against clean base `aafdaee`, after issues 43–45.
This report distinguishes offline implementation acceptance from live inference.
No credentials, private profiles, external inference, downloads, installation or
paid requests were used in this worker run.

## Implementation

- Adds optional `gemini.speech` / `Provider::Gemini` / speech/cloud with candidate
  model `gemini-3.5-transcribe`. Existing Gemini text and CLI engines stay separate;
  speech reuses the `gemini` OS credential account. New/default selections remain
  local/direct. Both settings readers add missing built-ins without replacing
  selections, app rules, text models or custom speech model overrides.
- Public `Engines::speech` dispatches explicitly to Gemini, never the OpenAI URL.
  Rich files use the shared normalized five-minute chunks, file queue, transcript
  validation, offsets and chunk-scoped speaker IDs. Strict-local rejects cloud.
- Uses fixed Developer API `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`,
  a validated model path segment and header-only `x-goog-api-key`.
  `contents[].parts[].inlineData` contains base64 WAV and `mimeType:audio/wav`.
  `generationConfig.audioTranscriptionConfig` uses `mode:VERBATIM` and
  `languageCodes:[]` for automatic language or `[hint]`. File calls additionally
  set `wordTimestamp:true` and `diarization:true`. No smart rewriting, vocabulary,
  prompt-driven synthetic transcript schema or Files upload lifecycle is added.
- Guards the file before reading and again afterward at 14 MB; the actual
  serialized JSON is capped at 20 MB. Normalized five-minute chunks fit (~9.6 MB
  PCM). Rejects invalid/empty WAV or durations beyond documented unary limits
  (one hour; 30 minutes when requesting timestamps).
- Selects one completed candidate; alternative candidates are not concatenated.
  Excludes thought parts. Structured `audioTranscription.text` takes precedence
  over duplicate text parts; ordinary text parts remain an untimed fallback.
  `speakerLabel` is retained, and actual `words[].word/startOffset/endOffset`
  values map from decimal-second Protobuf Duration strings. Malformed, negative,
  nonfinite, reversed, out-of-range or unordered times fail. Empty/absent word
  arrays or absent optional offsets preserve text and labels as `timing:none`.
  Mixed timing capability clears timing rather than inventing missing intervals.
- Existing English/Danish components expose Gemini in separate dictation/file
  model pickers and identify its shared provider key as “Speech and text”.
  CLI discovery, shared submission, result and export use the same engine enum.

## Documentation basis and live limits

MAIN rechecked the official Developer API documentation before the worker handoff:
[transcription guide](https://ai.google.dev/gemini-api/docs/generate-content/transcribe?hl=en)
and [GenerateContent REST schema](https://ai.google.dev/api/generate-content?hl=en).
The REST fields above belong to GenerateContent, not the differing Interactions
or Vertex schemas. Documented languages include Danish; diarization supports up
to eight speakers, with three or more experimental. These are documented limits,
not measured quality or account access.

**Not live tested:** availability/access of `gemini-3.5-transcribe`, Danish
recognition, actual timing/diarization behavior, billing and real microphone to
field delivery. A model catalogue result alone cannot close this gap. The bounded
live check belongs to the later authorized acceptance step. Existing Whisper
silence/noise quality failures from issue 45 remain failures; no tuning or quality
claim is hidden by these protocol tests.

## Verification

Release environment: `PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH`,
`DEVELOPER_DIR=/Library/Developer/CommandLineTools`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_RELEASE_DEBUG=0`, `CARGO_BUILD_JOBS=4`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo test --release -p vara-core -p vara-engine -p vara-app --locked` | Passed, 98 tests + 1 ignored real Redux weights fixture | `/tmp/vara-epic42-46-rust-final.log` |
| `cargo build --release -p vara-desktop --features test-harness --locked` | Passed; existing Keychain deprecations and `root_gate` warning | `/tmp/vara-epic42-46-build.log` |
| `python3 scripts/tests/agent_cli.py target/release/vara-desktop` | Passed; actual shared backend and seeded Gemini result/export | `/tmp/vara-epic42-46-cli-final.log` |
| `node desktop/tests/i18n.test.cjs` | Passed | Terminal output |
| `node desktop/tests/runtime-messages.test.cjs` | Passed, 453 runtime codes | `/tmp/vara-epic42-46-runtime-messages.log` |
| `node desktop/tests/ui-main-i18n.playwright.cjs` | Passed, EN/DA Gemini selection, unchanged text choice and shared-key dialog | `/tmp/vara-epic42-46-ui.log` |
| `node desktop/tests/transcription-i18n.playwright.cjs` | Passed, EN/DA Gemini file locality labels | `/tmp/vara-epic42-46-tx-i18n.log` |
| `node desktop/tests/runtime-i18n.playwright.cjs` | Passed, EN/DA engine label/runtime consumers | `/tmp/vara-epic42-46-runtime-i18n.log` |
| `node desktop/tests/legacy-runner.cjs` | Passed, 23 main UI + 17 transcription checks including Gemini file selection | `/tmp/vara-epic42-46-browser.log` |

Browser commands use the existing Playwright module and installed headless
Chromium via `PLAYWRIGHT_MODULE` / `PLAYWRIGHT_CHROMIUM`; no dependencies installed.
Canonical desktop design assets were prepared with `scripts/sync-desktop-design.py`.

The local HTTP suite asserts exact request path/headers/body, base64 bytes,
automatic/explicit language, direct/rich flags, actual timings and speakers,
text fallback, thought filtering and candidate selection. Both Gemini speech
routes reuse the existing error/redaction matrix: missing/blank/invalid keys,
401/403/429/500/503, connection errors, timeout, interrupted/oversized/malformed
responses, pre-cancel and cancellation while awaiting late success. Local server
threads are joined before tests finish. Additional tests exercise public dispatch,
missing/empty/partial timing, malformed durations, range guards, migration,
strict-local policy and shared file submission/cancellation. CLI ready results
are explicitly seeded fixtures, not inference evidence.
