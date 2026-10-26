# Vara — Rust desktop

The desktop app follows [DESIGN.md](../gdd/DESIGN.md) and the selected Warm
precision mockup. This is the sole active application implementation in this repository.

## Build and run

Install Rust using the pinned `rust-toolchain.toml`, CMake, the native C/C++
toolchain and Tauri 2 platform prerequisites. On macOS this means Xcode command
line tools. Install the Tauri CLI with `cargo install tauri-cli --locked`.

From the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo test --release --workspace --locked
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

See [implementation status](../IMPLEMENTATION.md) for current limits and
[gdd/reports](../gdd/reports) for detailed verification evidence. The user has
confirmed macOS global shortcuts, insertion in TextEdit/Codex, OpenAI and local
Parakeet + Qwen. Windows/Linux native integration remains unfinished.

For browser checks, serve `desktop` on localhost port 18770 and execute
`desktop/tests/ui.playwright.js` through the Playwright tool's file runner. The
suite renders the production UI with isolated test-only IPC fixtures.

For a microphone-free native pipeline test, build with the explicit developer
feature `test-harness`, set `VARA_TEST_WAV` to a local WAV and use an isolated
`VARA_DATA_DIR`. Normal builds exclude this feature. Never distribute a harness
build. Do not use fixture profiles as personal profiles.

Mac permission troubleshooting: microphone, Input Monitoring (global physical
shortcuts) and Accessibility (focus/automatic insertion) are separate permissions.
Use the current signed Vara bundle. Older unsigned/ad-hoc entries may need to be
removed and the current app re-added in System Settings; do not reset unrelated
applications' privacy permissions. The app retries physical-shortcut registration
on returning to its window after permission changes.

Credential presence queries attributes rather than retrieving a password. On Mac,
new/replaced keys have explicit application trust, and authenticated worker calls
time out after 15 seconds. The executable must call
`vara_platform::credential_worker_entry()` before initializing Tauri. Do not expose
an unauthenticated credential command-line API. Store errors offer retry; timed-out
writes may have completed and must be checked before repeating. See the
[credential verification report](../gdd/reports/CREDENTIAL-HARDENING-2026-10-24.md).

## File-transcription development

`vara-core::transcription` defines file documents, timing validation, export and a
separate document store. `vara-engine::{media,transcription}` decode and recognize
files. The host queue lives in `src-tauri/src/transcription.rs`; its UI composition
and controller are in `ui/transcription.js` using existing shared primitives and
model readiness components. Transcription media is exposed only through a scoped
native asset protocol; unrelated files are outside that playback scope.

For a native file-job smoke test, build with `test-harness`, set an isolated
`VARA_DATA_DIR` with available models and point `VARA_TEST_FILE` to a disposable
fixture. Optional `VARA_TEST_FILES_JSON` supplies a JSON array for a batch;
`VARA_TEST_CANCEL_FIRST=1` exercises cancellation. Normal builds exclude this
harness. File tests do not register a dictation shortcut or access the microphone.
Never use your personal data directory for harness tests.

Core/media tests run with the workspace. Browser journeys are in
`tests/transcription.playwright.js` using isolated IPC fixtures. Actual local
inference can be verified with the `vara-engine` example `file_transcription`.
See the feature report for evidence, limits and native acceptance status.

## Who local speaker labeling

Open a completed file document and choose **Find talere**. Download Who once via
the explicit supplier/license/usage disclosure; then run it on the private audio
copy. Speaker names and original word times remain inspectable; ambiguous words
are not assigned a single speaker. Empty/failed results retain prior annotations.

The Mac packaging script invokes `scripts/build-who-runtime.sh` (Swift 6.2+ is a
build dependency of the official SDK, not an end-user requirement). The runtime
is ignored build output under `src-tauri/runtime/who/`. Model weights remain in
the app-data model directory and are not part of the app bundle. Who needs
macOS 15+, while the rest of Vara keeps its existing floor. The same SDK C ABI is
used by the Rust integration on other platforms; their builds must provide
`libWhoAndroid.so`/`WhoAndroid.dll` and the official LiteRT dependencies under
`runtime/who`, then validate on those platforms. Do not ship a Mac dylib as a
crossplatform runtime.

Verification CLI (normalized mono 16 kHz PCM16 WAV):
`cargo run --release -p vara-engine --example who -- RUNTIME MODEL_DIR diarize AUDIO.wav`
The example also accepts `status`, `download`, and `cancel` for lifecycle checks.
