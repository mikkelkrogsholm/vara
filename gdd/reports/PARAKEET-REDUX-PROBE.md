# Parakeet Redux — lokal Rust-prøve, 23. oktober 2026

Parakeet Redux er afprøvet lokalt på denne **Apple M3 Max** gennem Rust-biblioteket
[voxrt](https://github.com/lmaksym/voxrt). Modellen understøtter dansk ifølge sit
[officielle modelkort](https://huggingface.co/moondream/parakeet-redux).
Prøven ændrer ikke appens modelvalg eller standard.

## Måling

Samme eksisterende `Samples/da_test.wav`: **10,312 sekunders dansk tale**.
Release-build, 8 worker-tråde. CPU og GPU kørt efter hinanden. Median af fem
målinger efter første gennemløb; modelindlæsning er opgjort separat.

| Kørsel | Modelindlæsning | Første transskription | Varm median | Audio / beregningstid |
| --- | ---: | ---: | ---: | ---: |
| CPU, NEON int8 | 1,14 s | 186,6 ms | **181,4 ms** | 57× |
| Apple GPU, Metal | 0,87 s | 59,1 ms | **60,6 ms** | 170× |

Benchmarkens modelindlæsning på GPU inkluderer shaderopvarmning. Inputfilen er
foruddekodet/resamplet i benchmarken. Den selvstændige transskriptionskommando,
der også læser WAV-filen, målte 184 ms på CPU og 61 ms på GPU.

GPU-kørslen brugte cirka 0,9 CPU-kerne under transskription mod cirka 6,5 på
CPU-kørslen. Dette er runtime-måling af CPU-tid, ikke en batterimåling.

## Faktisk tekst

GPU:

> Hej med dig, dette er en lille test af en diktoringsværdøj, som jeg er ved at bygge. Det hedder LSTNR og bruger elægen Land skribe til at omdanne tale til tekst.

CPU:

> Hej med dig, dette er en lille test af et toinsværdig, som jeg er ved at bygge. Det hedder LSTNR og bruger elægen Land skribe til at omdanne tale til tekst.

Hastigheden er lovende. Genkendelsen har stadig tydelige fejl og CPU/GPU giver
forskellige ord på prøven. Vi har ikke en uafhængigt gennemgået referencetekst og
beregner derfor ingen WER eller påstår en dokumenteret kvalitetsforbedring over
Whisper. Den tidligere Whisper-måling på 0,41 s inkluderede kold engine/model;
den må ikke bruges som kontrolleret sammenligning mod varm Redux-inference.

## Reproducerbarhed

Runtime-commit: `4fab626c00708d9bc97f837d309e0d362094282b`, version 0.2.1.
Model-revision: `2bf128600aac4b16946f7ed8372e56117fe5e23b`.
Fire model-filer fylder tilsammen **179.005.408 bytes**. Modelvægtene er kontrolleret
mod Hugging Faces SHA-256 og de øvrige filer mod deres Git blob-ID. Alle lokale
SHA-256-værdier samt rå resultater findes i
[maskinlæsbar rapport](parakeet-redux-probe-2026-10-23.json).

Kilder og vægte ligger isoleret under `target/experiments/`, uden for app-bundlen.
Ingen lyd blev uploadet, ingen cloud-API anvendt og ingen mikrofon optaget.

Byg fra den angivne runtime-revision:

```sh
cd target/experiments/voxrt
cargo +1.99.1 build --release --locked -p voxrt-cli --no-default-features --features metal
```

Fra Vara-repositoriets rod, med de verificerede model-filer i testmappen:

```sh
VOXRT_DEVICE=gpu target/experiments/voxrt/target/release/voxrt transcribe Samples/da_test.wav --model target/experiments/parakeet-redux --threads 8 --format json
VOXRT_DEVICE=gpu target/experiments/voxrt/target/release/voxrt bench Samples/da_test.wav --model target/experiments/parakeet-redux --threads 8 --seconds 10.312125 --repeat 5
```

Gentag med `VOXRT_DEVICE=cpu` for CPU. Ingen Python/ONNX-runtime bruges til
inference; Python er kun brugt til downloadkontrol og opsamling af rapporten.

## Betydning for Vara

God kandidat til en ekstra lokal talemotor. Rust-biblioteket kan indlejres direkte;
CLI'en er alene prøveværktøj. Før den bliver et appvalg, skal vi tilslutte modellen
via samme speech-kontrakt og installer, sikre annullering og teste pakning samt
flere danske prøver. Windows/Linux/mobil er ikke testet her. Whisper er fortsat
appens standard. Vægtene er CC-BY-4.0; runtime er Apache-2.0.
