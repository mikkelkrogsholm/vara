# Vara — Warm precision prototype

Open [index.html](index.html) directly in a browser. No install, bundler, server,
account or network dependency is needed. The browser loads local classic scripts;
ES-module imports are intentionally avoided so `file://` remains supported.

The reference is [the selected first mockup](mockups/01-warm-precision.png).
[DESIGN.md](../DESIGN.md) owns the design system. This is a clickable simulation:
no real microphone, global shortcut, provider, persistent history or clipboard
integration. Fixture data resets on reload. Use **Afprøv forløb** in the unobtrusive
prototype footer for simulated dictation, first use, empty history and long content.

## Ownership and component structure

| File | Owns | Does not own |
| --- | --- | --- |
| `scripts/generate_tokens.py` | Read DESIGN.md frontmatter and generate CSS custom properties; `--check` detects drift | App layout or behavior |
| `styles/tokens.css` | Generated color, type, radius and spacing tokens | Independently edited visual decisions |
| `styles/app.css` | Component states, three-column shell, responsive drilldown, motion and focus recipes | Data or event handling |
| `js/components.js` | Pure Button, IconButton, Field, Toggle, Notice, Section, EmptyState, AppIcon, Waveform and BrandMark renderers | Session state, navigation or I/O |
| `js/views.js` | Sidebar, GroupedHistory, HistoryRow, ReadingPane, OriginalDisclosure, AppPreferences, Settings, HUD and dialog compositions | Provider/OS calls or mutation of saved settings |
| `js/data.js` | Clearly labeled fixture records, app identities and fresh demo state | Persistence or real credentials |
| `js/app.js` | One state owner, delegated actions, validation, modal/focus management, frozen session choices and generation guards | Theme values or repeated UI markup |
| `tests/smoke.playwright.js` | Observable user flows and responsive acceptance using an existing Playwright page | Native/provider acceptance |

The shared `Vara.ui` and `Vara.views` namespaces expose a deliberately small classic
script boundary. Scripts load in the order listed in index.html. Components escape
all dynamic text; trusted icon markup and fixed view templates are the only HTML.
A new screen composes existing primitives instead of copying button/form styles.
A new repeated visual decision belongs in DESIGN.md before it becomes a token.

## Maintenance loop

From the Vara repository:

```sh
~/Projekter/gdd/.venv/bin/python gdd/design/scripts/generate_tokens.py
~/Projekter/gdd/.venv/bin/python gdd/design/scripts/generate_tokens.py --check
node --check gdd/design/js/components.js
node --check gdd/design/js/data.js
node --check gdd/design/js/views.js
node --check gdd/design/js/app.js
```

The generator reuses PyYAML from the existing GDD environment; it installs nothing.
Do not hand-edit generated tokens or duplicate the palette in app.css. Component
rules such as borders, shadows and min/max widths stay in app.css and DESIGN.md.
Use standard formatters on source; generated tokens retain generator formatting.

For browser checks, temporarily serve the bundle:

```sh
python3 -m http.server 18769 --bind 127.0.0.1 --directory gdd
```

`tests/smoke.playwright.js` is a standalone async `smoke(page)` function. Run it
through the existing Playwright browser tool with that file as `filename`, or call
it from a Playwright harness that supplies `page`. It returns checks and geometry
as JSON and throws on failure. No Playwright package is added to the application.
The test uses port 18769 and restores the default reference-size fixture at the end.
Additional file-URL and keyboard checks are recorded in the verification report.

## Fidelity and deliberate differences

At the reference viewport of 1586 × 992 the column dividers are approximately
x=310 and x=808, matching the selected image's x=309 and x=807 within two pixels.
The mockup's major composition, typography contrast, olive/paper palette, grouped
history, orange copy action, original inset and floating HUD are implemented with
real controls and layout, not a screenshot background.

The default original disclosure is collapsed according to DESIGN.md. The comparison
screenshot opens it and starts a simulated listening session to match the reference
state. Initial HUD says “Klar · Eksempel”; it never claims an actual microphone is
listening. Counts say “dikteringer”, not stored recordings. Dates and outcomes are
fixtures. Modern native Mail/Notes artwork differs slightly from the generated
reference; code-app artwork is a small illustrative vector. Mail/Notes PNGs were
materialized from the local installed app resources for this prototype.

On narrow windows the UI changes to list/detail navigation rather than shrinking
three columns. Full text remains selectable, long results use the compact reading
role, fields retain labels and focus, and async completion preserves a form being
edited. Repeated or cancelled simulated work cannot publish a late history result.

## Evidence

See [prototype verification](../reports/VERIFICATION-prototype-warm-precision.md)
for executed checks, screenshot paths and limitations. Historical HTML is preserved
under `../reports/history/prototype-before-warm-precision/`. Actual desktop
integration still belongs to the Rust implementation and its target-platform tests.
