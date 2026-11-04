const assert = require('node:assert/strict');
const i18n = require('../ui/i18n.js');
const {catalogs} = i18n;
const placeholders = value => [...value.matchAll(/\{([A-Za-z][\w]*)\}/g)].map(m => m[1]).sort().join(',');
const englishKeys = Object.keys(catalogs.en).sort();
for (const [locale, catalog] of Object.entries(catalogs)) {
  assert.deepEqual(Object.keys(catalog).sort(), englishKeys, `${locale} catalog keys`);
  for (const key of englishKeys) {
    const reference = catalogs.en[key], translated = catalog[key];
    assert.equal(typeof translated, typeof reference, `${locale}:${key} type`);
    if (typeof reference === 'string') assert.equal(placeholders(translated), placeholders(reference), `${locale}:${key} arguments`);
    else {
      assert.deepEqual(Object.keys(translated).sort(), new Intl.PluralRules(locale).resolvedOptions().pluralCategories.sort(), `${locale}:${key} plural categories`);
      for (const form of Object.keys(translated)) assert.equal(placeholders(translated[form]), placeholders(reference.other), `${locale}:${key}:${form} arguments`);
    }
  }
}
i18n.setLocale('da');
assert.equal(i18n.message('example.items', {count: 1}), '1 element');
assert.equal(i18n.message('example.items', {count: 2}), '2 elementer');
const label = catalogs.da['locale.heading']; delete catalogs.da['locale.heading'];
assert.equal(i18n.message('locale.heading'), catalogs.en['locale.heading']); catalogs.da['locale.heading'] = label;
assert.equal(i18n.number(1234.5), new Intl.NumberFormat('da').format(1234.5));
assert.equal(i18n.number(2, {style:'unit', unit:'minute'}), new Intl.NumberFormat('da',{style:'unit',unit:'minute'}).format(2));
assert.equal(i18n.date(new Date('2025-03-01T12:00:00Z'), {timeZone:'UTC'}), new Intl.DateTimeFormat('da',{timeZone:'UTC'}).format(new Date('2025-03-01T12:00:00Z')));
assert.equal(i18n.message('example.items', {count:'<img src=x onerror="x">'}), '<img src=x onerror="x"> elementer');
assert.equal(i18n.html('example.items', {count:'<img src=x onerror="x">'}), '&lt;img src=x onerror=&quot;x&quot;&gt; elementer');
i18n.setLocale('unknown');
assert.equal(i18n.getLocale(), 'en');
assert.equal(i18n.message('example.items', {count:2}), '2 items');
let changes = 0; const unsubscribe = i18n.subscribe(() => changes++);
i18n.setLocale('da'); i18n.setLocale('da'); unsubscribe(); i18n.setLocale('en');
assert.equal(changes, 1);
// Every literal key consumed by the owned main UI must exist in both catalogs.
const fs = require('node:fs'), path = require('node:path');
for (const file of ['app.js', 'components.js', 'views.js', 'transcription.js']) {
  const source = fs.readFileSync(path.join(__dirname, '../ui', file), 'utf8');
  for (const match of source.matchAll(/(?:Vara\.i18n\.bind|\btr)\(\s*["']([^"']+)["']/g)) {
    for (const [locale, catalog] of Object.entries(catalogs))
      assert.ok(Object.hasOwn(catalog, match[1]), `${file}: ${locale} missing ${match[1]}`);
  }
}
// The browser metadata must match the host's production language boundary.
const metadata = JSON.parse(fs.readFileSync(path.join(__dirname, '../../crates/vara-core/messages/locales.json')));
assert.deepEqual(Object.fromEntries(metadata.map(({id,name}) => [id,name])), i18n.localeNames);
assert.deepEqual(Object.keys(i18n.localeNames).sort(), Object.keys(catalogs).sort());
// A test-only catalog goes through the same normalization, formatting and binding code.
catalogs.fr = {...catalogs.en, 'example.items': {one:'{count} élément de test', other:'{count} éléments de test'}};
i18n.localeNames.fr = 'Français (test)';
i18n.setLocale('FR-ca');
assert.equal(i18n.getLocale(), 'fr');
assert.equal(i18n.message('example.items', {count:2}), '2 éléments de test');
assert.equal(i18n.resolve(i18n.bind('example.items', {count:2})), '2 éléments de test');
assert.equal(i18n.number(1234.5), new Intl.NumberFormat('fr').format(1234.5));
assert.equal(i18n.date(new Date('2025-03-01T12:00:00Z'), {timeZone:'UTC'}), new Intl.DateTimeFormat('fr',{timeZone:'UTC'}).format(new Date('2025-03-01T12:00:00Z')));
delete catalogs.fr; delete i18n.localeNames.fr;
console.log('Catalog parity, placeholders, escaping, plural, Intl formatting and locale hooks passed.');
