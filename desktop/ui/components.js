/* Live desktop compositions built on the canonical DESIGN.md primitives. No domain state. */
(() => {
  const { esc: e, icon, button } = Vara.ui;
  const mac = () => document.documentElement.dataset.platform === "macos";
  const binding = (value) =>
    String(value).replace(/^Physical:/, "")
      .split("+")
      .map(
        (key) =>
          ({
            Alt: mac() ? "⌥" : "Alt",
            Option: mac() ? "⌥" : "Alt",
            Ctrl: mac() ? "⌃" : "Ctrl",
            Control: mac() ? "⌃" : "Ctrl",
            Super: mac() ? "⌘" : "Win",
            Meta: mac() ? "⌘" : "Win",
            Command: "⌘",
            Cmd: "⌘",
            Shift: mac() ? "⇧" : "Shift",
            MetaLeft: Vara.i18n.bind("ui.left"), MetaRight: Vara.i18n.bind("ui.right"),
            AltLeft: Vara.i18n.bind("ui.left-2"), AltRight: Vara.i18n.bind("ui.right-2"),
            ControlLeft: Vara.i18n.bind("ui.left-3"), ControlRight: Vara.i18n.bind("ui.right-3"),
            ShiftLeft: Vara.i18n.bind("ui.left-4"), ShiftRight: Vara.i18n.bind("ui.right-4"),
            Space: "Space",
          })[key] || key.replace(/^(Key|Digit)/, ""),
      )
      .join(" ");
  const artwork = new Map();
  const appArtwork = (name, size = "", id = "") =>
    `<span class="app-icon ${e(size)}" data-app-id="${e(id)}" aria-hidden="true">${artwork.get(id) ? `<img src="${e(artwork.get(id))}" alt="" draggable="false">` : `<span class="monogram">${e(id === "unknown" ? "?" : String(name).slice(0,1).toUpperCase())}</span>`}</span>`;
  const setAppArtwork = (images) => {
    for (const [id, src] of Object.entries(images || {})) {
      if (typeof src === "string" && /^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(src)) artwork.set(id, src);
    }
    document.querySelectorAll(".app-icon[data-app-id]").forEach(node => {
      const src = artwork.get(node.dataset.appId);
      if (!src) return;
      const image = document.createElement("img");
      image.src = src; image.alt = ""; image.draggable = false;
      node.replaceChildren(image);
    });
  };
  const resultActions = (id) => `<div class="result-actions">${Vara.ui.iconButton("copy", Vara.i18n.bind("ui.copy-dictation"), "copy", `data-id="${e(id)}"`)}${Vara.ui.iconButton("trash", Vara.i18n.bind("ui.delete-dictation"), "confirm-delete", `data-id="${e(id)}"`)}</div>`;
  const shortcut = (value) =>
    `<div class="field shortcut-field"><span class="field-label">${Vara.i18n.bind("ui.shortcut")}</span><input type="hidden" id="shortcut" value="${e(value)}"><div class="shortcut-control"><kbd id="shortcut-preview">${e(binding(value))}</kbd>${button(Vara.i18n.bind("ui.change"), "record-shortcut", { attrs: `aria-label="${Vara.i18n.bind("shortcut.change")}"` })}</div><span class="field-error" id="shortcut-error"></span></div>`;
  const settingsSection = (title, description, content, wide = false) =>
    `<section class="settings-section ${wide ? "is-wide" : ""}"><header><h2>${e(title)}</h2><p class="muted">${e(description)}</p></header><div class="settings-section-body">${content}</div></section>`;
  const policyRow = (glyph, title, description) =>
    `<div class="policy-row">${icon(glyph)}<div><p>${e(title)}</p>${description ? `<p class="muted">${e(description)}</p>` : ""}</div></div>`;
  const processingChoice = (enabled) =>
    `<fieldset class="processing-choice"><legend class="sr-only">${Vara.i18n.bind("ui.after-transcription")}</legend><label for="direct-text"><input id="direct-text" type="radio" name="text-processing" value="direct" ${enabled ? "" : "checked"}><span>${Vara.i18n.bind("ui.keep-transcription")}<small>${Vara.i18n.bind("ui.insert-the-transcription-as-it-is")}</small></span></label><label for="general-ai"><input id="general-ai" type="radio" name="text-processing" value="ai" ${enabled ? "checked" : ""} aria-controls="text-processing-fields"><span>${Vara.i18n.bind("ui.process-with-ai")}<small>${Vara.i18n.bind("ui.process-the-text-before-inserting-it")}</small></span></label></fieldset>`;
  // The id and unedited default label must both match: custom names stay intact.
  const builtins = {
    "local.parakeet": [["Parakeet Redux · on device", "Parakeet Redux · på enheden"], "engine.parakeet"],
    "local.whisper": [["Whisper · on device", "Whisper · på enheden"], "engine.whisper"],
    "local.text": [["Qwen · on device", "Qwen · på enheden"], "engine.qwen"],
    "openai.speech": [["OpenAI · speech", "OpenAI · tale"], "engine.openai.speech"],
    "elevenlabs.speech": [["ElevenLabs · speech", "ElevenLabs · tale"], "engine.elevenlabs.speech"],
    "openai.text": [["OpenAI · text", "OpenAI · tekst"], "engine.openai.text"],
    "anthropic.text": [["Anthropic · text", "Anthropic · tekst"], "engine.anthropic.text"],
    "gemini.speech": [["Gemini · speech", "Gemini · tale"], "engine.gemini.speech"],
    "gemini.text": [["Gemini · text", "Gemini · tekst"], "engine.gemini.text"],
    "claude.cli": [["Claude Code · own login", "Claude Code · eget login"], "engine.claude.cli"],
    "codex.cli": [["Codex · own login", "Codex · eget login"], "engine.codex.cli"],
    "gemini.cli": [["Gemini CLI · own login", "Gemini CLI · eget login"], "engine.gemini.cli"],
    "ollama.text": [["Ollama · local service", "Ollama · lokal tjeneste"], "engine.ollama"],
  };
  const displayEngineName = engine => {
    const entry = builtins[engine.id];
    return entry && entry[0].includes(engine.label) ? Vara.i18n.bind(entry[1]) : engine.label;
  };
  const modelLabel = engine => engine.provider.startsWith("local_") ? displayEngineName(engine) : `${displayEngineName(engine)} · ${engine.model}`;
  const modelSelect = ({id, label, engines, selected, strict}) =>
    `<div class="field"><label for="${e(id)}">${e(label)}</label><select id="${e(id)}" aria-describedby="${e(id)}-error">${engines.map(engine=>`<option value="${e(engine.id)}" ${engine.id === selected ? "selected" : ""} ${strict && engine.locality !== "on_device" ? "disabled" : ""}>${e(modelLabel(engine))}${strict && engine.locality !== "on_device" ? Vara.i18n.bind("shortcut.external.restriction") : ""}</option>`).join("")}</select><span class="field-error" id="${e(id)}-error"></span></div>`;
  const permissionRow = ({
    title,
    description,
    kind,
    granted,
    available = true,
  }) =>
    `<div class="permission-row"><div><strong>${e(title)}</strong><p class="muted">${e(description)}</p></div>${granted ? `<span class="permission-granted">${icon("check")}${Vara.i18n.bind("permission.granted")}</span>` : available ? button(Vara.i18n.bind("ui.grant-access"), "permission", { attrs: `data-kind="${e(kind)}"` }) : ("<span class=\"status-pill\">"+Vara.i18n.bind("ui.manual-insertion")+"</span>")}</div>`;
  // Delivery is authoritative. Diagnostic metadata can explain a refusal, never
  // downgrade an uncertain handoff into permission to insert a second time.
  const insertionReasons = new Set([
    "accessibility_missing", "capture_failed", "target_gone", "field_unavailable",
    "app_changed", "focus_changed", "unsupported_role", "post_event_access_missing",
    "dispatch_failed", "handoff_unconfirmed", "confirmed", "not_attempted", "platform_unsupported",
  ]);
  const insertionStatus = (record) => {
    const delivery = ["inserted", "manual_ready", "unknown", "pending"].includes(record.delivery)
      ? record.delivery : "manual_ready";
    const diagnostic = record.insertion_diagnostic;
    const reason = insertionReasons.has(diagnostic?.reason) ? diagnostic.reason : null;
    return { delivery, reason,
      method: ["accessibility", "keyboard"].includes(diagnostic?.method) ? diagnostic.method : null,
      osError: Number.isInteger(diagnostic?.os_error) && diagnostic.os_error >= -2147483648 && diagnostic.os_error <= 2147483647 ? diagnostic.os_error : null,
      permission: reason === "accessibility_missing" ? "accessibility" : reason === "post_event_access_missing" ? "post_event" : null,
      row: delivery === "inserted" ? "" : Vara.i18n.bind(`insertion.row.${delivery}`),
    };
  };
  const insertionNotice = (record) => {
    const status = insertionStatus(record);
    if (status.delivery === "inserted") return "";
    const manualReason = status.reason && !["confirmed", "handoff_unconfirmed"].includes(status.reason) ? status.reason : "generic";
    const message = status.delivery === "manual_ready" ? Vara.i18n.bind(`insertion.reason.${manualReason}`) : Vara.i18n.bind(`insertion.${status.delivery}`);
    const help = status.permission ? button(Vara.i18n.bind("insertion.permission-help"), "permission", { attrs: `data-kind="${status.permission}"` }) : "";
    // Never expose arbitrary fields, target identifiers, text or native messages.
    const details = [status.reason, status.method, status.osError === null ? null : `AX ${status.osError}`].filter(value => value !== null);
    return `<div class="insertion-feedback">${Vara.ui.notice(message, status.delivery === "unknown" ? "warning" : "info", help)}${details.length ? `<details class="settings-disclosure insertion-details" id="insertion-details-${e(record.id)}"><summary><span>${Vara.i18n.bind("insertion.details")}</span>${icon("chevron")}</summary><p class="muted">${details.map(e).join(" · ")}</p></details>` : ""}</div>`;
  };
  const insertionPermission = (permissions) => {
    const supported = permissions.automatic_insertion === true;
    const accessibility = permissions.accessibility === true;
    const keyboardBlocked = supported && accessibility && permissions.post_event_access === false;
    const description = !supported ? "ui.on-this-platform-the-text-is-ready-to-insert-manually"
      : !accessibility ? "insertion.access.required"
      : keyboardBlocked ? "insertion.access.partial"
      : permissions.post_event_access === true ? "insertion.access.granted" : "insertion.access.legacy";
    return permissionRow({ title: Vara.i18n.bind("ui.insertion-in-other-apps"),
      description: Vara.i18n.bind(description), kind: keyboardBlocked ? "post_event" : "accessibility",
      granted: supported && accessibility && !keyboardBlocked, available: supported });
  };
  const disclosure = (id, title, description, content, open = false) =>
    `<details id="${e(id)}" class="settings-disclosure" ${open ? "open" : ""}><summary><span>${e(title)}</span>${icon("chevron")}</summary><p class="muted">${e(description)}</p><div>${content}</div></details>`;
  Vara.desktopUi = { binding, appArtwork, setAppArtwork, resultActions, shortcut, permissionRow, insertionStatus, insertionNotice, insertionPermission, disclosure, settingsSection, policyRow, processingChoice, modelSelect, modelLabel, displayEngineName };
})();
