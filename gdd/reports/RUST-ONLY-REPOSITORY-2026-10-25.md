# Rust-only application repository — 2026-10-25

User request: remove the retired Swift implementation from the repository and
continue with the new Rust application only. This is repository consolidation;
product behavior, GDD contracts, the selected design and user data are unchanged.

## Changes

Removed the old `App`, `Sources`, `Tests`, Xcode project, Swift package files,
XcodeGen configuration, Swift-only release/setup scripts, old overview and obsolete
environment template. Removed the four historical Swift source captures from the
working tree; their committed originals remain in Git at
`3eb56eb8dce1edff66f8b371acd9fb62ff08ecf5`.

Added a root README and updated development, release and implementation notes.
The Rust workspace, JavaScript desktop UI and native Objective-C platform bridges
are retained. Canonical GDD design assets, Rust benchmark scripts and speech
fixtures are retained. Historical GDD reports and snapshots retain their original
revision context and are not active build dependencies.

Before removal, local modifications and the untracked Swift listener were copied
to `../vara-legacy-backup-2026-10-25/`, including a binary Git diff. No credentials
or environment secrets were copied. The personal app profile and installed app
were not modified by this cleanup.

## Verification

- No tracked Swift/Xcode source remains on disk.
- No legacy build-path references remain in the active Rust, desktop or scripts.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Optimized workspace tests: 32 passed; one model-dependent runtime test skipped
  by its explicit weight-fixture requirement. No live provider calls were needed.
- Current GDD model schema, semantics and record integrity pass. The broader
  records activity checks still report missing applicability/authorization plans;
  this cleanup does not claim to close existing release-acceptance gaps.

The signed macOS packaging check and final build-cache cleanup are recorded below.

- `scripts/build-desktop-macos.sh` passes after removal: optimized Tauri build,
  Developer ID signing, strict signature verification and audio entitlement check.
- Removed generated build output after verification (approximately 1.9 GiB),
  plus the old Swift/Xcode caches. The installed Rust app's signature still
  verifies. No extra app bundle remains in the repository.
