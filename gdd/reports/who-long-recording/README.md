# Who long-recording model upgrade diagnostic — 2026-10-30

The unchanged official SDK 4.2.1/model v0.3.4 completed the full 6,921.877375-second recording and returned **21 speaker IDs / 3,456 turns**, compared with 20 IDs / 3,433 turns in the saved baseline. The user reports four actual people. This upgrade alone does **not** resolve the excess-ID result. No IDs were dropped, merged, forced to four or threshold-tuned.

The first four baseline IDs are long-lived: they occur in 23, 21, 19 and 12 five-minute windows. Their globally optimal one-to-one candidate mapping is respectively `who-0`, `who-1`, `who-2`, `who-3`. They preserve 5,836.88 of 5,842.54 baseline speaker-seconds (99.9031%). Across all 75 anchor/window pairs with baseline activity, both the dominant candidate and independently optimal window assignment agree with that global mapping. This comparison finds no additional anchor identity swaps relative to the saved baseline. It cannot exclude swaps shared by both outputs, and it is not human-verified diarization accuracy.

All 17 additional candidate IDs occur in only one five-minute window each, totaling 272.17 speaker-seconds; the baseline has 16 such IDs totaling 261.77 seconds. The all-ID optimal correspondence retains old IDs 0–8, then maps baseline IDs 9–19 to candidate IDs 10–20. Candidate `who-9` is the additional unmatched ID (first activity 1,559.17 s, last 1,657.28 s, 10.01 active seconds). This is consistent with persistent local fragmentation plus an extra ID, rather than a fresh set of identities every window. Extra labels are not assigned to named people: 98.91 of their 272.17 seconds overlap at least one baseline anchor, while the remaining 173.26 seconds do not. Overlap alone cannot reliably identify which human owns those extra labels.

Baseline/candidate union active time is 5,551.35 / 5,553.85 seconds, with 5,547.79 seconds common. These are unlabeled activity measures, not speech-recall scores.

## Evidence and method

- `provenance.json` records the exact SDK tag/commit, loaded library SHA-256, every downloaded model file SHA-256, raw-result fingerprints and runtime. The upstream source worktree was clean at `1a5efed67f44731b5eb5433aa9be3eebae05e269` (tag `v4.2.1`); catalog pins model `v0.3.4`.
- The SDK completed in 60.487 seconds with exit 0. The private ctypes driver calls the unmodified official C ABI, sends the entire mono 16 kHz recording in one invocation, checks response lengths and duration, frees its result, flushes normal SDK telemetry and destroys the handle. No private audio, text or embeddings were uploaded by the diagnostic.
- The installed signed runner refused the unsigned diagnostic library due to library validation. The standalone diagnostic therefore did not modify or replace the installed application. Its log includes CoreML cache-write warnings under the sandbox after the completed result; the result is complete, but equivalence to an unrestricted signed application run has not been established.
- Manager inspection of the official old/new model files found identical `who_meta.json` semantics and identical voice graph/embedding/sidecars. Encoder/query graph and weight files changed (16 files different, 8 identical; excluding the download manifest). The SDK matching algorithm is unchanged between the compared versions; graph-key/signature handling and model pin changed. No claim that the model update fixes joining follows from these changes.
- For each ID, overlapping or adjacent intervals are merged before duration/overlap calculations. `first_seconds`/`last_seconds` are extrema, `active_seconds` is the merged interval sum, and window counts use half-open five-minute windows. The final window ends at the recording duration.
- Intersections of merged intervals produce the global anchor×candidate and full baseline×candidate matrices. Maximum-weight bipartite assignment chooses one candidate for each anchor globally and separately per window. Window rows also retain the largest unconstrained per-anchor overlap, the global-mapped overlap and the number of candidate IDs with more than 0.1 seconds of overlap. Zero-activity anchors are excluded from disagreement counts. Multiple simultaneous IDs can make pairwise overlap sums exceed union activity; they are not accuracy percentages.
- Aggregate integrity checks passed: 21 result IDs, 3,456 total turns, valid interval bounds, complete 24×4 anchor-window rows and 24×4×21 overlap cells, and matched anchor time bounded by baseline anchor activity.

## Aggregate files

| File | Meaning |
| --- | --- |
| `baseline-speaker-summary.csv`, `candidate-speaker-summary.csv` | Every raw ID's first/last activity, merged active seconds, five-minute-window count and turn count |
| `anchor-overlap-seconds.csv` | Global first-four-baseline-ID × every-candidate-ID overlap matrix |
| `all-id-overlap-seconds.csv`, `all-id-optimal-mapping.csv` | Full overlap matrix and optimal baseline/candidate correspondence |
| `window-overlap-seconds.csv` | Every five-minute-window anchor/candidate overlap cell |
| `window-anchor-consistency.csv` | Global mapping versus dominant and independently optimal per-window mapping |
| `extra-id-anchor-overlap.csv` | All extra candidate IDs' overlap with each anchor and anchor union |
| `summary.json`, `provenance.json` | Aggregate findings and exact inference provenance |

Raw turn output, wire response, original document snapshot, logs, downloaded model and diagnostic scripts are retained only in the private temporary trial directory; they are not committed. Reproduction commands, using private input variables supplied outside the repository:

```sh
python3 "$TRIAL_DIR/driver.py" diarize "$SOURCE_WAV" > "$TRIAL_DIR/diarize-driver.log" 2>&1
python3 "$TRIAL_DIR/analyze.py" > "$TRIAL_DIR/analysis.log"
git -C "$UPSTREAM_DIR" rev-parse HEAD
git -C "$UPSTREAM_DIR" status --short
```

The requested conditional leading-silence stability experiment was not run because the candidate returned 21 IDs, not four. At the time of the initial diagnostic no production code, application data, speaker names or installed model files were changed. No diagnosis assigns names or human truth labels; those require independently supplied annotations.


## Lane evidence and conservative known-count implementation

A separately instrumented SDK run exported 101 lanes, of which 100 had voice embeddings. Its SDK wire output was byte-identical to the unmodified 4.2.1 run. The extra 17 one-window IDs all had embeddings and between 1.29 and 59.43 seconds of solo activity; they are not explained by the SDK's no-embedding fallback. Reference prototypes were the normalized means of the unit lane embeddings. In leave-one-five-minute-piece-out comparisons, all 83 voiced lanes of the four long-lived groups matched their own reference; the minimum cosine was 0.7287, median 0.9287, and minimum margin 0.3981. Of the 17 extras, five uniquely met the model's own 0.65 join cut, totaling 201.68 active seconds; 12 did not, totaling 70.49 seconds. These comparisons are not human identity labels. The one voiceless lane is retained as uncertain by the product.

Offline forcing ordinary average-linkage clustering to four groups merged two of the long-lived groups and retained an extra cluster. It was therefore rejected. The optional product path instead picks the requested number of recurring SDK groups by piece recurrence then active duration. It never merges those references, checks each reference lane against its own uniquely matching prototype with the whole current piece held out, and assigns other voiced lanes only for exactly one reference above the existing model cut. All other intervals are explicitly uncertain. The original SDK turns are retained independently; embeddings are temporary worker data. Automatic mode uses the SDK output as before.

The production Rust CLI ran the entire recording in both modes with the adapted SDK and v0.3.4 model. Automatic returned 21 IDs and 3,456 turns. Known four returned four reference IDs and 145 coalesced uncertain intervals totaling **75.64 seconds**; assigned activity totaled 6,042.30 speaker-seconds. The latter can exceed elapsed active time because speakers may overlap. Raw SDK output contains all 3,456 turns. The overall detected interval union was 5,553.85 seconds in both raw and derived output; the largest endpoint difference was 1.14e-13 seconds from floating-point representation. These measurements are not speech-recall or identity-accuracy scores.

A second full Rust CLI run prepended 60 seconds of silence, changing all five-minute boundaries; times were shifted back and clipped for comparison. It again returned four reference IDs, but uncertainty grew to **229.46 seconds**. After finding the global one-to-one ID mapping, compare only spans where each run has exactly one confident speaker and neither has uncertain or overlapping activity. Of 4,995.82 seconds of such speech in the unpadded run, 4,668.67 seconds (93.45%) were comparable; 4,614.35 seconds had matching IDs and **54.32 seconds (1.16%) differed**. An additional 147.92 seconds were uncertain and 93.19 seconds overlapping in the padded result. These categories describe the unpadded exclusive speech and do not include every reason for non-comparability. The largest window discrepancy was 18.60 of 231.54 comparable seconds in the first window. This test establishes a real difference between runs, not which one is correct; identity stability throughout the recording is not verified.

The pairwise overlap comparison (`product-padding-window-comparison.csv`) allows simultaneous speakers and must not be interpreted as exclusive identity accuracy. It has 79/79 active window/reference pairs matching the optimal per-window one-to-one mapping but 77/79 matching the greatest individual overlap. The independent exclusive comparison and its 24 window rows are in `product-padding-exclusive-summary.json` and `product-padding-exclusive-windows.csv`. `product-pipeline-summary.json` records aggregate union and padding results. `lane-diagnostic-*.csv/json` contain aggregate lane counts, scores, and leave-out comparisons; no embeddings, transcript text or raw intervals are committed.

Reproduction used the exact Rust worker/CLI with private paths supplied outside the repository:

```sh
cargo test --offline -p vara-engine who --lib
cargo test --offline -p vara-core --test transcription
cargo build --offline -p vara-engine --example who
who RUNTIME MODEL_DIR diarize ORIGINAL_WAV > PRIVATE_AUTO_JSON
who RUNTIME MODEL_DIR diarize ORIGINAL_WAV 4 > PRIVATE_KNOWN_JSON
who RUNTIME MODEL_DIR diarize PADDED_WAV 4 > PRIVATE_PADDED_JSON
```

The SDK patch is `scripts/patches/who-lanes.patch` (SHA-256 `48db9d8e8436ec967031d04cee3469a6b3aa32cb944e911f9cd6433d21e3791b`) applied to SDK commit `1a5efed67f44731b5eb5433aa9be3eebae05e269`. The test CLI SHA-256 was `e82995144f0b144c51eed636909039ea7ea7e3e1598597ce974e506fb0b6e63e`; unsigned tested runtime SHA-256 `5b6eccb31887b11077c3391845bd2a9b2b2cb6cf281422f008dc3b7a4183a6d7`. The Developer ID signed app was built and signature checked for review; it is not notarized or installed. Its executable SHA-256 is `b088469031e06eb78d2d1f08dd80a21d2f2fef9c63803b943b3982dfbf70bc96` and bundled signed runtime SHA-256 `add9bb9c08322209bcc56730c2d06f0309af60b739ccc44dcf451e409b2cd29e`. No installed app or personal database was modified.
