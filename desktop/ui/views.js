/* Live screen compositions. Shared pure components come from DESIGN.md. */
(() => {
  "use strict";
  const {
    esc: e,
    icon,
    mark,
    button: b,
    iconButton: ib,
    notice,
    field: f,
    toggle,
    section,
    empty,
  } = Vara.ui;
  const {
    appArtwork: appIcon,
    resultActions,
    binding,
    shortcut,
    permissionRow,
    disclosure,
  } = Vara.desktopUi;
  const date = (r) =>
    new Date(r.created_at * 1000).toLocaleString("da-DK", {
      day: "numeric",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
    }) + (r.duration_secs ? " · " + Math.round(r.duration_secs) + " sek." : "");
  const finished = (r) => r.processed ?? r.original;
  const loc = {
    on_device: "På denne enhed",
    lan: "På eget netværk",
    cloud: "Hos valgt udbyder",
    unknown: "Ekstern eller ukendt placering",
  };
  const phase = {
    recording: "Lytter",
    recognizing: "Transskriberer",
    processing: "Tilpasser teksten",
    committing: "Gemmer",
  };
  function apps(s) {
    const out = new Map();
    s.data.history.forEach((r) => out.set(r.origin.app_id, r.origin.app_name));
    Object.values(s.data.settings.app_rules).forEach((r) =>
      out.set(r.app_id, r.app_name),
    );
    return [...out];
  }
  function sidebar(s) {
    const p = s.data.settings;
    const local =
      p.engines.find((e) => e.id === p.speech_engine)?.locality === "on_device";
    const speech = p.engines.find((e) => e.id === p.speech_engine);
    const installed = !local || s.data.models[p.speech_engine];
    const micReady = ["authorized", "system"].includes(
      s.data.permissions.microphone,
    );
    const textEngine =
      s.data.session?.plan?.text?.engine ||
      (p.general_ai_enabled
        ? p.engines.find((e) => e.id === p.general_engine)
        : null);
    return `<div class="brand">${mark()}<span>vara</span></div><nav>${[
      ["history", "history", "Historik"],
      ["transcription", "document", "Transskribering"],
      ["apps", "apps", "AI pr. app"],
      ["settings", "settings", "Indstillinger"],
    ]
      .map(
        ([page, glyph, label]) =>
          `<button class="nav-item ${s.page === page ? "is-current" : ""}" data-action="navigate" data-page="${page}" ${s.page === page ? 'aria-current="page"' : ""}>${icon(glyph)}<span>${label}</span></button>`,
      )
      .join(
        "",
      )}</nav><div class="sidebar-footer"><div class="sidebar-status"><span class="status-dot ${installed && micReady && local ? "" : "is-unready"}"></span><span>${s.page === "transcription" ? "Diktering: " : "Lyd: "}${local ? "på enheden" : "cloud valgt"}</span><kbd>${e(binding(p.shortcut))}</kbd></div><p>${s.data.next_choice && s.data.next_choice !== "default" ? "Næste diktering: " + (s.data.next_choice === "direct" ? "direkte" : "generel AI") + "<br>" : ""}${p.activation === "hold" ? "Hold, tal og slip." : "Tryk for start / stop."}</p><p>${s.page === "transcription" ? "Genvejen starter diktering.<br>Filens model vælges i fanen." : local ? "Lokal transskription.<br>Din lyd forlader ikke enheden." : "Lyd behandles hos din valgte udbyder."}</p>${textEngine ? `<p>Tekst: ${e(loc[textEngine.locality])}</p>` : Object.values(p.app_rules).some((a) => a.enabled) ? "<p>AI efter dine appregler.</p>" : ""}${!installed || !micReady ? b(!installed ? "Hent talemodellen" : "Færdiggør opsætning", "setup", { variant: "quiet", cls: "sidebar-setup" }) : ""}<div class="sidebar-dictation-controls">${b(s.data.session?.stage === "recording" ? "Afslut" : "Diktér", s.data.session?.stage === "recording" ? "stop" : "start", { variant: "quiet", glyph: s.data.session?.stage === "recording" ? "stop" : "mic", attrs: s.data.session && s.data.session.stage !== "recording" ? "disabled" : "" })}${ib("more", "Behandling for næste diktering", "choose-processing")}</div></div>`;
  }
  function messages(s) {
    return `${s.uiError?.scope === "global" ? notice(s.uiError.message, "error", b("Luk besked", "dismiss-ui-error", { variant: "quiet" })) : ""}${s.data.message ? `<div class="inline-state">${icon("warning")}<p>${e(s.data.message)}</p>${ib("close", "Luk besked", "dismiss_message")}</div>` : ""}${s.data.session ? `<div class="inline-state session-state" role="status">${icon("mic")}<p>${e(phase[s.data.session.stage] || "Diktering")} · ${e(s.data.session.origin.app_name)}</p>${s.data.session.stage === "recording" ? b("Afslut", "stop", { glyph: "stop" }) : ""}${b("Annuller", "cancel", { variant: "quiet" })}</div>` : ""}${s.data.unsaved ? `<section class="pending-save"><h2>Teksten er endnu ikke gemt</h2><p class="transcript is-long">${e(finished(s.data.unsaved))}</p><div class="actions">${b("Kopiér", "copy-unsaved", { variant: "primary" })}${b("Prøv at gemme igen", "save_unsaved")}${b("Kassér", "discard-unsaved", { variant: "quiet" })}</div></section>` : ""}`;
  }
  function recovery(s) {
    return s.data.recoveries
      .map(
        (r) =>
          `<section class="recovery-task"><h3>${icon("warning")}Transskription fra ${e(r.origin.app_name)} fejlede</h3><p>Optagelsen udløber ${e(new Date(r.expires_at * 1000).toLocaleString("da-DK"))}. Fristen forlænges ikke ved genprøvning.</p><div class="actions">${b("Prøv igen", "retry", { attrs: `data-id="${e(r.id)}"` })}${b("Slet optagelse", "delete-recovery", { variant: "quiet", attrs: `data-id="${e(r.id)}"` })}</div></section>`,
      )
      .join("");
  }
  function history(s) {
    const rows = s.data.history.filter(
      (r) =>
        (s.filter === "all" || r.origin.app_id === s.filter) &&
        (!s.query ||
          (r.original + " " + (r.processed || ""))
            .toLocaleLowerCase()
            .includes(s.query.toLocaleLowerCase())),
    );
    const selected = s.data.history.find((r) => r.id === s.selected);
    const groups = apps(s)
      .map(([id, name]) => [
        id,
        name,
        rows.filter((r) => r.origin.app_id === id),
      ])
      .filter((x) => x[2].length);
    return `<section class="history-pane" aria-label="Historik"><header class="pane-toolbar"><h1 id="view-title" tabindex="-1">Historik</h1><span class="compact-dictation-controls">${ib(s.data.session?.stage === "recording" ? "stop" : "mic", s.data.session?.stage === "recording" ? "Afslut diktering" : "Start diktering", s.data.session?.stage === "recording" ? "stop" : "start", s.data.session && s.data.session.stage !== "recording" ? "disabled" : "")}${ib("more", "Behandling for næste diktering", "choose-processing")}</span>${ib("search", "Søg i historikken", "toggle-search")}</header>${s.searchOpen ? `<div class="history-search">${f({ id: "history-search", label: "Søg i historikken", type: "search", value: s.query })}${f({ id: "history-app", label: "Startet fra app", value: s.filter, options: [["all", "Alle apps"], ...apps(s)] })}</div>` : ""}<div class="history-scroll"><div class="activity-region">${messages(s)}</div>${recovery(s)}${groups.map(([id, name, rows]) => `<section class="app-group" aria-label="${e(name)}"><button class="group-header" data-action="toggle-group" data-id="${e(id)}" aria-expanded="${!s.collapsed.has(id)}">${appIcon(name, "", id)}<span class="group-name">${e(name)}</span><span class="group-count">${rows.length} ${rows.length === 1 ? "diktering" : "dikteringer"}</span>${icon("chevron", s.collapsed.has(id) ? "is-closed" : "")}</button>${s.collapsed.has(id) ? "" : `<ul class="history-rows">${rows.map((r) => `<li class="history-row ${r.id === s.selected ? "is-selected" : ""}"><button class="row-select" data-action="select-result" data-id="${e(r.id)}" aria-pressed="${r.id === s.selected}"><span class="row-preview">${e(finished(r))}</span><span class="row-meta">${date(r)}${r.delivery === "inserted" ? "" : " · Klar til indsættelse"}</span></button>${resultActions(r.id)}</li>`).join("")}</ul>`}</section>`).join("") || empty(s.query ? "Ingen match" : "Her begynder din historik", s.query ? "Prøv et andet ord eller en anden app." : `Hold ${s.data.settings.shortcut}, tal og slip. Dine dikteringer samles her.`, b(s.query ? "Ryd søgning" : "Kom i gang", s.query ? "clear-search" : "setup"))}</div></section><section class="detail-pane ${s.detailOpen ? "is-open" : ""}" aria-label="Valgt diktering">${selected ? detail(selected, s) : empty("Dine ord, samlet ét sted", "Vælg en diktering i historikken for at se teksten.")}</section>`;
  }
  function detail(r, s) {
    return `<button class="back-button" data-action="back-history">${icon("back")}Tilbage til historik</button><header class="detail-header"><div class="detail-identity">${appIcon(r.origin.app_name, "large", r.origin.app_id)}<div><h2>${e(r.origin.app_name)}</h2><p class="muted">${date(r)}</p></div></div>${resultActions(r.id)}</header><div class="detail-scroll">${s.uiError?.scope === "detail" ? notice(s.uiError.message, "error", b("Luk besked", "dismiss-ui-error", { variant: "quiet" })) : ""}<p class="transcript ${finished(r).length > 600 ? "is-long" : ""}" dir="auto">${e(finished(r))}</p>${r.fallback ? notice("AI-bearbejdning blev ikke færdig. Originalen er bevaret.", "warning") : ""}${r.delivery !== "inserted" ? notice(r.delivery === "unknown" ? "Indsættelsen kunne ikke bekræftes. Kontroller feltet før manuel indsættelse." : "Teksten er klar til manuel indsættelse.", "info") : ""}<div class="copy-bar">${b("Kopiér", "copy", { variant: "primary", glyph: "copy", attrs: `data-id="${e(r.id)}"` })}</div>${r.processed !== null ? `<details class="original-disclosure" ${s.originalOpen ? "open" : ""}><summary>${icon("chevron")}<span>${s.originalOpen ? "Skjul original" : "Vis original"}</span></summary><div class="original-panel"><p dir="auto">${e(r.original)}</p><p class="original-meta">Original transskription · ingen lyd i historikken</p>${b("Kopiér original", "copy-original", { variant: "quiet", attrs: `data-id="${e(r.id)}"` })}</div></details>` : ""}<div class="result-footnote"><span>Lyd: ${e(loc[r.plan.speech.locality])}${r.plan.text ? " · Tekst: " + e(loc[r.plan.text.engine.locality]) : ""}</span><span>${s.data.settings.retention_days ? "Oprydning efter " + s.data.settings.retention_days + " dage" : "Gemt, indtil du sletter"}</span></div></div>`;
  }
  function appPreferences(s) {
    const available = apps(s),
      name = available.find((a) => a[0] === s.prefApp)?.[1];
    const p = s.data.settings.app_rules[s.prefApp] || {
      enabled: false,
      prompt: s.data.settings.general_prompt,
      engine_id: s.data.settings.general_engine,
    };
    return `<div class="page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">AI pr. app</h1><p class="muted">Direkte som standard. Tilpas de apps, du vælger.</p></header><div class="activity-region">${messages(s)}</div><div class="app-add-action">${b("Tilføj app", "add-app", { variant: "quiet", glyph: "apps" })}</div>${available.length ? `<div class="app-preferences"><nav class="app-selector" aria-label="Vælg app">${available.map(([id, name]) => `<button class="app-choice ${s.prefApp === id ? "is-selected" : ""}" data-action="select-pref" data-id="${e(id)}">${appIcon(name, "", id)}${e(name)}</button>`).join("")}</nav>${name ? `<form id="app-form" class="preference-form"><div class="form-title">${appIcon(name, "large", s.prefApp)}<h2>${e(name)}</h2></div>${toggle("app-ai", "Brug egne AI-indstillinger", p.enabled, "Slå fra for at følge standarden under Diktering. Slå til for at bruge denne apps instruktioner og tekstmodel.")}<div id="app-ai-fields" ${p.enabled ? "" : "hidden"}>${f({ id: "app-prompt", label: "Instruktioner til AI", value: p.prompt, rows: true, help: "Originalen bliver altid gemt sammen med det færdige resultat." })}${modelPicker(s,"app-engine","Tekstmodel","text",p.engine_id)}</div><div class="form-message" role="status"></div><div class="actions">${b("Gem app-indstilling", "save-app", { variant: "primary", attrs: s.dirty ? "" : "disabled" })}${b("Fortryd ændringer", "reset-form", { variant: "quiet" })}<span class="unsaved" ${s.dirty ? "" : "hidden"}>Ikke gemt</span></div><p class="form-footnote">Ingen ekstra godkendelse ved diktering. Automatisk indsættelse ved uændret fokus.</p></form>` : empty("Vælg en app", "Vælg appen, hvis tekst du vil tilpasse.")}</div>` : empty("Tilpas de apps, du bruger", "Tilføj en app, eller begynd at diktere. Apps fra historikken dukker også op her.")}</div>`;
  }
  const providersList = [
    ["open_ai", "OpenAI", "Tale og tekst"],
    ["groq", "Groq", "Whisper til tale · AI til tekst"],
    ["eleven_labs", "ElevenLabs", "Tale"],
    ["anthropic", "Anthropic", "Tekst"],
    ["gemini", "Gemini", "Tekst"],
  ];
  function providers(s) {
    return providersList.map(([id, name, kind]) => {
      const key = s.keyStates[id], test = s.connectionChecks[id];
      const label = key?.error ? "Nøglestatus er utilgængelig" : key?.present === true ? "Nøgle gemt" + (test?.phase === "success" ? " · API-adgang testet" : " · Ikke testet") : key?.present === false ? "Ingen nøgle tilføjet" : "Kontrollerer nøglelager…";
      const checked = test?.phase === "success";
      const action = id === "open_ai" ? "check-openai" : "check-groq";
      return `<div class="provider-connection" data-provider="${id}"><div class="provider-row"><div class="provider-identity"><strong>${name}</strong><small>${kind}</small></div><div class="provider-credential"><span class="credential-status ${key?.present ? "has-key" : ""}">${key?.present ? icon("check") : ""}${e(label)}</span></div><div class="actions">${b(key?.present === false ? "Tilføj nøgle" : "Administrer nøgle", "open-key", { attrs: `data-id="${id}" data-name="${name}"`, glyph: "key" })}${key?.present && ["open_ai", "groq"].includes(id) ? b(test?.phase === "testing" ? "Tester…" : "Test nøgle", action, { attrs: `data-id="${id}" ${test?.phase === "testing" ? "disabled" : ""}` }) : ""}</div></div><p class="connection-check ${test?.phase === "error" ? "is-error" : checked ? "is-success" : ""}" data-connection-status="${id}" role="status" ${test?.message || key?.error ? "" : "hidden"}>${e(test?.message || key?.error || "")}</p></div>`;
    }).join("");
  }
  function models(s, only = null) {
    const catalog = {
      "local.parakeet": ["Parakeet Redux · tale", "Flersproget · 179 MB"],
      "local.whisper": ["Whisper Base · tale", "Valgfrit alternativ · 148 MB"],
      "local.text": ["Qwen3 0.6B · tekst", "Valgfri AI-bearbejdning · 651 MB"],
    };
    return Object.entries(catalog)
      .filter(([id]) => !only || id === only)
      .map(([id, [name, description]]) => {
        const download = s.data.downloads.find((d) => d.id === id);
        return `<div class="model-row"><div class="model-identity"><strong>${e(name)}</strong><small>${e(description)}</small></div><div class="model-state"><small>${download ? "Henter " + Math.round(download.received / 1000000) + " / " + Math.round(download.total / 1000000) + " MB" : s.data.models[id] ? "Installeret på enheden" : "Ikke hentet"}</small>${download ? `<progress class="model-progress" data-model-progress="${id}" max="${download.total || 1}" value="${download.received}" aria-label="Modeldownload"></progress>` : ""}</div>${b(download ? "Annuller" : s.data.models[id] ? "Kontrollér model" : "Hent model", download ? "cancel_install" : "install", { attrs: `data-id="${id}"`, glyph: download ? "close" : "download" })}</div>`;
      }).join("");
  }
  function permissions(s) {
    const p = s.data.permissions;
    return `<div class="permission-grid">${permissionRow({ title: "Mikrofon", description: p.microphone === "denied" ? "Tillad Vara under Mikrofon i systemets privatlivsindstillinger." : "Bruges kun, når du starter en diktering.", kind: "microphone", granted: p.microphone === "authorized", available: p.microphone !== "system" })}${typeof p.input_monitoring === "boolean" ? permissionRow({ title: "Globale genveje", description: "Inputovervågning lader Vara registrere din genvej, mens du arbejder i andre apps.", kind: "input_monitoring", granted: p.input_monitoring }) : ""}${permissionRow({ title: "Indsættelse i andre apps", description: p.automatic_insertion ? "Tilgængelighed gør det muligt at indsætte i det oprindelige tekstfelt." : "Teksten bliver klar til manuel indsættelse på denne platform.", kind: "accessibility", granted: p.accessibility, available: p.automatic_insertion })}</div>`;
  }
  function modelFeedback(s, engine, settings = s.data.settings) {
    if (!engine) return notice("Vælg en model.", "warning");
    const location = engine.locality === "cloud" ? "Cloud" : loc[engine.locality] || "Ukendt placering";
    if (settings.strict_local && engine.locality !== "on_device")
      return notice("Kun behandling på denne enhed er slået til. Vælg en lokal model, eller ændr reglen under Privatliv.", "warning");
    if (Object.hasOwn(s.data.models, engine.id)) {
      const progress = s.data.downloads.find(d => d.id === engine.id);
      return `<div class="model-readiness" role="status"><span>${icon(s.data.models[engine.id] ? "check" : "download")}${e(location)} · ${progress ? `Henter ${Math.round(progress.received / 1000000)} / ${Math.round(progress.total / 1000000)} MB` : s.data.models[engine.id] ? "Klar" : "Ikke hentet"}</span>${!s.data.models[engine.id] ? b(progress ? "Annuller" : "Hent model", progress ? "cancel_install" : "install", { variant: "quiet", attrs: `data-id="${e(engine.id)}"` }) : ""}</div>${progress ? `<progress class="model-progress" data-model-progress="${e(engine.id)}" value="${progress.received}" max="${progress.total || 1}" aria-label="Modeldownload"></progress>` : ""}`;
    }
    if (["claude_cli", "codex_cli", "gemini_cli"].includes(engine.provider))
      return `<div class="model-readiness"><span>${e(location)} · ${e(s.cli[engine.provider]?.detail || "CLI-login kræves")}</span>${b("Kontrollér CLI", "check-cli", { variant: "quiet", attrs: `data-id="${e(engine.provider)}"` })}</div>`;
    const optionalKey = ["custom", "ollama"].includes(engine.provider);
    const id = optionalKey ? engine.id : engine.provider;
    const key = s.keyStates[id], test = s.connectionChecks[id];
    const name = ({open_ai:"OpenAI",groq:"Groq",eleven_labs:"ElevenLabs",anthropic:"Anthropic",gemini:"Gemini"})[id] || engine.label;
    const text = key?.error ? "Nøglestatus er utilgængelig" : optionalKey ? "Eget endpoint · Nøgle er valgfri" : key?.present ? "Nøgle gemt" + (test?.phase === "success" ? " · API-adgang testet" : " · Ikke testet") : key?.present === false ? "Nøgle mangler" : "Kontrollerer nøglelager…";
    return `<div class="model-readiness" role="status"><span>${icon(engine.locality === "cloud" ? "globe" : "info")}${e(location)} · ${e(text)}</span>${key?.present && ["open_ai", "groq"].includes(id) ? b(test?.phase === "testing" ? "Tester…" : "Test nøgle", id === "open_ai" ? "check-openai" : "check-groq", { variant: "quiet", attrs: `data-id="${e(id)}" ${test?.phase === "testing" ? "disabled" : ""}` }) : b(!optionalKey && key?.present === false ? `Tilføj ${name}-nøgle` : "Administrer nøgle", "open-key", { variant: "quiet", attrs: `data-id="${e(id)}" data-name="${e(name)}"` })}</div>${key?.error || test?.phase === "error" ? notice(key?.error || test.message, "error") : ""}`;
  }
  function modelPicker(s, id, label, kind, selected) {
    const engine = s.data.settings.engines.find(e => e.id === selected);
    return `<div class="model-picker">${Vara.desktopUi.modelSelect({ id, label, engines: s.data.settings.engines.filter(e => e.kind === kind), selected, strict: s.data.settings.strict_local })}<div class="model-feedback" data-model-field="${e(id)}">${modelFeedback(s, engine)}</div></div>`;
  }
  function privacySummary(s, settings = s.data.settings) {
    const speech = settings.engines.find(e => e.id === settings.speech_engine);
    const text = settings.engines.find(e => e.id === settings.general_engine);
    const appRules = Object.values(settings.app_rules).filter(r => r.enabled);
    const conflicts = settings.strict_local && (speech?.locality !== "on_device" || settings.general_ai_enabled && text?.locality !== "on_device" || appRules.some(r => settings.engines.find(e => e.id === r.engine_id)?.locality !== "on_device"));
    return `<div class="processing-summary">${Vara.desktopUi.policyRow("mic", `Lyd: ${speech?.locality === "cloud" ? "Cloud" : loc[speech?.locality] || "Ukendt placering"}`, "")}${Vara.desktopUi.policyRow("globe", settings.general_ai_enabled ? `Tekst: ${text ? Vara.desktopUi.modelLabel(text) : "Ukendt model"} · ${text?.locality === "cloud" ? "Cloud" : loc[text?.locality] || "Ukendt placering"}` : "Tekst: Direkte transskription som standard", appRules.length ? "Apps med egne regler bruger deres valgte AI." : "")}</div><p class="muted">Cloud bruges kun til de funktioner, du selv vælger.</p>${conflicts ? notice("Dine valgte modeller eller appregler bruger behandling uden for enheden. Vælg lokale modeller under Diktering og AI pr. app, før du gemmer denne regel.", "warning") : ""}`;
  }
  function settings(s) {
    const p = s.data.settings, { settingsSection: group, processingChoice, policyRow } = Vara.desktopUi;
    let body = "";
    if (s.tab === "dictation") {
      body = group("Optagelse", "Sådan optager du, og hvad der skal bruges.", `<div class="settings-field-grid">${f({ id:"activation",label:"Betjening",value:p.activation,options:[["hold","Hold for at tale"],["toggle","Tryk for start / stop"]] })}${shortcut(p.shortcut)}${f({ id:"input-device",label:"Mikrofon",value:p.input_device || "",help:s.deviceError || "",options:[["","Systemets standard"],...s.devices.map(d=>[d.id,d.label])] })}${f({ id:"language",label:"Sprog",value:p.speech_engine === "local.parakeet" ? "" : p.language || "",attrs:p.speech_engine === "local.parakeet" ? "disabled" : "",help:p.speech_engine === "local.parakeet" ? "Parakeet registrerer automatisk sproget." : "Sprog til transskription.",options:[["","Registrer automatisk"],["da","Dansk"],["en","Engelsk"],["de","Tysk"],["sv","Svensk"],["no","Norsk"]] })}</div>`)
        + group("Tale til tekst", "Vælg den model, der transskriberer din tale.", modelPicker(s,"speech-engine","Transskriptionsmodel","speech",p.speech_engine))
        + group("Efter transskription", "Din standard for nye dikteringer.", processingChoice(p.general_ai_enabled) + `<div id="text-processing-fields" ${p.general_ai_enabled ? "" : "hidden"}>${modelPicker(s,"general-engine","Tekstmodel","text",p.general_engine)}${f({ id:"general-prompt",label:"Instruktioner til AI",value:p.general_prompt,rows:true })}</div><p class="app-rules-note">Apps med egne regler bruger deres egne indstillinger. ${b("Administrer apps →","navigate",{variant:"quiet",attrs:'data-page="apps"'})}</p>`)
        + group("Tilladelser", "Adgang til mikrofon, genvej og indsættelse i andre apps.", permissions(s));
    }
    if (s.tab === "connections") {
      body = group("Lokale modeller", "Hent én gang. Brug uden konto eller nøgle.", `<div class="model-list">${models(s)}</div><p class="form-footnote">Parakeet Redux: Moondream og NVIDIA · CC BY 4.0.</p>`, true)
        + group("API-nøgler", "Dine egne nøgler gemmes i systemets nøglelager.", `<div class="provider-list" data-connections>${providers(s)}</div>`, true)
        + disclosure("advanced-engines","Egne endpoints og modelnavne","Redigér modelnavne og kompatible tekstforbindelser.", `<div class="provider-list">${p.engines.filter(e=>!e.id.startsWith("local.") && !e.id.endsWith(".cli")).map(x=>`<div class="provider-row"><div><strong>${e(x.label)}</strong><small>${e(x.model)} · ${e(loc[x.locality])}</small></div>${b("Redigér","edit-engine",{attrs:`data-id="${e(x.id)}"`})}</div>`).join("")}</div>${b("Tilføj endpoint","add-endpoint")}`, s.disclosures?.["advanced-engines"])
        + disclosure("cli-engines","Installerede CLI-værktøjer","Valgfrie tekstforbindelser med eget login.", `<div class="provider-list">${[["claude_cli","Claude Code"],["codex_cli","Codex"],["gemini_cli","Gemini CLI"]].map(([id,name])=>`<div class="provider-row"><div><strong>${name}</strong><small>${e(s.cli?.[id]?.detail || "Ikke kontrolleret. Teksten kan behandles hos en ekstern udbyder.")}</small></div>${b("Kontrollér","check-cli",{attrs:`data-id="${id}"`})}</div>`).join("")}</div>`, s.disclosures?.["cli-engines"])
        + `<p class="model-selection-note">Modelvalg ændres under ${b("Diktering →","settings-tab",{variant:"quiet",attrs:'data-id="dictation"'})}.</p>`;
    }
    if (s.tab === "privacy") {
      body = group("Behandling", "Du bestemmer, hvor dine data behandles.", toggle("strict-local","Kun behandling på denne enhed",p.strict_local,"Når dette er slået til, kan Vara kun bruge lokale modeller.") + `<div id="privacy-processing-summary">${privacySummary(s)}</div>`)
        + group("Historik", "Dine dikteringer gemmes lokalt, grupperet efter app.", f({id:"retention",label:"Behold teksten",value:p.retention_days ? String(p.retention_days) : "",options:[["","Indtil jeg selv sletter"],["7","7 dage"],["30","30 dage"],["90","90 dage"],["365","1 år"],...(p.retention_days && ![7,30,90,365].includes(p.retention_days) ? [[String(p.retention_days),p.retention_days+" dage"]] : [])]}) + `<p class="muted">Du kan vælge automatisk oprydning efter en periode.</p><p class="form-footnote">Når du sletter en diktering, slettes både originalen og den bearbejdede tekst.</p>`)
        + group("Lyd", "Ingen lyd i historikken.", policyRow("trash","Efter vellykket transskription","Lyden slettes automatisk.") + policyRow("history","Hvis transskriptionen fejler","Lyden gemmes midlertidigt i højst 24 timer, så du kan prøve igen.") + `<p class="form-footnote">En vellykket genprøvning eller din sletning fjerner lyden tidligere.</p>`);
    }
    return `<div class="page-scroll settings-page"><header class="page-header"><h1 id="view-title" tabindex="-1">Indstillinger</h1><p class="muted">En stille hjælper, på dine præmisser.</p></header><nav class="settings-nav" aria-label="Indstillinger">${[["dictation","Diktering"],["connections","Modeller og nøgler"],["privacy","Privatliv"]].map(([id,name])=>`<button class="${s.tab===id ? "is-current" : ""}" data-action="settings-tab" data-id="${id}" ${s.tab===id ? 'aria-current="page"' : ""}>${name}</button>`).join("")}</nav><div class="activity-region">${messages(s)}</div><form class="settings-form" id="settings-form">${body}<div class="form-message" role="status"></div><div class="actions sticky-save"><span class="unsaved" ${s.dirty ? "" : "hidden"}>Ikke gemt</span>${b("Fortryd ændringer","reset-form",{variant:"quiet"})}${b("Gem indstillinger","save-settings",{variant:"primary",attrs:s.dirty ? "" : "disabled"})}</div></form></div>`;
  }
  function setup(s) {
    const steps = [
      ["prepare", "Gør Vara klar"],
      ["permissions", "Giv adgang"],
      ["trial", "Prøv din genvej"],
    ];
    const current = s.setupStep || "prepare",
      index = steps.findIndex(([id]) => id === current);
    const speechId = s.data.settings.speech_engine;
    const localSpeech = ["local.parakeet", "local.whisper"].includes(speechId);
    const ready = !localSpeech || s.data.models[speechId];
    let content = "";
    if (current === "prepare")
      content = `<h1 id="view-title" tabindex="-1">Dine ord, dér hvor du arbejder.</h1><p class="setup-description">Hold en tast, tal og slip. Vi begynder med lokal transskription, så din lyd bliver på enheden.</p><section class="setup-preparation">${models(s, localSpeech ? speechId : "local.parakeet")}</section><p class="muted">Modellen hentes én gang. Ingen konto eller ekstra installation.</p><div class="setup-actions">${b("Fortsæt", "setup-next", { variant: "primary", attrs: ready ? "" : "disabled" })}${b("Jeg foretrækker cloud", "open-dictation", { variant: "quiet" })}</div>`;
    if (current === "permissions")
      content = `<h1 id="view-title" tabindex="-1">Giv Vara adgang.</h1><p class="setup-description">Mikrofonen er nødvendig for at diktere. Du kan altid kopiere teksten manuelt, hvis du springer automatisk indsættelse over.</p>${permissions(s)}<div class="setup-actions">${b("Tilbage", "setup-back", { variant: "quiet" })}${b("Fortsæt", "setup-next", { variant: "primary", attrs: ["authorized", "system"].includes(s.data.permissions.microphone) ? "" : "disabled" })}</div>`;
    if (current === "trial")
      content = `<h1 id="view-title" tabindex="-1">Sig din første sætning.</h1><p class="setup-description">Åbn et tekstfelt i en anden app. ${s.data.settings.activation === "hold" ? "Hold din genvej, tal og slip." : "Tryk på din genvej, tal og tryk igen."} Resultatet samles i din historik.</p><div class="trial-shortcut"><kbd>${e(binding(s.data.settings.shortcut))}</kbd><span>${s.data.settings.activation === "hold" ? "Hold → tal → slip" : "Tryk → tal → tryk igen"}</span></div><p class="muted">Du kan også starte her. Så bliver teksten klar til manuel indsættelse.</p><div class="setup-actions">${b("Tilbage", "setup-back", { variant: "quiet" })}${b(s.data.session?.stage === "recording" ? "Afslut diktering" : "Prøv en diktering", s.data.session?.stage === "recording" ? "stop" : "start", { variant: "primary", glyph: s.data.session?.stage === "recording" ? "stop" : "mic", attrs: s.data.session && s.data.session.stage !== "recording" ? "disabled" : "" })}</div>`;
    return `<div class="page-scroll setup-page"><div class="initial-setup"><ol class="setup-steps" aria-label="Opsætning">${steps.map(([id, label], i) => `<li ${id === current ? 'aria-current="step"' : ""}><span class="step-number">${i < index ? icon("check") : i + 1}</span><span>${label}</span></li>`).join("")}</ol><div class="activity-region">${messages(s)}</div><div class="setup-content">${content}</div><div class="setup-later">${b("Fortsæt senere", "navigate", { variant: "quiet", attrs: 'data-page="history"' })}</div></div></div>`;
  }
  Vara.views = {
    sidebar,
    history,
    apps: appPreferences,
    settings,
    setup,
    messages,
    appsList: apps,
    models,
    providers,
    providersList,
    modelFeedback,
    modelPicker,
    privacySummary,
  };
})();
