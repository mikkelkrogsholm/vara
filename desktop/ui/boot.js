/* Keep a failed startup actionable, including errors before the IPC controller loads. */
(() => {
  function failed(message) {
    const startup = document.getElementById('startup');
    if (!startup) return;
    startup.replaceChildren();
    const title = document.createElement('h1');
    title.textContent = 'Vara kunne ikke åbne';
    const detail = document.createElement('p');
    detail.textContent = message;
    const retry = document.createElement('button');
    retry.className = 'button button-primary';
    retry.textContent = 'Prøv igen';
    retry.addEventListener('click', () => location.reload());
    startup.append(title, detail, retry);
  }
  window.addEventListener('error', event => failed(event.message || 'En del af brugerfladen kunne ikke indlæses.'));
  window.addEventListener('unhandledrejection', event => failed(String(event.reason?.message || event.reason)));
})();
