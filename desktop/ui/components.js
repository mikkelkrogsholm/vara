/* Live desktop compositions built on the canonical DESIGN.md primitives. No domain state. */
(() => {
  const { esc: e, icon, appIcon, button } = Vara.ui;
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
  const appArtwork = (name, size = "") =>
    appIcon(
      /mail/i.test(name)
        ? "mail"
        : /notes|noter/i.test(name)
          ? "notes"
          : /studio code|^code$/i.test(name)
            ? "code"
            : /safari|chrome|firefox|edge/i.test(name)
              ? "browser"
              : name,
      size,
    );
  const shortcut = (value) =>
    `<div class="shortcut-setting"><div><span class="field-label">Din genvej</span><p class="muted">Du vælger selv tasterne.</p></div><input type="hidden" id="shortcut" value="${e(value)}"><kbd id="shortcut-preview">${e(binding(value))}</kbd>${button("Skift genvej", "record-shortcut")}</div><span class="field-error" id="shortcut-error"></span>`;
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
  Vara.desktopUi = { binding, appArtwork, shortcut, permissionRow, disclosure };
})();
