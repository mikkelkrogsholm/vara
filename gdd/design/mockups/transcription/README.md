# Transskriberingsmockups — 2026-10-25

Genereret med den indbyggede imagegen ud fra `gdd/DESIGN.md`, den valgte
Warm precision-reference og `gdd/TRANSCRIPTION-SPEC.md`, med keep-ui-simple.
Designforslag, ikke implementeret eller brugeraccepteret funktionalitet.

- `01-editor.png`: bibliotek/kø, transskription, talere, ordmarkering, original,
  AI-fane, kopiering/sletning, eksport og afspiller.
- `02-import.png`: filvalg/drop, flere valgte filer, model/lokalitet, sprog,
  eksplicit start og seneste resultater.

Orddata, talere og undertekster er betinget af verificeret modelunderstøttelse.
Eksportmenuen skal rumme TXT, Markdown og JSON samt SRT/VTT, hvor tidsdata findes.
AI-output holdes adskilt fra den tidskodede tekst. Fejl/genprøvning, manglende kilde
og redigering med behov for nyt alignment er ikke afbildet i disse to hovedskærme.

Ved implementering bruges DESIGN.md's systemskrift til UI-overskrifter og de
fælles eksisterende navigationselementer/logo. Billederne har mindre variationer
i logo, overskrifttypografi og ikoner, som ikke er ændringer til designsystemet.

## Prompt — editor

Use case: ui-mockup. Create a high-fidelity premium desktop application screenshot for Vara, a Danish local-first dictation/transcription app. This is a proposed future transcription editor, not marketing art. One single fully visible landscape desktop window, straight-on, high resolution, readable Danish UI, almost edge to edge, with no inset screenshots or device photography.

Match Vara's established Warm precision design precisely: dark olive sidebar #2F3527, warm paper canvas #F7F4EE, ivory editor #FFFCF7, subtle thin dividers #DCD6CA, charcoal text #25251F, muted #636459, selected beige #E8E1D5, restrained burnt orange #A84417 for the single main action and playback highlight. Premium flat native macOS utility, 16px outer corners, 10px controls, generous but practical spacing. No gradients, glass, blue/purple, huge pills, giant cards, dashboard metrics or decorative art. System sans-serif interface, beautiful Georgia serif transcript text at readable editorial size. Small waveform brand mark and lowercase serif 'vara' in upper left, macOS traffic lights.

Layout: three columns. Narrow 190px olive global sidebar, 280px paper file library, broad 760px ivory transcript editor. Global navigation 'Historik', 'Transskribering' (selected olive lighter rectangle), 'AI pr. app', 'Indstillinger'. Bottom sidebar a modest green dot 'På enheden'. Library header 'Transskribering', search icon, then a clear outlined 'Tilføj filer' button. Three realistic files: selected 'Interview med Anna' with 'Dansk · 24:18 · Klar'; 'Workshop.mp4' with 'Transskriberer…' and small cancel X, indeterminate status not a fabricated percentage; 'Noter.m4a' with 'Venter'. At library bottom quiet note 'Lyd og video'.

Editor header: title 'Interview med Anna', subline 'interview-anna.m4a · Dansk · 24:18', saved check 'Gemt', small explicit copy icon and trash icon with sensible spacing, primary burnt-orange 'Eksportér' dropdown on right. Second compact toolbar: active tab 'Transskription', secondary tab 'AI-bearbejdning', aligned quiet 'Vis original' action, and search icon. Below, small understated metadata 'På enheden · Ordtidskoder' with tiny circled info icon, no unverified provider brands. Spacious editable transcript grouped in speaker paragraphs. Each paragraph has small editable speaker name and understated timestamp in margin:
'Anna' and '00:12'
'Vi skal gøre det nemt at komme i gang. Man skal kunne tilføje en fil og få en tekst, der er rar at arbejde videre med.'
'Mikkel' and '00:24'
'Ja, og når jeg klikker på et ord, vil jeg høre præcis den passage i optagelsen.'
'Anna' and '00:32'
'Originalen bliver bevaret, så vi altid kan gå tilbage og sammenligne.'
Highlight ONLY current word 'klikker' in softly tinted peach within the second paragraph, with tiny thin orange playback caret, not the whole paragraph. Speaker colors understated olive/terracotta, no avatar photos, no individual-word boxes. Tiny contextual helper under transcript 'Klik på et ord for at lytte herfra.' No giant timeline editor.

Bottom fixed player contained inside editor column: subtle top border, small file/audio icon, back 10 sec, circular burnt-orange pause button (currently playing), forward 10 sec, '00:27 / 24:18', a calm minimal waveform seek strip with subtle playhead, a small '1×' speed control. No microphone recording HUD. Preserve whitespace, clear navigation, one dominant action. All text crisp, actual interface quality, coherent reusable components, accessible high contrast. This screenshot depicts timing and speaker features when supported; don't imply every speech model supports them.

## Prompt — import

Use case: ui-mockup. Create a high-fidelity premium Danish desktop app screenshot of Vara's new file-transcription import screen. A single fully visible landscape native desktop window, straight-on, no scene photography. Match existing Vara Warm precision: olive sidebar #2F3527, warm paper #F7F4EE, ivory surfaces #FFFCF7, charcoal text #25251F, muted #636459, dividers #DCD6CA, selected beige #E8E1D5, burnt orange #A84417 only for the main action. Lowercase Georgia serif 'vara' logo with small orange waveform, macOS traffic lights. System sans-serif UI, editorial typography, 16px window radius, subtle 10px controls, restrained spacing, no gradients/glass/blue-purple, no excessive cards or pills.

Same app shell as proposed transcription editor: left 190px global olive sidebar with 'Historik', selected 'Transskribering', 'AI pr. app', 'Indstillinger', green dot 'På enheden' at bottom. Remaining large warm paper workspace. Top header 'Transskribering' and subtitle 'Fra lyd og video til tekst.' Existing library shown subtly below the main import area with small heading 'Seneste' and two plain rows 'Interview med Anna' / 'Dansk · 24:18' and 'Workshop.mp4' / 'Dansk · 12:06', row copy and trash icons, no metrics.

Central import composition is integrated in the page, not a modal. Spacious broad drop area with thin dashed warm-gray border, small outlined audio/document icon, serif heading 'Tilføj lyd eller video', supporting text 'Træk filer hertil, eller vælg dem på computeren.', an outlined 'Vælg filer' control. Below this area show a staged selection of two files in neat compact rows: 'Interview.m4a' with small audio icon and size '18 MB', 'Workshop.mp4' with small video icon and size '84 MB', each a clear remove X at right. Subtle line '2 filer klar'.

Under file list a simple two-column form with persistent labels 'Talemodel' and 'Sprog'. Dropdown values 'Parakeet Redux · På enheden' and 'Automatisk'. A quiet plain-language reassurance 'Filerne behandles på din enhed. Originalerne ændres ikke.' Include one small collapsed disclosure 'Flere indstillinger' for genuinely optional settings, without clutter or list of advanced controls. A clear burnt-orange primary 'Transskriber 2 filer' button below aligned right, with adjacent quiet 'Ryd valg'. Keep model and locality easy to identify before processing. No cloud logos, no invented progress percentages, no 'YouTube URL' field, no system audio recorder, no speaker feature switch that promises unverified capability.

Make the visual task-complete and genuinely simple: importing multiple audio/video files, reviewing selected inputs, local speech default, automatic language, clear explicit start. Same shared visual components as the editor screen, premium warm quiet aesthetic, readable Danish text, flat and implementation-ready, no annotations outside the window.

