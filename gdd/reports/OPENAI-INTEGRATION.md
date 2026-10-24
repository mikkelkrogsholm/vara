# OpenAI integration — 2026-10-23

OpenAI is optional. Local Parakeet Redux and direct transcription remain the
default. Add your own API key in Settings → Connections → OpenAI; it is stored
in the OS secret store under `open_ai`. Saving a key does not select cloud or AI.

## Current endpoints and models

- Speech: `POST https://api.openai.com/v1/audio/transcriptions`, multipart WAV,
  model `gpt-transcribe`, `response_format=json`, optional `languages[]=da`.
  Auto detection omits the language field. This is completed-recording dictation,
  so a continuous Realtime session is unnecessary.
- Optional text: `POST https://api.openai.com/v1/responses`, model `gpt-6-luna`,
  `reasoning.effort=none` for focused cleanup, frozen system prompt in
  `instructions`, current transcript in `input`, `max_output_tokens=8192`.
  `store=false` disables response state storage; this is not a claim of zero
  provider logging. No tools or conversation history are supplied.
- Key/catalogue check: `GET https://api.openai.com/v1/models`; no audio/text body.
  Key acceptance and catalogue presence do not guarantee inference access/quota.

New profiles receive these model defaults. Existing saved model names and chosen
engines are preserved. If an older profile has `gpt-4o-mini-transcribe` or
`gpt-4.1-mini`, update the relevant model name under Models and custom endpoints
to the values above. Legacy speech models still use their singular `language`
field. The older transcribe models are deprecated according to the current docs.

The Responses parser visits typed output items and only collects completed
assistant `output_text`; reasoning never becomes inserted text. Incomplete,
refused or empty results trigger the original-text fallback. Cancellation,
redacted authentication/rate errors, audio expiry and safe insertion retain
the existing shared contracts. Groq/custom/Ollama retain their own API route.

## Verification

27 Rust tests pass: 11 core/storage, 9 local HTTP protocol and 7 model integrity.
New HTTP tests verify real multipart language arrays, Responses request and
multi-part output, ignored reasoning, explicit no-storage, and incomplete,
refused and empty response failures. Existing tests cover catalogue GET,
401/429 redaction and pending-request cancellation using the shared transport.

13 production-UI browser groups pass, including OpenAI key clearing, current
model catalogue results, access errors and explicit cloud choice without
silently enabling AI. Mac release app built successfully (24.86 MiB), with the
normal production configuration. No native UI verification was performed, as
requested. No live OpenAI key/account or transcription was used; cloud quality,
latency, billing and actual model access remain unverified.

## Official references checked

- [File transcription](https://developers.openai.com/api/docs/guides/speech-to-text)
- [GPT-Transcribe migration and language hints](https://developers.openai.com/cookbook/examples/migrating_from_whisper_to_gpt_transcribe)
- [GPT-6 Luna model and supported reasoning effort](https://developers.openai.com/api/docs/models/gpt-6-luna)
- [Text generation and typed output](https://developers.openai.com/api/docs/guides/text)
- [Responses migration and storage](https://developers.openai.com/api/docs/guides/migrate-to-responses)
