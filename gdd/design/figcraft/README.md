# Vara in FigCraft — current 1.0.2 app

Open `Vara.figcraft` in FigCraft. This refresh uses the current app/UI source at `df3f936617b037c313745ea4c7ac08c49d474e67`, rather than the old `26f61b5` reconstruction. The original native file is preserved byte for byte in `history/Vara-26f61b5.figcraft`, with its SHA-256 in `history/source.json`.

The native document has **41 screen/state frames**, preserving all 31 previous frames, plus current updater available/ready/error states, recovery, scrolled Privacy, recording controls, speech model, local models, populated transcription Recent and a disabled per-app rule. Interface language and Updates are present above all three settings tabs. Running version is 1.0.2; 1.0.3 in updater examples is fictional, not a claim about a published release. Frame 10 means the import landing with no files selected; its fictional Recent documents are shown explicitly in frame 40. All captured settings, names, history, shortcuts, documents and failure states are fixtures. The capture pipeline reads no live profile, keychain, recording or transcript; MAIN separately observed the live UI as described below.

Text remains editable; icons, controls, focus outlines and underlines are vectors. Dialog shadows import as editable native drop-shadow effects. Sidebars, fields, primary buttons and save bars are genuine component instances. The single bitmap asset is the public Mail icon. Screens remain fixed snapshots, with line-based text (word-based around inline links), rather than responsive layouts or functional prototypes. The final manifest records actual layer/component/instance counts, frame IDs and hashes.

## Review evidence

Open [review.html](review.html) for every reference, fresh native-file export, side-by-side pair, 50% overlay and amplified difference. [manifest.json](manifest.json) binds every pair to its reference/export SHA and the actual source hashes. [visual-review.md](visual-review.md) records bounded visual checks and residual differences. Contact sheets in `comparisons/sheet-*.png` cover every frame, reference on the left, export on the right. All evidence is regenerated together after the final native build; no old SVGs or selected old exports are mixed into the refresh.

Browser references use **WebKit 26.0**, 1280 × 820 logical pixels, device scale 2, English, Europe/Copenhagen and a frozen 2026-11-06 clock. They exercise the actual current HTML/CSS/controllers with a deterministic in-memory Tauri host and the current runtime i18n catalog. Model options match the checked-in fixture contracts. The browser engine is closer to macOS WebView than Chromium, but these captures are not screenshots of the installed Tauri app.

The three HUD references render the **actual AppKit HUD source** in a short-lived, isolated test process, using fictional Mail origin, current runtime labels and a fixed audio level of 0.4. This process accesses no microphone, global shortcut or Vara profile. It exits after capture. The editable HUD frames follow the source geometry and measured native reference. The native reference captures panel contents; an operating-system window shadow is outside that capture. Mail icon rasterization and subpixel text antialiasing still differ slightly.

MAIN separately observed installed Vara 1.0.2 through CUA at 1280 × 820 / 2×, English: History with recovery, all Settings tabs including scrolled Privacy History/Audio, Transcription landing and Recent, and AI per app with custom settings off. Those observations corroborate current WebView layout. They do not make the fictional fixture screenshots native-app captures, and do not verify every dialog/state on the installed app. No private live content is included in this folder.

## Fonts and renderer limits

[font-mapping.json](font-mapping.json) records the installed system font axes. FigCraft's system font defaults to optical size 28. WebKit/AppKit choose text-size optical sizing, so the build sets the actual optical-size axis (minimum supported size 17) and preserves explicit weight axes. This avoids the previous title scale hack and the previous rule that promoted every 500 weight to bold. Computed letter spacing is retained. The current SVG importer names 600 as `Semi Bold` and cannot represent arbitrary 650 weight directly; capture IDs carry the actual CSS weights into the native variable-font mapping. No font files are redistributed. The reopened final file reports zero missing fonts. The existing FigCraft GUI session is restricted to another project’s automation read root and refused to open Vara; that root, its other open document and launch permissions were left unchanged. Native CLI reopen/render succeeded for every frame; GUI-open verification remains unavailable in that session.

The viewport receives the actual HTML canvas background, and editable clip-path groups preserve ancestor overflow/scroll clipping, including the one-pixel document offset caused by dialog autofocus. DOM Range positions determine wrapping, line positions and inline link word positions. Canvas font ascent/descent determines baselines. Active input selection paint is taken from the captured reference pixels: WebKit can retain selection endpoints while omitting the visible highlight. The serializer draws a Highlight rectangle only when the captured input actually shows it, and the capture checks window-focus stability. Native checkbox, radio, playback range thumb and speed stepper shapes are editable approximations of the observed WebKit controls. Native control gradients/shadows, exact Skia/CoreText glyph rasterization and fine optical tracking can differ. This is inspectable current-app parity work; it is not a universal pixel-equality claim. Review the full-size pairs before treating any residual renderer difference as accepted. Product code and canonical tokens remain unchanged; `gdd/design/styles` and actual app/UI source remain the authority, not FigCraft variables.

## Reproduce

Requires Python 3 + Pillow, Node + Playwright, installed WebKit and FigCraft CLI 0.15.1. Set paths appropriate to the machine. `build.py` and `export-review.py` honor `FIGCRAFT_CLI`. The capture runner honors `PLAYWRIGHT_MODULE` and optional `PLAYWRIGHT_WEBKIT` (the installed Playwright package may expect a different cached WebKit revision).

Start the existing source-only server in one terminal:

```sh
python3 gdd/design/figcraft/serve.py
```

In another terminal:

```sh
PLAYWRIGHT_MODULE=/absolute/path/to/playwright \
PLAYWRIGHT_WEBKIT=/absolute/path/to/webkit/pw_run.sh \
node gdd/design/figcraft/capture-runner.cjs
python3 gdd/design/figcraft/capture-hud.py
python3 gdd/design/figcraft/build.py
python3 gdd/design/figcraft/export-review.py
```

Stop the server after capture. Native HUD capture is a standalone AppKit development test and briefly displays its own nonactivating panel; it does not drive installed Vara. No Cargo build, model download or cloud call is needed. Build stages use narrow automation roots and a temporary native output; the current native file is replaced only after successful assembly. Do not rebuild over a file with unsaved user edits open in FigCraft. `export-review.py` verifies all 31 old frames, paired dimensions, archive hash, successful fresh exports and zero missing fonts before generating comparisons.

Actual command results are in `capture-results.txt`, `native-results.txt`, `build-*.jsonl`, `export-results.jsonl` and `info.json`. `export-commands.json` is generated with absolute paths for this checkout; regenerate it by running the script after moving the folder.
