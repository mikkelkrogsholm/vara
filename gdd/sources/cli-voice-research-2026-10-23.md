# Subscription CLI voice investigation — 2026-10-23

Scope: native voice availability, documented programmatic input, authentication
and evidence for use from Vara. This is desk research plus local static inspection.
No provider request, account entitlement check or real-audio transcription test
was executed. No credentials or token files were read.

| Tool | What is established | What is not yet established for Vara |
| --- | --- | --- |
| Claude Code | Native `/voice` transcribes microphone input remotely with a Claude.ai account; the docs exclude direct API-key auth from that feature. | No supported headless WAV/PCM transcription entry was found in the reviewed voice/headless docs. Native microphone UI is not automatically an embeddable STT adapter. |
| Codex | Native desktop voice is documented. Installed CLI 0.145.0 generates experimental `thread/realtime/start`, `appendAudio` and transcript notification schemas. | No live account test, stable STT-only contract or subscription entitlement for this particular integration was established. Experimental audio protocol existence is not a runtime PASS. |
| Gemini CLI | `read_file` accepts audio; documented headless execution returns structured output and can reuse cached authentication. Google sign-in supports personal/Pro/Ultra accounts. | This combination is a promising batch-audio transcription candidate, not a proven low-latency streaming dictation backend. Exact account/model, transcription fidelity and timing require a test. |

## Primary documentation

- Claude native voice: https://code.claude.com/docs/en/voice-dictation
- Claude programmatic CLI: https://code.claude.com/docs/en/headless
- Claude subscriber CLI/SDK usage update: https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan
- Claude integration/authentication conditions: https://code.claude.com/docs/en/legal-and-compliance
- Codex app-server and authentication: https://learn.chatgpt.com/docs/app-server
- Codex/desktop voice: https://learn.chatgpt.com/docs/features/voice
- Gemini audio file tool: https://geminicli.com/docs/tools/file-system/
- Gemini cached authentication and headless use: https://geminicli.com/docs/get-started/authentication/
- Gemini programmatic output: https://geminicli.com/docs/cli/headless/

These sources distinguish use of the genuine installed tool from extracting its
credentials for another API. Preserve tool identity and its own authentication
flow. Do not promise unlimited/free voice or transfer native-app entitlements to
an arbitrary endpoint. OpenAI app-server documentation distinguishes local/open-
source integrations from commercial/hosted auth use; this matters if distribution
scope changes. Anthropic’s current conditions distinguish unmodified tool use with
personal credentials from offering a third-party Claude.ai login or credential
proxy. No legal conclusion or blanket entitlement is inferred.

## Local inspection actually performed

- `codex --version`: 0.145.0. `claude --version`: 2.1.186.
- Generated Codex experimental JSON schema using its documented local generator;
  selected files and an inspection report are under `reports/cli-audio/`.
- Gemini installed package metadata: 0.29.5. Its `read-file.js` delegates to
  `processSingleFileContent`; local `fileUtils.js` builds inline audio parts.
- A combined version/help command stalled at `gemini --version` and was stopped.
  No model invocation was made and no authentication failure was diagnosed from
  that stall. Version was instead read from installed package metadata.

## Product interpretation

Include the three optional CLI text adapters behind provider contracts. Treat
actual remote processing as network use even when a subprocess runs on-device;
strict-local policy does not permit it merely because it is a CLI. Keep native
CLI authentication in the CLI and direct API credentials in Vara’s secret store.

For speech, prioritize a bounded synthetic-audio test of Gemini’s documented
headless/file route when refining that adapter. Codex realtime is an experimental
candidate requiring account and STT-semantics validation. Claude native voice is
established, but a supported externally supplied-audio route remains unknown.
No subscription speech engine becomes a default or release requirement here.
Any eventual speech adapter must fulfill the same cancellation, locality,
24-hour failure recovery and successful-audio-deletion contracts as other engines.
