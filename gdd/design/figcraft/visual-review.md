# Bounded visual review

The worker inspected paired contact sheets for every frame and full-size pairs for Settings, History, the transcription editor, Who dialogs, key/model dialogs and all three HUDs. MAIN independently reviewed the current UI and targeted pairs. This table describes what was checked; it does not assert that every pixel is equal. Reference/export hashes and actual native frame IDs are in `manifest.json`.

Browser geometry, wrapping and control state come from the current rendered DOM at 1280×820 / 2×. Browser references are WebKit 26.0 fixture captures. Only HUD references are source-rendered AppKit. Native installed-app observations corroborate layout separately, as described in README. No private live screenshots are stored here.

The final pass fixes explicit focus outlines/offsets, link underline vectors, dialog shadows, source system-font weights including 650, optical sizing, checkbox/radio shapes, gray playback track and white capsule thumb, speed stepper arrows and selected input text. No screenshot is used as a whole-screen design layer.

| Frame | ID | Check and bounded finding |
|---|---:|---|
| 01-history | 812 | History groups, selected row, action icons, reading column and sidebar checked; fictional mail text wraps on the same lines. |
| 02-original | 943 | Expanded original disclosure, divider and original text block checked; body text remains editable. |
| 03-apps | 1076 | Enabled Mail rule: switch, prompt, model field and save area checked; textarea uses measured font metrics. |
| 04-dictation | 1122 | Language, current version 1.0.2, Updates notice, checkbox and Recording fold checked; optical-size fix removes label/button gap. |
| 05-connections | 1180 | Current language/update blocks precede Local models; scrolled continuation 39 supplies the full model list. |
| 06-privacy | 1250 | Language/update blocks precede Processing summary; scrolled continuation 36 supplies History/Audio. |
| 10-transcription-empty | 1316 | Import landing with no files selected; disabled Transcribe and policy text checked. Existing fictional Recent continues in 40. |
| 11-transcription-import | 1361 | Fictional selected-file list, remove icons and import controls checked; no real file access or transcription. |
| 12-transcription-editor | 1417 | Library heading/row weights, title, reading text, speaker controls and playback checked; gray native range capsule and speed stepper preserved as vectors. |
| 24-transcription-ai | 1539 | AI tab, engine select, prompt/textarea and action checked; no model or cloud call. |
| 26-dictation-permissions | 1661 | Scrolled Speech/After transcription/Permissions geometry checked; selected radio is primary outer circle with white center. |
| 27-custom-endpoints | 1713 | Scrolled advanced model rows and actions checked; no endpoint contacted. |
| 28-cli-connections | 1789 | Scrolled CLI disclosures, rows and check buttons checked; no real CLI login checks. |
| 29-api-providers | 1845 | Scrolled provider labels, descriptions and Add key actions checked; empty fictional credentials. |
| 36-privacy-audio | 1940 | Scrolled History retention and Audio deletion policy are visible and checked, with save area preserved. |
| 37-recording-controls | 1985 | Recording section deliberately scrolled to its heading so all activation, shortcut, microphone and language controls are visible. |
| 38-speech-model | 2053 | Speech model, readiness, After transcription and permission continuation checked; fixture source options retained. |
| 39-local-models | 2121 | Local models section deliberately scrolled to its heading so current Parakeet/Whisper/Qwen rows and source sizes are visible. |
| 40-transcription-recent | 2214 | Populated Recent/search plus ready/transcribing/failed fictional document rows checked; unchanged actual source landing. |
| 41-app-rule-disabled | 2285 | Saved disabled custom setting for fictional Mail checked; prompt/model fields collapse as in the current app. |
| 07-processing-dialog | 2321 | Dialog geometry, backdrop, focused control, source drop-shadow and primary action checked. |
| 08-delete-dialog | 2537 | Deletion dialog layer order and focused close outline checked; no fixture deletion performed. |
| 13-who-download | 2748 | Full-size review: focus ring, native shadow, bold heading/action, licence underlines and inline whitespace checked. Inline link words use DOM positions. |
| 14-who-known-count | 3027 | Full-size review: current Who count control, licence text, modal geometry and source action state checked. |
| 15-who-uncertainty | 3312 | Uncertainty notice and expanded turns checked; all model/diarization results are synthetic host responses. |
| 09-setup-prepare | 3436 | Setup model/prepare state and main action geometry checked against source fixture. |
| 16-setup-permissions | 3476 | Permission rows, granted indicators and setup actions checked. |
| 17-setup-trial | 3526 | Trial instructions, shortcut and actions checked; no microphone operation performed. |
| 21-shortcut-dialog | 3563 | Source focused shortcut capture field and outline checked; shortcut is representative fixture Alt+Space, not private configuration. |
| 22-key-dialog | 3718 | Full-size review: input focus ring, bold provider/action, backdrop and native editable shadow checked; input is empty, no key material. |
| 23-app-picker | 3895 | Source picker, focused control and fictional application choices checked; no private application list. |
| 25-transcription-export | 4019 | Export choices, dialog geometry and actions checked; no external file export performed by the fixture. |
| 30-endpoint-dialog | 4224 | Focused name outline, fields and dialog shadow checked; no connection created. |
| 31-model-dialog | 4387 | Focused/selected Name field, model and action checked. Selection endpoints and visible paint are distinguished: an editable OS-Highlight background is emitted only if the captured input actually paints it; the reference is retained. |
| 32-update-available | 4555 | Hypothetical 1.0.3 available notice/action and sidebar indicator checked; installed version remains1.0.2. |
| 33-update-ready | 4623 | Hypothetical 1.0.3 ready/install notice/action and sidebar indicator checked; no update downloaded or installed. |
| 34-update-error | 4691 | Current-version 1.0.2/error notice and controls checked; synthetic updater failure only. |
| 35-recovery | 4749 | Failed transcription runtime message, retry/delete task and expiry checked; no recording data exists. |
| 18-hud-recording | 4886 | Actual isolated AppKit reference: source size 520×76, fixed 0.4 meter, stop/cancel, native labels and public Mail icon checked. Fine text/icon raster residual remains. |
| 19-hud-transcribing | 4915 | Actual isolated AppKit reference: inactive meter, absent stop control and runtime recognizing label checked. Fine text/icon raster residual remains. |
| 20-hud-processing | 4954 | Actual isolated AppKit reference: inactive meter and current runtime AI locality label checked. Fine text/icon raster residual remains. |

Fine renderer differences remain in system-font antialiasing/tracking, native control gradients/shadow edges and Mail icon rasterization. Native HUD detail glyph widths differed by approximately 3 physical pixels across the entire label in the measured 2× reference; source geometry and baselines are retained. This residual is visible in overlays rather than hidden by scaling or flattening. Native window shadow is excluded from the AppKit panel-content capture. The static document does not verify focus behavior, keyboard interactions, playback, accessibility or responsive sizes. No generic pixel score is used as an acceptance claim.

Final bottom-edge clipping regression check: dialog autofocus offsets the document by one logical pixel. The HTML viewport background and actual ancestor overflow clips are now preserved as editable shapes/masks. For frames 22, 30 and 31, the last eight physical rows over x=540…2539 contain zero dark pixels in both reference and export. The final row differs by at most one RGB channel level due to backdrop blending. This corrects the prior transparent-root/offscreen-content artifact without masking a screenshot.
