# Issue 47 — deterministic quality and latency baseline

NFC + Unicode casefold; punctuation/whitespace delimit tokens. Numbers and negations are retained. Only versioned per-case orthographic aliases apply. WER is descriptive: no broad acceptance threshold has been set from this baseline. Pass means the listed fixed checks passed. Broader ASR quality is baseline-only and not accepted; no WER threshold exists yet. A high-WER row can pass a narrow fact check. Insertions are candidate hallucinations; substitutions can reflect provisional source spelling. No listening or LLM judge.

| Path | Case / profile / language | Fixed checks | Raw WER | Final WER | Total s |
|---|---|---|---:|---:|---:|
| local.parakeet → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.parakeet → local.text | da-000 / punctuation_only / fixture | not_tested | — | — | — |
| openai.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.685 |
| openai.speech → openai.text | da-000 / punctuation_only / fixture | fail | 0.000 | 0.421 | 4.575 |
| local.parakeet → openai.text | da-000 / punctuation_only / fixture | pass | 0.000 | 0.000 | 2.332 |
| elevenlabs.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 2.036 |
| local.parakeet → gemini.text | da-000 / punctuation_only / fixture | pass | 0.000 | 0.000 | 2.427 |
| gemini.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 2.222 |
| gemini.speech → gemini.text | da-000 / punctuation_only / fixture | pass | 0.000 | 0.000 | 2.889 |
| elevenlabs.speech → openai.text | da-000 / punctuation_only / fixture | pass | 0.000 | 0.000 | 3.107 |
| local.parakeet → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | da-000 / direct / fixture | not_tested | — | — | — |
| openai.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.030 |
| openai.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.030 |
| elevenlabs.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.831 |
| elevenlabs.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.393 |
| gemini.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.416 |
| gemini.speech → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.324 |
| local.parakeet → direct | da-001 / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | en-000 / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | mixed-spliced / direct / auto | not_tested | — | — | — |
| local.parakeet → direct | da-paused / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | da-fast-structural / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | silence / direct / auto | not_tested | — | — | — |
| local.parakeet → direct | noise / direct / auto | not_tested | — | — | — |
| local.parakeet → direct | long-boundaries / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | four-speakers / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | da-001 / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | en-000 / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | mixed-spliced / direct / auto | not_tested | — | — | — |
| local.whisper → direct | da-paused / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | da-fast-structural / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | silence / direct / auto | not_tested | — | — | — |
| local.whisper → direct | noise / direct / auto | not_tested | — | — | — |
| local.whisper → direct | long-boundaries / direct / fixture | not_tested | — | — | — |
| local.whisper → direct | four-speakers / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | silence / direct / da | not_tested | — | — | — |
| local.parakeet → direct | noise / direct / da | not_tested | — | — | — |
| local.whisper → direct | silence / direct / da | not_tested | — | — | — |
| local.whisper → direct | noise / direct / da | not_tested | — | — | — |
| text → local.text | text-names-amount-date / punctuation_only / fixture | not_tested | — | — | — |
| text → local.text | text-instruction-as-data / punctuation_only / fixture | not_tested | — | — | — |
| text → local.text | text-similar-numbers / punctuation_only / fixture | not_tested | — | — | — |
| text → local.text | text-names-amount-date / general_prompt / fixture | not_tested | — | — | — |
| text → local.text | text-instruction-as-data / general_prompt / fixture | not_tested | — | — | — |
| text → local.text | text-similar-numbers / general_prompt / fixture | not_tested | — | — | — |
| text → local.text | text-names-amount-date / app_specific_prompt / fixture | not_tested | — | — | — |
| text → local.text | text-instruction-as-data / app_specific_prompt / fixture | not_tested | — | — | — |
| text → local.text | text-similar-numbers / app_specific_prompt / fixture | not_tested | — | — | — |
| text → openai.text | text-names-amount-date / punctuation_only / fixture | pass | — | 0.000 | 0.973 |
| text → openai.text | text-instruction-as-data / punctuation_only / fixture | pass | — | 0.000 | 1.280 |
| text → openai.text | text-similar-numbers / punctuation_only / fixture | pass | — | 0.000 | 3.384 |
| text → openai.text | text-names-amount-date / general_prompt / fixture | pass | — | 0.000 | 0.997 |
| text → openai.text | text-instruction-as-data / general_prompt / fixture | pass | — | 0.000 | 1.024 |
| text → openai.text | text-similar-numbers / general_prompt / fixture | pass | — | 0.077 | 0.990 |
| text → openai.text | text-names-amount-date / app_specific_prompt / fixture | pass | — | 0.000 | 1.062 |
| text → openai.text | text-instruction-as-data / app_specific_prompt / fixture | pass | — | 0.000 | 0.953 |
| text → openai.text | text-similar-numbers / app_specific_prompt / fixture | pass | — | 0.062 | 3.084 |
| text → gemini.text | text-names-amount-date / punctuation_only / fixture | pass | — | 0.000 | 5.782 |
| text → gemini.text | text-instruction-as-data / punctuation_only / fixture | pass | — | 0.000 | 2.480 |
| text → gemini.text | text-similar-numbers / punctuation_only / fixture | pass | — | 0.000 | 5.707 |
| text → gemini.text | text-names-amount-date / general_prompt / fixture | pass | — | 0.154 | 6.564 |
| text → gemini.text | text-instruction-as-data / general_prompt / fixture | pass | — | 0.062 | 4.452 |
| text → gemini.text | text-similar-numbers / general_prompt / fixture | fail | — | 0.231 | 2.008 |
| text → gemini.text | text-names-amount-date / app_specific_prompt / fixture | pass | — | 0.000 | 1.452 |
| text → gemini.text | text-instruction-as-data / app_specific_prompt / fixture | pass | — | 0.000 | 1.528 |
| text → gemini.text | text-similar-numbers / app_specific_prompt / fixture | pass | — | 0.000 | 1.083 |

## Short repeated timings

Cold = first request to an engine in one process; for local engines includes loading (OS cache uncontrolled), for cloud engines does not establish server model load. Warm = reused engine. Load cannot be separated from ASR/AI by this adapter. Three short observations support a median and range only; no percentiles or ranking.

- local.parakeet: cold []; warm n=0. Not tested.
- local.whisper: cold []; warm n=0. Not tested.
- openai.speech: cold [1.68535875]; warm n=2, median 1.030 s, range 1.030–1.030 s.
- elevenlabs.speech: cold [2.036445959]; warm n=2, median 1.612 s, range 1.393–1.831 s.
- gemini.speech: cold [2.22167175]; warm n=2, median 1.370 s, range 1.324–1.416 s.

## Limits

Source references are provisional public dataset transcripts. Structural composites are repeated/spliced clips, not natural long speech or conversation. Anchor checks compare full lexical sequences inside known clip intervals (±2 s); errors, duplicated and missing sequences remain inspectable in JSONL. Segment-only timing cannot establish word anchors. Raw provider speaker IDs are retained without equating IDs across chunks or to source identities. Exact acoustic word alignment and Who labeling remain not_tested. This report does not run the independent UI player fixture; its separate result is in 47-quality-run.md. No automatic prompt tuning, model download, user settings/history or private audio. Cloud rows require explicit fixture and provider filters plus opt-in. Inference errors and unavailable models are failures, never passes.
