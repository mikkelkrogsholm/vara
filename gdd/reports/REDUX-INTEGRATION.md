# Parakeet Redux integreret — 23. oktober 2026

Brugeren godkendte Redux som standard. Nye profiler bruger nu `local.parakeet`;
Whisper Base er et valgfrit alternativ. Eksisterende gemte valg bevares, mens
Redux tilføjes som en tilgængelig indbygget motor.

## Ændringen

- Fælles Rust-adapter via voxrt, låst til commit
  `4fab626c00708d9bc97f837d309e0d362094282b` (0.2.1).
- Metal på understøttede Apple-builds og CPU-fallback. Otte arbejdstråde, cached
  model og selvstændig decoder/VAD-session pr. diktering.
- Automatisk sproggenkendelse. UI forklarer det og deaktiverer det manuelle
  sprogvalg, når Redux er valgt.
- Fire filer, **179.005.408 bytes**, pinned revision og SHA-256-kontrol. Egen mappe
  `models/parakeet-redux/`, så Qwen-tokenizeren ikke overskrives.
- Onboarding, modeloversigt og klarstatus følger den valgte talemodel. Installeret
  Whisper skjuler ikke en manglende Redux-model eller omvendt.
- Lokal tale/tekst og CLI-auth læser ikke OS-lageret for API-nøgler.
- Kooperativ annullering før/efter indlæsning, mellem inputblokke og før/efter
  afslutning. Igangværende kernels afsluttes før backend træder ud. Hostens
  generationstjek/tombstone afviser sene resultater efter accepteret annullering.
- Backend-panik bliver en almindelig genkendelsesfejl med mulighed for genprøvning.
- Model- og runtime-attribution er inkluderet i appens `Resources/notices/`.

De eksisterende regler for historik, AI, fokus, udklipsholder og lydenes 24-timers
fejlfrist anvendes også til Redux. Ingen automatisk overgang til en cloud-motor.

## Faktisk verificeret

1. **Rigtig installering:** `local_probe` hentede og verificerede alle fire filer
   i en isoleret testprofil gennem appens egen installer. Derefter genkendte den
   `Samples/da_test.wav` gennem den nye produktionsadapter. Kold modelindlæsning
   plus transskription tog cirka 1,23 s på denne allerede shaderopvarmede Mac.
2. **Rust-kontrakter:** 17 almindelige tests består, inklusive profilopgradering,
   annulleret Redux-installering, manglende-model-fejl uden providerskift og
   separate tokenizer-mapper.
3. **Rigtige modelvægte:** separat opt-in runtime-test består. Den genkender den
   korte prøve, accepterer annullering af en lang gentaget prøve og genkender
   derefter originalprøven igen med samme cached model og identisk tekst.
4. **UI:** 11 browser-testgrupper består med produktions-HTML/CSS og isoleret IPC-
   fixture. Frisk profil viser Redux; valgfri Whisper og klarstatus er kontrolleret.
5. **Mac-bundle:** normal release-build uden `test-harness`, **24,77 MiB**, bygget.
   Model-notices er verificeret i bundlen. GDD-model og grafvalidering består.

## Begrænsninger

Mac'en blev låst under arbejdet. Brugeren valgte udtrykkeligt at fortsætte uden
native UI-kontrol. Der er derfor **ikke** gennemført en ny native Redux-session
eller mikrofon→tekstfelt-test i denne ændring. Tidligere native Whisper-kontrol er
historisk evidens og påstås ikke at være Redux-verifikation.

UI-fixtures beviser ikke OS-adfærd. Rigtig mikrofon, fysisk global genvej,
Windows/Linux og mobil mangler fortsat slutverificering. Windows/Linux native
fokus- og HUD-adaptere er stadig uafsluttede. Dansk modelkvalitet er ikke endeligt
accepteret på grundlag af én lydprøve.

## Gentag

```sh
cargo test --workspace
cargo run --release -p vara-engine --example local_probe -- target/redux-acceptance/models Samples/da_test.wav
VARA_TEST_MODELS="$PWD/target/redux-acceptance/models" cargo test --release -p vara-engine --test redux_runtime -- --ignored
cargo tauri build --bundles app
```

Release: `target/release/bundle/macos/Vara.app`. Genstart en allerede kørende app
for at indlæse den nye build. Ingen testhistorik eller testmodeller er lagt i den
almindelige brugerprofil. Se [maskinlæsbar evidens](redux-integration-2026-10-23.json).
