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
    `${e(r.date)}${r.duration ? " · " + r.duration + " sec." : ""}`;
  const locationNames = {
    on_device: "On this device",
    lan: "On your network",
    cloud: "At selected provider",
  };
  const engineOptions = [
    ["local", "On device · local text AI"],
    ["cloud", "Cloud · your API key"],
    ["cli", "Installed CLI · your login"],
  ];
  const locality = (engine) =>
    engine === "local"
      ? "The text stays on this device."
      : "The text is sent to the selected provider. This requires your explicit choice.";
  const statusNotice = (r) =>
    `${r.fallback ? notice("AI processing failed. The original is preserved.", "warning") : ""}${r.status === "manual" ? notice("Focus changed. The text is ready to paste manually.", "warning") : ""}`;
  function sidebar(s) {
    const remoteText = s.active
      ? s.active.processing && s.active.engine !== "local"
      : Object.values(s.prefs).some((p) => p.enabled && p.engine !== "local");
    return `<div class="window-controls" aria-hidden="true"><i></i><i></i><i></i></div><div class="brand">${mark()}<span>vara</span></div>
      <nav>${[
        ["history", "history", "History"],
        ["apps", "apps", "AI per app"],
        ["settings", "settings", "Settings"],
      ]
        .map(
          ([p, g, l]) =>
            `<button class="nav-item ${s.page === p ? "is-current" : ""}" data-action="navigate" data-page="${p}" ${s.page === p ? 'aria-current="page"' : ""}>${icon(g)}<span>${l}</span></button>`,
        )
        .join("")}</nav>
      <div class="sidebar-footer"><div class="sidebar-status"><span class="status-dot"></span><span>${s.settings.speech === "local" ? (remoteText ? "Audio: on device" : "On device") : "Audio: cloud selected"}</span><kbd id="nav-shortcut">${e(s.settings.shortcut)}</kbd></div><p id="nav-style">${s.settings.style === "hold" ? "Hold, speak, and release." : "Press to start / stop."}</p><p>${s.settings.speech === "local" ? "Local transcription.<br>Your audio stays on this device." : "Audio is processed by your selected provider."}</p>${remoteText ? "<p>Text AI: external or unknown location.</p>" : ""}</div>`;
  }
  function row(r, s) {
    return `<li class="history-row ${r.id === s.selected ? "is-selected" : ""}"><button class="row-select" data-action="select-result" data-id="${r.id}" aria-pressed="${r.id === s.selected}"><span class="row-preview">${e(r.text || r.raw)}</span><span class="row-meta">${date(r)}${r.status === "manual" ? '<span class="row-status"> · Ready to paste</span>' : ""}</span></button>${ib("more", "Dictation actions", "result-menu", `data-id="${r.id}"`)}</li>`;
  }
  function recovery(s) {
    return s.recoveries.length
      ? `<div class="recovery-list">${s.recoveries.map((r) => `<section class="recovery-task"><h3>${icon("warning")}Transcription from ${e(s.appNames[r.app])} failed</h3><p>The recording expires ${new Date(r.expires).toLocaleString("en-US", { hour: "2-digit", minute: "2-digit", day: "numeric", month: "short" })}. Retrying does not extend the deadline.</p><div class="actions">${btn("Try again", "retry", { attrs: `data-id="${r.id}"` })}${btn("Delete recording", "delete-recovery", { variant: "quiet", attrs: `data-id="${r.id}"` })}</div></section>`).join("")}</div>`
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
    return `<section class="history-pane" aria-label="History"><header class="pane-toolbar"><h1 id="view-title" tabindex="-1">History</h1>${ib("search", "Search history", "toggle-search")}</header>
      ${s.searchOpen ? `<div class="history-search">${field({ id: "history-search", label: "Search history", type: "search", value: s.query })}${field({ id: "history-app", label: "Started from app", value: s.filter, options: [["all", "All apps"], ...Object.entries(s.appNames)] })}</div>` : ""}
      <div class="history-scroll" id="history-scroll">${recovery(s)}${groups.map(([app, rows]) => `<section class="app-group" aria-label="${e(s.appNames[app])}"><button class="group-header" data-action="toggle-group" data-app="${app}" aria-expanded="${!s.collapsed.includes(app)}">${appIcon(app)}<span class="group-name">${e(s.appNames[app])}</span><span class="group-count">${rows.length} ${rows.length === 1 ? "dictation" : "dictations"}</span>${icon("chevron", s.collapsed.includes(app) ? "is-closed" : "")}</button>${s.collapsed.includes(app) ? "" : `<ul class="history-rows">${rows.map((r) => row(r, s)).join("")}</ul>`}</section>`).join("") || empty(s.query ? "No matches" : "Your history starts here", s.query ? "Try another word or app." : `Hold ${s.settings.shortcut}, speak, and release. Your dictations appear here.`, btn(s.query ? "Clear search" : "Try a dictation", s.query ? "clear-search" : "open-demo"))}</div></section>
      <section class="detail-pane ${s.detailOpen ? "is-open" : ""}" aria-label="Selected dictation">${selected ? detail(selected, s) : empty("Your words, all in one place", "Select a dictation in your history to see its text.")}</section>`;
  }
  function detail(r, s) {
    const compact = (r.text || r.raw).length > 600;
    return `<button class="back-button" data-action="back-history">${icon("back")}Back to history</button>
      <header class="detail-header"><div class="detail-identity">${appIcon(r.app, "large")}<div><h2>${e(s.appNames[r.app])}</h2><p class="muted">${date(r)}</p></div></div>${ib("more", "Selected dictation actions", "result-menu", `data-id="${r.id}"`)}</header>
      <div class="detail-scroll"><p class="transcript ${compact ? "is-long" : ""}" dir="auto">${e(r.text || r.raw)}</p>${statusNotice(r)}
      <div class="copy-bar">${btn("Copy", "copy-result", { variant: "primary", glyph: "copy", attrs: `data-id="${r.id}"` })}</div>
      ${r.processed ? `<details class="original-disclosure" ${s.originalOpen ? "open" : ""}><summary>${icon("chevron")}<span>Show original</span></summary><div class="original-panel"><p>${e(r.raw)}</p><p class="original-meta">${r.duration ? r.duration + " sec. · " : ""}Original transcript · no audio saved</p></div></details>` : ""}
      <div class="result-footnote"><span>${r.local ? "Processed on device" : "Processed by selected provider"}</span><span>${s.settings.cleanup ? "Automatic cleanup after " + s.settings.days + " days" : "Saved until you delete it"}</span></div></div>`;
  }
  function apps(s) {
    const p = s.prefs[s.prefApp];
    return `<div class="page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">AI per app</h1><p class="muted">Direct by default. Customize the apps you choose.</p></header><div class="app-preferences"><nav class="app-selector" aria-label="Choose app">${Object.entries(
      s.appNames,
    )
      .map(
        ([a, n]) =>
          `<button data-action="select-pref" data-app="${a}" class="app-choice ${a === s.prefApp ? "is-selected" : ""}" ${a === s.prefApp ? 'aria-current="true"' : ""}>${appIcon(a)}${e(n)}</button>`,
      )
      .join("")}</nav><form id="app-form" class="preference-form" novalidate>
      <div class="form-title">${appIcon(s.prefApp, "large")}<h2>${e(s.appNames[s.prefApp])}</h2></div>${toggle("app-ai", "AI processing", p.enabled, "Automatically use this app’s system prompt when you dictate here.")}
      ${field({ id: "app-prompt", label: "System prompt", value: p.prompt, rows: true, help: "Preserve the message, names, numbers, and my tone. The original is saved." })}
      ${field({ id: "app-engine", label: "AI for text processing", value: p.engine, options: engineOptions, help: locality(p.engine) })}
      <div id="app-form-message" class="form-message"></div><div class="actions">${btn("Save app setting", "save-app", { variant: "primary" })}${btn("Discard changes", "reset-app", { variant: "quiet" })}<span class="unsaved" id="unsaved" ${s.dirty ? "" : "hidden"}>Unsaved</span></div>
      <p class="form-footnote">No extra confirmation when dictating. Automatic insertion if focus is unchanged.</p></form></div></div>`;
  }
  function settings(s) {
    const p = s.settings;
    let body = "";
    if (s.settingsTab === "dictation")
      body =
        section(
          "Dictate your way",
          "The shortcut works in the app where you’re working.",
          field({
            id: "activation-style",
            label: "Activation",
            value: p.style,
            options: [
              ["hold", "Hold to speak"],
              ["toggle", "Press to start / stop"],
            ],
          }) +
            field({
              id: "input-device",
              label: "Microphone",
              value: p.device,
              options: [
                ["Indbygget mikrofon", "Built-in microphone (demo)"],
                ["Ekstern mikrofon", "External microphone (demo)"],
              ],
            }) +
            `<div class="shortcut-setting"><div><span class="field-label">Your shortcut</span><p class="muted">The prototype does not register a system shortcut.</p></div><kbd>${e(p.shortcut)}</kbd>${btn("Change shortcut", "record-shortcut")}</div>`,
        ) +
        section(
          "General system prompt",
          "Used when you choose AI for the next dictation.",
          field({
            id: "general-prompt",
            label: "Your instructions",
            value: p.general,
            rows: true,
          }) + btn("Save prompt", "save-general", { variant: "primary" }),
        );
    if (s.settingsTab === "connections")
      body =
        section(
          "Speech to text",
          "On device by default. Cloud when you choose it.",
          field({
            id: "speech-choice",
            label: "Audio processing",
            value: p.speech,
            options: [
              ["local", "On device"],
              ["cloud", "Cloud · your key"],
            ],
          }) +
            notice(
              p.speech === "local"
                ? "Recommended local speech model · ready in the demo. No audio is sent to the cloud."
                : "Cloud is selected. Add your own key for a speech provider.",
              p.speech === "local" ? "success" : "info",
            ) +
            toggle(
              "strict-local",
              "Process only on this device",
              p.strictLocal,
              "Reject cloud, LAN, and unknown locations for audio and text.",
            ),
        ) +
        section(
          "Connections",
          "Your own keys and provider accounts.",
          `<div class="provider-list">${["OpenAI", "Groq", "ElevenLabs", "Anthropic", "Gemini"].map((p) => `<div class="provider-row"><div><strong>${p}</strong><small>${p === "ElevenLabs" ? "Speech" : ["OpenAI", "Groq"].includes(p) ? "Speech and text" : "Text"}</small></div>${btn(s.keyConfigured[p] ? "Key added · demo" : "Add key", "open-key", { attrs: `data-provider="${p}"`, glyph: "key" })}</div>`).join("")}</div>`,
        ) +
        section(
          "Local services and custom endpoints",
          "Ollama and compatible text models. Location is an explicit choice.",
          s.endpoints
            .map((x) =>
              notice(
                x.name + " · " + x.model + " · " + locationNames[x.locality],
                "success",
              ),
            )
            .join("") +
            `<form id="endpoint-form" novalidate>${field({ id: "endpoint-name", label: "Name" })}${field({ id: "endpoint-url", label: "Endpoint-URL", type: "url", attrs: 'placeholder="http://localhost:11434/v1"' })}${field({ id: "endpoint-model", label: "Model" })}${field(
              {
                id: "endpoint-locality",
                label: "Where is the text processed?",
                value: "on_device",
                options: [
                  ["on_device", "On this device"],
                  ["lan", "On your network"],
                  ["cloud", "At an external provider"],
                ],
              },
            )}${field({ id: "endpoint-key", label: "API key (optional)", type: "password", help: "Only a fictional key in this prototype.", attrs: 'autocomplete="off"' })}<div id="endpoint-error"></div>${btn("Test connection (demo)", "save-endpoint", { variant: "primary" })}</form>`,
        ) +
        section(
          "Installed CLI tools",
          "Optional text models using the tool’s own login. The model may run at an external provider.",
          `<div class="provider-list">${["Claude Code", "Codex", "Gemini CLI"].map((n) => `<div class="provider-row"><strong>${n}</strong><span class="muted">Not checked in the demo</span></div>`).join("")}</div>`,
        );
    if (s.settingsTab === "privacy")
      body =
        section(
          "Your text history",
          "The original and finished text belong to the same result.",
          toggle(
            "cleanup-toggle",
            "Automatic cleanup",
            p.cleanup,
            "Off by default. Otherwise, text is saved until you delete it.",
          ) +
            `<div id="cleanup-period" ${p.cleanup ? "" : "hidden"}>${field({ id: "cleanup-days", label: "Delete text after this many days", value: p.days, type: "number", attrs: 'min="1" step="1"' })}</div><div id="retention-error"></div>` +
            btn("Save cleanup", "save-retention", { variant: "primary" }),
        ) +
        section(
          "No audio in history",
          "Audio is deleted after successful transcription. If it fails, audio is temporarily retained for retry for up to 24 hours. Retrying does not extend the deadline.",
          notice(
            "The recording is deleted sooner after a successful retry or manual deletion.",
            "info",
          ),
        ) +
        section(
          "Where are your words processed?",
          "Audio and any AI text processing each have their own destination.",
          `<div class="provider-row"><span>Audio → text</span><strong>${p.speech === "local" ? "On device" : "Selected cloud provider"}</strong></div><div class="provider-row"><span>Text → AI</span><strong>Only if you choose it</strong></div>`,
        );
    return `<div class="page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">Settings</h1><p class="muted">A good default, with choices when you need them.</p></header><nav class="settings-nav" aria-label="Settings sections">${[
      ["dictation", "Dictation"],
      ["connections", "Models & connections"],
      ["privacy", "History & privacy"],
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
      recording: "Listening",
      speech: "Finding your words",
      text: "Refining the text",
    };
    return `<div class="recording-hud ${a?.stage === "recording" ? "is-recording" : ""}"><div class="hud-origin">${appIcon(a?.app || s.demo.app)}<span>${e(s.appNames[a?.app || s.demo.app])}</span></div>${wave()}<div class="hud-status"><span>${a ? labels[a.stage] : e(s.hudMessage || "Ready")}</span><small>${a ? (a.stage === "recording" ? (a.processing ? "AI selected" : "Direct") : a.local ? "On device" : "Selected provider") : "Example"}</small></div>${a ? `${a.stage === "recording" ? ib("stop", "Stop dictation", "stop-session") : ""}${ib("close", "Cancel dictation", "cancel-session")}` : ib("mic", "Try dictation", "open-demo")}</div>`;
  }
  function dialog(s) {
    const close = ib("close", "Close dialog", "close-modal");
    let title = "",
      body = "";
    if (s.modal === "demo") {
      title = "Try dictation flows";
      body = `<p class="muted">All flows are simulated. No microphone, network calls, or clipboard are used.</p>${field({ id: "demo-app", label: "Start from app", value: s.demo.app, options: Object.entries(s.appNames) })}${field(
        {
          id: "demo-case",
          label: "Flow",
          value: s.demo.outcome,
          options: [
            ["normal", "Focus unchanged"],
            ["focus", "Focus changed"],
            ["asr", "Transcription fails"],
            ["ai", "AI fails → original"],
          ],
        },
      )}${field({
        id: "demo-processing",
        label: "Text this time",
        value: s.demo.processing,
        options: [
          ["default", "Use app setting"],
          ["direct", "Direct transcription"],
          ["general", "Use general system prompt"],
        ],
      })}<div id="dialog-error"></div><div class="actions">${btn("Start recording (demo)", "start-session", { variant: "primary", glyph: "mic" })}</div><div class="demo-secondary">${btn("Try first-time setup", "open-setup")}${btn("Show empty history", "empty-history", { variant: "quiet" })}${btn("Show long text", "long-result", { variant: "quiet" })}${btn("Restore sample data", "reset-demo", { variant: "quiet" })}</div>`;
    }
    if (s.modal === "key") {
      title = "Add your own key";
      body = `<form id="key-form" novalidate><p class="muted">Use a fictional key only. No account is contacted and no key is saved.</p>${field({ id: "key-provider", label: "Provider", value: s.keyProvider || "OpenAI", options: ["OpenAI", "Groq", "ElevenLabs", "Anthropic", "Gemini"].map((n) => [n, n]) })}${field({ id: "api-key", label: "API key", type: "password", attrs: 'autocomplete="off"' })}<div id="key-error"></div><div class="actions">${btn("Test and save (demo)", "save-key", { variant: "primary" })}${btn("Cancel", "close-modal", { variant: "quiet" })}</div></form>`;
    }
    if (s.modal === "delete") {
      title = "Delete dictation?";
      body = `<p>Both the original and finished text are deleted in the prototype.</p><div class="actions">${btn("Delete dictation", "confirm-delete", { variant: "destructive", glyph: "trash" })}${btn("Keep", "close-modal")}</div>`;
    }
    if (s.modal === "result-menu") {
      title = "Dictation";
      body = `<div class="result-actions">${btn("Copy text", "menu-copy", { glyph: "copy" })}${btn("Delete dictation", "ask-delete", { variant: "destructive", glyph: "trash" })}</div>`;
    }
    if (s.modal === "unsaved") {
      title = "You have unsaved changes";
      body = `<p>Save the setting for ${e(s.appNames[s.prefApp])}?</p><div class="actions">${btn("Save changes", "save-and-navigate", { variant: "primary" })}${btn("Discard changes", "discard-and-navigate")}${btn("Stay here", "close-modal", { variant: "quiet" })}</div>`;
    }
    if (s.modal === "shortcut") {
      title = "Choose your shortcut";
      body = `<p>Press a combination with a modifier key. The shortcut is tested only in this prototype.</p><div id="shortcut-recorder" class="shortcut-recorder" tabindex="0" role="group" aria-label="Record shortcut">${s.shortcutDraft ? "<kbd>" + e(s.shortcutDraft) + "</kbd>" : "Press your desired shortcut…"}</div><div id="shortcut-error"></div><div class="actions">${btn("Save shortcut", "save-shortcut", { variant: "primary" })}${btn("Cancel", "close-modal", { variant: "quiet" })}</div>`;
    }
    if (s.modal === "setup") {
      const p = s.setup;
      title = [
        "Your words, directly into your work.",
        "Set up Vara",
        "Try your first dictation",
      ][p.step];
      if (p.step === 0)
        body = `<p class="muted">No account. Local processing by default.</p><div class="setup-options">${btn("Start on this device", "setup-local", { cls: p.cloud ? "" : "is-selected", glyph: "shield" })}<small>Recommended · the model is downloaded through Vara.</small>${btn("I want to use the cloud", "setup-cloud", { cls: p.cloud ? "is-selected" : "", glyph: "globe" })}</div><div class="actions">${btn("Continue", "setup-next", { variant: "primary" })}</div>`;
      if (p.step === 1)
        body = `<p class="muted">Model downloads, permissions, and shortcut tests are simulated.</p>${p.cloud ? btn(p.key ? "Cloud key ready · demo" : "Add fictional key", "setup-key") : btn(p.model ? "Model ready · demo" : p.download ? "Downloading model…" : "Download local model (demo)", "setup-download", { glyph: "download", attrs: p.download ? "disabled" : "" })}${p.download ? '<progress aria-label="Model download (simulated)"></progress>' + btn("Cancel download", "cancel-download", { variant: "quiet" }) : ""}<div class="permission-row"><span>Microphone</span>${btn(p.mic ? "Allowed · demo" : "Grant access (demo)", "setup-mic")}</div><div class="permission-row"><span>Insertion in other apps</span>${btn(p.access ? "Allowed · demo" : "Grant access (demo)", "setup-access")}</div><div class="permission-row"><span>Shortcut <kbd>${e(s.settings.shortcut)}</kbd></span>${btn(p.shortcut ? "Tested · demo" : "Test shortcut (demo)", "setup-shortcut")}</div><div class="actions">${btn("Try a dictation", "setup-ready", { variant: "primary", attrs: (p.cloud ? !p.key : !p.model) || !p.mic || !p.access || !p.shortcut ? "disabled" : "" })}${btn("Back", "setup-back", { variant: "quiet" })}</div>`;
      if (p.step === 2)
        body = `<p class="muted">Hold <kbd>${e(s.settings.shortcut)}</kbd>, speak, and release. Use the button in this prototype.</p><div class="trial-result">${p.trial ? "Now I can use my voice wherever I work." : "Your first sentence appears here."}</div><div class="actions">${btn("Simulate a dictation", "setup-trial", { variant: p.trial ? "secondary" : "primary", attrs: p.trial ? "disabled" : "" })}${btn("Open Vara", "setup-finish", { variant: p.trial ? "primary" : "secondary", attrs: p.trial ? "" : "disabled" })}</div>`;
    }
    return `<div class="dialog-content">${close}<h2 id="dialog-title">${title}</h2>${body}</div>`;
  }
  Vara.views = { sidebar, history, apps, settings, hud, dialog, locality };
})();
