# Global shortcuts and macOS permissions — 2026-10-24

User reported that the physical modifier chord worked only while Vara had focus.
Native UI inspection showed microphone access granted but Accessibility missing.
The old bundle had only a linker ad-hoc signature, an unbound Info.plist and a
hash-derived executable identifier, rather than a stable signed app identity.
The physical-key listener also lacked an explicit Input Monitoring preflight
and the UI did not expose that permission separately.

## Correction

- Added `CGPreflightListenEventAccess` and explicit user-requested
  `CGRequestListenEventAccess` / macOS Input Monitoring settings navigation.
- Snapshot/UI now distinguish microphone, Input Monitoring (global shortcuts)
  and Accessibility (focus/automatic insertion).
- Physical global registration fails explicitly without Input Monitoring, instead
  of implying a listener is globally ready. Local shortcut capture can still
  fall back to DOM events without that permission.
- On focus return, failed physical-shortcut registration is retried. The saved
  user binding remains Left Option + Left Command.
- Added `scripts/build-desktop-macos.sh`, which requires an existing Apple signing
  identity and defaults to a local Developer ID identity, without embedding a
  personal certificate in the repository. It verifies the resulting bundle.

The rebuilt app passed `codesign --verify --deep --strict`; its identifier is now
`dk.vara.desktop`, Info.plist is bound and TeamIdentifier is set. It is signed
with the existing Developer ID certificate, not notarized. The script and README
provide the stable signing route for subsequent builds. No privacy database was
edited, no permissions were reset and no security protection was weakened.

## Verification

15 browser UI groups PASS, including independent Input Monitoring denial,
request action and status refresh while Accessibility is already granted.
Native chord-state tests PASS. Formatting/diff checks PASS. Signed Mac release
build PASS (24.78 MiB). App relaunched and shows all three permission rows.

The new signed identity currently requires user authorization. Its actual
preflight reports Input Monitoring missing; the native UI presents the precise
request instead of a false ready state. An explicit user confirmation was
requested before invoking this additional permission. Physical dictation from
another app remains pending this grant and hands-on verification; do not infer
end-to-end success from the browser or state-machine tests.

Build intermediates were removed after verification, retaining the signed app.

Reference: https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess()

## Live permission follow-up

After the user completed Touch ID, the current bundle was added to Input
Monitoring and restarted using macOS' Quit and Reopen. Both the system toggle and
Vara's own preflight then confirmed global keyboard access.

The Developer ID build had enabled hardened runtime without the separate
`com.apple.security.device.audio-input` entitlement. Added the minimal entitlement
file and Tauri macOS entitlements configuration, re-signed the existing bundle
without rebuilding, and verified the signature and embedded entitlement. The
signed-build script now asserts this entitlement so the omission is caught.
No library-validation or other security protections were disabled. The user's
microphone consent plus a full process restart then produced Adgang givet for
microphone in the app.

Accessibility remained false despite system toggles being on. The target Vara
entry was identified through Show in Finder as the current bundle path, removed
and re-added using System Settings; the other original application entry was not
purged. Earlier toggles were returned to their original on state. Accessibility
still requires verification; do not claim automatic insertion works yet.

A blank TextEdit document is ready. Automated modified-click input did not
produce a shortcut event; computer control cannot synthesize a physical
modifier-only keypress. A hands-on Option+Command test was requested from the
user to distinguish that limitation from application behavior.

## Accessibility repaired after global shortcut confirmation

The user confirmed the physical shortcut opens the HUD from another app.
Inspection of earlier result metadata (without logging transcript content) showed
no captured field token and `manual_ready`, consistent with missing AX access.

A fresh repair was required: quit Vara, run Apple's scoped
`tccutil reset Accessibility dk.vara.desktop`, launch the signed app, request AX
access from Vara, enable its newly off toggle in System Settings, and fully quit
and reopen the app. No microphone/Input Monitoring approval was reset.
After that sequence, Vara's own live UI confirms all three permissions granted,
including Accessibility. This replaces the earlier stale on-toggle state.

An empty TextEdit document was focused for a final user-assisted microphone to
insertion test. End-to-end insertion remains pending that test result. No new
binary build was required, and prior working history was preserved.
