# Issue 47 — reference 0.2.1 offline correction

This delivery corrects two fixed authored test defects using already captured
outputs. **No model was rerun, no provider request was made, and no output or
provider prompt was changed.** The 0.2.0 [local baseline](47-quality-run.md) and
[cloud baseline](47-cloud-run.md), their raw observations, scores and plans remain
frozen unchanged. New corpus/reference **0.2.1** scoring combines the same 38 local
and 32 cloud observations: **50 fixed-check passes, 14 failures, 6 not tested**.
A not-tested row here has an observation but lacks an applicable accepted broad
quality/word-anchor check. Counts are not general quality acceptance.

Artifacts: [new scored JSONL](47-reference021.jsonl), [new table](47-reference021.md),
[combined unchanged observations](47-reference021.observations.jsonl),
[provenance/hashes/status changes](47-reference021.provenance.json),
[offline selection metadata](47-reference021.scoring-selection.plan.json).
The selection plan is scorer metadata, not an inference invocation or spending
authorization. Original inference revisions and limitations are retained in the
provenance: local base `141d30f` with dirty worktree and initial binary-hash gap;
cloud clean `fdcc30b` with the actual signed binary hash. The current Git revision
in JSONL is named `scoring_app_revision`, never an inference revision.

## What changed

| Captured row | Frozen 0.2.0 | Corrected 0.2.1 | Reason |
|---|---|---|---|
| `gemini.text/text-names-amount-date/general_prompt/0` | pass | fail | `Figcraft vil ikke betale ...` is a statement about a payer's intent and fails the new imperative-payment relation |
| `gemini.text/text-similar-numbers/general_prompt/0` | fail | pass | `Mødet starter kl. 14.00 og slutter ...` uses an explicitly accepted equivalent start-time spelling |

These are the **only two changed row statuses**, caused by corrected assertions,
not improved model behavior. Cloud subset totals remain 30 pass / 2 fail, but the
failed rows change: the OpenAI punctuation-only user-wrapper insertion still
fails, and the Gemini payment intent change now fails instead of valid `14.00`.
Local statuses remain 20 pass / 12 fail / 6 not tested. Existing Danish Whisper
silence/noise failures, lexical/structure failures and all raw outputs remain.

`cleanup.json` now declares a contiguous `payment_instruction` relation for the
fixed names/amount/date case. Accepted instruction forms are `betal ikke` and
`undlad at betale`, directly followed by `1250 kroner`, `1.250 kroner` or `1250 kr.`.
The separate negation fact explicitly accepts the `undlad at betale` equivalent.
Original authored references pass; changing to `FigCraft vil ikke betale`,
`FigCraft må ikke betale` or `Vara betaler ikke` fails. Dropping payment negation,
negating `undlad`, or changing the amount also fails.

The start-time fact now includes its declared positive relation and following
boundary: `mødet starter ... og slutter`. It explicitly accepts `klokken 14`,
`klokken 14.00` and `kl. 14.00`; colon/dot normalize equivalently. This prevents
`klokken 14.30` from satisfying a mere `klokken 14` prefix and rejects inserted
start-time negation. The captured local equivalent `mødet der starter på klokken
14 og slutter` is separately allowed and remains passing. No unrestricted time
parser, model-dependent rule or generalized semantic judge was added.

All punctuation-only exact word/order checks are unchanged: converting `klokken
14` to `kl. 14.00` can satisfy the time fact while still correctly failing the
punctuation-only word-preservation check. Descriptive WER mechanics and broad
threshold policy are unchanged.

## Scope and verification

The corpus version, builder metadata version, cleanup supporting-file hash and
README are updated coherently. The validator rejects cleanup/manifest version
mismatches. **All 20 audio bytes, pinned source records and audio references are
unchanged.** Offline builder repetition is idempotent and downloads nothing.
No Rust source/defaults or remote adapters changed; a future Rust build includes
the new metadata through the existing `include_str!` dependency.

Checks:

```sh
python3 scripts/build-acceptance-corpus.py --offline
python3 scripts/validate-acceptance-corpus.py
python3 -m unittest discover -s scripts/tests -p 'test_acceptance*.py' -v
python3 scripts/evaluate-acceptance.py \
  --observations gdd/reports/epic42/47-reference021.observations.jsonl \
  --output gdd/reports/epic42/47-reference021
git diff --check
```

**28 automatic tests passed**, including captured bad payer output rejection,
authored references and deliberately allowed imperative forms, captured cloud
`14.00` and local relative-clause acceptance, wrong payer/intent/amount/time and
negation rejection, punctuation-only preservation, and version consistency.
Test log: `/tmp/vara-epic42-47-reference-tests.log`. Corpus validator passed.
Offline consistency verification reproduced every score with unchanged scorer
functions, checked exactly two status changes, compared all 20 asset/source
hashes with the frozen base, and verified all earlier report files byte-for-byte.
No Cargo run was needed because no Rust code changed.

After offline scoring, report-only metadata names the generated plan
`scoring_selection_plan`, adds original observation sources and reference hashes,
and records scoring revision separately. This does not alter scores or outputs.

These facts remain narrow declared relations. Unlisted legitimate paraphrases
may fail, and surrounding context can still change meaning while containing an
accepted phrase. Passing checks do not establish complete fact preservation,
general semantic equivalence, language adherence or broad model quality. Rich
file metadata/word timing/speakers and issue 48 remain outside this delivery.
No additional spend, inference, model download, install, push, merge or issue
closure occurred.
