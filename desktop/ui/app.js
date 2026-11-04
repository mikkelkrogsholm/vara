/* Native IPC controller. This file never handles microphone bytes or receives stored secrets. */
(async () => {
  "use strict";
  const { esc: e, button: b, field: f, notice } = Vara.ui;
  document.documentElement.dataset.platform = /Mac/.test(navigator.platform)
    ? "macos"
    : "other";
  const $ = (id) => document.getElementById(id);
  const state = {
    data: null,
    page: "history",
    tab: "dictation",
    selected: null,
    prefApp: null,
    query: "",
    filter: "all",
    searchOpen: false,
    collapsed: new Set(),
    detailOpen: false,
    originalOpen: false,
    dirty: false,
    devices: [],
    deviceError: null,
    cli: {},
    keyStates: {},
    connectionChecks: {},
    setupStep: "prepare",
    uiError: null,
    shortcutDraft: null,
    shortcutKeys: new Set(),
    shortcutImplied: new Set(),
    shortcutSources: { dom: new Set(), native: new Set() },
    nativeCapture: false,
    disclosures: {},
  };
  const api = async (action, payload = {}) => {
    if (!window.__TAURI__)
      throw uiError(
        Vara.i18n.bind("ui.open-vara-as-a-desktop-app-to-use-the-microphone-models-and-histo"),
      );
    return window.__TAURI__.core.invoke("command", { action, payload });
  };
  const uiError = token => {
    const error = Error(Vara.i18n.resolve(token));
    error.uiMessage = token;
    return error;
  };
  const report = (error) => {
    const message = error?.uiMessage || Vara.i18n.bindRuntime(String(error?.message || error));
    const target = document.querySelector("form .form-message");
    if (target) {
      target.innerHTML = notice(message, "error");
      target.scrollIntoView({ block: "nearest" });
    } else {
      state.uiError = {
        message,
        scope: state.page === "history" && state.selected ? "detail" : "global",
      };
      render();
    }
    $("live").textContent = message;
  };
  let toastTimer;
  function toast(message) {
    $("toast").textContent = message;
    $("toast").hidden = false;
    $("live").textContent = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => ($("toast").hidden = true), 5000);
  }
  function focusIdentity(node = document.activeElement) {
    if (!node || node === document.body) return null;
    if (node.id) return { selector: "#" + CSS.escape(node.id), index: 0 };
    if (node.matches(".original-disclosure summary"))
      return { selector: ".original-disclosure summary", index: 0 };
    if (node.dataset.action) {
      let selector = '[data-action="' + CSS.escape(node.dataset.action) + '"]';
      for (const key of ["id", "page", "kind"])
        if (node.dataset[key])
          selector +=
            "[data-" + key + '="' + CSS.escape(node.dataset[key]) + '"]';
      return {
        selector,
        index: [...document.querySelectorAll(selector)].indexOf(node),
      };
    }
    return null;
  }
  function restoreFocus(identity) {
    const node =
      identity && document.querySelectorAll(identity.selector)[identity.index];
    node?.focus({ preventScroll: true });
    return node;
  }
  const scrollPositions = new Map();
  let renderedPage = null;
  function render() {
    if (!state.data) return;
    if(state.page==="transcription"&&Vara.transcription?.retainEditor()){
      $("sidebar").innerHTML=Vara.views.sidebar(state);Vara.transcription.refreshEditorStatus();return;
    }
    const focus = focusIdentity(),
      focused = document.activeElement;
    const selection =
      focused && "selectionStart" in focused
        ? [focused.selectionStart, focused.selectionEnd]
        : null;
    const viewKey =
      state.page +
      (state.page === "settings"
        ? ":" + state.tab
        : state.page === "apps"
          ? ":" + state.prefApp
          : "");
    if (renderedPage)
      scrollPositions.set(renderedPage, [
        $("workspace").scrollTop,
        document.querySelector(".history-scroll")?.scrollTop || 0,
        document.querySelector(".detail-scroll")?.scrollTop || 0,
      ]);
    $("workspace").className =
      "workspace " +
      (state.page === "history" ? "is-history" : "is-form") +
      (state.detailOpen ? " has-detail" : "");
    $("sidebar").innerHTML = Vara.views.sidebar(state);
    $("workspace").innerHTML = Vara.views[state.page](state);
    if (pendingLocale !== null && $("ui-locale")) $("ui-locale").disabled = true;
    const node = restoreFocus(focus);
    if (selection && node?.setSelectionRange) {
      try {
        node.setSelectionRange(...selection);
      } catch {}
    }
    const [workspace, history, detail] = scrollPositions.get(viewKey) || [
      0, 0, 0,
    ];
    $("workspace").scrollTop = workspace;
    const h = document.querySelector(".history-scroll"),
      d = document.querySelector(".detail-scroll");
    if (h) h.scrollTop = history;
    if (d) d.scrollTop = detail;
    renderedPage = viewKey;
    refreshModelControls();
    if (state.page === "settings" && state.tab === "connections") {
      for (const [id] of Vara.views.providersList) if (!state.keyStates[id]) loadKeyStatus(id);
    }
  }
  const requestedIcons = new Set();
  function loadAppArtwork() {
    const ids = Vara.views.appsList(state).map(([id]) => id).filter(id => !requestedIcons.has(id));
    for (let offset = 0; offset < ids.length; offset += 128) {
      const batch = ids.slice(offset, offset + 128);
      batch.forEach(id => requestedIcons.add(id));
      api("application_icons", { ids: batch }).then(Vara.desktopUi.setAppArtwork).catch(() => {
        batch.forEach(id => requestedIcons.delete(id));
      });
    }
  }
  // Localize in place: subscribers must not replace live controls, dialogs, or editors.
  function updateLocalePicker() {
    const picker = $("locale-picker");
    if (!picker) return;
    const t = Vara.i18n.message;
    picker.querySelector("h2").textContent = t("locale.heading");
    picker.querySelector("header p").textContent = t("locale.description");
    picker.querySelector('label[for="ui-locale"]').textContent = t("locale.label");
    for (const option of picker.querySelectorAll("option")) option.textContent = option.value === "system" ? t("locale.system") : Vara.i18n.localeNames[option.value];
  }
  Vara.i18n.subscribe(updateLocalePicker);
  let localeWrites = Promise.resolve(), pendingLocale = null;
  function apply(data) {
    if (state.data && (data.ui_locale_revision ?? 0) < (state.data.ui_locale_revision ?? 0)) {
      data.settings.ui_locale = state.data.settings.ui_locale;
      data.resolved_ui_locale = state.data.resolved_ui_locale;
      data.ui_locale_revision = state.data.ui_locale_revision;
    }
    if (pendingLocale !== null) data.settings.ui_locale = pendingLocale;
    else Vara.i18n.setLocale(data.resolved_ui_locale || "da");
    const previous = state.data?.history[0]?.id;
    if (state.data && JSON.stringify(state.data) === JSON.stringify(data))
      return;
    state.data = data;
    Vara.transcription?.onSession(data.session);
    loadAppArtwork();
    if (!state.selected && data.history.length)
      state.selected = data.history[0].id;
    if (data.history[0]?.id && previous !== data.history[0].id) {
      state.selected = data.history[0].id;
      if (!state.dirty && state.page !== "transcription") {
        state.page = "history";
        state.detailOpen = true;
      } else toast(Vara.i18n.bind("ui.a-new-dictation-has-been-saved-in-history"));
    }
    if (state.selected && !data.history.some((r) => r.id === state.selected))
      state.selected = data.history[0]?.id || null;
    if (!state.prefApp)
      state.prefApp = Vara.views.appsList(state)[0]?.[0] || null;
    if (!state.dirty) render();
    else {
      const identity = focusIdentity();
      const focusedInSidebar = $("sidebar").contains(document.activeElement);
      $("sidebar").innerHTML = Vara.views.sidebar(state);
      if (focusedInSidebar) restoreFocus(identity);
      const activity = document.querySelector(".activity-region");
      if (activity) activity.innerHTML = Vara.views.messages(state);
      document.querySelectorAll(".model-list").forEach((node) => {
        node.innerHTML = Vara.views.models(state);
      });
      data.downloads.forEach((d) => {
        const progress = document.querySelector(
          `[data-model-progress="${CSS.escape(d.id)}"]`,
        );
        if (progress) {
          progress.value = d.received;
          progress.max = d.total || 1;
        }
      });
      refreshModelControls();
    }
  }
  const refresh = async (force = false) => {
    const data = await api("snapshot");
    if (force) {
      state.data = data;
      loadAppArtwork();
      render();
    } else apply(data);
  };
  function canLeave() {
    if (!state.dirty) return true;
    toast(Vara.i18n.bind("ui.save-your-changes-or-choose-discard-changes-first"));
    return false;
  }
  function value(id) {
    return $(id)?.value;
  }
  function checked(id) {
    return $(id)?.checked;
  }
  let modalInvoker = null;
  function modal(title, content) {
    if (!$("modal").open) modalInvoker = focusIdentity();
    $("modal").innerHTML =
      `<div class="dialog-content">${Vara.ui.iconButton("close", Vara.i18n.bind("ui.close-dialog"), "close-modal")}<h2 id="dialog-title">${e(title)}</h2>${content}<div class="form-message" role="alert" id="dialog-error"></div></div>`;
    if (!$("modal").open) $("modal").showModal();
    const input = $("modal").querySelector(
      "input:not([type=hidden]),select,textarea",
    );
    input?.focus();
  }
  function closeModal() {
    if ($("shortcut-recorder")) {
      state.shortcutKeys.clear();
      Object.values(state.shortcutSources).forEach(keys => keys.clear());
      state.shortcutImplied.clear();
      api("shortcut_capture", { enabled: false }).catch(report);
    }
    $("modal").close();
    $("modal").innerHTML = "";
    restoreFocus(modalInvoker);
    modalInvoker = null;
  }
  function invalid(id, message) {
    const input = $(id);
    input?.setAttribute("aria-invalid", "true");
    const error = $(id + "-error");
    if (error) error.textContent = message;
    input?.focus();
    throw uiError(message);
  }
  function requireValue(id, message) {
    if (!value(id)?.trim()) invalid(id, message);
  }
  function navigate(run) {
    $("toast").hidden = true;
    clearTimeout(toastTimer);
    if (!state.dirty) {
      run();
      render();
      return;
    }
    state.pendingRoute = run;
    modal(
      Vara.i18n.bind("ui.changes-are-not-saved"),
      `<p>${Vara.i18n.bind("ui.save-your-changes-first-or-discard-them-to-continue")}</p><div class="actions">${b(Vara.i18n.bind("ui.stay-here"), "close-modal", { variant: "primary" })}${b(Vara.i18n.bind("ui.discard-changes-2"), "discard-route", { variant: "quiet" })}</div>`,
    );
  }
  function settingsDraft() {
    const p = structuredClone(state.data.settings);
    if ($("activation")) {
      p.activation = value("activation");
      p.shortcut = value("shortcut").trim();
      p.input_device = value("input-device") || null;
      p.speech_engine = value("speech-engine");
      p.language = p.speech_engine === "local.parakeet" ? null : value("language") || null;
      p.general_ai_enabled = checked("general-ai");
      p.general_prompt = value("general-prompt");
      p.general_engine = value("general-engine");
    }
    if ($("strict-local")) p.strict_local = checked("strict-local");
    if ($("retention")) p.retention_days = value("retention") ? Number(value("retention")) : null;
    return p;
  }
  function refreshModelControls() {
    if (!state.data) return;
    if ($("text-processing-fields")) $("text-processing-fields").hidden = !checked("general-ai");
    if ($("app-ai-fields")) $("app-ai-fields").hidden = !checked("app-ai");
    const p = settingsDraft();
    document.querySelectorAll(".model-feedback[data-model-field]").forEach(node => {
      if (node.closest("[hidden]")) return;
      const engine = p.engines.find(e => e.id === value(node.dataset.modelField));
      node.innerHTML = Vara.views.modelFeedback(state, engine, p);
      if (!engine || engine.locality === "on_device" || ["claude_cli", "codex_cli", "gemini_cli"].includes(engine.provider)) return;
      const id = ["custom", "ollama"].includes(engine.provider) ? engine.id : engine.provider;
      if (!state.keyStates[id]) loadKeyStatus(id);
    });
    if ($("privacy-processing-summary")) $("privacy-processing-summary").innerHTML = Vara.views.privacySummary(state, p);
  }
  async function saveSettings() {
    const p = settingsDraft();
    if (state.tab === "dictation" && p.general_ai_enabled)
      requireValue("general-prompt", Vara.i18n.bind("ui.enter-instructions-before-enabling-ai"));
    await api("save_settings", p);
    state.dirty = false;
    await refresh(true);
    toast(Vara.i18n.bind("ui.settings-saved"));
  }
  function renderConnections() {
    const list = document.querySelector("[data-connections]");
    if (list) {
      const focus = focusIdentity();
      list.innerHTML = Vara.views.providers(state);
      restoreFocus(focus);
    }
    refreshModelControls();
  }
  async function loadKeyStatus(id) {
    state.keyStates[id] = { ...state.keyStates[id], loading: true };
    try {
      const present = await api("key_status", { id });
      if (state.keyStates[id]?.present !== present) delete state.connectionChecks[id];
      state.keyStates[id] = { present };
    } catch (error) {
      state.keyStates[id] = { error: Vara.i18n.bindRuntime(String(error?.message || error)) };
    }
    renderConnections();
  }
  async function keyDialog(id, name) {
    let exists;
    try { exists = await api("key_status", { id }); state.keyStates[id] = { present: exists }; renderConnections(); }
    catch (error) {
      modal(name, `<p role="alert">${e(Vara.i18n.bindRuntime(String(error?.message || error)))}</p><div class="actions">${b(Vara.i18n.bind("ui.try-again"), "open-key", { variant: "primary", attrs: `data-id="${e(id)}" data-name="${e(name)}"` })}</div>`);
      return;
    }
    modal(
      name,
      `<form id="key-form" data-key-id="${e(id)}"><p>${exists ? Vara.i18n.bind("ui.a-key-is-saved-in-the-system-key-store-you-can-replace-or-delete-") : Vara.i18n.bind("ui.add-your-own-api-key-it-is-saved-in-the-system-key-store")}</p>${["groq", "open_ai"].includes(id) ? ("<p>"+Vara.i18n.bind("ui.the-provider-can-transcribe-your-audio-and-process-text-with-ai-d")+"</p>") : ""}${f({ id: "api-key", label: Vara.i18n.bind("ui.api-key"), type: "password", attrs: 'autocomplete="off" spellcheck="false" autocapitalize="off"' })}<div class="actions">${b(Vara.i18n.bind("ui.save-key"), "save-key", { variant: "primary", attrs: `data-id="${e(id)}"` })}${exists ? b(Vara.i18n.bind("ui.delete-key"), "delete-key", { variant: "quiet", attrs: `data-id="${e(id)}"` }) : ""}${["groq", "open_ai"].includes(id) && exists ? b(Vara.i18n.bind("ui.test-key"), id === "groq" ? "check-groq" : "check-openai", { variant: "quiet" }) : ""}${b(Vara.i18n.bind("ui.try-again"), "open-key", { variant: "quiet", attrs: `id="retry-key-store" hidden data-id="${e(id)}" data-name="${e(name)}"` })}</div>${["groq", "open_ai"].includes(id) ? `<p id="${id === "groq" ? "groq" : "openai"}-access-status" role="status" class="muted"></p>` : ""}</form>`,
    );
  }
  function engineDialog(id) {
    if (!canLeave()) return;
    const engine = state.data.settings.engines.find((x) => x.id === id);
    const custom = !engine || ["custom", "ollama"].includes(engine.provider);
    modal(
      engine ? Vara.i18n.bind("ui.edit-connection") : Vara.i18n.bind("ui.custom-endpoint"),
      `<form id="engine-form" data-engine-id="${e(id || "")}">${f({ id: "engine-label", label: Vara.i18n.bind("ui.name"), value: engine?.label || "" })}${custom ? f({ id: "engine-endpoint", label: Vara.i18n.bind("ui.endpoint-url"), value: engine?.endpoint || "", help: Vara.i18n.bind("ui.openai-compatible-text-api-example-http-127-0-0-1-11434-v1") }) : ""}${f({ id: "engine-model", label: Vara.i18n.bind("ui.model"), value: engine?.model || "" })}${
        custom
          ? f({
              id: "engine-locality",
              label: Vara.i18n.bind("ui.where-is-text-processed"),
              value: engine?.locality || "on_device",
              options: [
                ["on_device", Vara.i18n.bind("ui.on-this-device-loopback-only")],
                ["lan", Vara.i18n.bind("ui.on-your-network-https")],
                ["cloud", Vara.i18n.bind("ui.at-an-external-provider-https")],
              ],
            })
          : ""
      }<div class="actions">${b(Vara.i18n.bind("ui.save-connection"), "save-engine", { variant: "primary", attrs: `data-id="${e(id || "")}"` })}${custom && engine ? b(Vara.i18n.bind("ui.manage-key"), "open-key", { attrs: `data-id="${e(engine.id)}" data-name="${e(engine.label)}"` }) : ""}</div></form>`,
    );
  }
  async function addApp() {
    if (!canLeave()) return;
    const apps = await api("applications");
    modal(
      Vara.i18n.bind("ui.add-app"),
      apps.length
        ? `${f({ id: "new-app", label: "App", options: apps.sort((a, b) => a.label.localeCompare(b.label)).map((a) => [a.id, a.label]) })}${b(Vara.i18n.bind("ui.add-app"), "save-new-app", { variant: "primary" })}`
        : notice(
            Vara.i18n.bind("ui.start-dictating-in-an-app-it-becomes-available-here-after-the-fir"),
          ),
    );
    state.availableApps = apps;
  }
  async function action(button) {
    const a = button.dataset.action,
      id = button.dataset.id;
    if (a.startsWith("tx-")) return Vara.transcription.action(button);
    switch (a) {
      case "navigate":
        navigate(() => {
          state.page = button.dataset.page;
          state.detailOpen = false;
        });
        break;
      case "choose-processing":
        modal(
          Vara.i18n.bind("ui.next-dictation"),
          f({
            id: "next-choice",
            label: Vara.i18n.bind("ui.processing"),
            value: state.data.next_choice || "default",
            options: [
              ["default", Vara.i18n.bind("ui.follow-your-settings")],
              ["direct", Vara.i18n.bind("ui.direct-transcription")],
              ["general_ai", Vara.i18n.bind("ui.use-general-system-prompt")],
            ],
          }) +
            ("<p>"+Vara.i18n.bind("ui.applies-to-one-dictation-including-when-you-use-the-shortcut-in-a")+"</p>") +
            b(Vara.i18n.bind("ui.select"), "arm-processing", { variant: "primary" }),
        );
        break;
      case "arm-processing":
        await api("set_next_choice", { choice: value("next-choice") });
        closeModal();
        await refresh();
        break;
      case "check-cli":
        state.cli[id] = await api("check_cli", { provider: id });
        if (state.dirty) refreshModelControls(); else render();
        break;
      case "record-shortcut":
        state.nativeCapture = !!(await api("shortcut_capture", { enabled: true }))?.native;
        state.shortcutDraft = null;
        state.shortcutKeys.clear();
        Object.values(state.shortcutSources).forEach(keys => keys.clear());
      state.shortcutImplied.clear();
        modal(
          Vara.i18n.bind("ui.change-shortcut"),
          `<p>${Vara.i18n.bind("ui.press-and-release-a-key-or-combination-left-and-right-modifier-ke")}</p><div class="shortcut-recorder" id="shortcut-recorder" tabindex="0" role="group" aria-label="${Vara.i18n.bind("ui.record-shortcut")}" aria-live="polite">${Vara.i18n.bind("ui.press-your-shortcut")}</div><p class="muted" id="shortcut-warning">${Vara.i18n.bind("ui.regular-typing-keys-and-system-shortcuts-can-also-affect-the-app-")}</p><div class="actions">${b(Vara.i18n.bind("ui.use-shortcut"), "use-shortcut", { variant: "primary", attrs: "disabled" })}${b(Vara.i18n.bind("ui.cancel"), "close-modal")}</div>`,
        );
        $("shortcut-recorder").focus();
        break;
      case "use-shortcut":
        if (!state.shortcutDraft)
          throw uiError(
            Vara.i18n.bind("ui.press-and-release-the-desired-key-or-combination"),
          );
        $("shortcut").value = state.shortcutDraft;
        $("shortcut-preview").textContent = Vara.desktopUi.binding(
          state.shortcutDraft,
        );
        $("shortcut").dispatchEvent(new Event("input", { bubbles: true }));
        closeModal();
        break;
      case "discard-route":
        state.dirty = false;
        closeModal();
        state.pendingRoute?.();
        state.pendingRoute = null;
        render();
        break;
      case "dismiss-ui-error":
        state.uiError = null;
        render();
        break;
      case "setup-next":
        state.setupStep =
          state.setupStep === "prepare" ? "permissions" : "trial";
        render();
        $("view-title")?.focus();
        break;
      case "setup-back":
        state.setupStep =
          state.setupStep === "trial" ? "permissions" : "prepare";
        render();
        $("view-title")?.focus();
        break;
      case "setup":
        if (canLeave()) {
          state.page = "setup";
          render();
        }
        break;
      case "open-dictation":
        if (canLeave()) {
          state.page = "settings";
          state.tab = "dictation";
          render();
          $("speech-engine")?.focus();
        }
        break;
      case "settings-tab":
        navigate(() => {
          state.tab = id;
        });
        break;
      case "toggle-search":
        state.searchOpen = !state.searchOpen;
        render();
        $("history-search")?.focus();
        break;
      case "clear-search":
        state.query = "";
        state.filter = "all";
        render();
        break;
      case "toggle-group":
        state.collapsed.has(id)
          ? state.collapsed.delete(id)
          : state.collapsed.add(id);
        render();
        break;
      case "select-result":
        state.uiError = null;
        state.selected = id;
        state.detailOpen = true;
        state.originalOpen = false;
        render();
        break;
      case "back-history":
        state.detailOpen = false;
        render();
        break;
      case "copy":
      case "copy-original":
        await api("copy", { id, original: a === "copy-original" });
        state.uiError = null;
        render();
        toast(a === "copy" ? Vara.i18n.bind("ui.copied") : Vara.i18n.bind("ui.original-copied"));
        break;
      case "copy-unsaved":
        await api("copy", { unsaved: true });
        toast(Vara.i18n.bind("ui.text-copied"));
        break;
      case "confirm-delete":
        modal(
          Vara.i18n.bind("ui.delete-dictation-2"),
          `<p>${Vara.i18n.bind("ui.both-the-original-and-finished-text-will-be-deleted-from-history")}</p><div class="actions">${b(Vara.i18n.bind("ui.delete-dictation"), "delete-result", { variant: "destructive", attrs: `data-id="${e(id)}"` })}${b(Vara.i18n.bind("ui.keep"), "close-modal")}</div>`,
        );
        break;
      case "delete-result":
        await api("delete_result", { id });
        closeModal();
        await refresh();
        break;
      case "delete-recovery":
        await api("delete_recovery", { id });
        await refresh();
        break;
      case "select-pref":
        navigate(() => {
          state.prefApp = id;
        });
        break;
      case "save-app": {
        if (checked("app-ai"))
          requireValue(
            "app-prompt",
            Vara.i18n.bind("ui.enter-a-system-prompt-before-enabling-ai"),
          );
        const p = structuredClone(state.data.settings);
        const name = Vara.views
          .appsList(state)
          .find((a) => a[0] === state.prefApp)?.[1];
        p.app_rules[state.prefApp] = {
          app_id: state.prefApp,
          app_name: name,
          enabled: checked("app-ai"),
          prompt: value("app-prompt"),
          engine_id: value("app-engine"),
        };
        await api("save_settings", p);
        state.dirty = false;
        await refresh(true);
        toast(Vara.i18n.bind("ui.app-setting-saved"));
        break;
      }
      case "reset-form":
        state.dirty = false;
        render();
        break;
      case "save-settings":
        await saveSettings();
        break;
      case "open-key":
        if ($("modal").open) closeModal();
        await keyDialog(id, button.dataset.name);
        break;
      case "save-key":
        requireValue("api-key", Vara.i18n.bind("ui.enter-your-api-key"));
        try {
          await api("set_key", { id, key: value("api-key") });
          closeModal();
          delete state.connectionChecks[id];
          await loadKeyStatus(id);
          toast(Vara.i18n.bind("ui.the-key-is-saved-in-the-system-key-store"));
        } finally {
          if ($("api-key")) $("api-key").value = "";
        }
        break;
      case "check-groq":
      case "check-openai": {
        const openai = a === "check-openai", provider = openai ? "open_ai" : "groq";
        if (state.connectionChecks[provider]?.phase === "testing") break;
        const status = $(openai ? "openai-access-status" : "groq-access-status");
        const check = { phase: "testing", message: Vara.i18n.bind("ui.testing-key-and-model-access") };
        state.connectionChecks[provider] = check;
        renderConnections();
        if (status) status.textContent = Vara.i18n.bind("ui.testing-key-and-model-access");
        try {
          const result = await api(openai ? "check_openai" : "check_groq");
          if (state.connectionChecks[provider] !== check) break;
          const message = Vara.i18n.bind("key.test.result", {
            speechModel: result.speech_model,
            speechStatus: Vara.i18n.bind(result.speech_available ? "key.test.available" : "key.test.missing"),
            textModel: result.text_model,
            textStatus: Vara.i18n.bind(result.text_available ? "key.test.available" : "key.test.missing"),
          });
          state.connectionChecks[provider] = { phase: "success", message };
          if (status?.isConnected) status.textContent = message;
        } catch (error) {
          if (state.connectionChecks[provider] !== check) break;
          state.connectionChecks[provider] = { phase: "error", message: Vara.i18n.bindRuntime(String(error?.message || error)) };
          if (status?.isConnected) { status.textContent = ""; throw error; }
          $("live").textContent = state.connectionChecks[provider].message;
        } finally { renderConnections(); }
        break;
      }
      case "delete-key":
        await api("delete_key", { id });
        closeModal();
        delete state.connectionChecks[id];
        await loadKeyStatus(id);
        toast(Vara.i18n.bind("ui.key-deleted"));
        break;
      case "add-endpoint":
        engineDialog(null);
        break;
      case "edit-engine":
        engineDialog(id);
        break;
      case "save-engine": {
        requireValue("engine-label", Vara.i18n.bind("ui.give-the-connection-a-name"));
        requireValue("engine-model", Vara.i18n.bind("ui.enter-a-model"));
        if ($("engine-endpoint")) {
          requireValue("engine-endpoint", Vara.i18n.bind("ui.enter-the-endpoint-url"));
          try {
            new URL(value("engine-endpoint"));
          } catch {
            invalid("engine-endpoint", Vara.i18n.bind("ui.enter-a-valid-url"));
          }
        }
        const p = structuredClone(state.data.settings);
        const index = p.engines.findIndex((x) => x.id === id);
        const previous = p.engines[index];
        const engine = previous
          ? { ...previous }
          : {
              id: "custom." + crypto.randomUUID(),
              provider: "custom",
              kind: "text",
            };
        engine.label = value("engine-label");
        engine.model = value("engine-model");
        if ($("engine-endpoint")) {
          engine.endpoint = value("engine-endpoint").trim();
          engine.locality = value("engine-locality");
        }
        if (index >= 0) p.engines[index] = engine;
        else p.engines.push(engine);
        await api("save_settings", p);
        closeModal();
        await refresh();
        toast(Vara.i18n.bind("ui.connection-saved"));
        break;
      }
      case "add-app":
        await addApp();
        break;
      case "save-new-app": {
        const app = state.availableApps.find((a) => a.id === value("new-app"));
        if (!app) throw uiError(Vara.i18n.bind("ui.select-an-app-2"));
        const p = structuredClone(state.data.settings);
        if (!p.app_rules[app.id])
          p.app_rules[app.id] = {
            app_id: app.id,
            app_name: app.label,
            enabled: false,
            prompt: p.general_prompt,
            engine_id: p.general_engine,
          };
        await api("save_settings", p);
        state.prefApp = app.id;
        closeModal();
        await refresh();
        break;
      }
      case "close-modal":
        closeModal();
        break;
      case "discard-unsaved":
        modal(
          Vara.i18n.bind("ui.discard-text"),
          `<p>${Vara.i18n.bind("ui.the-text-is-not-saved-in-history-copy-it-first-if-you-want-to-kee")}</p>${b(Vara.i18n.bind("ui.discard-text-2"), "discard_unsaved", { variant: "destructive" })}`,
        );
        break;
      case "discard_unsaved":
        await api(a);
        closeModal();
        await refresh();
        break;
      case "permission":
        await api(a, { kind: button.dataset.kind });
        await refresh();
        break;
      case "install":
      case "cancel_install":
        await api(a, { id });
        await refresh();
        break;
      case "start":
      case "stop":
      case "cancel":
      case "retry":
      case "save_unsaved":
      case "dismiss_message":
        await api(a, { id });
        await refresh();
        break;
    }
  }
  document.addEventListener("click", async (event) => {
    const button = event.target.closest("[data-action]");
    if (!button || button.disabled) return;
    event.preventDefault();
    button.disabled = true;
    try {
      await action(button);
    } catch (error) {
      if ($("retry-key-store")) $("retry-key-store").hidden = false;
      if ($("modal").open)
        $("dialog-error").innerHTML = notice(
          error?.uiMessage || Vara.i18n.bindRuntime(String(error?.message || error)),
          "error",
        );
      else report(error);
    } finally {
      if (button.isConnected) button.disabled = false;
    }
  });
  document.addEventListener("submit", (event) => event.preventDefault());
  document.addEventListener("input", (event) => {
    if (event.target.getAttribute("aria-invalid") === "true") {
      event.target.removeAttribute("aria-invalid");
      const error = $(event.target.id + "-error");
      if (error) error.textContent = "";
    }
    if (event.target.id === "history-search") {
      state.query = event.target.value;
      render();
      return;
    }
    if (event.target.closest("#settings-form,#app-form")) {
      state.dirty = true;
      document.querySelectorAll(".unsaved").forEach((e) => (e.hidden = false));
      document
        .querySelectorAll(
          '[data-action="save-settings"],[data-action="save-app"]',
        )
        .forEach((e) => (e.disabled = false));
    }
  });
  document.addEventListener("change", (event) => {
    if (event.target.id === "ui-locale") {
      if (pendingLocale !== null) { event.target.value = pendingLocale; return; }
      const preference = event.target.value;
      event.target.disabled = true;
      const previous = state.data.settings.ui_locale || "da";
      pendingLocale = preference;
      localeWrites = localeWrites.then(async () => {
        try {
          const result = await api("set_ui_locale", {preference});
          state.data.settings.ui_locale = preference;
          state.data.resolved_ui_locale = result.locale;
          state.data.ui_locale_revision = result.revision ?? 0;
          Vara.i18n.setLocale(result.locale);
        } catch (error) {
          if (pendingLocale === preference) {
            pendingLocale = null;
            state.data.settings.ui_locale = previous;
            if ($("ui-locale")) $("ui-locale").value = previous;
          }
          if ($("ui-locale")) $("ui-locale").disabled = false;
          report(uiError(Vara.i18n.bind("locale.error")));
          return;
        }
        if (pendingLocale === preference) pendingLocale = null;
        if ($("ui-locale")) $("ui-locale").disabled = false;
      });
      return;
    }
    if (event.target.id === "history-app") {
      state.filter = event.target.value;
      render();
    } else if (event.target.closest("#settings-form,#app-form")) {
      if (event.target.id === "speech-engine") {
        const language = $("language"), automatic = value("speech-engine") === "local.parakeet";
        if (automatic && !language.disabled) { language.dataset.previousLanguage = language.value; language.value = ""; }
        if (!automatic && language.disabled) language.value = language.dataset.previousLanguage || "";
        language.disabled = automatic;
        document.getElementById("language-help").textContent = automatic ? Vara.i18n.bind("ui.parakeet-detects-the-language-automatically") : "";
      }
      if (["speech-engine", "general-engine", "general-ai", "direct-text", "app-engine", "app-ai", "strict-local"].includes(event.target.id)) refreshModelControls();
      state.dirty = true;
      document.querySelectorAll(".unsaved").forEach((e) => (e.hidden = false));
      document
        .querySelectorAll(
          '[data-action="save-settings"],[data-action="save-app"]',
        )
        .forEach((e) => (e.disabled = false));
    }
  });
  document.addEventListener(
    "toggle",
    (event) => {
      if (event.target.matches?.(".original-disclosure")) {
        state.originalOpen = event.target.open;
        event.target.querySelector("summary span").textContent = event.target
          .open
          ? Vara.i18n.bind("ui.hide-original")
          : Vara.i18n.bind("ui.show-original");
      }
      if (event.target.matches?.(".settings-disclosure"))
        state.disclosures[event.target.id] = event.target.open;
    },
    true,
  );
  $("modal").addEventListener("cancel", (event) => {
    event.preventDefault();
    closeModal();
  });
  function recordShortcutKey(code, down, source = "dom", flags = null) {
    if (!$("modal").open || !$("shortcut-recorder")) return;
    if (!code || code === "Unidentified") {
      $("shortcut-warning").textContent = Vara.i18n.bind("ui.the-system-did-not-send-a-recognizable-key-try-another-key");
      return;
    }
    if (source === "dom" && flags && !/^(Control|Alt|Shift|Meta)(Left|Right)$/.test(code)) {
      state.shortcutImplied.clear();
      if (down) {
        for (const [prefix, flag] of [["Control",flags.ctrlKey],["Alt",flags.altKey],["Shift",flags.shiftKey],["Meta",flags.metaKey]]) {
          if (flag && ![...state.shortcutSources.dom, ...state.shortcutSources.native].some(key => key.startsWith(prefix))) state.shortcutImplied.add(prefix + "Left");
        }
      }
    }
    const keysForSource = state.shortcutSources[source];
    if (down) keysForSource.add(code); else keysForSource.delete(code);
    state.shortcutKeys = new Set([...state.shortcutSources.dom, ...state.shortcutSources.native, ...state.shortcutImplied]);
    if (down) {
      const keys = [...state.shortcutKeys].sort();
      const modifier = /^(Control|Alt|Shift|Meta)(Left|Right)$/;
      const normal = keys.filter(k => !modifier.test(k));
      if (!state.nativeCapture && normal.length === 1 && !["Fn", "CapsLock"].includes(normal[0])) {
        const mods = ["Control", "Alt", "Shift", "Meta"].filter(m => keys.some(k => k.startsWith(m))).map(m => ({Control:"Ctrl",Meta:"Super"})[m] || m);
        const key = normal[0].replace(/^(Key|Digit)/, "");
        state.shortcutDraft = [...mods, key].join("+");
      } else state.shortcutDraft = "Physical:" + keys.join("+");
      $("shortcut-recorder").textContent = Vara.desktopUi.binding(state.shortcutDraft);
    }
    $("modal").querySelector('[data-action="use-shortcut"]').disabled = !state.shortcutDraft || state.shortcutKeys.size > 0;
  }
  document.addEventListener("keydown", event => {
    if ($("modal").open && $("shortcut-recorder") && document.activeElement === $("shortcut-recorder")) {
      if (event.key !== "Tab") { event.preventDefault(); event.stopPropagation(); }
      if (!event.repeat) recordShortcutKey(event.code, true, "dom", event);
      return;
    }
    if (event.key === "Escape" && state.data?.session && !$("modal").open) api("cancel").catch(report);
  }, true);
  document.addEventListener("keyup", event => {
    if ($("modal").open && $("shortcut-recorder") && state.shortcutKeys.has(event.code)) {
      if (event.key !== "Tab") event.preventDefault();
      recordShortcutKey(event.code, false, "dom", event);
    }
  }, true);
  window.addEventListener("blur", () => {
    if ($("shortcut-recorder")) closeModal();
  });
  window.addEventListener("focus", () => {
    if (state.data) refresh().catch(report);
  });
  try {
    try { Vara.i18n.setRuntimeCatalog(await api("runtime_catalog")); } catch { /* Older hosts and test fixtures may not expose the catalog. */ }
    const data = await api("snapshot");
    if (!data.settings.onboarding_complete && data.history.length === 0)
      state.page = "setup";
    apply(data);
    Vara.transcription.init({api,render,modal,closeModal,toast},state);
    await window.__TAURI__.event.listen("vara:state", (event) =>
      apply(event.payload),
    );
    await window.__TAURI__.event.listen("vara:shortcut-key", event => {
      if (state.nativeCapture && (document.activeElement === $("shortcut-recorder") || (!event.payload.down && state.shortcutSources.native.has(event.payload.code))))
        recordShortcutKey(event.payload.code, event.payload.down, "native");
    });
    // Device drivers must never hold onboarding or history hostage.
    let deviceDeadline;
    Promise.race([
      api("devices"),
      new Promise((_, reject) => {
        deviceDeadline = setTimeout(() => reject(uiError(Vara.i18n.bind("ui.the-microphone-list-could-not-be-loaded-the-system-default-can-st"))), 5000);
      }),
    ]).then(devices => {
      state.devices = devices;
    }).catch(error => {
      state.deviceError = error.uiMessage || Vara.i18n.bindRuntime(String(error.message || error));
    }).finally(() => {
      clearTimeout(deviceDeadline);
      if (state.page === "settings" && !state.dirty) render();
    });
  } catch (error) {
    $("workspace").innerHTML = notice(error?.uiMessage || Vara.i18n.bindRuntime(String(error?.message || error)), "error");
  }
})();
