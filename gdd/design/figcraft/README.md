# Vara i FigCraft

Åbn `Vara.figcraft` i FigCraft. Filen dokumenterer desktop-UI'et fra revision `891335492bc47738bb6fbebc615b687dad8d2bac` ved 1280 × 820. Den er et redigerbart udgangspunkt, ikke en interaktiv prototype. Tekst er tekstlag, ikoner og former er vektorer, og den eneste bitmap er den faktiske Mail-appillustration. Indholdet er fiktive testdata.

- **01 · Primære skærme:** historik og original, AI pr. app, alle tre indstillingsfaner, transskriptionsimport og dokumentvisning, AI-fanen samt fortsættelser af indstillingerne ved scroll.
- **02 · Dialoger og tilstande:** behandlingsvalg, sletning, Who-download, kendt antal og usikkerhed, tre opsætningstrin, genvej, nøgle, appvælger, eksport, endpoint og model samt tre HUD-tilstande.
- **03 · Komponenter:** dele udtrukket fra de faktiske renderede elementer. Sidepaneler med navigation, primære knapper, felter og gemmelinjer bruges som instanser i skærmene. Varianter med forskellig tekst, position eller tilstand har hvert sit hovedkomponent.

Filen har 31 skærm-/tilstandsframes, 31 hovedkomponenter og 51 instanser. CLI'ens `info` rapporterer 3.831 lag; den native JSON indeholder 1.397 tekstlag. Teksten er opdelt efter synlige linjer og kan redigeres, men skærmene har ikke responsivt autolayout eller fungerende handlinger. Komponenterne indeholder også redigerbar tekst og vektorer. Dialogskærme er selvstændige snapshots for at bevare lagrækkefølgen under backdrop.

## Visuel kontrol og begrænsninger

`references/` indeholder browseroptagelser af det aktuelle UI med en mock af Tauri og de sanitiserede fixtures fra `desktop/tests`. `exports/` indeholder udvalgte eksporter fra den endelige native fil. `comparisons/` viser de samme par side om side, browser til venstre og FigCraft til højre. Sammenligningerne dækker historik, appregel, diktering, transskriptionseditor, Who og de scrollede indstillinger. De viser, at placeringer og farver ligger tæt på referencen, men ikke er pixelidentiske: tekstvægt, skriftrendering, enkelte tekstmellemrum/ellipsis, fokusmarkeringer, skygger og native kontroller kan afvige. FigCraft mangler ingen fonte ifølge `info`; browser og FigCraft kan alligevel tegne systemfonte forskelligt. Browserreferencen er Chromium, ikke en måling af den kørende macOS WebView.

HUD-frame 18–20 er rekonstrueret ud fra `crates/vara-platform/src/macos_hud.m`, tokenfarverne og `desktop/src-tauri/src/service.rs`: 520 × 76, de faktiske etiketter, kontroller og et illustrativt lydniveau. Den er **ikke** optaget fra en kørende macOS-HUD og har ingen pixelreference. Den reelle app bestemmer appikon og lydmåler dynamisk.

## Genskab

FigCraft CLI 0.12.1 kan samle den native fil fra de medfølgende SVG- og fragmentdata uden at køre appen:

```sh
python3 gdd/design/figcraft/build.py
'/Applications/FigCraft Tools/figcraft-cli-0.12.1-macos-universal/figcraft-cli' info gdd/design/figcraft/Vara.figcraft --json
'/Applications/FigCraft Tools/figcraft-cli-0.12.1-macos-universal/figcraft-cli' export gdd/design/figcraft/Vara.figcraft -o /tmp/vara-history.png --node 01-history
```

Sæt `FIGCRAFT_CLI` til den lokale CLI-sti, hvis den er installeret et andet sted. `build.py` opretter komponenter fra de udtrukne dele, importerer skærmene og placerer instanserne med CLI; ingen screenshots indlæses i dokumentet.

For at tage browserreferencerne igen kræves Node, en lokal Playwright-installation og en kompatibel Chromium. Start `python3 gdd/design/figcraft/serve.py` i en terminal, og kør derefter:

```sh
PLAYWRIGHT_MODULE=/absolut/sti/til/node_modules/playwright \
PLAYWRIGHT_CHROMIUM=/absolut/sti/til/chrome-headless-shell \
node gdd/design/figcraft/capture-runner.cjs
python3 gdd/design/figcraft/build.py
```

Stop serveren efterfølgende. Hvis Playwright og dens browser findes i standardplaceringerne, kan de to miljøvariabler udelades. Capture-scriptene indeholder en fastfrosset kopi af de fiktive fixtures og Tauri-mocks fra nævnte revision; de læser UI-koden og CSS fra checkoutet. Ved ændringer i fixtures eller mockkontrakter skal capture-scriptene opdateres, før optagelserne betragtes som en ny reference. Ingen rigtige nøgler, optagelser eller transskriptioner bruges.
