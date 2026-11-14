# macOS insertion diagnostics: bounded backend evidence

Date: 2026-11-14. Scope: core/store, macOS native adapter and Rust bridge,
Service orchestration, deterministic tests. UI recovery presentation is separate.
No claim is made about the cause on Peter's computer or compatibility with a
specific editor; no personal application was controlled for this delivery.

The prior backend collapsed capture, focus, permission and unsupported-target
refusals into a missing token or `manual_ready`. This delivery retains a closed
reason, insertion method and numeric AX error where available. It preserves the
original transcript and existing clipboard-free insertion behavior. Runtime target
handles are omitted from newly serialized durable state. Existing history loads
without new metadata. The full contract is in [Rust bindings](../RUST-BINDINGS.md#macos-insertion-diagnostics-2026-11-14).

## Automated evidence

Environment for Cargo commands: `PATH="$HOME/.cargo/bin:$PATH"`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_RELEASE_DEBUG=0`, `CARGO_BUILD_JOBS=4`.

- `cargo test -p vara-core -p vara-platform -p vara-app`: 53 tests passed.
  Log: `/tmp/vara-insertion-core-platform.log`.
- `cargo build --release -p vara-desktop --features test-harness`: native Objective-C
  and Rust FFI build passed. Log: `/tmp/vara-insertion-desktop-build.log`.
- `target/release/vara-desktop --vara-test-service NEW_ISOLATED_PROFILE Samples/acceptance/audio/da-000.wav`:
  passed with `VARA_TEST_BACKEND_NO_SECRET=1`, `VARA_TEST_BACKEND_IDLE_MS=500`,
  `VARA_TEST_ASR_FIXTURE=TASK_OWNED_TRANSCRIPT_JSON`, `VARA_TEST_ASR_DELAY_MS=600`.
  Log: `/tmp/vara-insertion-service-final.log`. Real Service/store/audio lifecycle;
  inference and OS insertion explicitly mocked. Checks the persisted Unknown marker
  at dispatch, metadata, original/processed text, repeat completion without replay,
  existing no-speech, fallback, recovery, cancellation and concurrent-file behavior.
- `cargo check --release -p vara-desktop`: production configuration passed.
  Log: `/tmp/vara-insertion-desktop-default-check.log`.
- `git diff --check`: passed.

Core tests verify missing metadata and old token deserialization, omission of tokens
from new serialization, numeric diagnostics after restart, idempotent commit, and an
injected SQLite final-update failure leaving durable Unknown intact. A missing
result or a zero-row UPDATE (injected RAISE(IGNORE) trigger) cannot authorize a pre-dispatch write.

The native harness compiles and executes the same pure C policy header used by
`macos.m`: Accessibility denial, missing/dead target, app switch, unavailable field,
focus switch, keyboard role rejection, post-event denial, event allocation failure,
and before/after-send interruption status. This validates deterministic branch
policy, not real Accessibility APIs, permission grants, CG event delivery or editor
behavior. Native wire tests also verify closed diagnostic values and token handling.

The first Service-fixture invocation omitted its scripted ASR delay and failed the
existing concurrent-file "must still be in flight" assertion; this is retained in
`/tmp/vara-insertion-service.log`. Supplying the fixture's documented 600ms delay
made the bounded acceptance pass. No product workaround was added for that setup
mistake. Existing Security-framework deprecation warnings and a desktop dead-code
warning remain in build output.

## Remaining acceptance

No real target-field insertion, TCC grant/revocation, editor-specific behavior,
clipboard observation, or user-facing diagnostic recovery was manually exercised.
No release, signing, version change, push or GitHub update was performed. Windows
and Linux were not run locally; their adapter continues to return manual-ready
with `platform_unsupported` and nullable post-event capability.
