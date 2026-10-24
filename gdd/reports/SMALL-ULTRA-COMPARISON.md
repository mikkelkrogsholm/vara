# Whisper Small og Parakeet Ultra — 23. oktober 2026

Begge er kørt lokalt gennem Rust på denne Apple M3 Max, med den samme danske
lydprøve som Redux: `Samples/da_test.wav`, **10,312 sekunder**. Ingen lyd er sendt
ud af maskinen. Appens modelvalg og standard er ikke ændret.

## Resultat

Transskriptionstiden er medianen af fem gentagelser efter første gennemløb.
Indlæsning er målt separat én gang pr. proces og påvirkes af filcache og opvarmning.

| Konfiguration | Varm transskription | Observeret indlæsning | Model-filer |
| --- | ---: | ---: | ---: |
| Whisper Base, CPU | 507 ms | 89 ms | 148 MB |
| **Whisper Small, CPU** | **852 ms** | 199 ms | 488 MB |
| **Parakeet Ultra, ONNX CPU** | **379 ms** | 9,06 s | 2,55 GB |
| Parakeet Redux, CPU | 187 ms | 4,32 s | 179 MB |
| Parakeet Redux, Metal GPU | 60 ms | 884 ms | 179 MB |

Alle CPU-konfigurationer bruger otte tråde. Bygning og download var afsluttet før
de endelige målinger; en tidlig Small-kørsel under kompilering er udeladt.
Redux-indlæsning på CPU tog 1,14 s ved den tidligere prøve og 4,32 s her. En enkelt
indlæsningsmåling er derfor ikke et stabilt opstartsløfte.

Dette sammenligner **de afprøvede konfigurationer**: præcision, runtime og CPU/GPU
er forskellige. Ultra er en float32-ONNX-eksport, ikke en kvantiseret udgave.
Whisper og Redux foruddekoder WAV i benchmarken, mens Ultra-kaldet inkluderer
fildekodning. Det er ikke en kontrolleret rangering af modelarkitekturer alene.

## Faktisk tekst

### Whisper Small

> Hej med dig, det er det er en lille test af Edictoringsværdøj, som jeg er ved at bygge. Det hedder LSTNR og bruger eleden Ladt skrepe til at omdane tale til tekst.

### Parakeet Ultra

> Hej med dig, dette er en lille test af edektoringsværktøj, som jeg er ved at bygge. Det hedder lstnr og bruger Elenen Landskribe til at omdanne tale til tekst.

Begge gav samme tekst i alle seks gennemløb. Der er stadig tydelige problemer i
ordene omkring dikteringsværktøj og tjenestenavnet. Small viser ikke en overbevisende
kvalitetsgevinst på denne ene prøve; Ultra løser heller ikke alle problemerne.
Vi har ingen uafhængigt gennemgået referencetekst og beregner derfor ikke WER eller
udråber en generel kvalitetsvinder. Redux-teksterne fra den forrige prøve findes i
[Redux-rapporten](PARAKEET-REDUX-PROBE.md).

Redux er fortsat den hurtigste af de testede konfigurationer. Ultra er en mulig
kvalitetskandidat, men denne eksport har en stor modelpakke og mærkbar opstart.
Small giver her længere ventetid uden en tydelig tekstforbedring. Flere danske
optagelser er nødvendige før valg af ny standard.

## Kilder og reproduktion

- [Whisper-vægte](https://huggingface.co/ggerganov/whisper.cpp), revision
  `5359861c739e955e79d9a303bcbc70fb988958b1`, `ggml-small.bin`. Fuld model, ikke `.en`.
- [Parakeet Ultra ONNX-eksport](https://huggingface.co/altunenes/parakeet-rs/tree/b53b9101fa87271041773b5036aeb8db9d810d40/parakeet-ultra),
  fire filer, revision `b53b9101fa87271041773b5036aeb8db9d810d40`.
- [Parakeet Rust-runtime](https://github.com/altunenes/parakeet-rs), commit
  `187dbf3e81d869483cf6daa8ee4a97b2aa458835`, version 0.4.3, `ort` 2.0.0-rc.15.
- [Original Ultra-model](https://huggingface.co/moondream/parakeet-ultra).
- [Rå resultater, gentagelser og verificerede SHA-256-værdier](small-ultra-comparison-2026-10-23.json).

Store model-filer er verificeret mod Hugging Faces SHA-256; små filer mod deres
Git blob-ID. Kilder og vægte er isoleret under `target/experiments/`.

Whisper-proben bruger samme greedy best_of=1, dansk, no_context og otte CPU-tråde
som den tidligere lokale prøve. Den nye udviklerprobe kan vælge en modelsti og
holde modellen indlæst mellem gentagelser:

```sh
cargo run --release -p vara-engine --example whisper_bench -- target/experiments/model-candidates/whisper-small/ggml-small.bin Samples/da_test.wav 6
```

Ultra-proben er gemt som `scripts/bench-parakeet-ultra.rs`. Kopiér den til
`examples/vara_ultra_probe.rs` i den pinnede `parakeet-rs`-checkout. Når checkouten
ligger under Vara, tilføjes en tom `[workspace]`-tabel i dens egen Cargo.toml,
så forsøget forbliver uden for Vara-workspacet. Byg med:

```sh
cargo +1.99.1 build --release --locked --example vara_ultra_probe
```

Kør derefter fra Vara-roden:

```sh
target/experiments/parakeet-rs/target/release/examples/vara_ultra_probe target/experiments/model-candidates/parakeet-ultra Samples/da_test.wav
```

Whisper-vægte/runtime er MIT; Ultra-vægte er CC-BY-4.0. Ingen modeller er tilsluttet
appens installer eller valgt som ny standard ved denne prøve. Annullering,
app-pakning, Windows/Linux og mobil er ikke verificeret for disse kandidater.
