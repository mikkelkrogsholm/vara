# Vara spacing polish — DRAFT for design review

This proposal preserves Vara's olive navigation, warm paper, burnt-orange actions, Georgia reading/settings type and system controls. It makes small grouping corrections; it does not change navigation, workflow, feature availability, privacy/legal copy or production files. The HUD is deliberately unchanged: its existing source-controlled compact native composition is already appropriate. No design approval or app integration is implied.

Open [review.html](review.html) for all twelve before/after pairs; [overview.png](overview.png) provides a quick overview. The before images link to the unchanged baseline fictional WebKit fixtures. After images are fresh exports from [Vara-polish.figcraft](Vara-polish.figcraft), with editable text, vectors, genuine shared components and instances. Screens are fixed snapshots, not responsive native Figcraft layouts or working controls. The separate shared component page is part of the native document. Components have role/state names; text layers carry readable content names after font-axis mapping. No screenshot is used as a flattened screen. The final document contains 12 screen frames, 29 shared components, 35 instances and 694 editable text layers.

## Confirmed issues and restrained changes

- Updates: the automatic checkbox label touched the check button. A wrapping row now gives actual 16 px separation and 8 px checkbox/text separation. Version, notice and actions use one 16 px gap source; the notice's old margins are reset to avoid doubled whitespace.
- Settings: the first Recording section's padding was reset at the form boundary. It now begins 24 px below the global Updates divider. Existing global Language/Updates blocks and all three current tabs stay visible. Field labels/controls/help use 8 px; row/column gaps use 24 px. Shortcut labels use the same actual gap rather than adding their old margin to the field gap.
- Model/provider rows: preserve stable identity/status/action columns and native wrap rules; 16 px insets/gaps give consistent row rhythm without new cards. Peer actions use 12 px. Long content can wrap; no truncation is newly introduced.
- AI per app: reduce title-to-toggle spacing, align the switch to its label row, keep the explanatory paragraph, and give the toggle/prompt and subsequent form groups 24 px separation. Prevent margin collapse through the fields wrapper. Help text remains visible; explanatory prose is bounded to 64–68ch.
- History: preserve 76 px row density and direct copy/delete. Metadata separates from title by 4 px; row insets remain 12 px; direct icon actions remain 36 px with 8 px between peers. Sidebar width and reading text sizes stay unchanged.
- Transcription: preserve library/editor/content hierarchy, three familiar header actions, speaker/edit/play actions, timestamps, original access and player. Apply 12–16 px toolbar grouping, 8 px fields and 24 px import options. Existing responsive typography rules are inherited, not redesigned.
- Dialogs: preserve 32 px padding, 16 px title/body spacing and all copy. Reserve title space for the existing close action, keep its existing 36 px target (above our 32 px minimum) and existing focus outline, and use 24 px field/action grouping. Who's long notice remains in the existing scrollable dialog.

## Proposed contract and future mapping

`proposal.css` is review input for later approved integration. It is not a new production token authority. Existing `gdd/design/styles/tokens.css`, `app.css` and `desktop/ui/desktop.css` remain canonical. Prefix `--draft-` deliberately identifies provisional roles.

| Semantic role | Proposed logical CSS pixels | Existing integration surface |
| --- | ---: | --- |
| Label/control/help |8| `.field`, `.field small`, `.shortcut-field .field-label` |
| Peer actions |12| `.provider-connection .actions`, `#app-form > .actions`, `.tx-detail-header .actions`, dialog actions |
| Related controls |16| update body, model rows, `.tx-tabs`, `.tx-player` |
| Field groups/grid |24| `.settings-field-grid`, `#app-ai-fields`, `.tx-import-options` |
| Section breathing |24| `.settings-section`, first form section, app toggle/prompt |
| Section columns |32 existing (24 at inherited breakpoint)| `.settings-section` |
| Save bar |16 inset/separation;32 last-section bottom| `.settings-form .sticky-save`; retain existing 120 px scroll clearance |
| History row |76 minimum,12 inset,4 title/meta| `.row-select`, `.row-meta`, `.result-actions` |
| Dialog |32 inset,16 prose,24 form/action| `.dialog-content`, `dialog .field`, `dialog .actions` |

These values are Vara choices on its existing 4-based scale. They are not Apple or WCAG mandates. Existing control heights, colors, radii, type scale and breakpoint structures are retained. Implementation must map these rules into the existing shared canonical components after design approval, rather than blindly appending this override sheet, then be verified in the actual target app. This draft does not authorize that work.

## All 41 baseline states: disposition

Only the 12 **rendered draft** states below were rebuilt as new native frames. The remainder are mapped to shared rules or explicitly retained; they are not represented as newly rendered designs.

| Shared rule / keep decision | Rendered draft | Other baseline states, reviewed by rule only |
| --- | --- | --- |
| History row/actions |01-history|02-original;35-recovery (message/recovery content retained) |
| AI app fields/toggle |03-apps|41-app-rule-disabled (hidden-fields state retained) |
| Settings global blocks/fields/sections/save bar |04-dictation,37-recording-controls,36-privacy-audio|05-connections;06-privacy;26-dictation-permissions;27-custom-endpoints;28-cli-connections;32-update-available;33-update-ready;34-update-error;38-speech-model |
| Model/provider columns |39-local-models,29-api-providers| Shared with 05-connections/27-custom-endpoints/28-cli-connections above |
| Transcription field/toolbar rhythm |11-transcription-import,12-transcription-editor,24-transcription-ai|10-transcription-empty;14-who-known-count;15-who-uncertainty;40-transcription-recent |
| Dialog field/action/close spacing |22-key-dialog,13-who-download|07-processing-dialog;08-delete-dialog;21-shortcut-dialog;23-app-picker;25-transcription-export;30-endpoint-dialog;31-model-dialog |
| Keep existing onboarding composition; inherit field/dialog rules only |—|09-setup-prepare;16-setup-permissions;17-setup-trial |
| Keep native HUD unchanged |—|18-hud-recording;19-hud-transcribing;20-hud-processing |

## Evidence and limitations

The manifest binds source, baseline, draft, proposal CSS, frame IDs and image hashes. The baseline native SHA-256 remains `a0b8d5d7739dace528e0854152ed5575f88e89182895bed866f25f5997fd1ab8`. The build reuses the baseline DOM serializer, ancestor clipping, font optical-size/weight-axis corrections and component extraction. Native reopen/export reports zero missing fonts. Proposal browser renders and amplified native-render differences are retained in `render-evidence/`. Native controls, Skia/CoreText antialiasing and exact system font tracking remain bounded renderer approximations; no universal pixel-equality claim is made.

Fresh no-proposal control captures use the same fictional host, WebKit 26.0, 1280×820/2×, English, Europe/Copenhagen and frozen clock. History is pixel-identical to its baseline reference; its group header is also identical between old and draft native exports. A perceived count shift in browser-versus-native comparison is retained importer/renderer behavior, not a spacing proposal. The editor control capture differs from baseline only within the player region (physical bounds 1675,1504–2348,1608): WebKit's native range now paints a thinner shaded track/round thumb. The native draft retains the baseline serializer's editable control approximation. This system-control variation is documented, not claimed as improved design fidelity. `control-comparison.json`, control images and `player-control-investigation.png` retain that evidence. No forced theme or selection change is introduced by the proposal; current system media is recorded in capture metadata. Old metadata does not contain OS native-control appearance settings, so exact cause is unverified.

`checks.json` records 38 passing bounded browser DOM checks at 1280×820 and 1000×760 for all 12 states, including label/control geometry, update separation, close target/title clearance, history density and focused settings control clearance. Additional 1000×760 Danish long-label plus WCAG text-spacing override stress captures cover Settings, AI per app and Who. At 1280 px the History rows measure 76 px and both direct actions measure 36×36 px with 8 px separation. At 1000 px the inherited selected-detail responsive state hides the list; the visible result copy/delete actions still measure 36×36 px. Hidden zero-size list targets are not counted as validated targets. Update label/button separation measures 16 px, all measured field gaps measure 8 px and the app toggle/prompt separation measures 24 px. Focus checks use visible enabled controls: general-ai has over 342 px footer clearance normally and 341 px under text-spacing stress; Privacy retention has over 329 px. An initial check incorrectly selected a hidden textarea and was corrected before final evidence. These are content-stress fixtures, not a Danish translation review. A separate 150% check multiplies each element’s initial computed font size by 1.5 (field labels 14→21 px) at 1000×760 for the same three states, without applying the text-spacing override simultaneously. Field/title/control geometry and final-action reachability pass; the existing sidebar scroll makes its Dictate controls reachable. Settings focus retains over 332 px footer clearance at this scale. Top/end screenshots are retained. This bounded check is not 200% zoom or full WCAG conformance. The legal copy stays intact and its final action is reachable after scrolling; the stress top and end images record both views. Fixed native frames are not claimed to adapt; responsive and keyboard checks exercise the browser proposal, not the installed Tauri app. Screenshots and geometry do not establish full accessibility compliance or expert usability. No live user data, profile, keys, microphone, ASR, model download or cloud call is used.

## Research basis

[Apple layout](https://developer.apple.com/design/human-interface-guidelines/layout) supports alignment, grouping and whitespace; [typography](https://developer.apple.com/design/human-interface-guidelines/typography) supports a restrained type hierarchy. [Apple accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) uses platform-specific control guidance; this desktop proposal does not apply mobile 44 px everywhere. [Fluent2 layout](https://fluent2.microsoft.design/layout) and [Windows spacing](https://learn.microsoft.com/en-us/windows/apps/design/style/spacing) inform proximity and a 4-based ramp, with platform-specific examples rather than universal requirements.

[WCAG target size minimum](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum) defines a 24 CSS px floor with exceptions; our existing 32–40 px desktop icon targets are design choices. [Text spacing](https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html) addresses user override tolerance, not a requirement for default 1.5 line-height everywhere. [Focus not obscured](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html) supports retaining scroll clearance around sticky actions. MAIN's bounded opaque canonical-token contrast calculations support keeping the palette: on-surface/canvas 14.03, muted/canvas 5.47, muted/surface 5.87, on-primary/primary 5.99, on-sidebar/selected 7.66, sidebar-muted/sidebar 8.04, border/surface 4.02, focus/canvas 6.80. These are selected token pairs under [contrast minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html), not a full UI compliance result.

## Reproduce without production writes

Requires the existing installed CLI 0.15.1, Python/Pillow, Node/Playwright and cached WebKit. No browser downloads or Cargo build. From the repository root, start `python3 gdd/design/figcraft/serve.py` only if port 18770 is free. Stop that owned process after capture/checks.

```sh
export PLAYWRIGHT_MODULE=/Users/mikkelfreltoftkrogsholm/.bun/install/global/node_modules/playwright
export PLAYWRIGHT_WEBKIT=/Users/mikkelfreltoftkrogsholm/Library/Caches/ms-playwright/webkit-2311/pw_run.sh
node gdd/design/figcraft/polish/capture.cjs
DRAFT_CONTROL=1 node gdd/design/figcraft/polish/capture.cjs
python3 gdd/design/figcraft/polish/build.py
python3 gdd/design/figcraft/polish/export.py
node gdd/design/figcraft/polish/check.cjs
```

Build/export automation roots are limited to the draft folder and its temporary native assembly directory. The adapters read the unchanged baseline scripts without running their baseline write paths. Review artifacts live only in this folder; the sole outside edit is a draft pointer in `gdd/DESIGN.md`. No GUI session is changed. No version change, app build/install, push or release action is part of this proposal.

Final worker verification: all 38 bounded checks pass; all 12 native exports reopen with zero missing fonts. The owned source server was stopped after checks. All browser/build/export jobs terminated normally; no setup hook or GUI job was launched. No production file or baseline artifact changed.
