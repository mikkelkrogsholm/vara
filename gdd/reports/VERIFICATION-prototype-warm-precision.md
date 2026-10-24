# Warm precision prototype — verification

Executed 2026-10-23. Specification revision: `sha256:bec5246fcd5b3634cfbed0f21a53943cc5c59b3273b229b594528ccc2c39f6e7`.
Source hashes: prototype-warm-precision-assets.json.
Raw browser outcomes: prototype-warm-precision-checks.json.

## Outcome

- 22 browser check groups PASS, no JavaScript errors observed.
- Direct file-URL loading and interaction PASS; no server/build is required to use the prototype.
- Generated token drift check PASS; JavaScript syntax checks PASS.
- Official DESIGN.md CLI 0.4.0: no errors, four documented prose-only color warnings.
- GDD schema/semantics/record integrity, core structural readiness and graph projection PASS.
- GDD screen readiness remains UNKNOWN without a current trusted readiness_review; no human review or native acceptance is fabricated.

## Visual reference comparison

Inspected against design/mockups/01-warm-precision.png at the same 1586 × 992
viewport. Browser columns begin at x=39 / 310 / 808 with widths 271 / 498 / 739.
The image's column dividers are approximately x=309 and x=807, so the browser
geometry matches within two pixels. Palette, serif reading surface, app grouping,
quiet rules, copy action, inset original and floating waveform HUD were visually
reviewed. See prototype-warm-reference-state.png for the matching expanded/listening
state, and prototype-warm-desktop.png for the normal collapsed/ready state.

This is not a claim of identical raster pixels. The current native app artwork,
font rasterization, actual fixture text, explicit delivery status and correct
“dikteringer” terminology can differ. The initial original is collapsed per
DESIGN.md and the idle HUD says “Klar · Eksempel”. Warm paper is rendered flat
inside the app; screenshot wallpaper/lighting is not painted into content.

Additional inspected screens: prototype-warm-app-ai.png,
prototype-warm-connections.png and prototype-warm-narrow.png. Forms reuse the same
component language; the inactive HUD is hidden outside History, while active
recording/processing remains visible. Responsive content uses drilldown rather
than shrinking a three-column interface.

## Executed browser checks

- Mockup column geometry and default disclosure
- Grouped history, original/direct representations and selection focus
- Search, no-match and app filter
- App AI save, unsaved edits and invalid draft
- Activation mode and shortcut capture/conflict
- Opt-in retention and period validation
- Key validation and secret clearing
- Endpoint configuration and strict-local guard
- Direct, app-override and general-prompt sessions
- Focus-safe manual outcome and AI fallback
- Recognition recovery, retry cancellation and fixed deadline
- Cancellation invalidates late provider response
- Async completion preserves settings editing
- Deliberate delete confirmation
- Local/cloud onboarding, install cancellation and trial gates
- Long result retains full text
- History/detail/settings reflow at 1280, 900, 600 and 320px
- Reduced-motion state feedback
- Direct file URL loading and keyboard navigation
- Modal keyboard focus containment and Escape
- 320px listening/processing HUD labels and controls fit
- Narrow master/detail navigation preserves list scroll position

## Maintenance and remaining evidence

DESIGN.md owns tokens; the generator produces tokens.css and --check catches drift.
Pure shared components, composed views, fixture state and session controller are
separate files. The executable smoke function is checked in beside the prototype.
List selection/focus and scroll position survive master/detail navigation; async
completion does not erase a form draft. Cancellation rejects late publication,
and cancelled model installation cannot become ready later.

The browser checks use simulated results/timers and fake keys. They do not test
native microphone, global binding, clipboard formats, target focus, secret-store,
audio-file deletion, real account authentication, installers or Windows/Linux
runtime support. Native nonactivation/OS permission behavior and actual screen
reader/forced-colors/font rendering on every target remain implementation checks.
Reflow was exercised at 1280, 900, 600 and 320 CSS px; this is not a claim that a
separate native screen-reader or browser 200% zoom test was executed.

Earlier prototype HTML and model metadata are preserved in
history/prototype-before-warm-precision. Earlier evidence remains historical and
is not silently rebound to this source bundle.
