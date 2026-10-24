# Vara — desired product

Derived from `gdd.json`. Edit the model, then run `python3 render_product.py`.

Discussion draft, not a committed release plan. Confirmed direction: clean-write, requirements before implementation, and crossplatform exploration. Specific features and priorities remain proposals unless explicitly accepted later.

**core** = proposed baseline; **next** = proposed follow-on; **explore** = validate usefulness first. All nodes remain sketches.

## First successful use

Get from installation to a successful real dictation with necessary permissions and understandable processing choices.

### First successful dictation — core
`action.first_success`

Let a newcomer complete a real dictation without first learning the engine/provider catalog.

- **User action:** Open Vara for the first time and follow the recommended path.
- **Expected outcome:** The user understands activation and processing location and completes an actual speech-to-text insertion or explicitly selected copy workflow. The normal local first-use path requires no terminal commands, endpoint configuration, API credentials or Vara account. Deliberately chosen cloud processing uses personal provider keys entered through a guided app flow. The normal path defaults to local processing and guides necessary setup; cloud is available only through an explicit informed user choice. Speech and text processing destinations remain separately understandable.

### Understand and fix permissions — core
`action.permissions`

Request permissions when required and explain their purpose with a path to recovery.

- **User action:** Start an operation requiring microphone or desktop integration.
- **Expected outcome:** Granted permissions enable the operation; denied permissions produce a specific explanation and available alternatives.

### Choose a working setup — core
`action.recommended_setup`

Offer a machine-appropriate recommendation with visible download, network and credential implications.

- **User action:** Accept or change a recommended setup.
- **Expected outcome:** The chosen setup is checked for actual readiness; unavailable models or credentials are not presented as ready. The normal local first-use path requires no terminal commands, endpoint configuration, API credentials or Vara account. Deliberately chosen cloud processing uses personal provider keys entered through a guided app flow. The normal path defaults to local processing and guides necessary setup; cloud is available only through an explicit informed user choice. Speech and text processing destinations remain separately understandable.

## Everyday dictation

Capture speech reliably and put the intended text in the intended working context.

### Start and stop dictation — core
`action.capture`

Speak while remaining in the working application.

- **User action:** Hold the configured shortcut to speak and release to finish (default), or select toggle mode and press once to start and once to stop. Accessible manual controls remain available.
- **Expected outcome:** Only the requested session captures audio, receives an explicit stop, and produces a completed capture for transcription. Hold-to-talk is the default: key down starts capture and matching key release stops capture for transcription. An optional toggle mode starts on one deliberate activation and stops on the next; changing the configured shortcut does not require code changes.

### Select and recover microphone — core
`action.input_device`

Know which microphone is active and handle device changes.

- **User action:** Select an input or disconnect the active device.
- **Expected outcome:** The active input is clear; a disconnect is reported without silently switching to an unintended input.

### Understand current status — core
`action.session_feedback`

Know whether Vara is listening, processing, finished or waiting for help without losing focus.

- **User action:** Start, stop or inspect an active session.
- **Expected outcome:** Accessible status reflects the real operation; cancel remains discoverable and the original working app keeps focus.

### Recognize speech accurately — core
`action.transcribe`

Recognize everyday Danish/English speech, technical terms and mixed-language utterances.

- **User action:** Finish a capture with an available speech engine.
- **Expected outcome:** The result preserves spoken content, significant names, numbers, negation and uncertainty; silence yields no invented text. When no optional processing is selected, send the transcript directly to delivery/history without invoking a text model. Delete captured audio once speech recognition succeeds. Later text-processing or delivery failures use retained text and do not cause audio retention.

### Deliver to intended destination — core
`action.insert`

Put completed text where the user intended while protecting focus and clipboard state.

- **User action:** Complete a dictation and request insertion or copy.
- **Expected outcome:** Insert only when the destination remains trustworthy; otherwise retain the result and offer explicit copy or insertion. Never claim success without the available confirmation. Automatically insert only when the original text target still has focus. A changed or uncertain target leaves completed text ready for explicit manual insertion. Automatic delivery preserves the preexisting clipboard, including applicable non-text formats. Temporary paste content must not replace or later overwrite a newer concurrent user copy. If safe preservation is unavailable, keep the result for manual use. Use these same focus/clipboard delivery rules for direct and AI-processed text. AI processing does not add a preview/approval gate; completed text inserts automatically when original focus remains unchanged.

### Choose activation mode and shortcut — core
`action.configure_shortcut`

Choose a personal global shortcut and either hold-to-talk or toggle behavior, with hold-to-talk as the default.

- **User action:** Choose an activation mode, record a shortcut and test it in settings.
- **Expected outcome:** The user can choose and test a supported shortcut and activation style. Hold-to-talk is the initial default. Persist a successful choice; an unsuccessful assignment leaves the previous working choice intact.

## Meaning and text processing

Separate faithful transcription, cleanup, rewriting and generated answers.

### Clean text without changing meaning — core
`action.cleanup`

Improve punctuation and remove speech disfluencies while retaining facts, intent and tone.

- **User action:** Choose faithful cleanup for a transcript.
- **Expected outcome:** Return cleaned text and retain access to the raw transcript; add no new facts or instructions.

### Keep the raw transcript — core
`action.verbatim`

Allow direct use of the recognized words without optional AI rewriting.

- **User action:** Select raw transcription.
- **Expected outcome:** No optional generative processing is applied; the user can copy or insert the transcript. This is the initial default for a fresh profile: use the speech model transcript directly, without invoking optional text AI.

### Rewrite with explicit intent — core
`action.rewrite`

Optionally send a transcript to text AI with explicit editable general or app-specific system instructions for the desired use case.

- **User action:** Choose a rewrite preset or custom instruction.
- **Expected outcome:** The transformed result is distinguishable from raw text and can be inspected in saved history without a required pre-insertion confirmation. Use a visible configured system prompt and preserve original/finished text in history. This optional transformation is separate from default direct transcription and remains subject to the chosen local/cloud policy. Deliver the finished text through the ordinary insertion path without extra approval: automatic with unchanged original focus, otherwise retained for deliberate manual insertion.

### Ask AI explicitly — explore
`action.answer`

Treat spoken input as a question only through an explicit separate user choice.

- **User action:** Choose the question/answer workflow.
- **Expected outcome:** Present generated content as an AI answer; do not silently replace ordinary dictation with generated claims.

## Trust and recovery

Cancel safely and recover completed work without repeating speech when possible.

### Cancel without late effects — core
`action.cancel`

Stop or abandon an operation without surprise insertion or saved output afterward.

- **User action:** Cancel during capture, transcription, cleanup or pending delivery.
- **Expected outcome:** The session becomes cancelled; its late results cannot insert text or create a new history entry. Existing completed results are not silently deleted.

### Recover a failed transcription — core
`action.retry`

Retry a failed stage without speaking again when retained input is available.

- **User action:** Select retry after a processing failure.
- **Expected outcome:** Retry the selected stage with an allowed engine; missing or expired audio is explained and strict-local policy is maintained. Temporarily retained failed-transcription audio is available for retry until expiry/deletion. Delete it after successful recognition; do not keep it as a history attachment.

### Recover failed delivery — core
`action.retry_delivery`

Use already completed text after an insertion failure.

- **User action:** Copy or explicitly retry delivery of a completed result.
- **Expected outcome:** Use the existing result without re-recording or retranscribing, and do not duplicate a previously confirmed delivery.

### Find and reuse results — core
`action.history`

Save completed results in Vara, grouped by the application from which dictation was invoked.

- **User action:** Finish a dictation or open saved results in Vara.
- **Expected outcome:** Completed results are saved in Vara and grouped by the application captured at invocation, even if the user changed focus before completion or automatic insertion was skipped. Keep delivery status distinct from processing success. Cancelled/abandoned work is not a completed result. Expose the result for explicit copy or manual reuse; persistence details obey the selected retention policy. Completed text history remains available until manual deletion by default. User-selected automatic cleanup can remove expired entries only when enabled. An unprocessed dictation is stored/shown as one transcript without a fabricated second version. For genuinely processed dictations, persist both the original transcript and processed text; show the finished text first and make the original accessible. When optional processing occurs, save both original and processed text in the same originating-app result. Show the finished text first and let the user inspect the original. Both representations obey the same text-history deletion/expiry policy. Text history contains saved text and metadata, with no retained audio attachment or playback feature.

## Audio and video files

Transcribe existing media with progress, recovery and useful exports.

### Transcribe an existing file — next
`action.import_media`

Turn a supported audio/video file into a transcript without requiring a separate conversion tool.

- **User action:** Drop or choose a media file.
- **Expected outcome:** Validate support, process the audio, show truthful progress and retain the original input unchanged.

### Control long file jobs — next
`action.file_control`

Cancel or retry lengthy transcription without confusing partial and complete output.

- **User action:** Inspect, cancel or retry a file job.
- **Expected outcome:** Progress and partial output are labeled accurately; cancellation stops future effects and retries do not create duplicate exports.

### Export readable transcripts — next
`action.export_text`

Keep transcripts in portable flat files.

- **User action:** Export a completed transcript.
- **Expected outcome:** Save UTF-8 TXT or Markdown at the chosen destination and report write failure without discarding the result.

### Export subtitles and speakers — explore
`action.timed_export`

Use real timing/speaker metadata when provided by a capable engine.

- **User action:** Request SRT/VTT or speaker-labeled output.
- **Expected outcome:** Export only supported and actually produced metadata; do not invent timestamps or speaker identity.

### Queue several files — explore
`action.file_queue`

Process multiple files without selecting each one again.

- **User action:** Add multiple files to a queue.
- **Expected outcome:** Show independent states and per-file retry/cancel; one failure does not erase other results.

## Privacy and data control

Make processing destinations and retention visible and enforce user choices.

### Choose where processing runs — core
`action.processing_policy`

Understand and control the destinations of both audio and text.

- **User action:** Inspect or change the processing policy.
- **Expected outcome:** Show speech and text processing separately and require an explicit choice before expanding where content may be sent. The normal path defaults to local processing and guides necessary setup; cloud is available only through an explicit informed user choice. Speech and text processing destinations remain separately understandable. Cloud is an explicit optional route using the user’s own API keys. Local default operation needs no Vara account or cloud-provider key.

### Work only on this device — core
`action.strict_local`

Use local processing without hidden cloud fallback.

- **User action:** Enable only-on-this-device processing.
- **Expected outcome:** Audio and text processing remain on-device; unavailable local capability blocks the relevant stage or offers raw output rather than uploading. The normal path defaults to local processing and guides necessary setup; cloud is available only through an explicit informed user choice. Speech and text processing destinations remain separately understandable.

### Control history and recovery retention — core
`action.retention`

Control whether text/audio persists and when it is deleted.

- **User action:** Set retention, turn history off or delete retained content.
- **Expected outcome:** Clearly distinguish text history, temporary recovery audio and diagnostics; enforce the selected retention without implying secure erasure guarantees that are not established. By default retain text history until the user deletes it, without an implicit age or item-count cap. Only an explicitly enabled automatic-cleanup setting applies a user-chosen retention period. Text history and audio retention are distinct policies. Deleting or expiring a history result covers both its original and processed text where both exist; do not leave the original as hidden retained history. Successful-transcription audio is deleted regardless of text-history lifetime. Failed-transcription audio is retained only temporarily for retry, with a distinct expiry; it is not a history item. Failed-transcription audio has a fixed maximum 24-hour recovery lifetime, deleted earlier on successful retry or explicit user deletion. Retry attempts do not extend the original expiry. Expired input cannot be reused; purge it at expiry while running and on next startup/resume if execution was suspended. Report cleanup failures instead of claiming deletion succeeded.

### Manage provider credentials — core
`action.credentials`

Configure the user’s own optional cloud-provider API keys through an approachable in-app flow.

- **User action:** Select a cloud provider and add, replace, validate or remove a personal API key in the app.
- **Expected outcome:** Store secrets using appropriate OS facilities, redact them from diagnostics and accurately distinguish configured from validated credentials. Direct cloud APIs use personal provider API keys. Explicitly selected CLI adapters use their genuine tool’s own authentication as a separate accepted route. No Vara account, managed proxy or billing backend is required. Let the user choose a supported provider and add, replace or remove its key through the app with clear guidance. Distinguish missing, configured, validated and rejected credentials without exposing secrets.

## Personal vocabulary and control

Support personal terminology, deliberate presets and advanced configuration without burdening first use.

### Teach personal terminology — next
`action.vocabulary`

Improve recognition of names, technical terms and mixed-language vocabulary.

- **User action:** Add terms and preferred spelling.
- **Expected outcome:** Apply supported recognition hints and disclose when the selected engine cannot use them.

### Save explicit text corrections — next
`action.corrections`

Reuse a deliberate correction without monitoring typing in other applications.

- **User action:** Choose to save a correction as a rule.
- **Expected outcome:** Preview, edit, disable or remove the rule; apply it predictably without hidden learning from unrelated app activity.

### Configure reusable AI system prompts — core
`action.presets`

Create/edit a general system prompt or an app-specific prompt for optional cleanup and use-case-specific adjustment.

- **User action:** Create or edit a general system prompt or assign instructions to a named application.
- **Expected outcome:** Save editable instructions for optional text processing. Allow a general prompt and app-specific prompts, and make the selected instructions inspectable. Validate required engine capabilities and honor local/cloud policy. Saving a prompt alone does not enable automatic processing. An explicit app enable setting activates its prompt automatically; otherwise direct transcription is the default.

### Enable automatic AI for an app — core
`action.app_rules`

Explicitly enable automatic AI processing with a system prompt for a chosen originating application; all other apps use direct transcription by default.

- **User action:** Enable AI processing for a chosen app and configure its system prompt.
- **Expected outcome:** The user can enable or disable automatic AI processing for a specific app and configure its prompt. When enabled, dictations invoked in that app automatically use its prompt. When no enabled app-specific setting applies, direct transcription is the default. A stored general or app prompt alone does not activate processing. Resolve the processing choice from the originating app at invocation; later focus changes do not switch the prompt. An app AI setting cannot authorize a disallowed cloud destination.

### Configure advanced engines — core
`action.advanced_engines`

Use optional local services, custom endpoints or installed CLI tools when they offer a concrete benefit.

- **User action:** Open advanced configuration and add an engine.
- **Expected outcome:** Validate capabilities, readiness and processing destination; unsupported settings are not shown as working. Support the observed cloud-provider families and existing custom OpenAI-compatible text endpoints as the initial baseline. Expose Ollama/local-service configuration as an advanced route. Extend providers through common contracts, without importing the old app architecture or freezing legacy model IDs. Configure custom text endpoints with name, URL, model and optional secret reference, and classify actual processing locality. Unsupported protocol/capability is reported rather than treated as compatible. Optional Claude Code, Codex and Gemini CLI text-processing routes are included through their own provider contracts.

### Start hands-free — explore
`action.wake_word`

Explore deliberate hands-free activation and silence stopping.

- **User action:** Explicitly enable wake-word listening.
- **Expected outcome:** Show ongoing microphone use, avoid microphone contention, provide manual cancel and measure false activations and resource cost before inclusion.

### Use an optional installed CLI for text AI — core
`action.cli_processing`

Use Claude Code, Codex or Gemini CLI as an optional text-processing provider through its own installed/authenticated tool and an explicit provider contract.

- **User action:** Explicitly choose an installed Claude Code, Codex or Gemini CLI text provider.
- **Expected outcome:** Offer the three CLI text routes only as optional integrations with actual tool/readiness detection. Use the tool’s own authentication; do not require a Vara subscription or copy login tokens. Apply the configured system prompt and return only the intended text result through the same history/delivery contracts. Respect actual network locality, cancellation and timeout; failure preserves raw text.

## Platforms and product quality

Deliver an accessible, responsive, installable desktop product with explicit platform support.

### Use supported desktop platforms — core
`action.desktop_support`

Target macOS, Windows and Linux from the beginning of the first release; develop on Mac first and verify Windows/Linux at the final stage.

- **User action:** Install and use Vara on a declared supported OS/desktop environment.
- **Expected outcome:** Publish a tested capability matrix; unavailable shortcuts, insertion or overlays have explicit alternatives rather than false parity claims. The first-release core dictation flow is designed for all three target platforms from the start. Final runtime verification establishes actual support; building on Mac or using Rust alone is not treated as proof.

### Use accessible controls — core
`action.accessible_use`

Complete essential flows with keyboard and assistive technology.

- **User action:** Navigate setup, dictation controls, recovery and results without pointer-only interaction.
- **Expected outcome:** Expose semantic controls, visible focus, text status, scaling and reduced-motion support.

### Receive timely results — core
`action.responsive_use`

Keep the app responsive and reduce real waiting time without sacrificing capture or meaning.

- **User action:** Start capture, stop speech or inspect progress.
- **Expected outcome:** Measure startup, capture-start, raw-result and final-delivery latency separately; use preparation or streaming only where it improves the tested experience. Prioritize fast response using suitable compact local models; do not impose or claim a universal seconds budget before representative model/hardware measurements. Treat optional text cleanup and final delivery as part of the user-perceived wait.

### Resolve actionable problems — core
`action.diagnostics`

Understand microphone, engine, processing and delivery failures without sharing private content.

- **User action:** Open an error explanation or export diagnostics.
- **Expected outcome:** Identify the failed stage and next action; diagnostic exports exclude content and secrets by default and expose what is included.

### Install and update safely — next
`action.updates`

Install an identifiable build and receive updates while preserving settings/data.

- **User action:** Install or apply an update.
- **Expected outcome:** Use platform-appropriate signing and distribution; report failures and preserve usable data.

### Bring useful existing data — out_of_scope
`action.migration`

Excluded: the new Vara starts with an empty profile and does not import the old app’s history, settings or credentials.

- **User action:** Out of scope. Start with a fresh profile. Do not implement migration/import under this product mandate.
- **Expected outcome:** Preview supported settings/history imports, handle incompatible values, and leave the old app’s data untouched.

### Use the same outcomes from a terminal — explore
`action.cli`

Explore scriptable file transcription and portable outputs without opening the desktop UI.

- **User action:** Invoke a documented CLI command.
- **Expected outcome:** Apply the same processing and privacy rules, return explicit exit status and support cancellation.

## Open decisions

- **question.quality_budgets:** Fast model-dependent response is accepted; no universal seconds limit is selected. After representative model/hardware measurements, fix the Danish/English corpus, meaning/name/number quality criteria and cold/warm end-to-end latency budgets. This calibration is pending acceptance evidence, not a demand for an arbitrary upfront time limit.
- **question.file_scope:** Which containers, maximum duration/size, progress/resume semantics and export metadata must the file workflow support?
- **question.engine_scope:** The existing cloud-provider families, Ollama and custom OpenAI-compatible text endpoints are the baseline, captured in source.provider_inventory. Still evaluate compact local models/runtime choices and revalidate current provider model/protocol compatibility during implementation. Optional CLI text providers are included. Subscription speech capability is separately tracked in question.subscription_speech.
- **question.wake_viability:** Does hands-free activation justify its false-trigger rate, resource cost and ongoing microphone use?
- **question.interaction_design:** Develop a coherent interaction/design proposal for the accepted dictation, per-app history and settings experience. Bring consequential product/design choices to the user; do not require a fine-grained UI questionnaire. File processing is deferred.
- **question.technology:** With Rust as the working implementation direction and all three desktop platforms targeted from the start, which UI toolkit, audio/media dependencies and OS adapters best satisfy the contracts? Their real crossplatform behavior must be verified in the agreed final phase.
- **question.release_distribution:** Which signing, distribution, update/recovery and sustainable maintenance arrangements are needed?
- **question.specification:** Before implementation, refine the chosen slice with complete types, external effects, risk/error coverage and resolved relevant questions; no contract sketch here claims readiness.
- **question.platform_envelope:** Within the accepted macOS/Windows/Linux first-release targets, which OS versions, CPU architectures, Linux distributions/desktop environments and integration fallbacks define the supported/tested envelope? Set this before final platform acceptance; it does not require early access to Windows/Linux. Include supported configurable shortcut forms, any modifier-only requirements and conflict/permission limitations; no universal key-support claim is implied.
- **question.subscription_speech:** Which supported subscription/account-authenticated CLI speech routes can satisfy Vara’s STT contract? Research found documented Gemini audio+cached-headless capability, experimental local Codex realtime schemas, and Claude native dictation without an established external-audio/headless entry. Live entitlement, fidelity, latency and cancellation remain unverified; no route is promised as a release speech engine.

## Resolved decisions

- **question.audience:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.release_scope:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.platforms:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.default_processing:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.activation:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.delivery:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.data_lifecycle:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.text_modes:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.migration_scope:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.cloud_access:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.text_retention:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.text_versions:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.audio_policy:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.ai_activation:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.ai_preview:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.provider_baseline:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.
- **question.cli_routes:** See `decision.fresh_profile.2026-10-23` in `records.json` and `DECISIONS.md`.

## Deliberate exclusions

- No inherited provider catalog, screen layout or software module structure.
- No chosen Rust/Tauri stack or mandatory cloud/account/billing service.
- No arbitrary desktop agent or silent collection of other applications’ content.
- No implementation or executed application-test claim.
