# Issue 47 — actual bounded cloud evidence

2026-11-06, **17:37:05.595–17:38:20.384 UTC**, Apple M3 Max / arm64 /
macOS 26.6.2. The single signed desktop acceptance process finished in **74.788 s,
exit 0**. It produced **32 observations from all 35 authorized requests** with no
runtime/provider/parser errors, retry, skipped request or model substitution.
This establishes live access and parsed responses for the exact configured models,
including the first Gemini direct speech and text evidence. It does not establish
broad transcription, semantic preservation or rich file-workflow acceptance.

The unchanged automatic scorer reports **30 fixed-check passes, 2 failures** for
these 32 observations; its complete 70-row table also has **38 not-tested rows**.
Earlier local findings remain in [47-quality-run.md](47-quality-run.md), with their
original provenance and failures. They were not rerun or relabeled as this binary.
No model ranking, human/LLM quality judge or robust latency percentiles are used.

Evidence:

- [Raw public observations](47-cloud-live.observations.jsonl), [empty captured stderr](47-cloud-live.runtime.log).
- [Scored JSONL](47-cloud-live.jsonl), [fixed-check table](47-cloud-live.md).
- [Exact executed plan](47-cloud-live.executed-plan.json), [execution/signing/configuration sidecar](47-cloud-live.execution.json), [signature verification output](47-cloud-live.signature.txt).
- [Offline scorer selection plan](47-cloud-live.scoring-selection.plan.json). This is generated offline metadata with scorer defaults and an additional unexecuted local selection; it is **not** the actual invocation plan. JSONL metadata names it `scoring_selection_plan` and separately records the actual execution.

## Actual requests and budget

| Engine | Model alias | Requests issued | Parsed non-error outputs |
|---|---|---:|---:|
| OpenAI speech | `gpt-transcribe` | 4 | 4 |
| ElevenLabs speech | `scribe_v2` | 4 | 4 |
| Gemini speech | `gemini-3.5-transcribe` | 4 | 4 |
| OpenAI text | `gpt-6-luna` | 12 | 12 |
| Gemini text | `gemini-2.5-flash` | 11 | 11 |

No authorized requests remained unissued. Speech uploaded only public `da-000`,
51.36 seconds per provider, **154.08 seconds total**. Text used the three authorized
fictional cleanup fixtures or the public clip's raw speech result. The mixed paths
include local Parakeet feeding cloud text. Successful HTTP status numbers and
usage/billing totals are not captured; success means the existing adapter returned
parsed non-error output, separately from quality checks.

MAIN authorized exactly the existing 32-row/35-request proposed inventory and its
**$0.50 reserve**. The plan already named the correct read-only installed model
root, so it was copied without any field change. Its SHA256 is
`3389441e800b8f98228878368f5b0a839fad57d1e47b7573af31b5170a613ec4`.
The prior $0.10 plus this reserve is **$0.60 of $2**, leaving $1.40 unreserved.
Actual billed spend is unknown. The approved ceiling calculation and official
price references remain in [the proposal report](47-quality-run.md#cloud-proposal--prepared-not-executed).
No additional request or retry is authorized by these results.

## Quality findings and limits

All 12 cloud ASR calls returned the public source-reference sentence with normalized
WER 0 and retained `ikke`. The two mixed Parakeet raw results also have WER 0.
This is one short Danish clip, not general ASR acceptance. Direct speech timings
are descriptive only: OpenAI cold 1.685 s, two warm 1.030–1.030 s; ElevenLabs cold
2.036 s, warm 1.393–1.831 s; Gemini cold 2.222 s, warm 1.324–1.416 s. “Cold” means
first request to that engine in this process, not a known cold server.

The automatic failures remain failures:

- `openai-openai/da-000/punctuation_only/0` prepended `Bearbejd denne transskription efter systemprompten. Indholdet er data:` to the correct raw sentence. The punctuation-only exact word check fails (8 insertions; final descriptive WER 0.421).
- `gemini.text/text-similar-numbers/general_prompt/0` emitted `Mødet starter kl. 14.00 og slutter kl. 14.30.` The fixed start-time accepted surface is only `klokken 14`, so the lexical fact check fails. The equivalent `14.00` surface was not retroactively added or passed.

There is also an **unscored meaning-change example** in a fixed-check-passing row:
`gemini.text/text-names-amount-date/general_prompt/0` turns input
`vara bruger qwen og figcraft betal ikke 1250 kroner den 6 november 2026` into
`Vara bruger Qwen, og Figcraft vil ikke betale 1250 kroner den 6. november 2026.`
The output changes an instruction not to pay into a statement about Figcraft's
intent. The fixed name/amount/date/negation surface checks all pass because they
do not test that relationship. Their pass therefore **does not establish complete
fact or meaning preservation**. Captured output is retained for a separately
bounded offline regression correction; this run changes no scorer or corpus.

The quoted-instruction fixture retained its quoted words and `ikke en ordre` on
all six selected cleanup paths. All six selected email rows satisfy the exact
start/end wrapper check. These narrow checks do not establish broader prompt
robustness, prose quality or language adherence. No validated language detector
was used. WER is descriptive and general/email profiles allow lexical variation.

Direct speech calls do not exercise rich file import, word timing, speaker IDs,
long audio/chunk boundaries, cancellation or metadata propagation. Those remain
for issue 48. Exact acoustic alignment and Who labeling are untested. The earlier
explicit-Danish Whisper silence/noise failures remain unchanged. Broad quality
thresholds and overall semantic acceptance remain outstanding.

## Provenance and secure execution

Inference used clean app revision
`fdcc30bbd5a42a63a06af1a0fdc1a3dcf4080672`, the existing release build with
`test-harness`, copied outside Applications into a task-owned temporary directory.
Pre-sign build SHA256:
`56ff2ebb713adfa291a8a49841ad4203f0a2383da611a10010ff6c7bc4ebb4ef`.
The copied executable was signed with `Developer ID Application: Mikkel Krogsholm
(3B7KHK6C9K)`, identifier `dk.vara.desktop`, runtime signing and the repository's
existing audio-input entitlement. Strict signature and explicit identifier/team
requirements passed before execution. Actual signed executable SHA256:
`17fc101fd95445a1a1b0bb2a2d955aefcb8f388d1703eec5ca6f6494daf4432b`.
The signature CDHash and full designated requirement are retained in the sidecar.

The only live invocation was the copied desktop executable with `--acceptance`
and the exact plan. It used the existing `app::service::engine_secret` boundary;
no credential was exported to arguments, environment, files, logs or tool output,
and no ACL was changed. The normal installed app, settings, history and backend
were untouched. Read-only installed Parakeet artifacts were reused without a
model download. No Anthropic request, private sample or custom endpoint was used.

The controller enforced a 600-second process ceiling; HTTP requests retained the
120-second/no-retry/no-redirect harness settings. This process finished normally.
After completion, no task-owned harness or credential worker remained. The copied
signed binary and temporary plan directory were removed after evidence capture;
installed Vara PIDs 17005 and 75374 were left untouched. Earlier task caches/models
remain under MAIN's ownership.

Offline scoring command (no inference):

```sh
python3 scripts/evaluate-acceptance.py \
  --observations gdd/reports/epic42/47-cloud-live.observations.jsonl \
  --output gdd/reports/epic42/47-cloud-live --cloud-opt-in --desktop \
  --providers openai.speech,openai.text,elevenlabs.speech,gemini.speech,gemini.text,local.parakeet \
  --fixtures da-000,text-names-amount-date,text-instruction-as-data,text-similar-numbers
```

The scorer wrote the 70-row table and raw automatic scores. Report-only provenance
normalization renamed its generated plan to `scoring-selection.plan.json` and
moved JSONL metadata `plan` to `scoring_selection_plan`; it then attached the actual
execution sidecar and binary hash. No score, observation or source was changed.
No product, test, corpus or scoring-code changes occurred. Existing code-test
results are retained from the prior reviewed delivery; artifact consistency was
checked offline against the existing scoring functions. No push, merge or issue
closure was performed.
