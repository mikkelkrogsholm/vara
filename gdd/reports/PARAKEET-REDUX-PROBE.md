# Parakeet Redux — local Rust probe, October 23, 2026

Parakeet Redux was tested locally on this **Apple M3 Max** through the Rust library
[voxrt](https://github.com/lmaksym/voxrt). According to its [official model card](https://huggingface.co/moondream/parakeet-redux),
the model supports Danish. This probe does not change the app's model choice or default.

## Measurement

The same existing `Samples/da_test.wav`: **10.312 seconds of Danish speech**.
Release build, 8 worker threads. CPU and GPU ran sequentially. Median of five
measurements after the first run; model loading is measured separately.

| Run | Model loading | First transcription | Warm median | Audio / computation time |
| --- | ---: | ---: | ---: | ---: |
| CPU, NEON int8 | 1.14 s | 186.6 ms | **181.4 ms** | 57× |
| Apple GPU, Metal | 0.87 s | 59.1 ms | **60.6 ms** | 170× |

The benchmark's GPU model loading includes shader warmup. The input file is
pre-decoded/resampled in the benchmark. The standalone transcription command, which
also reads the WAV file, measured 184 ms on CPU and 61 ms on GPU.

The GPU run used about 0.9 CPU cores during transcription versus about 6.5 on the
CPU run. This measures runtime CPU time, not battery use.

## Actual text

The following Danish transcripts are measured output, preserved verbatim:

GPU:

> Hej med dig, dette er en lille test af en diktoringsværdøj, som jeg er ved at bygge. Det hedder LSTNR og bruger elægen Land skribe til at omdanne tale til tekst.

CPU:

> Hej med dig, dette er en lille test af et toinsværdig, som jeg er ved at bygge. Det hedder LSTNR og bruger elægen Land skribe til at omdanne tale til tekst.

Speed is promising. Recognition still has clear errors and CPU/GPU produce different
words on this sample. There is no independently reviewed reference text, so no WER is
calculated or documented quality improvement over Whisper claimed. The previous
Whisper measurement of 0.41 s included a cold engine/model; it must not be used as
a controlled comparison against warm Redux inference.

## Reproducibility

Runtime commit: `4fab626c00708d9bc97f837d309e0d362094282b`, version 0.2.1.
Model revision: `2bf128600aac4b16946f7ed8372e56117fe5e23b`.
Four model files total **179,005,408 bytes**. Model weights were checked against
Hugging Face SHA-256 and the other files against their Git blob IDs. All local
SHA-256 values and raw results are in the [machine-readable report](parakeet-redux-probe-2026-10-23.json).

Sources and weights are isolated under `target/experiments/`, outside the app bundle.
No audio was uploaded, cloud API used or microphone recorded.

Build from the specified runtime revision:

```sh
cd target/experiments/voxrt
cargo +1.99.1 build --release --locked -p voxrt-cli --no-default-features --features metal
```

From the Vara repository root, with the verified model files in the test directory:

```sh
VOXRT_DEVICE=gpu target/experiments/voxrt/target/release/voxrt transcribe Samples/da_test.wav --model target/experiments/parakeet-redux --threads 8 --format json
VOXRT_DEVICE=gpu target/experiments/voxrt/target/release/voxrt bench Samples/da_test.wav --model target/experiments/parakeet-redux --threads 8 --seconds 10.312125 --repeat 5
```

Repeat with `VOXRT_DEVICE=cpu` for CPU. No Python/ONNX runtime is used for inference;
Python was used only for download verification and report collection.

## Implications for Vara

A good candidate for an additional local speech engine. The Rust library can be
embedded directly; the CLI is only a probe tool. Before it becomes an app option,
connect the model through the same speech contract and installer, ensure cancellation
and test packaging and more Danish samples. Windows/Linux/mobile were not tested here.
Whisper remains the app default. The weights are CC-BY-4.0; the runtime is Apache-2.0.
