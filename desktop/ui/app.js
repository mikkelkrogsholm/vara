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
      throw Error(
        "Åbn Vara som desktopapp for at bruge mikrofon, modeller og historik.",
      );
    return window.__TAURI__.core.invoke("command", { action, payload });
  };
  const report = (error) => {
    const message = String(error?.message || error);
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
  }
  function apply(data) {
    const previous = state.data?.history[0]?.id;
    if (state.data && JSON.stringify(state.data) === JSON.stringify(data))
      return;
    state.data = data;
    if (!state.selected && data.history.length)
      state.selected = data.history[0].id;
    if (data.history[0]?.id && previous !== data.history[0].id) {
      state.selected = data.history[0].id;
      if (!state.dirty) {
        state.page = "history";
        state.detailOpen = true;
      } else toast("En ny diktering er gemt i historikken.");
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
    }
  }
  const refresh = async (force = false) => {
    const data = await api("snapshot");
    if (force) {
      state.data = data;
      render();
    } else apply(data);
  };
  function canLeave() {
    if (!state.dirty) return true;
    toast("Gem ændringerne, eller vælg Fortryd ændringer først.");
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
      `<div class="dialog-content">${Vara.ui.iconButton("close", "Luk dialog", "close-modal")}<h2 id="dialog-title">${e(title)}</h2>${content}<div class="form-message" role="alert" id="dialog-error"></div></div>`;
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
    throw Error(message);
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
      "Ændringerne er ikke gemt",
      `<p>Gem først dine ændringer, eller kassér dem for at fortsætte.</p><div class="actions">${b("Bliv her", "close-modal", { variant: "primary" })}${b("Kassér ændringer", "discard-route", { variant: "quiet" })}</div>`,
    );
  }
  async function saveSettings() {
    const p = structuredClone(state.data.settings);
    if (state.tab === "dictation") {
      p.activation = value("activation");
      p.shortcut = value("shortcut").trim();
      p.input_device = value("input-device") || null;
      p.language = value("language") || null;
      p.general_ai_enabled = checked("general-ai");
      p.general_prompt = value("general-prompt");
      p.general_engine = value("general-engine");
      if (p.general_ai_enabled)
        requireValue(
          "general-prompt",
          "Skriv en systemprompt, før du slår AI til.",
        );
    }
    if (state.tab === "connections") {
      p.speech_engine = value("speech-engine");
      p.strict_local = checked("strict-local");
    }
    if (state.tab === "privacy")
      p.retention_days = value("retention") ? Number(value("retention")) : null;
    await api("save_settings", p);
    state.dirty = false;
    await refresh(true);
    toast("Indstillinger gemt.");
  }
  async function keyDialog(id, name) {
    if (!canLeave()) return;
    const exists = await api("key_status", { id });
    modal(
      name,
      `<form id="key-form" data-key-id="${e(id)}"><p>${exists ? "En nøgle er gemt i systemets nøglelager. Du kan erstatte eller slette den." : "Tilføj din egen API-nøgle. Den gemmes i systemets nøglelager."}</p>${["groq", "open_ai"].includes(id) ? "<p>Udbyderen kan transskribere din lyd og bearbejde tekst med AI. Data til de valgte funktioner behandles i cloud.</p>" : ""}${f({ id: "api-key", label: "API-nøgle", type: "password", attrs: 'autocomplete="off" spellcheck="false" autocapitalize="off"' })}<div class="actions">${b("Gem nøgle", "save-key", { variant: "primary", attrs: `data-id="${e(id)}"` })}${exists ? b("Slet nøgle", "delete-key", { variant: "quiet", attrs: `data-id="${e(id)}"` }) : ""}${["groq", "open_ai"].includes(id) && exists ? b("Kontrollér adgang", id === "groq" ? "check-groq" : "check-openai", { variant: "quiet" }) : ""}</div>${["groq", "open_ai"].includes(id) ? `<p id="${id === "groq" ? "groq" : "openai"}-access-status" role="status" class="muted"></p>` : ""}</form>`,
    );
  }
  function engineDialog(id) {
    if (!canLeave()) return;
    const engine = state.data.settings.engines.find((x) => x.id === id);
    const custom = !engine || ["custom", "ollama"].includes(engine.provider);
    modal(
      engine ? "Redigér forbindelse" : "Eget endpoint",
      `<form id="engine-form" data-engine-id="${e(id || "")}">${f({ id: "engine-label", label: "Navn", value: engine?.label || "" })}${custom ? f({ id: "engine-endpoint", label: "Endpoint-URL", value: engine?.endpoint || "", help: "OpenAI-kompatibel tekst-API. Eksempel: http://127.0.0.1:11434/v1" }) : ""}${f({ id: "engine-model", label: "Model", value: engine?.model || "" })}${
        custom
          ? f({
              id: "engine-locality",
              label: "Hvor behandles teksten?",
              value: engine?.locality || "on_device",
              options: [
                ["on_device", "På denne enhed · kun loopback"],
                ["lan", "På eget netværk · HTTPS"],
                ["cloud", "Hos ekstern udbyder · HTTPS"],
              ],
            })
          : ""
      }<div class="actions">${b("Gem forbindelse", "save-engine", { variant: "primary", attrs: `data-id="${e(id || "")}"` })}${custom && engine ? b("Administrer nøgle", "open-key", { attrs: `data-id="${e(engine.id)}" data-name="${e(engine.label)}"` }) : ""}</div></form>`,
    );
  }
  async function addApp() {
    if (!canLeave()) return;
    const apps = await api("applications");
    modal(
      "Tilføj app",
      apps.length
        ? `${f({ id: "new-app", label: "App", options: apps.sort((a, b) => a.label.localeCompare(b.label)).map((a) => [a.id, a.label]) })}${b("Tilføj app", "save-new-app", { variant: "primary" })}`
        : notice(
            "Begynd at diktere i en app. Den bliver tilgængelig her, når den første diktering er gemt.",
          ),
    );
    state.availableApps = apps;
  }
  async function action(button) {
    const a = button.dataset.action,
      id = button.dataset.id;
    switch (a) {
      case "navigate":
        navigate(() => {
          state.page = button.dataset.page;
          state.detailOpen = false;
        });
        break;
      case "choose-processing":
        modal(
          "Næste diktering",
          f({
            id: "next-choice",
            label: "Behandling",
            value: state.data.next_choice || "default",
            options: [
              ["default", "Følg dine indstillinger"],
              ["direct", "Direkte transskription"],
              ["general_ai", "Brug generel systemprompt"],
            ],
          }) +
            "<p>Gælder én diktering, også når du bruger genvejen i en anden app. AI bruger din valgte tekstforbindelse.</p>" +
            b("Vælg", "arm-processing", { variant: "primary" }),
        );
        break;
      case "arm-processing":
        await api("set_next_choice", { choice: value("next-choice") });
        closeModal();
        await refresh();
        break;
      case "check-cli":
        state.cli[id] = await api("check_cli", { provider: id });
        render();
        break;
      case "record-shortcut":
        state.nativeCapture = !!(await api("shortcut_capture", { enabled: true }))?.native;
        state.shortcutDraft = null;
        state.shortcutKeys.clear();
        Object.values(state.shortcutSources).forEach(keys => keys.clear());
      state.shortcutImplied.clear();
        modal(
          "Skift genvej",
          `<p>Tryk og slip en tast eller en kombination. Venstre og højre modifikationstast kan vælges hver for sig.</p><div class="shortcut-recorder" id="shortcut-recorder" tabindex="0" role="group" aria-label="Optag genvej" aria-live="polite">Tryk din genvej…</div><p class="muted" id="shortcut-warning">Almindelige skrivetaster og systemgenveje kan også påvirke den app, du bruger.</p><div class="actions">${b("Brug genvej", "use-shortcut", { variant: "primary", attrs: "disabled" })}${b("Annuller", "close-modal")}</div>`,
        );
        $("shortcut-recorder").focus();
        break;
      case "use-shortcut":
        if (!state.shortcutDraft)
          throw Error(
            "Tryk og slip den ønskede tast eller kombination.",
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
      case "open-connections":
        if (canLeave()) {
          state.page = "settings";
          state.tab = "connections";
          render();
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
        toast(a === "copy" ? "Kopieret." : "Originalen er kopieret.");
        break;
      case "copy-unsaved":
        await api("copy", { unsaved: true });
        toast("Teksten er kopieret.");
        break;
      case "result-menu":
        modal(
          "Diktering",
          `<div class="actions">${b("Kopiér", "copy", { attrs: `data-id="${e(id)}"` })}${b("Slet diktering", "confirm-delete", { variant: "quiet", attrs: `data-id="${e(id)}"` })}</div>`,
        );
        break;
      case "confirm-delete":
        modal(
          "Slet diktering?",
          `<p>Både originalen og den færdige tekst slettes fra historikken.</p><div class="actions">${b("Slet diktering", "delete-result", { variant: "destructive", attrs: `data-id="${e(id)}"` })}${b("Behold", "close-modal")}</div>`,
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
            "Skriv en systemprompt, før du slår AI til.",
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
        toast("App-indstilling gemt.");
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
        requireValue("api-key", "Indtast din API-nøgle.");
        try {
          await api("set_key", { id, key: value("api-key") });
          closeModal();
          toast("Nøglen er gemt i systemets nøglelager.");
        } finally {
          if ($("api-key")) $("api-key").value = "";
        }
        break;
      case "check-groq":
      case "check-openai": {
        const openai = a === "check-openai";
        const status = $(openai ? "openai-access-status" : "groq-access-status");
        if (status) status.textContent = "Kontrollerer nøgle og modeladgang…";
        let result;
        try { result = await api(openai ? "check_openai" : "check_groq"); }
        catch (error) { if (status?.isConnected) status.textContent = ""; throw error; }
        if (status?.isConnected) status.textContent = `Nøglen accepteres. Tale (${result.speech_model}): ${result.speech_available ? "i modelkataloget" : "mangler i modelkataloget"}. Tekst (${result.text_model}): ${result.text_available ? "i modelkataloget" : "mangler i modelkataloget"}. Ingen lyd eller tekst blev sendt ved kontrollen.${!result.speech_available || !result.text_available ? " Ret eventuelt modelnavnet under Modeller og egne endpoints." : ""}`;
        break;
      }
      case "delete-key":
        await api("delete_key", { id });
        closeModal();
        toast("Nøglen er slettet.");
        break;
      case "add-endpoint":
        engineDialog(null);
        break;
      case "edit-engine":
        engineDialog(id);
        break;
      case "save-engine": {
        requireValue("engine-label", "Giv forbindelsen et navn.");
        requireValue("engine-model", "Angiv en model.");
        if ($("engine-endpoint")) {
          requireValue("engine-endpoint", "Angiv endpointets URL.");
          try {
            new URL(value("engine-endpoint"));
          } catch {
            invalid("engine-endpoint", "Angiv en gyldig URL.");
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
        toast("Forbindelsen er gemt.");
        break;
      }
      case "add-app":
        await addApp();
        break;
      case "save-new-app": {
        const app = state.availableApps.find((a) => a.id === value("new-app"));
        if (!app) throw Error("Vælg en app.");
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
          "Kassér teksten?",
          `<p>Teksten er ikke gemt i historikken. Kopiér den først, hvis du vil beholde den.</p>${b("Kassér teksten", "discard_unsaved", { variant: "destructive" })}`,
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
      if ($("modal").open)
        $("dialog-error").innerHTML = notice(
          String(error?.message || error),
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
    if (event.target.id === "history-app") {
      state.filter = event.target.value;
      render();
    } else if (event.target.closest("#settings-form,#app-form")) {
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
          ? "Skjul original"
          : "Vis original";
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
      $("shortcut-warning").textContent = "Systemet sender ikke en genkendelig tast. Prøv en anden tast.";
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
    if ($("modal").open && $("shortcut-recorder")) {
      event.preventDefault(); event.stopPropagation();
      if (!event.repeat) recordShortcutKey(event.code, true, "dom", event);
      return;
    }
    if (event.key === "Escape" && state.data?.session && !$("modal").open) api("cancel").catch(report);
  }, true);
  document.addEventListener("keyup", event => {
    if ($("modal").open && $("shortcut-recorder")) {
      event.preventDefault();
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
    const data = await api("snapshot");
    if (!data.settings.onboarding_complete && data.history.length === 0)
      state.page = "setup";
    apply(data);
    await window.__TAURI__.event.listen("vara:state", (event) =>
      apply(event.payload),
    );
    await window.__TAURI__.event.listen("vara:shortcut-key", event => {
      if (state.nativeCapture) recordShortcutKey(event.payload.code, event.payload.down, "native");
    });
    // Device drivers must never hold onboarding or history hostage.
    let deviceDeadline;
    Promise.race([
      api("devices"),
      new Promise((_, reject) => {
        deviceDeadline = setTimeout(() => reject(Error("Mikrofonlisten kunne ikke indlæses. Systemets standard kan stadig vælges.")), 5000);
      }),
    ]).then(devices => {
      state.devices = devices;
    }).catch(error => {
      state.deviceError = String(error.message || error);
    }).finally(() => {
      clearTimeout(deviceDeadline);
      if (state.page === "settings" && !state.dirty) render();
    });
  } catch (error) {
    $("workspace").innerHTML = notice(String(error?.message || error), "error");
  }
})();
