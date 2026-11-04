// Existing Playwright and browser only; no app data, credentials, or network calls.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const assert = require('node:assert/strict');
const desktop = path.resolve(__dirname, '..');
const fixture = JSON.parse(fs.readFileSync(path.join(__dirname,'fixtures/ui-state.json')));
(async () => {
  const server = http.createServer((req,res) => {
    const relative = decodeURIComponent(new URL(req.url,'http://localhost').pathname);
    const file = path.resolve(desktop, '.' + relative);
    if (!file.startsWith(desktop + path.sep)) { res.writeHead(403).end(); return; }
    fs.readFile(file,(err,data) => {
      if (err) { res.writeHead(404).end(); return; }
      const type = {'.js':'text/javascript','.css':'text/css','.html':'text/html','.json':'application/json'}[path.extname(file)] || 'application/octet-stream';
      res.writeHead(200,{'Content-Type':type}).end(data);
    });
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await chromium.launch({headless:true, ...(process.env.PLAYWRIGHT_CHROMIUM ? {executablePath:process.env.PLAYWRIGHT_CHROMIUM} : {})});
    const page = await browser.newPage();
    const errors = []; page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(data => {
      window.testState = structuredClone(data); window.testCalls = [];
      window.testState.settings.ui_locale = 'da'; window.testState.resolved_ui_locale = 'da'; window.testState.ui_locale_revision = 0;
      window.testDocument = {id:'file-1',title:'Interview',source_name:'audio.m4a',created_at:1792953000,engine:data.settings.engines.find(e=>e.id==='local.parakeet'),language:'da',stage:'ready',error:null,duration:4,completed_seconds:4,revision:0,edited_text:null,ai_text:null,ai_engine:null,speakers:{},transcript:{text:'Original text',timing:'none',segments:[]}};
      window.__TAURI__ = {core:{invoke:async (_, {action,payload}) => {
        window.testCalls.push({action,payload});
        const d = window.testState;
        if (action === 'snapshot') return structuredClone(d);
        if (action === 'devices') return [];
        if (action === 'file_list') return {documents:[structuredClone(window.testDocument)],media:{}};
        if (action === 'application_icons') return {};
        if (action === 'set_ui_locale') { if(window.waitLocale) await new Promise(resolve=>window.releaseLocale=resolve); if(window.failLocale) throw Error('fixture failure'); d.settings.ui_locale=payload.preference; d.resolved_ui_locale=payload.preference === 'system' ? 'en' : payload.preference; return {locale:d.resolved_ui_locale,revision:++d.ui_locale_revision}; }
        if (action === 'save_settings') { const locale=d.settings.ui_locale; d.settings=structuredClone(payload); d.settings.ui_locale=locale; }
        return null;
      }},event:{listen:async(name,cb)=>{if(name==='vara:state') {window.emitState=()=>cb({payload:structuredClone(window.testState)}); window.emitPayload=p=>cb({payload:structuredClone(p)});} return ()=>{};}}};
    },fixture);
    await page.goto(`http://127.0.0.1:${server.address().port}/ui/index.html`);
    await page.locator('[data-page="settings"]').click();
    await page.locator('#general-ai').check();
    const draft = page.locator('#general-prompt'); await draft.fill('Unsaved <prompt> & text');
    await draft.evaluate(el=>{el.focus();el.setSelectionRange(4,12)});
    await page.evaluate(()=>{ testState.downloads=[{id:'local.whisper',received:2,total:10,phase:'downloading'}]; testState.session={stage:'recording',origin:{app_name:'Test editor'}}; emitState(); });
    const before = await page.evaluate(()=>({settings:structuredClone(testState.settings),calls:testCalls.length,session:structuredClone(testState.session),downloads:structuredClone(testState.downloads)}));
    // Programmatically dispatch to retain focus and selection as a state-change event can do.
    await page.locator('#ui-locale').evaluate(el=>{el.value='en';el.dispatchEvent(new Event('change',{bubbles:true}));});
    await page.waitForFunction(()=>document.documentElement.lang==='en');
    assert.equal(await draft.inputValue(),'Unsaved <prompt> & text');
    assert.deepEqual(await draft.evaluate(el=>[document.activeElement===el,el.selectionStart,el.selectionEnd]),[true,4,12]);
    assert.equal(await page.locator('#ui-locale option[value=da]').textContent(),'Dansk');
    assert.equal(await page.locator('#ui-locale option[value=en]').textContent(),'English');
    assert.equal(await page.locator('.unsaved').isVisible(),true);
    const after = await page.evaluate(()=>({settings:structuredClone(testState.settings),calls:testCalls.slice()}));
    assert.deepEqual({...after.settings,ui_locale:'da'},before.settings);
    assert.deepEqual(await page.evaluate(()=>[testState.session,testState.downloads]),[before.session,before.downloads]);
    assert.deepEqual(after.calls.slice(before.calls).map(c=>c.action),['set_ui_locale']);
    // An older snapshot must not undo a successful locale save.
    await page.evaluate(()=>{const stale=structuredClone(testState);stale.settings.ui_locale='da';stale.resolved_ui_locale='da';stale.ui_locale_revision=0; emitPayload(stale);});
    assert.equal(await page.evaluate(()=>document.documentElement.lang),'en');
    // A snapshot arriving while a save fails must roll back both control and state.
    await page.evaluate(()=>{window.failLocale=true;window.waitLocale=true;});
    await page.locator('#ui-locale').selectOption('da');
    await page.waitForFunction(()=>typeof window.releaseLocale==='function');
    await page.evaluate(()=>{emitState(); const picker=document.querySelector("#ui-locale"); picker.value="system"; picker.dispatchEvent(new Event("change",{bubbles:true})); releaseLocale();});
    await page.waitForFunction(()=>!document.querySelector('#ui-locale').disabled);
    assert.equal(await page.locator('#ui-locale').inputValue(),'en');
    assert.equal(await draft.inputValue(),'Unsaved <prompt> & text');
    assert.equal(await page.locator('.form-message').textContent(),'The language could not be saved.');
    assert.equal(await page.evaluate(()=>testState.settings.ui_locale),'en');
    assert.equal(await page.evaluate(()=>testCalls.filter(c=>c.action==='set_ui_locale').length),2);
    await page.evaluate(()=>{window.failLocale=false;window.waitLocale=false;});
    await page.locator('[data-action="save-settings"]').click();
    assert.equal(await page.evaluate(()=>testState.settings.ui_locale),'en');
    assert.equal(await page.evaluate(()=>testState.settings.general_prompt),'Unsaved <prompt> & text');
    await page.locator('#ui-locale').selectOption('system');
    await page.waitForFunction(()=>!document.querySelector('#ui-locale').disabled);
    assert.equal(await page.evaluate(()=>testState.settings.ui_locale),'system');
    assert.equal(await page.evaluate(()=>document.documentElement.lang),'en');
    // Use the app's actual shortcut dialog while a locale event is delivered.
    await page.locator('[data-action="record-shortcut"]').click();
    const dialogIdentity = await page.evaluate(()=>{window.actualDialog=document.querySelector('#modal'); return actualDialog.open;});
    assert.equal(dialogIdentity,true);
    const modalCalls = await page.evaluate(()=>testCalls.length);
    await page.locator('#ui-locale').evaluate(el=>{el.value='da';el.dispatchEvent(new Event('change',{bubbles:true}));});
    await page.waitForFunction(()=>document.documentElement.lang==='da');
    assert.equal(await page.evaluate(()=>actualDialog===document.querySelector('#modal') && actualDialog.open),true);
    assert.deepEqual(await page.evaluate(start=>testCalls.slice(start).map(x=>x.action),modalCalls),['set_ui_locale']);
    await page.locator('dialog [data-action="close-modal"]').first().click();
    // Check in-place binding on an open modal and retained editor without replacing nodes.
    await page.evaluate(()=>{
      Vara.i18n.setLocale('en');
      const dialog=document.querySelector('#modal'); dialog.innerHTML='<span data-i18n="locale.heading"></span><textarea id="test-editor">User &lt;text&gt;</textarea>'; dialog.showModal();
      const editor=document.querySelector('#test-editor'); editor.focus(); editor.setSelectionRange(1,4);
      window.savedEditor=editor; window.savedDialog=dialog;
      Vara.i18n.setLocale('da');
    });
    assert.deepEqual(await page.evaluate(()=>[document.querySelector('#test-editor')===savedEditor,document.querySelector('#modal')===savedDialog,savedDialog.open,document.activeElement===savedEditor,savedEditor.selectionStart,savedEditor.selectionEnd,savedEditor.value]),[true,true,true,true,1,4,'User <text>']);
    assert.equal(await page.locator('dialog [data-i18n]').textContent(),'Grænsefladens sprog');
    await page.evaluate(()=>document.querySelector('#modal').close());
    await page.evaluate(()=>{testState.session=null; testState.downloads=[]; emitState();});
    await page.locator('[data-page="transcription"]').click();
    await page.locator('[data-action="tx-select"][data-id="file-1"]').click();
    await page.locator('[data-action="tx-edit"]').click();
    const txEditor=page.locator('#tx-editor-text'); await txEditor.fill('Draft <transcript> & content');
    await txEditor.evaluate(el=>{el.focus();el.setSelectionRange(2,8); window.actualEditor=el;});
    const beforeTxCalls=await page.evaluate(()=>testCalls.length);
    await page.evaluate(()=>{testState.settings.ui_locale='en';testState.resolved_ui_locale='en';emitState();});
    assert.deepEqual(await txEditor.evaluate(el=>[el===actualEditor,document.activeElement===el,el.selectionStart,el.selectionEnd,el.value]),[true,true,2,8,'Draft <transcript> & content']);
    assert.deepEqual(await page.evaluate(start=>testCalls.slice(start).map(x=>x.action),beforeTxCalls),[]);
    // Plain text and HTML template variants preserve visible hostile text without creating markup.
    const safe = await page.evaluate(()=>{
      const arg='<>&"\' '; const expected=Vara.i18n.message('example.items',{count:arg});
      const text=document.createElement('div'); text.textContent=expected;
      const html=document.createElement('div'); html.innerHTML=Vara.i18n.html('example.items',{count:arg});
      const input=document.createElement('input'); input.setAttribute('aria-label', expected);
      return [expected,text.textContent,html.textContent,html.children.length,input.getAttribute('aria-label')];
    });
    assert.equal(safe[0],safe[1]); assert.equal(safe[0],safe[2]); assert.equal(safe[3],0); assert.equal(safe[0],safe[4]);
    assert.deepEqual(errors,[]);
    console.log('Browser: locale save, dirty settings, selection, modal/editor bindings, HTML safety and isolated effects passed.');
  } finally { if(browser) await browser.close(); await new Promise(resolve=>server.close(resolve)); }
})().catch(error=>{console.error(error); process.exitCode=1;});
