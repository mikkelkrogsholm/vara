# Whisper family — October 23, 2026

All six current multilingual sizes were tested locally on the same **10.312-second**
Danish audio file. Apple M3 Max, 128 GiB RAM. No audio was uploaded. The app's models
and GPU configuration were not changed.

## Speed

Median of five repetitions after the first run. The model remains loaded; each
transcription gets a fresh Whisper state as in the current app.

| Model | CPU | Metal GPU | Model file size |
| --- | ---: | ---: | ---: |
| tiny | 499 ms | 601 ms | 78 MB |
| base | 313 ms | 140 ms | 148 MB |
| small | 841 ms | 303 ms | 488 MB |
| medium | 2205 ms | 862 ms | 1534 MB |
| large-v3 | 4201 ms | 1454 ms | 3095 MB |
| large-v3-turbo | 2691 ms | 662 ms | 1625 MB |

These are specific test configurations on one clip, not guarantees for all recordings.
Speed is also affected by the text and tokens the model generates. Tiny is therefore
slower than Base on this particular sample.

Base/Small times from earlier reports are historical measurements; this table shows
a new combined run with the same build and test setup for all six models.

## Actual GPU transcripts

The measured Danish output is preserved verbatim:

### tiny

> Hvad har jeg med dig, det er en lille test af et et tøngsverter, som jeg er ved at bygge. Det hedder LST-in er opråder eller en lætteskribet til at omdannet tale tidigt.

### base

> Hej med dig, det er en lille test af et dett tøjens hvertag, som jeg er ved at bygge. Det hedder LSTNR og bruger i linjen land, Skribe til at omdane tale til et tekst.

### small

> Hej med dig, det er det er en lille test af Edictoringsværdøj, som jeg er ved at bygge. Det hedder LSTNR og bruger eleden Ladt skrepe til at omdane tale til tekst.

### medium

> Hej med dig. Dette er en lille test af et diktøjens værktøj, som jeg er ved at bygge. Det hedder "LSTNR" og bruger "Elen Lad" skribe til at omdanne tale til tekst.

### large-v3

> Hej med dig, dette er en lille test af ediktørens værktøj, som jeg er ved at bygge. Det hedder LSTNR og bruger elenen lad skribe til at omdanne tale til tekst.

### large-v3-turbo

> Hej med dig. Dette er en lille test af eddiktøjens hverdøj, som jeg er ved at bygge. Det hedder LSTNR og bruger L.E.N. Lad skrive til at omdanne tale til tekst.

## Assessment

None of the models resolves all the difficult terms/names in this sample. Large-v3
and Turbo therefore show no clear quality gain that justifies their size based on
this sample alone. There is no independently reviewed reference text and no WER is
calculated. A general quality ranking requires more Danish recordings.

Metal makes a substantial difference for Base, Small, Medium and Large here.
Large-v3 Turbo is faster than both Medium and Large-v3 on GPU in this test. CPU
measurements alone should therefore not determine model choice for a Mac.

## Loading and method

Loading is separate from transcription time. The JSON report numbers are single
measurements with file and shader caches after earlier test runs. The first Metal
smoke test took about 9.16 s to initialize. That is not included in the table's warm
times and must not be presented as a guaranteed fast fresh installation.

The Rust probe uses whisper-rs 0.16.0 / whisper-rs-sys 0.15.0, greedy best_of=1,
eight threads, Danish, no_context and full multilingual models without quantization.
WAV decoding is outside the transcription timer. Metal was confirmed in native
backend logs for all GPU runs; a GPU flag was not the only evidence.

All model files were obtained from the [whisper.cpp model repository](https://huggingface.co/ggerganov/whisper.cpp)
at revision `5359861c739e955e79d9a303bcbc70fb988958b1` and SHA-256 verified.
English `.en` variants and older Large-v1/v2 are not included in this Danish test.

[Raw data, transcripts, loading times and checksums](whisper-family-comparison-2026-10-23.json).

## Repeat the test

With the already verified model files under `target/experiments/`:

```sh
cargo build --release -p vara-engine --example whisper_bench --features whisper-rs/metal
python3 scripts/compare-whisper.py
```

Use `--cpu-only` for CPU only. The script requires all local model files and refuses
to report a GPU run unless native logs confirm the Metal backend. This developer
probe does not change the app's model choice or enable Metal in the app.
