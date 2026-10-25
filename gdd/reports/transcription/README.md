# File transcription — verification, 2026-10-25

Scope: TRANSCRIPTION-SPEC.md and the user-approved import/editor mockups, with
shared DESIGN.md primitives and preserved dictation. Evidence reflects this branch;
model accuracy and untested platforms are not inferred from a passing UI fixture.

## Requirement evidence

| Requirement | Implementation and observed evidence |
| --- | --- |
| Audio/video import, original safety | Rust Symphonia normalization plus optional installed FFmpeg. `formats.json`: actual MP3, FLAC, M4A, Ogg, Opus and MP4 speech runs pass; no-audio video fails usefully. WAV tests verify stereo conversion, sample rate and unchanged source bytes. Source overwrite is rejected, including aliases on export. |
| Batch, cancel, retry and restart | Separate SQLite queue in the actual Tauri host. `native-host.json`: cancelled WAV, ready MP4, failed no-audio video; cancellation removes private audio. Retry succeeds from retained PCM even when the external original has moved. Interrupted job becomes retryable failed state at restart without automatic replay. Browser journeys cover retry and per-file controls. |
| Local/cloud model and language | Shared model picker/readiness/key components. Explicit start freezes file model/language. Strict-local checked in the host. UI tests verify local default and cloud destination before processing. |
| Word/segment/no-timing contracts | `real-redux.json` and `real-video.json` contain actual model output with validated word intervals. WAV: 10.312s, 31 words, about 1.24s inference on this Mac. This is one functional fixture, not a Danish accuracy benchmark. |
| Reading, playback and word navigation | Production UI browser tests play actual HTML audio, seek/highlight words and exercise roving keyboard navigation. The native host produces real playable PCM and a 128-bin measured waveform. Playback assets are scoped to the profile's transcription-copy directory. |
| Corrections, original and autosave | Core tests preserve both versions across restart; browser tests verify autosave, original switching, visible failed-save recovery, recovery-copy/export of the current draft, and native undo retained across autosave. Corrected text never borrows verified original timing. |
| Optional AI and speakers | Separate text-model/prompt/result pane, explicit locality and cancellation; backend checks revision before publishing AI output. Original and corrections remain intact. Speaker names are editable when a provider returns them. Live cloud diarization quality is not claimed. |
| Copy and export | TXT, Markdown, full-document JSON, SRT and VTT; explicit text version. Core/browser tests reject subtitle export from edited or untimed text and accept timed original. AI output has no subtitle timings. Native OS save dialog chooses the destination. |
| Library and deletion | Search, rename and direct deletion, with confirmation explaining document/private-copy deletion. Original media remains untouched. Playback closes before deletion; unknown/path-like IDs cannot address files. |
| Existing dictation | Existing 23 UI regression groups pass. Its session/store, recording HUD, shortcuts and insertion pipeline retain their interfaces. File work uses a separate engine instance/mutex; file playback pauses when recording starts and cannot be started over a microphone recording. |
| Design and responsive UI | `import.png` and `editor.png` render actual production UI. New journey tests pass at 1586, 1280, 1040, 720 and 320px without horizontal overflow. Canonical olive/paper palette, serif reading, shared buttons/fields/dialogs and model controls are reused. |
| GDD | Model schema, semantics, record integrity and regenerated visualization pass. Contracts include separate file documents/words, file lifecycle, recognition and document operations. Earlier dictation contracts are retained. |

## Current capability limits

- Redux returns actual word timings locally. Whisper currently returns segment
  timing. Groq's rich file route requests word timing; ElevenLabs requests words
  and speakers. These new rich cloud requests are checked against official docs
  and parsed with contract tests, not tested against live user accounts here.
  OpenAI's existing selected route may return text only; the UI reports that.
- Independent provider chunks do not establish persistent speaker identity.
  ElevenLabs speaker labels are scoped to each five-minute request; the UI says
  this explicitly. No Who model/runtime/license integration was added.
- Files are bounded to eight hours of normalized audio. Whisper/cloud process
  bounded chunks; Redux uses streaming VAD. Supported containers still depend on
  their actual audio codec. Opus here used the optional local FFmpeg fallback.
- AI model context/output limits remain real. Failure preserves the transcript;
  no silent truncation, provider switch or invented AI result is performed.
- Word times are estimates. The fixture contains recognition errors. No forced
  alignment model or automatic realignment of corrected text is implemented;
  corrected text remains marked as requiring new alignment.
- Private playback WAVs are retained until document deletion, disclosed before
  import. External originals remain untouched. User text-retention preferences
  for dictation do not silently delete file documents.
- Windows/Linux runtime acceptance remains open as in the existing desktop app.
  Rust media/core contracts are platform neutral; native OS behavior needs target
  testing. No mobile shell, URL import or meeting/system-audio capture is added.

Protocol sources: [Groq speech docs](https://console.groq.com/docs/speech-to-text),
[ElevenLabs create transcript](https://elevenlabs.io/docs/api-reference/speech-to-text/convert).
Symphonia decoder example: https://github.com/pdeljanov/Symphonia/tree/v0.5.5.

## Final checks

- `cargo fmt --all -- --check`: passed.
- Optimized workspace suite: 46 tests passed; optional real-weight test then ran
  explicitly and passed (`dictation-runtime.txt`).
- Production browser UI: 23 dictation and 13 transcription groups passed
  (`browser-checks.json`).
- Current GDD schema, semantics and record integrity: passed (`model.json`).
  The generated graph is validated separately (`visualization.json`).
- Native visual interaction remains pending a manual walkthrough. Native picker/save
  dialogs and playback in macOS WebKit have not been manually exercised. The real
  native file worker itself passes with a disposable profile (`native-host.json`).

## Installed candidate

The final signed production app is installed and running from
`/Applications/Vara.app`; its binary hash, signature/resource checks and build
cleanup are recorded in `packaging.json`. The development harness is excluded.
The existing personal profile, models and credential store were retained.
3.43 GiB of build intermediates were removed after installation. This is a local
signed build, not a notarized public release. No push or merge was performed.

Native UI gate remains open: macOS is now unlocked, but computer use cannot
resolve the installed Vara app (details in `native-ui-attempt.json`).
The installed candidate is ready for file-pick → local transcription → playback
→ edit → copy/export, plus a normal dictation smoke test in another application.
