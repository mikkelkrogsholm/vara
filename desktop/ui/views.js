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
  const { bind: tr, bindDate, bindNumber } = Vara.i18n;
  const date = (r) => bindDate(r.created_at * 1000, { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" }) + (r.duration_secs ? " · " + tr("history.seconds", { count: Math.round(r.duration_secs) }) : "");
  const finished = (r) => r.processed ?? r.original;
  const loc = {
    on_device: Vara.i18n.bind("ui.on-this-device"),
    lan: Vara.i18n.bind("ui.on-your-network"),
    cloud: Vara.i18n.bind("ui.at-selected-provider"),
    unknown: Vara.i18n.bind("ui.external-or-unknown-location"),
  };
  const phase = {
    recording: Vara.i18n.bind("ui.listening"),
    recognizing: Vara.i18n.bind("ui.transcribing"),
    processing: Vara.i18n.bind("ui.processing-text"),
    committing: Vara.i18n.bind("ui.saving"),
  };
  const originName = origin => origin.app_id === "unknown" ? Vara.i18n.bindRuntime('vara-message-v1:{"code":"app.unknown"}') : origin.app_name;
  const appName = (id, name) => originName({app_id: id, app_name: name});
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
      ["history", "history", Vara.i18n.bind("ui.history")],
      ["transcription", "document", Vara.i18n.bind("ui.transcription")],
      ["apps", "apps", Vara.i18n.bind("ui.ai-per-app")],
      ["settings", "settings", Vara.i18n.bind("ui.settings")],
    ]
      .map(
        ([page, glyph, label]) =>
          `<button class="nav-item ${s.page === page ? "is-current" : ""}" data-action="navigate" data-page="${page}" ${s.page === page ? 'aria-current="page"' : ""}>${icon(glyph)}<span>${label}</span></button>`,
      )
      .join(
        "",
      )}</nav><div class="sidebar-footer"><div class="sidebar-status"><span class="status-dot ${installed && micReady && local ? "" : "is-unready"}"></span><span>${tr(`sidebar.${s.page === "transcription" ? "dictation" : "audio"}.${local ? "local" : "cloud"}`)}</span><kbd>${e(binding(p.shortcut))}</kbd></div><p>${s.data.next_choice && s.data.next_choice !== "default" ? tr(s.data.next_choice === "direct" ? "sidebar.next.direct" : "sidebar.next.ai") + "<br>" : ""}${p.activation === "hold" ? Vara.i18n.bind("ui.hold-speak-and-release") : Vara.i18n.bind("ui.press-to-start-stop")}</p><p>${s.page === "transcription" ? tr("sidebar.file.hint") : local ? tr("sidebar.local.hint") : Vara.i18n.bind("ui.audio-is-processed-by-your-selected-provider")}</p>${textEngine ? `<p>${tr("history.text", { location: loc[textEngine.locality] })}</p>` : Object.values(p.app_rules).some((a) => a.enabled) ? ("<p>"+Vara.i18n.bind("ui.ai-follows-your-app-rules")+"</p>") : ""}${!installed || !micReady ? b(!installed ? Vara.i18n.bind("ui.download-speech-model") : Vara.i18n.bind("ui.finish-setup"), "setup", { variant: "quiet", cls: "sidebar-setup" }) : ""}<div class="sidebar-dictation-controls">${b(s.data.session?.stage === "recording" ? Vara.i18n.bind("ui.stop") : Vara.i18n.bind("ui.dictate"), s.data.session?.stage === "recording" ? "stop" : "start", { variant: "quiet", glyph: s.data.session?.stage === "recording" ? "stop" : "mic", attrs: s.data.session && s.data.session.stage !== "recording" ? "disabled" : "" })}${ib("more", Vara.i18n.bind("ui.processing-for-next-dictation"), "choose-processing")}</div></div>`;
  }
  function updateNotice(s) {
    const u = s.update;
    if (!u) return "";
    const installable = u.version && ["available", "ready", "blocked", "error"].includes(u.status);
    return notice(tr(`update.${u.status}`, {version:u.version || ""}), "info",
      installable ? b(Vara.i18n.bind(u.downloaded ? "update.install" : "update.download"), u.downloaded ? "update-install" : "update-download", {attrs: s.data.session || s.dirty ? "disabled" : ""}) : "");
  }
  function messages(s) {
    return `${s.update?.version ? updateNotice(s) : ""}${s.uiError?.scope === "global" ? notice(s.uiError.message, "error", b(Vara.i18n.bind("ui.close-message"), "dismiss-ui-error", { variant: "quiet" })) : ""}${s.data.message ? `<div class="inline-state">${icon("warning")}<p>${e(Vara.i18n.bindRuntime(s.data.message))}</p>${ib("close", Vara.i18n.bind("ui.close-message"), "dismiss_message")}</div>` : ""}${s.data.session ? `<div class="inline-state session-state" role="status">${icon("mic")}<p>${e(phase[s.data.session.stage] || Vara.i18n.bind("ui.dictation"))} · ${e(originName(s.data.session.origin))}</p>${s.data.session.stage === "recording" ? b(Vara.i18n.bind("ui.stop"), "stop", { glyph: "stop" }) : ""}${b(Vara.i18n.bind("ui.cancel"), "cancel", { variant: "quiet" })}</div>` : ""}${s.data.unsaved ? `<section class="pending-save"><h2>${Vara.i18n.bind("ui.text-has-not-been-saved-yet")}</h2><p class="transcript is-long">${e(finished(s.data.unsaved))}</p><div class="actions">${b(Vara.i18n.bind("ui.copy"), "copy-unsaved", { variant: "primary" })}${b(Vara.i18n.bind("ui.try-saving-again"), "save_unsaved")}${b(Vara.i18n.bind("ui.discard"), "discard-unsaved", { variant: "quiet" })}</div></section>` : ""}`;
  }
  function recovery(s) {
    return s.data.recoveries
      .map(
        (r) =>
          `<section class="recovery-task"><h3>${icon("warning")}${tr("recovery.failed", { app: originName(r.origin) })}</h3><p>${tr("recovery.expiry", { date: bindDate(r.expires_at * 1000) })}</p><div class="actions">${b(Vara.i18n.bind("ui.try-again"), "retry", { attrs: `data-id="${e(r.id)}"` })}${b(Vara.i18n.bind("ui.delete-recording"), "delete-recovery", { variant: "quiet", attrs: `data-id="${e(r.id)}"` })}</div></section>`,
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
        appName(id, name),
        rows.filter((r) => r.origin.app_id === id),
      ])
      .filter((x) => x[2].length);
    return `<section class="history-pane" aria-label="${Vara.i18n.bind("ui.history")}"><header class="pane-toolbar"><h1 id="view-title" tabindex="-1">${Vara.i18n.bind("ui.history")}</h1><span class="compact-dictation-controls">${ib(s.data.session?.stage === "recording" ? "stop" : "mic", s.data.session?.stage === "recording" ? Vara.i18n.bind("ui.stop-dictation") : Vara.i18n.bind("ui.start-dictation"), s.data.session?.stage === "recording" ? "stop" : "start", s.data.session && s.data.session.stage !== "recording" ? "disabled" : "")}${ib("more", Vara.i18n.bind("ui.processing-for-next-dictation"), "choose-processing")}</span>${ib("search", Vara.i18n.bind("ui.search-history"), "toggle-search")}</header>${s.searchOpen ? `<div class="history-search">${f({ id: "history-search", label: Vara.i18n.bind("ui.search-history"), type: "search", value: s.query })}${f({ id: "history-app", label: Vara.i18n.bind("ui.started-from-app"), value: s.filter, options: [["all", Vara.i18n.bind("ui.all-apps")], ...apps(s).map(([id,name]) => [id,appName(id,name)])] })}</div>` : ""}<div class="history-scroll"><div class="activity-region">${messages(s)}</div>${recovery(s)}${groups.map(([id, name, rows]) => `<section class="app-group" aria-label="${e(name)}"><button class="group-header" data-action="toggle-group" data-id="${e(id)}" aria-expanded="${!s.collapsed.has(id)}">${appIcon(name, "", id)}<span class="group-name">${e(name)}</span><span class="group-count">${tr("history.count", { count: rows.length })}</span>${icon("chevron", s.collapsed.has(id) ? "is-closed" : "")}</button>${s.collapsed.has(id) ? "" : `<ul class="history-rows">${rows.map((r) => `<li class="history-row ${r.id === s.selected ? "is-selected" : ""}"><button class="row-select" data-action="select-result" data-id="${e(r.id)}" aria-pressed="${r.id === s.selected}"><span class="row-preview">${e(finished(r))}</span><span class="row-meta">${date(r)}${r.delivery === "inserted" ? "" : tr("history.ready")}</span></button>${resultActions(r.id)}</li>`).join("")}</ul>`}</section>`).join("") || empty(s.query ? Vara.i18n.bind("ui.no-matches") : Vara.i18n.bind("ui.your-history-starts-here"), s.query ? Vara.i18n.bind("ui.try-another-word-or-app") : tr("history.empty.hint", { shortcut: binding(s.data.settings.shortcut) }), b(s.query ? Vara.i18n.bind("ui.clear-search") : Vara.i18n.bind("ui.get-started"), s.query ? "clear-search" : "setup"))}</div></section><section class="detail-pane ${s.detailOpen ? "is-open" : ""}" aria-label="${Vara.i18n.bind("ui.selected-dictation")}">${selected ? detail(selected, s) : empty(Vara.i18n.bind("ui.your-words-in-one-place"), Vara.i18n.bind("ui.select-a-dictation-in-your-history-to-view-its-text"))}</section>`;
  }
  function detail(r, s) {
    return `<button class="back-button" data-action="back-history">${icon("back")}${tr("ui.back-to-history")}</button><header class="detail-header"><div class="detail-identity">${appIcon(originName(r.origin), "large", r.origin.app_id)}<div><h2>${e(originName(r.origin))}</h2><p class="muted">${date(r)}</p></div></div>${resultActions(r.id)}</header><div class="detail-scroll">${s.uiError?.scope === "detail" ? notice(s.uiError.message, "error", b(Vara.i18n.bind("ui.close-message"), "dismiss-ui-error", { variant: "quiet" })) : ""}<p class="transcript ${finished(r).length > 600 ? "is-long" : ""}" dir="auto">${e(finished(r))}</p>${r.fallback ? notice(Vara.i18n.bind("ui.ai-processing-did-not-finish-the-original-is-preserved"), "warning") : ""}${r.delivery !== "inserted" ? notice(r.delivery === "unknown" ? Vara.i18n.bind("ui.insertion-could-not-be-confirmed-check-the-field-before-inserting") : Vara.i18n.bind("ui.the-text-is-ready-to-insert-manually"), "info") : ""}<div class="copy-bar">${b(Vara.i18n.bind("ui.copy"), "copy", { variant: "primary", glyph: "copy", attrs: `data-id="${e(r.id)}"` })}</div>${r.processed !== null ? `<details class="original-disclosure" ${s.originalOpen ? "open" : ""}><summary>${icon("chevron")}<span>${s.originalOpen ? Vara.i18n.bind("ui.hide-original") : Vara.i18n.bind("ui.show-original")}</span></summary><div class="original-panel"><p dir="auto">${e(r.original)}</p><p class="original-meta">${Vara.i18n.bind("ui.original-transcription-no-audio-in-history")}</p>${b(Vara.i18n.bind("ui.copy-original"), "copy-original", { variant: "quiet", attrs: `data-id="${e(r.id)}"` })}</div></details>` : ""}<div class="result-footnote"><span>${r.plan.text ? tr("history.audio.text", { audio: loc[r.plan.speech.locality], text: loc[r.plan.text.engine.locality] }) : tr("history.audio", { location: loc[r.plan.speech.locality] })}</span><span>${s.data.settings.retention_days ? tr("history.retention", { count: s.data.settings.retention_days }) : Vara.i18n.bind("ui.saved-until-you-delete-it")}</span></div></div>`;
  }
  function appPreferences(s) {
    const available = apps(s).map(([id,name]) => [id,appName(id,name)]),
      name = available.find((a) => a[0] === s.prefApp)?.[1];
    const p = s.data.settings.app_rules[s.prefApp] || {
      enabled: false,
      prompt: s.data.settings.general_prompt,
      engine_id: s.data.settings.general_engine,
    };
    return `<div class="page-scroll"><header class="page-header"><h1 id="view-title" tabindex="-1">${Vara.i18n.bind("ui.ai-per-app")}</h1><p class="muted">${Vara.i18n.bind("ui.direct-by-default-customize-the-apps-you-choose")}</p></header><div class="activity-region">${messages(s)}</div><div class="app-add-action">${b(Vara.i18n.bind("ui.add-app"), "add-app", { variant: "quiet", glyph: "apps" })}</div>${available.length ? `<div class="app-preferences"><nav class="app-selector" aria-label="${Vara.i18n.bind("ui.select-app")}">${available.map(([id, name]) => `<button class="app-choice ${s.prefApp === id ? "is-selected" : ""}" data-action="select-pref" data-id="${e(id)}">${appIcon(name, "", id)}${e(name)}</button>`).join("")}</nav>${name ? `<form id="app-form" class="preference-form"><div class="form-title">${appIcon(name, "large", s.prefApp)}<h2>${e(name)}</h2></div>${toggle("app-ai", Vara.i18n.bind("ui.use-custom-ai-settings"), p.enabled, Vara.i18n.bind("ui.turn-off-to-follow-the-dictation-defaults-turn-on-to-use-this-app"))}<div id="app-ai-fields" ${p.enabled ? "" : "hidden"}>${f({ id: "app-prompt", label: Vara.i18n.bind("ui.instructions-for-ai"), value: p.prompt, rows: true, help: Vara.i18n.bind("ui.the-original-is-always-saved-with-the-finished-result") })}${modelPicker(s,"app-engine",Vara.i18n.bind("ui.text-model"),"text",p.engine_id)}</div><div class="form-message" role="status"></div><div class="actions">${b(Vara.i18n.bind("ui.save-app-setting"), "save-app", { variant: "primary", attrs: s.dirty ? "" : "disabled" })}${b(Vara.i18n.bind("ui.discard-changes"), "reset-form", { variant: "quiet" })}<span class="unsaved" ${s.dirty ? "" : "hidden"}>${Vara.i18n.bind("ui.unsaved")}</span></div><p class="form-footnote">${Vara.i18n.bind("ui.no-extra-confirmation-when-dictating-automatically-inserts-if-foc")}</p></form>` : empty(Vara.i18n.bind("ui.select-an-app"), Vara.i18n.bind("ui.select-the-app-whose-text-you-want-to-customize"))}</div>` : empty(Vara.i18n.bind("ui.customize-the-apps-you-use"), Vara.i18n.bind("ui.add-an-app-or-start-dictating-apps-from-your-history-also-appear-"))}</div>`;
  }
  const providersList = [
    ["open_ai", "OpenAI", Vara.i18n.bind("ui.speech-and-text")],
    ["groq", "Groq", Vara.i18n.bind("ui.whisper-for-speech-ai-for-text")],
    ["eleven_labs", "ElevenLabs", Vara.i18n.bind("ui.speech")],
    ["anthropic", "Anthropic", Vara.i18n.bind("ui.text")],
    ["gemini", "Gemini", Vara.i18n.bind("ui.text")],
  ];
  function providers(s) {
    return providersList.map(([id, name, kind]) => {
      const key = s.keyStates[id], test = s.connectionChecks[id];
      const label = key?.error ? Vara.i18n.bind("ui.key-status-is-unavailable") : key?.present === true ? tr(test?.phase === "success" ? "key.saved.tested" : "key.saved.untested") : key?.present === false ? Vara.i18n.bind("ui.no-key-added") : Vara.i18n.bind("ui.checking-key-store");
      const checked = test?.phase === "success";
      const action = id === "open_ai" ? "check-openai" : "check-groq";
      return `<div class="provider-connection" data-provider="${id}"><div class="provider-row"><div class="provider-identity"><strong>${name}</strong><small>${kind}</small></div><div class="provider-credential"><span class="credential-status ${key?.present ? "has-key" : ""}">${key?.present ? icon("check") : ""}${e(label)}</span></div><div class="actions">${b(key?.present === false ? Vara.i18n.bind("ui.add-key") : Vara.i18n.bind("ui.manage-key"), "open-key", { attrs: `data-id="${id}" data-name="${name}"`, glyph: "key" })}${key?.present && ["open_ai", "groq"].includes(id) ? b(test?.phase === "testing" ? Vara.i18n.bind("ui.testing") : Vara.i18n.bind("ui.test-key"), action, { attrs: `data-id="${id}" ${test?.phase === "testing" ? "disabled" : ""}` }) : ""}</div></div><p class="connection-check ${test?.phase === "error" ? "is-error" : checked ? "is-success" : ""}" data-connection-status="${id}" role="status" ${test?.message || key?.error ? "" : "hidden"}>${e(test?.message || key?.error || "")}</p></div>`;
    }).join("");
  }
  function models(s, only = null) {
    const catalog = {
      "local.parakeet": [Vara.i18n.bind("ui.parakeet-redux-speech"), Vara.i18n.bind("ui.multilingual-179-mb")],
      "local.whisper": [Vara.i18n.bind("ui.whisper-base-speech"), Vara.i18n.bind("ui.optional-alternative-148-mb")],
      "local.text": [Vara.i18n.bind("ui.qwen3-0-6b-text"), Vara.i18n.bind("ui.optional-ai-processing-651-mb")],
    };
    return Object.entries(catalog)
      .filter(([id]) => !only || id === only)
      .map(([id, [name, description]]) => {
        const download = s.data.downloads.find((d) => d.id === id);
        return `<div class="model-row"><div class="model-identity"><strong>${e(name)}</strong><small>${e(description)}</small></div><div class="model-state"><small>${download ? tr("model.download", { received: bindNumber(Math.round(download.received / 1000000)), total: bindNumber(Math.round(download.total / 1000000)) }) : s.data.models[id] ? Vara.i18n.bind("ui.installed-on-device") : Vara.i18n.bind("ui.not-downloaded")}</small>${download ? `<progress class="model-progress" data-model-progress="${id}" max="${download.total || 1}" value="${download.received}" aria-label="${Vara.i18n.bind("ui.model-download")}"></progress>` : ""}</div>${b(download ? Vara.i18n.bind("ui.cancel") : s.data.models[id] ? Vara.i18n.bind("ui.check-model") : Vara.i18n.bind("ui.download-model"), download ? "cancel_install" : "install", { attrs: `data-id="${id}"`, glyph: download ? "close" : "download" })}</div>`;
      }).join("");
  }
  function permissions(s) {
    const p = s.data.permissions;
    return `<div class="permission-grid">${permissionRow({ title: Vara.i18n.bind("ui.microphone"), description: p.microphone === "denied" ? Vara.i18n.bind("ui.allow-vara-under-microphone-in-system-privacy-settings") : Vara.i18n.bind("ui.used-only-when-you-start-a-dictation"), kind: "microphone", granted: p.microphone === "authorized", available: p.microphone !== "system" })}${typeof p.input_monitoring === "boolean" ? permissionRow({ title: Vara.i18n.bind("ui.global-shortcuts"), description: Vara.i18n.bind("ui.input-monitoring-lets-vara-detect-your-shortcut-while-you-work-in"), kind: "input_monitoring", granted: p.input_monitoring }) : ""}${permissionRow({ title: Vara.i18n.bind("ui.insertion-in-other-apps"), description: p.automatic_insertion ? Vara.i18n.bind("ui.accessibility-allows-insertion-into-the-original-text-field") : Vara.i18n.bind("ui.on-this-platform-the-text-is-ready-to-insert-manually"), kind: "accessibility", granted: p.accessibility, available: p.automatic_insertion })}</div>`;
  }
  function modelFeedback(s, engine, settings = s.data.settings) {
    if (!engine) return notice(Vara.i18n.bind("ui.select-a-model"), "warning");
    const location = engine.locality === "cloud" ? "Cloud" : loc[engine.locality] || Vara.i18n.bind("ui.unknown-location");
    if (settings.strict_local && engine.locality !== "on_device")
      return notice(Vara.i18n.bind("ui.on-device-processing-only-is-enabled-select-a-local-model-or-chan"), "warning");
    if (Object.hasOwn(s.data.models, engine.id)) {
      const progress = s.data.downloads.find(d => d.id === engine.id);
      return `<div class="model-readiness" role="status"><span>${icon(s.data.models[engine.id] ? "check" : "download")}${e(location)} · ${progress ? tr("model.download", { received: bindNumber(Math.round(progress.received / 1000000)), total: bindNumber(Math.round(progress.total / 1000000)) }) : s.data.models[engine.id] ? Vara.i18n.bind("ui.ready") : Vara.i18n.bind("ui.not-downloaded")}</span>${!s.data.models[engine.id] ? b(progress ? Vara.i18n.bind("ui.cancel") : Vara.i18n.bind("ui.download-model"), progress ? "cancel_install" : "install", { variant: "quiet", attrs: `data-id="${e(engine.id)}"` }) : ""}</div>${progress ? `<progress class="model-progress" data-model-progress="${e(engine.id)}" value="${progress.received}" max="${progress.total || 1}" aria-label="${Vara.i18n.bind("ui.model-download")}"></progress>` : ""}`;
    }
    if (["claude_cli", "codex_cli", "gemini_cli"].includes(engine.provider))
      return `<div class="model-readiness"><span>${e(location)} · ${e(s.cli[engine.provider]?.detail ? Vara.i18n.bindRuntime(s.cli[engine.provider].detail) : Vara.i18n.bind("ui.cli-login-required"))}</span>${b(Vara.i18n.bind("ui.check-cli"), "check-cli", { variant: "quiet", attrs: `data-id="${e(engine.provider)}"` })}</div>`;
    const optionalKey = ["custom", "ollama"].includes(engine.provider);
    const id = optionalKey ? engine.id : engine.provider;
    const key = s.keyStates[id], test = s.connectionChecks[id];
    const name = ({open_ai:"OpenAI",groq:"Groq",eleven_labs:"ElevenLabs",anthropic:"Anthropic",gemini:"Gemini"})[id] || engine.label;
    const text = key?.error ? Vara.i18n.bind("ui.key-status-is-unavailable") : optionalKey ? Vara.i18n.bind("ui.custom-endpoint-key-is-optional") : key?.present ? tr(test?.phase === "success" ? "key.saved.tested" : "key.saved.untested") : key?.present === false ? Vara.i18n.bind("ui.key-missing") : Vara.i18n.bind("ui.checking-key-store");
    return `<div class="model-readiness" role="status"><span>${icon(engine.locality === "cloud" ? "globe" : "info")}${e(location)} · ${e(text)}</span>${key?.present && ["open_ai", "groq"].includes(id) ? b(test?.phase === "testing" ? Vara.i18n.bind("ui.testing") : Vara.i18n.bind("ui.test-key"), id === "open_ai" ? "check-openai" : "check-groq", { variant: "quiet", attrs: `data-id="${e(id)}" ${test?.phase === "testing" ? "disabled" : ""}` }) : b(!optionalKey && key?.present === false ? tr("key.add.provider", { provider: name }) : Vara.i18n.bind("ui.manage-key"), "open-key", { variant: "quiet", attrs: `data-id="${e(id)}" data-name="${e(name)}"` })}</div>${key?.error || test?.phase === "error" ? notice(key?.error || test.message, "error") : ""}`;
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
    return `<div class="processing-summary">${Vara.desktopUi.policyRow("mic", tr("privacy.audio", { location: speech?.locality === "cloud" ? "Cloud" : loc[speech?.locality] || tr("ui.unknown-location") }), "")}${Vara.desktopUi.policyRow("globe", settings.general_ai_enabled ? tr("privacy.text", { model: text ? Vara.desktopUi.modelLabel(text) : tr("ui.unknown-model"), location: text?.locality === "cloud" ? "Cloud" : loc[text?.locality] || tr("ui.unknown-location") }) : Vara.i18n.bind("ui.text-direct-transcription-by-default"), appRules.length ? Vara.i18n.bind("ui.apps-with-custom-rules-use-their-selected-ai") : "")}</div><p class="muted">${Vara.i18n.bind("ui.cloud-is-used-only-for-the-features-you-choose")}</p>${conflicts ? notice(Vara.i18n.bind("ui.your-selected-models-or-app-rules-process-data-outside-this-devic"), "warning") : ""}`;
  }
  function settings(s) {
    const p = s.data.settings, { settingsSection: group, processingChoice, policyRow } = Vara.desktopUi;
    let body = "";
    if (s.tab === "dictation") {
      body = group(Vara.i18n.bind("ui.recording"), Vara.i18n.bind("ui.how-to-record-and-what-to-use"), `<div class="settings-field-grid">${f({ id:"activation",label:Vara.i18n.bind("ui.activation"),value:p.activation,options:[["hold",Vara.i18n.bind("ui.hold-to-talk")],["toggle",Vara.i18n.bind("ui.press-to-start-stop-2")]] })}${shortcut(p.shortcut)}${f({ id:"input-device",label:Vara.i18n.bind("ui.microphone"),value:p.input_device || "",help:s.deviceError || "",options:[["",Vara.i18n.bind("ui.system-default")],...s.devices.map(d=>[d.id,d.label])] })}${f({ id:"language",label:Vara.i18n.bind("ui.language"),value:p.speech_engine === "local.parakeet" ? "" : p.language || "",attrs:p.speech_engine === "local.parakeet" ? "disabled" : "",help:p.speech_engine === "local.parakeet" ? Vara.i18n.bind("ui.parakeet-detects-the-language-automatically") : Vara.i18n.bind("ui.language-for-transcription"),options:[["",Vara.i18n.bind("ui.detect-automatically")],["da",Vara.i18n.bind("ui.danish")],["en",Vara.i18n.bind("ui.english")],["de",Vara.i18n.bind("ui.german")],["sv",Vara.i18n.bind("ui.swedish")],["no",Vara.i18n.bind("ui.norwegian")]] })}</div>`)
        + group(Vara.i18n.bind("ui.speech-to-text"), Vara.i18n.bind("ui.choose-the-model-that-transcribes-your-speech"), modelPicker(s,"speech-engine",Vara.i18n.bind("ui.transcription-model"),"speech",p.speech_engine))
        + group(Vara.i18n.bind("ui.after-transcription"), Vara.i18n.bind("ui.your-default-for-new-dictations"), processingChoice(p.general_ai_enabled) + `<div id="text-processing-fields" ${p.general_ai_enabled ? "" : "hidden"}>${modelPicker(s,"general-engine",Vara.i18n.bind("ui.text-model"),"text",p.general_engine)}${f({ id:"general-prompt",label:Vara.i18n.bind("ui.instructions-for-ai"),value:p.general_prompt,rows:true })}</div><p class="app-rules-note">${tr("ui.apps-with-custom-rules-use-their-own-settings")} ${b(Vara.i18n.bind("ui.manage-apps"),"navigate",{variant:"quiet",attrs:'data-page="apps"'})}</p>`)
        + group(Vara.i18n.bind("ui.permissions"), Vara.i18n.bind("ui.access-to-microphone-shortcut-and-insertion-in-other-apps"), permissions(s));
    }
    if (s.tab === "connections") {
      body = group(Vara.i18n.bind("ui.local-models"), Vara.i18n.bind("ui.download-once-use-without-an-account-or-key"), `<div class="model-list">${models(s)}</div><p class="form-footnote">${tr("model.license")}</p>`, true)
        + group(Vara.i18n.bind("ui.api-keys"), Vara.i18n.bind("ui.your-own-keys-are-saved-in-the-system-key-store"), `<div class="provider-list" data-connections>${providers(s)}</div>`, true)
        + disclosure("advanced-engines",Vara.i18n.bind("ui.custom-endpoints-and-model-names"),Vara.i18n.bind("ui.edit-model-names-and-compatible-text-connections"), `<div class="provider-list">${p.engines.filter(e=>!e.id.startsWith("local.") && !e.id.endsWith(".cli")).map(x=>`<div class="provider-row"><div><strong>${e(Vara.desktopUi.displayEngineName(x))}</strong><small>${e(x.model)} · ${e(loc[x.locality])}</small></div>${b(Vara.i18n.bind("ui.edit"),"edit-engine",{attrs:`data-id="${e(x.id)}"`})}</div>`).join("")}</div>${b(Vara.i18n.bind("ui.add-endpoint"),"add-endpoint")}`, s.disclosures?.["advanced-engines"])
        + disclosure("cli-engines",Vara.i18n.bind("ui.installed-cli-tools"),Vara.i18n.bind("ui.optional-text-connections-with-their-own-login"), `<div class="provider-list">${[["claude_cli","Claude Code"],["codex_cli","Codex"],["gemini_cli","Gemini CLI"]].map(([id,name])=>`<div class="provider-row"><div><strong>${name}</strong><small>${e(s.cli?.[id]?.detail ? Vara.i18n.bindRuntime(s.cli[id].detail) : Vara.i18n.bind("ui.not-checked-text-may-be-processed-by-an-external-provider"))}</small></div>${b(Vara.i18n.bind("ui.check"),"check-cli",{attrs:`data-id="${id}"`})}</div>`).join("")}</div>`, s.disclosures?.["cli-engines"])
        + `<p class="model-selection-note">${tr("connection.selection.note")} ${b(Vara.i18n.bind("ui.dictation-2"),"settings-tab",{variant:"quiet",attrs:'data-id="dictation"'})}.</p>`;
    }
    if (s.tab === "privacy") {
      body = group(Vara.i18n.bind("ui.processing"), Vara.i18n.bind("ui.you-decide-where-your-data-is-processed"), toggle("strict-local",Vara.i18n.bind("ui.on-device-processing-only"),p.strict_local,Vara.i18n.bind("ui.when-enabled-vara-can-only-use-local-models")) + `<div id="privacy-processing-summary">${privacySummary(s)}</div>`)
        + group(Vara.i18n.bind("ui.history"), Vara.i18n.bind("ui.your-dictations-are-saved-locally-grouped-by-app"), f({id:"retention",label:Vara.i18n.bind("ui.keep-text"),value:p.retention_days ? String(p.retention_days) : "",options:[["",Vara.i18n.bind("ui.until-i-delete-it")],["7",Vara.i18n.bind("ui.7-days")],["30",Vara.i18n.bind("ui.30-days")],["90",Vara.i18n.bind("ui.90-days")],["365",Vara.i18n.bind("ui.1-year")],...(p.retention_days && ![7,30,90,365].includes(p.retention_days) ? [[String(p.retention_days),tr("history.days", { count: p.retention_days })]] : [])]}) + `<p class="muted">${Vara.i18n.bind("ui.you-can-choose-to-delete-automatically-after-a-period")}</p><p class="form-footnote">${Vara.i18n.bind("ui.deleting-a-dictation-deletes-both-the-original-and-processed-text")}</p>`)
        + group(Vara.i18n.bind("ui.audio"), Vara.i18n.bind("ui.no-audio-in-history"), policyRow("trash",Vara.i18n.bind("ui.after-successful-transcription"),Vara.i18n.bind("ui.the-audio-is-deleted-automatically")) + policyRow("history",Vara.i18n.bind("ui.if-transcription-fails"),Vara.i18n.bind("ui.audio-is-stored-temporarily-for-up-to-24-hours-so-you-can-retry")) + `<p class="form-footnote">${Vara.i18n.bind("ui.a-successful-retry-or-deletion-removes-the-audio-sooner")}</p>`);
    }
    const t = Vara.i18n.message;
    const updateSettings = s.update ? group(Vara.i18n.bind("update.heading"), Vara.i18n.bind("update.description"),
      updateNotice(s) + `<label><input id="update-automatic" type="checkbox" ${s.update.automatic ? "checked" : ""}> ${tr("update.automatic")}</label>` +
      b(Vara.i18n.bind("update.check"), "update-check", {attrs: ["disabled","checking","downloading","installing"].includes(s.update.status) ? "disabled" : ""})) : "";
    const languagePicker = group(t("locale.heading"), t("locale.description"), f({id:"ui-locale", label:t("locale.label"), value:p.ui_locale || "da", options:[["system",t("locale.system")],...Object.entries(Vara.i18n.localeNames)]}));
    return `<div class="page-scroll settings-page"><header class="page-header"><h1 id="view-title" tabindex="-1">${Vara.i18n.bind("ui.settings")}</h1><p class="muted">${Vara.i18n.bind("ui.a-quiet-helper-on-your-terms")}</p></header><nav class="settings-nav" aria-label="${Vara.i18n.bind("ui.settings")}">${[["dictation",Vara.i18n.bind("ui.dictation")],["connections",Vara.i18n.bind("ui.models-and-keys")],["privacy",Vara.i18n.bind("ui.privacy")]].map(([id,name])=>`<button class="${s.tab===id ? "is-current" : ""}" data-action="settings-tab" data-id="${id}" ${s.tab===id ? 'aria-current="page"' : ""}>${name}</button>`).join("")}</nav><div class="activity-region">${messages(s)}</div><div id="locale-picker">${languagePicker}${updateSettings}</div><form class="settings-form" id="settings-form">${body}<div class="form-message" role="status"></div><div class="actions sticky-save"><span class="unsaved" ${s.dirty ? "" : "hidden"}>${Vara.i18n.bind("ui.unsaved")}</span>${b(Vara.i18n.bind("ui.discard-changes"),"reset-form",{variant:"quiet"})}${b(Vara.i18n.bind("ui.save-settings"),"save-settings",{variant:"primary",attrs:s.dirty ? "" : "disabled"})}</div></form></div>`;
  }
  function setup(s) {
    const steps = [
      ["prepare", Vara.i18n.bind("ui.prepare-vara")],
      ["permissions", Vara.i18n.bind("ui.grant-access")],
      ["trial", Vara.i18n.bind("ui.try-your-shortcut")],
    ];
    const current = s.setupStep || "prepare",
      index = steps.findIndex(([id]) => id === current);
    const speechId = s.data.settings.speech_engine;
    const localSpeech = ["local.parakeet", "local.whisper"].includes(speechId);
    const ready = !localSpeech || s.data.models[speechId];
    let content = "";
    if (current === "prepare")
      content = `<h1 id="view-title" tabindex="-1">${Vara.i18n.bind("ui.your-words-where-you-work")}</h1><p class="setup-description">${Vara.i18n.bind("ui.hold-a-key-speak-and-release-we-start-with-local-transcription-so")}</p><section class="setup-preparation">${models(s, localSpeech ? speechId : "local.parakeet")}</section><p class="muted">${Vara.i18n.bind("ui.the-model-is-downloaded-once-no-account-or-extra-installation")}</p><div class="setup-actions">${b(Vara.i18n.bind("ui.continue"), "setup-next", { variant: "primary", attrs: ready ? "" : "disabled" })}${b(Vara.i18n.bind("ui.i-prefer-cloud"), "open-dictation", { variant: "quiet" })}</div>`;
    if (current === "permissions")
      content = `<h1 id="view-title" tabindex="-1">${Vara.i18n.bind("ui.give-vara-access")}</h1><p class="setup-description">${Vara.i18n.bind("ui.the-microphone-is-needed-for-dictation-you-can-always-copy-text-m")}</p>${permissions(s)}<div class="setup-actions">${b(Vara.i18n.bind("ui.back"), "setup-back", { variant: "quiet" })}${b(Vara.i18n.bind("ui.continue"), "setup-next", { variant: "primary", attrs: ["authorized", "system"].includes(s.data.permissions.microphone) ? "" : "disabled" })}</div>`;
    if (current === "trial")
      content = `<h1 id="view-title" tabindex="-1">${Vara.i18n.bind("ui.say-your-first-sentence")}</h1><p class="setup-description">${tr(s.data.settings.activation === "hold" ? "setup.trial.hold" : "setup.trial.toggle")}</p><div class="trial-shortcut"><kbd>${e(binding(s.data.settings.shortcut))}</kbd><span>${s.data.settings.activation === "hold" ? Vara.i18n.bind("ui.hold-speak-release") : Vara.i18n.bind("ui.press-speak-press-again")}</span></div><p class="muted">${Vara.i18n.bind("ui.you-can-also-start-here-the-text-will-be-ready-to-insert-manually")}</p><div class="setup-actions">${b(Vara.i18n.bind("ui.back"), "setup-back", { variant: "quiet" })}${b(s.data.session?.stage === "recording" ? Vara.i18n.bind("ui.stop-dictation") : Vara.i18n.bind("ui.try-a-dictation"), s.data.session?.stage === "recording" ? "stop" : "start", { variant: "primary", glyph: s.data.session?.stage === "recording" ? "stop" : "mic", attrs: s.data.session && s.data.session.stage !== "recording" ? "disabled" : "" })}</div>`;
    return `<div class="page-scroll setup-page"><div class="initial-setup"><ol class="setup-steps" aria-label="${Vara.i18n.bind("ui.setup")}">${steps.map(([id, label], i) => `<li ${id === current ? 'aria-current="step"' : ""}><span class="step-number">${i < index ? icon("check") : i + 1}</span><span>${label}</span></li>`).join("")}</ol><div class="activity-region">${messages(s)}</div><div class="setup-content">${content}</div><div class="setup-later">${b(Vara.i18n.bind("ui.continue-later"), "navigate", { variant: "quiet", attrs: 'data-page="history"' })}</div></div></div>`;
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
