# Approved spacing integrated into Vara

The user approved the native interactive prototype on 2026-11-06 and requested
one-to-one implementation in the real app. Its immutable source is
[`../interactive/Vara-interactive.figcraft`](../interactive/Vara-interactive.figcraft),
SHA-256 `e5cab0844b0c92d2287f071f14d17f01e95c91cbdbadc398cd2dd24691e4d438`.
The accepted browser spacing input is `../proposal.css`, SHA-256
`dac99c8745dba0cf5dfce5284112e560bdada773e9fb2b695e6cc79c77a969ca`.
Those historical draft artifacts and baseline evidence remain preserved.

Canonical `gdd/design/styles/app.css` and `desktop/ui/desktop.css` now implement
the accepted spacing using existing tokens: 8 px label/control/help, 12 px peer
actions, 16 px related controls and 24 px groups/sections. No alternate stylesheet
is loaded. Existing components, real inputs, scrolling, keyboard interactions,
i18n and backend contracts remain in use. Palette, type, layout, 76 px history
rows, 36 px copy/delete actions, 32 px dialog inset, legal copy and native HUD
remain as approved. The interactive prototype's simulated example transitions
and discrete scroll frames are design presentation only.

## Evidence

- `capture-metadata.json`: actual loaded production CSS in WebKit 26.0,
  1280×820 CSS px at 2×, frozen clock, fictional host; no proposal injection.
- `comparison.json`: all 12 production captures have **zero differing pixels**
  against the preserved approved browser renders in `../render-evidence/`.
  Entries bind approved/actual image hashes and both native frame IDs.
- `checks.json`: all 38 reused bounded checks pass, including 1000×760, Danish
  long-label/text-spacing stress, 150% computed font size, visible-control focus,
  close/title separation and final-action/legal-copy reachability.
- `sources.json`: hashes bind actual canonical source, unchanged fixture/check
  inputs, approved CSS/native document and native route manifest.
- `test-results.md`: existing functional and localization regression results.

These are browser fixture screenshots of the actual UI sources. Native FigCraft
exports approximate another renderer. The separately managed native package
build is not evidence of native screenshot equality or OS integration testing.
No app install, version change, release, model/network/audio operation or personal
profile is included in this implementation evidence.

## Reproduce

Use the existing installed Playwright and cached browser; no download or Cargo
build is required. Start `python3 gdd/design/figcraft/serve.py` only when port
18770 is free, retain ownership, then stop that server after verification.
From repository root:

```sh
export PLAYWRIGHT_MODULE=/Users/mikkelfreltoftkrogsholm/.bun/install/global/node_modules/playwright
export PLAYWRIGHT_WEBKIT=/Users/mikkelfreltoftkrogsholm/Library/Caches/ms-playwright/webkit-2311/pw_run.sh
node gdd/design/figcraft/polish/implementation/verify.cjs
python3 gdd/design/figcraft/polish/implementation/compare.py
```

`verify.cjs` reads existing fixture transitions and bounded checks, intercepting
only capture/output paths and removing the historical proposal injection.
`compare.py` reads preserved approved references and writes only new evidence in
this implementation directory. A pixel mismatch fails comparison. No native
design, source proposal, manifest, baseline screenshot or historical integrity
verifier is rewritten. `render-evidence/` includes all 12 production states and
new responsive/stress captures.
