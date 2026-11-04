# Releasing Vara desktop

The current delivery artifact is a Developer ID–signed macOS app built from the
Rust/Tauri workspace. Its bundle identifier is `dk.vara.desktop`.

## Verify and build

1. Run the optimized workspace tests from the repository root:
   ```sh
   CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo test --release --workspace --locked
   ```
2. Build with the existing Apple Developer ID identity:
   ```sh
   sh scripts/build-desktop-macos.sh
   ```
   The script selects an installed Developer ID Application certificate, or uses
   `APPLE_SIGNING_IDENTITY` when set. It verifies the signature and audio-input
   entitlement. Use the same identity across rebuilds to keep macOS permissions
   associated with the app.
3. Inspect `target/release/bundle/macos/Vara.app`. Quit the installed app before
   replacing `/Applications/Vara.app`, then launch that installed copy. Preserve
   user data and OS credential-store entries; installation is not a profile reset.
4. Verify microphone, the chosen global shortcut and insertion into another app.
   Input Monitoring, Accessibility and microphone access are separate permissions.
5. Install and verify the bundled CLI launcher using [CLI.md](CLI.md). The guide
   is also included as `Contents/Resources/CLI.md`; it invokes the installed app,
   and no test harness or model weights belong in the release.

For an opt-in, isolated local ASR and Who smoke test with repeated tracked `Samples/da_test.wav`, synthetic video, and a short synthetic Sara voice, run `python3 scripts/tests/real_local_cli.py APP MODELS NEW_OUTPUT_DIR [LAUNCHER]`.
Point `APP` at the signed app and `MODELS` at an existing model directory; the
new output directory retains the profile, inputs and JSON/stderr for
review. Supply the installed launcher as the optional fourth argument to test
the public entrypoint of `/Applications/Vara.app`; otherwise it uses the internal
developer entrypoint against the supplied app. It does not verify native UI. The known-count audio repeats the 10.312-second sample 31 times to span a
300-second SDK window; this is a smoke test, not an accuracy score. No models are
downloaded and no cloud engine is selected. Run only with models whose license
has already been accepted.

For a version change, keep the workspace version in `Cargo.toml`, the package
version in `desktop/src-tauri/Cargo.toml` and `desktop/src-tauri/tauri.conf.json`
consistent, and update `Cargo.lock` as necessary.

## Distribution status

The app is signed but is not currently notarized. Public download packaging,
notarization and automatic updates need their own implementation and verification
before a public release. The retired Swift app's Sparkle feed and DMG script do
not release this app. Windows/Linux release acceptance remains open.

After verification, generated `target` output can be removed to reclaim disk
space. Keep the installed app and user profile; do not commit build output or keys.
