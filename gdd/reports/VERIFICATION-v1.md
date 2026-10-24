# Vara v1 — verification

Executed 2026-10-23 against the current specification and HTML prototype.

GDD snapshot: `sha256:c48c8bf051dbb77f73d88a9d641e0a8fa93da75e5f982d9d8a278c8754ba6f1a`. Normative document hashes and report revisions match.

| Check | Actual outcome |
| --- | --- |
| GDD model schema, semantics and record integrity | PASS |
| Core action structural readiness (`action.use_vara`) | PASS; this does not prove implementation behavior |
| Screen readiness (`screen.history`) | UNKNOWN: requires a current trusted design-review decision; no such approval is invented |
| Generated graph projection | PASS |
| HTML prototype | 13 executed flow checks PASS; raw outcomes in prototype-v1.json |
| Narrow layout | History and connections inspected at 600px viewport; no page horizontal overflow observed |
| Generated Python view scripts | Parsed and executed successfully |

## Prototype checks

- App filter and original disclosure
- Per-app AI saved
- Activation and opt-in retention
- Fake key form validation and clearing
- Default direct session
- One-session direct/general-prompt choices
- Processed result with changed focus
- AI error preserves original
- Failed recognition and explicit safe retry
- Escape cancellation rejects late completion
- Cloud onboarding key and trial gates
- Local onboarding independent of cloud key
- Empty history

The prototype tests exercise DOM controls and simulated async outcomes, including cancellation after processing starts. Screenshots: design-v1.png and design-v1-narrow.png. The prototype hash is recorded in prototype-v1.json.

## Limits

No Rust/native application was implemented or tested in this task. No microphone, provider account, real secret store, OS shortcut, clipboard/focus handoff, durable history/audio expiry, installer or Windows/Linux runtime test was executed. Prototype fixture success is not evidence for those behaviors. The model/runtime choice, measured quality/latency and actual target support remain engineering acceptance work.

The first visual design is proposed and ready to review. GDD screen-readiness UNKNOWN records the absence of trusted design approval; it does not prevent delivering this requested design proposal.
