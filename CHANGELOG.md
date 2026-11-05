# Changelog

## Unreleased — 1.0.1

- Prepare the first signed updater delivery from a 1.0.0 bootstrap carrying the production public key. Native upgrade and notarization acceptance remain pending.
- Add a macOS Accessibility helper for the real Settings/check/download/install journey, with explicit installation opt-in and no permission-dialog handling.

## 1.0.0 — development baseline (not publicly released)

- Identify the Rust/Tauri app as 1.0.0, independently of the retired Swift app.
- Add localized GitHub update checks, automatic-check opt-out, signed downloads and explicit installation/restart guarded against active work and unsaved edits.
- Prepare a manually dispatched, signed macOS Apple Silicon draft-release workflow and verified updater manifest generation. Production updater keys and the GitHub updater environment are now configured; Apple CI signing credentials, notarized release artifacts and a real native upgrade remain unverified. No public release has occurred.

- Added a contributor guide, issue forms and a pull request template.
- Added the MIT license and clarified separate third-party and model notices.
- Documented the version and release policy and added a workspace version guard, fixture tests and lightweight CI check.
