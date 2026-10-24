# Groq — opdateret integration, 23. oktober 2026

Groq var allerede til stede i grundkoden. Denne ændring færdiggør de relevante
modelvalg, opdaterer tekstmodellens standard, tilføjer en nøgle/katalogkontrol og
verificerer de rigtige HTTP-forespørgsler med en lokal server.

## Brug i appen

1. Genstart den opdaterede app. Åbn **Indstillinger → Forbindelser → Groq →
   Administrer nøgle**. Indtast din egen Groq-nøgle og vælg **Gem nøgle**.
2. Åbn nøgle-dialogen igen og vælg **Kontrollér adgang**, hvis du vil kontrollere
   nøglens accept og om de konfigurerede modeller findes i modelkataloget.
3. Vælg **Groq · Whisper Turbo** eller **Groq · Whisper Large-v3** under
   **Behandling af lyd**, og gem indstillingerne for at bruge Groq til tale.
4. Til valgfri AI-bearbejdning vælges **Groq · GPT-OSS 120B** i generelle
   AI-indstillinger eller i en appregel. At gemme en nøgle slår ikke cloud eller AI til.

Redux er fortsat standard for nye profiler. Cloudvalget fremgår af sidebjælken.
Streng lokal tilstand afviser også Groq. Direkte transskription og historikregler
ændres ikke. API-nøglen gemmes i OS-nøglelageret, aldrig i historik eller UI-snapshot.

Gemte modelnavne bevares. Hvis en ældre profil stadig har en udgået Groq-tekstmodel,
kan den ændres under **Modeller og egne endpoints**; adgangskontrollen viser, hvis
modelnavnet mangler i kataloget.

## Modeller og API

- Tale: `whisper-large-v3-turbo` og `whisper-large-v3` på Groqs faste
  `/openai/v1/audio/transcriptions`-endpoint, multipart WAV, Bearer-auth, valgfrit
  sprog, JSON-svar og temperatur 0.
- Tekst: ny standard `openai/gpt-oss-120b` på `/openai/v1/chat/completions`.
  Systemprompt og transskription sendes som adskilte beskeder. Reasoning holdes ude
  af svaret; lav reasoning effort og en udtrykkelig token-grænse anvendes.
- Adgangskontrol: kun `GET /openai/v1/models` med den gemte nøgle. Ingen lyd eller
  transskription sendes. Katalogtilstedeværelse er ikke en faktisk
  transskriptionstest eller garanti for modeltilladelser, kvote og betaling.

Den tidligere standard `llama-3.3-70b-versatile` er ifølge Groq udgået for almindelige
konti. Kilder kontrolleret den 23. oktober 2026:
[Speech to Text](https://console.groq.com/docs/speech-to-text),
[deprecations](https://console.groq.com/docs/deprecations),
[GPT-OSS 120B](https://console.groq.com/docs/model/openai/gpt-oss-120b),
[reasoning-format](https://console.groq.com/docs/reasoning).

## Verifikation

**24 Rust-tests består**: 11 domæne-/lagringstests, 7 model-/integritetstests og 6
nye lokale HTTP-kontrakttests. Sidstnævnte kontrollerer:

- Rigtig multipart-lydforespørgsel, model, sprog, fil og Authorization-header.
- Rigtig tekst-JSON med systemprompt og kun den færdige tekst som resultat.
- Ufuldstændigt AI-svar bliver en fejl, så originalteksten kan bruges.
- 401/429-fejl giver brugbare beskeder uden at gengive upstream-fejl eller nøgler.
- Katalogkontrollen sender en GET uden lyd-/tekst-body.
- Accepteret annullering afbryder en afventende cloud-forespørgsel.

**12 UI-testgrupper består**, inklusive Groq-nøgle, adgangssvar/adgangsfejl,
hemmelig inputryddelse, begge talemodelvalg og eksplicit cloudvalg. Testnøglen er
fiktiv og bruges kun mod en lokal server/IPC-fixture. Release-bundle er bygget til
Mac uden test-harness. GDD-model og grafvalidering består.

## Begrænsninger

Ingen rigtig Groq-nøgle er læst eller brugt, og ingen lyd er sendt til Groq under
arbejdet. Live-konto, latency/kvalitet og den nye native UI er derfor ikke
slutverificeret. Brugerens tidligere ønske om at fortsætte uden native UI-kontrol
er respekteret. Den ekstra real-weight Redux-test var allerede bestået ved den
foregående integration; den er ikke gentaget for denne cloudændring.
