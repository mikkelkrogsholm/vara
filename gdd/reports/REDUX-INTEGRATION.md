# Parakeet Redux integrated — October 23, 2026

The user approved Redux as the default. New profiles now use `local.parakeet`;
Whisper Base is an optional alternative. Existing saved choices are preserved while
Redux is added as an available built-in engine.

## Change

- Shared Rust adapter via voxrt, pinned to commit
  `4fab626c00708d9bc97f837d309e0d362094282b` (0.2.1).
- Metal on supported Apple builds and CPU fallback. Eight worker threads, cached
  model and an independent decoder/VAD session per dictation.
- Automatic language detection. The UI explains this and disables manual language
  selection when Redux is selected.
- Four files, **179,005,408 bytes**, pinned revision and SHA-256 checks. Separate
  `models/parakeet-redux/` directory so the Qwen tokenizer is not overwritten.
- Onboarding, model overview and readiness follow the selected speech model. An
  installed Whisper model does not hide a missing Redux model or vice versa.
- Local speech/text and CLI auth do not read the OS API key store.
- Cooperative cancellation before/after loading, between input blocks and before/after
  finalization. In-flight kernels finish before the backend exits. The host generation
  check/tombstone rejects late results after accepted cancellation.
- Backend panic becomes a normal recognition error with retry available.
- Model and runtime attribution is included in the app's `Resources/notices/`.

Existing history, AI, focus, clipboard and 24-hour failed-audio deadline rules also
apply to Redux. There is no automatic switch to a cloud engine.

## Actually verified

1. **Real installation:** `local_probe` downloaded and verified all four files in an
   isolated test profile through the app's installer. It then recognized
   `Samples/da_test.wav` through the new production adapter. Cold model loading plus
   transcription took about 1.23 s on this already shader-warmed Mac.
2. **Rust contracts:** 17 ordinary tests pass, including profile upgrade, cancelled
   Redux installation, missing-model error without provider switch and separate tokenizer directories.
3. **Real model weights:** a separate opt-in runtime test passes. It recognizes the
   short sample, accepts cancellation of a long repeated sample and then recognizes
   the original sample again with the same cached model and identical text.
4. **UI:** 11 browser test groups pass with production HTML/CSS and an isolated IPC
   fixture. A fresh profile shows Redux; optional Whisper and readiness were checked.
5. **Mac bundle:** normal release build without `test-harness`, **24.77 MiB**, built.
   Model notices were verified in the bundle. GDD model and graph validation pass.

## Limitations

The Mac was locked during the work. The user explicitly chose to continue without
native UI checks. Therefore **no** new native Redux session or microphone-to-text-field
test was conducted for this change. Earlier native Whisper checks are historical
evidence and are not claimed as Redux verification.

UI fixtures do not prove OS behavior. Real microphone, physical global shortcut,
Windows/Linux and mobile still lack final verification. Windows/Linux native focus and
HUD adapters remain incomplete. Danish model quality has not been finally accepted
on the basis of one audio sample.

## Reproduce

```sh
cargo test --workspace
cargo run --release -p vara-engine --example local_probe -- target/redux-acceptance/models Samples/da_test.wav
VARA_TEST_MODELS="$PWD/target/redux-acceptance/models" cargo test --release -p vara-engine --test redux_runtime -- --ignored
cargo tauri build --bundles app
```

Release: `target/release/bundle/macos/Vara.app`. Restart an already running app to
load the new build. No test history or test models were placed in the normal user
profile. See [machine-readable evidence](redux-integration-2026-10-23.json).
