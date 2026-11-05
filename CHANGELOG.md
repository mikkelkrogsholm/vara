# Changelog

## 1.0.2 — 2026-11-05

- Replace an authenticated older idle file-service owner after an app upgrade, so GUI and CLI use the running build. Preserve busy accepted jobs and defer handoff until idle; never replace a newer or unverified owner. The private protocol remains v2 and Ping now reports its build version.
- Publish the Developer ID signed, notarized macOS Apple Silicon app. Verify the real 1.0.1→1.0.2 in-app check, signed download, installation and restart, including automatic retirement of the previous idle file owner before any new CLI command, with retained data, models, credential presence and microphone permission.
- Reject incompatible macOS updater archives before manifest generation, including top-level AppleDouble files that prevent Tauri installation. Add signed archive regression fixtures and an extraction diagnostic.

## 1.0.1 — 2026-11-05

- Release the Developer ID signed and notarized macOS Apple Silicon app. Verify the real in-app 1.0.0→1.0.1 check, signed download, installation and GUI restart with retained data, models, credential presence and microphone permission.
- Package the updater archive without macOS AppleDouble metadata. Automatic retirement of the previous independent CLI owner was unresolved in this release; the test required a manual idle-owner handoff, resolved by the verified 1.0.2 upgrade ([#37](https://github.com/mikkelkrogsholm/vara/issues/37)).
- Show the running version in Settings and a shared update action on every main view, preserving live transcription drafts.
- Pass the production public key to the release bundler as well as the runtime through a generated configuration overlay.
- Add a macOS Accessibility helper for the real Settings/check/download/install journey, with explicit installation opt-in and no permission-dialog handling.

## 1.0.0 — development baseline (not publicly released)

- Identify the Rust/Tauri app as 1.0.0, independently of the retired Swift app.
- Add localized GitHub update checks, automatic-check opt-out, signed downloads and explicit installation/restart guarded against active work and unsaved edits.
- Prepare a manually dispatched, signed macOS Apple Silicon draft-release workflow and verified updater manifest generation. This baseline was not published; the production updater configuration and native acceptance are recorded under 1.0.1.

- Added a contributor guide, issue forms and a pull request template.
- Added the MIT license and clarified separate third-party and model notices.
- Documented the version and release policy and added a workspace version guard, fixture tests and lightweight CI check.
