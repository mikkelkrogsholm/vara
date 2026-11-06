# Issue 48 — native continuation on unlocked Mac

The native app launches, Dictate/Stop runs real local Parakeet recognition, and
the native Transcription file chooser completes a public file with word timing.
The final 2026-11-06 continuation verified actual TextEdit insertion with an
empty clipboard. Global keyboard activation remains unverified; the user
explicitly skipped the final changed-focus test. Codex access remains blocked
by CUA.

This continues the frozen [original acceptance report](48-product-acceptance.md).
Structured observations and signed artifact provenance are in
[48-native-continuation.json](48-native-continuation.json).

## Earlier native observations (before the final continuation)

In the earlier observations below, MAIN performed app launches, clicks, focus
attempts and keyboard operations through CUA. In the final continuation, the
user physically foregrounded TextEdit as recorded below. The worker built/staged artifacts, ran offline checks and read only
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
the observed TextEdit item and its “Åbn Finderemne” action). At 18:50:24 UTC the
observer returned `com.apple.TextEdit`, `textedit_front:true`. This established
a normal UI activation route without shell activation or guard bypass.

A subsequent timed Dictate/Finder/TextEdit attempt encountered `loginwindow`;
CUA state at 18:51 UTC confirmed the Mac had locked again. No new session or
insertion was verified at that point. An unlock request was pending then; the
final continuation below supersedes that historical status. No unlock request
or changed-focus trial remains pending in this report.

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


## Final continuation — 2026-11-06, recorded at 21:05 UTC

The user physically foregrounded TextEdit. After the existing native Dictate
button initiated the guarded 6 s delayed fixture, the read-only foreground
observer confirmed `com.apple.TextEdit`. The real local Parakeet pipeline then
inserted the exact public fixture transcript into the actual TextEdit document.
Fresh native AX showed the transcript once. No scripted recognition, insertion,
focus control or guard bypass was used.

| Result | Read-only store evidence | Scope |
| --- | --- | --- |
| `1f18b3cd-bb55-4374-9056-d0cd39ea9199` | TextEdit origin, `inserted`, 12.84 s, original public transcript, no AI text | Successful stable-focus native insertion; fresh AX confirmed one transcript |
| `0194db8e-d116-4f94-898c-76a5a295e2bb` | TextEdit origin, `inserted`, 12.84 s, original public transcript, no AI text | A separately initiated run completed before a second document was created; this is not duplicate publication or a changed-focus test |

Clipboard observer session `52557`, marker `clipboard-check-8`, reported
`unchanged:true`, `before_type_count:0`, `after_type_count:0` during the first
successful insertion. This verifies preservation of an **empty clipboard only**.
Earlier comparisons with three types occurred without successful insertion;
non-empty clipboard preservation across insertion remains unverified. No
clipboard contents, type names or content hashes were exported.

A later attempted focus-change run guard-aborted when Codex became foreground;
no additional result was published. The user then explicitly instructed
“Spring den sidste fokusprøve over”. The final changed-focus test is
**skipped_by_user**, not passed and not awaiting another unlock. Changed-focus
manual-ready delivery remains unverified by native testing.

Read-only inspection at 21:05 UTC confirmed three total results: the original
Vara `manual_ready` row and these two distinct TextEdit `inserted` rows, with
zero recoveries and zero staged originals. The native file document
`660139c9-03f2-4e0e-877e-70e1eeab6c1d` remains `ready`. The JSON retains the
public transcripts and current result evidence; transient native target tokens
are represented only by their presence.

Current native acceptance status:

- Stable-focus TextEdit insertion: **passed**, through the guarded delayed
  public-file fixture and real native insertion path.
- Empty clipboard preservation during successful insertion: **passed**.
- Non-empty clipboard preservation during successful insertion: **not_tested**.
- Final changed-focus trial: **skipped_by_user**; no fallback success claimed.
- Global shortcut and hold/toggle activation: **not_tested**.
- Codex input: **not_tested**, due CUA's explicit safety restriction.
- Native public file selection/local transcription/word-control rendering:
  **passed** within the earlier recorded scope.
- Native playback, seeking and export UI actions: **not_tested**.

MAIN sent Cmd-Q to the temporary test app and obtained the normal
`/Applications/Vara.app` through CUA; running inventory showed the normal app.
Installed files were untouched. MAIN’s read-only check at 21:05 UTC found only
the normal app and its personal-profile backend running, with no temporary test
app/backend or Cargo jobs. Compiler intermediates had been cleaned: `target`
was 67,868 KiB; the retained staged signed app/profile directory was 35,640 KiB.
This report update performs no app control, deletion or rebuild.
There were zero cloud calls in this continuation. The aggregate remains
39 requests and USD 0.80 reserved, not measured billing. This partial native
acceptance does not close the remaining unverified requirements.
