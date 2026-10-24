# Credential hardening — 2026-10-24

Scope: the three requested fixes before the user creates an OpenAI API key.
No live OpenAI request, existing user-key read, or credential migration was
performed. Store probes used unique disposable accounts and fictional values.

## Result

1. `key_status` calls `has_secret`. The Mac query requests only attributes, never
   password data. An unavailable/locked store is an error, not “no key”. Other
   desktop adapters use keyring's attributes API.
2. New and replaced Mac entries have an explicit ACL trusting the signed Vara
   executable. Apple's designated-requirement semantics preserve trust across
   updates signed by the same identity. Existing service/account identifiers
   remain unchanged; existing credentials are not silently rewritten. Replacing
   an existing key applies the explicit ACL. Authorization remains subject to
   macOS and the user's Keychain controls; this is not protection against an
   already-compromised user session or a user approving another application.
3. On Mac, keychain operations run in an isolated process with a 15-second
   deadline, including pipe writes. Expiry kills/reaps the worker. The worker
   accepts only its real parent satisfying its own designated signing requirement;
   an arbitrary process cannot use the command-line mode to obtain keys. Anonymous
   pipes carry values, not command-line arguments, environment or temporary files.
   Interactive Keychain prompts are disabled. Locked/denied/unavailable failures
   explain how to unlock/check access and retry. The UI clears the entered key
   even on failure and offers a metadata retry. Mutations that time out are
   reported as possibly completed and are never automatically replayed.

## Verification

- Real native disposable-keychain test: metadata, save, signed-parent check, read,
  denial from a different signed executable, deletion and confirmed absence PASS.
  `desktop/tests/credentials-native.m`, linked with `macos_credentials.m`; test
  executables were signed with distinct ad-hoc identifiers.
- Real Rust adapter probe, signed with the same Developer ID identity as Vara:
  missing status, save, present status, read, replacement, delete, missing read PASS.
  `cargo build --release -p vara-platform --example credential_probe`; sign the
  executable before running. The test account is unique to its PID and cleaned
  up on return/unwind.
- Calling the signed probe's worker mode from the shell exits 77, without output.
- `cargo test --release -p vara-platform credentials:: --lib`: stalled-worker
  test PASS. Exercises status/set/delete deadlines with a shortened test deadline;
  verifies the process was reaped and mutations report uncertainty.
- Production UI Playwright harness: all 16 groups PASS. New fixture scenarios
  cover locked status, retry, uncertain mutation timeout and secret clearing.
- Signed Mac application build PASS; deep/strict code-signing verification and
  microphone entitlement check PASS. The native OpenAI key dialog correctly
  reports an absent key. Existing OS permissions still report granted.
- GDD model, record integrity and generated visualization validated. Broader
  implementation/readiness plans remain UNKNOWN; no release acceptance asserted.

## Limits

Windows/Linux adapters and platform security behavior were not exercised here;
Mac worker deadline and explicit application ACL are platform-specific. The Mac
login-keychain ACL APIs are legacy/deprecated APIs, used deliberately for explicit
per-application trust in an unsandboxed Developer ID application. No login-keychain
lock or permission changes were made to simulate failure. Live OpenAI inference
verification awaits a user-created key.
