/* Keep a failed startup actionable, including errors before the IPC controller loads. */
(() => {
  function failed(message) {
    const startup = document.getElementById('startup');
    if (!startup) return;
    startup.replaceChildren();
    const title = document.createElement('h1');
    const i18n = window.Vara?.i18n;
    const danish = i18n ? i18n.getLocale() === 'da' : (navigator.languages || [navigator.language]).some(l => /^da(?:-|$)/i.test(l));
    title.textContent = i18n ? i18n.bind('startup.failed') : (danish ? 'Vara kunne ikke åbne' : 'Vara could not open');
    const detail = document.createElement('p');
    // Before the runtime catalog is available, do not disclose arbitrary error
    // text: it may contain paths, user content or credentials.
    detail.textContent = i18n ? i18n.bindRuntime(message) : (danish ? 'Der opstod en ukendt fejl.' : 'An unknown error occurred.');
    const retry = document.createElement('button');
    retry.className = 'button button-primary';
    retry.textContent = i18n ? i18n.bind('startup.retry') : (danish ? 'Prøv igen' : 'Try again');
    retry.addEventListener('click', () => location.reload());
    startup.append(title, detail, retry);
  }
  window.addEventListener('error', event => failed(event.message || ''));
  window.addEventListener('unhandledrejection', event => failed(String(event.reason?.message || event.reason)));
})();
