# Vara native click-through

Open [Vara-interactive.figcraft](Vara-interactive.figcraft) in FigCraft. Start the **History · start here** flow in Present. This is an editable native document, with native interactions, compact choice overlays, variables, shared components, text and vectors. No website is part of the delivery.

The original 41 source states are retained. Five additional source-rendered examples show AI defaults enabled, another history selection, Notes app preferences, import actions below the fold, and an AI result. The existing spacing proposal applies to every browser-sourced state; the three original native HUD states keep their baseline geometry.

## Presenter walkthrough

1. History: select the second dictation, return to the first, disclose/hide the original, copy, open Delete and choose **Keep**. The sidebar opens all four main areas.
2. Settings: choose **Models and keys**, then **Privacy**, then **Dictation**. Each opens its main tab, rather than a scrolled continuation.
3. Models and keys: click the bottom half of the narrow right scrollbar gutter, or press **PageDown**, to inspect Local models and API keys. Choose **Add key**, click the empty key field to show a fictional example, then **X** or **Escape** to return to the exact entry screen. Saving shows native preview feedback. Continue down for custom endpoints, edit dialogs and CLI checks. **PageUp** or the upper gutter returns to the previous section. The sidebar Settings action returns to the top.
4. Transcription: **Choose files** opens the staged import. Use the lower gutter or **PageDown** to reveal **Transcribe 2 files**, then select the ready interview. In the editor, click **AI processing**, **Process text**, and **Copy AI text**. Switch back to **Transcript**.
5. Choose **Find speakers**, **Download Who**, a count choice, then **Find speakers**. Inspect the uncertainty example. **Export** opens its format choices; **X**, **Escape** and native back restore context.
6. AI per app: choose Mail or Notes, toggle Mail on/off, open **Add app** and inspect its native choice list. Dictation runs through the unchanged three HUD stages and returns to History.

All choice panels close on outside click or Escape. Copy/save/check/download feedback closes automatically or when clicked. **F1**, **F2**, **F3** from History expose setup, recovery and update-error examples respectively; these are presenter shortcuts, without new product buttons.

## Scope and limitations

This is a click-through with fictional fixture data. Choice lists update native prototype variables; selector labels imported as component instances are bound to those variables. Empty key/endpoint/search fields select fixed examples on click. They are not free-form native input controls. Existing disabled controls remain disabled, including saves in unchanged forms and the shortcut confirmation before a shortcut has been captured.

History and document selections demonstrate representative fixture states; they are not a full CRUD simulation for every fixture row. Delete and export confirmation lead to preview feedback. No clipboard content, files, settings, providers or app data are actually changed. Recording, media playback, installation, network calls, permission prompts and model execution are visual examples. Model/key links that would leave the file have no external action. Text editing and playback controls give bounded visual feedback rather than real editing/audio. This follows the available native text/variable/state capability; no browser controls or hidden services have been substituted.

Long settings/import sections use native frame navigation between source-rendered scroll positions. The rightmost 12 px gutter and PageDown/PageUp expose those continuations. This is discrete section navigation, not continuous scrolling. Full-window dialog snapshots navigate normally and use native back for X/cancel/Escape; compact dropdown/feedback panels use native overlays, so dialog backdrops are not doubled.

## Evidence and regeneration

[manifest.json](manifest.json) records source hashes, frame IDs, named flow starts, font status and representative rendering checks. [routes.json](routes.json) maps controls, actual DOM rectangles, native destination/action and hotspot node IDs. [controls.json](controls.json) preserves the rendered DOM control metadata, including disabled state and native options.

Headless integrity: all 41 original frame names retained, all 46 source frames reachable from History, 885 native interactions, 748 hotspot nodes including overlay controls, no external actions, zero missing fonts. Three fresh native exports in [exports](exports/) are pixel-identical to the same static-polish frames: History, Dictation settings and the transcription editor. Baseline native, static polish native, production sources and proposal CSS are unchanged. Structural checks and image exports do **not** establish native presenter click behaviour; MAIN performs that separately in an isolated FigCraft session.

Capture uses the unchanged baseline fixture/serializer, WebKit 26.0 at 1280×820 / 2× and the existing proposal.css. The scripts do not write product sources. Run an owned source server with `python3 gdd/design/figcraft/serve.py`, then:

```sh
node gdd/design/figcraft/polish/interactive/capture.cjs
python3 gdd/design/figcraft/polish/interactive/build.py
python3 gdd/design/figcraft/polish/interactive/prototype.py
python3 gdd/design/figcraft/polish/interactive/verify.py
```

Stop the owned server afterwards. Capture/browser jobs terminate themselves. Build starts a fresh native assembly each time; prototype attaches interactions to that fresh assembly. CLI roots are restricted to this output folder and its temporary assembly directory. Captured SVG/component fragments and command/result files make the native file reproducible and inspectable. Intermediate full capture PNGs are retained outside Git in the task scratchpad; the commit contains only three representative exports.
