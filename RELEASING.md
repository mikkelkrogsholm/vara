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
   python3 scripts/check-version.py --tag v1.0.2
   ```

Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html):
`MAJOR.MINOR.PATCH`. From 1.0, use patch releases for fixes, minor releases for compatible features,
and major releases for incompatible public changes. Public compatibility contracts,
including CLI schemas and RPC, can be versioned independently as needed. A
maintainer creates a `vX.Y.Z` tag only for an explicit release; these checks do
not create tags or publish anything. The changelog records unreleased changes
until an actual release is made.

## Distribution status

[Vara 1.0.1](https://github.com/mikkelkrogsholm/vara/releases/tag/v1.0.1) is public
for **macOS Apple Silicon (`darwin-aarch64`) only**, from tag `v1.0.1` at
`11c5176`. The Developer ID signed app was notarized, stapled and accepted by
Gatekeeper. The real installed 1.0.0 bootstrap upgraded through the visible
Settings/check/download/install flow and restarted as 1.0.1. Automatic retirement
of the previous independent CLI owner did **not** pass; the native GUI upgrade
passed, while overall updater acceptance remains partial. See the evidence below.
The retired Swift app's Sparkle feed and DMG script do not release this app.
Windows/Linux and Intel Mac release acceptance remain open.

The 1.0.2 candidate repairs independent-owner handoff ([#37](https://github.com/mikkelkrogsholm/vara/issues/37)).
GUI and CLI probe the private v2 owner's `build_version`. Older idle owners,
including the 1.0.0/1.0.1 owners that report `busy` without a build version, are
retired only after taking the exclusive profile admission gate and rechecking
busy state. The PID is authenticated by the kernel's Unix socket peer metadata.
Busy accepted work continues on its existing owner until idle. Current/newer,
unknown-busy and incompatible peers are retained. A concurrent installer keeps
its Ping usable while it holds the gate. Signed 1.0.1→1.0.2 native GUI acceptance
and publication remain pending; these changes do not amend the public 1.0.1 tag.

Isolated macOS verification for this candidate used the retained notarized 1.0.0
and 1.0.1 binaries on fresh temporary profiles. Each legacy owner exited through
automatic handoff and the public `cli models --json` command reached a 1.0.2
owner; repeated CLI calls retained that owner. The 13 native backend harness
tests cover concurrent admission, retained durable jobs and cancellation, idle
handoff, current/newer/unknown-owner retention, and the existing socket lifecycle.
The three production-build admission-gate tests also pass. This evidence does
not replace signed GUI upgrade acceptance on the installed profile.

After verification, generated `target` output can be removed to reclaim disk
space. Keep the installed app and user profile; do not commit build output or keys.

## GitHub updates and release configuration

The workspace is **1.0.2, an unreleased candidate**; the public app remains **1.0.1, released 2026-11-05**. The retired Swift distribution and its
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
Downloaded bytes are held only in memory. Native GUI replacement, retained data
and microphone permission were verified for 1.0.0→1.0.1; automatic independent
CLI-owner handoff remains open. Settings shows the running package
version separately from any available version. A quiet sidebar update button is
reachable from every main view, including the transcription editor; downloading
preserves that editor, and installing still requires saved work.

### Release bundler public-key configuration

The runtime compile-time key and the Tauri CLI bundler must receive the same
public key. An empty `plugins.updater.pubkey` can allow compilation but fails
updater artifact signing with “Missing comment in public key”. Generate the
release overlay from the public environment value, rather than storing a key in
tracked configuration. From the repository root, after preparing the existing
signing/notarization environment and Who runtime:

```sh
export VARA_RELEASE_BUILD=1
# VARA_UPDATER_PUBLIC_KEY must contain the existing Tauri base64 public key.
cd desktop/src-tauri
release_config="$(python3 ../../scripts/generate-release-config.py)"
cargo tauri build --config "$release_config" --bundles app,dmg -- --locked
```

The generated JSON combines `tauri.release.conf.json` and the public key and is
passed directly to the CLI, which merges it with `tauri.conf.json`. The generator
fails before building if the key is absent, malformed or missing its Tauri
comment. The draft workflow uses this same method. Private signing credentials
remain in the existing secure environment; this command does not create keys.

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
executed. On 2026-11-05, the production updater key was generated encrypted outside
Git, its password retained in Keychain, and the GitHub `release` environment was
configured with the updater private-key/password secrets and public-key variable.
A local Developer ID identity and working `vara-notary` profile are available;
this does not configure the CI Apple certificate/password secrets.

The generator requires nonempty, versioned artifact files, real `.sig` paths and
a public-key file. It cryptographically verifies the artifact and trusted comment,
key ID and exact signed version before writing. URLs use those exact filenames
under this repository's `vX.Y.Z` release; no placeholder manifest is committed:

```sh
python3 scripts/generate-updater-manifest.py --version 1.0.1 \
  --public-key-file /secure/path/updater.pub \
  --artifact darwin-aarch64 release-assets/Vara_1.0.1_aarch64.app.tar.gz release-assets/Vara_1.0.1_aarch64.app.tar.gz.sig \
  --notes-file CHANGELOG.md --output release-assets/latest.json
```

When recreating a macOS updater archive after stapling, suppress AppleDouble
metadata, then sign the **new bytes** with the existing updater key and regenerate
the manifest:

```sh
COPYFILE_DISABLE=1 tar --no-mac-metadata -czf release-assets/Vara_1.0.1_aarch64.app.tar.gz \
  -C target/release/bundle/macos Vara.app
# Sign this archive using the existing secure signer environment before running
# generate-updater-manifest.py with its new .sig file.
```

A root `._Vara.app` entry is incompatible with Tauri's macOS extraction: removing
the first path component leaves an empty file path and installation fails before
the swap. The manifest generator now validates real macOS tar.gz structure and
rejects AppleDouble roots, multiple/non-app roots, unsafe paths and escaping
symlinks in addition to its cryptographic checks. This guard is a post-release
change; it does not alter the published 1.0.1 app or tag. Metadata-free extraction
of the actual repaired app also passed strict signature and stapler validation.

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

### Native 1.0.0→1.0.1 evidence — 2026-11-05

The installed bootstrap carried the production public key. Accessibility invoked
the real visible Settings, manual check, download, verified-ready and install/restart
controls. The GUI restarted; the installed bundle and public CLI reported 1.0.1,
and Settings displayed `Du bruger Vara 1.0.1`. Deep/strict signature verification
and Gatekeeper's Notarized Developer ID assessment passed. Apple accepted these
notarization submissions:

- 1.0.0 bootstrap: `7f8cbeb1-25cb-4ef9-a39e-fd9e32f783dd`.
- 1.0.1 release: `5c0294ed-cab0-4e58-a03a-6645c3857e7f`.

The original 344 dictation rows matched their baseline hash; two new user
dictations brought the total to 346. All four transcriptions retained their table
bytes/hashes. Preferences, recovery, cancelled and staging data matched the
baseline. The 55 model files retained names, sizes and modification times. OpenAI
credential presence and the microphone permission value (`auth_value=2`) remained
unchanged. Only preservation counts/hashes and credential presence are recorded
here; no credential values or transcript content are included. Private
Accessibility logs remain local and are not release evidence files.

The previous independent CLI owner survived the GUI restart. After authoritative
busy status was false, it was manually retired under the exclusive update lock;
a new 1.0.1 owner then served the public `vara models --json` command successfully.
This is **manual recovery**, not evidence of automatic handoff. Automatic owner
retirement is tracked in [#37](https://github.com/mikkelkrogsholm/vara/issues/37),
and keeps the broader [#34 acceptance](https://github.com/mikkelkrogsholm/vara/issues/34)
partial. Native upgrade evidence does not establish Windows/Linux/Intel Mac
support or execution of the CI draft workflow; its Apple signing credentials
remain unconfigured. Live active-work refusal scenarios beyond the recorded
component/browser checks are not claimed here.

### Native macOS UI acceptance helper

Build the standalone Accessibility helper without rebuilding Vara:

```sh
DEVELOPER_DIR=/Library/Developer/CommandLineTools swiftc desktop/tests/updater-native.swift -o /tmp/vara-updater-native
/tmp/vara-updater-native --pid PID tree
/tmp/vara-updater-native --pid PID settings
/tmp/vara-updater-native --pid PID status
/tmp/vara-updater-native --pid PID check
/tmp/vara-updater-native --pid PID download
# Run only when installation/restart is intended:
/tmp/vara-updater-native --pid PID install --allow-install
```

Inspect `status` separately after each action and select the new PID after restart.
The helper reads the real WKWebView Accessibility hierarchy, matches visible
Danish/English button labels and invokes AXPress. It targets only `dk.vara.desktop`,
requests no permissions and does not operate OS permission dialogs. Missing
accessibility or disabled buttons fail explicitly. Tree/status output can contain
visible user text; retain it locally. Successful AXPress alone is not upgrade
acceptance: verify the installed version, app relaunch, retained data/models/keys
and permissions separately.

References: [Tauri updater](https://v2.tauri.app/plugin/updater/),
[Apple signing](https://v2.tauri.app/distribute/sign/macos/),
[Windows signing](https://v2.tauri.app/distribute/sign/windows/).
