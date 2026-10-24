# Vara v1 — clean-write

The first release is specified from the accepted user outcomes. The previous
Swift app remains an experience/source reference; this is not automatic feature
parity. The Rust desktop implementation now lives in `crates/` and `desktop/`; platform and
provider acceptance gaps remain explicit. See [Rust bindings](RUST-BINDINGS.md),
[build instructions](../desktop/README.md) and [UI review](reports/DESKTOP-UI-REVIEW.md).

Start here:

- [Product specification](PRODUCT-SPEC.md): purpose, first-release requirements,
  everyday journeys, exclusions and acceptance.
- [Module contracts](CONTRACTS.md): six responsibilities, data types, public
  interfaces, errors and effects, derived from the canonical graph.
- [Clickable first design](design/index.html): Danish UI for history, app AI,
  settings, onboarding and simulated session/error flows. It now implements Warm
  precision using shared components and generated tokens; open it directly offline.
  See [prototype maintenance](design/README.md).
- [Selected design system](DESIGN.md): Warm precision, exact tokens, component
  contracts, states, responsive rules and accessibility guidance.
- [Current interactive GDD graph](graph.html), backed by [gdd.json](gdd.json).
- [Implementation plan](PLAN.md): build order inside the accepted release scope.
- [Feature index](PRODUCT.md) and [decision history](DECISIONS.md).

## What is settled

Local/direct defaults; optional cloud and prompt-based app AI; configurable hold
or toggle; original-target-safe insertion and clipboard preservation; app-grouped
text history; optional timed text cleanup; failed-audio recovery capped at 24h;
personal direct-API keys and optional installed CLI text integrations; fresh
profile; macOS/Windows/Linux targets with Mac-first development.

Remaining work is engineering validation of runtime/toolkit, actual platform
coverage and measured model quality/latency. The Warm precision direction is selected; its detailed component rules are the
implementation specification. No additional
broad product questionnaire is needed to begin implementation.

## Reproduce the views and checks

From the Vara repository, using the local GDD installation:

```sh
python3 gdd/render_contracts.py
python3 gdd/render_product.py
~/Projekter/gdd/.venv/bin/gdd validate gdd/gdd.json --profile model --output gdd/reports/model-prototype-warm.json
~/Projekter/gdd/.venv/bin/gdd validate gdd/gdd.json --profile readiness --scope action.use_vara --output gdd/reports/readiness-prototype-warm-core.json
~/Projekter/gdd/.venv/bin/gdd validate gdd/gdd.json --profile readiness --scope screen.history --output gdd/reports/readiness-prototype-warm-design.json
~/Projekter/gdd/.venv/bin/gdd view gdd/gdd.json --output gdd/graph.html
~/Projekter/gdd/.venv/bin/gdd validate gdd/gdd.json --profile visualization --view gdd/graph.html --output gdd/reports/view-prototype-warm.json
~/Projekter/gdd/.venv/bin/gdd snapshot gdd/gdd.json --output gdd/reports/snapshot-prototype-warm.json
```

The CLI deliberately refuses to overwrite output files: use fresh output paths
or remove only the generated report/view before rechecking.

The core readiness check is structural. Screen readiness may remain unresolved
without an applicable trusted design-review decision; this is explicitly not
human design approval or an application-test failure. See
[verification notes](reports/VERIFICATION-prototype-warm-precision.md) for actual outcomes and limitations.
Normative PRODUCT-SPEC.md/DESIGN.md are hash-pinned in the graph; editing either
requires updating the source hash, snapshot and any current revision-bound record.
The generated human views are not additional independent specification sources.

## Earlier material

The broad ideal exploration before v1 contracts is archived byte-for-byte in
[reports/history/ideal-before-v1](reports/history/ideal-before-v1). It contains
deferred ideas and superseded wording; it is not the current release checklist.
Chronological decisions and research sources remain available. Historical records
are retained as history, never loaded as fresh approvals for the current model.
