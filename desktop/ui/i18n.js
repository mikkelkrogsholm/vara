/* message returns plain text; html escapes the entire message for HTML templates. */
((root) => {
  const catalogs = {
    en: {
      "locale.heading": "Interface language",
      "locale.description": "Choose the language Vara uses for its interface.",
      "locale.label": "Interface language",
      "locale.system": "System language",
      "locale.da": "Dansk",
      "locale.en": "English",
      "locale.error": "The language could not be saved.",
      "example.items": { one: "{count} item", other: "{count} items" },
    },
    da: {
      "locale.heading": "Grænsefladens sprog",
      "locale.description": "Vælg det sprog, Vara bruger i grænsefladen.",
      "locale.label": "Grænsefladens sprog",
      "locale.system": "Systemets sprog",
      "locale.da": "Dansk",
      "locale.en": "English",
      "locale.error": "Sproget kunne ikke gemmes.",
      "example.items": { one: "{count} element", other: "{count} elementer" },
    },
  };
  let locale = "en";
  const listeners = new Set();
  const escape = value => String(value).replace(/[&<>"']/g, c => ({"&":"&amp;", "<":"&lt;", ">":"&gt;", '"':"&quot;", "'":"&#39;"})[c]);
  const normalize = value => value === "da" ? "da" : "en";
  function message(key, args = {}) {
    let value = catalogs[locale][key] ?? catalogs.en[key];
    if (value == null) return String(key);
    if (typeof value === "object") value = value[new Intl.PluralRules(locale).select(Number(args.count))] ?? value.other;
    return value.replace(/\{([A-Za-z][\w]*)\}/g, (_, name) => String(args[name] ?? ""));
  }
  // Annotate only leaf text nodes or attributes; never bind an editable container.
  // This can refresh open dialogs and retained editors without replacing them.
  function refreshBindings(scope = root.document) {
    if (!scope) return;
    scope.querySelectorAll("[data-i18n]").forEach(node => {
      node.textContent = message(node.dataset.i18n);
    });
    scope.querySelectorAll("[data-i18n-aria-label]").forEach(node => {
      node.setAttribute("aria-label", message(node.dataset.i18nAriaLabel));
    });
    scope.querySelectorAll("[data-i18n-placeholder]").forEach(node => {
      node.setAttribute("placeholder", message(node.dataset.i18nPlaceholder));
    });
  }
  function setLocale(value) {
    const next = normalize(value);
    if (root.document) root.document.documentElement.lang = next;
    if (next === locale) return;
    locale = next;
    refreshBindings();
    for (const listener of listeners) listener(locale);
  }
  const i18n = {
    catalogs, message, refreshBindings, html: (key, args) => escape(message(key, args)), setLocale, getLocale: () => locale,
    subscribe(listener) { listeners.add(listener); return () => listeners.delete(listener); },
    number(value, options) { return new Intl.NumberFormat(locale, options).format(value); },
    date(value, options) { return new Intl.DateTimeFormat(locale, options).format(value); },
  };
  if (typeof module !== "undefined" && module.exports) module.exports = i18n;
  else { root.Vara = root.Vara || {}; root.Vara.i18n = i18n; }
})(typeof window === "undefined" ? globalThis : window);
