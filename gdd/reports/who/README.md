# Who integration — verification, 2026-10-26

Optional local speaker labeling is available through **Find talere** in the file
transcription editor. The original ASR, source file, corrections and ordinary
dictation are preserved. No Who call runs merely from opening Vara.

## Implementation

- Shared Rust `Diarization`/`SpeakerTurn` contracts store actual overlapping
  intervals independently of the source transcript. Derived labels require one
  unambiguous voice with at least half the word/segment covered. Raw turns remain
  available when words cannot be assigned or a provider supplies no word timing.
- Official unchanged desert-ant-core SDK 4.1.0, pinned to commit
  `86b9b55fe8c61ec1c27c5b6383f545c6df1d6ceb`; model v0.3.1. Rust calls its C ABI
  in a separate registered child. Cancel/quit kills and reaps the child; normal
  completion flushes the SDK's license requests. No former Swift app returns.
- Explicit model setup with supplier attribution and license/usage disclosure.
  No audio/text is sent. The official SDK sends an installation ID and app/model/
  platform metadata to the supplier. This reporting is retained, not bypassed.
- Per-document reruns/cancellation, atomic publication, separate saved errors,
  retained previous labels on failure/zero turns, editable names and JSON export.
  A derived projection cache avoids repeating label matching on progress refresh.
- Shared UI components, native dialogs and existing reading/seek/rename controls.
  No new navigation section or speech-model/provider fallback.

## Evidence

- `runtime-evaluation.json` plus individual result JSON: actual official runtime
  runs on natural Danish audio, repeated Danish audio and synthetic two-voice
  recordings. The 329.758-second test crosses the five-minute window boundary;
  both voices keep their identity. These are functional fixtures, not a natural
  Danish multi-speaker accuracy benchmark.
- `native.json`: actual installed signed app, native import/local ASR, Who action,
  two speakers/four paragraphs, name change and persistence after restart. The
  short Danish empty-result error preserves the earlier correction.
- `browser.json`: 23 existing dictation groups and 16 transcription groups pass,
  including Who setup, license/usage disclosure, speaker intervals/name change,
  failure/cancellation, immutable original/corrections and narrow reflow.
- `workspace-tests.txt`: 49 workspace tests pass. The optional model-weight
  dictation regression has its earlier independent evidence; no existing
  dictation API or shortcut/insertion implementation changed in this feature.
- `cancel.txt`: the exact native worker is cancelled and reaped before returning
  output. Native runtime and app signatures verified; no weights bundled.
- Current GDD schema, semantics, record integrity and visualization reports pass.

## Limits

The 10.312-second natural Danish fixture produced zero Who turns. Repeating it to
41.248 seconds produced one voice. Empty output is surfaced honestly and does
not erase text or earlier labels. Short, quiet, noisy or overlapping speech can
be missed or confused; this integration does not promise perfect diarization.
The SDK supports four voices per five-minute piece and matches across pieces.
Speaker IDs mean a voice within one document, not an identified person.

Who on Mac requires macOS 15+. Its first Core ML preparation can take minutes;
subsequent runs reuse system caches. The UI gives an indeterminate stage and
cancellation, without fabricated percentages. The app's other functionality keeps
its existing OS floor. Linux/Windows Rust/C ABI boundaries are implemented, but
their official SDK/LiteRT packaging and native runtime acceptance remain open.
No forced alignment or automatic timing of edited text was added.

Sources: [official SDK](https://github.com/Desert-Ant-Labs/desert-ant-core),
[Who model card](https://huggingface.co/desert-ant-labs/who),
[SDK documentation](https://desertant.com/docs/who/),
[license](https://license.desertant.com/1.0).
