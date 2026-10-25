# Transskribering i Vara

Dato: 2026-10-25. Status: implementeret på arbejdsbranchen; native slutkontrol afventer en ulåst Mac.

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

Talermarkering er en særskilt kvalitetstest før produktløfte. Who er en kandidat,
men der er ikke truffet valg af model, runtime eller licensvilkår. Brugeren har
ikke bestilt en Who-integration.

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
