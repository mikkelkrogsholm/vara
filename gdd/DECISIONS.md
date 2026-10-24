# Product decisions

## Audience — accepted 2026-10-23

**Mikkel first; approachable setup for others.**

The normal first-use path must not require technical engine setup. Advanced
configuration can remain available. This constrains onboarding and recommended
setup, but does not decide local versus managed processing, accounts or pricing.

Source: `sources/audience-decision-2026-10-23.md`.
Original record: `decision.audience.2026-10-23`, preserved in
`reports/history/records-audience-original.json`. The current combined direction
retains this audience unchanged in `decision.release_scope.2026-10-23`.

## First release and modularity — accepted 2026-10-23

The first release delivers approachable setup, daily faithful dictation with optional light
cleanup into the intended app, safe cancellation and recovery of failed work.
File transcription, AI answers, wake word and automatic app-specific customization
are deferred. Other proposals remain undecided rather than silently included.
Relevant privacy/accessibility/reliability obligations still apply to the core flow.

Build modularly with clear GDD contracts for implemented responsibilities, public
actions, effects, errors and scenarios. This is a design/contract requirement, not
a decision to introduce microservices, a plugin system or a fixed module count.

Source: `sources/release-scope-decision-2026-10-23.md`.
Record: `decision.release_scope.2026-10-23` in `records.json`.

## Platforms — accepted 2026-10-23

Target **macOS, Windows and Linux from the first release**. Develop and test on
macOS first; Windows/Linux runtime verification happens at the end. Design OS
integration contracts for all three from the outset. Rust is the working technical
direction; UI toolkit and dependencies remain undecided. Targeted support is
unverified until the relevant real environment is tested.

Versions, CPU architectures and Linux environment coverage remain a separate open
acceptance question (`question.platform_envelope`). This does not impose early
crossplatform test access.

Source: `sources/platform-decision-2026-10-23.md`.
Current combined direction: `decision.platforms.2026-10-23` in `records.json`.
Earlier records are immutable history in `reports/history/`; their accepted
constraints remain in force, with current resolution links carried forward.

## Processing — accepted 2026-10-23

Support local and cloud processing. **Local is the default**, with setup guided
through the app. Cloud is an explicit discoverable choice, never silent fallback.
Explain audio and text destinations separately. Account, provider and payment
choices remain open (`question.cloud_access`); no hosted service is commissioned.

Source: `sources/processing-decision-2026-10-23.md`.
Current combined record: `decision.processing.2026-10-23` in `records.json`.
Earlier records remain immutable history; current question links point to this
combined direction without changing the previous accepted requirements.

## Model-dependent responsiveness — accepted direction 2026-10-23

It must feel fast. Actual latency depends on the selected model; do not invent a
universal seconds limit before measurement. Investigate compact local models for
the Rust app. Whisper, Parakeet TDT and Parakeet Redux are candidates, not selected
engines. Rust integration may use suitable native libraries or bindings.

Model choice and exact quality/latency budgets remain open for measured comparison.
Published model throughput is not end-to-end Vara latency. Redux’s documented
speed uses Photon; ordinary Parakeet Rust bindings do not establish Redux runtime
compatibility. See `sources/local-model-research-2026-10-23.md`.

Source: `sources/model-speed-decision-2026-10-23.md`.
Current combined record: `decision.model_speed.2026-10-23` in `records.json`.
Prior accepted directions remain unchanged; earlier records are immutable history.

## Activation — accepted 2026-10-23

Support both hold-to-talk and press-to-start/press-to-stop. **Hold-to-talk is the
default.** The user chooses the shortcut. Provide actionable feedback on an
unsupported/conflicting assignment and retain the previous working choice.
Wake word remains deferred. Exact factory key and per-OS shortcut forms are not
selected; resolve technical details in the relevant integration contracts.

Source: `sources/activation-decision-2026-10-23.md`.
Current combined record: `decision.activation.2026-10-23` in `records.json`.
Earlier accepted decisions remain unchanged; original records are immutable history.

## Delivery and saved per-app results — accepted 2026-10-23

Automatically insert when the original text target still has focus. Otherwise,
keep the completed result ready for deliberate manual insertion. Preserve existing
clipboard content during automatic delivery and respect newer concurrent user
copies. Explicit Copy is a deliberate clipboard change.

Save completed results in Vara, grouped by the app where dictation was invoked,
including results awaiting manual insertion. A later focus change does not change
the history group. This extends the accepted first-release scope with result
history. Cancelled/abandoned sessions remain excluded from completed history.
Retention duration and raw-versus-final persistence are not decided yet.

Source: `sources/delivery-history-decision-2026-10-23.md`.
Current combined record: `decision.delivery.2026-10-23` in `records.json`.
Original earlier records remain immutable history.

## Text history retention — accepted 2026-10-23

**Keep history until the user deletes it by default.** Offer optional automatic
cleanup after a user-selected period. Do not impose an implicit age or item-count
cap. This decision concerns text history; raw-versus-final text and recovery audio
remain open within `question.data_lifecycle`.

Source: `sources/text-retention-decision-2026-10-23.md`.
Current combined record: `decision.text_retention.2026-10-23` in `records.json`.
Original earlier records remain immutable history; previous accepted requirements
are carried forward without change.

## Optional processing — clarified 2026-10-23

Not all text is processed. Raw dictation bypasses text AI and produces one text
result for delivery/history. Light cleanup is available, not mandatory. Default
mode is still open. When processing is selected, whether both original and processed
versions are saved is still an unanswered retention question.

Source: `sources/optional-processing-correction-2026-10-23.md`.
Current combined record: `decision.optional_processing.2026-10-23` in `records.json`.

## Text versions — accepted 2026-10-23

When text is processed, **save both original and finished text**. Show the finished
text first and make the original inspectable. Unprocessed dictation has only one
transcript. Both saved versions follow the result’s text-retention policy and
are removed together on deletion or expiry. Audio retention is a separate choice.

Source: `sources/text-versions-decision-2026-10-23.md`.
Current combined record: `decision.text_versions.2026-10-23` in `records.json`.
Earlier record statements about versions being open are historical; this decision
resolves that subquestion without changing the optional nature of processing.

## Audio retention — accepted 2026-10-23

Delete audio after successful transcription. Temporarily retain available audio
on transcription failure so the user can retry; delete it when retry succeeds.
No audio in history or history playback feature. Later cleanup/delivery failures
use the saved text and do not justify keeping successfully recognized audio.

Source: `sources/audio-retention-decision-2026-10-23.md`.
Current combined record: `decision.audio_retention.2026-10-23` in `records.json`.
Earlier accepted directions remain unchanged, with immutable records in history.

## Failure-audio deadline — accepted 2026-10-23

**Maximum 24 hours**, with earlier deletion on successful retry or explicit user
deletion. Assign a fixed deadline; retries do not renew it. Expired audio is not
recoverable. Cleanup runs while the app runs and on startup/resume after a stopped
or suspended period; no background daemon or physical deletion while powered off
is implied. Deletion failures must be reported.

This completes the product-level retention decisions in `question.data_lifecycle`.
Detailed implementation remains subject to GDD contract refinement.

Source: `sources/audio-expiry-decision-2026-10-23.md`.
Current combined record: `decision.audio_expiry.2026-10-23` in `records.json`.
Earlier records remain immutable history.

## Default mode and prompts — accepted 2026-10-23

**Direct transcription is the default.** Optional AI processing uses an editable
system prompt, either general or tied to the originating app, for cleanup and
use-case-specific adjustment. Local/cloud rules and saved text-version rules still
apply. Prompt management and app associations are now first-release features,
explicitly replacing the earlier app-customization deferral. No old mode catalog
or specific UI structure is required.

Whether AI runs automatically after an opt-in setting, session overrides and preview
behavior are open in `question.ai_activation`. Prompt assignment alone does not
resolve these details. Ask AI remains deferred.

Source: `sources/default-prompts-decision-2026-10-23.md`.
Current combined record: `decision.default_prompts.2026-10-23` in `records.json`.
Historical earlier decisions remain in history; only the specified deferral changes.

## Per-app automatic AI — accepted 2026-10-23

Explicitly enable AI for an app to automatically use that app’s prompt. **Otherwise
use direct transcription by default.** Prompt existence alone does not enable AI.
Use the originating app’s setting throughout the session. Cloud permission stays
separate. General prompts remain available for deliberate optional processing.

This resolves activation; the previous activation-open statements are historical.
Whether transformed text requires preview is the next separate decision.

Source: `sources/app-ai-decision-2026-10-23.md`.
Current combined record: `decision.app_ai.2026-10-23` in `records.json`.
Earlier records remain immutable history.

## AI result delivery — accepted 2026-10-23

AI-processed text uses **the same automatic insertion rules**, with no extra
preview/approval. Insert when the original text target has unchanged focus;
otherwise retain it for deliberate manual insertion. Clipboard preservation,
per-origin-app history and both saved text versions remain unchanged.

Source: `sources/ai-delivery-decision-2026-10-23.md`.
Current combined record: `decision.ai_delivery.2026-10-23` in `records.json`.
Prior preview-open statements are historical and resolved by this answer.

## Cloud credentials — accepted 2026-10-23

**Always use the user’s own API keys for cloud**, with easy guided in-app setup.
No Vara-managed cloud service, Vara account/login or usage-payment backend in this
scope. Local remains the approachable key-free default. Store keys appropriately,
show real readiness, and redact secrets from diagnostics. Provider/model inclusion
is still open; this does not select any particular service or pricing.

Source: `sources/cloud-byok-decision-2026-10-23.md`.
Current combined record: `decision.cloud_byok.2026-10-23` in `records.json`.
Earlier accepted directions remain unchanged; original records are immutable history.

## Provider baseline — accepted 2026-10-23

Use current Vara provider families and own endpoints as the baseline. Observed
cloud speech: OpenAI, Groq and ElevenLabs. Cloud text: OpenAI, Groq, Anthropic and
Gemini. Include Ollama/local-service and custom OpenAI-compatible text endpoints.
Captured source files and inventory are in `sources/provider-inventory*`.

Provider contracts describe inputs/outputs, errors, cancellation, capabilities,
readiness and actual locality. New integrations should not rewrite capture,
delivery or history. Do not freeze old model IDs or inherit Swift architecture.
The existing custom endpoint feature is for text AI; custom STT is not observed.
Local model/runtime selection remains an evaluation, including Mac-only options.

Source: `sources/provider-baseline-decision-2026-10-23.md`.
Current combined record: `decision.provider_baseline.2026-10-23` in `records.json`.
Prior accepted requirements remain unchanged; original records are historical.

## Optional CLI integrations and voice research — accepted 2026-10-23

Include Claude Code, Codex and Gemini CLI as optional text-AI integrations via
the genuine installed tool’s own authentication. This is a scoped alternative to
personal keys for direct cloud APIs. No token extraction or Vara login/backend.
Actual remote processing remains subject to network/strict-local policy.

Voice research is recorded in `sources/cli-voice-research-2026-10-23.md`:
Gemini documents a promising audio-file/headless/cached-login combination; Codex’s
installed experimental protocol exposes audio/transcript methods; Claude documents
native voice but no reviewed external-audio headless STT entry. None has been
live-tested for this account or accepted as a working Vara speech engine.

Source: `sources/cli-voice-decision-2026-10-23.md`.
Current combined record: `decision.cli_routes.2026-10-23` in `records.json`.
Earlier accepted directions remain unchanged except this explicit CLI-auth route.
Original records remain immutable history.

## Fresh start and focus — accepted 2026-10-23

**Start with an empty profile. No migration/import of old history, settings or
credentials.** Keep the discussion at the product level; implementation details,
benchmarks and adapter questions are deferred until they are needed. Remaining
graph questions are not a mandatory questionnaire. CLI voice research is parked;
no further investigation or unattended work is scheduled.

Source: `sources/fresh-profile-focus-2026-10-23.md`.
Current combined record: `decision.fresh_profile.2026-10-23` in `records.json`.
Earlier accepted product requirements remain in force; records are historical.


## 2026-10-23 — First-release implementation specification and first design

User request: “Lav produktspecifikationen, de klare GDD-kontrakter og et første
design ud fra det, vi har besluttet. Hold fokus på første udgave.”

PRODUCT-SPEC.md consolidates the accepted scope. gdd.json now specifies six
internal responsibility groups and typed action contracts; CONTRACTS.md and
PRODUCT.md are generated reading views. DESIGN.md and design/index.html are a
first interaction/visual proposal with simulated fixtures, not a new user-approved
product direction or working native app. PLAN.md supplies the proposed build order.
The previous ideal manifest/records/plan/view are archived under
reports/history/ideal-before-v1 before replacement. Existing research and accepted
requirements are preserved; deferred features remain excluded from v1.

The main agent chose reversible implementation-spec details: generation guards,
serialized cancellation/finalization, result commit before OS dispatch, no replay
of uncertain paste, private interrupted-text recovery and explicit pending cleanup.
These explain the accepted reliability outcomes; they are not claims of tested
application behavior. No additional major product decision is required to begin.


## 2026-10-23 — Warm precision selected; component design system

User selected the first imagegen mockup and requested a thorough DESIGN.md,
with particular attention to components. gdd/DESIGN.md now specifies that direction:
olive navigation, warm paper, burnt-orange actions, serif reading type, typed
foundation tokens and component/state rules. Its exact values are authored
implementation choices, not recovered source values from the raster image.

The prior short design specification, model, records, graph and snapshot are
preserved under reports/history/design-before-warm-precision. The earlier HTML
prototype remains an interaction study and was not restyled in this documentation
task. Product scope is unchanged. Visual direction selection is not an invented
trusted GDD readiness review of all token details or native behavior.


## 2026-10-23 — Implement the selected design in the clickable prototype

User requested parity with DESIGN.md and the first mockup, using reusable
components to keep maintenance straightforward. The prototype now has the selected
three-column olive/paper/serif composition, original-text inset and floating HUD.
Foundation CSS is generated from DESIGN.md. Pure shared components, view
compositions, fixture data and the demo controller have separate ownership.

At the reference viewport, the two column dividers differ from the image by
about one pixel. Proportional columns and larger reading/preview roles were
reconciled into DESIGN.md to reflect the selected image accurately. Original text
remains collapsed by default per the design contract; the matching-state screenshot
opens it and simulates listening. The prototype remains explicit about simulation.

Twenty-two browser check groups cover visual geometry, history, forms, setup,
async cancellation/recovery, focus, file-URL use and reflow from 320px upward.
This is browser prototype evidence, not native OS/provider/platform acceptance.
The earlier self-contained prototype and previous model metadata are archived
under reports/history/prototype-before-warm-precision.
