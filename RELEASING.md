# Releasing Vara desktop

The current delivery artifact is a Developer ID–signed macOS app built from the
Rust/Tauri workspace. Its bundle identifier is `dk.vara.desktop`.

## Verify and build

The signed build requires Python 3.11 or newer for the version check.

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

## Versions

The workspace version in `Cargo.toml` is the source of truth; every workspace
package, including the desktop manifest, inherits it. Keep the literal version
in `desktop/src-tauri/tauri.conf.json` and the local packages in `Cargo.lock` in
sync. For an intentional version change:

1. Change `[workspace.package].version` in `Cargo.toml` and the literal `version`
   in `desktop/src-tauri/tauri.conf.json` to the same SemVer value. Package
   manifests continue to inherit the workspace version.
2. Refresh `Cargo.lock` without compiling and review its diff:
   ```sh
   cargo metadata --format-version 1 > /dev/null
   git diff -- Cargo.lock
   ```
   Cargo may need access to dependencies when resolving metadata.
3. Update [CHANGELOG.md](CHANGELOG.md) for the change. Validate with Python 3.11+:

   ```sh
   python3 scripts/check-version.py
   python3 -m unittest discover -s scripts/tests -p 'test_version.py'
   # For an explicit release, replace the example with the intended tag:
   python3 scripts/check-version.py --tag v1.0.0
   ```

Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html):
`MAJOR.MINOR.PATCH`. From 1.0, use patch releases for fixes, minor releases for compatible features,
and major releases for incompatible public changes. Public compatibility contracts,
including CLI schemas and RPC, can be versioned independently as needed. A
maintainer creates a `vX.Y.Z` tag only for an explicit release; these checks do
not create tags or publish anything. The changelog records unreleased changes
until an actual release is made.

## Distribution status

The locally built app is signed but is not currently notarized. The updater and
draft release tooling below are implemented; no public 1.0.0 release, notarized
CI artifact or real upgrade has been verified yet. The retired Swift app's Sparkle feed and DMG script do
not release this app. Windows/Linux release acceptance remains open.

After verification, generated `target` output can be removed to reclaim disk
space. Keep the installed app and user profile; do not commit build output or keys.

## GitHub updates and release configuration

The Rust app is **1.0.0, unreleased**. The retired Swift distribution and its
Sparkle feed are unrelated. Stable discovery uses only
`https://github.com/mikkelkrogsholm/vara/releases/latest/download/latest.json`.
Drafts and prereleases do not activate this endpoint. Until a stable release
contains this asset, checks can fail; failure never means “up to date”.

The maintained Tauri updater verifies artifact signatures, requires a signed
version matching the manifest, and accepts newer versions only. Checks occur at
startup and at most once per 24 hours thereafter, with an hourly opportunity
while open. Failed attempts count toward the throttle. Manual checks bypass it;
automatic checks can be disabled in Settings. Checks contact GitHub with the updater user agent; GitHub receives the network
request and the requested asset identifies its platform. No audio, text, models
or secrets are sent. Profile `updates.json` stores only
the preference and timestamp.

Set **compile-time** `VARA_UPDATER_PUBLIC_KEY` to the base64 public key emitted by
Tauri's signer. Runtime environment changes do not change an installed trust
root. Local builds without it remain usable and show updates disabled in
Settings. `VARA_RELEASE_BUILD=1` fails builds without that key. Keep the private
key out of Git, back it up securely and preserve the public trust root across
releases. Deliberately generate production keys with `cargo tauri signer generate`;
none are generated or installed by this implementation. Use Tauri CLI **2.12.4**
or a compatible newer CLI that embeds the version in the signed trusted comment.

Users first download/verify, then separately choose **Install and restart**.
Unsaved Settings, dialogs and transcription drafts/saves block that action; the
UI becomes inert during installation. A profile OS admission lock blocks native
recording/retry/model starts, GUI file jobs and CLI backend operations during the
final swap. Already accepted work is checked again after taking the exclusive
lock. Older backends lacking authoritative busy evidence fail closed. User data,
models and credentials are preserved; failed installation needs a fresh download.
Downloaded bytes are held only in memory. Native replacement and permission
continuity still need real verification.

### Explicit draft workflow

`.github/workflows/draft-release.yml` runs only by manual dispatch, requires an
**existing** `vX.Y.Z` tag matching the checkout, never creates tags or publishes,
and refuses to modify published releases. It targets **macOS Apple Silicon only**:
sign/notarize, build updater artifacts, verify signatures and version, generate
`latest.json`, and upload to a draft. Configure required reviewers on the GitHub
`release` environment before use. Set its variable `VARA_UPDATER_PUBLIC_KEY` and
secrets:

- `TAURI_SIGNING_PRIVATE_KEY` and, when needed, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Base64 Developer ID Application PKCS#12 in `APPLE_CERTIFICATE`, its password in
  `APPLE_CERTIFICATE_PASSWORD`, exact certificate name in `APPLE_SIGNING_IDENTITY`.
- Notarization `APPLE_ID`, app-specific `APPLE_PASSWORD`, `APPLE_TEAM_ID`.

The certificate is imported into a temporary hosted-runner keychain, discarded
with that ephemeral runner; the exported PKCS#12 is removed. Updater signing is
separate from Apple signing/notarization. Dependencies include Xcode/Swift, the
pinned Who SDK, Rust from `rust-toolchain.toml`, Python 3.11+, Tauri CLI and OpenSSL
3. The workflow selects an installed Xcode with Swift 6.2+ and fails clearly if none is available. Expensive release tests/builds run only on dispatch. This workflow has not been
executed and no release credentials have been configured by this task.

The generator requires nonempty, versioned artifact files, real `.sig` paths and
a public-key file. It cryptographically verifies the artifact and trusted comment,
key ID and exact signed version before writing. URLs use those exact filenames
under this repository's `vX.Y.Z` release; no placeholder manifest is committed:

```sh
python3 scripts/generate-updater-manifest.py --version 1.0.0 \
  --public-key-file /secure/path/updater.pub \
  --artifact darwin-aarch64 release-assets/Vara_1.0.0_aarch64.app.tar.gz release-assets/Vara_1.0.0_aarch64.app.tar.gz.sig \
  --notes-file CHANGELOG.md --output release-assets/latest.json
```

Recognized mappings: `darwin-aarch64`/`darwin-x86_64` → `.app.tar.gz`,
`windows-x86_64-nsis`/`windows-aarch64-nsis` → `.exe`, and
`linux-x86_64-appimage`/`linux-aarch64-appimage` → `.AppImage`. These are manifest
mappings, **not verified shipping platforms**. Intel Mac, Windows and Linux are
not built by this workflow. Windows needs compatible NSIS packaging,
Authenticode signing/timestamping and tested installer exit/relaunch behavior;
Linux needs verified AppImage/system runtime dependencies. Verify each platform
before adding release assets.

Before a separate manual publication, inspect artifacts/URLs and notarization,
clean installation and a real **1.0.0→1.0.1** upgrade. Exercise active recording,
CLI/transcription jobs, model downloads, dirty editors, network failure and invalid
signatures, data/model/credential retention and OS permissions. Confirm the old
idle CLI owner exits and subsequent calls launch the new owner. Drafts alone do
not make updates available.

References: [Tauri updater](https://v2.tauri.app/plugin/updater/),
[Apple signing](https://v2.tauri.app/distribute/sign/macos/),
[Windows signing](https://v2.tauri.app/distribute/sign/windows/).
