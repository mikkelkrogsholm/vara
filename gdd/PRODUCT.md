# Vara v1 — feature index

Derived from `gdd.json`; see PRODUCT-SPEC.md for accepted release scope and CONTRACTS.md for interfaces. The screens are a first design proposal. The earlier broad ideal exploration is preserved in reports/history/ideal-before-v1.

## Session

- **Run one dictation** (`action.use_vara`): A valid generation produces at most one completed result, grouped by invocation app, with separate delivery status.
- **Obtain original transcript** (`action.transcribe`): Recognized text survives later optional AI failure; successful recognition audio is not kept as history.
- **Use direct transcript** (`action.verbatim`): Direct mode stores/displays one text representation and preserves recognized text.
- **Apply optional prompt** (`action.rewrite`): Successful AI stores both versions. Failure uses the original through ordinary safe delivery, without extra approval.
- **Apply general cleanup choice** (`action.cleanup`): No text receives AI cleanup solely because a prompt exists.
- **Cancel current generation** (`action.cancel`): Accepted cancellation cannot later publish history or insert text; already committed work is reported honestly.
- **Retry failed recognition** (`action.retry`): A retry never extends expiry, duplicates a completed result or pastes into a different target.
- **Use saved result manually** (`action.retry_delivery`): Failed/unknown automatic insertion is never silently replayed. Manual copy is allowed to replace the clipboard by explicit intent.

## Desktop integration

- **Capture microphone audio** (`action.capture`): Return an opaque private capture handle; no binary audio enters UI state.
- **Inspect or request permissions** (`action.permissions`): Report actual grants and unsupported capabilities without pretending success.
- **Register activation shortcut** (`action.configure_shortcut`): A working user-selected shortcut invokes hold or toggle semantics; default profile is hold.
- **Choose input device** (`action.input_device`): Selection affects subsequent captures; active capture is not silently moved.
- **Report desktop capabilities** (`action.desktop_support`): Report known/unsupported capabilities and honest limitations; source portability is not a passed runtime test.
- **Deliver completed text safely** (`action.insert`): Automatic delivery never intentionally targets a changed field. A newer user copy is preserved. Unknown paste is never automatically replayed.

## Engines

- **Recognize speech** (`action.engine_speech`): Return original text without silently switching providers or uploading audio.
- **Process transcript text** (`action.engine_text`): Text processing never owns capture/history/delivery and cannot silently change destinations.
- **Prepare recommended local model** (`action.recommended_setup`): No terminal/account/key is needed for the default local speech path. Cancelled/incomplete models are not ready.
- **Configure optional engine** (`action.advanced_engines`): New providers implement the speech or text adapter contract without changes to session/history/desktop ownership.
- **Store personal provider key** (`action.credentials`): Direct cloud providers use the user key; no Vara login/billing proxy. Providers retrieve secrets through private handles.
- **Invoke optional installed text CLI** (`action.cli_processing`): CLI integrations are optional text engines, not a promised subscription speech backend or a claim of on-device inference.

## Preferences

- **Resolve invocation processing plan** (`action.resolve_processing`): Disabled app prompt does not activate AI. New profiles resolve direct/local. Focus/settings changes after invocation do not mutate the plan.
- **Choose speech and text destinations** (`action.processing_policy`): Local speech and direct text are fresh-profile defaults. Cloud is an explicit selection.
- **Set strict on-device processing** (`action.strict_local`): Strict-local plans never upload audio or text; readiness must identify blocked configuration.
- **Save general system prompt** (`action.presets`): Saving a prompt does not enable processing unless enabled=true. Transcript data is never appended to a system instruction string.
- **Save app processing preference** (`action.app_rules`): Enabled app AI uses this prompt automatically on subsequent invocations; other apps remain direct by default.

## History and lifecycle

- **Commit completed result** (`action.save_result`): History has one direct text or both processed/original versions. Retry of the same generation returns the existing result. No audio reference is stored.
- **Read app-grouped text history** (`action.history`): A focus change does not move the result to a different app. No audio is exposed as history.
- **Delete a completed result** (`action.delete_result`): Manual deletion removes both representations; absence is an idempotent success.
- **Apply text retention policy** (`action.retention`): No silent count limit. Cleanup never deletes a younger result or just one text representation.
- **Manage private temporary recovery** (`action.recovery_audio`): Retry cannot extend deadline. No successful audio persists intentionally, and physical cleanup failure is visible/pending. An offline app cannot promise wall-clock physical deletion.

## Experience

- **Present session status** (`action.session_feedback`): UI shows actual stage/locality and cancellation availability; never claims a completed paste or cleanup without its outcome.
- **Guide first use** (`action.first_success`): Default setup needs no terminal/account/key. A mock result or saved key alone cannot satisfy onboarding.
- **Show actionable readiness and errors** (`action.diagnostics`): User can identify unavailable engine, permission, failed stage and pending cleanup without exposing content.

## Remaining engineering acceptance

No additional product questionnaire is required to begin. These require implementation evidence:

- Select/benchmark the compact local speech/text runtime and UI toolkit; preserve the typed contracts regardless of implementation choice.
- Establish actual supported OS/hardware/desktop environments through macOS-first and final Windows/Linux runtime/installation checks.
- Measure representative Danish/English/mixed-language quality and cold/warm latency before setting release budgets.

Deferred features are listed in PRODUCT-SPEC.md and are not first-release graph obligations.
