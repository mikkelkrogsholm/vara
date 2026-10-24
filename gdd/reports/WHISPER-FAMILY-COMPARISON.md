# Hele Whisper-serien — 23. oktober 2026

Alle seks aktuelle flersprogede størrelser er prøvet lokalt med samme danske
lydfil på **10,312 sekunder**. Apple M3 Max, 128 GiB RAM. Ingen lyd er uploadet.
Appens modeller og GPU-opsætning er ikke ændret.

## Hastighed

Median af fem gentagelser efter første gennemløb. Modellen holdes indlæst;
hver transskription får en frisk Whisper-state som i den aktuelle app.

| Model | CPU | Metal GPU | Model-filstørrelse |
| --- | ---: | ---: | ---: |
| tiny | 499 ms | 601 ms | 78 MB |
| base | 313 ms | 140 ms | 148 MB |
| small | 841 ms | 303 ms | 488 MB |
| medium | 2205 ms | 862 ms | 1534 MB |
| large-v3 | 4201 ms | 1454 ms | 3095 MB |
| large-v3-turbo | 2691 ms | 662 ms | 1625 MB |

Dette er de konkrete testkonfigurationer på ét klip, ikke garantier om alle
optagelser. Hastighed påvirkes også af den tekst og de tokens, modellen genererer.
Tiny er således langsommere end Base på netop denne prøve.

Base/Small-tiderne fra tidligere rapporter er historiske målinger; tabellen her
viser en samlet ny kørsel med samme build og testopsætning for alle seks modeller.

## Faktiske GPU-transskriptioner

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

## Vurdering

Ingen af modellerne løser alle de svære termer/navne i denne prøve. Large-v3 og
Turbo viser derfor ikke en klar kvalitetsgevinst, som retfærdiggør deres størrelse
ud fra denne prøve alene. Vi har ingen uafhængigt gennemgået referencetekst og
beregner ikke WER. En generel kvalitetsrangering kræver flere danske optagelser.

Metal gør en væsentlig forskel for Base, Small, Medium og Large-modellerne her.
Large-v3 Turbo er hurtigere end både Medium og Large-v3 på GPU i denne test.
Det betyder, at CPU-målingerne alene ikke bør bestemme modelvalg til en Mac.

## Indlæsning og metode

Indlæsning er separat fra transskriptionstiden. Tallene i JSON-rapporten er
enkeltmålinger med fil- og shadercache efter tidligere testkørsler. Første Metal-
smoketest tog cirka 9,16 s at initialisere. Det er ikke inkluderet i tabellens
varme tider og må ikke præsenteres som en garanteret hurtig frisk installation.

Rust-proben bruger whisper-rs 0.16.0 / whisper-rs-sys 0.15.0, greedy best_of=1,
otte tråde, dansk, no_context og de fulde flersprogede modeller uden kvantisering.
WAV-dekodning ligger uden for transskriptionstimeren. Metal er bekræftet i native
backendlogs for samtlige GPU-kørsler; der er ikke blot sat et GPU-flag.

Alle model-filer er hentet fra [whisper.cpp-modelrepositoriet](https://huggingface.co/ggerganov/whisper.cpp)
på revision `5359861c739e955e79d9a303bcbc70fb988958b1` og SHA-256-verificeret.
De engelske `.en`-varianter og ældre Large-v1/v2 er ikke med i denne danske test.

[Rå data, tekster, indlæsningstider og kontrolsummer](whisper-family-comparison-2026-10-23.json).

## Gentag testen

Med de allerede verificerede model-filer under `target/experiments/`:

```sh
cargo build --release -p vara-engine --example whisper_bench --features whisper-rs/metal
python3 scripts/compare-whisper.py
```

Brug `--cpu-only` til kun CPU. Scriptet kræver alle lokale model-filer og afslår
at rapportere en GPU-kørsel, hvis native logs ikke bekræfter Metal-backend.
Denne udviklerprobe ændrer ikke appens modelvalg eller aktiverer Metal i appen.
