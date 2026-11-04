# Transskribering i Vara

Dato: 2026-10-25. Status: implementeret; native macOS-sluttest bestået 2026-10-26.
Evidens og modelafhængige begrænsninger: [rapport](reports/transcription/README.md).

Brugeren har ønsket en selvstændig transskriberingsfane, der kan tage indhold
med lyd, og bedt om at få featurelisten og ordniveau-tidsstempler skrevet ned.
Nedenstående samler retningen og det foreslåede første leveranceomfang.
Konkrete formater, modelunderstøttelse, opbevaring og testresultater fremgår nu af
[implementeringsrapporten](reports/transcription/README.md). Tabellen nedenfor
bevarer produktkravene; modellen bestemmer de tilgængelige tids- og talerdata.

## Formål og brugerflade

En fane **Transskribering** til lyd- og videofiler, adskilt fra den hurtige
hold-tast-og-diktér-funktion. Primært forløb:

**Tilføj fil → transskriber → læs og lyt → ret → kopiér eller eksportér.**

Følg det valgte Warm precision-design i DESIGN.md og genbrug fælles komponenter.
Vis få relevante valg ved import og en læsbar tekst efter transskription.
Den interne opdeling i ord må ikke gøre editoren til en tabel over enkeltord.

## Features

| Feature | Ønsket adfærd |
| --- | --- |
| Lyd- og videoimport | Træk filer ind, eller vælg dem fra computeren. Udtræk selv lyd fra video. Vis en forståelig fejl ved manglende lyd eller et format, der ikke kan læses. “Alt med lyd” er produktretningen; de faktisk understøttede formater skal verificeres. |
| Flere filer | Tilføj flere filer til en kø med individuel status og resultater. |
| Modelvalg | Genbrug Varas modeller og nøglehåndtering. Lokalt som standard; cloud vælges bevidst, med tydelig destination for lyd. Ingen automatisk cloud-fallback. |
| Sprog | Automatisk genkendelse med mulighed for et eksplicit sprog, fx dansk, hvor modellen understøtter det. |
| Fremdrift og kontrol | Vis filens aktuelle trin, tillad annullering og genprøvning ved fejl. Opfind ikke procenttal eller resterende tid. Annulleret arbejde må ikke senere publicere et resultat. |
| Teksteditor | Læs og ret teksten med automatisk lagring. Bevar originaltransskriptionen separat. |
| Afspilning | Afspil kilden med tidsposition og justerbar hastighed. Hop fra tekst til lyd med den præcision, der faktisk findes. |
| Talermarkering | Vis og omdøb Taler 1, Taler 2 osv., når modellen leverer brugbare talerdata. Lokal understøttelse skal kvalitetstestes, før den loves. |
| Valgfri tekst-AI | Genbrug Varas tekstmodeller til oprydning, resumé og egne instruktioner. Gem resultatet adskilt fra den tidskodede original. |
| Kopiering og eksport | Kopiér tekst; eksportér TXT, Markdown og struktureret JSON. SRT/VTT kræver verificerede tidsdata og passende undertekstsegmentering. |
| Bibliotek | Gem transskriptioner under fil/titel, med søgning, omdøbning og sletning. Importerede filer skal ikke grupperes under en tilfældig aktiv app. |
| Kildesikkerhed | Ændr eller slet aldrig brugerens originalfiler. Ryd midlertidige arbejdsfiler op. Hvis en ekstern kilde flyttes eller slettes, bevar teksten og forklar, hvorfor afspilning ikke er tilgængelig. |

## Ordniveau som datagrundlag

Kontrakten skal fra starten kunne rumme **start- og sluttid pr. ord** samt
valgfri talerreference. En ordtidskode er en estimeret placering i lyden,
ikke en garanti for perfekt præcision.

Det skal gøre følgende muligt:

- Klik på et ord for at afspille fra dets start.
- Fremhæv det aktuelle ord under afspilning.
- Markér en passage og afspil dens tidsinterval.
- Saml ord i læsbare afsnit og passende undertekstlinjer.
- Eksportér ord, tidsstempler og eventuelle talere i JSON.

### Foreslået kontraktgrænse

- Et dokument har kildeoplysninger, model/proveniens, originaltekst og ordnede
  segmenter. Originalresultatet bevares uafhængigt af rettelser og AI-output.
- Segmenter kan indeholde ord med stabile ID'er, tekst, start/slut i sekunder
  relativt til kildens begyndelse og valgfri talerreference. Tegnsætning kan være
  uden et selvstændigt lydinterval; tokenisering må ikke antages ens mellem modeller.
- Tidsunderstøttelse angives eksplicit: ingen, segmentniveau eller ordniveau.
  Oprindelsen angives også: talegenkendelse eller efterfølgende alignment.
- Manglende tidsdata er fraværende, ikke nul eller beregnede gæt. Modeller, der kun
  leverer tekst eller segmenter, skal stadig kunne bruges med synlige begrænsninger.
- Alle tidsdata skal være endelige, ikke-negative og have slut ≥ start, inden for
  kildens varighed med en dokumenteret tolerance. Talere kan overlappe i tid.
- Retning og ord-ID'er skal bevare forbindelsen til originalen. Ændrede eller nye
  ord får status **kræver tidsjustering**; gamle tidsstempler må ikke præsenteres
  som verificeret alignment af den redigerede tekst.
- Tidsafhængig eksport af rettet tekst kræver gyldigt alignment eller en eksplicit
  vist begrænsning. TXT/Markdown forbliver tilgængelige uden tidsdata.
- AI-resumé og omskrivning er separate tekstresultater uden arvede ordtidskoder.

Separat **forced alignment** kan undersøges for at placere en kendt tekst i lyden.
Det skal afprøves på dansk og på de valgte modeller; det er ikke valgt eller
implementeret endnu. Robusthed og målt kvalitet går forud for at love ordniveau
for alle modeller.

## Første leverance og efterfølgende arbejde

Foreslået første leverance: import, kø, eksisterende modelvalg/sprog, status,
annullering/genprøvning, original + redigeret tekst, bibliotek og tekst/JSON-eksport.
Kontrakten for ordniveau skal med fra begyndelsen. Afspilning, klik/fremhævning og
SRT/VTT medtages, når de valgte runtime-/modelkombinationer leverer verificerede
tidsdata. Konkrete kompatibilitetsgrænser skal fremgå af UI og dokumentation.

Who er valgt som valgfri lokal taleropdeling efter brugerens bestilling
2026-10-26. “Find talere” i editoren kører den tilpassede SDK-runtime via en
C-grænse fra Rust i en separat, annullerbar proces. Ingen automatisk download,
cloud-fallback eller ændring af hurtig diktering.

Original ASR-tekst, ord-ID’er og tider forbliver uændrede. Who-turns gemmes separat
med model/version og kan overlappe. En afledt læsevisning kobler kun en taler til
et ord/segment, når der er én tydelig overlappende stemme med mindst halv dækning.
Tvetydige ord står uden en valgt Who-taler. Uden ordtider vises de faktiske
lydintervaller uden at opfinde en fordeling af tekst. Rettet tekst får ikke nye
tids- eller talerdata. JSON indeholder rå ASR, Who-turns og redigering hver for sig.
Talernavne kan omdøbes i den eksisterende komponent.

Automatisk er standard. Brugeren kan angive et kendt antal fra 1 til 16; dette
kræver tilstrækkeligt mange tilbagevendende SDK-grupper. De stærkeste vælges efter
antal lydstykker og aktiv varighed og bevares som adskilte referencer. Et
referencelydspor skal entydigt matche sin egen reference, når hele det aktuelle
lydstykke er udeladt fra prototyperne; ellers markeres det usikkert. Øvrige
lydspor matches kun ved præcis ét cosinusmatch over modellens egen grænse;
uklare spor, herunder spor uden stemmevektor, beholdes som usikre og tildeles
ikke ord. JSON bevarer rå SDK-intervaller, afledte intervaller, valgt antal,
metode og grænse. Ved kendt antal køres en anden kontrol med 60 sekunders
foranstillet stilhed. En entydig global én-til-én-kobling kræves ud fra
udelukkende enestemmige, sikre tidsintervaller. Et primært interval får kun sine
sikre talere, når begge kørslers komplette talersæt er ens og ingen er usikker;
øvrig primær aktivitet er usikker. Der tilføjes ikke aktivitet, som kun findes i
kontrolkørslen. Derfor kan antallet af sikre ID'er blive lavere end det valgte
antal. Metoden gemmer kontrolforskydningen som en versionsangivet del af
metoden. Midlertidige stemmevektorer gemmes ikke i dokumentet.

Modellen hentes eksplicit i Who-dialogen, som viser leverandør, licens og SDK’ets
brugstælling før download/brug. Lyden og teksten behandles på enheden; SDK’et sender installations-id og model-/app-/platformmetadata
til leverandørens licenssystem. SDK’ets rapportering bevares. Modellerne følger
ikke med appen. Core ML på Mac kræver macOS 15 eller nyere; appens øvrige funktioner
bevarer deres eksisterende OS-grænse. Linux/Windows bruger samme Rust-kontrakt og
SDK-C-grænse; deres native runtime-pakning skal verificeres på målplatformene.

Fejl, ingen fundne talere og annullering bevarer tekst, rettelser og eventuel
sidste vellykkede taleropdeling. Ingen talere er ikke bevis for, at ingen taler:
et naturligt dansk klip på 10 sekunder gav ingen Who-turns, mens samme klip gentaget
til 41 sekunder gav én stemme. To syntetiske stemmer blev genkendt og matchet på
tværs af et femminutters skel. Dette er funktionstest, ikke en kvalitetsgaranti
på naturlige danske samtaler.

Links til fx YouTube og optagelse af møde-/systemlyd er efterfølgende inputtyper.
Valgfri tekst-AI er del af featurelisten; den præcise placering i leverancerækkefølgen
fastlægges ved implementering.

## Afklaringer før implementering

- Hvilke lokale/cloud-modeller leverer reelle ord- eller segmenttidsdata gennem
  de nuværende adaptere, og er separat alignment nødvendig?
- Skal afspilning bruge en reference til originalfilen eller en bruger-valgt kopi
  i Vara? Reglerne for at slette dikteringslyd må ikke automatisk genbruges som
  regler for importerede filer og deres eventuelle afspilningskopier.
- Konkrete grænser for filstørrelse/varighed, streaming/chunking og fejlgendannelse
  skal fastlægges ud fra runtime og udbydere, uden at kilden ændres.

## Verifikation af første version

Afprøv dansk lyd, video med lyd, en fil uden lyd, flere køelementer, annullering,
fejl/genprøvning og genstart. Bevar originalfil og originaltekst i alle forløb.
Kontroller ordklik og fremhævning mod faktisk afspilning, rettelsers alignmentstatus,
taleroverlap og eksport. UI skal også fungere for en model uden tidsdata og en
flyttet kildefil. Eksisterende diktering, AI-indstillinger og historik skal bevares.

Før implementering skal de konkrete kontrakter indarbejdes i den aktive GDD-model
med tilhørende evidens. Denne note ændrer ikke i sig selv eksisterende kontrakter.

Annullering af et uafsluttet filjob fjerner dets private arbejdslyd; originalfilen
forbliver urørt. Ved genprøvning af mislykket talegenkendelse kan en gyldig lokal
lydkopi genbruges, selv hvis den eksterne original er flyttet.

## Grænsefladesprog og talernavne

Filtransskriptionens app-ejede labels, dialoger og tilgængelige navne vises på
dansk eller engelsk uafhængigt af talesproget. Sprogskift opdaterer synlige
labels uden at starte arbejde eller erstatte en aktiv editor eller afspiller.
Ord, tidsdata, stier, AI-resultater og brugervalgte talernavne ændres ikke.
Nye automatisk genererede talernavne får eksplicit ordinal-proveniens, så UI kan
vise Taler 1 eller Speaker 1. Gamle gemte navne fortolkes aldrig heuristisk;
omdøbning fjerner proveniensen for det pågældende navn. Proveniensen gemmes i
dokumentet, men indgår ikke i eksport, hvis eksisterende eksportformat skal
bevares. Usikre passager tælles ikke som en person.

## Fælles filjobgrænse — 2026-11-04

På macOS deler desktop og den lokale backend én `vara-app::Files`-ejer pr.
profil. Ejeren låser profilen, ejer køen, dokumentlisten og publiceringen og
bruger den eksisterende fil-SQLite og mediemappe. Desktop er adapter for filvælger, clipboard, eksportdialog, links og UI-events;
backend ejer ressourcesti og legitimationsopslag. Windows og øvrige platforme
beholder den eksisterende in-process adapter; deres lokale transport er ikke leveret. Det eksisterende dokumentformat og redigering, AI, Who, afspilning og
eksport bevares. En separat CLI-database eller konkurrerende kø er ikke tilladt.

Den fælles Rust-grænse tilbyder `submit`, `status`, `result` og `cancel`.
`submit` modtager én sti, valgfrit tale-engine-ID, valgfrit talesprog og valgfrit
request-ID; kvitteringens job-ID er dokument-ID'et. Indstillinger for engine og
sprog gemmes på dokumentet ved accept uden at ændre globale indstillinger.
`status` returnerer dokumentets faktiske stage; `result` kræver Ready og giver
adgang til det fulde dokument og dets eksisterende eksportmetoder. Eksplicit
annullering bruger den eksisterende cancellation og kan ikke publicere et sent
resultat. Desktopbatch anvender samme kø og lager, uden request-ID som standard.

Et request-ID er scoped til profilen og det tilhørende dokuments levetid. Det
skal være 1–200 UTF-8-byte, indeholde mindst ét tegn, der ikke er whitespace,
og kan kun bruges til ét filjob. Inputidentiteten består af
absolut sti, det anmodede engine-ID (også udeladt) og normaliseret talesprog (tomt
sprog svarer til automatisk). Den identificerer stien, ikke filens indhold eller
symlink-mål. Samme ID og identitet giver den oprindelige kvittering, også efter
at kilden er fjernet; ændrede parametre giver `file.request_conflict`.
Accept og requestregistrering committes atomisk, før kvittering/publicering.
Sletning af dokumentet sletter også registreringen; derefter kan ID'et bruges
på ny. En klient, som indsender en relativ sti over IPC, skal gøre den
absolut i klientens eget arbejdskatalog, inden den sendes til backend.

Ved genstart markeres alle afbrudte ikke-terminale jobs Failed med den
versionerede `file.interrupted`-årsag; de køres ikke automatisk igen. En
forbindelse eller ventende klient ejer ikke et accepteret jobs levetid. Mac-
transporten er en versioneret Unix-domain-socket (intern version 2, adskilt fra CLI-JSON version 1) begrænset til samme OS-bruger,
med én låst backend pr. profil og automatisk opstart i en separat procesgruppe,
så klientens Ctrl-C ikke afbryder backend. Forskellige interne versioner afvises, så en gammel backend ikke kan ignorere en Who-bestilling. Versionkonflikt stopper ikke en ældre backend eller dens aktive jobs automatisk. Desktop følger instans og
revision og genindlæser ved ændringer; seneste fejlmeddelelse vises højst én
gang pr. instans og revision. Aktiv AI og Who samt Who-download kan ses og
annulleres efter GUI-genforbindelse. Lukkes GUI'en, fortsætter accepteret arbejde;
backend stopper normalt efter 60 sekunders inaktivitet uden klienter eller jobs.
Framing er tidsbegrænset, men ventetid på AI/Who er ikke en jobdeadline.
Backend læser profilens gemte indstillinger og holder nøgler hos den betroede
worker; den eksponerer ikke en generel invoke- eller secret-grænse.

## Agent-CLI v1 — kontrakt (#27–28)

Desktop-binæren accepterer `cli` før kommandoen; den signerede `vara`-launcher
leveres i #29. Kommandoer: `transcribe FILE [--model ID] [--language CODE] [--diarize] [--speakers 1–16]
[--original] [--request-id ID] [--no-wait] [--timeout SECONDS] [--format json|txt|md|srt|vtt]
[--output FILE] [--overwrite]`, `job status|wait|result|cancel ID` (wait/result
accepterer format/output/overwrite/original, wait også timeout), `models [--json]`,
`--help`, `--version`. Ingen interaktion. Udeladt model er den gemte
`speech_engine`, udeladt sprog automatisk; Redux understøtter kun automatisk.
Inputstier gøres absolutte i klientens aktuelle mappe; deres eksisterende
forældremappe opløses, uden krav om at filen stadig findes ved retry. Request-ID sammenligner
rå valg inkl. udeladt model før defaults opløses, og første job fastholder sin
engine. Ingen globale indstillinger ændres.

JSON på stdout er ét objekt med `schema_version: 1`, `ok`, og ved succes
`job_id`, `document_id`, `status` og eventuelt `document`; models bruger
`models`, `default_speech_engine` og `strict_local`; cloud-readiness `null`
betyder ikke kontrolleret. `timing_capability` beskriver adapterens mulighed
(`word_or_segment`, `segment`, `word_requested`, `none`), mens dokumentets
`transcript.timing` og `timing_source` beskriver faktisk data. Fejl har `error: {code, message?}` med sikre,
stabile maskinkoder og katalogiserede numeriske årsager. Exit: 0 succes/kvittering,
2 ugyldig brug/input, 3 utilgængelig model/nøgle/runtime/privacy, 4 job fejlet
eller annulleret, 5 transport/version, 6 eksport, 7 timeout/afbrud. Help/version
har menneskeligt output. Kun eksplicit ikke-JSON færdig eksport uden `--output`
skriver eksporttekst på stdout. `--output` giver JSON-kvittering og kræver
`--overwrite` for eksisterende mål. Alle kilder kendt af dokumentejeren, arbejdslyd og profilinterne filer må
ikke overskrives, heller ikke via links. Ejerens lås dækker kildekontrol og
atomisk publicering. En relativ outputsti opløses i klientens arbejdsmappe.
RPC-dokument og JSON-eksport bruger samme sikre projektion af gamle endpoints
og rå fejl uden at ændre det gemte dokument eller transskriptionens tekst. Eksportfejl ændrer ikke dokumentet.

Default venter uden jobdeadline; `--no-wait` giver holdbar kvittering. Accepteret
job-ID skrives straks på stderr. Før submit skriver klienten altid et request-ID
på stderr: enten brugerens eller et nyt UUID for netop dette kald. `--no-wait` med `--output` eller `--overwrite` er en inputfejl; brug
`job result` til fileksport efterfølgende. `--no-wait --format` giver JSON-kvittering. Hvis accept
er ukendt ved timeout/afbrydelse, indeholder fejl-JSON dette `request_id`;
gentag de samme argumenter med `--request-id` for at genfinde jobbet. Separate
kald uden eksplicit ID er separate jobs. `--no-wait` giver status `accepted`.
Timeout, signal og lukket pipe stopper alene
klientventning; cancel er eksplicit. Timing og faktiske ord/segmenter kommer fra
dokumentet, uden konstruerede ordtider. Backendfejl er katalogiserede og ingen
nøgler, endpoints eller provider-fejltekster returneres.


`--diarize` gemmer et påkrævet Who-trin med dokumentet; `--speakers` kræver
`--diarize` og skal være 1–16. Valgene indgår i request-identiteten. Who benytter
eksisterende lokal runtime og opsætning uden download eller licensaccept. ASR
forbliver `Ready` efter gemt ASR, mens CLI viser `processing` indtil Who er færdig.
Kvitteringen indeholder særskilt `asr_stage` og `who`. Who-fejl eller annullering
giver ikke-nul exit, status `who_failed`/`who_cancelled` og dokument-ID; tekst, lyd
og eksisterende data bevares. Pending/running Who markeres afbrudt efter backend-
genstart og genkøres ikke automatisk. Cancel dækker begge faser, også overgangen. Hvis ASR fejler før Who, gemmes Who som `skipped`.

`--original` vælger oprindelig ASR-tekst og timing til eksport; ellers bruges gemt
redigering. Usendte GUI-udkast er ikke tilgængelige. SRT/VTT kræver reelle tider og
afviser redigeret tekst uden passende timing. JSON inkluderer separat rå ASR,
Who-intervaller og `speaker_transcript` afledt af samme core-annotation som UI;
`selected_text` og `original_selected` viser eksportvalget. Usikkerhed er ikke en
ny taler, og u-timede ord får ikke fabrikerede tider.
