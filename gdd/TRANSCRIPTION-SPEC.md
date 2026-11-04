# Transcription in Vara

Date: 2026-10-25. Status: implemented; final native macOS test passed 2026-10-26.
Evidence and model-dependent limitations: [report](reports/transcription/README.md).

The user requested a separate transcription tab for content containing audio and
asked for the feature list and word-level timestamps to be documented. This note
collects the direction and proposed first delivery scope. Concrete formats, model
support, storage and test results are now in the [implementation report](reports/transcription/README.md).
The table preserves the product requirements; the model determines the timing and
speaker data available.

## Purpose and interface

A **Transcription** tab for audio and video files, separate from quick
hold-to-dictate. The primary flow:

**Add file → transcribe → read and listen → edit → copy or export.**

Follow the selected Warm precision design in DESIGN.md and reuse shared components.
Show a few relevant choices at import and readable text after transcription. The
internal division into words must not turn the editor into a table of individual words.

## Features

| Feature | Desired behavior |
| --- | --- |
| Audio and video import | Drop files or choose them from the computer. Extract audio from video. Show an understandable error for missing audio or an unreadable format. “Anything with audio” is the product direction; actual supported formats must be verified. |
| Multiple files | Add multiple files to a queue with individual statuses and results. |
| Model selection | Reuse Vara's models and key management. Local by default; cloud is selected deliberately, with a clear audio destination. No automatic cloud fallback. |
| Language | Automatic recognition with the option of an explicit language, e.g. Danish, where the model supports it. |
| Progress and control | Show the file's current stage; allow cancellation and retry after failure. Do not invent percentages or time remaining. Cancelled work must not publish a result later. |
| Text editor | Read and edit text with automatic saving. Preserve the original transcript separately. |
| Playback | Play the source with a time position and adjustable speed. Jump from text to audio with the precision actually available. |
| Speaker labels | Show and rename Speaker 1, Speaker 2, etc. when the model supplies usable speaker data. Local support must be quality-tested before it is promised. |
| Optional text AI | Reuse Vara's text models for cleanup, summaries and custom instructions. Save results separately from the timed original. |
| Copy and export | Copy text; export TXT, Markdown and structured JSON. SRT/VTT requires verified timing and suitable subtitle segmentation. |
| Library | Save transcripts under file/title, with search, rename and delete. Imported files must not be grouped under an arbitrary active app. |
| Source safety | Never change or delete the user's original files. Clean up temporary working files. If an external source is moved or deleted, preserve the text and explain why playback is unavailable. |

## Word-level data

The contract must support **start and end times per word** and an optional speaker
reference from the outset. A word timestamp is an estimated position in audio, not
a guarantee of perfect precision.

This should enable:

- Clicking a word to play from its start.
- Highlighting the current word during playback.
- Selecting a passage and playing its time interval.
- Grouping words into readable paragraphs and suitable subtitle lines.
- Exporting words, timestamps and any speakers in JSON.

### Proposed contract boundary

- A document has source information, model/provenance, original text and ordered
  segments. The original result is preserved independently of edits and AI output.
- Segments may contain words with stable IDs, text, start/end in seconds relative
  to the source start, and an optional speaker reference. Punctuation may have no
  independent audio interval; tokenization must not be assumed identical across models.
- Timing support is explicit: none, segment-level or word-level. Its origin is
  also specified: speech recognition or subsequent alignment.
- Missing timing data is absent, not zero or a calculated guess. Models that return
  only text or segments must remain usable with visible limitations.
- All times must be finite, non-negative and have end ≥ start, within the source
  duration with a documented tolerance. Speakers may overlap in time.
- Edits and word IDs must preserve the connection to the original. Changed or new
  words get **requires timing adjustment** status; old timestamps must not be
  presented as verified alignment of edited text.
- Timed export of edited text requires valid alignment or an explicitly displayed
  limitation. TXT/Markdown remain available without timing data.
- AI summaries and rewrites are separate text results without inherited word timestamps.

Separate **forced alignment** can be investigated to place known text in audio.
It must be tested in Danish and on the selected models; it has not yet been selected
or implemented. Robustness and measured quality take priority over promising word-level
support for all models.

## First delivery and subsequent work

Proposed first delivery: import, queue, existing model/language selection, status,
cancellation/retry, original + edited text, library and text/JSON export. The
word-level contract must be included from the beginning. Playback, clicking/highlighting
and SRT/VTT are included when the selected runtime/model combinations provide verified
timing data. Specific compatibility limits must appear in the UI and documentation.

Who was selected for optional local speaker diarization at the user's request on
2026-10-26. “Find speakers” in the editor runs the adapted SDK runtime via a C
interface from Rust in a separate, cancellable process. No automatic download,
cloud fallback or change to quick dictation.

Original ASR text, word IDs and times remain unchanged. Who turns are saved separately
with model/version and may overlap. A derived reading view assigns a speaker to a
word/segment only when one clearly overlapping voice covers at least half of it.
Ambiguous words have no selected Who speaker. Without word times, actual audio
intervals are displayed without inventing a distribution of text. Edited text does
not receive new timing or speaker data. JSON contains raw ASR, Who turns and edits
separately. Speaker names can be renamed in the existing component.

Automatic is the default. The user can specify a known count from 1 to 16; this
requires enough recurring SDK groups. The strongest are selected by number of audio
pieces and active duration and kept as separate references. A reference audio lane
must uniquely match its own reference with the entire current piece omitted from the
prototypes; otherwise it is marked uncertain. Other lanes match only with exactly
one cosine match above the model's own threshold; unclear lanes, including those
without a voice vector, remain uncertain and are not assigned words. JSON preserves
raw SDK intervals, derived intervals, selected count, method and threshold. For a
known count a second check runs with 60 seconds of prepended silence. An unambiguous
global one-to-one mapping is required based only on unanimous, certain intervals.
A primary interval gets its certain speakers only when the complete speaker sets of
both runs agree and none is uncertain; other primary activity is uncertain. Activity
found only in the check run is not added. Therefore the number of certain IDs can
be lower than the selected count. The method saves the check offset as a versioned
part of the method. Temporary voice vectors are not saved in the document.

The model is explicitly downloaded in the Who dialog, which shows the vendor, license
and SDK usage counting before download/use. Audio and text are processed on-device;
the SDK sends an installation ID and model/app/platform metadata to the vendor's
licensing system. SDK reporting is retained. Models are not bundled with the app.
Core ML on Mac requires macOS 15 or later; the app's other features retain their
existing OS limit. Linux/Windows use the same Rust contract and SDK C interface;
their native runtime packaging must be verified on the target platforms.

Errors, no speakers found and cancellation preserve text, edits and any last
successful diarization. No speakers is not proof that no one speaks: a natural
10-second Danish clip produced no Who turns, while the same clip repeated to 41
seconds produced one voice. Two synthetic voices were recognized and matched across
a five-minute boundary. This is a functional test, not a quality guarantee for
natural Danish conversations.

Links such as YouTube and recording meeting/system audio are subsequent input types.
Optional text AI is part of the feature list; its precise place in the delivery
sequence is determined during implementation.

## Questions before implementation

- Which local/cloud models provide real word or segment timing through the current
  adapters, and is separate alignment necessary?
- Should playback use a reference to the original file or a user-selected copy in
  Vara? Rules for deleting dictation audio must not automatically be reused as rules
  for imported files and their possible playback copies.
- Specific file size/duration, streaming/chunking and recovery limits must be set
  based on runtimes and providers without changing the source.

## First-version verification

Test Danish audio, video with audio, a file without audio, multiple queued items,
cancellation, error/retry and restart. Preserve the original file and text in every
flow. Check word clicks and highlighting against actual playback, edits' alignment
status, speaker overlap and export. The UI must also work with a model without timing
data and a moved source file. Preserve existing dictation, AI settings and history.

Before implementation, incorporate the specific contracts in the active GDD model
with associated evidence. This note does not itself change existing contracts.

Cancelling an unfinished file job removes its private working audio; the original
file remains untouched. On retry after failed speech recognition, a valid local audio
copy can be reused even if the external original has moved.

## Interface language and speaker names

App-owned file transcription labels, dialogs and accessible names are displayed in
Danish or English independently of the spoken language. Changing language updates
visible labels without starting work or replacing an active editor or player. Words,
timing data, paths, AI results and user-selected speaker names do not change. New
automatically generated speaker names have explicit ordinal provenance, allowing the
UI to show Taler 1 or Speaker 1. Old saved names are never interpreted heuristically;
renaming removes that name's provenance. Provenance is stored in the document but
excluded from export if the existing export format must be preserved. Uncertain
passages do not count as a person.

## Shared file-job boundary — 2026-11-04

On macOS, desktop and the local backend share one `vara-app::Files` owner per
profile. The owner locks the profile, owns the queue, document list and publication,
and uses the existing file SQLite database and media directory. Desktop adapts the
file picker, clipboard, export dialog, links and UI events; the backend owns the
resource path and credential lookup. Windows and other platforms retain the existing
in-process adapter; their local transport has not been delivered. The existing document
format and editing, AI, Who, playback and export remain intact. A separate CLI
database or competing queue is not allowed.

The shared Rust boundary offers `submit`, `status`, `result` and `cancel`. `submit`
accepts one path, optional speech engine ID, optional spoken language and optional
request ID; the receipt's job ID is the document ID. Engine and language settings
are saved on the document upon acceptance without changing global settings. `status`
returns the document's actual stage; `result` requires Ready and provides access to
the full document and its existing export methods. Explicit cancellation uses the
existing cancellation mechanism and cannot publish a late result. Desktop batches
use the same queue and storage, without a request ID by default.

A request ID is scoped to the profile and associated document's lifetime. It must be
1–200 UTF-8 bytes, contain at least one non-whitespace character, and can be used for
only one file job. Input identity consists of the absolute path, requested engine ID
(including omission) and normalized spoken language (empty means automatic). It
identifies the path, not file contents or symlink target. The same ID and identity
return the original receipt, even after the source has been removed; changed
parameters return `file.request_conflict`. Acceptance and request registration are
committed atomically before receipt/publication. Deleting the document also deletes
the registration; the ID can then be reused. A client submitting a relative path
over IPC must make it absolute in its own working directory before sending it to
the backend.

On restart, all interrupted non-terminal jobs are marked Failed with the versioned
`file.interrupted` reason; they are not automatically rerun. A connection or waiting
client does not own an accepted job's lifetime. The Mac transport is a versioned
Unix-domain socket (internal version 2, distinct from CLI JSON version 1), restricted
to the same OS user, with one locked backend per profile and automatic startup in a
separate process group so the client's Ctrl-C does not interrupt the backend.
Different internal versions are rejected so an old backend cannot ignore a Who request.
A version conflict does not automatically stop an older backend or its active jobs.
Desktop tracks instance and revision and reloads on changes; the latest error is
shown at most once per instance and revision. Active AI and Who, and Who downloads,
can be viewed and cancelled after GUI reconnection. Closing the GUI leaves accepted
work running; the backend normally stops after 60 seconds of inactivity with no
clients or jobs. Framing is time-limited, but waiting for AI/Who is not a job deadline.
The backend reads the profile's saved settings and keeps keys with the trusted worker;
it does not expose a general invoke or secret boundary.

## Agent CLI v1 — contract (#27–28)

The packaged `vara` launcher forwards to the signed app in `/Applications`; the
internal desktop entry point `cli` is for development only. Installation is described
in `CLI.md`. Commands: `transcribe FILE [--model ID] [--language CODE] [--diarize] [--speakers 1–16]
[--original] [--request-id ID] [--no-wait] [--timeout SECONDS] [--format json|txt|md|srt|vtt]
[--output FILE] [--overwrite]`, `job status|wait|result|cancel ID` (wait/result
accept format/output/overwrite/original, wait also timeout), `models [--json]`,
`--help`, `--version`. No interaction. An omitted model uses saved `speech_engine`;
omitted language is automatic; Redux supports only automatic. Input paths are made
absolute in the client's current directory; their existing parent directory is
resolved without requiring that the file still exist on retry. Request ID compares
raw choices including an omitted model before defaults are resolved, and the first
job retains its engine. No global settings are changed.

JSON on stdout is one object with `schema_version: 1`, `ok`, and on success
`job_id`, `document_id`, `status` and optionally `document`; models uses `models`,
`default_speech_engine` and `strict_local`; cloud readiness `null` means not checked.
`timing_capability` describes the adapter's capability (`word_or_segment`, `segment`,
`word_requested`, `none`), while the document's `transcript.timing` and
`timing_source` describe actual data. Errors have `error: {code, message?}` with safe,
stable machine codes and catalogued numeric reasons. Exit: 0 success/receipt,
2 invalid usage/input, 3 unavailable model/key/runtime/privacy, 4 job failed or
cancelled, 5 transport/version, 6 export, 7 timeout/interruption. Help/version
have human-readable output. Only an explicitly non-JSON completed export without
`--output` writes export text to stdout. `--output` produces a JSON receipt and
requires `--overwrite` for an existing destination. No sources known to the document
owner, working audio or profile-internal files may be overwritten, including through
links. The owner's lock covers source checking and atomic publication. A relative
output path is resolved in the client's working directory. RPC documents and JSON
exports use the same safe projection of legacy endpoints and raw errors without
changing the stored document or transcript text. Export errors do not change the document.

By default, the client waits without a job deadline; `--no-wait` returns a durable
receipt. The accepted job ID is immediately written to stderr. Before submit the
client always writes a request ID to stderr: either the user's or a new UUID for
that call. `--no-wait` with `--output` or `--overwrite` is an input error; use
`job result` for later file export. `--no-wait --format` returns a JSON receipt.
If acceptance is unknown on timeout/interruption, error JSON includes this `request_id`;
repeat the same arguments with `--request-id` to recover the job. Separate calls
without an explicit ID are separate jobs. `--no-wait` returns status `accepted`.
Timeout, signal and closed pipe stop only client waiting; cancel is explicit. Timing
and actual words/segments come from the document without fabricated word times.
Backend errors are catalogued and no keys, endpoints or provider error text are returned.

`--diarize` saves a required Who step with the document; `--speakers` requires
`--diarize` and must be 1–16. The choices are part of request identity. Who uses the
existing local runtime and setup without download or license acceptance. Missing
speaker intervals produce `file.who_unclear`; a known speaker count without sufficient
evidence across pieces produces `who.known_count`. ASR remains `Ready` after ASR
is saved, while the CLI shows `processing` until Who finishes. The receipt separately
contains `asr_stage` and `who`. Who failure or cancellation yields a nonzero exit,
status `who_failed`/`who_cancelled` and document ID; text, audio and existing data
are preserved. Pending/running Who is marked interrupted after backend restart and
is not rerun automatically. Cancel covers both phases, including the transition.
If ASR fails before Who, Who is saved as `skipped`. Explicit ASR retry resets Who to
pending; explicit Who retry updates the same phase and selected speaker count. If the
Who result or phase completion status cannot be saved, the active owner reports
`file.result_unsaved` without losing saved ASR.

`--original` selects original ASR text and timing for export; otherwise saved edits
are used. Unsubmitted GUI drafts are unavailable. SRT/VTT requires real timing and
rejects edited text without suitable timing. JSON separately includes raw ASR, Who
intervals and `speaker_transcript` derived from the same core annotation as the UI;
`selected_text` and `original_selected` indicate the export choice. Uncertainty is
not a new speaker, and untimed words do not receive fabricated times.
