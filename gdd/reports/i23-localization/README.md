# UI localization verification — issue 23

The production language set is Danish and English. The longer strings in the
`long-settings` screenshots are test-only translations, appended in memory to
the actual catalog; they are not shipped. Screenshots use synthetic, sanitized
fixtures and never a user profile, recording, key or personal file. The HUD
images are the unchanged AppKit fixture output from the issue 22b run described
in [the native report](../i22-localization/README.md).

The browser checks render the actual desktop UI with an in-memory host; they do
not prove a native session or model/provider behavior. The static catalog check
compares keys, placeholder names and the actual locale's plural categories;
checks all 441 runtime messages in each declared production locale; checks
literal UI keys and host runtime codes; and verifies Intl number/date/plural
formatting. The `fr` fixture is not a production locale: Rust substitutes only
its test metadata and runs the ordinary serde parser, ordered system resolver,
SQLite save/reopen, and runtime formatting (English fallback). Browser tests add
the test catalog and endonym, use the actual picker and host command, and check
DOM binding, HTML language, Intl formatting and runtime fallback and formatting.

| Coverage | Evidence and boundary |
| --- | --- |
| Profiles and selection | Real temporary SQLite: new System, missing legacy field Danish, malformed/unknown English, explicit choices and reopening. Browser fixture tests System/explicit changes, revision ordering, save failure rollback, and saving only the language while settings have unsaved edits. |
| History, onboarding, settings, app AI | Both locales exercise history/copy/delete/failure/recovery; onboarding steps; app rule and user prompt; tabs, keys and endpoint dialogs. Key input is synthetic. Browser locale changes retain active drafts, focus, selection, modal identity and settings. |
| File transcription / Who / export | Synthetic media and in-memory host exercise import, queued/cancel/retry, editor, playback, AI, Who, speakers and TXT/MD/JSON/SRT/VTT exports. The same editor and audio objects, selection and playback position survive a locale switch. Fixture source text, speaker names, user prompt and controlled mutations are checked. No model execution or real audio is involved. |
| Effects and runtime | Simulated recording, download, transcription, AI and Who work do not trigger extra host effects on language switches. Runtime errors, unknown redaction and nested causes are tested; startup failure selects the first supported preferred language even before catalog initialization. |
| Layout, keyboard and accessible names | Both locales at 1280 × 820 and 720 × 560; settings/editor/Who/export are tested with computed element font sizes doubled at the minimum viewport. Who/export title, close and primary action are each scrolled into view and bounds-checked. Long test translations check essential settings controls, labels and keyboard dialog closing. Dialog labels, sidebar names and individual accessibility labels are inspected by DOM/role checks; this is not a screen-reader session. The shortcut fixture covers Tab navigation, Escape and side-specific modifier chords. |
| Native | Reused issue 22b AppKit HUD, menu, file-panel and Foundation fixture evidence; native state-machine fixture constructs events for left Option+Command without posting input. The real microphone permission prompt, installed app and native Windows/Linux were not tested. OS-owned permission prompts follow OS language. |

Reproduce the main checks using an existing target cache and installed browser:

```sh
CARGO_BIN=/Users/mikkelfreltoftkrogsholm/.cargo/bin/cargo \
CARGO_TARGET_DIR=/tmp/vara-i19-target \
PLAYWRIGHT_MODULE=/Users/mikkelfreltoftkrogsholm/Projekter/QTI2025/frontend/node_modules/playwright \
PLAYWRIGHT_CHROMIUM='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
VARA_I18N_SHOTS=/tmp/vara-i23-final \
scripts/check-localization.sh > /tmp/vara-i23-check.log 2>&1
```

The complete entry passed on 2026-11-04: core test groups 5, 11, 10 and 2;
extra-language group 3; desktop library 3; desktop cargo check; catalog and
runtime checks; all four locale browser suites; and legacy UI/transcription
fixtures (23 and 17 checks). macOS emits pre-existing deprecated Keychain API
warnings. Separately, `clang -fobjc-arc -fblocks -framework AppKit -framework
ApplicationServices -framework IOKit desktop/tests/shortcut-native.m -o
/tmp/vara-i23-shortcut-native && /tmp/vara-i23-shortcut-native` passed. The
feature-only Rust locale test was rerun after adding the runtime fallback assertion. The main browser suite was rerun after adding explicit long-label navigation and 720 × 560 screenshots. The transcription suite was rerun after adding the dialog title and close-control checks.

After source hash updates, `python3 gdd/render_contracts.py` regenerated the
contract view. `gdd validate gdd/gdd.json --profile model --output
/tmp/vara-i23-gdd.json` returned model/schema PASS and the known historical
`decision.who.2026-10-26` snapshot UNKNOWN, not a new localization failure.
The source extension procedure and the native work needed for another release
language are in [RUST-BINDINGS.md](../../RUST-BINDINGS.md).
