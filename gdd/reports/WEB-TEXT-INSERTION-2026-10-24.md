# Web text insertion — 2026-10-24

User confirmed that insertion works in TextEdit but not in Codex. Earlier Mac
AX permission problems have been repaired separately. Result metadata for the
later Codex attempt showed a captured field and `inserted`, while the user saw
no inserted text. This makes an AX success code insufficient evidence for that
editor. Computer Use explicitly prohibits access to com.openai.codex; no bypass
or automated interaction with the Codex UI was attempted after that denial.

## Change

- Native controls keep direct AXSelectedText insertion.
- AX text controls with a web-area ancestor use targeted Unicode keyboard input
  to their captured process; editable controls lacking writable AXSelectedText
  can use the same route. Generic buttons/web areas are rejected.
- Verify exact captured AX field plus frontmost PID before each chunk. Do not
  activate another app or steal focus. Focus change before submission yields
  manual-ready; partial or unconfirmed submission yields Unknown.
- Text is sent in at most 20 UTF-16 code units per event without splitting a
  surrogate pair. Modifiers are cleared on generated text events.
- Neither clipboard nor destination text is read or retained. Delivery can be
  confirmed from the selected-range caret advance. No automatic second attempt
  follows an uncertain write, avoiding duplicate insertion.
- Generated events carry a marker ignored by Vara's hotkey monitor, preventing
  recursion for arbitrary single-key bindings such as A.

## Verification

Native tests PASS for chunk boundaries/emoji and hotkey state including ignoring
Vara-generated key events. Signed Mac release build PASS, signature and audio
entitlement verified by the signed-build script. App restarted; native UI confirms
all three permissions remain granted after this rebuild. Existing browser UI
checks remain applicable because no frontend behavior changed in this patch.
GDD implementation provenance updated; model and generated graph validated.

Live Codex insertion remains pending the user's test: the computer-control tool
is not allowed to inspect or interact with Codex. The implementation targets web
controls generically and contains no Codex app-specific exception. Do not claim
confirmed Codex delivery until that user test succeeds. TextEdit direct insertion
had been confirmed by the user before this patch.

Build intermediates removed after verification; signed app retained.

## User acceptance

After installing and restarting the patched signed app, the user confirmed:
“perfekt - det virker.” This confirms the requested Codex insertion behavior
through the user's physical test. The prior automation restriction remains;
this is user-reported acceptance, not an automated Codex UI test.
