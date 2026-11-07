// Real UI with an in-memory host. No credentials, models or external services.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const assert = require('node:assert/strict');
const desktop = path.resolve(__dirname, '..');
const fixture = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/ui-state.json')));
const runtimeCatalog = JSON.parse(fs.readFileSync(path.join(__dirname, '../../crates/vara-core/messages/runtime.json')));
const shots = process.env.VARA_I18N_SHOTS || '/tmp/vara-i20-screens';
fs.mkdirSync(shots, {recursive:true});
const server = http.createServer((req, res) => {
  const file = path.resolve(desktop, '.' + new URL(req.url, 'http://localhost').pathname);
  if (!file.startsWith(desktop + path.sep)) return res.writeHead(403).end();
  fs.readFile(file, (err, data) => {
    if (err) return res.writeHead(404).end();
    res.setHeader('Content-Type', {'.js':'text/javascript','.css':'text/css','.html':'text/html','.json':'application/json'}[path.extname(file)] || 'application/octet-stream');
    res.end(data);
  });
});
(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await chromium.launch({headless:true, ...(process.env.PLAYWRIGHT_CHROMIUM ? {executablePath:process.env.PLAYWRIGHT_CHROMIUM} : {})});
    const errors = [];
    for (const lang of ['da', 'en']) {
      const page = await browser.newPage({viewport:{width:1100,height:800}});
      page.on('pageerror', error => errors.push(error.message));
      await page.addInitScript(({data,lang,runtimeCatalog}) => {
        window.testState=structuredClone(data); window.calls=[]; window.keys=new Set(['open_ai']);
        testState.settings.ui_locale=lang; testState.resolved_ui_locale=lang; testState.ui_locale_revision=0;
        window.__TAURI__={core:{invoke:async(_,{action,payload})=>{
          calls.push({action,payload}); const s=testState;
          switch(action){
            case 'runtime_catalog': return runtimeCatalog;
            case 'snapshot': return structuredClone(s);
            case 'devices': return [];
            case 'file_list': return {documents:[],media:{}};
            case 'application_icons': return {};
            case 'applications': return [{id:'custom.app',label:'Søren 👩‍💻 <& "app"'}];
            case 'set_ui_locale': s.settings.ui_locale=payload.preference;s.resolved_ui_locale=payload.preference;return {locale:payload.preference,revision:++s.ui_locale_revision};
            case 'key_status': return keys.has(payload.id);
            case 'check_openai': return {speech_model:'speech-model',speech_available:true,text_model:'text-model',text_available:false};
            case 'save_settings': s.settings=structuredClone(payload);return;
            case 'delete_result': s.history=s.history.filter(r=>r.id!==payload.id);return;
            case 'copy': if(window.failCopy) throw Error('vara-message-v1:{"code":"host.unsaved"}');return;
          }
          return null;
        }},event:{listen:async(name,cb)=>{if(name==='vara:state')window.emit=()=>cb({payload:structuredClone(testState)});return()=>{};}}};
      },{data:fixture,lang,runtimeCatalog});
      await page.goto(`http://127.0.0.1:${server.address().port}/ui/index.html`);
      await page.locator('.history-row').first().waitFor();
      assert.equal(await page.locator('#sidebar').getAttribute('aria-label'),lang==='da'?'Hovednavigation':'Main navigation');
      // The original grid is minmax(224px, 18fr) / 82fr at these widths.
      for (const width of [1280,1586]) {
        await page.setViewportSize({width,height:900});
        const sidebarWidth=await page.locator('#sidebar').evaluate(el=>el.getBoundingClientRect().width);
        assert.ok(Math.abs(sidebarWidth-Math.max(224,width*.18))<1, `baseline sidebar ${width}: ${sidebarWidth}`);
      }
      await page.setViewportSize({width:1280,height:820});
      await page.screenshot({path:path.join(shots,`${lang}-history-1280.png`)});
      await page.setViewportSize({width:1100,height:800});
      const t = (da,en)=>lang==='da'?da:en;
      assert.equal(await page.locator('#view-title').textContent(),t('Historik','History'));
      await page.locator('.history-row [data-action=copy]').first().click();
      assert.equal(await page.locator('#toast').textContent(),t('Kopieret.','Copied.'));
      await page.evaluate(()=>{window.savedToast=document.querySelector('#toast');Vara.i18n.setLocale(document.documentElement.lang==='da'?'en':'da');});
      assert.equal(await page.locator('#toast').textContent(),t('Copied.','Kopieret.'));
      assert.equal(await page.evaluate(()=>document.querySelector('#toast')===savedToast),true);
      await page.evaluate(()=>Vara.i18n.setLocale(testState.resolved_ui_locale));
      await page.evaluate(()=>window.failCopy=true);
      await page.locator('.detail-pane [data-action=copy]').first().click();
      assert.match(await page.locator('.detail-pane .notice--error').textContent(),t(/Teksten kunne ikke gemmes/,/The text could not be saved/));
      await page.locator('.history-row [data-action=confirm-delete]').first().click();
      assert.equal(await page.locator('#dialog-title').textContent(),t('Slet diktering?','Delete dictation?'));
      await page.locator('dialog [data-action=close-modal]').first().click();
      await page.evaluate(()=>{testState.recoveries=[{id:'r',origin:{app_name:'Søren 👩‍💻 <& "app"'},expires_at:1800000000}];emit();});
      const recovery=page.locator('.recovery-task h3');
      assert.match(await recovery.textContent(),/Søren 👩‍💻 <& "app"/);
      assert.equal(await recovery.locator('app').count(),0);
      await page.evaluate(()=>Vara.i18n.setLocale(document.documentElement.lang==='da'?'en':'da'));
      assert.match(await recovery.textContent(),t(/Transcription from Søren/,/Transskription fra Søren/));
      await page.evaluate(()=>Vara.i18n.setLocale(testState.resolved_ui_locale));
      await page.locator('[data-page=apps]').click();
      await page.locator('[data-action=add-app]').click();
      assert.equal(await page.locator('#dialog-title').textContent(),t('Tilføj app','Add app'));
      await page.locator('dialog [data-action=save-new-app]').click();
      assert.equal(await page.locator('#app-form h2').textContent(),'Søren 👩‍💻 <& "app"');
      await page.locator('#app-ai').check(); await page.locator('#app-prompt').fill('My prompt æøå <& 👩‍💻');
      await page.locator('[data-action=save-app]').click();
      assert.equal(await page.evaluate(()=>testState.settings.app_rules['custom.app'].prompt),'My prompt æøå <& 👩‍💻');
      await page.locator('[data-page=settings]').click();
      assert.match(await page.locator('#locale-picker header p').textContent(),t(/talesprog/,/speech/));
      await page.setViewportSize({width:1280,height:820});
      await page.screenshot({path:path.join(shots,`${lang}-settings-1280.png`)});
      await page.setViewportSize({width:1100,height:800});
      assert.match(await page.locator('#speech-engine option[value="local.parakeet"]').textContent(),t(/på enheden/,/on device/));
      await page.evaluate(()=>{testState.settings.engines.find(e=>e.id==='local.whisper').label='Mine Ændrede';emit();});
      assert.equal(await page.locator('#speech-engine option[value="local.whisper"]').textContent(),'Mine Ændrede');
      assert.match(await page.locator('#speech-engine option[value="gemini.speech"]').textContent(),t(/Gemini · tale/,/Gemini · speech/));
      await page.locator('#speech-engine').selectOption('gemini.speech');
      await page.locator('[data-action=save-settings]').click();
      assert.equal(await page.evaluate(()=>testState.settings.speech_engine),'gemini.speech');
      assert.equal(await page.evaluate(()=>testState.settings.general_engine),fixture.settings.general_engine);
      await page.locator('[data-action=settings-tab][data-id=connections]').click();
      const geminiRow=page.locator('.provider-row').filter({hasText:'Gemini'}).first();
      assert.match(await geminiRow.textContent(),t(/Tale og tekst/,/Speech and text/));
      await geminiRow.locator('[data-action=open-key]').click();
      assert.equal(await page.locator('#api-key').count(),1);
      await page.locator('dialog [data-action=close-modal]').first().click();
      await page.locator('[data-action=open-key][data-id=open_ai]').first().click();
      await page.locator('#api-key').fill('fixture-key');
      const modal=page.locator('dialog');
      await modal.locator('[data-action=check-openai]').click();
      assert.match(await page.locator('#openai-access-status').textContent(),t(/Nøglen accepteres/,/Key accepted/));
      await page.evaluate(()=>{window.modalInput=document.querySelector('#api-key');modalInput.focus();modalInput.setSelectionRange(1,4);Vara.i18n.setLocale(document.documentElement.lang==='da'?'en':'da');});
      assert.equal(await page.locator('#api-key').inputValue(),'fixture-key');
      assert.deepEqual(await page.evaluate(()=>[document.querySelector('#api-key')===modalInput,document.activeElement===modalInput,modalInput.selectionStart,modalInput.selectionEnd]),[true,true,1,4]);
      assert.match(await page.locator('#openai-access-status').textContent(),t(/Key accepted/,/Nøglen accepteres/));
      await page.evaluate(()=>Vara.i18n.setLocale(testState.resolved_ui_locale));
      await modal.locator('[data-action=close-modal]').first().click();
      await page.locator('#advanced-engines summary').click();
      await page.locator('[data-action=add-endpoint]').click();
      assert.equal(await page.locator('#dialog-title').textContent(),t('Eget endpoint','Custom endpoint'));
      await page.locator('#engine-label').fill('Custom æøå');
      await page.evaluate(()=>{window.engineInput=document.querySelector('#engine-label');Vara.i18n.setLocale(document.documentElement.lang==='da'?'en':'da');});
      assert.equal(await page.locator('#engine-label').inputValue(),'Custom æøå');
      assert.equal(await page.evaluate(()=>document.querySelector('#engine-label')===engineInput),true);
      assert.equal(await page.locator('#dialog-title').textContent(),t('Custom endpoint','Eget endpoint'));
      await page.evaluate(()=>Vara.i18n.setLocale(testState.resolved_ui_locale));
      await modal.locator('[data-action=close-modal]').first().click();
      await page.locator('[data-action=settings-tab][data-id=privacy]').click();
      assert.equal(await page.locator('label[for=retention]').textContent(),t('Behold teksten','Keep text'));
      // Binding overrides must win; removed render trees must not accumulate.
      const binding = await page.evaluate(async()=>{
        const host=document.createElement('div');document.body.append(host);
        host.innerHTML=`<span id="bound">${Vara.i18n.bind('recovery.failed',{app:'Søren 👩‍💻 <& "app"'})}</span><button id="bound-button" aria-label="${Vara.i18n.bind('ui.copy')}"></button>`;
        await new Promise(requestAnimationFrame);
        const initial=host.querySelector('span').textContent, children=host.querySelector('span').children.length;
        host.querySelector('span').firstChild.data='new live status';host.querySelector('button').setAttribute('aria-label','new action');
        Vara.i18n.setLocale(document.documentElement.lang==='da'?'en':'da');
        const after=[host.querySelector('span').textContent,host.querySelector('button').getAttribute('aria-label')];
        host.innerHTML=`<span>${Vara.i18n.bind('ui.copy')}</span><button aria-label="${Vara.i18n.bind('ui.copy')}"></button>`;
        await new Promise(requestAnimationFrame);
        const sameText=host.querySelector('span').firstChild, sameButton=host.querySelector('button');
        const sameBefore=[sameText.data,sameButton.getAttribute('aria-label')];
        sameText.data=sameText.data;sameButton.setAttribute('aria-label',sameButton.getAttribute('aria-label'));
        Vara.i18n.setLocale(document.documentElement.lang==='da'?'en':'da');
        const sameAfter=[sameText.data,sameButton.getAttribute('aria-label')];
        for(let i=0;i<80;i++){host.innerHTML=`<span>${Vara.i18n.bind('model.download',{received:i,total:80})}</span>`;await new Promise(requestAnimationFrame);}
        const count=Vara.i18n.bindingCount();host.remove();await new Promise(requestAnimationFrame);const released=Vara.i18n.bindingCount();
        return {initial,children,after,sameBefore,sameAfter,count,released};
      });
      assert.match(binding.initial,/Søren 👩‍💻 <& "app"/);assert.equal(binding.children,0);
      assert.deepEqual(binding.after,['new live status','new action']);assert.deepEqual(binding.sameAfter,binding.sameBefore);assert.ok(binding.count-binding.released<=2,JSON.stringify(binding));
      await page.evaluate(()=>Vara.i18n.setLocale(testState.resolved_ui_locale));
      assert.equal(await page.evaluate(()=>JSON.stringify(calls.filter(c=>['save_settings','copy'].includes(c.action))).includes('__vara_')),false);
      await page.setViewportSize({width:720,height:560});
      await page.locator('[data-action=settings-tab][data-id=dictation]').click();
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
      await page.screenshot({path:path.join(shots,`${lang}-settings-720.png`)});
      // Double computed text sizes without scaling the viewport or controls.
      await page.evaluate(()=>{const nodes=[...document.querySelectorAll('body *')];const sizes=nodes.map(el=>parseFloat(getComputedStyle(el).fontSize));nodes.forEach((el,i)=>el.style.fontSize=(sizes[i]*2)+'px');});
      await page.screenshot({path:path.join(shots,`${lang}-settings-200-percent.png`)});
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),true);
      const navReachable = await page.evaluate(()=>[...document.querySelectorAll('.sidebar .nav-item')].every(el=>{
        el.focus(); el.scrollIntoView({block:'nearest'}); const text=el.querySelector('span').getBoundingClientRect(), side=document.querySelector('.sidebar').getBoundingClientRect();
        return document.activeElement===el && text.left>=side.left && text.right<=side.right && text.top>=side.top && text.bottom<=side.bottom;
      }));
      assert.equal(navReachable,true,'Sidebar labels remain within the scrollable navigation and keyboard reachable');
      await page.locator('.sidebar .nav-item').first().focus();
      await page.keyboard.press('Tab');
      assert.equal(await page.evaluate(()=>document.activeElement?.dataset.page),'transcription');
      for (const selector of ['.settings-nav [data-id="dictation"]','.settings-nav [data-id="connections"]','.settings-nav [data-id="privacy"]','#ui-locale','[data-action="save-settings"]']) {
        const item=page.locator(selector).first(); await item.scrollIntoViewIfNeeded();
        const fits=await item.evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight;});
        assert.equal(fits,true,`${selector} is reachable at enlarged text size`);
      }
      await page.evaluate(()=>{document.querySelectorAll('body *').forEach(el=>el.style.removeProperty('font-size'));testState.history=[];testState.settings.onboarding_complete=false;testState.models['local.parakeet']=true;testState.permissions.microphone='authorized';emit();});
      await page.locator('[data-page=history]').click();
      await page.locator('[data-action=setup]').last().click();
      assert.equal(await page.locator('#view-title').textContent(),t('Dine ord, dér hvor du arbejder.','Your words, where you work.'));
      await page.locator('[data-action=setup-next]').click();
      assert.equal(await page.locator('#view-title').textContent(),t('Giv Vara adgang.','Give Vara access.'));
      await page.locator('[data-action=setup-next]').click();
      assert.equal(await page.locator('#view-title').textContent(),t('Sig din første sætning.','Say your first sentence.'));
      await page.screenshot({path:path.join(shots,`${lang}-onboarding-720.png`)});
      // Test-only longer catalog entries exercise the real binding/render path.
      await page.evaluate(lang => {
        const catalog=Vara.i18n.catalogs[lang];
        for (const key of Object.keys(catalog)) {
          const extend = value => value + (lang === 'da' ? ' ekstra forklarende tekst' : ' additional descriptive wording');
          if (typeof catalog[key] === 'string') catalog[key] = extend(catalog[key]);
          else for (const form of Object.keys(catalog[key])) catalog[key][form] = extend(catalog[key][form]);
        }
      }, lang);
      await page.setViewportSize({width:1280,height:820});
      await page.locator('[data-page=settings]').click();
      await page.screenshot({path:path.join(shots,`${lang}-long-settings-1280.png`)});
      await page.setViewportSize({width:720,height:560});
      await page.screenshot({path:path.join(shots,`${lang}-long-settings-720.png`)});
      const scaling=await page.evaluate(()=>{
        const nodes=[...document.querySelectorAll('body *')];
        const sizes=nodes.map(el=>parseFloat(getComputedStyle(el).fontSize));
        nodes.forEach((el,i)=>el.style.fontSize=(sizes[i]*2)+'px');
        const target=document.querySelector('#ui-locale');
        const index=nodes.indexOf(target);
        return [sizes[index],parseFloat(getComputedStyle(target).fontSize)];
      });
      assert.ok(Math.abs(scaling[1]-scaling[0]*2)<0.1, `computed font scaling ${scaling}`);
      for (const selector of ['#ui-locale','[data-action="record-shortcut"]','[data-action="save-settings"]']) {
        const control=page.locator(selector).first(); await control.scrollIntoViewIfNeeded();
        const box=await control.boundingBox();
        assert.ok(box && box.x>=0 && box.x+box.width<=720 && box.y>=0 && box.y+box.height<=560,`${lang} long ${selector} ${JSON.stringify(box)}`);
      }
      const longNavReachable=await page.evaluate(()=>[...document.querySelectorAll('.sidebar .nav-item')].every(el=>{
        el.focus(); el.scrollIntoView({block:'nearest'});
        const text=el.querySelector('span').getBoundingClientRect(), side=document.querySelector('.sidebar').getBoundingClientRect();
        return document.activeElement===el && text.left>=side.left && text.right<=side.right && text.top>=side.top && text.bottom<=side.bottom;
      }));
      assert.equal(longNavReachable,true,`${lang} long navigation remains reachable`);
      await page.screenshot({path:path.join(shots,`${lang}-long-settings-720-200.png`)});
      await page.locator('[data-action="record-shortcut"]').click();
      assert.equal(await modal.evaluate(el=>el.open),true);
      assert.ok(await modal.getAttribute('aria-labelledby'));
      assert.equal(await page.evaluate(()=>document.activeElement.id),'shortcut-recorder');
      await page.keyboard.press('Tab');
      assert.equal(await page.evaluate(()=>document.activeElement.dataset.action),'close-modal');
      assert.match(await page.locator('#shortcut-recorder').textContent(),/Tab/);
      await page.keyboard.press('Enter');
      assert.equal(await modal.evaluate(el=>el.open),false);
      await page.close();
    }
    assert.deepEqual(errors, []);
    console.log(`Both locales: history/copy/delete/failure/recovery, app rule, settings, real key/endpoint dialogs, retained drafts, UTF-8 and binding lifecycle passed. Screenshots: ${shots}`);
  } finally { if(browser) await browser.close(); await new Promise(resolve=>server.close(resolve)); }
})().catch(error=>{console.error(error);process.exitCode=1;});
