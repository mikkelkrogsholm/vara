# Transcription implementation — 2026-10-25

Objective: implement TRANSCRIPTION-SPEC.md and approved import/editor mockups
using shared DESIGN.md components while preserving existing dictation.

## Implemented and verified

- Separate Rust document/timing/export contracts and SQLite file library.
- Rust audio/video normalization, optional installed FFmpeg, bounded chunks and
  real streaming Redux word times. Original files remain untouched.
- Persisted file queue, individual cancellation/retry, restart recovery and
  playback-copy reuse after an external source moves.
- Import, search, rename, original/corrections, autosave with retained browser
  undo, recoverable failed-save drafts, optional AI and conditional speaker names.
- Playback, word seeking/highlighting, passage playback, speed and measured
  waveform. File playback pauses during dictation recording.
- TXT/Markdown/JSON and timed-original SRT/VTT, with explicit version selection.
- Shared model/key/readiness controls and explicit local/cloud destinations.
- 46 workspace tests, real Redux cancellation/next-session test, 23 existing
  dictation UI groups and 13 file-transcription UI groups pass.
- Real native host exercises import, batch, cancellation, retry and restart.
  Actual speech files in WAV/MP3/FLAC/M4A/Ogg/Opus/MP4 were processed.
- GDD contracts, provenance and design rules reconciled. See detailed evidence in
  [the verification report](reports/transcription/README.md).

## Native delivery verified — 2026-10-26

Computer-use lookup was repaired by removing obsolete Launch Services entries
and restarting the stale computer-use helper. No permission or safety settings
were changed. Fresh `cua.getApp("dk.vara.desktop")` now resolves the installed app.

The actual native walkthrough passed: file picker, local Redux inference (31
words / 10.312s), word seek/playback to end, corrections/autosave, preserved
original, clipboard paste into a new TextEdit document, TXT save dialog and exact
export contents, normal quit/relaunch and retained document/correction. A new
real user dictation during this test was observed with delivery `inserted` in
Codex. Details: `reports/transcription/native-final.json`.

Cloud rich-timing protocols are contract-tested, not live-account acceptance.
Windows/Linux acceptance and forced alignment remain open. Optional local Who
integration was added on 2026-10-26; see [Who verification](reports/who/README.md)
for actual runtime tests and Danish-context limitations.

Work is on `codex/file-transcription`, based on main at `7aba54d`. Cancelled,
pre-existing untracked dictionary research remains untouched and unpublished.

## Interface language — 2026-11-04

The file workspace uses the shared Danish/English catalog for import, jobs,
editor, player, AI, Who and export. Language changes update existing controls
and open dialogs in place. Newly generated speaker labels carry explicit ordinal
provenance; legacy labels and renamed speakers remain literal. The provenance
survives the file store but is omitted from the existing JSON export shape.

Verification uses sanitised in-memory UI fixtures and the existing local browser:
`desktop/tests/transcription-i18n.playwright.cjs` exercises both languages,
active AI/Who jobs, actual audio instance and editor selection retention, open
rename/Who/export dialogs, every export option, 1280×820 views and 720×560 views
with computed text sizes doubled. `desktop/tests/transcription.playwright.js`
continues to cover the existing transcription journey. Core store/export and
host speaker-provenance tests cover the persisted representation and legacy
names. These browser fixtures do not establish native Windows/Linux acceptance or
exercise real inference, cloud providers, downloads or user profiles. Native and
provider error localization remains separate work.

## Shared file jobs — 2026-11-04

`vara-app` now owns the existing desktop document list, persisted queue and worker.
The desktop adapter retains platform interactions. A per-profile lock prevents a
second queue owner; one-file request IDs are stored transactionally with their
documents and are removed on explicit document deletion. The typed Rust API
provides submit/status/result/cancel. No Who submit option is implemented.
See TRANSCRIPTION-SPEC.md for path, interruption and idempotency semantics.


## Internal local backend — 2026-11-04 (#26)

The Mac desktop executable has an internal worker entry after the existing Who
and credential worker entries and before Tauri starts. It uses the same `Files`
queue owner, profile, models, credential boundary and persisted documents. The
Rust `local_backend::call` helper uses an explicitly supplied executable and
profile, starts the internal worker if needed, and discards its standard streams.
`default_profile()` uses `VARA_DATA_DIR` or the same `dirs::data_dir()` plus
`dk.vara.desktop` as Tauri. An app-bundle worker resolves resources from
`Contents/Resources`; only test-harness builds accept `VARA_RESOURCE_DIR` as
an explicit test override. Who is never located through PATH.

The Mac adapter places a profile-hashed Unix socket and ownership lock in a
private per-user temporary directory, checks effective peer UID, and holds the
lock for the server lifetime. Owned profile directories are tightened to 0700;
symlinked profile endpoints, insecure sockets or lock files are rejected. A
stale socket is removed only while holding its lock and after a failed connection.
Each connection has one length-prefixed version-1 request, bounded to 32 MiB;
responses are bounded to 128 MiB. Frame transfer has a 30-second deadline. AI,
Who and Who-download calls may wait longer for the first response byte, after
which the frame deadline applies; cancellation uses another connection. The
worker starts in a separate process group, so the starter's Ctrl-C does not stop
it. A dropped connection does not cancel accepted work. The worker exits normally
after 60 seconds with no client activity and no file/AI/Who work; a later client starts it
again. Test-harness builds alone can shorten this idle period and substitute a
no-secret test host. Interrupted persisted work becomes failed, not replayed.

The internal protocol is a typed allowlist of file operations and model readiness;
it does not expose arbitrary invocation, secrets, shell or network listeners.
Settings are read afresh from SQLite without initializing dictation history. Local
model readiness does not download models; remote readiness is unknown rather than
inferred from a key. The response carries a server instance UUID and monotonic
revision for inexpensive GUI polling. A new instance invalidates the old revision.
Responses also include the latest safe asynchronous Files error with its own
revision. The Mac GUI polls instance and revision and emits `vara:files` on
changes, and emits `vara:file-error` once per instance and notice revision; a
later non-error revision does not erase that notice. Older overlapping list
responses are discarded. AI, Who and Who-download busy state remains observable
after GUI reconnection.
Engine endpoint URLs and uncatalogued raw error text are excluded from replies;
known runtime errors retain catalogued numeric parameters and causes.

Errors in the response have a structured `code` and optional safe `message`.
The backend-specific codes (`backend.permissions`, `backend.peer_denied`,
`backend.unavailable`, `backend.already_running`, `backend.start_failed`,
`backend.transport`, `backend.timeout`, `backend.response_invalid`,
`backend.malformed`, `backend.frame_limit`,
`backend.version_mismatch`, `backend.profile_unavailable`,
`backend.operation_failed`, and `backend.unsupported_platform`) are stable wire
codes. The Mac adapter maps them to catalogued Danish/English
UserMessages without exposing identifiers; known application messages retain safe
parameters and causes. The helper can also return wire codes before a response
exists.

**Current boundary:** the Mac GUI is a client of this backend and shares its
queue, documents and profile media with other local clients. Native dialogs,
clipboard, links, playback scope and draft-aware export remain in the GUI;
closing the GUI does not cancel accepted backend jobs. Existing Windows and
other non-Mac GUI file handling remains in process until a native transport
adapter exists. The desktop executable now exposes the #27 `cli` subcommand;
the packaged launcher is still #29. An isolated Tauri adapter fixture
and socket/process checks used synthetic data on the development Mac, including
reconnection, long AI waiting and cancellation, asset scope and GUI exit. A
different OS user, signed installation, actual model inference, credential
prompts, and Windows/Linux transport were not verified.


## Agent CLI (#27)

The desktop executable dispatches `cli` after internal workers and before Tauri.
The CLI uses the shared backend and document exporter. A test-harness-only delay
can hold submit responses or status calls; it does not simulate inference. Process
tests use synthetic audio and a separately seeded ready-document fixture to test
export and timing. They do not exercise real local inference or cloud providers.
Windows/Linux local transport and the signed launcher remain unverified.

The process regression checks the executable's test-harness-only version marker
before creating a profile or starting the backend; it refuses normal builds.
Build and run the fixture using the existing release cache and no network:

```sh
DEVELOPER_DIR=/Library/Developer/CommandLineTools CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo build --release --locked --offline -p vara-desktop --features test-harness
python3 scripts/tests/agent_cli.py target/release/vara-desktop
```
