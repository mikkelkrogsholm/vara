# Source and intent ledger

- `sources/user-direction.md`: selected direct user messages copied verbatim from the chat. Curated capture, not a full transcript. Registered as `source.brief` with its actual SHA-256.
- `sources/scratchpad-2026-10-23.md`: exact byte copy of the scratchpad before this graph was created. Registered as `source.scratchpad` with its actual SHA-256. Contains assistant proposals, not universally accepted requirements.
- The GDD tool was read and executed from `/Users/mikkelfreltoftkrogsholm/Projekter/gdd` (package 0.8.0, GDD 0.3). Its repository was not modified.

Both sources are provenance, not promoted normative implementation contracts.
All `vara.intent` annotations are non-normative metadata. `basis: user_direction`
records direct direction; `basis: proposed` records synthesis. Priority is a proposal.
All nodes remain `maturity: sketch`, with no executed implementation evidence.
Questions target the affected scope and block its implementation/acceptance, not
continued modeling. No artificial resolution or approval record was created.

The old code/Graphify graph was deliberately not imported: this deliverable models
desired behavior, not an observed architecture. Future product changes should keep
stable IDs, update affected scenarios/questions, and regenerate views/reports.
