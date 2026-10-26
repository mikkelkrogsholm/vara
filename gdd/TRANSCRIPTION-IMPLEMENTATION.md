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
Windows/Linux acceptance, forced alignment and local diarization/Who remain
explicit scope/capability limits; no new Who integration is claimed.

Work is on `codex/file-transcription`, based on main at `7aba54d`. Cancelled,
pre-existing untracked dictionary research remains untouched and unpublished.
