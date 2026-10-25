# Vara working notes

The active app is the Rust/Tauri clean-write. Product decisions and contracts
live in gdd/PRODUCT-SPEC.md, gdd/DECISIONS.md and gdd/gdd.json. Follow
DESIGN.md under gdd for the approved Warm precision design and shared components.

Current implementation and limits: IMPLEMENTATION.md. Development: README.md
and desktop/README.md. Release process: RELEASING.md.

The Swift/Xcode implementation has been retired. Earlier exploration is retained
in Git history and gdd/sources/scratchpad-2026-10-23.md. Historical source captures
and reports describe their original revision; they are not current build inputs.

Do not resume the cancelled dictionary/regex investigation without a new request.
Keep the next work bounded to the user's chosen feature or concrete defect.

## Transskriberingsfane

Featureliste, første leverance, ordniveau-tidsstempler, redigering/alignment,
kildehåndtering og åbne tekniske afklaringer er samlet i
[gdd/TRANSCRIPTION-SPEC.md](gdd/TRANSCRIPTION-SPEC.md). Implementeret på
`codex/file-transcription`; native slutkontrol afventer en ulåst Mac. Evidens og
begrænsninger: [rapport](gdd/reports/transcription/README.md).
