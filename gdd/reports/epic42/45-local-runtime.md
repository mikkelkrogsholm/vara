# Issue 45 — actual local runtime acceptance

Run: 2026-11-06, macOS 26.6.2 / arm64, Apple M3 Max, release profile.
**Runtime contracts passed after one cancellation fix; no-speech quality acceptance failed.**
No user listening, microphone, insertion, personal settings/history/keys, cloud inference,
Ollama requests, model replacement or model tuning occurred.

## Results

`cargo test --release -p vara-engine`: **50 passed, 1 explicitly ignored real-weight test**
(34 unit, 5 media, 10 integrity, 1 runtime-message; no doc tests), exit 0. Normal tests do not
install/download weights. Real test: **0 passed, 1 failed in 11.42 s**, intentionally
failing its final quality assertion after all runtime scenarios had completed, exit 101.
Machine-readable observations and exact artifact hashes: [45-local-runtime.jsonl](45-local-runtime.jsonl).

| Engine | Cold smoke | Warm timed file | Cancellation, including 100 ms delay | Cached next request |
|---|---:|---:|---:|---:|
| Parakeet Redux | 1.029 s | 0.102 s | 0.128 s | 0.103 s |
| Whisper Base | 0.347 s | 0.431 s | 0.308 s | 0.266 s |
| Embedded Qwen3-0.6B | 1.636 s | n/a | 2.335 s | 0.885 s |

Cold means a new `Engines` instance, including model loading, with OS disk caches
uncontrolled. Warm file inference uses the same loaded instance. Timings are observations,
not thresholds. Cached requests succeeded after removing only disposable symlinks to
model files, proving runtime reuse without reloading; their results exactly matched the
first request after accepted in-flight cancellation. In-flight tests assert the request
is unfinished after dispatch and 100 ms, then cancel and await termination.

Parakeet used voxrt 0.2.1 revision `4fab626c00708d9bc97f837d309e0d362094282b`, Int8,
Metal GPU encoder (confirmed by `VOXRT_PROFILE=1` GPU encoder diagnostics).
Whisper used whisper-rs 0.16.0 / whisper-rs-sys 0.15.0, CPU (`use_gpu(false)`), at most
8 threads. Embedded Qwen used Candle 0.11.0 quantized Qwen3 on explicit CPU. The real
Parakeet output was passed into embedded Qwen through `Engines::text`; no Ollama service
was involved. Qwen cancellation waits for the current forward pass: this fixture took
about 2.2 s after cancellation request, which is observable latency, not immediate abort.

The speech fixture is `Samples/acceptance/audio/da-000.wav` (12.84 s). Actual timed
output passed `Transcript::validate(duration)`: Parakeet declared word timing, 19 words,
one segment ending at 12.88 s (within the contract's 0.1 s tolerance); Whisper declared
segment timing, one segment ending at 10 s and no words. This validates bounds and
capability, not linguistic alignment/accuracy. Whisper's Danish output was visibly
inaccurate; corpus scoring belongs to issue 47. Dataset text remains provisional,
without a human-review gate for runtime testing.

## Failing acceptance and diagnosed fix

Parakeet returned `engine.no_speech` for both three-second silence and seeded noise.
Whisper returned successful phantom dictation:

- Silence: `Det er ikke en del af det, vi kan ikke tage den.`
- Noise: `(tronk)`

Both inputs are deterministic mono 16 kHz PCM16 corpus files, three seconds each:
`Samples/acceptance/audio/silence.wav` SHA256
`d303811b8c84619667cd0501342f84ec6cbe69f7aa3856dcf52fabda374c92b8`, and
`Samples/acceptance/audio/noise.wav` SHA256
`5ed27471871a206c34879b18b184cec8e07a5db6811a2ffa0a484952cdfa2445`.

The ignored real test explicitly fails on these successes; it does not normalize or
suppress them. **No-speech acceptance is unsatisfied for Whisper.** The probe never
persists/inserts anything; it cannot establish host-level prevention of phantom
persistence/insertion. Nonempty API success is exposed to callers and requires follow-up
before claiming this behavior safe. No inference tuning was made in this issue.

The first real run exposed a separate runtime defect: Whisper's native abort returned
`whisper.transcription` with native code -6 before the cancellation check ran. The narrow
fix checks accepted cancellation immediately after `state.full` returns and before
mapping its error. The second real run verified `runtime.cancelled`, then successful
identical next-session output. Noncancelled native errors retain their existing mapping.

Normal integrity tests additionally exercise absent/corrupt files for all three local
engines (explicit local errors, no provider switching), same-size hash corruption,
separate tokenizer namespaces, cancellation/publication ordering, and interrupted `.part`
cleanup while preserving existing final files. A real pinned Whisper download was also
cancelled after 689,969 bytes: it returned `runtime.cancelled` and left neither final nor
partial file. This live download cancellation was outside the network-deny inference run.

## Reproduction and evidence

Build environment used throughout:

```sh
export PATH=/Users/mikkelfreltoftkrogsholm/.cargo/bin:$PATH
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
export CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_BUILD_JOBS=4
cargo test --release -p vara-engine
cargo run --release -p vara-engine --example install_whisper_acceptance -- /tmp/vara-epic42-45-whisper-models
cargo build --release -p vara-engine --example install_whisper_acceptance
# A separate Python subprocess.run(..., timeout=90) bounded this invocation:
target/release/examples/install_whisper_acceptance /tmp/vara-epic42-45-interrupted-models --cancel-after-progress
```

The model installer refuses an existing destination. Never pass the personal model root.
Installed read-only Parakeet and Qwen artifacts came from the existing verified directory;
Whisper Base was installed only in the disposable directory above, with catalogue SHA256
`60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe` (147,951,465 bytes).
All seven artifact sizes/hashes were verified before real inference; exact hashes are in
the JSONL (4 Redux artifacts, Whisper Base, Qwen GGUF, Qwen tokenizer).

After building, the real test executable was invoked directly, avoiding Cargo's own
network activity, from a Python `subprocess.run(..., timeout=480)` controller:

```sh
VARA_TEST_MODELS="$HOME/Library/Application Support/dk.vara.desktop/models" \
VARA_TEST_WHISPER_MODELS=/tmp/vara-epic42-45-whisper-models VOXRT_PROFILE=1 \
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  target/release/deps/redux_runtime-abb83bdf3649ad02 --ignored --nocapture --test-threads=1
```

Executable hash suffix is build-specific; use the `redux_runtime` executable emitted by
`cargo test --release -p vara-engine --test redux_runtime --no-run`. The sandbox applies
only to this process and descendants. A Python socket control using the identical profile
confirmed both loopback `bind` and `connect` fail with errno 1 / EPERM. Successful local
inference within that profile proves it needs no provider network access, including
localhost Ollama. Global networking/security/preferences were unchanged.

Logs: `/tmp/vara-epic42-45-build.log`, `-normal.log`, `-install.log`, `-install-cancel.log`,
`-network-control.log`, `-real-before-fix.log`, `-real.log` (all share the prefix).
All owned processes are terminal at handoff. Disposable test roots are RAII-cleaned and
the empty interrupted-download directory was removed. The verified Whisper directory is
retained by explicit MAIN instruction for issue 47; its cleanup ownership transfers to
MAIN. Compiler intermediates remain in the serial epic's existing target cache until
MAIN's final cleanup. No cache enrollment, copying or unrelated cleanup was performed.
