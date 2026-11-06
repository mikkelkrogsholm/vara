# Vara acceptance corpus 0.1.0

**Prepared, not yet human reviewed. Issue #43 remains incomplete.** This small
public corpus supplies fixed inputs to later provider comparisons. It contains
18 short cases, one 321-second boundary case and one four-speaker case, plus five
text-only cleanup cases. No provider has been called and nothing is uploaded by
these scripts. `cloud_upload_permitted` records rights for an explicitly chosen
future run; it is not an automatic-send instruction.

```sh
python3 scripts/validate-acceptance-corpus.py
python3 -m unittest discover -s scripts/tests -p 'test_acceptance_corpus.py' -v
python3 scripts/validate-acceptance-corpus.py --require-reviewed
```

The first two commands must pass. The last intentionally fails until listening
review and text-expectation review are complete. Structural validation alone is
not quality acceptance. All WAV assets are mono 16 kHz signed PCM16. Paths are
relative to this directory, durations come from sample counts, and SHA-256 covers
actual files. `sources.json` and `cleanup.json` are also hashed by the manifest.

## Coverage and limits

| Cases | Intended coverage | Limitation |
| --- | --- | --- |
| 12 `da-*` source clips | Danish, geography, names, AI, numbers, date, percentage, negation | Read encyclopedia-style sentences, not everyday conversation |
| `en-000` | English technical vocabulary | One source clip |
| `mixed-spliced` | Danish/English language transition | Artificial splice, not natural code switching |
| `da-paused` | Leading/trailing silence | Artificial padding |
| `da-fast-structural` | Short accelerated input | Sample skipping raises pitch; not natural fast speech |
| `silence`, `noise` | Empty transcript; hallucination detection | Generated signals, not recorded background noise |
| `long-boundaries` | Speech placements across 30, 60, 150 and 300 seconds | Repeated source in silence, not a natural long recording |
| `four-speakers` | Four metadata-distinct speaker IDs with turn times | English clips spliced with silence, no overlap or natural dialogue |
| `cleanup.json` | Names Vara/Qwen/FigCraft, dates, amounts, negation, mixed English, apparent instructions | Authored text expectations, not audio or trusted model-generated facit |

Natural everyday Danish, natural fast speech, natural mixed Danish/English,
spoken Vara/Qwen/FigCraft and spoken monetary amounts remain coverage gaps.
Existing `Samples/da_test.wav` is excluded: provenance and cloud rights were not
established. No private recordings, history, model directories or credentials
were used. Private opt-in fixtures must stay outside Git and require their own
explicit rights and cloud permission; these scripts never discover them.

## References, scoring and annotation extent

`reference.text` preserves the source dataset text (or the corresponding derived
sequence). FLEURS original and normalized transcriptions are separately retained
in `sources.json`. Source read prompts/transcripts are provenance evidence, not
proof of a new listening review or guaranteed verbatim agreement. No synthetic
speech or generated model transcript is used as human ground truth.

Every case currently has `human_listening_review: pending`, null reviewer/date,
and pending interval review where applicable. No human-listening or hand-checked
anchor claim is made. Long/four-speaker `reference.text` is null: their listed
anchors/turns are the declared annotation extent, never a full transcript.
Placement times are computed from sample counts, not hand-aligned word timings.
The repeated long anchors start at 28, 58, 148 and 298 seconds. Their source
intervals straddle nominal 30-second chunk boundaries; listening must establish
which words actually occur around each boundary, including the five-minute mark.
Four speaker IDs come directly from LibriSpeech metadata: 6930, 1320, 260, 7729.
The `speaker_id` values identify source speakers; source utterance labels must
not be treated as additional identities.

Use case-specific `critical_facts` independently of WER. The current highlighted
fact list is not exhaustive and remains pending review; all source content must
still be preserved. Accepted surfaces are
explicit examples, not global dictionary replacements. WER may ignore ordinary
case/punctuation and collapse whitespace, but must not drop numbers, negations,
words or hallucinations. Decimal/thousands separators require fact-aware handling.
A lower WER cannot pass an output that changes a critical fact. Do not strip
numbers or apparent instructions as normalization. Extra or omitted content must
remain visible. No quality score is claimed by this fixture validator.

Cleanup profiles have independent prompts and expected strings. Punctuation-only
preserves all words and their order; general cleanup may remove listed fillers;
app-specific cleanup adds only the specified email framing. Exact strings are
reviewable exemplars, with only listed orthographic variants permitted. Apparent
instructions in `text-instruction-as-data` must remain quoted/transcribed data.
Do not execute that input or allow it to replace the cleanup prompt.

## Provenance and rebuilding

FLEURS is attributed to Alexis Conneau, Min Ma, Simran Khanuja, Yu Zhang,
Vera Axelrod, Siddharth Dalmia, Jason Riesa, Clara Rivera and Ankur Bapna,
*FLEURS: Few-shot Learning Evaluation of Universal Representations of Speech*
(2022). Its [official dataset card](https://huggingface.co/datasets/google/fleurs)
declares [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
We use 12 Danish validation rows and one English validation row at revision
`70bb2e84b976b7e960aa89f1c648e09c59f894dd`. Source text is unchanged; audio is
converted from float32 WAV to PCM16. Derived padding, acceleration, repetition
and splicing are declared in each case. Source audio bytes have their own hashes.

LibriSpeech is attributed to Vassil Panayotov, Guoguo Chen, Daniel Povey and
Sanjeev Khudanpur, *LibriSpeech: an ASR corpus based on public domain audio books*
(2015). The [official OpenSLR release](https://us.openslr.org/12/) declares
CC BY 4.0; the [dataset card](https://huggingface.co/datasets/openslr/librispeech_asr)
provides the `speaker_id` metadata. Four `clean/test` assets at revision
`71cacbfb7e2354c4226d01e70d77d5fca3d04ba1` are decoded from FLAC and concatenated.
Generated silence/noise is project-authored and dedicated under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Permissions here are
based on these public source licenses, checked 2026-11-06; no rights are inferred
for unrelated recordings or platform TTS voices.

To rebuild deliberately, run `python3 scripts/build-acceptance-corpus.py` from the
repository. It requires Python 3.10+ and `ffmpeg` on PATH for four FLAC clips,
fetches bounded dataset-server row requests and individual assets only, checks
source revisions, and overwrites this corpus's audio/manifest/source metadata.
There are no installs, full corpus downloads, cloud calls or background jobs.
Signed asset query strings and local source cache paths are not retained.
Review the Git diff and hashes after rebuilding; source/server changes fail
explicitly rather than authorizing a silent corpus revision.

## Human review checklist

1. Listen to every short file against its source reference; correct any mismatch
   with an attributed annotation change. Verify all names, numbers and negations.
2. Listen around each long boundary and record the audible word anchors. Review
   all four source turns and speaker changes. Keep the annotation extent honest;
   sparse anchors must never become full-recording ground truth.
3. Have a Danish-speaking reviewer approve all five cleanup cases and their three
   transformations, preserving uncertainty, facts and literal instruction text.
4. Record reviewer and date, mark reviewed case/interval fields only when actually
   reviewed, update supporting-file hashes if changed, and run the review gate.
5. Add separately licensed natural audio for the declared coverage gaps before
   claiming complete issue #43 acceptance or representative Danish quality.
