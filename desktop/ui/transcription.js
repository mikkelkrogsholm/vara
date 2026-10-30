/* File-transcription screen and controller. Reuses canonical UI primitives. */
(() => {
  "use strict";
  const {
    esc: e,
    button: b,
    iconButton: ib,
    icon,
    field: f,
    notice,
    empty,
  } = Vara.ui;
  const tx = {
    documents: [],
    media: {},
    selected: null,
    importing: true,
    files: [],
    engine: null,
    language: "",
    query: "",
    tab: "transcript",
    original: false,
    drafts: new Map(),
    error: null,
    saving: new Set(),
    aiBusy: new Set(),
    whoBusy: new Set(),
    whoDownload: false,
    whoStatus: null,
  };
  let host, main, saveTimer, refreshTimer;
  const audio = new Audio();
  audio.preload = "metadata";
  const busy = (d) => ["queued", "decoding", "transcribing"].includes(d.stage);
  const stage = {
    queued: "Venter",
    decoding: "Åbner lyd…",
    transcribing: "Transskriberer…",
    ready: "Klar",
    failed: "Mislykkedes",
    cancelled: "Annulleret",
  };
  const stamp = (t) => {
    t = Math.max(0, Math.floor(t || 0));
    return `${Math.floor(t / 60)}:${String(t % 60).padStart(2, "0")}`;
  };
  const doc = () => tx.documents.find((d) => d.id === tx.selected);
  const selectedEngine = () =>
    main?.data.settings.engines.find((x) => x.id === tx.engine);
  const error = () =>
    tx.error
      ? notice(
          tx.error,
          "error",
          b("Luk besked", "tx-dismiss", { variant: "quiet" }),
        )
      : "";
  function fileRow(d) {
    return `<li class="tx-file ${d.id === tx.selected ? "is-selected" : ""}"><button class="tx-file-select" data-action="tx-select" data-id="${e(d.id)}" aria-pressed="${d.id === tx.selected}">${icon("document")}<span><strong>${e(d.title)}</strong><small>${e(d.language || "Automatisk")} · ${d.duration ? stamp(d.duration) + " · " : ""}${stage[d.stage]}</small></span></button>${busy(d) ? ib("close", "Annuller " + d.title, "tx-cancel", `data-id="${e(d.id)}"`) : ib("trash", "Slet " + d.title, "tx-delete", `data-id="${e(d.id)}"`)}</li>`;
  }
  function importView() {
    const engine = selectedEngine(),
      local = engine?.locality === "on_device";
    return `<section class="tx-import page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">Transskribering</h1><p class="muted">Fra lyd og video til tekst.</p></header>${error()}<div class="tx-drop" id="tx-drop" role="region" aria-label="Tilføj lyd eller video"><span class="tx-drop-icon">${icon("document")}</span><h2>Tilføj lyd eller video</h2><p>Træk filer hertil, eller vælg dem på computeren.</p>${b("Vælg filer", "tx-pick")}<p class="tx-format-note">WAV, MP3, FLAC, M4A og video med understøttet lydspor</p></div>${tx.files.length ? `<ul class="tx-staged">${tx.files.map((v, i) => `<li>${icon("document")}<span>${e(v.name)}<small>${(v.bytes / 1e6).toLocaleString("da-DK", { maximumFractionDigits: 1 })} MB</small></span>${ib("close", "Fjern " + v.name, "tx-remove", `data-id="${i}"`)}</li>`).join("")}</ul><p class="muted">${tx.files.length} ${tx.files.length === 1 ? "fil klar" : "filer klar"}</p>` : ""}<div class="tx-import-options">${Vara.views.modelPicker(main, "tx-engine", "Talemodel", "speech", tx.engine)}${f(
      {
        id: "tx-language",
        label: "Sprog",
        value: tx.language,
        options: [
          ["", "Automatisk"],
          ["da", "Dansk"],
          ["en", "Engelsk"],
          ["sv", "Svensk"],
          ["no", "Norsk"],
          ["de", "Tysk"],
        ],
        attrs: engine?.provider === "local_parakeet" ? "disabled" : "",
      },
    )}</div><p class="muted tx-locality">${local ? "Filerne behandles på din enhed." : `Lyd sendes til ${e(engine?.label || "den valgte udbyder")}.`} Originalerne ændres ikke.</p><p class="muted tx-storage">Vara gemmer en lydkopi til afspilning og genprøvning. Slet dokumentet for at fjerne kopien; annullering rydder arbejdslyden. Op til 8 timer pr. fil.</p>${engine?.provider === "local_parakeet" ? '<p class="muted tx-format-note">Parakeet registrerer automatisk sproget.</p>' : ""}<div class="tx-start actions">${tx.files.length ? b("Ryd valg", "tx-clear", { variant: "quiet" }) : ""}${b(tx.files.length > 1 ? `Transskriber ${tx.files.length} filer` : "Transskriber", "tx-start", { variant: "primary", attrs: tx.files.length ? "" : "disabled" })}</div>${tx.documents.length ? `<section class="tx-recent"><header><h2>Seneste</h2>${f({ id: "tx-query", label: "Søg i transskriptioner", type: "search", value: tx.query })}</header><ul>${filtered().map(fileRow).join("")}</ul></section>` : ""}</section>`;
  }
  function filtered() {
    const q = tx.query.toLocaleLowerCase();
    return tx.documents.filter(
      (d) =>
        !q ||
        `${d.title} ${d.transcript?.text || ""} ${d.edited_text || ""}`
          .toLocaleLowerCase()
          .includes(q),
    );
  }
  function whoControls(d) {
    const running = d.who_busy || tx.whoBusy.has(d.id);
    const turns = d.diarization?.turns || [];
    const count = new Set(turns.filter((t) => t.speaker !== "who-uncertain").map((t) => t.speaker)).size;
    const uncertain = turns.filter((t) => t.speaker === "who-uncertain").sort((a, b) => a.start - b.start);
    let seconds = 0, end = 0;
    for (const t of uncertain) { seconds += Math.max(0, t.end - Math.max(end, t.start)); end = Math.max(end, t.end); }
    return `${b(running ? "Finder talere…" : d.diarization ? "Taleropdeling · Who" : "Find talere", "tx-who-open", { variant: "quiet" })}${running ? b("Annuller taleropdeling", "tx-who-cancel", { variant: "quiet" }) : ""}${d.diarization ? `<span class="muted">${count} ${count === 1 ? "taler" : "talere"}${uncertain.length ? ` · ${uncertain.length} usikre intervaller (${Math.round(seconds)} sek.)` : ""} · på enheden</span>` : ""}${d.diarization_error ? notice(d.diarization_error, "error") : ""}`;
  }
  function whoTurns(d) {
    if (!d.diarization) return "";
    return `<details class="tx-who-turns"><summary>Talerintervaller · Who</summary><p class="muted">Stemmer navngives Taler 1, Taler 2 osv. Overlap og usikre ord kan stå uden én taler. ${d.transcript.timing !== "word" ? "Uden ordtidskoder kan vi ikke fordele teksten præcist mellem talerne." : "Rettelser ændrer ikke originalens talermarkeringer."}</p>${d.diarization.turns.map((t) => `<div class="tx-who-turn">${t.speaker === "who-uncertain" ? '<span>Usikker taler</span>' : b(d.speakers[t.speaker] || t.speaker, "tx-speaker", { variant: "quiet", attrs: `data-id="${e(t.speaker)}"` })}${b(`${stamp(t.start)}–${stamp(t.end)}`, "tx-seek", { variant: "quiet", attrs: `data-time="${t.start}"` })}</div>`).join("")}</details>`;
  }
  function whoDialog() {
    const status = tx.whoStatus,
      d = doc(),
      running = d?.who_busy || tx.whoBusy.has(d?.id);
    if (!d) return;
    host.modal(
      "Taleropdeling med Who",
      `<p>Find hvem der taler hvornår. Lyden behandles lokalt; transskriptionen, rettelser og originalfil bevares.</p><p class="muted">Who by <a href="https://desertant.com" data-action="tx-who-link" data-id="vendor">Desert Ant Labs</a> · model v0.3.4. Vara anvender en tilpasset SDK-version. Op til fire stemmer pr. fem minutter; stemmer matches på tværs af længere optagelser. Modellen kan forveksle stemmer, især ved overlap, støj og korte indspark. Korte eller stille klip kan give ingen talere.</p><p class="muted">SDK’et sender en installations-id samt model-, platform- og appoplysninger til Desert Ant Labs’ licenssystem. Lyd og tekst sendes ikke. <a href="https://license.desertant.com/1.0" data-action="tx-who-link" data-id="license">Licens og brugstælling</a>.</p>${status?.error ? notice(status.error, "error") : ""}${tx.whoDownload ? `<p role="status">Henter og klargør Who…</p>${b("Annuller download", "tx-who-download-cancel")}` : running ? `<p role="status">Finder talere. Du kan fortsætte med resten af Vara.</p>${b("Annuller", "tx-who-cancel")}` : status?.ready ? `<label for="tx-who-count">Antal talere</label><select id="tx-who-count"><option value="">Automatisk</option>${Array.from({length: 16}, (_, i) => `<option value="${i + 1}" ${d.diarization?.known_speakers?.count === i + 1 ? "selected" : ""}>${i + 1}</option>`).join("")}</select><p class="muted">Angiv kun antal, hvis du kender det. Passager uden et entydigt match vises som usikre.</p>${b(d.diarization ? "Find talere igen" : "Find talere", "tx-who-run", { variant: "primary" })}` : status?.available ? `<p>Hent modellen før første brug (omkring 81 MB på Mac). Download og brug sker under den viste licens.</p>${b("Hent Who", "tx-who-download", { variant: "primary" })}` : ""}`,
    );
  }
  function transcriptText(d) {
    const t = d.speaker_transcript || d.transcript;
    if (!t)
      return empty(
        stage[d.stage],
        d.error ||
          (d.stage === "transcribing"
            ? `${stamp(d.completed_seconds)} af ${stamp(d.duration)} lyd behandlet.`
            : "Du kan bruge resten af Vara imens."),
        ["failed", "cancelled"].includes(d.stage)
          ? b("Prøv igen", "tx-retry", {
              variant: "primary",
              attrs: `data-id="${e(d.id)}"`,
            })
          : b("Annuller", "tx-cancel", { attrs: `data-id="${e(d.id)}"` }),
      );
    const draft = tx.drafts.get(d.id);
    const edited = draft?.text ?? d.edited_text;
    if (!tx.original && edited !== null && edited !== undefined)
      return `<div class="tx-edit-notice">${notice("Teksten er rettet og kræver ny tidsjustering. Afspilning følger den bevarede original.", "info")}</div><textarea class="tx-editor-text" id="tx-editor-text" data-document="${e(d.id)}" aria-label="Redigér transskription" spellcheck="true" dir="auto">${e(edited)}</textarea>`;
    return `<div class="tx-segments">${t.segments.map((s) => `<section class="tx-segment"><div class="tx-segment-meta">${s.speaker ? `<button data-action="tx-speaker" data-id="${e(s.speaker)}" aria-label="Omdøb taler ${e(d.speakers[s.speaker] || s.speaker)}">${e(d.speakers[s.speaker] || s.speaker)}</button>` : ""}${s.start !== null ? `<button class="tx-segment-time" data-action="tx-seek" data-time="${s.start}" aria-label="Afspil fra ${stamp(s.start)}">${stamp(s.start)}</button>` : ""}</div><p class="tx-prose" dir="auto">${s.words.length ? s.words.map((w) => `<button class="tx-word" tabindex="-1" data-action="tx-seek" data-word="${e(w.id)}" data-time="${w.start}" data-end="${w.end}">${e(w.text)}</button>`).join(" ") : e(s.text)}</p></section>`).join("")}</div>`;
  }
  function editor() {
    const d = doc();
    if (!d) return importView();
    const timed = d.transcript?.timing || "none",
      draft = tx.drafts.get(d.id),
      hasMedia = !!tx.media[d.id];
    return `<section class="tx-library"><header class="pane-toolbar"><h1 id="view-title" tabindex="-1">Transskribering</h1></header><div class="tx-library-controls">${b("Tilføj filer", "tx-import", { glyph: "plus" })}${f({ id: "tx-query", label: "Søg i transskriptioner", type: "search", value: tx.query })}</div><ul class="tx-library-list">${filtered().map(fileRow).join("") || '<li class="muted">Ingen match</li>'}</ul><p class="tx-library-note">Lyd og video · Lokale afspilningskopier</p></section><section class="tx-detail" aria-label="Valgt transskription"><header class="tx-detail-header"><div><button class="tx-title" data-action="tx-rename" aria-label="Omdøb ${e(d.title)}"><h2>${e(d.title)}</h2>${icon("edit")}</button><p class="muted">${e(d.source_name)} · ${e(d.language || "Automatisk")} ${d.duration ? " · " + stamp(d.duration) : ""}</p></div><div class="actions"><span id="tx-save-status" class="muted" role="status">${tx.saving.has(d.id) ? "Gemmer…" : draft || (d.transcript && d.stage === "failed") ? "Ikke gemt" : "Gemt"}</span>${d.transcript ? ib("copy", "Kopiér transskription", "tx-copy") : ""}${ib("trash", "Slet transskription", "tx-delete", `data-id="${e(d.id)}"`)}${b("Eksportér", "tx-export", { variant: "primary", attrs: d.transcript ? "" : "disabled" })}</div></header><nav class="tx-tabs" aria-label="Dokumentvisning"><button data-action="tx-tab" data-id="transcript" class="${tx.tab === "transcript" ? "is-current" : ""}" aria-pressed="${tx.tab === "transcript"}">Transskription</button><button data-action="tx-tab" data-id="ai" class="${tx.tab === "ai" ? "is-current" : ""}" aria-pressed="${tx.tab === "ai"}">AI-bearbejdning</button>${d.transcript ? b(tx.original ? "Vis rettet tekst" : "Vis original", "tx-original", { variant: "quiet" }) : ""}</nav><div class="tx-reading"><div id="tx-errors">${error()}</div>${d.error && d.transcript ? notice(d.error, "error", b("Gem resultat igen", "tx-persist")) : ""}<p class="tx-provenance"><span class="status-dot"></span>${e(d.engine.locality === "on_device" ? "På enheden" : d.engine.label)} · ${timed === "word" ? "Ordtidskoder" : timed === "segment" ? "Tidskoder pr. afsnit" : "Ingen tidskoder"}</p>${d.engine.provider === "eleven_labs" ? '<p class="muted tx-format-note">Talere genkendes pr. lydstykke på op til fem minutter. Navne forbindes ikke automatisk mellem lydstykker.</p>' : ""}${d.transcript && tx.tab === "transcript" ? `<div id="tx-who-tools" class="tx-text-tools">${whoControls(d)}</div>${whoTurns(d)}<div class="tx-text-tools">${b(draft || d.edited_text !== null ? "Gem igen" : "Redigér tekst", draft || d.edited_text !== null ? "tx-save" : "tx-edit", { variant: "quiet" })}${draft || d.edited_text !== null ? b("Gendan original", "tx-reset", { variant: "quiet" }) : ""}${timed !== "none" ? b("Afspil markering", "tx-selection", { variant: "quiet" }) : ""}</div>` : ""}${tx.tab === "ai" ? aiView(d) : transcriptText(d)}${d.transcript && tx.tab === "transcript" ? '<p class="tx-reading-help muted">Klik på et ord eller en tidskode for at lytte herfra. Originalen bevares, når du retter teksten.</p>' : ""}</div><footer class="tx-player"><div class="tx-player-actions">${b("−10 s", "tx-rewind", { variant: "quiet", attrs: hasMedia ? "" : "disabled" })}${b(audio.paused ? "Afspil" : "Pause", "tx-play", { variant: "primary", attrs: hasMedia ? "" : "disabled" })}${b("+10 s", "tx-forward", { variant: "quiet", attrs: hasMedia ? "" : "disabled" })}</div><span id="tx-time" class="tx-time">${stamp(audio.currentTime)} / ${stamp(d.duration)}</span><div class="tx-seek-track"><div class="tx-waveform" aria-hidden="true">${(d.waveform || []).map((v) => `<i style="height:${Math.max(2, Math.min(1, v) * 30)}px"></i>`).join("")}</div><input id="tx-position" type="range" min="0" max="${d.duration || 1}" step="0.1" value="${audio.currentTime || 0}" aria-label="Afspilningsposition" ${hasMedia ? "" : "disabled"}></div><label class="sr-only" for="tx-speed">Afspilningshastighed</label><select id="tx-speed">${[0.75, 1, 1.25, 1.5, 2].map((n) => `<option ${audio.playbackRate === n ? "selected" : ""} value="${n}">${n}×</option>`).join("")}</select>${!hasMedia ? '<p class="tx-missing">Afspilningskopien mangler. Teksten er bevaret.</p>' : ""}</footer></section>`;
  }
  function aiView(d) {
    return `<div class="tx-ai"><h3>Bearbejd teksten</h3><p class="muted">Oprydning, resumé eller dine egne instruktioner. Den tidskodede original ændres ikke.</p>${Vara.views.modelPicker(main, "tx-ai-engine", "Tekstmodel", "text", tx.aiEngine || d.ai_engine?.id || main.data.settings.general_engine)}${f({ id: "tx-ai-prompt", label: "Instruktioner til AI", rows: true, value: tx.aiPrompt || "Lav et kort resumé af transskriptionen." })}<p class="muted" id="tx-ai-destination">${aiDestination()}</p>${b(tx.aiBusy.has(d.id) ? "Bearbejder…" : "Bearbejd tekst", "tx-ai", { variant: "primary", attrs: !d.transcript || tx.aiBusy.has(d.id) ? "disabled" : "" })}${tx.aiBusy.has(d.id) ? b("Annuller", "tx-ai-cancel") : ""}${d.ai_text ? `<article class="tx-ai-result"><h3>AI-resultat</h3><p dir="auto">${e(d.ai_text)}</p>${b("Kopiér AI-tekst", "tx-copy-ai", { glyph: "copy" })}</article>` : ""}</div>`;
  }
  function aiDestination() {
    const id =
      document.getElementById("tx-ai-engine")?.value ||
      tx.aiEngine ||
      doc()?.ai_engine?.id ||
      main.data.settings.general_engine;
    const engine = main.data.settings.engines.find((v) => v.id === id);
    return engine?.locality === "on_device"
      ? "Teksten behandles på din enhed."
      : `Teksten sendes til ${engine?.label || "din valgte forbindelse"}.`;
  }
  async function reload() {
    const data = await host.api("file_list");
    tx.documents = data.documents;
    tx.media = data.media;
    if (tx.selected && !doc()) {
      tx.selected = null;
      tx.importing = true;
      audio.pause();
    }
    draw();
  }
  function draw() {
    if (main?.page === "transcription") host.render();
  }
  function setAudio() {
    const d = doc(),
      path = d && tx.media[d.id];
    if (!path) {
      audio.pause();
      audio.removeAttribute("src");
      return;
    }
    const src = window.__TAURI__.core.convertFileSrc(path);
    if (audio.src !== src) {
      audio.pause();
      audio.src = src;
      audio.load();
    }
  }
  const pendingSaves = new Map();
  async function flush(id = tx.selected) {
    clearTimeout(saveTimer);
    if (pendingSaves.has(id)) {
      await pendingSaves.get(id);
      return flush(id);
    }
    const draft = tx.drafts.get(id);
    if (!draft) return;
    const save = (async () => {
      tx.saving.add(id);
      document.getElementById("tx-save-status")?.replaceChildren("Gemmer…");
      try {
        const result = await host.api("file_save", {
          id,
          text: draft.text,
          revision: draft.revision,
        });
        const current = tx.drafts.get(id),
          d = tx.documents.find((d) => d.id === id);
        if (d) {
          d.revision = result.revision;
          d.edited_text = draft.text === d.transcript.text ? null : draft.text;
        }
        if (current === draft) tx.drafts.delete(id);
        else if (current) current.revision = result.revision;
      } catch (err) {
        tx.error = String(err.message || err);
        throw err;
      } finally {
        tx.saving.delete(id);
        draw();
      }
    })();
    pendingSaves.set(id, save);
    try {
      await save;
    } finally {
      pendingSaves.delete(id);
    }
  }
  async function seek(time) {
    if (main.data.session?.stage === "recording")
      throw Error("Afslut dikteringen, før du afspiller lyd.");
    if (!tx.media[tx.selected]) throw Error("Afspilningskopien mangler.");
    setAudio();
    audio.currentTime = Math.max(0, Math.min(time, doc().duration));
    tx.stopAt = null;
    await audio.play();
    draw();
  }
  async function action(button) {
    const a = button.dataset.action,
      id = button.dataset.id,
      d = doc();
    tx.error = null;
    switch (a) {
      case "tx-who-link":
        await host.api("file_who_link", { target: id });
        break;
      case "tx-who-open":
        tx.whoStatus = await host.api("file_who_status");
        whoDialog();
        break;
      case "tx-who-download":
        tx.whoDownload = true;
        whoDialog();
        try {
          await host.api("file_who_download", { acknowledge_license: true });
          tx.whoStatus = await host.api("file_who_status");
        } catch (error) {
          tx.whoStatus = {
            ...tx.whoStatus,
            error: String(error.message || error),
          };
        } finally {
          const stillOpen = document.querySelector(
            '#modal[open] [data-action="tx-who-download-cancel"]',
          );
          tx.whoDownload = false;
          if (stillOpen && main.page === "transcription") whoDialog();
        }
        break;
      case "tx-who-download-cancel":
        await host.api("file_who_cancel", { download: true });
        break;
      case "tx-who-cancel":
        await host.api("file_who_cancel", { id: d.id });
        host.closeModal();
        break;
      case "tx-who-run": {
        const selectedCount = document.getElementById("tx-who-count")?.value;
        const knownCount = selectedCount ? Number(selectedCount) : null;
        await flush();
        const target = d.id;
        tx.whoBusy.add(target);
        host.closeModal();
        draw();
        let failure = null;
        try {
          await host.api("file_who", { id: target, known_speaker_count: knownCount });
          if (tx.selected === target) tx.original = true;
        } catch (error) {
          failure = String(error.message || error);
        } finally {
          tx.whoBusy.delete(target);
          await reload();
        }
        // Persisted Who failures have their own single contextual notice.
        // Cancellation retains the document without an error toast.
        if (
          failure &&
          !failure.includes("Annulleret") &&
          !doc()?.diarization_error
        ) {
          tx.error = failure;
          draw();
        }
        break;
      }
      case "tx-pick": {
        const files = await host.api("file_pick");
        for (const file of files)
          if (!tx.files.some((x) => x.path === file.path)) tx.files.push(file);
        break;
      }
      case "tx-import":
        await flush();
        audio.pause();
        tx.importing = true;
        break;
      case "tx-remove":
        tx.files.splice(Number(id), 1);
        break;
      case "tx-clear":
        tx.files = [];
        break;
      case "tx-start": {
        const ids = await host.api("file_add", {
          paths: tx.files.map((v) => v.path),
          engine: tx.engine,
          language:
            selectedEngine()?.provider === "local_parakeet" ? "" : tx.language,
        });
        tx.files = [];
        tx.selected = ids[0];
        tx.importing = false;
        await reload();
        break;
      }
      case "tx-select":
        await flush();
        tx.selected = id;
        tx.importing = false;
        tx.original = false;
        tx.tab = "transcript";
        tx.aiEngine = doc()?.ai_engine?.id || main.data.settings.general_engine;
        setAudio();
        break;
      case "tx-dismiss":
        tx.error = null;
        break;
      case "tx-cancel":
        await host.api("file_cancel", { id });
        await reload();
        break;
      case "tx-retry":
        await host.api("file_retry", { id });
        await reload();
        break;
      case "tx-delete":
        host.modal(
          "Slet transskription?",
          `<p>Tekst, rettelser og Varas lydkopi slettes. Din originalfil ændres ikke.</p><div class="actions">${b("Slet transskription", "tx-delete-confirm", { variant: "destructive", attrs: `data-id="${e(id)}"` })}${b("Behold", "close-modal")}</div>`,
        );
        break;
      case "tx-delete-confirm":
        if (id === tx.selected) {
          audio.pause();
          audio.removeAttribute("src");
          audio.load();
        }
        await host.api("file_delete", { id });
        tx.drafts.delete(id);
        host.closeModal();
        await reload();
        break;
      case "tx-tab":
        await flush();
        tx.tab = id;
        break;
      case "tx-original":
        tx.original = !tx.original;
        break;
      case "tx-edit":
        tx.original = false;
        tx.drafts.set(d.id, {
          text: d.edited_text ?? d.transcript.text,
          revision: d.revision,
        });
        draw();
        document.getElementById("tx-editor-text")?.focus();
        return;
      case "tx-persist":
        await host.api("file_persist", { id: d.id });
        await reload();
        break;
      case "tx-save":
        await flush();
        break;
      case "tx-reset":
        tx.drafts.set(d.id, { text: d.transcript.text, revision: d.revision });
        await flush();
        break;
      case "tx-copy":
        await host.api("file_copy", {
          id: d.id,
          original: tx.original,
          draft: !tx.original ? tx.drafts.get(d.id)?.text : undefined,
        });
        host.toast("Teksten er kopieret.");
        break;
      case "tx-copy-ai":
        await host.api("file_copy", { id: d.id, ai: true });
        host.toast("AI-teksten er kopieret.");
        break;
      case "tx-rename":
        host.modal(
          "Omdøb transskription",
          f({ id: "tx-title-input", label: "Titel", value: d.title }) +
            b("Gem titel", "tx-rename-save", { variant: "primary" }),
        );
        break;
      case "tx-rename-save":
        await host.api("file_rename", {
          id: d.id,
          title: document.getElementById("tx-title-input").value,
        });
        host.closeModal();
        await reload();
        break;
      case "tx-speaker":
        tx.speaker = id;
        host.modal(
          "Omdøb taler",
          f({
            id: "tx-speaker-input",
            label: "Navn",
            value: d.speakers[id] || id,
          }) + b("Gem navn", "tx-speaker-save", { variant: "primary" }),
        );
        break;
      case "tx-speaker-save":
        await host.api("file_speaker", {
          id: d.id,
          speaker: tx.speaker,
          name: document.getElementById("tx-speaker-input").value,
        });
        host.closeModal();
        await reload();
        break;
      case "tx-export":
        host.modal(
          "Eksportér transskription",
          f({
            id: "tx-export-format",
            label: "Format",
            value: "txt",
            options: [
              ["txt", "Tekst (.txt)"],
              ["md", "Markdown (.md)"],
              ["json", "Dokument med orddata (.json)"],
              ["srt", "Undertekster (.srt)"],
              ["vtt", "Web-undertekster (.vtt)"],
            ],
          }) +
            f({
              id: "tx-export-original",
              label: "Tekstversion",
              value: tx.original ? "original" : "edited",
              options: [
                ["edited", "Aktuel tekst"],
                ["original", "Tidskodet original"],
                ...(d.ai_text ? [["ai", "AI-resultat"]] : []),
              ],
            }) +
            '<p class="muted">Undertekster kræver tidsdata. Rettet tekst skal tidsjusteres igen; vælg originalen til undertekster.</p>' +
            b("Gem fil", "tx-export-save", { variant: "primary" }),
        );
        break;
      case "tx-export-save": {
        const saved = await host.api("file_export", {
          id: d.id,
          format: document.getElementById("tx-export-format").value,
          original:
            document.getElementById("tx-export-original").value === "original",
          ai: document.getElementById("tx-export-original").value === "ai",
          draft:
            document.getElementById("tx-export-original").value === "edited"
              ? tx.drafts.get(d.id)?.text
              : undefined,
        });
        if (saved !== false) {
          host.closeModal();
          host.toast("Eksporten er gemt.");
        }
        break;
      }
      case "tx-play":
        if (audio.paused) await seek(audio.currentTime);
        else audio.pause();
        break;
      case "tx-rewind":
        await seek(audio.currentTime - 10);
        break;
      case "tx-forward":
        await seek(audio.currentTime + 10);
        break;
      case "tx-seek":
        if (!window.getSelection()?.isCollapsed) return;
        await seek(Number(button.dataset.time));
        break;
      case "tx-selection": {
        const selection = window.getSelection();
        if (!selection || selection.isCollapsed)
          throw Error("Markér først en passage i den tidskodede original.");
        const nodes = [...document.querySelectorAll(".tx-word")].filter((n) =>
          selection.containsNode(n, true),
        );
        if (!nodes.length)
          throw Error(
            "Markering kræver ordtidskoder. Klik på en afsnitstid for segmenter.",
          );
        await seek(Number(nodes[0].dataset.time));
        tx.stopAt = Number(nodes.at(-1).dataset.end);
        break;
      }
      case "tx-ai": {
        await flush();
        const engine = document.getElementById("tx-ai-engine").value,
          prompt = document.getElementById("tx-ai-prompt").value;
        tx.aiPrompt = prompt;
        tx.aiBusy.add(d.id);
        draw();
        try {
          await host.api("file_ai", { id: d.id, engine, prompt });
          await reload();
        } finally {
          tx.aiBusy.delete(d.id);
        }
        break;
      }
      case "tx-ai-cancel":
        await host.api("file_ai_cancel", { id: d.id });
        break;
    }
    draw();
  }
  function init(controller, s) {
    host = controller;
    main = s;
    tx.engine = s.data.settings.speech_engine;
    reload().catch((err) => {
      tx.error = String(err);
      draw();
    });
    window.__TAURI__.event.listen("vara:files", () => {
      clearTimeout(refreshTimer);
      refreshTimer = setTimeout(
        () =>
          reload().catch((err) => {
            tx.error = String(err);
            draw();
          }),
        200,
      );
    });
    window.__TAURI__.event.listen("vara:file-error", (ev) => {
      tx.error = ev.payload;
      draw();
    });
    window.__TAURI__.event.listen("tauri://drag-drop", async (ev) => {
      if (main.page !== "transcription") return;
      tx.importing = true;
      for (const path of ev.payload.paths || [])
        if (!tx.files.some((f) => f.path === path))
          tx.files.push({ path, name: path.split(/[\\/]/).at(-1), bytes: 0 });
      draw();
    });
    document.addEventListener("keydown", (ev) => {
      if (
        !ev.target.matches?.(".tx-word") ||
        ev.shiftKey ||
        !["ArrowLeft", "ArrowRight"].includes(ev.key)
      )
        return;
      const words = [...document.querySelectorAll(".tx-word")],
        at = words.indexOf(ev.target),
        next =
          words[
            Math.max(
              0,
              Math.min(
                words.length - 1,
                at + (ev.key === "ArrowRight" ? 1 : -1),
              ),
            )
          ];
      ev.preventDefault();
      words.forEach((n) => (n.tabIndex = -1));
      next.tabIndex = 0;
      next.focus();
    });
    document.addEventListener("input", (ev) => {
      if (ev.target.id === "tx-editor-text") {
        const d = doc(),
          previous = tx.drafts.get(d.id);
        tx.drafts.set(d.id, {
          text: ev.target.value,
          revision: previous?.revision ?? d.revision,
        });
        document.getElementById("tx-save-status").textContent = "Ikke gemt";
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => flush(d.id).catch(() => {}), 600);
      }
      if (ev.target.id === "tx-query") {
        tx.query = ev.target.value;
        draw();
      }
      if (ev.target.id === "tx-position") {
        audio.currentTime = Number(ev.target.value);
        tx.stopAt = null;
      }
      if (ev.target.id === "tx-ai-prompt") tx.aiPrompt = ev.target.value;
    });
    document.addEventListener("change", (ev) => {
      if (ev.target.id === "tx-engine") {
        tx.engine = ev.target.value;
        draw();
      }
      if (ev.target.id === "tx-language") tx.language = ev.target.value;
      if (ev.target.id === "tx-speed")
        audio.playbackRate = Number(ev.target.value);
      if (ev.target.id === "tx-ai-engine") {
        tx.aiEngine = ev.target.value;
        document.getElementById("tx-ai-destination").textContent =
          aiDestination();
        draw();
      }
    });
    // Native media events also occur at a passage boundary or recording start.
    // Update only the player label so editor focus/selection stays intact.
    const updatePlaybackLabel = () => {
      const button = document.querySelector('[data-action="tx-play"]');
      if (button) button.textContent = audio.paused ? "Afspil" : "Pause";
    };
    audio.addEventListener("play", updatePlaybackLabel);
    audio.addEventListener("pause", updatePlaybackLabel);
    audio.addEventListener("timeupdate", () => {
      if (tx.stopAt && audio.currentTime >= tx.stopAt) {
        audio.pause();
        tx.stopAt = null;
      }
      const position = document.getElementById("tx-position"),
        time = document.getElementById("tx-time");
      if (position) position.value = audio.currentTime;
      if (time)
        time.textContent = `${stamp(audio.currentTime)} / ${stamp(doc()?.duration)}`;
      document
        .querySelectorAll(".tx-word")
        .forEach((n) =>
          n.classList.toggle(
            "is-playing",
            audio.currentTime >= Number(n.dataset.time) &&
              audio.currentTime < Number(n.dataset.end),
          ),
        );
    });
    audio.addEventListener("error", () => {
      if (audio.getAttribute("src")) {
        tx.error = "Afspilningskopien kunne ikke læses. Teksten er bevaret.";
        draw();
      }
    });
    audio.addEventListener("ended", draw);
  }
  Vara.transcription = {
    init,
    onSession(session) {
      if (session?.stage === "recording") audio.pause();
    },
    async action(button) {
      try {
        return await action(button);
      } catch (error) {
        tx.error = String(error.message || error);
        draw();
        throw error;
      }
    },
    flush,
    retainEditor() {
      const field = document.getElementById("tx-editor-text"),
        d = doc();
      return (
        main?.page === "transcription" &&
        !tx.importing &&
        tx.tab === "transcript" &&
        !tx.original &&
        field &&
        field.dataset.document === d?.id &&
        field.value ===
          (tx.drafts.get(d.id)?.text ?? d.edited_text ?? d.transcript?.text)
      );
    },
    refreshEditorStatus() {
      const d = doc();
      if (!d) return;
      document
        .getElementById("tx-save-status")
        ?.replaceChildren(
          tx.saving.has(d.id)
            ? "Gemmer…"
            : tx.drafts.has(d.id)
              ? "Ikke gemt"
              : "Gemt",
        );
      const who = document.getElementById("tx-who-tools");
      if (who) who.innerHTML = whoControls(d);
      const errors = document.getElementById("tx-errors");
      if (errors) errors.innerHTML = error();
      const list = document.querySelector(".tx-library-list");
      if (list) list.innerHTML = filtered().map(fileRow).join("");
      const title = document.querySelector(".tx-title h2");
      if (title) title.textContent = d.title;
      const hasMedia = !!tx.media[d.id];
      document
        .querySelectorAll(
          '[data-action="tx-play"],[data-action="tx-rewind"],[data-action="tx-forward"],#tx-position',
        )
        .forEach((n) => (n.disabled = !hasMedia));
      if (!hasMedia && !document.querySelector(".tx-missing")) {
        const p = document.createElement("p");
        p.className = "tx-missing";
        p.textContent = "Afspilningskopien mangler. Teksten er bevaret.";
        document.querySelector(".tx-player")?.append(p);
      }
    },
    render(s) {
      main = s;
      tx.engine ??= s.data.settings.speech_engine;
      queueMicrotask(() => {
        const words = document.querySelectorAll(".tx-word");
        if (words.length && !Array.from(words).some((n) => n.tabIndex === 0))
          words[0].tabIndex = 0;
      });
      return `<div class="transcription-screen ${tx.importing ? "is-import" : "is-editor"}">${tx.importing ? importView() : editor()}</div>`;
    },
  };
  Vara.views.transcription = (s) => Vara.transcription.render(s);
})();
