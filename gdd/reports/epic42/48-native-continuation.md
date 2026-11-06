# Issue 48 — native continuation on unlocked Mac

The native app launches, Dictate/Stop runs real local Parakeet recognition, and
the native Transcription file chooser completes a public file with word timing.
OS insertion and global keyboard activation remain unverified. The initially
unlocked Mac later locked again; Codex access is separately blocked by CUA.

This continues the frozen [original acceptance report](48-product-acceptance.md).
Structured observations and signed artifact provenance are in
[48-native-continuation.json](48-native-continuation.json).

## Observed native behavior

MAIN performed all app launches, clicks, focus attempts and keyboard operations
through CUA. The worker built/staged artifacts, ran offline checks and read only
the task-owned fixture databases. Neither worker nor helper controlled another
app or synthesized OS key events.

- Native launch succeeded using CUA with the full temporary app bundle path.
  The actual current WebView showed local Parakeet ready and all three existing
  permissions granted. No TCC, ACL or permission changes were made.
- `Physical:F18` (native keycode 79) was recognized in the native shortcut capture
  UI. CUA F18 attempts in TextEdit and Vara did not start a session. This does
  not establish successful global activation or diagnose a product defect.
  Hold/toggle activation remains **not_tested**.
- The existing Dictate and Stop buttons produced one real local transcript from
  public `da-000.wav` (12.84 s). It was stored under `dk.vara.desktop` / Vara with
  no target token, no AI text and `manual_ready` delivery. TextEdit stayed empty.
  Read-only SQLite inspection confirmed one result, zero recoveries and zero
  staged originals; the isolated audio directory was empty. This proves native button-to-local-ASR/history behavior,
  not insertion into another app.
- Two attempts used the opt-in timed button fixture. MAIN immediately directed
  CUA AXRaise and AX click, or a screenshot-coordinate click, to TextEdit. After
  at least 12 s both attempts showed “An unknown error occurred”; neither added a
  result or recovery, and TextEdit stayed empty. No focus or insertion guard was
  bypassed. Successful stable-focus insertion and changed-focus fallback remain
  **not_tested**; later foreground observations below explain why these attempts do not
  establish insertion behavior.
- One read-only clipboard comparison timed out. Three later comparisons reported
  `unchanged:true`, with three types before and after. No insertion succeeded
  during these comparisons, so they do not prove preservation across successful
  insertion. Clipboard contents, types and hashes were never exported.
- CUA explicitly rejected access to `com.openai.codex` for safety reasons.
  No alternative identifier, path or scripted control was attempted. Codex input
  remains **not_tested** because of this tool restriction, not a locked Mac.

## Fixture and validation

The test-only code commit is `216cf4d7f6be2a9d8d0febf022208588fa931c0b`.
With the `test-harness` feature and explicit environment opt-in, the existing
Dictate action waits 6 s, calls the real `Service::start(Direct)`, then auto-stops
6 s later. It requires the actual captured foreground app to be TextEdit with a
native AX target token, an isolated temporary profile, a WAV fixture, strict
local Parakeet settings and disabled AI/app rules. Cancellation invalidates the
pending timer; auto-stop checks the same generation. Recognition, permissions,
focused-field checks and native insertion remain the production paths. With
feature or environment absent, normal button behavior remains unchanged.

The input remains the full 12.84 s public WAV; timer duration does not represent
microphone recording duration. No ambient microphone samples, scripted ASR or
scripted insertion were used. Models were referenced through an isolated-profile
symlink and used read-only; no download, install or model deletion occurred.
The saved personal app profile, history and keys were not changed.

| Check | Outcome | Log/artifact |
| --- | --- | --- |
| Three focused fixture guards: bounded timers, local/direct settings, TextEdit identity/native token | Passed | `/tmp/vara-epic42-native-fixture-test.log` |
| Six normal-feature desktop library tests | Passed | `/tmp/vara-epic42-native-normal-lib-test.log` |
| Release build with test harness | Passed | `/tmp/vara-epic42-native-fixture-build.log` |
| Developer ID signing and strict verification | Passed, same identifier/team and existing entitlements | `/tmp/vara-epic42-native-stage/timed/app-verify.log` |

The timed bundle is `/tmp/vara-epic42-native-stage/timed/Vara.app`, executable
SHA256 `73801b08a48c06286fbecac1b642f4db6f9b09db93131f9d04e23051854b6b51`.
The original normal executable and original build bundle were preserved.
No install, push, merge, paid request or worker inference occurred.

## Foreground diagnostic and remaining evidence

A separate signed read-only diagnostic returns only
`NSWorkspace.frontmostApplication.bundleIdentifier` and `textedit_front`.
It performs no activation, UI control, clipboard access or event injection.
MAIN observed `ai.storyteller.figcraft`, `textedit_front:false` at 18:48:12 UTC
following the CUA TextEdit AX/coordinate focus attempt. CUA targeting therefore
had not established TextEdit as the actual foreground app at that observation.
MAIN then used standard Finder UI (a separate window, `/System/Applications`,
the observed TextEdit item and its “Åbn Finderemne” action). At18:50:24 UTC the
observer returned `com.apple.TextEdit`, `textedit_front:true`. This established
a normal UI activation route without shell activation or guard bypass.

A subsequent timed Dictate/Finder/TextEdit attempt encountered `loginwindow`;
CUA state at 18:51 UTC confirmed the Mac had locked again. No new session or
insertion was verified. MAIN has one asynchronous unlock request pending.
The signed timed app and task-owned profile are retained for quick resumption;
compiler-cache cleanup belongs to MAIN after associated jobs are terminal.

## Native public file import

MAIN used the actual Transcription WebView's Choose files control and native
NSOpenPanel to select public `da-000.wav`, then selected local Parakeet.
Full native AX showed Ready, “On device · Word timestamps”, all 19 word buttons
and player/export controls. Read-only task-owned SQLite confirms document
`660139c9-03f2-4e0e-877e-70e1eeab6c1d`, `stage:ready`, on-device Parakeet,
12.84 s duration, one segment and 19 ASR-timed words. Source SHA256 was revalidated
unchanged as `823386a11c070104e26aaa6761cc7db68159a6d2e4e4fd83f16ebfbccb2542f7`.
The public document and raw transcript are retained in the companion JSON.

The raw final word/segment end is 12.88 s, 40 ms beyond decoded 12.84 s; it was not
corrected. This is observed output, not a broad timing-quality claim. Native
playback, seeking and export UI actions were not exercised in this continuation;
visible controls alone are not a playback/export pass. Earlier frozen export
and browser evidence retains its original scope.
