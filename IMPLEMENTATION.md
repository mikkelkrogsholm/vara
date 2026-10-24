# Vara desktop implementation — current work

User mandate: build the agreed first desktop release, following GDD, DESIGN.md and
the selected prototype. Shared Rust engine must be reusable on iOS/Android later.
Existing uncommitted Swift changes are preserved. Branch: codex/vara-clean-write.

Architecture: pure Rust domain/storage in vara-core; local speech/text and provider
adapters in vara-engine; microphone/credentials/native targeting in vara-platform;
Tauri 2 desktop shell and shared generated Warm precision UI components. No mobile
UI implementation is included in this release. Native APIs stay outside the core.

Current boundary: complete the agreed desktop flows, compile/package Mac app,
exercise actual local inference using Samples/da_test.wav and available local model,
run meaningful core/provider/storage/session tests, and verify rendered live UI.
Mac-first development; Windows/Linux runtime support remains explicitly unverified
until actual target testing. Never report a placeholder/stub as working behavior.

Implemented: Rust workspace, Tauri desktop host, shared Warm precision UI, local
Parakeet Redux/Whisper/Qwen inference, history/recovery/cancellation, OS credentials and provider
adapters. See desktop/README.md for build instructions and explicit remaining gaps.

Verified: 30 core/model/HTTP contract tests; 13 production-UI browser check groups; native
Mac main window and HUD inspection; real local fixture pipeline with manual delivery
and successful audio cleanup. Latest optimized sample: ASR 0.41s, local text 2.57s.
The sample has recognition errors; this is not quality acceptance.

The agreed release is not yet fully accepted: Windows/Linux native focus/HUD
adapters, real microphone-to-field/hardware shortcut testing and live cloud account
verification remain open. Codex CLI is unavailable pending a verified tools-off
contract. Existing Swift work is untouched.


Redux integration: new-profile speech default is Parakeet Redux via pinned voxrt,
Metal on Apple/CPU fallback, isolated model files and actual installer verification.
Existing saved speech choices are retained. Local inference no longer reads the
OS API-key store. See gdd/reports/REDUX-INTEGRATION.md for current acceptance.

Groq: optional Turbo/Large-v3 speech choices, current GPT-OSS 120B text default,
key/catalogue probe and local HTTP protocol tests. Live keys/audio not used.
See gdd/reports/GROQ-INTEGRATION.md.

OpenAI: current GPT-Transcribe speech and Responses API / GPT-6 Luna text, with OS-key catalogue check. See [integration evidence](gdd/reports/OPENAI-INTEGRATION.md). Saved model overrides remain unchanged.

Headless verification: 30 workspace tests plus the real Redux cancellation/reuse test and 13 browser groups pass. See [verification report](gdd/reports/HEADLESS-VERIFICATION-2026-10-23.md).

## Live Groq verification — 2026-10-23

Reused the legacy project credential without exposing it. Live model catalogue, both Whisper models and GPT-OSS text pass. Fixed duplicate multipart response_format found by live speech; 12 HTTP tests pass with field-uniqueness regression. OpenAI key inaccessible (old Keychain read timed out; no key in documented env sources). See gdd/reports/LIVE-CLOUD-VERIFICATION-2026-10-23.md.

Free shortcuts (2026-10-24): Mac physical single-key/side-specific/chord recording and global adapter, paused capture, registration rollback. 14 UI groups, platform mapping and native chord tests pass. Windows/Linux physical chord adapter remains open. See gdd/reports/FREE-SHORTCUTS-2026-10-24.md.

Mac global shortcuts (2026-10-24): added separate Input Monitoring preflight/request/UI, retry on focus after grant, and stable Developer ID signing script. 15 UI groups and native matcher tests pass; signed app rebuilt and verified. User must authorize the signed identity; other-app physical activation remains pending. See gdd/reports/MAC-GLOBAL-SHORTCUT-PERMISSIONS-2026-10-24.md.

Mac web-editor insertion now uses focus-checked targeted Unicode input with uncertain-delivery handling; native TextEdit keeps AX insertion. See gdd/reports/WEB-TEXT-INSERTION-2026-10-24.md for verification and pending live Codex test.

## Credential hardening — 2026-10-24

Credential presence now queries attributes. The Mac adapter uses explicit signed-app
ACLs for new/replaced keys and an authenticated worker with a 15-second deadline.
Locked/denied access and uncertain mutation timeouts have actionable errors and
metadata retry; entered secrets clear after failed saves. Disposable native/Rust
probes and all 16 UI groups pass. See
[gdd/reports/CREDENTIAL-HARDENING-2026-10-24.md](gdd/reports/CREDENTIAL-HARDENING-2026-10-24.md).
Live OpenAI testing awaits a user-created key. Existing keys were not read or
silently rewritten. Build intermediates are removed after verification.

## Local app artwork and direct actions — 2026-10-24

History, selected results and app preferences use a shared local app-artwork
component. Mac icons come from NSWorkspace by app identity, with monogram fallback.
Copy/delete icons now act directly from rows and selected-result headers; deletion
still confirms, and the intermediate two-option modal is removed. Native icons,
all 17 UI groups and the signed app build pass; the actual ChatGPT/TextEdit icons
and action buttons were visually checked in the reopened app. See
[gdd/reports/HISTORY-ACTIONS-2026-10-24.md](gdd/reports/HISTORY-ACTIONS-2026-10-24.md).

## Visible connection status and key test — 2026-10-24

Connections now shows metadata-only stored/missing/unavailable key states and
exposes Test nøgle directly for OpenAI/Groq. Saving/deleting invalidates cached or
pending validation; settings edits remain intact. All 18 UI groups and the signed
Mac build pass. A live test through the native button confirmed the user's OpenAI
key and both configured models in the catalogue, without sending audio/text.
Speech/text inference remains a separate test. See
[gdd/reports/CONNECTION-STATUS-2026-10-24.md](gdd/reports/CONNECTION-STATUS-2026-10-24.md).

## Settings v2 — 2026-10-24

Implemented the approved Diktering / Modeller og nøgler / Privatliv structure and
shared section, model, processing-choice and policy components. Both active models
now live on Diktering; strict locality and retention live on Privatliv. Existing
provider, endpoint, CLI, permission, history and app-rule capabilities remain.
Contextual setup preserves pending edits. DESIGN.md §18/§19 and Rust bindings
updated; all 23 UI groups and the signed Mac build pass. Native pages visually
checked without changing saved preferences. See
[gdd/reports/SETTINGS-V2-2026-10-24.md](gdd/reports/SETTINGS-V2-2026-10-24.md).
