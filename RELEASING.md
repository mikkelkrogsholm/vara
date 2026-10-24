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
