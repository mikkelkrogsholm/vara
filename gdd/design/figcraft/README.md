# Vara in FigCraft

Open `Vara.figcraft` in FigCraft. The file documents the desktop UI from revision `26f61b5` at 1280 × 820. It is an editable starting point, not an interactive prototype. Text uses text layers, icons and shapes are vectors, and the only bitmap is the actual Mail app illustration. The content is fictional test data.

- **01 · Primary screens:** history and original, AI per app, all three settings tabs, transcription import and document view, the AI tab, and scrolled continuations of settings.
- **02 · Dialogs and states:** processing choices, deletion, Who download, known count and uncertainty, three setup steps, shortcut, key, app picker, export, endpoint and model, and three HUD states.
- **03 · Components:** parts extracted from the actual rendered elements. Sidebars with navigation, primary buttons, fields and save bars are used as instances in the screens. Variants with different text, positions or states each have their own main component.

The file has 31 screen/state frames, 33 main components and 54 instances. The CLI's `info` reports 3,834 nodes; the native JSON contains 1,375 text layers. Text is split by visible line and can be edited, but the screens have no responsive auto layout or working actions. The components also contain editable text and vectors. Dialog screens are separate snapshots to preserve layer order under the backdrop.

## Visual checks and limitations

`references/` contains browser captures of the current UI with a Tauri mock and fictional fixtures adapted from `desktop/tests`. `exports/` contains selected exports from the final native file. `comparisons/` shows the same pairs side by side, browser on the left and FigCraft on the right. Comparisons cover history, app rule, dictation, transcription editor, Who and scrolled settings. They show that positions and colors are close to the reference but not pixel-identical: text weight, font rendering, some spacing/ellipsis, focus indicators, shadows and native controls can differ. FigCraft reports no missing fonts in `info`; browsers and FigCraft can still render system fonts differently. The browser reference is Chromium, not a measurement of the running macOS WebView.

HUD frames 18–20 are reconstructed from `crates/vara-platform/src/macos_hud.m`, the token colors and `desktop/src-tauri/src/service.rs`: 520 × 76, actual labels and controls, and an illustrative audio level. They were **not** captured from a running macOS HUD and have no pixel reference. The actual app determines the app icon and audio meter dynamically.

## Rebuild

FigCraft CLI 0.15.0 can assemble the native file from the included SVG and fragment data without running the app:

```sh
python3 gdd/design/figcraft/build.py
'/Applications/FigCraft Tools/current/figcraft-cli' info gdd/design/figcraft/Vara.figcraft --json
'/Applications/FigCraft Tools/current/figcraft-cli' export gdd/design/figcraft/Vara.figcraft -o /tmp/vara-history.png --node 01-history
```

Set `FIGCRAFT_CLI` to the local CLI path if installed elsewhere. `build.py` creates components from the extracted parts, imports the screens and places instances using the CLI; no screenshots are loaded into the document.

Recapturing browser references requires Node, a local Playwright installation and a compatible Chromium. Start `python3 gdd/design/figcraft/serve.py` in a terminal, then run:

```sh
PLAYWRIGHT_MODULE=/absolute/path/to/node_modules/playwright \
PLAYWRIGHT_CHROMIUM=/absolute/path/to/chrome-headless-shell \
node gdd/design/figcraft/capture-runner.cjs
python3 gdd/design/figcraft/build.py
```

Stop the server afterwards. If Playwright and its browser are in their standard locations, the two environment variables can be omitted. The capture scripts contain a frozen English adaptation of fictional fixtures and Tauri mocks for the specified revision; they read UI code and CSS from the checkout. If fixtures or mock contracts change, update the capture scripts before treating the captures as a new reference. No real keys, recordings or transcripts are used.
