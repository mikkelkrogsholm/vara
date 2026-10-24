# Live cloud verification — 2026-10-23

User confirmed credentials existed in the old Vara project. The Groq credential
was read from its existing `.env` and passed only through a subprocess stdin
pipe to the Rust production engine. No credential was printed, copied into
reports or imported into the new OS secret store. The probe is a developer
example and is not included in the desktop bundle.

## Live results after correction

- Groq model catalogue: accepted key; all three configured model IDs present.
- Whisper Large-v3 Turbo: actual transcription PASS, 160 characters, 0.265 s.
- Whisper Large-v3: actual transcription PASS, 158 characters, 0.391 s.
- GPT-OSS 120B: actual text cleanup PASS, 36 characters, 0.361 s.

Speech used the existing `Samples/da_test.wav` recording and Danish language
hint. Text used a short synthetic Danish sentence. Timings are single requests,
not a benchmark or accuracy evaluation. Neither complete transcript nor key is
included in the report. Machine-readable evidence:
[live-cloud-verification-2026-10-23.json](live-cloud-verification-2026-10-23.json).

## Diagnosed and corrected defect

Initial live speech calls failed although catalogue and text succeeded. The
Groq multipart request supplied `response_format=json` twice after the OpenAI
adapter update. Removed the duplicate, retained the shared JSON field and
Groq-specific temperature field. Both live speech models then passed.

The HTTP speech regression test now requires exactly one response-format field.
All 12 HTTP contract tests pass after the fix. This defect was not found by the
previous local test, which checked field presence without field uniqueness.
The release Mac app was rebuilt with the correction.

## OpenAI limitation

No OpenAI credential was available in the process environment, project `.env`,
or the old app's documented `~/.vara/.env` / `~/.config/vara/.env` locations.
A noninteractive read of the old Vara Keychain service timed out after 15 s;
no unlock or permission prompt was requested. Therefore OpenAI live inference
remains untested. This does not establish whether an OpenAI key exists in the
Keychain; it establishes that it was inaccessible to this session.

Native microphone, shortcut, focus and insertion tests still require an unlocked
Mac session. Existing headless UI/core test evidence remains applicable.
