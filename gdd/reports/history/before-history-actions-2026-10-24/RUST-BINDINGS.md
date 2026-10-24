# Vara v1 — bindings to the Rust implementation

Implementation inventory, not release acceptance. PRODUCT-SPEC.md and DESIGN.md
remain the normative requirements. The graph's logical interfaces describe
responsibilities; the following maps them to actual code rather than asserting
that every logical field is a serialized wire-schema field.

| GDD module | Implementation | Boundary |
| --- | --- | --- |
| component.session | `crates/vara-core/src/session.rs`, `desktop/src-tauri/src/service.rs` | Session transitions, frozen plan, generation cancellation, host orchestration |
| component.preferences | `crates/vara-core/src/model.rs`, desktop service | Validated settings, app rules, explicit one-shot choice, immutable resolved engine/prompt configuration |
| component.history | `crates/vara-core/src/store.rs`, desktop service | SQLite transactions, both text versions, retention, staged originals, recovery metadata and fixed expiry |
| component.engines | `crates/vara-engine/src/` | Local Parakeet Redux/Whisper and Candle Qwen, speech/text provider adapters, verified model installs, optional constrained CLI text adapters |
| component.desktop | `crates/vara-platform/src/`, Tauri host | Microphone worker, OS secrets, exact Mac focus target, direct accessible insertion, clipboard copy, shortcut/tray and native HUD |
| component.experience | `desktop/ui/`, `crates/vara-platform/src/macos_hud.m` | Shared tokens and pure components, live IPC controller, accessible dialogs, native nonactivating recording capsule |

`Settings` resolves to a cloned `Plan` at invocation, including complete selected
engine configurations, prompt, language and strict-local policy. This freezes the
actual values instead of relying on a mutable settings revision. `Destination`
contains app ID/name and an opaque native target token; no target-field text is
captured. `Dictation` contains original/optional processed text and delivery state.
No secret values or audio bytes cross the UI snapshot interface.

## Lifecycle and effects

1. Capture original app/field and resolve policy before audio capture.
2. Persist recovery metadata before recording; recognition failures retain audio
   only within the original 24-hour deadline.
3. Stage recognized original text durably before deleting successful audio.
4. Optional AI uses the frozen plan. Failure falls back to the original text.
5. Commit history before dispatch. Persist uncertain delivery before attempting
   insertion, so restarting never replays a paste.
6. Insert only into the same verified Mac field. Other/unknown targets become
   manual-ready. Automatic insertion does not modify the clipboard.
7. Accepted cancellation writes a durable tombstone; stale work cannot stage or
   commit after that generation was cancelled.

Desktop host IPC is one `command(action, payload)` dispatcher. Pure core/engine
APIs remain available to a future mobile host. OS bindings are kept outside them.

## Evidence and gaps

See [implementation instructions and limitations](../desktop/README.md) and
[UI review](reports/DESKTOP-UI-REVIEW.md). Browser fixtures verify presentation,
not native permissions, provider authentication or recognition quality. Native
Mac fixture replay verifies the real local pipeline without microphone capture.
Windows/Linux OS adapters, live cloud accounts and microphone-to-field acceptance
remain open. Codex CLI is deliberately reported unavailable. These are engineering
gaps against the agreed release, not changes to its scope.

## Local speech default — Redux

New profiles select `local.parakeet` / `LocalParakeet`, pinned to Parakeet Redux.
Existing saved engine choices are preserved; missing built-in entries are added
when settings load. Whisper Base remains an explicit alternative.

The runtime is voxrt at commit `4fab626c00708d9bc97f837d309e0d362094282b`, with
Metal on supported Apple builds and CPU fallback elsewhere. Models are cached;
each request has independent LongForm/VAD state. Cancellation is checked before
loading, between 10-second input chunks, before flush and before returning text.
A running kernel completes before the adapter exits; the host's generation gate
immediately prevents a cancelled request from saving or inserting late output.
Backend panics are converted to retryable recognition failures. Local models and
CLI authentication never require an API-key-store lookup.

Four pinned, size/SHA-256-verified Redux artifacts install under
`models/parakeet-redux/`. Its tokenizer cannot overwrite Qwen's tokenizer.
Downloads, privacy, retry expiry and text delivery use the existing contracts.

## Groq cloud choices and credential check

Groq remains optional cloud treatment: `groq.speech` is Whisper Large-v3 Turbo,
`groq.speech.accurate` is Whisper Large-v3, and new `groq.text` defaults to
`openai/gpt-oss-120b`. Stored model overrides are preserved. The GPT-OSS request
excludes reasoning from the response and requests low reasoning effort.

`check_groq` reads the stored Groq credential and queries the fixed Groq `/models`
endpoint; no recording/transcript is included. It reports whether configured model
IDs occur in the catalogue. This verifies key acceptance/catalogue presence, not
actual transcription, billing capacity or every model permission. The UI states
what was checked. A saved key does not enable cloud speech or AI automatically.

Groq requests are covered by local HTTP protocol tests for speech multipart,
text/system prompt, incomplete output, error redaction and cancellation. Live
account credentials and new native UI behavior were not exercised.

## OpenAI current API contracts

New profiles use `gpt-transcribe` for OpenAI speech through the fixed
`POST https://api.openai.com/v1/audio/transcriptions` endpoint. Multipart requests
use `response_format=json` and `languages[]` for this model; legacy configured
models retain the singular language field. Existing saved model overrides and
local/direct choices are preserved.

OpenAI text uses `POST https://api.openai.com/v1/responses`, with the frozen
system prompt in `instructions` and the current transcript in `input`.
New profiles use `gpt-6-luna` with reasoning effort `none`. The adapter explicitly
sets `store=false`, sends no tools/conversation history, and reads only completed
assistant `output_text` parts across the typed output array. Incomplete, refused
or empty responses fail into the existing original-text fallback.

`check_openai` reads the existing OS key under `open_ai` and calls the fixed
`GET https://api.openai.com/v1/models`. It shares the Groq catalogue-check UI and
reports configured model presence without sending recordings or transcripts.
Catalogue presence does not guarantee inference permission, quota or billing.
Local HTTP tests cover multipart and Responses payloads and failure handling;
UI fixtures cover key clearing, explicit cloud selection and check errors.
See [OpenAI integration evidence](reports/OPENAI-INTEGRATION.md).

## Web text-field insertion

Mac native text controls retain direct AXSelectedText insertion. Web-backed text
controls use targeted Unicode keyboard events to the captured PID after checking
the same AX field and frontmost app before each bounded UTF-16 chunk. This avoids
web editors that accept an AX assignment without updating their editor state.
Clipboard contents are never read or changed. Field text is not captured for
verification; caret movement confirms completion where supported. Submitted but
unconfirmed/partial delivery remains Unknown and is never automatically replayed.
Generated events are marked and ignored by Vara's shortcut listener to prevent
single-key bindings from triggering themselves. See
[web insertion evidence](reports/WEB-TEXT-INSERTION-2026-10-24.md).

## OS credential boundary (2026-10-24)

`has_secret` queries credential attributes rather than retrieving a password.
On macOS, `credentials.rs` delegates to `macos_credentials.m` through an isolated
instance of the signed executable, before Tauri initializes. The worker verifies
its actual parent's designated signing requirement; invoking its command-line
mode from another program is rejected without reading input or returning data.
Requests and results use anonymous pipes, never arguments, environment or files.

The macOS adapter uses the default user keychain and the existing service/account
names. New/replaced keys receive an explicit trusted-application ACL for the
signed executable. Other programs cannot silently decrypt them; normal Keychain
access remains under the user's control. Existing entries are not silently
rewritten; replacing a key applies the explicit ACL. Login-keychain ACL APIs are
legacy macOS APIs, also underlying the prior keyring implementation; they are
used deliberately for explicit per-application access on this unsandboxed app.

Interactive security prompts are disabled in the worker. Locked/unavailable or
unauthorized access produces actionable, redacted errors and a status retry in
the key dialog. A 15-second deadline kills and reaps a stalled worker, including
stalled pipe I/O. A timed-out mutation is reported as possibly completed; retry
queries metadata before the user repeats it. Key fields clear on save failure.
There is no automatic replay of credential mutations.

Windows/Linux retain the native keyring adapters; presence uses their attributes
API. macOS ACL enforcement, signed-worker authentication and deadline behavior
are Mac-specific. Windows/Linux behavior has not been exercised on this machine.
Test evidence: reports/CREDENTIAL-HARDENING-2026-10-24.md. No live OpenAI API test
or user-key read is asserted by this change.
