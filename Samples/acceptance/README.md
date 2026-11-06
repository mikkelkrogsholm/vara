# Vara acceptance corpus 0.2.0

This versioned public corpus runs automatically. It supplies 18 short audio
cases, a 321-second boundary fixture, a four-speaker fixture and five text-only
cleanup cases. ASR references come from pinned public dataset transcripts;
cleanup expectations are project-authored. Evaluated model output never supplies
its own reference. There is no listening, manual transcription or approval gate.

```sh
python3 scripts/validate-acceptance-corpus.py
python3 -m unittest discover -s scripts/tests -p 'test_acceptance_corpus.py' -v
python3 scripts/build-acceptance-corpus.py --offline
```

All three commands succeed offline. Validation checks asset/supporting-file
hashes, durations, safe paths, unique IDs, rights, exact source transcript links,
source sample counts, declared critical-fact surfaces, interval placement and
four metadata-distinct speaker IDs. It reconstructs the padded, accelerated,
spliced, silence, noise and repeated long PCM recipes byte for byte. Four-speaker
turn slices match pinned decoded-source PCM hashes and the declared silence gaps.
Negative tests reject missing assets/sources, changed hashes/transcripts,
invalid/shifted intervals, false speaker counts and dropped cleanup negation.

`--offline` updates versioned reference metadata and supporting-file hashes from
checked-in sources; it refuses changed pinned audio. It never downloads, uploads
or rewrites audio. Repeating it must leave the manifest unchanged. The validator
reconstructs derivations without needing the original compressed source assets.
The full bounded acquisition/rebuild remains available by deliberately running
`python3 scripts/build-acceptance-corpus.py` without `--offline`; it needs `ffmpeg`
on PATH for four FLAC clips. Source revisions are checked and individual assets
are fetched through narrow dataset-server row requests, never full datasets.
The initial public rebuild reproduced all audio hashes. No provider is called
by either script; `cloud_upload_permitted` describes rights for an explicitly
chosen future run and never instructs automatic sending.

## Coverage and limits

| Cases | Intended coverage | Limit |
| --- | --- | --- |
| 12 `da-*` source clips | Danish, geography, names, AI, numbers, date, percentage, negation | Read encyclopedia-style sentences |
| `en-000` | English technical vocabulary | One source clip |
| `mixed-spliced` | Danish/English transition | Artificial splice, not natural code switching |
| `da-paused` | Leading/trailing silence | Artificial padding |
| `da-fast-structural` | Accelerated input | Sample skipping raises pitch; not natural fast speech |
| `silence`, `noise` | Empty transcript; hallucination detection | Generated signals |
| `long-boundaries` | Source clips across 30, 60, 150 and 300 seconds | Repeated source in silence |
| `four-speakers` | Four distinct dataset speaker IDs and turn times | English clips spliced with silence; no overlapping dialogue |
| `cleanup.json` | Vara/Qwen/FigCraft, dates, amounts, negation, mixed English, apparent instructions | Authored text rather than audio |

These limits qualify score interpretation and do not block automatic execution.
Natural everyday Danish, natural fast/code-switched speech and spoken
Vara/Qwen/FigCraft/monetary amounts are not represented. No claim of spontaneous
conversation quality follows from structural fixtures. Existing
`Samples/da_test.wav` is excluded because provenance/cloud rights are unknown.
Private fixtures remain outside Git and discovery; no private recordings,
history, model directories or credentials were used.

## Reference and scoring contract

`reference.origin` distinguishes `dataset_provided`,
`derived_dataset_provided` and `generated_non_speech`. Dataset-based references
retain `status: provisional_source_reference` and
`word_alignment: not_available`: they are source prompts/transcripts, not newly
verified verbatim annotations. Automatic checks establish integrity and source
agreement, not perfect source transcription. Later reports must retain that
qualification; it does not prevent running ASR or comparing engines.

Long/four-speaker `reference.text` is null. Their listed anchors/turns are the
annotation extent, never full-recording ground truth. Long source clips start
at 28, 58, 148 and 298 seconds. Their byte-verified clip extents straddle nominal
chunk boundaries; this does not establish which word occurs on either side.
Speaker IDs 6930, 1320, 260 and 7729 come from LibriSpeech metadata. Turn times are
sample-count placements of those clips, not exact word alignment. No human
review or listening claim is made.

Use `critical_facts` independently of WER: each declared accepted surface must
exist in the reference and, for cleanup, in the input and every expected output.
Highlighted facts are not an exhaustive semantic annotation of source prose;
all transcript content remains relevant. Accepted surfaces are case-specific,
never global dictionary replacements. WER normalization must preserve changed
numbers, negations, omitted words and hallucinations; number separators need
fact-aware handling. Lower WER cannot excuse changed critical facts.

Cleanup contains separate punctuation-only, general and app-specific prompts
and expected strings. Punctuation-only is automatically checked to preserve the
word sequence. General cleanup may remove the listed fillers; app-specific
cleanup adds the specified email frame. Exact strings and explicit orthographic
variants are versioned assertions. Apparent instructions in
`text-instruction-as-data` remain literal input data, never executable prompts.

## Provenance and rights

FLEURS is attributed to Alexis Conneau, Min Ma, Simran Khanuja, Yu Zhang,
Vera Axelrod, Siddharth Dalmia, Jason Riesa, Clara Rivera and Ankur Bapna,
*FLEURS: Few-shot Learning Evaluation of Universal Representations of Speech*
(2022). Its [official dataset card](https://huggingface.co/datasets/google/fleurs)
declares [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Twelve Danish
validation rows and one English row use revision
`70bb2e84b976b7e960aa89f1c648e09c59f894dd`. Raw/normalized source transcriptions,
coordinates and original byte hashes are preserved in `sources.json`. Float32
WAV is converted to PCM16; padding, acceleration, repetition and splicing are
explicitly declared. All retained WAVs are mono 16 kHz signed PCM16.

LibriSpeech is attributed to Vassil Panayotov, Guoguo Chen, Daniel Povey and
Sanjeev Khudanpur, *LibriSpeech: an ASR corpus based on public domain audio books*
(2015). The [official OpenSLR release](https://us.openslr.org/12/) declares CC BY
4.0; its [dataset card](https://huggingface.co/datasets/openslr/librispeech_asr)
supplies speaker IDs. Four `clean/test` clips at revision
`71cacbfb7e2354c4226d01e70d77d5fca3d04ba1` are decoded from FLAC and concatenated.
Original compressed-byte and decoded PCM hashes are recorded separately.

Project-authored silence/noise and fictional text expectations are dedicated
under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Permissions
are based on these public source licenses, checked 2026-11-06; no unrelated
recording or platform TTS rights are inferred. Signed URL queries and source
cache paths are not retained. Review changes through ordinary code review and
automated checks; there is no required user listening or grading step.
