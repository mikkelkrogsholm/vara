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
  const disclosure = (id, title, description, content, open = false) =>
    `<details id="${e(id)}" class="settings-disclosure" ${open ? "open" : ""}><summary><span>${e(title)}</span>${icon("chevron")}</summary><p class="muted">${e(description)}</p><div>${content}</div></details>`;
  Vara.desktopUi = { binding, appArtwork, setAppArtwork, resultActions, shortcut, permissionRow, disclosure, settingsSection, policyRow, processingChoice, modelSelect, modelLabel, displayEngineName };
})();
