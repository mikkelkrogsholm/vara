# Main UI localization — browser evidence

The screenshots use the actual desktop UI with the in-memory fixture host, not a native desktop session or live credentials. The Danish text in history and prompts is fixture/user content and is intentionally unchanged.

- `da/en-history-1280.png`, `da/en-settings-1280.png`: normal 1280 × 820 layout.
- `da/en-settings-720.png`, `da/en-onboarding-720.png`: 720 × 560 layout.
- `da/en-settings-200-percent.png`: 720 × 560 with computed text sizes doubled; navigation is scrollable and the test checks keyboard reachability and control bounds.

`desktop/tests/ui-main-i18n.playwright.cjs` covers both catalogs, history and recovery, copy/failure/delete dialogs, app rule persistence, key/endpoint dialogs, locale changes with live drafts, Unicode and markup safety, mutation/binding lifecycle, onboarding and the above layouts. `desktop/tests/ui-locale.playwright.cjs` covers persistence, revision ordering, failure rollback and retained transcription editor. `desktop/tests/ui.playwright.js` was run against an isolated fixture server for the existing Danish regression journeys. The main i18n unit test checks catalog parity, placeholders, plural rules and literal catalog references.

GDD model validation reports structural checks PASS and the pre-existing historical revision `decision.who.2026-10-26` UNKNOWN. These browser fixtures do not establish native OS behavior. File transcription and native host messages are tracked separately.
