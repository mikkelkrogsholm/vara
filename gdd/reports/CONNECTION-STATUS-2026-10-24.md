# Visible credential status and direct key test — 2026-10-24

The user added an OpenAI key and requested clear stored-key status plus an easy
way to test it.

## Delivered

Connections shows distinct loading, stored, missing and unavailable key states.
Stored credentials expose Test nøgle directly for OpenAI/Groq. Key management
uses the same test action. Presence queries still read metadata only; stored keys
are never returned to the UI or logged. Saving/deleting refreshes presence and
invalidates prior test results. In-flight responses for an old/replaced key are
ignored. Status updates patch only provider rows, preserving unsaved settings.

The test reuses the existing fixed-provider model-catalogue endpoint. Visible
progress, success and errors appear beside the provider; errors from a retest
clear prior success. Explanatory text distinguishes API/catalogue access from a
successful speech/text inference. No audio, transcript or prompt is sent by this
test. Validation state is session-local; stored-key presence is reloaded from the
OS store when the connections view first opens.

## Verification

- All 18 production UI groups PASS: includes visible presence, test success/error,
  missing-key state, save/delete refresh, pending-test disabling and stale response
  rejection after key replacement. Test fixtures use fictional keys.
- Signed Mac app build PASS, including deep/strict signature and audio-input
  entitlement checks. Updated app reopened and visually checked.
- Actual OpenAI key test run through the new native UI button: PASS. OpenAI
  accepted the user's stored key; configured speech `gpt-transcribe` and text
  `gpt-6-luna` both appeared in the returned model catalogue. The UI displayed
  “Nøgle gemt · API-adgang testet”. No credential value was read back through UI,
  copied to tool output or written into this report. No user key was changed.
- No live transcription or text generation was performed by this catalogue test.
- GDD model/record integrity and generated visualization validate; unrelated broad
  release-readiness checks remain UNKNOWN.

Build intermediates are removed after verification; the current app is retained.
