/* Single demo controller. All rendering delegates to views/components; no real OS/provider I/O. */
(() => {
  "use strict";
  const s = Vara.makeState(),
    $ = (selector) => document.querySelector(selector),
    { notice, esc } = Vara.ui;
  let toastTimer,
    returnFocus = null;
  const announce = (text) => ($("#live").textContent = text);
  function toast(text) {
    clearTimeout(toastTimer);
    $("#toast").textContent = text;
    $("#toast").hidden = false;
    announce(text);
    toastTimer = setTimeout(() => ($("#toast").hidden = true), 4000);
  }
  function render({ focus = false } = {}) {
    const history = $("#history-scroll");
    if (history?.clientHeight) s.historyScroll = history.scrollTop;
    $("#sidebar").innerHTML = Vara.views.sidebar(s);
    $("#workspace").className =
      "workspace " +
      (s.page === "history" ? "is-history" : "is-form") +
      (s.detailOpen ? " has-detail" : "");
    $("#workspace").innerHTML = Vara.views[s.page](s);
    if ($("#history-scroll"))
      $("#history-scroll").scrollTop = s.historyScroll || 0;
    renderHUD();
    if (focus) $("#view-title")?.focus();
  }
  function renderHUD() {
    $("#hud-region").hidden = !s.active && s.page !== "history";
    $("#hud-region").innerHTML = Vara.views.hud(s);
  }
  function openModal(type, { replace = false } = {}) {
    if (!replace) returnFocus = document.activeElement;
    s.modal = type;
    $("#modal").innerHTML = Vara.views.dialog(s);
    if (!$("#modal").open) $("#modal").showModal();
    if (type === "shortcut") $("#shortcut-recorder").focus();
  }
  function refreshModal() {
    if (s.modal) $("#modal").innerHTML = Vara.views.dialog(s);
  }
  function closeModal() {
    $("#modal")
      .querySelectorAll("input[type=password]")
      .forEach((x) => (x.value = ""));
    $("#modal").close();
    $("#modal").innerHTML = "";
    s.modal = null;
    s.shortcutCapture = false;
    if (returnFocus?.isConnected) returnFocus.focus();
    else $("#view-title")?.focus();
  }
  function guardNavigate(next) {
    if (s.dirty) {
      s.pendingNavigation = next;
      openModal("unsaved");
      return;
    }
    next();
  }
  function navigate(page) {
    guardNavigate(() => {
      s.page = page;
      s.detailOpen = false;
      render({ focus: true });
    });
  }
  function error(id, text) {
    const target = $("#" + id);
    if (target) target.innerHTML = notice(text, "error");
    announce(text);
  }
  function saveApp() {
    const prompt = $("#app-prompt").value.trim(),
      enabled = $("#app-ai").checked,
      engine = $("#app-engine").value;
    if (enabled && !prompt) {
      $("#app-prompt").setAttribute("aria-invalid", "true");
      $("#app-prompt").setAttribute("aria-describedby", "app-prompt-error");
      $("#app-prompt-error").textContent =
        "Enter a system prompt before enabling AI.";
      $("#app-prompt").focus();
      return false;
    }
    if (enabled && s.settings.strictLocal && engine !== "local") {
      error(
        "app-form-message",
        "This engine is not on device. Choose a local model or change your local policy.",
      );
      return false;
    }
    s.prefs[s.prefApp] = { enabled, prompt, engine };
    s.dirty = false;
    return true;
  }
  function selectResult(id) {
    s.selected = id;
    s.originalOpen = false;
    s.detailOpen = true;
    render();
    if (matchMedia("(max-width:1039px)").matches)
      $(".detail-header h2")?.setAttribute("tabindex", "-1");
    if (matchMedia("(max-width:1039px)").matches)
      $(".detail-header h2")?.focus();
    else $(".row-select[aria-pressed=true]")?.focus({ preventScroll: true });
  }
  function startSession() {
    if (s.active) return;
    const app = s.demo.app,
      p = { ...s.prefs[app] },
      choice = s.demo.processing;
    const processing =
      choice === "general" || (choice === "default" && p.enabled);
    const engine = choice === "general" ? "local" : p.engine;
    if (
      s.settings.strictLocal &&
      (s.settings.speech !== "local" || (processing && engine !== "local"))
    ) {
      error(
        "dialog-error",
        "On-device-only processing is selected. Change the engine before continuing.",
      );
      return;
    }
    s.active = {
      token: ++s.generation,
      app,
      processing,
      prompt: choice === "general" ? s.settings.general : p.prompt,
      promptKind: choice === "general" ? "general" : "app",
      engine,
      local:
        s.settings.speech === "local" && (!processing || engine === "local"),
      outcome: s.demo.outcome,
      stage: "recording",
      created: Date.now(),
    };
    closeModal();
    s.hudMessage = "";
    renderHUD();
    announce("Listening in the demo. No audio is recorded.");
  }
  function stopSession() {
    if (!s.active || s.active.stage !== "recording") return;
    const token = s.active.token;
    s.active.stage = "speech";
    renderHUD();
    announce("Finding your words in the demo.");
    setTimeout(() => {
      if (!s.active || s.active.token !== token) return;
      const a = s.active;
      if (a.retryId && a.expires <= Date.now()) {
        s.recoveries = s.recoveries.filter((r) => r.id !== a.retryId);
        s.active = null;
        render();
        toast("The recording expired and cannot be retried.");
        return;
      }
      if (a.outcome === "asr") {
        s.recoveries.push({
          ...a,
          id: s.nextId++,
          expires: a.created + 86400000,
        });
        s.active = null;
        s.hudMessage = "Could not transcribe";
        if (s.page === "history") render();
        else renderHUD();
        announce(
          "Transcription failed. The recording can be retried before the deadline.",
        );
        return;
      }
      if (a.processing) {
        s.active.stage = "text";
        renderHUD();
        announce("Refining the text in the demo.");
        setTimeout(() => finishSession(token), 550);
      } else finishSession(token);
    }, 550);
  }
  function finishSession(token) {
    if (!s.active || s.active.token !== token) return;
    const a = s.active;
    if (a.retryId && a.expires <= Date.now()) {
      s.recoveries = s.recoveries.filter((r) => r.id !== a.retryId);
      s.active = null;
      render();
      toast("The recording expired before the retry finished.");
      return;
    }
    const processed = a.processing && a.outcome !== "ai";
    const r = {
      id: s.nextId++,
      app: a.app,
      date: "Just now",
      duration: 12,
      raw: "hi anna thanks for your email thursday at ten works for me please send an invitation",
      text: processed
        ? "Hi Anna. Thanks for your email. Thursday at 10 works for me. Please send an invitation."
        : undefined,
      processed,
      fallback: a.processing && a.outcome === "ai",
      local: a.local,
      status: a.outcome === "focus" ? "manual" : "inserted",
    };
    s.results.unshift(r);
    s.recoveries = s.recoveries.filter((r) => r.id !== a.retryId);
    s.selected = r.id;
    s.originalOpen = false;
    s.detailOpen = true;
    s.query = "";
    s.filter = "all";
    s.collapsed = [];
    s.active = null;
    s.hudMessage =
      r.status === "manual" ? "Ready to paste" : "Inserted · demo";
    if (s.page === "history") render();
    else renderHUD();
    toast(
      r.status === "manual"
        ? "Focus changed. The text is ready in history."
        : r.fallback
          ? "AI failed. The original was used in the demo."
          : "The dictation was inserted and saved in the demo.",
    );
  }
  function cancelSession() {
    if (!s.active) return;
    ++s.generation;
    s.active = null;
    s.hudMessage = "Cancelet";
    renderHUD();
    toast("The dictation was cancelled. No result was saved.");
  }
  function retry(id) {
    if (s.active) {
      toast("Finish or cancel the current dictation first.");
      return;
    }
    const r = s.recoveries.find((r) => r.id === id);
    if (!r) return;
    if (r.expires <= Date.now()) {
      s.recoveries = s.recoveries.filter((r) => r.id !== id);
      render();
      toast("The recording expired.");
      return;
    }
    s.active = {
      ...r,
      retryId: r.id,
      token: ++s.generation,
      outcome: "focus",
      stage: "recording",
    };
    stopSession();
  }
  const actions = {
    navigate: (b) => navigate(b.dataset.page),
    "select-result": (b) => selectResult(Number(b.dataset.id)),
    "back-history": () => {
      s.detailOpen = false;
      render();
      $(".row-select[aria-pressed=true]")?.focus({ preventScroll: true });
    },
    "toggle-group": (b) => {
      const a = b.dataset.app;
      s.collapsed = s.collapsed.includes(a)
        ? s.collapsed.filter((x) => x !== a)
        : [...s.collapsed, a];
      render();
      $(`[data-action="toggle-group"][data-app="${a}"]`)?.focus();
    },
    "toggle-search": () => {
      s.searchOpen = !s.searchOpen;
      render();
      if (s.searchOpen) $("#history-search").focus();
    },
    "clear-search": () => {
      s.query = "";
      s.filter = "all";
      render();
    },
    "copy-result": () =>
      toast("Copying is simulated. Your clipboard was not changed."),
    "result-menu": (b) => {
      s.deleteId = Number(b.dataset.id);
      openModal("result-menu");
    },
    "menu-copy": () => {
      closeModal();
      toast("Copying is simulated. Your clipboard was not changed.");
    },
    "ask-delete": () => openModal("delete", { replace: true }),
    "confirm-delete": () => {
      s.results = s.results.filter((r) => r.id !== s.deleteId);
      if (s.selected === s.deleteId) s.selected = s.results[0]?.id ?? null;
      closeModal();
      render();
      toast("The dictation and both text versions were deleted in the demo.");
    },
    "select-pref": (b) =>
      guardNavigate(() => {
        s.prefApp = b.dataset.app;
        render();
        $(".form-title h2")?.setAttribute("tabindex", "-1");
        $(".form-title h2")?.focus();
      }),
    "save-app": () => {
      if (saveApp()) {
        render();
        toast("The app setting was saved in the demo.");
      }
    },
    "reset-app": () => {
      s.dirty = false;
      render();
    },
    "save-and-navigate": () => {
      const next = s.pendingNavigation;
      closeModal();
      if (saveApp()) {
        s.pendingNavigation = null;
        next?.();
      }
    },
    "discard-and-navigate": () => {
      s.dirty = false;
      const next = s.pendingNavigation;
      closeModal();
      s.pendingNavigation = null;
      next?.();
    },
    "settings-tab": (b) => {
      s.settingsTab = b.dataset.tab;
      render();
      $(`[data-tab="${s.settingsTab}"]`)?.focus();
    },
    "save-general": () => {
      const t = $("#general-prompt").value.trim();
      if (!t) {
        $("#general-prompt-error").textContent = "Enter a system prompt.";
        $("#general-prompt").focus();
        return;
      }
      s.settings.general = t;
      toast("The general prompt was saved in the demo.");
    },
    "record-shortcut": () => {
      s.shortcutDraft = "";
      s.shortcutCapture = true;
      openModal("shortcut");
    },
    "save-shortcut": () => {
      if (!s.shortcutDraft) {
        error(
          "shortcut-error",
          "Press a combination with Ctrl, Alt, or Command.",
        );
        return;
      }
      if (/(?:⌘|Ctrl) Q$/.test(s.shortcutDraft)) {
        error(
          "shortcut-error",
          "This shortcut is reserved in the demo. The previous shortcut was kept.",
        );
        return;
      }
      s.settings.shortcut = s.shortcutDraft;
      closeModal();
      render();
      toast("The shortcut was saved in the demo. No system shortcut was registered.");
    },
    "open-key": (b) => {
      s.keyProvider = b.dataset.provider;
      openModal("key");
    },
    "save-key": () => {
      if ($("#api-key").value.trim().length < 4) {
        error("key-error", "Enter a fictional key of at least four characters.");
        $("#api-key").setAttribute("aria-invalid", "true");
        $("#api-key").focus();
        return;
      }
      s.keyConfigured[$("#key-provider").value] = true;
      $("#api-key").value = "";
      closeModal();
      render();
      toast(
        "The key status is simulated. No key was saved or validated.",
      );
    },
    "save-endpoint": () => {
      const name = $("#endpoint-name").value.trim(),
        model = $("#endpoint-model").value.trim(),
        locality = $("#endpoint-locality").value;
      try {
        if (!name || !model) throw Error("Enter a name and model.");
        const url = new URL($("#endpoint-url").value);
        if (!["http:", "https:"].includes(url.protocol))
          throw Error("Enter an HTTP or HTTPS address.");
        if (s.settings.strictLocal && locality !== "on_device")
          throw Error(
            "On-device-only processing is selected. LAN/cloud cannot be added under this policy.",
          );
        s.endpoints.push({ name, model, locality });
        $("#endpoint-key").value = "";
        render();
        toast(
          "The connection was saved as an example. No network calls were made.",
        );
      } catch (e) {
        error(
          "endpoint-error",
          e instanceof TypeError
            ? "Enter a valid HTTP or HTTPS address."
            : e.message,
        );
      }
    },
    "save-retention": () => {
      const days = Number($("#cleanup-days").value),
        enabled = $("#cleanup-toggle").checked;
      if (enabled && (!Number.isInteger(days) || days < 1)) {
        error("retention-error", "Choose a positive whole number of days.");
        return;
      }
      s.settings.cleanup = enabled;
      s.settings.days = days > 0 ? days : 30;
      toast("Cleanup was saved in the demo.");
    },
    "open-demo": () => openModal("demo"),
    "close-modal": () => closeModal(),
    "start-session": () => startSession(),
    "stop-session": () => stopSession(),
    "cancel-session": () => cancelSession(),
    retry: (b) => retry(Number(b.dataset.id)),
    "delete-recovery": (b) => {
      const id = Number(b.dataset.id);
      if (s.active?.retryId === id) cancelSession();
      s.recoveries = s.recoveries.filter((r) => r.id !== id);
      render();
      toast("The recording was deleted in the demo.");
    },
    "empty-history": () => {
      cancelSession();
      s.results = [];
      s.recoveries = [];
      s.selected = null;
      s.page = "history";
      s.detailOpen = false;
      closeModal();
      render();
    },
    "reset-demo": () => {
      cancelSession();
      Object.assign(s, Vara.makeState());
      closeModal();
      render();
    },
    "long-result": () => {
      const r = {
        ...s.results[0],
        id: s.nextId++,
        app: "code",
        date: "Just now",
        raw: "A longer technical note with names and numbers. Preserve the meaning and let the text wrap naturally.\n\n".repeat(
          120,
        ),
        text: undefined,
        processed: false,
        status: "manual",
      };
      s.results.unshift(r);
      s.selected = r.id;
      s.page = "history";
      s.detailOpen = true;
      closeModal();
      render();
    },
    "open-setup": () => {
      s.setup.step = 0;
      openModal("setup", { replace: true });
    },
    "setup-local": () => {
      if (s.setup.cloud) s.setup.trial = false;
      s.setup.cloud = false;
      refreshModal();
    },
    "setup-cloud": () => {
      if (!s.setup.cloud) s.setup.trial = false;
      s.setup.cloud = true;
      refreshModal();
    },
    "setup-next": () => {
      s.setup.step = 1;
      refreshModal();
    },
    "setup-back": () => {
      s.setup.step = 0;
      refreshModal();
    },
    "setup-download": () => {
      const token = (s.setup.downloadToken = (s.setup.downloadToken || 0) + 1);
      s.setup.download = true;
      refreshModal();
      setTimeout(() => {
        if (token !== s.setup.downloadToken) return;
        s.setup.model = true;
        s.setup.download = false;
        if (s.modal === "setup") refreshModal();
      }, 600);
    },
    "cancel-download": () => {
      s.setup.downloadToken++;
      s.setup.download = false;
      refreshModal();
    },
    "setup-key": () => {
      s.setup.key = true;
      refreshModal();
    },
    "setup-mic": () => {
      s.setup.mic = true;
      refreshModal();
    },
    "setup-access": () => {
      s.setup.access = true;
      refreshModal();
    },
    "setup-shortcut": () => {
      s.setup.shortcut = true;
      refreshModal();
    },
    "setup-ready": () => {
      s.setup.step = 2;
      refreshModal();
    },
    "setup-trial": () => {
      s.setup.trial = true;
      refreshModal();
    },
    "setup-finish": () => {
      closeModal();
      toast("First-time setup is complete in the demo. No real audio was recorded.");
    },
  };
  document.addEventListener("click", (event) => {
    const b = event.target.closest("[data-action]");
    if (!b || b.disabled) return;
    actions[b.dataset.action]?.(b);
  });
  document.addEventListener("submit", (event) => {
    event.preventDefault();
    if (event.target.id === "key-form") actions["save-key"]();
    else if (event.target.id === "endpoint-form") actions["save-endpoint"]();
    else if (event.target.id === "app-form") actions["save-app"]();
  });
  document.addEventListener("input", (event) => {
    const t = event.target;
    if (t.closest("#app-form")) {
      s.dirty = true;
      $("#unsaved").hidden = false;
      t.removeAttribute("aria-invalid");
      if ($("#" + t.id + "-error")) $("#" + t.id + "-error").textContent = "";
    }
    if (t.id === "history-search") {
      const pos = t.selectionStart;
      s.query = t.value;
      render();
      $("#history-search").focus();
      try {
        $("#history-search").setSelectionRange(pos, pos);
      } catch {}
    }
  });
  document.addEventListener("change", (event) => {
    const t = event.target;
    if (t.closest("#app-form")) {
      s.dirty = true;
      $("#unsaved").hidden = false;
    }
    if (["demo-app", "demo-case", "demo-processing"].includes(t.id)) {
      s.demo[
        {
          "demo-app": "app",
          "demo-case": "outcome",
          "demo-processing": "processing",
        }[t.id]
      ] = t.value;
      renderHUD();
    }
    if (t.id === "history-app") {
      s.filter = t.value;
      render();
      $("#history-app")?.focus();
    }
    if (t.id === "activation-style") {
      s.settings.style = t.value;
      $("#sidebar").innerHTML = Vara.views.sidebar(s);
      toast("Activation was changed in the demo.");
    }
    if (t.id === "input-device") {
      s.settings.device = t.value;
      toast("The microphone selection is simulated.");
    }
    if (t.id === "speech-choice") {
      if (s.settings.strictLocal && t.value !== "local") {
        t.value = "local";
        toast("On-device-only processing is selected. Cloud was rejected.");
        return;
      }
      s.settings.speech = t.value;
      render();
    }
    if (t.id === "strict-local") {
      if (
        t.checked &&
        (s.settings.speech !== "local" ||
          Object.values(s.prefs).some((p) => p.enabled && p.engine !== "local"))
      ) {
        t.checked = false;
        toast(
          "Choose local audio and text models first. The previous policy was kept.",
        );
        return;
      }
      s.settings.strictLocal = t.checked;
      toast("The local policy was saved in the demo.");
    }
    if (t.id === "app-engine")
      $("#app-engine-help").textContent = Vara.views.locality
        ? Vara.views.locality(t.value)
        : t.value === "local"
          ? "The text stays on this device."
          : "The text is sent to the selected provider.";
    if (t.id === "cleanup-toggle") $("#cleanup-period").hidden = !t.checked;
  });
  document.addEventListener(
    "toggle",
    (event) => {
      if (event.target.matches?.(".original-disclosure"))
        s.originalOpen = event.target.open;
    },
    true,
  );
  document.addEventListener("keydown", (event) => {
    if (s.modal === "shortcut" && event.key !== "Escape") {
      if (["Meta", "Control", "Alt", "Shift", "Tab"].includes(event.key))
        return;
      if (event.metaKey || event.ctrlKey || event.altKey) {
        event.preventDefault();
        const key =
          event.key === " "
            ? "Space"
            : event.key.length === 1
              ? event.key.toUpperCase()
              : event.key;
        s.shortcutDraft = [
          event.metaKey ? "⌘" : "",
          event.ctrlKey ? "Ctrl" : "",
          event.altKey ? "Alt" : "",
          event.shiftKey ? "Shift" : "",
          key,
        ]
          .filter(Boolean)
          .join(" ");
        $("#shortcut-recorder").innerHTML =
          "<kbd>" + esc(s.shortcutDraft) + "</kbd>";
      }
    } else if (event.key === "Escape" && s.active && !$("#modal").open) {
      event.preventDefault();
      cancelSession();
    }
  });
  $("#modal").addEventListener("cancel", (event) => {
    event.preventDefault();
    closeModal();
  });
  $("#modal").addEventListener("close", () => {
    $("#modal")
      .querySelectorAll("input[type=password]")
      .forEach((x) => (x.value = ""));
  });
  render();
})();
