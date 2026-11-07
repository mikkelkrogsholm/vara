# Issue 47 — fixed-corpus regression policy 1.0.0

Offline evaluation of 70 frozen observations (38 local, 32 cloud): **46 pass / 21 fail / 3 not_tested**. Zero inference requests or downloads.

Initial MAIN gates selected after freezing the baseline; baseline-informed, not independent validation or production acceptance. Gates are deliberately allowed to fail this baseline. Source references remain provisional; spliced/accelerated/repeated fixtures are controlled tests, not natural conversation or statistical production/language acceptance.

Raw clean Danish/English/paused WER ≤0.10; mixed/accelerated WER ≤0.30. Zero declared fact/relation violations, zero punctuation-only word edits, exact email first/last lines. Non-speech requires empty/no-speech; all other inference errors fail. Raw failure remains visible regardless of cleanup. Missing reference/word timing stays not_tested. Structure retains captured production duration validation (+0.1 s) and full lexical clip anchors (±2 s); no acoustic alignment or speaker identity claim.

Short total latency: local first-use ≤5 s, warm ≤2 s; any cloud stage ≤10 s. Applied to the explicit short case list in the policy, including fixed text; silence/noise and long/four-speaker fixtures excluded. Recorded Apple M3 Max release environment only. First-use includes loading with uncontrolled OS cache; cloud phase is process reuse, not server model load. No p95, SLO or model ranking.

| Observation | Result | Failed / untested gates |
|---|---|---|
| parakeet/da-000/direct/0 | pass | all applicable gates passed |
| whisper/da-000/direct/0 | fail | raw_quality: fail |
| parakeet-qwen/da-000/punctuation_only/0 | pass | all applicable gates passed |
| openai/da-000/direct/0 | pass | all applicable gates passed |
| openai-openai/da-000/punctuation_only/0 | fail | punctuation_only: fail |
| parakeet-openai/da-000/punctuation_only/0 | pass | all applicable gates passed |
| elevenlabs/da-000/direct/0 | pass | all applicable gates passed |
| parakeet-gemini/da-000/punctuation_only/0 | pass | all applicable gates passed |
| gemini/da-000/direct/0 | pass | all applicable gates passed |
| gemini-gemini/da-000/punctuation_only/0 | pass | all applicable gates passed |
| elevenlabs-openai/da-000/punctuation_only/0 | pass | all applicable gates passed |
| parakeet/da-000/direct/1 | pass | all applicable gates passed |
| parakeet/da-000/direct/2 | pass | all applicable gates passed |
| whisper/da-000/direct/1 | fail | raw_quality: fail |
| whisper/da-000/direct/2 | fail | raw_quality: fail |
| openai/da-000/direct/1 | pass | all applicable gates passed |
| openai/da-000/direct/2 | pass | all applicable gates passed |
| elevenlabs/da-000/direct/1 | pass | all applicable gates passed |
| elevenlabs/da-000/direct/2 | pass | all applicable gates passed |
| gemini/da-000/direct/1 | pass | all applicable gates passed |
| gemini/da-000/direct/2 | pass | all applicable gates passed |
| parakeet/da-001/direct/0 | pass | all applicable gates passed |
| parakeet/en-000/direct/0 | pass | all applicable gates passed |
| parakeet/mixed-spliced/direct/0 | fail | raw_quality: fail |
| parakeet/da-paused/direct/0 | pass | all applicable gates passed |
| parakeet/da-fast-structural/direct/0 | fail | raw_quality: fail |
| parakeet/silence/direct/0 | pass | all applicable gates passed |
| parakeet/noise/direct/0 | pass | all applicable gates passed |
| parakeet/long-boundaries/direct/0 | fail | raw_quality: not_tested, structure: fail |
| parakeet/four-speakers/direct/0 | not_tested | raw_quality: not_tested |
| whisper/da-001/direct/0 | fail | raw_quality: fail, asr_critical_facts: fail |
| whisper/en-000/direct/0 | pass | all applicable gates passed |
| whisper/mixed-spliced/direct/0 | fail | raw_quality: fail |
| whisper/da-paused/direct/0 | fail | raw_quality: fail |
| whisper/da-fast-structural/direct/0 | fail | raw_quality: fail, asr_critical_facts: fail |
| whisper/silence/direct/0 | pass | all applicable gates passed |
| whisper/noise/direct/0 | pass | all applicable gates passed |
| whisper/long-boundaries/direct/0 | not_tested | raw_quality: not_tested, structure: not_tested |
| whisper/four-speakers/direct/0 | not_tested | raw_quality: not_tested, structure: not_tested |
| parakeet-explicit-da/silence/direct/0 | pass | all applicable gates passed |
| parakeet-explicit-da/noise/direct/0 | pass | all applicable gates passed |
| whisper-explicit-da/silence/direct/0 | fail | raw_quality: fail |
| whisper-explicit-da/noise/direct/0 | fail | raw_quality: fail |
| local.text/text-names-amount-date/punctuation_only/0 | fail | cleanup_critical_facts: fail, punctuation_only: fail |
| local.text/text-instruction-as-data/punctuation_only/0 | pass | all applicable gates passed |
| local.text/text-similar-numbers/punctuation_only/0 | fail | punctuation_only: fail |
| local.text/text-names-amount-date/general_prompt/0 | fail | cleanup_critical_facts: fail |
| local.text/text-instruction-as-data/general_prompt/0 | fail | cleanup_critical_facts: fail |
| local.text/text-similar-numbers/general_prompt/0 | pass | all applicable gates passed |
| local.text/text-names-amount-date/app_specific_prompt/0 | fail | cleanup_critical_facts: fail, email_wrapper: fail |
| local.text/text-instruction-as-data/app_specific_prompt/0 | fail | email_wrapper: fail |
| local.text/text-similar-numbers/app_specific_prompt/0 | fail | email_wrapper: fail |
| openai.text/text-names-amount-date/punctuation_only/0 | pass | all applicable gates passed |
| openai.text/text-instruction-as-data/punctuation_only/0 | pass | all applicable gates passed |
| openai.text/text-similar-numbers/punctuation_only/0 | pass | all applicable gates passed |
| openai.text/text-names-amount-date/general_prompt/0 | pass | all applicable gates passed |
| openai.text/text-instruction-as-data/general_prompt/0 | pass | all applicable gates passed |
| openai.text/text-similar-numbers/general_prompt/0 | pass | all applicable gates passed |
| openai.text/text-names-amount-date/app_specific_prompt/0 | pass | all applicable gates passed |
| openai.text/text-instruction-as-data/app_specific_prompt/0 | pass | all applicable gates passed |
| openai.text/text-similar-numbers/app_specific_prompt/0 | pass | all applicable gates passed |
| gemini.text/text-names-amount-date/punctuation_only/0 | pass | all applicable gates passed |
| gemini.text/text-instruction-as-data/punctuation_only/0 | pass | all applicable gates passed |
| gemini.text/text-similar-numbers/punctuation_only/0 | pass | all applicable gates passed |
| gemini.text/text-names-amount-date/general_prompt/0 | fail | cleanup_critical_facts: fail |
| gemini.text/text-instruction-as-data/general_prompt/0 | pass | all applicable gates passed |
| gemini.text/text-similar-numbers/general_prompt/0 | pass | all applicable gates passed |
| gemini.text/text-names-amount-date/app_specific_prompt/0 | pass | all applicable gates passed |
| gemini.text/text-instruction-as-data/app_specific_prompt/0 | pass | all applicable gates passed |
| gemini.text/text-similar-numbers/app_specific_prompt/0 | pass | all applicable gates passed |

## Recorded repeated short timings

| Direct da-000 engine / phase | n | Median s | Range s |
|---|---:|---:|---:|
| local.parakeet / cold | 1 | 1.101 | 1.101–1.101 |
| local.parakeet / warm | 2 | 0.110 | 0.100–0.120 |
| local.whisper / cold | 1 | 0.338 | 0.338–0.338 |
| local.whisper / warm | 2 | 0.268 | 0.268–0.269 |
| openai.speech / cold | 1 | 1.685 | 1.685–1.685 |
| openai.speech / warm | 2 | 1.030 | 1.030–1.030 |
| elevenlabs.speech / cold | 1 | 2.036 | 2.036–2.036 |
| elevenlabs.speech / warm | 2 | 1.612 | 1.393–1.831 |
| gemini.speech / cold | 1 | 2.222 | 2.222–2.222 |
| gemini.speech / warm | 2 | 1.370 | 1.324–1.416 |

## Provenance and limits

JSONL retains frozen inference provenance and hashes separately from current policy/scorer revision and hashes. All earlier #47 artifacts and raw bytes remain unchanged. Local inference used dirty base 141d30f with an initial binary-hash gap; cloud used clean fdcc30b and its recorded signed binary hash. Current source was not used to rerun these outputs.

The high-WER Whisper da-000 rows fail despite their passing narrow negation fact. Parakeet accelerated and mixed-spliced rows fail; captured Danish Whisper non-speech hallucinations and cleanup payment-intent/wrapper failures remain failures. Passing declared relations cannot establish unrestricted meaning preservation. Validated language detection, native OS insertion/UI, Who and exact acoustic alignment remain not_tested. Separate #48 native adapter evidence does not fill the native OS gap.

Reproduce: `python3 scripts/evaluate-acceptance-policy.py --output gdd/reports/epic42/47-regression-policy-v1`.
Normal CI: `python3 -m unittest discover -s scripts/tests -p 'test_acceptance*.py'`. 44 tests passed, including 10 new offline policy regressions. No Cargo/product/runtime changes.
