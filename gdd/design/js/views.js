/* Screen compositions built from shared pure UI components. */
(() => {
  "use strict";
  const {
    esc: e,
    icon,
    mark,
    wave,
    appIcon,
    button: btn,
    iconButton: ib,
    notice,
    field,
    toggle,
    section,
    empty,
  } = Vara.ui;
  const date = (r) =>
    `${e(r.date)}${r.duration ? " · " + r.duration + " sek." : ""}`;
  const locationNames = {
    on_device: "På denne enhed",
    lan: "På eget netværk",
    cloud: "Hos valgt udbyder",
  };
  const engineOptions = [
    ["local", "På enheden · lokal tekst-AI"],
    ["cloud", "Cloud · egen API-nøgle"],
    ["cli", "Installeret CLI · eget login"],
  ];
  const locality = (engine) =>
    engine === "local"
      ? "Teksten bliver på enheden."
      : "Teksten sendes til den valgte udbyder. Dette er et eksplicit tilvalg.";
  const statusNotice = (r) =>
    `${r.fallback ? notice("AI-bearbejdningen fejlede. Originalen er bevaret.", "warning") : ""}${r.status === "manual" ? notice("Fokus ændrede sig. Teksten er klar til manuel indsættelse.", "warning") : ""}`;
  function sidebar(s) {
    const remoteText = s.active
      ? s.active.processing && s.active.engine !== "local"
      : Object.values(s.prefs).some((p) => p.enabled && p.engine !== "local");
    return `<div class="window-controls" aria-hidden="true"><i></i><i></i><i></i></div><div class="brand">${mark()}<span>vara</span></div>
      <nav>${[
        ["history", "history", "Historik"],
        ["apps", "apps", "AI pr. app"],
        ["settings", "settings", "Indstillinger"],
      ]
        .map(
          ([p, g, l]) =>
            `<button class="nav-item ${s.page === p ? "is-current" : ""}" data-action="navigate" data-page="${p}" ${s.page === p ? 'aria-current="page"' : ""}>${icon(g)}<span>${l}</span></button>`,
        )
        .join("")}</nav>
      <div class="sidebar-footer"><div class="sidebar-status"><span class="status-dot"></span><span>${s.settings.speech === "local" ? (remoteText ? "Lyd: på enheden" : "På enheden") : "Lyd: cloud valgt"}</span><kbd id="nav-shortcut">${e(s.settings.shortcut)}</kbd></div><p id="nav-style">${s.settings.style === "hold" ? "Hold, tal og slip." : "Tryk for start / stop."}</p><p>${s.settings.speech === "local" ? "Lokal transskription.<br>Din lyd forlader ikke denne enhed." : "Behandling af lyd hos din valgte udbyder."}</p>${remoteText ? "<p>Tekst-AI: ekstern eller ukendt placering.</p>" : ""}</div>`;
  }
  function row(r, s) {
    return `<li class="history-row ${r.id === s.selected ? "is-selected" : ""}"><button class="row-select" data-action="select-result" data-id="${r.id}" aria-pressed="${r.id === s.selected}"><span class="row-preview">${e(r.text || r.raw)}</span><span class="row-meta">${date(r)}${r.status === "manual" ? '<span class="row-status"> · Klar til indsættelse</span>' : ""}</span></button>${ib("more", "Handlinger for diktering", "result-menu", `data-id="${r.id}"`)}</li>`;
  }
  function recovery(s) {
    return s.recoveries.length
      ? `<div class="recovery-list">${s.recoveries.map((r) => `<section class="recovery-task"><h3>${icon("warning")}Transskriptionen fra ${e(s.appNames[r.app])} fejlede</h3><p>Optagelsen udløber ${new Date(r.expires).toLocaleString("da-DK", { hour: "2-digit", minute: "2-digit", day: "numeric", month: "short" })}. Fristen forlænges ikke ved genprøvning.</p><div class="actions">${btn("Prøv igen", "retry", { attrs: `data-id="${r.id}"` })}${btn("Slet optagelse", "delete-recovery", { variant: "quiet", attrs: `data-id="${r.id}"` })}</div></section>`).join("")}</div>`
      : "";
  }
  function history(s) {
    let visible = s.results.filter(
      (r) =>
        (s.filter === "all" || r.app === s.filter) &&
        (!s.query ||
          (r.text || r.raw).toLowerCase().includes(s.query.toLowerCase()) ||
          r.raw.toLowerCase().includes(s.query.toLowerCase())),
    );
    const groups = Object.keys(s.appNames)
      .map((app) => [app, visible.filter((r) => r.app === app)])
      .filter(([, r]) => r.length);
    const selected = s.results.find((r) => r.id === s.selected);
    return `<section class="history-pane" aria-label="Historik"><header class="pane-toolbar"><h1 id="view-title" tabindex="-1">Historik</h1>${ib("search", "Søg i historikken", "toggle-search")}</header>
      ${s.searchOpen ? `<div class="history-search">${field({ id: "history-search", label: "Søg i historikken", type: "search", value: s.query })}${field({ id: "history-app", label: "Startet fra app", value: s.filter, options: [["all", "Alle apps"], ...Object.entries(s.appNames)] })}</div>` : ""}
      <div class="history-scroll" id="history-scroll">${recovery(s)}${groups.map(([app, rows]) => `<section class="app-group" aria-label="${e(s.appNames[app])}"><button class="group-header" data-action="toggle-group" data-app="${app}" aria-expanded="${!s.collapsed.includes(app)}">${appIcon(app)}<span class="group-name">${e(s.appNames[app])}</span><span class="group-count">${rows.length} ${rows.length === 1 ? "diktering" : "dikteringer"}</span>${icon("chevron", s.collapsed.includes(app) ? "is-closed" : "")}</button>${s.collapsed.includes(app) ? "" : `<ul class="history-rows">${rows.map((r) => row(r, s)).join("")}</ul>`}</section>`).join("") || empty(s.query ? "Ingen match" : "Her begynder din historik", s.query ? "Prøv et andet ord eller en anden app." : `Hold ${s.settings.shortcut}, tal og slip. Dine dikteringer samles her.`, btn(s.query ? "Ryd søgning" : "Prøv en diktering", s.query ? "clear-search" : "open-demo"))}</div></section>
      <section class="detail-pane ${s.detailOpen ? "is-open" : ""}" aria-label="Valgt diktering">${selected ? detail(selected, s) : empty("Dine ord, samlet ét sted", "Vælg en diktering i historikken for at se teksten.")}</section>`;
  }
  function detail(r, s) {
    const compact = (r.text || r.raw).length > 600;
    return `<button class="back-button" data-action="back-history">${icon("back")}Tilbage til historik</button>
      <header class="detail-header"><div class="detail-identity">${appIcon(r.app, "large")}<div><h2>${e(s.appNames[r.app])}</h2><p class="muted">${date(r)}</p></div></div>${ib("more", "Handlinger for valgt diktering", "result-menu", `data-id="${r.id}"`)}</header>
      <div class="detail-scroll"><p class="transcript ${compact ? "is-long" : ""}" dir="auto">${e(r.text || r.raw)}</p>${statusNotice(r)}
      <div class="copy-bar">${btn("Kopiér", "copy-result", { variant: "primary", glyph: "copy", attrs: `data-id="${r.id}"` })}</div>
      ${r.processed ? `<details class="original-disclosure" ${s.originalOpen ? "open" : ""}><summary>${icon("chevron")}<span>Vis original</span></summary><div class="original-panel"><p>${e(r.raw)}</p><p class="original-meta">${r.duration ? r.duration + " sek. · " : ""}Original transskription · ingen lyd gemt</p></div></details>` : ""}
      <div class="result-footnote"><span>${r.local ? "Behandlet på enheden" : "Behandlet hos valgt udbyder"}</span><span>${s.settings.cleanup ? "Automatisk oprydning efter " + s.settings.days + " dage" : "Gemt, indtil du sletter"}</span></div></div>`;
  }
  function apps(s) {
    const p = s.prefs[s.prefApp];
    return `<div class="page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">AI pr. app</h1><p class="muted">Direkte som standard. Tilpas de apps, du vælger.</p></header><div class="app-preferences"><nav class="app-selector" aria-label="Vælg app">${Object.entries(
      s.appNames,
    )
      .map(
        ([a, n]) =>
          `<button data-action="select-pref" data-app="${a}" class="app-choice ${a === s.prefApp ? "is-selected" : ""}" ${a === s.prefApp ? 'aria-current="true"' : ""}>${appIcon(a)}${e(n)}</button>`,
      )
      .join("")}</nav><form id="app-form" class="preference-form" novalidate>
      <div class="form-title">${appIcon(s.prefApp, "large")}<h2>${e(s.appNames[s.prefApp])}</h2></div>${toggle("app-ai", "AI-bearbejdning", p.enabled, "Brug appens systemprompt automatisk, når du dikterer her.")}
      ${field({ id: "app-prompt", label: "Systemprompt", value: p.prompt, rows: true, help: "Bevar budskab, navne, tal og min tone. Originalen bliver gemt." })}
      ${field({ id: "app-engine", label: "AI til tekstbearbejdning", value: p.engine, options: engineOptions, help: locality(p.engine) })}
      <div id="app-form-message" class="form-message"></div><div class="actions">${btn("Gem app-indstilling", "save-app", { variant: "primary" })}${btn("Fortryd ændringer", "reset-app", { variant: "quiet" })}<span class="unsaved" id="unsaved" ${s.dirty ? "" : "hidden"}>Ikke gemt</span></div>
      <p class="form-footnote">Ingen ekstra godkendelse ved diktering. Automatisk indsættelse ved uændret fokus.</p></form></div></div>`;
  }
  function settings(s) {
    const p = s.settings;
    let body = "";
    if (s.settingsTab === "dictation")
      body =
        section(
          "Diktér på din måde",
          "Genvejen virker i den app, hvor du arbejder.",
          field({
            id: "activation-style",
            label: "Betjening",
            value: p.style,
            options: [
              ["hold", "Hold for at tale"],
              ["toggle", "Tryk for start / stop"],
            ],
          }) +
            field({
              id: "input-device",
              label: "Mikrofon",
              value: p.device,
              options: [
                ["Indbygget mikrofon", "Indbygget mikrofon (demo)"],
                ["Ekstern mikrofon", "Ekstern mikrofon (demo)"],
              ],
            }) +
            `<div class="shortcut-setting"><div><span class="field-label">Din genvej</span><p class="muted">Bindingen i prototypen registreres ikke på systemet.</p></div><kbd>${e(p.shortcut)}</kbd>${btn("Skift genvej", "record-shortcut")}</div>`,
        ) +
        section(
          "Generel systemprompt",
          "Bruges, når du selv vælger AI til den næste diktering.",
          field({
            id: "general-prompt",
            label: "Dine instruktioner",
            value: p.general,
            rows: true,
          }) + btn("Gem prompt", "save-general", { variant: "primary" }),
        );
    if (s.settingsTab === "connections")
      body =
        section(
          "Tale til tekst",
          "På enheden som standard. Cloud, når du vælger det.",
          field({
            id: "speech-choice",
            label: "Behandling af lyd",
            value: p.speech,
            options: [
              ["local", "På enheden"],
              ["cloud", "Cloud · egen nøgle"],
            ],
          }) +
            notice(
              p.speech === "local"
                ? "Anbefalet lokal talemodel · klar i demoen. Ingen lyd sendes til cloud."
                : "Cloud er valgt. Tilføj din egen nøgle hos en taleudbyder.",
              p.speech === "local" ? "success" : "info",
            ) +
            toggle(
              "strict-local",
              "Kun behandling på denne enhed",
              p.strictLocal,
              "Afvis cloud, LAN og ukendt placering for lyd og tekst.",
            ),
        ) +
        section(
          "Forbindelser",
          "Egne nøgler. Dine egne konti hos udbyderne.",
          `<div class="provider-list">${["OpenAI", "Groq", "ElevenLabs", "Anthropic", "Gemini"].map((p) => `<div class="provider-row"><div><strong>${p}</strong><small>${p === "ElevenLabs" ? "Tale" : ["OpenAI", "Groq"].includes(p) ? "Tale og tekst" : "Tekst"}</small></div>${btn(s.keyConfigured[p] ? "Nøgle tilføjet · demo" : "Tilføj nøgle", "open-key", { attrs: `data-provider="${p}"`, glyph: "key" })}</div>`).join("")}</div>`,
        ) +
        section(
          "Lokale tjenester og egne endpoints",
          "Ollama og kompatible tekstmodeller. Lokalitet er et eksplicit valg.",
          s.endpoints
            .map((x) =>
              notice(
                x.name + " · " + x.model + " · " + locationNames[x.locality],
                "success",
              ),
            )
            .join("") +
            `<form id="endpoint-form" novalidate>${field({ id: "endpoint-name", label: "Navn" })}${field({ id: "endpoint-url", label: "Endpoint-URL", type: "url", attrs: 'placeholder="http://localhost:11434/v1"' })}${field({ id: "endpoint-model", label: "Model" })}${field(
              {
                id: "endpoint-locality",
                label: "Hvor behandles teksten?",
                value: "on_device",
                options: [
                  ["on_device", "På denne enhed"],
                  ["lan", "På eget netværk"],
                  ["cloud", "Hos ekstern udbyder"],
                ],
              },
            )}${field({ id: "endpoint-key", label: "API-nøgle (valgfri)", type: "password", help: "Kun en fiktiv nøgle i denne prototype.", attrs: 'autocomplete="off"' })}<div id="endpoint-error"></div>${btn("Afprøv forbindelse (demo)", "save-endpoint", { variant: "primary" })}</form>`,
        ) +
        section(
          "Installerede CLI-værktøjer",
          "Valgfrie tekstmodeller med værktøjets eget login. Modellen kan køre hos en ekstern udbyder.",
          `<div class="provider-list">${["Claude Code", "Codex", "Gemini CLI"].map((n) => `<div class="provider-row"><strong>${n}</strong><span class="muted">Ikke kontrolleret i demoen</span></div>`).join("")}</div>`,
        );
    if (s.settingsTab === "privacy")
      body =
        section(
          "Din teksthistorik",
          "Original og færdig tekst hører til samme resultat.",
          toggle(
            "cleanup-toggle",
            "Automatisk oprydning",
            p.cleanup,
            "Slået fra som standard. Ellers gemmes teksten, indtil du sletter.",
          ) +
            `<div id="cleanup-period" ${p.cleanup ? "" : "hidden"}>${field({ id: "cleanup-days", label: "Slet tekst efter antal dage", value: p.days, type: "number", attrs: 'min="1" step="1"' })}</div><div id="retention-error"></div>` +
            btn("Gem oprydning", "save-retention", { variant: "primary" }),
        ) +
        section(
          "Ingen lyd i historikken",
          "Lyd slettes efter vellykket transskription. Ved fejl beholdes den midlertidigt til genprøvning i højst 24 timer. Fristen forlænges ikke ved genprøvning.",
          notice(
            "Optagelsen slettes tidligere ved vellykket genprøvning eller manuel sletning.",
            "info",
          ),
        ) +
        section(
          "Hvor behandles dine ord?",
          "Lyd og eventuel AI-bearbejdning af tekst har hver deres destination.",
          `<div class="provider-row"><span>Lyd → tekst</span><strong>${p.speech === "local" ? "På enheden" : "Valgt cloududbyder"}</strong></div><div class="provider-row"><span>Tekst → AI</span><strong>Kun ved dit tilvalg</strong></div>`,
        );
    return `<div class="page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">Indstillinger</h1><p class="muted">En god standard. Mulighed for at vælge selv.</p></header><nav class="settings-nav" aria-label="Indstillingsafsnit">${[
      ["dictation", "Diktering"],
      ["connections", "Modeller & forbindelser"],
      ["privacy", "Historik & privatliv"],
    ]
      .map(
        ([id, n]) =>
          `<button data-action="settings-tab" data-tab="${id}" ${id === s.settingsTab ? 'aria-current="page"' : ""}>${n}</button>`,
      )
      .join("")}</nav><div class="settings-content">${body}</div></div>`;
  }
  function hud(s) {
    const a = s.active;
    const labels = {
      recording: "Lytter",
      speech: "Finder dine ord",
      text: "Tilpasser teksten",
    };
    return `<div class="recording-hud ${a?.stage === "recording" ? "is-recording" : ""}"><div class="hud-origin">${appIcon(a?.app || s.demo.app)}<span>${e(s.appNames[a?.app || s.demo.app])}</span></div>${wave()}<div class="hud-status"><span>${a ? labels[a.stage] : e(s.hudMessage || "Klar")}</span><small>${a ? (a.stage === "recording" ? (a.processing ? "AI valgt" : "Direkte") : a.local ? "På enheden" : "Valgt udbyder") : "Eksempel"}</small></div>${a ? `${a.stage === "recording" ? ib("stop", "Afslut diktering", "stop-session") : ""}${ib("close", "Annuller diktering", "cancel-session")}` : ib("mic", "Afprøv diktering", "open-demo")}</div>`;
  }
  function dialog(s) {
    const close = ib("close", "Luk dialog", "close-modal");
    let title = "",
      body = "";
    if (s.modal === "demo") {
      title = "Afprøv dikteringsflow";
      body = `<p class="muted">Alle forløb er simulerede. Ingen mikrofon, netværkskald eller udklipsholder bliver brugt.</p>${field({ id: "demo-app", label: "Start fra app", value: s.demo.app, options: Object.entries(s.appNames) })}${field(
        {
          id: "demo-case",
          label: "Forløb",
          value: s.demo.outcome,
          options: [
            ["normal", "Uændret fokus"],
            ["focus", "Fokus ændret"],
            ["asr", "Transskription fejler"],
            ["ai", "AI fejler → original"],
          ],
        },
      )}${field({
        id: "demo-processing",
        label: "Tekst denne gang",
        value: s.demo.processing,
        options: [
          ["default", "Brug app-indstilling"],
          ["direct", "Direkte transskription"],
          ["general", "Brug generel systemprompt"],
        ],
      })}<div id="dialog-error"></div><div class="actions">${btn("Start optagelse (demo)", "start-session", { variant: "primary", glyph: "mic" })}</div><div class="demo-secondary">${btn("Prøv første opsætning", "open-setup")}${btn("Vis tom historik", "empty-history", { variant: "quiet" })}${btn("Vis lang tekst", "long-result", { variant: "quiet" })}${btn("Gendan eksempeldata", "reset-demo", { variant: "quiet" })}</div>`;
    }
    if (s.modal === "key") {
      title = "Tilføj din egen nøgle";
      body = `<form id="key-form" novalidate><p class="muted">Kun en fiktiv nøgle. Ingen konto kontaktes, og ingen nøgle gemmes.</p>${field({ id: "key-provider", label: "Udbyder", value: s.keyProvider || "OpenAI", options: ["OpenAI", "Groq", "ElevenLabs", "Anthropic", "Gemini"].map((n) => [n, n]) })}${field({ id: "api-key", label: "API-nøgle", type: "password", attrs: 'autocomplete="off"' })}<div id="key-error"></div><div class="actions">${btn("Afprøv og gem (demo)", "save-key", { variant: "primary" })}${btn("Annuller", "close-modal", { variant: "quiet" })}</div></form>`;
    }
    if (s.modal === "delete") {
      title = "Slet diktering?";
      body = `<p>Både den originale og den færdige tekst slettes i prototypen.</p><div class="actions">${btn("Slet diktering", "confirm-delete", { variant: "destructive", glyph: "trash" })}${btn("Behold", "close-modal")}</div>`;
    }
    if (s.modal === "result-menu") {
      title = "Diktering";
      body = `<div class="result-actions">${btn("Kopiér tekst", "menu-copy", { glyph: "copy" })}${btn("Slet diktering", "ask-delete", { variant: "destructive", glyph: "trash" })}</div>`;
    }
    if (s.modal === "unsaved") {
      title = "Du har ændringer, der ikke er gemt";
      body = `<p>Vil du gemme indstillingen for ${e(s.appNames[s.prefApp])}?</p><div class="actions">${btn("Gem ændringer", "save-and-navigate", { variant: "primary" })}${btn("Kassér ændringer", "discard-and-navigate")}${btn("Bliv her", "close-modal", { variant: "quiet" })}</div>`;
    }
    if (s.modal === "shortcut") {
      title = "Vælg din genvej";
      body = `<p>Tryk en kombination med en modifikationstast. Bindingen afprøves kun i denne prototype.</p><div id="shortcut-recorder" class="shortcut-recorder" tabindex="0" role="group" aria-label="Optag genvej">${s.shortcutDraft ? "<kbd>" + e(s.shortcutDraft) + "</kbd>" : "Tryk din ønskede genvej…"}</div><div id="shortcut-error"></div><div class="actions">${btn("Gem genvej", "save-shortcut", { variant: "primary" })}${btn("Annuller", "close-modal", { variant: "quiet" })}</div>`;
    }
    if (s.modal === "setup") {
      const p = s.setup;
      title = [
        "Dine ord, direkte ind i dit arbejde.",
        "Gør Vara klar",
        "Prøv din første diktering",
      ][p.step];
      if (p.step === 0)
        body = `<p class="muted">Ingen konto. Lokal behandling som standard.</p><div class="setup-options">${btn("Start på denne enhed", "setup-local", { cls: p.cloud ? "" : "is-selected", glyph: "shield" })}<small>Anbefalet · modellen hentes gennem Vara.</small>${btn("Jeg vil bruge cloud", "setup-cloud", { cls: p.cloud ? "is-selected" : "", glyph: "globe" })}</div><div class="actions">${btn("Fortsæt", "setup-next", { variant: "primary" })}</div>`;
      if (p.step === 1)
        body = `<p class="muted">Modeldownload, tilladelser og genvejstest er simulationer.</p>${p.cloud ? btn(p.key ? "Cloudnøgle klar · demo" : "Tilføj fiktiv nøgle", "setup-key") : btn(p.model ? "Model klar · demo" : p.download ? "Henter model…" : "Hent lokal model (demo)", "setup-download", { glyph: "download", attrs: p.download ? "disabled" : "" })}${p.download ? '<progress aria-label="Modeldownload (simuleret)"></progress>' + btn("Annuller download", "cancel-download", { variant: "quiet" }) : ""}<div class="permission-row"><span>Mikrofon</span>${btn(p.mic ? "Tilladt · demo" : "Giv adgang (demo)", "setup-mic")}</div><div class="permission-row"><span>Indsættelse i andre apps</span>${btn(p.access ? "Tilladt · demo" : "Giv adgang (demo)", "setup-access")}</div><div class="permission-row"><span>Genvej <kbd>${e(s.settings.shortcut)}</kbd></span>${btn(p.shortcut ? "Afprøvet · demo" : "Afprøv genvej (demo)", "setup-shortcut")}</div><div class="actions">${btn("Prøv en diktering", "setup-ready", { variant: "primary", attrs: (p.cloud ? !p.key : !p.model) || !p.mic || !p.access || !p.shortcut ? "disabled" : "" })}${btn("Tilbage", "setup-back", { variant: "quiet" })}</div>`;
      if (p.step === 2)
        body = `<p class="muted">Hold <kbd>${e(s.settings.shortcut)}</kbd>, tal og slip. Brug knappen i denne prototype.</p><div class="trial-result">${p.trial ? "Nu kan jeg bruge min stemme, dér hvor jeg arbejder." : "Din første sætning vises her."}</div><div class="actions">${btn("Simulér en diktering", "setup-trial", { variant: p.trial ? "secondary" : "primary", attrs: p.trial ? "disabled" : "" })}${btn("Åbn Vara", "setup-finish", { variant: p.trial ? "primary" : "secondary", attrs: p.trial ? "" : "disabled" })}</div>`;
    }
    return `<div class="dialog-content">${close}<h2 id="dialog-title">${title}</h2>${body}</div>`;
  }
  Vara.views = { sidebar, history, apps, settings, hud, dialog, locality };
})();
