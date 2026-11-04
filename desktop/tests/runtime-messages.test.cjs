// Tests the same catalog embedded by Rust and delivered over IPC, without a second catalog.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'../..'), i18n=require('../ui/i18n.js');
const catalog=JSON.parse(fs.readFileSync(path.join(root,'crates/vara-core/messages/runtime.json')));
i18n.setRuntimeCatalog(catalog);
const placeholders=s=>[...s.matchAll(/\{(\w+)\}/g)].map(x=>x[1]).sort();
for(const [code,entry] of Object.entries(catalog)){for(const locale of Object.keys(i18n.localeNames)){assert.ok(entry[locale],`${code}: missing ${locale}`);assert.deepEqual(placeholders(entry[locale]),placeholders(entry.en),`${code}: ${locale} placeholders`);}}
for(const file of ['desktop/src-tauri/src/lib.rs','desktop/src-tauri/src/service.rs','desktop/src-tauri/src/transcription.rs','crates/vara-app/src/lib.rs']){
 const source=fs.readFileSync(path.join(root,file),'utf8');
 for(const m of source.matchAll(/(?:host_message|host_wrapped|host_io|host_json|UserMessage::new)\(\s*"([^"]+)"/g)) assert.ok(catalog[m[1]],`${file}: ${m[1]}`);
}
for(const locality of ['local','lan','cloud','unknown']){
 assert.ok(catalog[`hud.audio.${locality}`]);for(const prompt of ['app','general'])assert.ok(catalog[`hud.ai.${locality}.${prompt}`]);
}
const wire='vara-message-v1:'+JSON.stringify({code:'provider.http',params:{status:503},cause:{code:'provider.network'}});
i18n.catalogs.fr={...i18n.catalogs.en}; i18n.localeNames.fr='Français (test)';
i18n.setLocale('fr');
assert.match(i18n.runtimeText(wire),/HTTP 503.*Connection failed/); // missing runtime translation falls back to English
const frCatalog=structuredClone(catalog); frCatalog['provider.http'].fr='Échec HTTP {status}';
i18n.setRuntimeCatalog(frCatalog);
assert.match(i18n.runtimeText(wire),/Échec HTTP 503/);
i18n.setRuntimeCatalog(catalog); delete i18n.catalogs.fr; delete i18n.localeNames.fr;
i18n.setLocale('en');assert.match(i18n.runtimeText(wire),/HTTP 503.*Connection failed/);const token=i18n.bindRuntime(wire);assert.match(i18n.resolve(token),/HTTP 503/);assert.equal(i18n.bindRuntime(token),token);
i18n.setLocale('da');assert.match(i18n.resolve(token),/HTTP 503.*Forbindelsen mislykkedes/);
assert.match(i18n.runtimeText('Audio backend unavailable'),/Audio backend unavailable/);
assert.match(i18n.runtimeText('HTTP 429 token=private'),/HTTP 429/);
for(const raw of ['Audio backend unavailable token=secret /private/transcript','vara-message-v2:{"key":"secret"}','vara-message-v1:{bad','vara-message-v1:{"code":"provider.http","params":{"status":"secret"}}']){
 const result=i18n.runtimeText(raw);assert.doesNotMatch(result,/secret|private|vara-message/);
}
assert.equal(i18n.runtimeCode('vara-message-v1:{"code":"runtime.cancelled"}'),'runtime.cancelled');
assert.equal(i18n.runtimeCode('Annulleret'),null);
console.log(`PASS: ${Object.keys(catalog).length} runtime codes, host origins, dynamic HUD keys, both languages, safe causes and unknown redaction`);
