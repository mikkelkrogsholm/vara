# Vara agent CLI

Install Vara in `/Applications`, then install the bundled launcher for your user:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 /Applications/Vara.app/Contents/Resources/vara "$HOME/.local/bin/vara"
"$HOME/.local/bin/vara" --version
```

Use the absolute path above if `~/.local/bin` is not on your PATH. The installer
changes no shell profile and requires no administrator access. The launcher calls
the signed app binary in `/Applications/Vara.app`; app updates therefore use the
same executable for the app and CLI. If the app is moved or missing, restore it
to `/Applications`. An older app without CLI support returns `cli.app_incompatible`;
a missing or moved app returns `cli.app_missing`, both as JSON with exit 5. Reinstall the launcher after updating Vara to pick up any
future launcher changes. On macOS it starts or connects to the same local backend
and profile as the application. An incompatible older backend is reported as
`backend.version_mismatch`; it and active old jobs are not stopped automatically.
Use matching application and CLI versions after the old work has finished.

For development only, the equivalent internal entrypoint is `vara-desktop cli`.

```sh
vara models --json
vara transcribe "interview æ.wav" --diarize --speakers 2 --no-wait --request-id interview-1
vara job status JOB_ID
vara job wait JOB_ID --timeout 120
vara job result JOB_ID --format srt --original --output interview.srt
vara job cancel JOB_ID
```

Default transcribe waits without a deadline. The configured speech engine and
automatic language are the defaults. `--model ID` and `--language CODE` apply
only to the job; Redux detects language and rejects an explicit override. No
model is downloaded. `--diarize` requests the existing local Who runtime after ASR; `--speakers 1..16` optionally supplies a known count and requires `--diarize`. Who must already be installed through Vara after reviewing its license; no download or acceptance is performed by the CLI. `models` reports cloud readiness as `null` (not checked),
not a promise that a credential or provider is available.

Each submit prints `request_id` before contacting the backend and `job_id` as
soon as accepted, both on stderr. An automatically generated request ID is new
per invocation. If a submit times out or is interrupted before its receipt,
retry identical input/options with the printed `--request-id`; acceptance is
idempotent even if the configured default changes. `--no-wait` returns an
`accepted` JSON receipt; query the job for its current state. `--no-wait --output` (including `--overwrite`) is an input error; use
`job result --output` after completion. `--no-wait --format` is allowed and
returns the JSON job receipt. `job wait` timeout,
Ctrl-C and closed stdout stop only the client. Use `job cancel` to cancel work.

JSON is versioned with `schema_version: 1`; job and document IDs are the same.
Completed JSON includes the shared document, its actual timing, timing source,
segments and any real words. `speaker_transcript` is the same core-derived annotation
used by the UI; raw ASR and Who intervals remain separate. Uncertain or overlapping
intervals do not invent a speaker or word times. `selected_text` identifies the
exported text and `original_selected` records the choice. `--original` on transcribe,
job wait or job result selects original ASR text and timing; the default uses the
current saved edit. Unsaved GUI drafts are unavailable. SRT/VTT require real timing
and, after an edit, require `--original` until timing supports the edited text. `timing_capability` describes the adapter's maximum
or requested capability, not a guarantee for a specific transcript. `--format
json|txt|md|srt|vtt` uses the shared exporter. With a non-JSON format and no
`--output`, stdout contains only exported content. `--output FILE` atomically
publishes the export and returns a JSON receipt; `--overwrite` is required for an
existing file. All sources known to the shared document owner, profile files and linked
protected targets are rejected. Relative output paths use the client working
directory. An export error leaves the completed job available under its ID.

A diarized job remains processing after ASR becomes ready until Who finishes.
The status receipt includes `asr_stage` and `who`; Who failure or cancellation
returns a nonzero error with `who_failed` or `who_cancelled` and the job ID.
ASR text and audio remain accessible in Vara. A backend interruption marks unfinished
Who failed with `file.interrupted`; it does not silently rerun inference. An explicit
retry in Vara restarts the required phase and updates its current outcome. If the
Who outcome cannot be saved, the active owner reports `file.result_unsaved`; the
ASR stays available and a restart reports the unfinished phase as interrupted.

A status query for a failed or cancelled job returns the corresponding nonzero
error with its job ID and terminal status. JSON errors contain stable `error.code` and optional catalogued numeric causes;
no key, endpoint or provider body is returned. Exit codes and next actions:

| Exit | Meaning | Next action |
| --- | --- | --- |
| 0 | Success or accepted receipt | Inspect status or result. |
| 2 | Invalid arguments, input or request conflict | Correct input; use a new request ID for changed options. |
| 3 | Model, credential, runtime or privacy restriction | Configure the model/key or choose a permitted available engine in Vara. |
| 4 | Job failed or was cancelled | Inspect the safe error and submit a new job if appropriate. |
| 5 | Backend transport or version | Check the local profile and compatible application executable. |
| 6 | Export failed | Choose a safe output path or supported format; the job remains available. |
| 7 | Timeout or interruption | Retry using the printed request ID or inspect the printed job ID. |

`--help` and `--version` use human-readable output. Other JSON operations emit
one JSON object on stdout; progress and diagnostics use stderr. A closed pipe
cannot receive an error object. No API keys are accepted in arguments.

Common stable codes include `cli.invalid_arguments`, `cli.timeout`,
`cli.interrupted`, `cli.output_closed`, `file.source_missing`,
`file.request_conflict`, `file.language_unsupported`, `file.model_download`,
`file.privacy_only`, `provider.missing_key`, `file.export_source`,
`file.export_overwrite`, `file.export_save`, `transcription.subtitle_timing`,
and `backend.version_mismatch`. Other classified backend and job failures retain
their catalog code and safe numeric causes. When a wait times out after a job ID
is known, the error contains that ID; if submit acceptance is unknown, it contains
the request ID instead.
