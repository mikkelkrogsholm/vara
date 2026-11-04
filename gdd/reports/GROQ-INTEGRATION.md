# Groq — integration update, October 23, 2026

Groq was already present in the codebase. This change completes the relevant model
choices, updates the default text model, adds a key/catalog check and verifies the
actual HTTP requests with a local server.

## Using it in the app

1. Restart the updated app. Open **Settings → Connections → Groq → Manage key**.
   Enter your own Groq key and choose **Save key**.
2. Open the key dialog again and choose **Check access** to check whether the key
   is accepted and the configured models exist in the model catalog.
3. Select **Groq · Whisper Turbo** or **Groq · Whisper Large-v3** under audio
   processing and save settings to use Groq for speech.
4. For optional AI processing select **Groq · GPT-OSS 120B** in general AI
   settings or an app rule. Saving a key does not enable cloud or AI.

Redux remains the default for new profiles. The cloud choice appears in the sidebar.
Strict local mode also rejects Groq. Direct transcription and history rules do not
change. The API key is stored in the OS key store, never in history or a UI snapshot.

Saved model names are preserved. If an older profile still has a discontinued Groq
text model, it can be changed under **Models and custom endpoints**; the access check
shows if the model name is missing from the catalog.

## Models and API

- Speech: `whisper-large-v3-turbo` and `whisper-large-v3` on Groq's fixed
  `/openai/v1/audio/transcriptions` endpoint, multipart WAV, Bearer auth, optional
  language, JSON response and temperature 0.
- Text: new default `openai/gpt-oss-120b` on `/openai/v1/chat/completions`.
  System prompt and transcript are sent as separate messages. Reasoning is kept
  out of the response; low reasoning effort and an explicit token limit are used.
- Access check: only `GET /openai/v1/models` with the saved key. No audio or
  transcript is sent. Catalog presence is not a real transcription test or a
  guarantee of model permissions, quota or billing.

According to Groq, the previous default `llama-3.3-70b-versatile` is discontinued
for ordinary accounts. Sources checked October 23, 2026:
[Speech to Text](https://console.groq.com/docs/speech-to-text),
[deprecations](https://console.groq.com/docs/deprecations),
[GPT-OSS 120B](https://console.groq.com/docs/model/openai/gpt-oss-120b),
[reasoning format](https://console.groq.com/docs/reasoning).

## Verification

**24 Rust tests pass**: 11 domain/storage tests, 7 model/integrity tests and 6 new
local HTTP contract tests. The latter check:

- Correct multipart audio request, model, language, file and Authorization header.
- Correct text JSON with system prompt and only the finished text as result.
- An incomplete AI response becomes an error so the original text can be used.
- 401/429 errors produce useful messages without reproducing upstream errors or keys.
- The catalog check sends a GET without an audio/text body.
- Accepted cancellation interrupts a pending cloud request.

**12 UI test groups pass**, including Groq key, access response/error, secret input
clearing, both speech model choices and explicit cloud selection. The test key is
fictional and used only against a local server/IPC fixture. A release bundle was
built for Mac without the test harness. GDD model and graph validation pass.

## Limitations

No real Groq key was read or used and no audio was sent to Groq during this work.
Live account, latency/quality and the new native UI therefore lack final verification.
The user's earlier request to continue without native UI checks was respected. The
extra real-weight Redux test had already passed during the previous integration; it
was not repeated for this cloud change.
