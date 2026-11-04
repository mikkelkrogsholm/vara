# Whisper Small and Parakeet Ultra — October 23, 2026

Both ran locally through Rust on this Apple M3 Max with the same Danish audio sample
as Redux: `Samples/da_test.wav`, **10.312 seconds**. No audio left the machine. The
app's model choices and default were not changed.

## Results

Transcription time is the median of five repetitions after the first run. Loading
was measured separately once per process and is affected by file cache and warmup.

| Configuration | Warm transcription | Observed loading | Model files |
| --- | ---: | ---: | ---: |
| Whisper Base, CPU | 507 ms | 89 ms | 148 MB |
| **Whisper Small, CPU** | **852 ms** | 199 ms | 488 MB |
| **Parakeet Ultra, ONNX CPU** | **379 ms** | 9.06 s | 2.55 GB |
| Parakeet Redux, CPU | 187 ms | 4.32 s | 179 MB |
| Parakeet Redux, Metal GPU | 60 ms | 884 ms | 179 MB |

All CPU configurations use eight threads. Building and downloads were complete before
the final measurements; an early Small run during compilation is excluded. Redux CPU
loading took 1.14 s in the earlier probe and 4.32 s here. A single loading measurement
is therefore not a stable startup promise.

This compares **the tested configurations**: precision, runtime and CPU/GPU differ.
Ultra is a float32 ONNX export, not a quantized version. Whisper and Redux predecode
WAV in the benchmark, while the Ultra call includes file decoding. This is not a
controlled ranking of model architectures alone.

## Actual text

The measured Danish transcripts are preserved verbatim:

### Whisper Small

> Hej med dig, det er det er en lille test af Edictoringsværdøj, som jeg er ved at bygge. Det hedder LSTNR og bruger eleden Ladt skrepe til at omdane tale til tekst.

### Parakeet Ultra

> Hej med dig, dette er en lille test af edektoringsværktøj, som jeg er ved at bygge. Det hedder lstnr og bruger Elenen Landskribe til at omdanne tale til tekst.

Both produced the same text in all six runs. There are still clear problems around
the words for dictation tool and the service name. Small shows no convincing quality
gain on this one sample; Ultra does not solve every problem either. There is no
independently reviewed reference text, so no WER is calculated and no general quality
winner declared. Redux transcripts from the previous probe are in the [Redux report](PARAKEET-REDUX-PROBE.md).

Redux remains the fastest tested configuration. Ultra is a potential quality
candidate, but this export has a large model package and noticeable startup. Small
has longer latency here without a clear text improvement. More Danish recordings are
needed before choosing a new default.

## Sources and reproduction

- [Whisper weights](https://huggingface.co/ggerganov/whisper.cpp), revision
  `5359861c739e955e79d9a303bcbc70fb988958b1`, `ggml-small.bin`. Full model, not `.en`.
- [Parakeet Ultra ONNX export](https://huggingface.co/altunenes/parakeet-rs/tree/b53b9101fa87271041773b5036aeb8db9d810d40/parakeet-ultra),
  four files, revision `b53b9101fa87271041773b5036aeb8db9d810d40`.
- [Parakeet Rust runtime](https://github.com/altunenes/parakeet-rs), commit
  `187dbf3e81d869483cf6daa8ee4a97b2aa458835`, version 0.4.3, `ort` 2.0.0-rc.15.
- [Original Ultra model](https://huggingface.co/moondream/parakeet-ultra).
- [Raw results, repetitions and verified SHA-256 values](small-ultra-comparison-2026-10-23.json).

Large model files were verified against Hugging Face SHA-256; small files against
their Git blob IDs. Sources and weights are isolated under `target/experiments/`.

The Whisper probe uses the same greedy best_of=1, Danish, no_context and eight CPU
threads as the previous local probe. The new developer probe can select a model path
and keep the model loaded between repetitions:

```sh
cargo run --release -p vara-engine --example whisper_bench -- target/experiments/model-candidates/whisper-small/ggml-small.bin Samples/da_test.wav 6
```

The Ultra probe is saved as `scripts/bench-parakeet-ultra.rs`. Copy it to
`examples/vara_ultra_probe.rs` in the pinned `parakeet-rs` checkout. When that
checkout is under Vara, add an empty `[workspace]` table to its own Cargo.toml so
the experiment stays outside the Vara workspace. Build with:

```sh
cargo +1.99.1 build --release --locked --example vara_ultra_probe
```

Then run from the Vara root:

```sh
target/experiments/parakeet-rs/target/release/examples/vara_ultra_probe target/experiments/model-candidates/parakeet-ultra Samples/da_test.wav
```

Whisper weights/runtime are MIT; Ultra weights are CC-BY-4.0. No models were connected
to the app installer or selected as a new default in this probe. Cancellation, app
packaging, Windows/Linux and mobile were not verified for these candidates.
