/* Pure reusable components. No application state or event listeners here. */
(() => {
  "use strict";
  const V = (window.Vara = {});
  const esc = (value) =>
    String(value ?? "").replace(
      /[&<>"']/g,
      (c) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[c],
    );
  const paths = {
    document: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6M8 13h8M8 17h6"/>',
    plus: '<path d="M12 4v16M4 12h16"/>',
    edit: '<path d="m16 3 5 5-12 12-6 1 1-6L16 3Z"/>',
    history: '<circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/>',
    apps: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
    settings:
      '<path d="m9 3-1 3-3 1-2 3 2 2-1 3 3 2 3-1 2 2 3-1 1-3 3-1 1-3-2-2 1-3-3-2-3 1-2-1Z"/><circle cx="12" cy="11" r="3"/>',
    search: '<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/>',
    copy: '<rect x="8" y="7" width="12" height="14" rx="2"/><path d="M15 7V3H4v14h4"/>',
    chevron: '<path d="m6 9 6 6 6-6"/>',
    more: '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
    close: '<path d="m6 6 12 12M6 18 18 6"/>',
    back: '<path d="m14 5-7 7 7 7"/>',
    check: '<path d="m5 12 4 4L19 6"/>',
    warning: '<path d="m12 3 10 18H2L12 3Z"/><path d="M12 9v5m0 3v.1"/>',
    info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10v.1"/>',
    trash: '<path d="M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7"/>',
    stop: '<rect x="6" y="6" width="12" height="12" rx="2"/>',
    mic: '<rect x="9" y="2" width="6" height="13" rx="3"/><path d="M5 10v2a7 7 0 0 0 14 0v-2M12 19v3m-4 0h8"/>',
    download: '<path d="M12 3v12m-5-5 5 5 5-5M4 17v4h16v-4"/>',
    key: '<circle cx="8" cy="9" r="5"/><path d="m12 13 8 8m-5-5 3-3m0 6 3-3"/>',
    globe:
      '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c6 5 6 13 0 18-6-5-6-13 0-18Z"/>',
    shield:
      '<path d="m12 2 8 4v6c0 5-8 10-8 10S4 17 4 12V6l8-4Z"/><path d="m8 12 3 3 5-6"/>',
  };
  const icon = (name, cls = "") =>
    `<svg class="icon ${esc(cls)}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths[name] || paths.info}</svg>`;
  const wave = (cls = "") =>
    `<span class="wave ${esc(cls)}" aria-hidden="true">${[5, 10, 18, 27, 36, 44, 32, 48, 39, 29, 20, 12, 19, 11, 7, 5].map((h) => `<i style="--level:${h}px"></i>`).join("")}</span>`;
  const mark = () =>
    '<svg class="brand-mark" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" aria-hidden="true"><path d="M3 15v2m6-8v14m7-21v28m7-21v14m6-8v2"/></svg>';
  const appIcon = (app, size = "") => {
    const artwork = {
      mail: '<img src="assets/mail.png" alt="" draggable="false">',
      notes: '<img src="assets/notes.png" alt="" draggable="false">',
      code: '<svg viewBox="0 0 40 40" aria-hidden="true"><path fill="#168BD2" d="m29 2 9 4v28l-9 4L13 25 6 31l-5-3 11-9L1 10l5-3 7 6L29 2Zm0 10L18 20l11 8V12Z"/><path fill="#0876B9" d="m29 2 9 4v28l-9 4V2Z"/></svg>',
      browser: icon("globe"),
    };
    return `<span class="app-icon ${esc(size)}" aria-hidden="true">${artwork[app] || '<span class="monogram">' + esc(String(app).slice(0, 1).toUpperCase()) + "</span>"}</span>`;
  };
  const button = (
    label,
    action,
    { variant = "secondary", glyph = "", attrs = "", cls = "" } = {},
  ) =>
    `<button type="button" class="button button--${variant} ${esc(cls)}" data-action="${esc(action)}" ${attrs}>${glyph ? icon(glyph) : ""}<span>${esc(label)}</span></button>`;
  const iconButton = (name, label, action, attrs = "") =>
    `<button type="button" class="icon-button" aria-label="${esc(label)}" title="${esc(label)}" data-action="${esc(action)}" ${attrs}>${icon(name)}</button>`;
  const notice = (text, type = "info", extra = "") =>
    `<div class="notice notice--${esc(type)}">${icon(type === "success" ? "check" : type === "error" || type === "warning" ? "warning" : "info")}<div>${esc(text)}${extra}</div></div>`;
  const field = ({
    id,
    label,
    value = "",
    type = "text",
    help = "",
    required = false,
    options = null,
    rows = false,
    attrs = "",
  }) => {
    attrs += ` aria-describedby="${esc(id)}-error${help ? " " + esc(id) + "-help" : ""}"`;
    const content = options
      ? `<select id="${esc(id)}" name="${esc(id)}" ${attrs}>${options.map(([v, l]) => `<option value="${esc(v)}" ${v === value ? "selected" : ""}>${esc(l)}</option>`).join("")}</select>`
      : rows
        ? `<textarea id="${esc(id)}" name="${esc(id)}" ${attrs}>${esc(value)}</textarea>`
        : `<input id="${esc(id)}" name="${esc(id)}" type="${esc(type)}" value="${esc(value)}" ${required ? "required" : ""} ${attrs}>`;
    return `<div class="field"><label for="${esc(id)}">${esc(label)}</label>${content}${help ? `<small id="${esc(id)}-help">${esc(help)}</small>` : ""}<span class="field-error" id="${esc(id)}-error"></span></div>`;
  };
  const toggle = (id, label, checked, help = "") =>
    `<div class="toggle-row"><div><label for="${esc(id)}">${esc(label)}</label>${help ? `<p class="muted">${esc(help)}</p>` : ""}</div><input class="switch" role="switch" type="checkbox" id="${esc(id)}" ${checked ? "checked" : ""}></div>`;
  const section = (title, description, content) =>
    `<section class="form-section"><header><h2>${esc(title)}</h2>${description ? `<p class="muted">${esc(description)}</p>` : ""}</header>${content}</section>`;
  const empty = (title, text, action = "") =>
    `<div class="empty-state"><h2>${esc(title)}</h2><p>${esc(text)}</p>${action}</div>`;
  V.ui = {
    esc,
    icon,
    wave,
    mark,
    appIcon,
    button,
    iconButton,
    notice,
    field,
    toggle,
    section,
    empty,
  };
})();
