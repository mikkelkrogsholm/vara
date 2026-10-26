# Vara

Vara is a desktop dictation app with a shared Rust engine. Hold your chosen
shortcut, speak, and release to insert text into the active app. Local speech is
the default; cloud providers and optional AI text processing use your own keys.

This repository contains the Rust/Tauri version. The retired Swift application
and its Xcode project are available only in Git history.

## Development

Install the pinned Rust toolchain, CMake, a native C/C++ toolchain and the Tauri 2
platform prerequisites. On macOS, install Xcode command line tools. Install the
Tauri CLI with `cargo install tauri-cli --locked`.

```sh
cargo tauri dev
CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo test --release --workspace --locked
sh scripts/build-desktop-macos.sh
```

The signed macOS bundle is written to `target/release/bundle/macos/Vara.app`.
See [desktop development](desktop/README.md) and [release instructions](RELEASING.md).

## Repository map

| Path | Responsibility |
| --- | --- |
| `crates/vara-core` | Domain contracts, settings and persistence |
| `crates/vara-engine` | Local models and provider adapters |
| `crates/vara-platform` | Audio, credentials, shortcuts and native insertion |
| `desktop/src-tauri` | Tauri desktop host and packaging |
| `desktop/ui` | Desktop views and behavior |
| `gdd` | Product specification, contracts, design and evidence |
| `scripts` | Rust desktop build, design synchronization and model benchmarks |
| `Samples` | Speech verification fixtures |

The reusable core and engine have no Tauri dependency. Native macOS Objective-C
bridges belong to the platform adapter. The desktop UI uses JavaScript/CSS; these
are intentional parts of the Rust desktop architecture.

Canonical design assets live under `gdd/design`; generated `desktop/ui/design`
files are build output. Follow [DESIGN.md](gdd/DESIGN.md) and
[Rust contract bindings](gdd/RUST-BINDINGS.md).

macOS is the tested development target. Windows/Linux native integration and
mobile shells still require implementation and testing. See
[implementation status](IMPLEMENTATION.md) for current limits.

## File transcription

Open **Transskribering**, add audio/video files, choose a model and start the batch.
The file library is separate from app-grouped dictation history. It retains the
original transcript, optional corrections and separate AI output. Word/segment
navigation and subtitle export are available only when actual model timings exist.

Vara keeps a private normalized playback copy until the document is deleted;
original user files are never modified or deleted. The dictation recording's
24-hour retry policy does not apply to these expressly retained playback copies.
Common formats decode locally in Rust. An independently installed FFmpeg can add
codec support; Vara does not download or bundle it. Unsupported inputs are reported.
See [transcription specification](gdd/TRANSCRIPTION-SPEC.md) and
[verification evidence](gdd/reports/transcription/README.md).

Cancellation of an unfinished file job removes its private working audio; the
source file remains untouched. Failed recognition can reuse a valid retained
playback copy on retry even if the external original has moved.
