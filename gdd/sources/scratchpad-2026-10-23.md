# Vara — scratchpad

Levende arbejdsnoter. Her samler vi idéer, spørgsmål og mulige ændringer, før de bliver til konkrete opgaver.

## 2026-10-23 — Ny arbejdsretning: clean-write

Brugerens forslag: Beskriv først, hvad Vara skal kunne, og byg derefter fra bunden ud fra den bedste løsning. Den eksisterende implementation skal ikke sætte formen for den nye.

Dette er den aktuelle arbejdsretning for udforskningen. Noterne nedenfor om Rust, Tauri, arkitektur og migrationsrækkefølge er tidligere forslag og kandidater, ikke bindende valg. Ønsket om crossplatform består; præcist platformsscope og teknologistak skal afklares gennem kravene.

### Principper

- Start med brugere, brugssituationer og ønskede resultater. Beskriv derefter de nødvendige funktioner.
- Ingen automatisk videreførelse af eksisterende skærme, modes, motorer, indstillinger eller moduler. Hver del skal have en begrundelse i det nye produkt.
- Brug den eksisterende app som kilde til erfaringer, konkrete behov og fejlscenarier. Undgå at gøre dens struktur til specifikationen.
- Bevar værdifuld viden om fx annullering, tab af tale, fokus og privatliv som adfærdskrav. Vælg frit, hvordan de opfyldes.
- Adskil krav fra løsningsforslag: “brugeren kan diktere uden at sende lyd væk” er et produktkrav; en bestemt Whisper-binding er en mulig løsning.
- Vælg arkitektur, sprog, UI og biblioteker efter den ønskede adfærd og afprøvning af de vigtigste tekniske usikkerheder.
- Clean-write betyder frihed til at vælge løsningen. Dokumenteret egnede biblioteker og standarder kan stadig genbruges.
- Bevar den nuværende app og lokale ændringer under udforskningen. Eventuel dataimport er et separat overgangskrav og må ikke bestemme den nye interne struktur.

### Næste konkrete leverance: en produktbeskrivelse

Skriv en kort, selvstændig beskrivelse, som kan forstås uden at læse Swift-koden:

1. Hvem Vara er til, i hvilke situationer, og hvilket problem den løser.
2. De vigtigste brugerforløb med konkrete eksempler på input og forventet resultat.
3. Funktioner opdelt i nødvendige for første udgave, senere muligheder og åbne idéer.
4. Krav til kvalitet: betydningsbevarelse, sprog, svartid, pålidelighed, privatliv, tilgængelighed og platforme.
5. Adfærd ved fejl, annullering, manglende forbindelse, ændret fokus og genstart.
6. Tydelige fravalg og spørgsmål, der stadig kræver en beslutning.

For hver foreslået funktion: beskriv behovet, brugerens handling, det synlige resultat og hvordan vi kan afgøre, om det fungerer. Mål skal fastlægges; de skal ikke opfindes som om de allerede var aftalt.

Afprøv derefter de mest risikable antagelser med små prototyper og vælg teknik på baggrund af resultaterne. Ingen fuld implementation er startet eller bestilt med dette notat.

## 2026-10-18 — Rust og crossplatform

### Ønske

Undersøge en omskrivning af Vara til Rust, så appen kan køre på macOS, Windows og Linux. Brugeren ønsker dette skrevet ned som udgangspunkt for flere overvejelser. Implementering og endelig teknologistak er endnu ikke aftalt.

### Foreløbig retning

- Fælles Rust-kerne til dikteringssessioner, transskription, AI-bearbejdning, modes, historik og indstillinger.
- Tauri som kandidat til desktop-appen, med HTML/CSS/TypeScript til brugerfladen. Det er en anbefaling, ikke en beslutning om frontend-framework.
- Et separat CLI, som bruger samme kerne.
- Små platformsspecifikke adaptere til funktioner, der kræver integration med operativsystemet.
- Byg den nye version ved siden af Swift-appen. Bevar eksisterende funktionalitet og lokale ændringer, mens den nye løsning afprøves.

### Nuværende funktioner, der skal tages stilling til

| Område | Mulig vej | Afklaring / arbejde |
|---|---|---|
| Push-to-talk og menu-start/stop | Fælles sessionstilstand i Rust | Bevar hurtig respons, korrekt tast-slip og annullering |
| Mikrofonoptagelse | Fx CPAL | Mikrofonvalg, enhedsskift, sample rate, tilladelser og lydniveauer |
| Cloudtransskription | Rust-klienter til eksisterende tjenester | Batch, streaming, fejl, timeouts og fallback |
| Lokal Whisper | Fx whisper.cpp via Rust-bindinger | Modeldownload, hardwareacceleration, dansk kvalitet, hastighed og pakning |
| Local Hviske | Mulig fortsat subprocess-integration | Installation og understøttelse pr. platform skal undersøges |
| Apple Speech | Valgfri macOS-adapter | Ingen fælles erstatning besluttet |
| Apple Foundation Models | Valgfri macOS-adapter | Fælles AI-alternativer er allerede API, Ollama og CLI |
| AI-bearbejdning og modes | Fælles Rust-logik | Raw, Clean text, Professional, VibeCode, Ask AI og egne modes |
| AI via CLI | Rust-subprocesser | Binærdetektion, login, miljøvariabler, timeout og afbrydelse pr. OS |
| Globale genveje | Tauri-plugin og evt. native adaptere | Modifier-taster alene, venstre/højre tast, hold/slip og konflikter |
| Tekstindsættelse | Platformsspecifik adapter | Aktiv app, fokus, clipboard, tastaturlayout og særlige tekstfelter |
| App-regler | Platformsspecifik app-identifikation | macOS bundle-id skal have en tilsvarende model på Windows/Linux |
| HUD | Tauri-vindue og evt. native tilpasning | Fokus, placering ved markør, flere skærme, skalering og fuldskærm |
| Dashboard, settings og onboarding | Ny fælles brugerflade | Bevar dansk/engelsk, tilgængelighed og enkel opsætning |
| Filtransskription | Fælles import- og transskriptionsflow | Erstat AVFoundation-normalisering; afklar formater, codecs og distribution |
| Historik og indstillinger | Fælles lagring | Import fra Swift-versionen, dataversionering, søgning og sletning |
| Credentials | OS-nøglering via adapter | Migrering på Mac, Windows/Linux-lagring og eksisterende .env-fallback |
| “Hey Vara” | Kandidat til fælles wake-word-motor | Prototype i dag; ressourceforbrug, mikrofonoverdragelse og stop ved stilhed |
| Opdateringer og installation | Fælles distributionsstrategi | Erstat/supplér Sparkle; signering, notarization og pakker pr. OS |

### Første foreslåede milepæl

En lille, komplet dikteringskæde på hver målplatform:

1. Hold en global genvej nede og optag mikrofonen.
2. Slip tasten og transskriber med én cloudmotor.
3. Indsæt teksten i den app, brugeren dikterede i.
4. Vis status og tillad annullering uden efterfølgende tekstindsættelse.

Afprøv fokus, tast-slip, mikrofontilladelser og fejl i rigtige mål-apps. Denne milepæl skal vise, om den centrale oplevelse fungerer, før hele brugerfladen og alle motorer flyttes.

Derefter kan vi udvide med modes/AI, historik, lokale modeller, filtransskription og wake word. Rækkefølgen er foreløbig.

### Adfærd, vi skal bevare

- Annullering må ikke efterlade en sen tekstindsættelse eller historikpost.
- AI-fejl eller timeout skal kunne falde tilbage til rå transskription.
- Ved transskriptionsfejl skal optaget tale kunne reddes, hvor det er muligt.
- Lokale/cloud-badges skal afspejle, hvor behandling faktisk sker.
- API-nøgler og transskriptioner skal håndteres bevidst i logning og fejldiagnostik.
- Genvej, lyd og sessionstilstand skal fungere, mens appen er i baggrunden.

### Åbne spørgsmål

- Er alle tre platforme mål for første udgave, eller prioriterer vi macOS og Windows først?
- Hvor tæt skal første Rust-version være på funktionsparitet med Swift-versionen?
- Skal første udgave fungere helt lokalt uden API-nøgle?
- Skal macOS beholde særlige Apple-motorer og native detaljer?
- Hvilke genveje skal være mulige på hver platform?
- Hvilke Linux-miljøer vil vi understøtte, særligt X11 kontra Wayland?
- Skal designet videreføres, eller ønsker vi samtidig at gentænke brugerfladen?
- Hvilke data skal flyttes automatisk fra den eksisterende Mac-app?
- Hvilken wake-word-oplevelse ønsker vi, og er den nødvendig i første udgave?
- Hvilke lyd-/videoformater skal filtransskription garantere?
- Hvordan skal modeller, codecs, licenser, installation og opdateringer distribueres?
- Hvordan får vi reel testadgang til Windows og de valgte Linux-miljøer?

### Kendte forhold ved udgangspunktet

- Den nuværende kode er opdelt i App, VaraCore og VaraCLI.
- `VARA_OVERVIEW.md` er nyttig, men beskriver ikke alle nyere ændringer.
- Der ligger allerede lokale, ucommittede ændringer omkring wake word, filtransskription, medienormalisering og tilhørende UI. De skal afstemmes og bevares før migrationsarbejde.
- Ingen omskrivning er startet. Denne scratchpad er ikke en verificeret implementeringsplan eller et løfte om identisk funktionalitet på alle platforme.

### Teknisk research fra samtalen

- [Tauri-plugins](https://v2.tauri.app/plugin/) og [globale genveje](https://v2.tauri.app/plugin/global-shortcut/).
- [CPAL](https://github.com/RustAudio/cpal) til lydoptagelse på flere platforme.
- [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) som mulig binding til whisper.cpp.
- [Enigo](https://github.com/enigo-rs/enigo) til inputsimulation. Wayland-understøttelse er angivet som eksperimentel; faktisk egnethed skal afprøves.

## 2026-10-18 — Hvad skal en ny Vara gøre bedre?

Produktforslag baseret på læsning af den eksisterende kode og projektoversigt. Det er hypoteser og anbefalinger, ikke brugerundersøgelser eller aftalte ændringer. Arbejdsantagelse: den primære bruger vil have sine egne ord hurtigt ind i den app, hvor vedkommende arbejder; avancerede brugere skal fortsat kunne styre motorer, prompts og endpoints.

### 1. Kortere vej til første vellykkede diktering

- Start med et anbefalet setup til den aktuelle maskine og en tydelig forklaring af behandling, download, eventuelle nøgler og omkostninger.
- Spørg kun om de nødvendige tilladelser, lad brugeren prøve en rigtig diktering, og vis hvordan genvejen bruges.
- Flyt valg af specifikke motorer, modeller og endpoints til et avanceret setup, som stadig er let at finde.
- Mål: tid fra første åbning til første korrekt indsatte sætning, samt hvor brugeren går i stå.
- Afklaring: lokal behandling som standard kræver en løsning, der også er hurtig og brugbar på de understøttede maskiner. Cloud kræver afklaring af nøgle/betaling. Ingen standard er besluttet.

### 2. Gør hver diktering mulig at redde

- Gør den eksisterende lydgendannelse synlig: genprøv med samme eller en anden motor, og kopiér rå tekst når AI-bearbejdning fejler.
- Skeln mellem transskriptionsfejl og fejl ved indsættelse. En korrekt tekst skal kunne hentes igen uden ny optagelse.
- Gem kun lyd efter en tydelig, afgrænset politik. Vis hvornår den slettes, og giv mulighed for straks at slette den. Gendannelse og privatliv skal designes sammen.
- Ved usikkert eller ændret fokus: tilbyd kopi/manuel indsættelse frem for at gætte på et nyt tekstfelt.
- Mål: optagelser der mister indhold, samt om en fejl kan løses uden at gentage det sagte.

### 3. Oplevet hastighed som en kerneegenskab

- Undersøg forberedelse af valgt motor og transskription under optagelse, hvor motoren understøtter det.
- Vis genkendt tekst undervejs, hvis det hjælper; skeln tydeligt mellem foreløbig og færdig tekst. Indsæt som udgangspunkt først ved afslutning.
- Hold styr på tid til optagelsesstart, slip-til-rå-tekst og slip-til-indsat-tekst separat. AI-bearbejdning må ikke skjule, hvor tiden bruges.
- Sammenlign typiske og langsomme forløb på korte danske dikteringer. Sæt konkrete hastighedsmål efter en baseline.

### 4. Klarere skel mellem at skrive og at spørge AI

- Foreslå trofast diktering med let oprydning som hovedforløb: betydning, navne, tal og forbehold skal bevares.
- Behold ordret tekst og brugerdefinerede modes som tilgængelige valg.
- Gør det tydeligt, når et forløb genererer et svar frem for at gengive brugerens ord. Ask AI kan kræve en anden præsentation og forhåndsvisning end diktering.
- Flyt prompts og modelvalg til redigering af den relevante mode. Undersøg om de nuværende indbyggede modes er forståelige med konkrete før/efter-eksempler.
- Afklar særskilt adfærd for kode, mails og længere tekst; undgå automatisk omskrivning alene ud fra app-navnet, medmindre brugeren har valgt en regel.

### 5. Bedre dansk, fagord og rettelser

- Giv mulighed for udtale/stavning af navne, fagord, forkortelser og faste vendinger.
- Skeln mellem ordforråd, som hjælper genkendelsen, og eksplicitte tekstregler som “skriv altid X som Y”. Vis begrænsninger for den valgte motor.
- Tilbyd at gemme en rettelse som regel, når brugeren selv beder om det. Ingen skjult overvågning af det, der skrives i andre apps.
- Undersøg dansk/engelsk sprogskift, tekniske termer, tal, datoer og tegnsætning med egne repræsentative eksempler.

### 6. En mere målrettet daglig brugerflade

- Menubakke og HUD skal gøre aktiv mode, mikrofon og optagelsesstatus tydelige uden at afbryde arbejdet.
- Hovedvinduet bør prioritere seneste resultat, genprøvning, historik og filtransskription. Statistik kan være sekundær.
- Saml problemer og deres løsning i kontekst: mikrofon, genvej, model, forbindelse eller indsættelse.
- Design tastaturbetjening, skærmlæser, skalering og reduceret bevægelse fra starten.
- Bevar nyttige genveje og tætte ekspertvisninger, hvor de gør gentagne opgaver hurtigere.

### 7. Filtransskription som et selvstændigt arbejdsforløb

- Træk/slip, tydelig fremdrift, annullering, genprøvning og eksport til flade filer.
- Foreslå TXT/Markdown til læsning og SRT/VTT når tidskoder findes.
- Overvej kø til flere filer, tidskoder og taleropdeling efter det grundlæggende forløb. Motorernes kapacitet skal afgøre, hvad UI lover.
- Længere optagelser kræver en plan for størrelsesgrænser, opdeling og fortsættelse efter fejl.
- Hold sammenfatning og andre AI-resultater adskilt fra selve transskriptionen.

### 8. Privatliv som en forståelig indstilling

- Vis separat, hvor lydtransskription og tekstbearbejdning foregår. Lokal talegenkendelse kan stadig følges af tekst sendt til cloud.
- Overvej en streng “kun på denne enhed”-indstilling, som forhindrer cloud-fallback.
- Giv forståelig styring af teksthistorik, gendannelseslyd, automatisk sletning og diagnosticering.
- Log som udgangspunkt tekniske hændelser uden talens indhold. Wake-word-prototypens indholdslogning skal vurderes før en ny udgave.

### 9. En ny kerne med klare ansvarsområder

- Én session styrer optagelse, transskription, bearbejdning, indsættelse og annullering; hvert trin har et tydeligt resultat.
- Brug motorernes faktiske egenskaber til at styre valgmuligheder: sprog, streaming, ordforråd, lokalitet, tidskoder og taleropdeling.
- Gør mikrofon, motor, lagring og tekstindsættelse udskiftelige, så fejl og timing kan testes uden at tale ind i en mikrofon hver gang.
- Genbrug relevante scenarier fra Swift-testene som adfærdskrav, især annullering, fallback, indstillingsmigration og lydbehandling.
- Tilføj et lille, versionsstyret evalueringssæt med tilladte danske/engelske lydklip og forventninger til navne, tal, betydning og hastighed.

### Foreløbig prioritering

| Prioritet | Forslag | Hvorfor |
|---|---|---|
| Først | Pålidelig optagelse, korrekt indsættelse, annullering og gendannelse | Brugeren skal kunne stole på hver diktering |
| Først | Enkel opsætning og et godt anbefalet setup | Hurtigere første succes |
| Først | Hastighed og dansk kvalitet målt på realistiske eksempler | Direkte værdi i daglig brug |
| Derefter | Klarere modes, ordforråd og synlig privatlivsstyring | Mere kontrol med mindre usikkerhed |
| Derefter | Bedre filforløb og migration fra Mac-appen | Udvider nytten og gør overgangen mulig |
| Afprøv separat | Wake word, automatisk kontekst og mere autonom AI | Kræver bevis for nytte, præcision og acceptabelt ressourceforbrug |

### Hvad vi bør udfordre før vi viderefører det

- Om alle nuværende motorer skal med i første version. En adapter bør have en konkret fordel og kunne vedligeholdes.
- Om mange indbyggede modes hjælper mere end få tydelige standarder og egne modes.
- Om et statistikdomineret dashboard hjælper brugeren med dagens opgave.
- Om wake word skal være en del af første release.
- Om identisk UI på alle platforme er vigtigere end genkendelig, passende adfærd på hver platform.
- Om fuld funktionsparitet er et godt første mål. Den centrale dikteringsoplevelse og migrationsmuligheden bør styre rækkefølgen.

## Flere idéer og ønsker

Tilføjes løbende. Næste produktafgrænsning: hvem første nye udgave primært er til, og hvilke nuværende irritationer der vejer tungest for brugeren.
