# Free shortcut selection — 2026-10-24

The user requested individual keys and arbitrary combinations, including Left
Option + Left Command. Removed the UI requirement for a modifier plus a normal
key. The recorder tracks physical key identity, supports modifier-only chords,
accepts Escape itself, preserves left/right distinction and waits for release
before enabling Use shortcut. It warns that ordinary keys/system shortcuts can
also affect the foreground app; it does not arbitrarily reject those choices.

On macOS, `Physical:` settings use a separate native CGEventTap adapter in
`crates/vara-platform/src/{shortcut.rs,macos_shortcut.m}`. It is listen-only,
leaves normal input untouched, suppresses repeats, releases the chord when any
required key is released and re-enables a disabled tap. Raw physical key events
are exposed to the UI only during explicit shortcut capture. Ordinary existing
shortcut settings retain the Tauri global-shortcut route. Registration failures
restore the previous binding; settings failures also roll back registration.

Global dictation is paused during recording of a binding. Shortcut callbacks are
queued onto a serial worker so start/release order is preserved without blocking
the native input callback. UI and native capture are combined per source;
virtual keyboards that send modifier flags without individual modifier events
retain those flags. Blurring or cancelling the recorder clears capture state.

## Verification

- 14 production browser UI test groups PASS, including single A, Right Option,
  Left Option + Left Command, Escape and virtual-keyboard modifier flags.
- Rust platform key-mapping test PASS: left/right distinction, modifier chord,
  arbitrary A+B, Fn, duplicate/unknown validation.
- Native Objective-C state-machine executable PASS using CGEvent objects (not
  posted input): left Option + left Command, wrong-side exclusion, release and
  autorepeat behavior for arbitrary A+B.
- Mac release bundle built successfully, 24.88 MiB.
- Native app reopened and actual recorder verified with single A and synthetic
  Ctrl+Shift+D. All modifiers are retained and Use shortcut becomes available.
  The saved user binding remains Alt+Space; no test binding was persisted.
- Formatting and diff checks PASS. Build intermediates removed after verification.

Computer-control input cannot press a modifier-only chord as a physical key
sequence; it rejects modifier-only pressKey and its modified mouse click sends
no corresponding physical keyboard transitions. Therefore the user's physical
Option+Command activation still needs their hands-on confirmation. The native
matching logic is tested separately as described above; this is not a claim of
microphone-to-field acceptance.

## Platform limits

Mac hardware key names/code mapping are isolated in the platform adapter.
Windows/Linux retain the existing global-shortcut backend for single keys and
standard modifier+key shortcuts. Their physical modifier-only/multiple-normal-
key backend is not implemented in this change; registration returns an explicit
adapter limitation rather than silently saving an unusable setting. This remains
a cross-platform release gap. Fn/media/system-reserved keys depend on events the
OS exposes; no promise is made that every hardware key is observable everywhere.
