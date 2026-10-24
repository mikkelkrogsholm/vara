# Bounded next work

## Current authorized deliverable

An ideal-product graph, source capture, feature view, explicit questions and GDD
model/view checks. Application implementation is outside this request.
Write scope: `gdd/` plus a pointer in the root scratchpad. Existing Swift code and
local changes are preserved. No new worktree, commit, push or external publication.

## Proposed next slices (not started)

1. Set audience, first-release scope, processing default, supported environments
   and measurable quality budgets (`question.audience`, `release_scope`,
   `default_processing`, `platforms`, `quality_budgets`).
2. Refine `action.capture`, `action.transcribe`, `action.insert`, `action.cancel`
   and `action.retry` together: exact inputs, effects, trust boundaries, failure
   semantics and risk/scenario coverage. Include applicable privacy rules.
3. Probe the uncertain OS integrations on declared target environments with
   bounded prototypes before selecting implementation architecture/framework.
4. Agree interaction/design sources for first use, status and recovery; fully
   specify only the first useful vertical slice before building it.

Deliverables for the next product pass: updated model, concrete chosen slice,
fixed expected scenarios, resolved decisions and visible remaining uncertainty.
Planned implementation checks: real foreground-app insertion/focus change,
late-result cancellation, microphone loss, raw fallback, strict-local network
isolation, recovery expiry, Danish/English quality and cold/warm latency.
These checks are not executed by authoring or validating the graph.

## Advisory design assessment of this model

- **KISS — reviewed:** one provisional product component, eight feature communities;
  no forced service/module architecture. Relevant IDs: `component.vara`, `community.*`.
- **DRY — reviewed:** `gdd.json` is the canonical intent graph. `PRODUCT.md` and
  `graph.html` are derived views, not independent editable requirements. Shared
  obligations live in `rule.*` and are linked to features/scenarios.
- **Separation of concerns — reviewed:** provenance, proposed intent, scoped open
  decisions and executed model-tool reports are separated. File processing and
  generative answers have explicit workflows (`action.import_media`, `action.answer`).
- **Modularity — provisional:** feature/OS/engine boundaries are conceptual only.
  Actual code modularity is UNKNOWN because no new implementation exists.

Assessment concerns the authored model at `gdd/gdd.json`, not the existing Swift
implementation. Exact delivered snapshot/revision and changed IDs are recorded in
`reports/summary.json`; raw reports are sibling files. No implementation/code
revision or application acceptance is claimed.
