# Issue 44 — offline provider contracts

Completed 2026-11-06 against base `6490aef`. No provider credentials, external
endpoints, model calls/downloads, private profiles or paid tests were used.

## Executed coverage

- Existing OpenAI speech and Responses tests remain: authenticated multipart,
  language array versus legacy/automatic language, selected model, frozen prompt,
  `store:false`, no previous response/tools, completed assistant output only,
  refusal/incomplete/empty failures.
- ElevenLabs dictation and rich-file HTTP requests run production helpers against
  loopback servers. They assert exact path, header-only `xi-api-key`, selected
  `model_id`, WAV MIME/filename/body, explicit versus automatic language,
  `tag_audio_events:false`, dictation `diarize:false`, and file `diarize:true`
  plus word granularity. Wrong OpenAI field names are absent.
- Rich-file responses preserve original text, skip spacing records, retain word
  times and speaker changes. No word array/empty array produces untimed text;
  missing, nonnumeric, negative and reversed word times fail rather than being
  invented. Missing word text also fails.
- Gemini HTTP assertions cover selected model path, header-only `x-goog-api-key`,
  exact frozen system instruction/user JSON and token configuration. STOP returns
  concatenated final text with thought parts excluded. MAX_TOKENS, SAFETY,
  prompt blocking, missing candidates/parts, empty/whitespace/thought-only and
  nontext output fail safely.
- Every new route (ElevenLabs dictation/file, Gemini text) executes missing,
  blank and invalid-header key checks; 401/403/429/500/503; connection refusal;
  timeout; truncated body; oversized body; malformed JSON; pre-cancel and
  cancellation after the server receives the request but before late success.
  Failures assert safe wire codes and absence of response bodies, keys and URLs.
  All server threads and spawned tasks finish before their tests return.
- Actual `Files::command("file_ai")` calls for OpenAI/Groq/Gemini with missing
  credentials preserve the full stored original, edits, existing AI and revision,
  verified in memory and through a freshly opened database. Exactly the selected
  engine requests credentials; there is no silent switch. A blocked credential
  lookup permits actual `file_ai_cancel`, then resumes into the production engine
  cancellation guard: no later processing/publication occurs and disk is unchanged.
  Existing core session/store tests also cover sticky cancellation and rejection
  of staging/commit after cancellation, including after restart.

## Production changes and regression evidence

Two private endpoint helpers extract existing Gemini and rich-file request code;
production URLs remain fixed. File requests now reject pre-cancel/missing keys
before reading audio, matching dictation's existing ordering.

The new Gemini empty-output test initially failed on `Ok("")` (log below).
The Gemini adapter now returns `ai.empty_original_preserved` for empty final
text. `Engines::text` already rejected empty text at the public boundary before
this change; this is adapter-contract hardening, not evidence of prior app data
loss. Other existing failure codes and redaction remain unchanged.

## Verification

Environment: `PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH`,
`DEVELOPER_DIR=/Library/Developer/CommandLineTools`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_RELEASE_DEBUG=0`, `CARGO_BUILD_JOBS=4`.

| Command | Result | Local log |
| --- | --- | --- |
| `cargo test --release -p vara-engine --locked gemini_rejects_incomplete_blocked_missing_and_empty_final_text` before guard | Expected regression failure on empty success | `/tmp/vara-epic42-44-red.log` |
| `cargo test --release -p vara-engine --locked` | 47 passed, 1 ignored (real Redux weights required) | `/tmp/vara-epic42-44-engine.log` |
| `cargo test --release -p vara-core -p vara-app --locked` | 41 passed | `/tmp/vara-epic42-44-callers-final.log` |
| `rustfmt --edition 2024 --check crates/vara-engine/src/remote.rs crates/vara-app/src/lib.rs` and `git diff --check` | Passed | Terminal output |

An initial caller test tried to reopen the profile immediately after dropping its
service and hit `file.profile_busy` in the parallel suite. The assertion now opens
the database directly, which tests persisted content without reacquiring the
unrelated profile lock. Final caller suite is green.

## Explicit issue 48 gap

These tests prove HTTP adapter contracts, file caller failure/cancel preservation,
and existing domain persistence gates separately. They do not connect a Gemini
HTTP failure/inflight cancellation through the native desktop dictation service
into history and OS delivery, nor exercise installed-app UI, paste/copy or real
provider behavior. Issue 48 must verify that end-to-end link; engine errors or
localized “original preserved” wording alone do not prove it. The file caller
cancellation test occurs during credential lookup, not during HTTP; the latter is
covered separately at the actual adapter. Gemini speech remains issue 46.
