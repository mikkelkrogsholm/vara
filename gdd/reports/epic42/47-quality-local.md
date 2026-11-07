# Issue 47 — deterministic quality and latency baseline

NFC + Unicode casefold; punctuation/whitespace delimit tokens. Numbers and negations are retained. Only versioned per-case orthographic aliases apply. WER is descriptive: no broad acceptance threshold has been set from this baseline. Pass means the listed fixed checks passed. Broader ASR quality is baseline-only and not accepted; no WER threshold exists yet. A high-WER row can pass a narrow fact check. Insertions are candidate hallucinations; substitutions can reflect provisional source spelling. No listening or LLM judge.

| Path | Case / profile / language | Fixed checks | Raw WER | Final WER | Total s |
|---|---|---|---:|---:|---:|
| local.parakeet → direct | da-000 / direct / fixture | pass | 0.000 | — | 1.101 |
| local.whisper → direct | da-000 / direct / fixture | pass | 0.895 | — | 0.338 |
| local.parakeet → local.text | da-000 / punctuation_only / fixture | pass | 0.000 | 0.000 | 1.972 |
| openai.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| openai.speech → openai.text | da-000 / punctuation_only / fixture | not_tested | — | — | — |
| local.parakeet → openai.text | da-000 / punctuation_only / fixture | not_tested | — | — | — |
| elevenlabs.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.parakeet → gemini.text | da-000 / punctuation_only / fixture | not_tested | — | — | — |
| gemini.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| gemini.speech → gemini.text | da-000 / punctuation_only / fixture | not_tested | — | — | — |
| elevenlabs.speech → openai.text | da-000 / punctuation_only / fixture | not_tested | — | — | — |
| local.parakeet → direct | da-000 / direct / fixture | pass | 0.000 | — | 0.120 |
| local.parakeet → direct | da-000 / direct / fixture | pass | 0.000 | — | 0.100 |
| local.whisper → direct | da-000 / direct / fixture | pass | 0.895 | — | 0.268 |
| local.whisper → direct | da-000 / direct / fixture | pass | 0.895 | — | 0.269 |
| openai.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| openai.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| elevenlabs.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| elevenlabs.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| gemini.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| gemini.speech → direct | da-000 / direct / fixture | not_tested | — | — | — |
| local.parakeet → direct | da-001 / direct / fixture | pass | 0.077 | — | 0.058 |
| local.parakeet → direct | en-000 / direct / fixture | not_tested | 0.000 | — | 0.040 |
| local.parakeet → direct | mixed-spliced / direct / auto | not_tested | 0.355 | — | 0.116 |
| local.parakeet → direct | da-paused / direct / fixture | pass | 0.000 | — | 0.107 |
| local.parakeet → direct | da-fast-structural / direct / fixture | pass | 0.923 | — | 0.028 |
| local.parakeet → direct | silence / direct / auto | pass | 0.000 | — | 0.020 |
| local.parakeet → direct | noise / direct / auto | pass | 0.000 | — | 0.020 |
| local.parakeet → direct | long-boundaries / direct / fixture | fail | — | — | 2.491 |
| local.parakeet → direct | four-speakers / direct / fixture | pass | — | — | 0.193 |
| local.whisper → direct | da-001 / direct / fixture | fail | 0.846 | — | 0.248 |
| local.whisper → direct | en-000 / direct / fixture | not_tested | 0.000 | — | 0.229 |
| local.whisper → direct | mixed-spliced / direct / auto | not_tested | 0.774 | — | 0.428 |
| local.whisper → direct | da-paused / direct / fixture | pass | 0.789 | — | 0.279 |
| local.whisper → direct | da-fast-structural / direct / fixture | fail | 1.077 | — | 0.314 |
| local.whisper → direct | silence / direct / auto | pass | 0.000 | — | 0.354 |
| local.whisper → direct | noise / direct / auto | pass | 0.000 | — | 0.359 |
| local.whisper → direct | long-boundaries / direct / fixture | not_tested | — | — | 6.218 |
| local.whisper → direct | four-speakers / direct / fixture | not_tested | — | — | 0.358 |
| local.parakeet → direct | silence / direct / da | pass | 0.000 | — | 0.909 |
| local.parakeet → direct | noise / direct / da | pass | 0.000 | — | 0.020 |
| local.whisper → direct | silence / direct / da | fail | — | — | 0.717 |
| local.whisper → direct | noise / direct / da | fail | — | — | 0.214 |
| text → local.text | text-names-amount-date / punctuation_only / fixture | fail | — | 0.077 | 0.643 |
| text → local.text | text-instruction-as-data / punctuation_only / fixture | pass | — | 0.000 | 0.578 |
| text → local.text | text-similar-numbers / punctuation_only / fixture | fail | — | 0.077 | 0.674 |
| text → local.text | text-names-amount-date / general_prompt / fixture | fail | — | 0.231 | 0.610 |
| text → local.text | text-instruction-as-data / general_prompt / fixture | fail | — | 0.250 | 0.626 |
| text → local.text | text-similar-numbers / general_prompt / fixture | pass | — | 0.385 | 0.670 |
| text → local.text | text-names-amount-date / app_specific_prompt / fixture | fail | — | 0.062 | 0.856 |
| text → local.text | text-instruction-as-data / app_specific_prompt / fixture | fail | — | 0.000 | 0.844 |
| text → local.text | text-similar-numbers / app_specific_prompt / fixture | fail | — | 0.000 | 0.798 |
| text → openai.text | text-names-amount-date / punctuation_only / fixture | not_tested | — | — | — |
| text → openai.text | text-instruction-as-data / punctuation_only / fixture | not_tested | — | — | — |
| text → openai.text | text-similar-numbers / punctuation_only / fixture | not_tested | — | — | — |
| text → openai.text | text-names-amount-date / general_prompt / fixture | not_tested | — | — | — |
| text → openai.text | text-instruction-as-data / general_prompt / fixture | not_tested | — | — | — |
| text → openai.text | text-similar-numbers / general_prompt / fixture | not_tested | — | — | — |
| text → openai.text | text-names-amount-date / app_specific_prompt / fixture | not_tested | — | — | — |
| text → openai.text | text-instruction-as-data / app_specific_prompt / fixture | not_tested | — | — | — |
| text → openai.text | text-similar-numbers / app_specific_prompt / fixture | not_tested | — | — | — |
| text → gemini.text | text-names-amount-date / punctuation_only / fixture | not_tested | — | — | — |
| text → gemini.text | text-instruction-as-data / punctuation_only / fixture | not_tested | — | — | — |
| text → gemini.text | text-similar-numbers / punctuation_only / fixture | not_tested | — | — | — |
| text → gemini.text | text-names-amount-date / general_prompt / fixture | not_tested | — | — | — |
| text → gemini.text | text-instruction-as-data / general_prompt / fixture | not_tested | — | — | — |
| text → gemini.text | text-similar-numbers / general_prompt / fixture | not_tested | — | — | — |
| text → gemini.text | text-names-amount-date / app_specific_prompt / fixture | not_tested | — | — | — |
| text → gemini.text | text-instruction-as-data / app_specific_prompt / fixture | not_tested | — | — | — |
| text → gemini.text | text-similar-numbers / app_specific_prompt / fixture | not_tested | — | — | — |

## Short repeated timings

Cold = first request to an engine in one process; for local engines includes loading (OS cache uncontrolled), for cloud engines does not establish server model load. Warm = reused engine. Load cannot be separated from ASR/AI by this adapter. Three short observations support a median and range only; no percentiles or ranking.

- local.parakeet: cold [1.100566458]; warm n=2, median 0.110 s, range 0.100–0.120 s.
- local.whisper: cold [0.337600666]; warm n=2, median 0.268 s, range 0.268–0.269 s.
- openai.speech: cold []; warm n=0. Not tested.
- elevenlabs.speech: cold []; warm n=0. Not tested.
- gemini.speech: cold []; warm n=0. Not tested.

## Limits

Source references are provisional public dataset transcripts. Structural composites are repeated/spliced clips, not natural long speech or conversation. Anchor checks compare full lexical sequences inside known clip intervals (±2 s); errors, duplicated and missing sequences remain inspectable in JSONL. Segment-only timing cannot establish word anchors. Raw provider speaker IDs are retained without equating IDs across chunks or to source identities. Exact acoustic word alignment and Who labeling remain not_tested. This report does not run the independent UI player fixture; its separate result is in 47-quality-run.md. No automatic prompt tuning, model download, user settings/history or private audio. Cloud rows require explicit fixture and provider filters plus opt-in. Inference errors and unavailable models are failures, never passes.
