# Existing Vara provider inventory — captured 2026-10-23

This describes provider choices and wiring observed in local Swift files, not a
claim that every service/model is current, available or tested. The original source registrations and hashes are retained in historical GDD
revisions. The captured Swift files are now available through Git history.

| Purpose | Observed provider options | Intended handling |
| --- | --- | --- |
| Cloud speech recognition | ElevenLabs Scribe, Groq Whisper, OpenAI realtime and batch transcription | Initial cloud-provider baseline, own API keys |
| Cloud text AI | OpenAI, Groq, Anthropic, Google Gemini | Initial cloud-provider baseline, own API keys |
| Local text service | Ollama | Include configurable local-service integration as an advanced option |
| Custom text endpoints | OpenAI-compatible base URL, model selection and optional key | Include own endpoints; do not assume an endpoint is on-device just because it is custom |
| Existing local speech options | Local Hviske, WhisperKit, Apple Speech | Observed capability references, not a requirement to reuse Swift/Python runtimes; new local model/runtime is evaluated independently |
| Existing local text API | Apple Foundation Models | Observed Mac-specific option; do not imply availability on other platforms |
| Existing text CLI routes | Claude Code, Codex, Gemini CLI | Observed installed/authenticated subprocess options; inclusion and policy treatment need explicit clarification with own-key cloud policy |

The custom endpoint model in the inspected code is **LLM/text-specific**. The
current speech-backend selector has no arbitrary custom-STT endpoint case. Do not
invent prior custom-STT support or silently expand the baseline to a new protocol.

Launching a CLI on-device does not by itself establish that its text processing
stays on-device. Provider locality/capabilities must describe actual processing,
not the launch location or a legacy flag.

Model IDs and API protocols are revalidated against primary documentation during
implementation. No calls were made to providers, no credentials were read, and
no endpoint conformance or platform compatibility was tested.

ElevenLabs project guidance was read and the official docs index checked at
https://elevenlabs.io/docs/llms.txt. This inventory uses local provider wiring;
it makes no current API-schema/model/price assertions.

## Retired implementation

The Swift source captures were removed from the working tree on 2026-10-25.
They remain available in Git at revision
`3eb56eb8dce1edff66f8b371acd9fb62ff08ecf5` under
`gdd/sources/provider-inventory/`. This inventory is historical evidence, not
a dependency of the Rust application.
