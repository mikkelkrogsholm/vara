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

The requested conditional leading-silence stability experiment was not run because the candidate returned 21 IDs, not four. No production code, application data, speaker names or installed model files were changed. No diagnosis assigns names or human truth labels; those require independently supplied annotations. The next implementation decision remains with the manager.
