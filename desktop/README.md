# Vara — Rust desktop

The desktop app follows [DESIGN.md](../gdd/DESIGN.md) and the selected Warm
precision mockup. The existing Swift app is preserved alongside this clean-write.

## Build and run

Install Rust using the pinned `rust-toolchain.toml`, CMake, the native C/C++
toolchain and Tauri 2 platform prerequisites. On macOS this means Xcode command
line tools. Install the Tauri CLI with `cargo install tauri-cli --locked`.

From the repository root:

```sh
cargo test --workspace
cargo tauri dev
sh scripts/build-desktop-macos.sh # macOS: stable Apple signing identity
```

The macOS application is `target/release/bundle/macos/Vara.app`. Build with `scripts/build-desktop-macos.sh` on macOS to sign using an existing
Developer ID identity (or explicit `APPLE_SIGNING_IDENTITY`). This preserves the
bundle identity across rebuilds for macOS permissions. The app is not notarized. The build exports
the canonical design tokens, pure components and assets automatically; edit their
sources under `gdd/design`, not generated `desktop/ui/design` files. Desktop
compositions and behavior live in `desktop/ui/{components,views,app}.js`.

The app starts with an empty profile and guides model download, permissions and
a first dictation. Default: local Parakeet Redux, hold Alt+Space, direct text. Keys
are stored in the OS credential store. Models and SQLite history live in the
platform app-data directory. `VARA_DATA_DIR` selects an isolated developer profile.
Do not use a development fixture profile as your personal profile.

Redux downloads four verified files (179 MB) into `models/parakeet-redux`, with
Metal on supported Apple builds and CPU elsewhere. Speech language is detected
automatically. Whisper Base remains an optional local engine. Saved engine choices
are preserved when an older profile gains Redux as an available built-in. Model
and runtime notices are included in the bundle's resources under `notices/`.

## Modules and contracts

See [Rust bindings](../gdd/RUST-BINDINGS.md). Domain/storage and engine APIs have
no Tauri dependency. Native focus, audio capture, credentials and HUD stay behind
`vara-platform`; a later mobile shell can reuse the engine without the desktop UI.

## Verification and remaining work

[UI review](../gdd/reports/DESKTOP-UI-REVIEW.md) records screens, states and native
inspection. `cargo test --workspace` covers 30 domain/storage/model-integrity and HTTP protocol
contracts. The browser suite renders actual production UI with test-only IPC
fixtures; fixtures are outside the packaged UI directory.

For browser checks, serve `desktop` on localhost port 18770 and execute
`desktop/tests/ui.playwright.js` through the Playwright tool's file runner. The
script creates its own isolated context and writes screenshots under gdd/reports.

For a microphone-free native pipeline test, build with the explicit developer
feature `test-harness` and set `VARA_TEST_WAV` to a local WAV, together with an
isolated `VARA_DATA_DIR`. Normal builds exclude this feature and ignore that WAV
setting. Never distribute the harness build.

Current limits:

- macOS is the tested development target. Windows/Linux focus insertion and native
  floating HUD adapters are unfinished; their current safe fallback is manual
  insertion in the main UI. Cross-platform release acceptance remains open.
- Real microphone-to-field insertion and physical global-hotkey activation still
  need hands-on verification; synthetic app-bound keys did not exercise the OS
  hotkey in the UI automation. The native fixture verified inference, history,
  manual delivery and successful audio cleanup.
- Cloud adapters require the user's own keys and have not been exercised against
  live accounts. Claude/Gemini CLI paths are optional and restricted to verified
  text-only capabilities. Codex CLI stays unavailable until a tools-off contract
  can be verified; installation alone does not imply a working integration.
- Local Danish sample inference works, but Redux and Whisper made recognition errors.
  No broad accuracy benchmark or human model-quality acceptance is claimed.

Current Redux integration evidence: [acceptance report](../gdd/reports/REDUX-INTEGRATION.md).

Groq setup and current model/API verification: [Groq integration](../gdd/reports/GROQ-INTEGRATION.md).

OpenAI: current GPT-Transcribe speech and Responses API / GPT-6 Luna text, with OS-key catalogue check. See [integration evidence](../gdd/reports/OPENAI-INTEGRATION.md). Saved model overrides remain unchanged.

Headless verification: 30 workspace tests plus the real Redux cancellation/reuse test and 13 browser groups pass. See [verification report](../gdd/reports/HEADLESS-VERIFICATION-2026-10-23.md).

Mac permission troubleshooting: microphone, Input Monitoring (global physical
shortcuts) and Accessibility (focus/automatic insertion) are separate permissions.
Use the current signed Vara bundle. Older unsigned/ad-hoc entries may need to be
removed and the current app re-added in System Settings; do not reset unrelated
applications' privacy permissions. The app retries physical-shortcut registration
on returning to its window after permission changes.
