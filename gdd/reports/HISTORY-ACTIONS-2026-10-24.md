# Local app artwork and direct history actions — 2026-10-24

The user requested actual installed-app logos and separate copy/delete icons in
place of the two-option overflow modal.

## Delivered

- Optional local `application_icons` adapter resolves OS app IDs via NSWorkspace,
  preferring a running app's bundle URL. Returns transparent 96px PNG artwork.
  No UI-supplied file paths, external logo service, or domain/history schema change.
- Shared `appArtwork` component uses the same identity/artwork in group headers,
  selected results and app preferences. Async loading does not replace forms,
  reset selection or block startup. Missing artwork retains a monogram. Windows
  and Linux currently use that fallback pending their native icon adapters.
- Shared `resultActions` renders accessible 36px copy/trash buttons in history
  rows and the selected-result header. Copy addresses the clicked result without
  opening a modal or selecting that row. Delete opens the existing confirmation
  directly; the intermediate two-option modal and handler have been removed.
- DESIGN.md and Rust bindings updated to reflect the requested interaction.

## Verification

- Native icon test PASS: actual TextEdit and Vara icons decode as visible 96px
  PNGs; an unknown app ID returns no artwork. Test reads app metadata only.
  Source: `desktop/tests/application-icons-native.m`.
- All 17 production UI test groups PASS, including icon identity/fallback,
  direct row copy, unchanged selection, direct deletion confirmation, existing
  credential scenarios, keyboard behavior and widths down to 320px.
- Signed Mac application build PASS, including strict/deep signing verification
  and the audio-input entitlement check.
- Updated application reopened. Native UI visually checked: actual ChatGPT and
  TextEdit icons appear; separate copy/trash buttons appear beside each history
  row and selected result. No user dictation was deleted or copied during this
  native visual review.
- GDD model/record integrity and generated graph validate. Broader release
  readiness remains UNKNOWN; no user release acceptance is inferred.

Build intermediates are removed after verification; the latest app is retained.
