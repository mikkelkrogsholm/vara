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
            MetaLeft: "Venstre ⌘", MetaRight: "Højre ⌘",
            AltLeft: "Venstre ⌥", AltRight: "Højre ⌥",
            ControlLeft: "Venstre ⌃", ControlRight: "Højre ⌃",
            ShiftLeft: "Venstre ⇧", ShiftRight: "Højre ⇧",
            Space: "Space",
          })[key] || key.replace(/^(Key|Digit)/, ""),
      )
      .join(" ");
  const artwork = new Map();
  const appArtwork = (name, size = "", id = "") =>
    `<span class="app-icon ${e(size)}" data-app-id="${e(id)}" aria-hidden="true">${artwork.get(id) ? `<img src="${e(artwork.get(id))}" alt="" draggable="false">` : `<span class="monogram">${e(String(name).slice(0,1).toUpperCase())}</span>`}</span>`;
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
  const resultActions = (id) => `<div class="result-actions">${Vara.ui.iconButton("copy", "Kopiér diktering", "copy", `data-id="${e(id)}"`)}${Vara.ui.iconButton("trash", "Slet diktering", "confirm-delete", `data-id="${e(id)}"`)}</div>`;
  const shortcut = (value) =>
    `<div class="field shortcut-field"><span class="field-label">Genvej</span><input type="hidden" id="shortcut" value="${e(value)}"><div class="shortcut-control"><kbd id="shortcut-preview">${e(binding(value))}</kbd>${button("Skift", "record-shortcut", { attrs: 'aria-label="Skift genvej"' })}</div><span class="field-error" id="shortcut-error"></span></div>`;
  const settingsSection = (title, description, content, wide = false) =>
    `<section class="settings-section ${wide ? "is-wide" : ""}"><header><h2>${e(title)}</h2><p class="muted">${e(description)}</p></header><div class="settings-section-body">${content}</div></section>`;
  const policyRow = (glyph, title, description) =>
    `<div class="policy-row">${icon(glyph)}<div><p>${e(title)}</p>${description ? `<p class="muted">${e(description)}</p>` : ""}</div></div>`;
  const processingChoice = (enabled) =>
    `<fieldset class="processing-choice"><legend class="sr-only">Efter transskription</legend><label for="direct-text"><input id="direct-text" type="radio" name="text-processing" value="direct" ${enabled ? "" : "checked"}><span>Behold transskriptionen<small>Indsæt transskriptionen, som den er.</small></span></label><label for="general-ai"><input id="general-ai" type="radio" name="text-processing" value="ai" ${enabled ? "checked" : ""} aria-controls="text-processing-fields"><span>Bearbejd med AI<small>Bearbejd teksten, før den indsættes.</small></span></label></fieldset>`;
  const modelLabel = (engine) => engine.provider.startsWith("local_") ? engine.label : `${engine.label} · ${engine.model}`;
  const modelSelect = ({id, label, engines, selected, strict}) =>
    `<div class="field"><label for="${e(id)}">${e(label)}</label><select id="${e(id)}" aria-describedby="${e(id)}-error">${engines.map(engine=>`<option value="${e(engine.id)}" ${engine.id === selected ? "selected" : ""} ${strict && engine.locality !== "on_device" ? "disabled" : ""}>${e(modelLabel(engine))}${strict && engine.locality !== "on_device" ? " · Kræver tilladelse til ekstern behandling" : ""}</option>`).join("")}</select><span class="field-error" id="${e(id)}-error"></span></div>`;
  const permissionRow = ({
    title,
    description,
    kind,
    granted,
    available = true,
  }) =>
    `<div class="permission-row"><div><strong>${e(title)}</strong><p class="muted">${e(description)}</p></div>${granted ? `<span class="permission-granted">${icon("check")}Adgang givet</span>` : available ? button("Giv adgang", "permission", { attrs: `data-kind="${e(kind)}"` }) : '<span class="status-pill">Manuel indsættelse</span>'}</div>`;
  const disclosure = (id, title, description, content, open = false) =>
    `<details id="${e(id)}" class="settings-disclosure" ${open ? "open" : ""}><summary><span>${e(title)}</span>${icon("chevron")}</summary><p class="muted">${e(description)}</p><div>${content}</div></details>`;
  Vara.desktopUi = { binding, appArtwork, setAppArtwork, resultActions, shortcut, permissionRow, disclosure, settingsSection, policyRow, processingChoice, modelSelect, modelLabel };
})();
