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
- The signed macOS app is not notarized. Public distribution and automatic updates
  require separate work.
- Codex CLI integration is unavailable pending a verified tools-off contract;
  optional Claude/Gemini CLI text paths remain capability/version gated.
- Model accuracy varies; no broad Danish accuracy acceptance is claimed.

The dictionary investigation was stopped at the user's request. It is not an
implemented feature or active delivery task.
