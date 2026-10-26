# Vara v1 — product specification

Status: implementation specification derived from the accepted product decisions.
The interaction design is a first proposal, not a claim of user approval or a
working desktop application. The current GDD model owns the detailed contracts.

## Purpose

Help Mikkel turn everyday Danish/English speech into usable text in the desktop
application where he is working. Other users must be able to get started without
terminal commands or infrastructure setup. The product should disappear into
the working day: activate, speak, finish, continue working.

## First-release scope

| Requirement | Observable behavior | GDD responsibility |
| --- | --- | --- |
| V1-01 First success | Start with an empty profile, install the recommended local model through the app, grant necessary permissions, choose/test a shortcut and complete a real trial dictation. No account or API key on the default path. | `action.first_success`, `action.recommended_setup` |
| V1-02 Activation | Default hold-to-talk; optional press-to-start/press-to-stop. The user chooses the shortcut. Display the actual binding and reject conflicts without losing a working binding. | `action.capture`, `action.configure_shortcut` |
| V1-03 Direct by default | Use the speech model's transcript directly unless the user deliberately selects text AI or enables it for the originating app. Speech-model punctuation is not a separate AI cleanup step. | `action.verbatim`, `action.resolve_processing` |
| V1-04 Optional text AI | Editable general and app-specific system prompts. An explicitly enabled app setting automatically applies that app's prompt. Resolve the app and processing choice at invocation; later focus changes do not change the instructions. | `action.presets`, `action.app_rules`, `action.rewrite` |
| V1-05 Safe delivery | Automatically insert when the original text target remains focused. Changed/unknown focus leaves text ready for deliberate manual use. No extra approval for AI output. Preserve clipboard contents during automatic delivery and respect a newer concurrent user copy. | `action.insert` |
| V1-06 App-grouped history | Save completed results under the app where dictation started. Present processed text first with original inspectable; unprocessed results contain one text. A delivery failure does not discard a successful result. | `action.save_result`, `action.history` |
| V1-07 Text retention | Keep history until manual deletion by default. Optional automatic cleanup uses a user-selected period. Deleting/expiring a result removes its original and processed text together. No hidden count cap. | `action.retention`, `action.delete_result` |
| V1-08 Audio lifecycle | Delete successful-transcription audio. Retain failed-transcription audio only for retry, at most 24 hours, deleted earlier on successful retry or manual deletion. Retry never extends the original deadline. No audio in history. | `action.recovery_audio`, `action.retry` |
| V1-09 Local and cloud | Local is the default for speech and any selected text processing. Cloud is a deliberate choice; local failure never authorizes upload. Explain audio and text destinations separately. Custom LAN endpoints and remote CLI models are not on-device processing. | `action.processing_policy`, `action.strict_local` |
| V1-10 Providers | Existing cloud speech families: OpenAI, Groq, ElevenLabs. Cloud text: OpenAI, Groq, Anthropic, Gemini. Include Ollama and own compatible text endpoints. Direct APIs use personal keys with guided setup. Optional Claude Code/Codex/Gemini CLI text providers use their own genuine tool/authentication. | `action.advanced_engines`, `action.credentials`, `action.cli_processing` |
| V1-11 Crossplatform | Target macOS, Windows and Linux from the beginning. Develop/test on Mac first; validate Windows/Linux at the end. Actual capabilities and unsupported environments must be reported honestly. | `action.desktop_support` |
| V1-12 Reliability | Show listening/processing/delivery state, allow cancellation, preserve raw text after optional AI failure, expose failed-stage retry and never let a cancelled generation later paste or publish history. | `action.use_vara`, `action.cancel`, `action.retry_delivery` |

## Primary journeys

### First use

Welcome → recommended on-device setup → necessary permissions and tested shortcut
→ try a real dictation → open the everyday experience. Model download shows size,
progress, cancel and retry. Setup is complete only after a real result, not merely
because a key exists or a text box contains typed text. A user can deliberately
choose cloud and enter their own provider key instead; that route is optional.

### Dictate while working

1. At activation, capture an opaque app/text-target identity, invocation time,
   engine selection, processing preference and locality policy for this session.
2. Start the microphone and show listening state without stealing focus.
3. Stop on release or the second toggle activation. Repeated keydown is not a
   second activation. Never overlap microphone captures.
4. Recognize speech. Show progress and cancellation; silence yields no insertion.
5. If selected, run the configured system prompt through the allowed text engine.
   Failure/timeout uses the original transcript with a visible fallback status.
6. Finalize one result under its originating app, then attempt delivery using
   the original target identity. A changed field in the same app also counts as
   changed focus. Failed/uncertain delivery leaves the result ready for manual use.

### Revisit a result

Open History → choose an originating app → choose a result → read the finished
text, inspect the original when processing occurred, copy deliberately or delete.
History and recovery are distinct: failed audio appears as a temporary recovery
task, not as an audio attachment in completed history. Search is a small proposed
convenience in the design, not a new semantic dependency of dictation.

### Customize one app

Open App preferences → choose an installed application → enable automatic AI
→ edit its system prompt and choose an available text provider → save. Display
where text will be processed. A prompt alone does not enable AI. No app-content
inspection, clipboard learning or silent context capture is required.

## Reliability and commit points

- A session has a generation token. Accepted cancellation invalidates pending
  work, stops capture and prevents late publication/delivery. Provider cancellation
  can be best effort; stale results are rejected independently of provider behavior.
- Cancellation and finalization are serialized. Once a result is committed and
  delivery has crossed its irreversible OS handoff, cancellation cannot promise
  undo. Return an honest `AlreadyCommitted` outcome; never display false cancellation.
- Saving text and pasting into another process are not one database transaction.
  Save the completed result before dispatching the irreversible delivery. Record
  delivery independently. A crash after dispatch can leave delivery unknown;
  never automatically replay an unknown paste on restart.
- Raw text may be staged privately before optional processing to survive a crash.
  It is not completed history. Accepted pre-commit cancellation removes staging.
  On restart, show an interrupted-text recovery choice; never auto-paste it.
- If history cannot be written, retain the available text in memory, show the
  storage error and offer deliberate copy/retry. Do not claim it was saved or
  silently create repeated automatic deliveries.
- Delete captured audio after successful recognition, independently of later
  text-model or delivery failure. A failed delete is visible and queued for
  cleanup; the app must not report deletion as completed when it did not happen.
- Enforce the fixed audio expiry before retry and during cleanup. On startup or
  resume, purge expired/orphaned capture files before offering recovery. An app
  that is not running cannot physically delete a file; expired audio is never
  advertised as recoverable merely because physical cleanup was delayed.

## Quality and accessibility

Preserve meaning, names, numbers, negation and uncertainty. User-authored prompts
can intentionally change style, but must not accidentally turn input into model
instructions or fabricate factual content. Treat the transcript as data distinct
from the system prompt. Evaluate Danish, English, mixed technical language,
silence/noise and representative short/long dictations using permitted fixtures.

Record cold/warm model loading, activation-to-capture, stop-to-original and
stop-to-delivery separately. Choose a compact default from measurements; there is
no invented universal seconds promise. Foreground response and cancellation must
remain responsive while providers work.

Keyboard navigation, visible focus, semantic labels and screen-reader status
announcements apply to the whole essential journey. Do not require color, hover,
motion or a global shortcut for access to essential controls. Honor text scaling
and reduced motion. The design proposal uses Danish UI; the release retains a
Danish/English localization boundary. Exact translations are implementation work.

## Architecture direction

One desktop application, one shared Rust domain core and explicit adapter
boundaries. Six responsibility groups in GDD describe contracts, not six deployed
services: Session, Desktop integration, Engines, Preferences, History/lifecycle,
and Experience. No hosted backend, plugin marketplace or generic agent framework.

Speech recognition and text AI are separate contracts. A provider reports actual
capabilities, locality and readiness. The domain owns session policy, result
retention and delivery decisions; providers own protocol-specific I/O. Adding a
provider must not rewrite microphone capture, history or focus-safe insertion.

The UI toolkit, precise native integrations and selected local model are pending
engineering validation. Internal audio/text handles avoid sending binary data or
credentials through presentation state. Each OS adapter reports what it actually
supports; source portability is not runtime acceptance evidence.

## Not in v1

File transcription, queues, subtitles, speaker labels, wake word, standalone Ask
AI, an end-user Vara CLI, automatic vocabulary learning, old-data migration,
Vara accounts, cloud billing/proxy infrastructure and subscription-based speech
integration as a promised backend. Existing subscription voice research is parked.
Optional installed CLI **text** providers remain in scope.

## Acceptance and remaining engineering work

All v1 behaviors above must be delivered; incremental build slices do not reduce
release scope. Start with a local direct dictation vertical slice, then include
history/recovery, prompt-based processing and the accepted provider catalog.
Finish with target-platform integration tests and installation checks.

Before claiming release support, establish exact OS/hardware/desktop-environment
coverage, tested provider/runtime versions, a representative quality baseline and
measured latency envelopes. These are engineering acceptance decisions, not an
additional feature questionnaire. GDD structural readiness, a working mockup and
application acceptance remain separate claims.

## Approved extension — file transcription (2026-10-25)

The user approved implementation of a separate Transskribering tab following
`TRANSCRIPTION-SPEC.md` and the two transcription mockups. This extends the
application beyond the original dictation-only release scope. It does not replace
or weaken V1-01 through V1-12 above.

Imported documents own a distinct library, durable file jobs, original/corrected
text, explicit timing precision, optional speaker labels and separate AI output.
Selecting files does not start processing; a deliberate local/cloud model choice
and explicit start do. No automatic insertion is performed for imported files.

Playback uses a private local normalized copy retained with the file document
until deletion, explained before import. Original user files remain untouched.
The dictation audio-deletion/24-hour recovery rule applies to captured dictation,
not to these expressly retained file playback copies. Local failure never grants
cloud permission. Only actual model timings may drive word navigation or exports.
See the dedicated specification for scope, conditional capabilities and exclusions.

Cancellation of an unfinished file job removes its private working audio; the
source file remains untouched. Failed recognition can reuse a valid retained
playback copy on retry even if the external original has moved.
