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

## Remaining delivery gate

The Mac is locked. Native file-picker/save-dialog interaction and playback in
macOS WebKit still need an unlocked-Mac walkthrough. Browser playback and native
file processing pass separately; these are not a claim of that final walkthrough.
Cloud rich-timing protocols are contract-tested, not live-account acceptance.
Windows/Linux native acceptance and forced alignment remain explicit limitations.

Work is on `codex/file-transcription`, based on main at `7aba54d`. Cancelled,
pre-existing untracked dictionary research remains untouched and unpublished.
