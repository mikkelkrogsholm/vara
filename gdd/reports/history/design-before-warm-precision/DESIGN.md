# Vara v1 — first interaction design

Status: proposed visual/interaction contract for a desktop product. The HTML
prototype demonstrates flows with in-memory fixture data; it does not record
audio, install models, access secrets, register shortcuts, paste into other apps
or persist real history. Product decisions in PRODUCT-SPEC.md are the basis.

## Direction

Quiet, warm and precise. Off-white workspace, near-black navigation, restrained
burnt-orange action color and readable system typography. Spacious content with
dense enough history for daily use. No statistics dashboard or decorative hero
competes with results. UI examples are Danish; specification prose remains English.

Use a system-tray/menu-bar entry for everyday activation, a small nonactivating
HUD during a session, and one main window for history and configuration. The
web prototype represents these surfaces inside one canvas; it does not prove
native nonactivation or overlay behavior.

## Main window

Target starting desktop size: 1120 × 760, with a 220 px navigation area and a
readable content region. Show current local readiness and actual shortcut in
navigation. Primary destinations: History, App preferences, Settings.

History is the default destination after onboarding. Left content column lists
originating apps and counts; the main area shows dated result rows. A selected
result opens in a side panel with completed text first, delivery/locality status,
and an original-text disclosure only when processing occurred. Copy is explicit.
Delete removes both versions. If no results exist, explain the shortcut and offer
a trial dictation rather than displaying fabricated history.

## App preferences

List installed/known apps. The selected app has a labeled automatic-AI switch,
editable system prompt, a text-engine selector and a save action. Default is off.
The engine's processing location is visible next to its selection. Save/cancel
and changed-state feedback are explicit. Invalid input leaves the last saved
preference intact. A general prompt is configured separately in Settings.

## Settings

Three task sections: Dictation, Models & connections, History & privacy.
Dictation contains activation style and editable shortcut. Connections defaults
to the recommended local speech engine; downloading/not-ready/ready/error states
must be distinct. Cloud providers, custom endpoints and installed CLIs are
discoverable without dominating first use. A cloud-key form identifies its
provider and presents a masked key field plus meaningful validation state.
Custom endpoints expose name, URL, model, optional credential and locality.

History defaults to keep-until-deleted; timed cleanup is opt-in with an explicit
period. Recovery audio has a separate fixed 24-hour maximum. No audio playback.
Explain audio and text processing destinations separately.

## First use

A compact welcome and local setup with model download disclosure, required
permissions, activation style/binding and a real trial. No provider comparison
grid, account creation or key requirement on the normal path. Choosing cloud is
a deliberate alternative leading to the user-key form. In the prototype all
downloads and permission checks are explicitly simulations.

## Session HUD

Listening: status, input level, elapsed time, originating app and applicable
processing choice. Processing: speech/text stage, cancel, locality. Finished:
brief inserted/manual-ready feedback. Error: failed stage and available retry.
Do not steal focus. Escape follows the same cancellation contract as Cancel.
Do not display a processed preview as a mandatory approval stage. A menu-bar
quick choice can select direct or general-prompt AI for the next session without
changing saved app defaults; the prototype exposes it in its demonstration controls.

The prototype has a clearly labeled demonstration control to simulate recording,
focus change, recognition failure and AI failure. Demonstration controls belong
to the prototype only and must not become production navigation.

## States and access

Every essential surface defines empty, ready, busy, success and failure states.
Use native semantic buttons, labels, form controls and disclosures. Provide a
skip link, visible focus, keyboard navigation and polite status announcements.
Use text with color/status indicators. Avoid animation required for understanding.
At narrow widths, collapse navigation to a top bar and stack history/detail;
keep controls reachable at 200% zoom. No page-wide fixed-height clipping.

## Prototype walkthrough acceptance

1. Open History and filter by originating app; open a processed result and
   inspect the original. Direct results show one representation.
2. Enable AI for an app, edit/save a prompt, return and inspect the saved setting.
3. Switch between hold and toggle and change the shortcut in Settings.
4. Inspect local-default setup, optional API-key/custom endpoint/CLI paths and
   opt-in history cleanup. No real key or OS interaction is performed.
5. Simulate direct dictation, enabled-app processing, changed focus, failed
   recognition/retry, AI fallback and cancellation. State and history must follow
   the accepted contracts; no extra AI confirmation.
6. Simulate first use and empty history. Finish with truthful ready feedback.

These checks validate prototype behavior only. Native clipboard formats, focus
identity, microphone timing, storage expiry, provider authentication and OS support
require implementation tests.
