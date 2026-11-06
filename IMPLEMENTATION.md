# Vara implementation status

The active implementation is the Rust/Tauri workspace on `main`. The retired
Swift app, Xcode project and Swift-specific release scripts were removed on
2026-10-25. Their history remains in Git.

The shared Rust domain and engine are independent of the desktop shell. Native
platform services provide microphone capture, credential storage, shortcuts,
focus checks and text insertion. See [README.md](README.md) for the module map
and [Rust bindings](gdd/RUST-BINDINGS.md) for the GDD contracts.

Implemented desktop flows include local Parakeet Redux/Whisper speech, local
Qwen text processing, optional cloud adapters, app-specific prompts, history,
recovery, cancellation, model installation and OS credential storage. Settings
follow the approved three-page layout: dictation, models/keys, and privacy.

The user has confirmed global shortcut activation and insertion into TextEdit
and Codex, successful OpenAI usage, local Parakeet + Qwen usage, and the revised
settings UI. These are observed personal workflows, not broad release acceptance.
Detailed test evidence is retained under [gdd/reports](gdd/reports), including
settings-v2, credential hardening, local app artwork and native insertion reports.

## Remaining limits

- macOS is the development/test platform. Windows/Linux native focus, insertion,
  floating HUD and physical shortcut integration still require completion and
  validation on those systems.
- Mobile can reuse the Rust engine, but mobile hosts and UI have not been built.
- The signed, notarized 1.0.2 macOS Apple Silicon release is public. The real
  in-app 1.0.1→1.0.2 check, signed download, installation and GUI restart passed,
  including automatic retirement of the old independent CLI owner before any
  new CLI command. Data, models, credential presence and microphone permission
  were preserved. This resolves the 1.0.1 manual-handoff limitation in
  [#37](https://github.com/mikkelkrogsholm/vara/issues/37) and completes the
  macOS Apple Silicon [#34 acceptance](https://github.com/mikkelkrogsholm/vara/issues/34).
  Intel Mac remains unverified; the CI Apple signing/notarization secrets are
  unconfigured, and the CI release workflow has not run. See
  [RELEASING.md](RELEASING.md) for evidence and platform limits.
- Codex CLI integration is unavailable pending a verified tools-off contract;
  optional Claude/Gemini CLI text paths remain capability/version gated.
- Model accuracy varies; no broad Danish accuracy acceptance is claimed.

The dictionary investigation was stopped at the user's request. It is not an
implemented feature or active delivery task.

## File transcription — 2026-10-25

The Transcription tab adds a separate file queue/library, local media decoding,
real Redux word times, segment/plain-text fallbacks, an editable transcript with
preserved original, playback, copying/export and separate optional text AI. Its
own SQLite store and explicitly retained playback copies do not enter dictation
history or its captured-audio retention. See gdd/reports/transcription/README.md
for current verification and conditional model capabilities.


## Gemini speech — 2026-11-06

The optional `gemini.speech` engine uses the existing Gemini API key for verbatim
dictation and rich file transcription, separately from Gemini text/CLI settings.
It shares existing cloud policy, normalized chunks, queue, result/export and
English/Danish components. The default `gemini-3.5-transcribe` was verified on
one public Danish clip through the direct speech path; see the
[frozen cloud baseline](gdd/reports/epic42/47-cloud-run.md). The same Danish clip
and a public four-speaker fixture returned real word timing and provider speaker
IDs through saved-key production CLI and native desktop adapter paths; see
[automated product acceptance](gdd/reports/epic42/48-product-acceptance.md).
This bounded evidence does not establish broad Danish accuracy, cross-chunk
speaker identity, native OS insertion or UI behavior. Existing local-first
selections and custom saved models remain unchanged.
