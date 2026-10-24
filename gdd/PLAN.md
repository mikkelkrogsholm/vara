# Vara v1 — implementation plan

Status: implementation exists in `crates/` and `desktop/`. This remains the agreed
acceptance plan; implementation does not imply every criterion below has passed.
See [Rust bindings](RUST-BINDINGS.md) and [current evidence and gaps](../desktop/README.md).

## 1. Local dictation through the whole path

Choose and validate the smallest suitable Rust-compatible local speech runtime
and desktop toolkit on macOS. Use one app with the six responsibility boundaries
in CONTRACTS.md; crate boundaries may follow actual implementation needs.
Implement a real local direct session: user-selected hold/toggle shortcut,
microphone, generation cancellation, original transcript, durable result and
focus-safe delivery. Include known/unknown target handling and clipboard ownership.

Keep domain contracts platform-neutral. Put OS behavior behind Desktop adapters
from the first slice; do not spread macOS calls through domain/presentation code.
Create Windows/Linux adapter/build boundaries now, with runtime verification later
as agreed. Do not claim their support from compilation alone.

Acceptance: real Mac dictation reaches its original field, changed focus becomes
manual-ready, direct mode never invokes text AI, and late cancelled work does not
save or paste. A storage failure and uncertain dispatch remain recoverable.

## 2. First use, history and recovery

Implement an empty profile, guided local model download, actual permission and
shortcut checks and a real trial. Add app-grouped history, original/processed
presentation, explicit copy/delete and optional timed text cleanup. Implement
successful-audio deletion, failed-recognition retry with the fixed 24h deadline,
startup/resume orphan purge and truthful pending-cleanup states.

Acceptance: no account/key/terminal on the default path. Restart/crash/expiry tests
preserve text appropriately, do not replay pastes, and do not offer expired audio.
A result never contains audio and both text versions delete together.

## 3. Optional processing and providers

Implement immutable invocation policy resolution, general and app system prompts,
explicit app enablement and raw-text fallback after AI failure. Separate the
speech and text provider contracts. Include the accepted provider families,
Ollama/custom text endpoints and optional genuinely installed Claude Code/Codex/
Gemini CLI text paths. Personal direct-API keys use OS secret storage. Capabilities,
authentication, actual locality and readiness must be reported separately.

Acceptance: app/prompt/engine remain snapshotted even if focus/settings change;
cloud is never an implicit fallback; strict on-device rejects LAN/cloud/unknown
locations; optional CLI absence is actionable rather than a fabricated login.
Provider adapter checks use redacted fixtures before deliberate real account tests.

## 4. Main experience and end-to-end acceptance

Turn DESIGN.md and design/index.html into the native main window, configuration,
onboarding and nonactivating HUD. The HTML is a flow/visual proposal, not required
production markup. Wire each surface to domain outcomes. Essential controls remain
keyboard accessible, labeled and usable with text scaling and reduced motion.

Acceptance: the DESIGN.md walkthroughs run against real application behavior;
readiness/status distinguish model installation, permission, auth, processing and
delivery. No prototype-only controls or example history ship as real user data.

## 5. Release verification on all three targets

Finish actual Windows/Linux testing and installers; verify macOS packaging too.
Record exact OS/hardware/desktop environments, global shortcut and insertion
capabilities, credential storage and local runtime availability. Test representative
Danish/English/mixed dictation and measure cold/warm activation, recognition and
delivery separately. Use measurements to choose the recommended compact model and
set honest latency/quality baselines.

Release requires the whole accepted scope, not just the first vertical slice.
File transcription, wake word, standalone Ask AI, end-user CLI, migration and a
Vara cloud service stay outside this plan. Subscription speech stays research.

## Contract and design assessment

The current GDD graph declares ownership, types, calls, effects, typed errors,
risks and scenarios. Structural validation is distinct from behavior tests.
Screens are a proposed design: there is no invented human design-review approval.
Runtime/model choice, target environment coverage and measured quality/latency are
explicit acceptance questions, not unresolved major product directions.
