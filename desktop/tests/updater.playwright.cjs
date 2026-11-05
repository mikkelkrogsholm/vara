// Real app/controller and shared components, with a deterministic in-memory host.
const {chromium}=require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs=require('node:fs'),http=require('node:http'),path=require('node:path'),assert=require('node:assert/strict');
const desktop=path.resolve(__dirname,'..');
const fixture=JSON.parse(fs.readFileSync(path.join(__dirname,'fixtures/ui-state.json')));
const runtime=JSON.parse(fs.readFileSync(path.join(__dirname,'../../crates/vara-core/messages/runtime.json')));
const server=http.createServer((req,res)=>{const file=path.resolve(desktop,'.'+new URL(req.url,'http://localhost').pathname);if(!file.startsWith(desktop+path.sep))return res.writeHead(403).end();fs.readFile(file,(err,data)=>{if(err)return res.writeHead(404).end();res.setHeader('Content-Type',{'.js':'text/javascript','.html':'text/html','.css':'text/css'}[path.extname(file)]||'application/octet-stream');res.end(data);});});
(async()=>{let browser;await new Promise(r=>server.listen(0,'127.0.0.1',r));try{
 browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM?{executablePath:process.env.PLAYWRIGHT_CHROMIUM}:{})});
 for(const locale of ['en','da']){
 const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(({fixture,runtime,locale})=>{
  fixture.settings.ui_locale=locale;fixture.resolved_ui_locale=locale;window.calls=[];window.update={current_version:'1.0.0',status:'disabled',automatic:true,version:null,downloaded:false};
  const document={id:'update-editor',title:'Update editor fixture',source_name:'fixture.wav',created_at:0,engine:fixture.settings.engines[0],language:'en',stage:'ready',duration:1,completed_seconds:1,revision:0,edited_text:null,ai_text:null,speakers:{},transcript:{text:'Original text',timing:'none',segments:[{text:'Original text',start:null,end:null,speaker:null,words:[]}]}};
  window.__TAURI__={core:{invoke:async(_,{action,payload})=>{calls.push({action,payload});
   if(action==='runtime_catalog')return runtime;if(action==='snapshot')return fixture;if(action==='file_list')return {documents:[document],media:{}};if(action==='file_edit')return new Promise(()=>{});if(action==='devices')return [];if(action==='application_icons')return {};
   if(action==='update_preference')update.automatic=payload.automatic;
   if(action.startsWith('update_'))return {...update};return null;
  }},event:{listen:async(name,cb)=>{if(name==='vara:update')window.emitUpdate=value=>{Object.assign(update,value);cb({payload:{...update}});};return()=>{};}}};
 },{fixture,runtime,locale});
 await page.goto(`http://127.0.0.1:${server.address().port}/ui/index.html`);
 await page.locator('.history-row').first().waitFor();
 assert.equal(await page.locator('[data-action=update-install]').count(),0);
 await page.locator('[data-action=navigate][data-page=settings]').click();
 await page.locator('[data-action=update-check]').waitFor();
 assert.equal(await page.locator('[data-action=update-check]').isDisabled(),true);
 assert.equal(await page.locator('#update-current-version').textContent(),locale==='da'?'Du bruger Vara 1.0.0':'Running Vara 1.0.0');
 for(const status of ['checking','current','error','available','downloading','ready','blocked']){
  await page.evaluate(status=>emitUpdate({status,version:['available','downloading','ready','blocked'].includes(status)?'1.0.1':null,downloaded:['ready','blocked'].includes(status)}),status);
  assert.equal(await page.locator('[data-action=update-check]').isDisabled(),['checking','downloading'].includes(status),status);
  if(status==='available'){assert.ok(await page.locator('[data-action=update-download]').count());assert.ok((await page.locator('#sidebar-update').textContent()).includes('1.0.1'));assert.ok((await page.locator('#update-current-version').textContent()).includes('1.0.0'));}
  if(['ready','blocked'].includes(status))assert.ok(await page.locator('[data-action=update-install]').count());
 }
 await page.locator('#update-automatic').uncheck();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_preference').at(-1).payload.automatic),false);
 await page.locator('[data-action=update-check]').click();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_check').at(-1).payload.manual),true);
 // The action itself protects drafts even if a stale enabled button remains in DOM.
 await page.locator('label[for=general-ai]').click();
 await page.locator('#general-prompt').fill('unsaved settings draft');
 await page.locator('[data-action=update-install]').first().click();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length),0);
 await page.locator('[data-action=update-check]').click();assert.equal(await page.locator('#general-prompt').inputValue(),'unsaved settings draft');
 await page.locator('#update-automatic').check();assert.equal(await page.locator('#general-prompt').inputValue(),'unsaved settings draft');
 await page.locator('[data-action=reset-form]').click();
 await page.evaluate(()=>{window.realUnsavedChanges=Vara.transcription.hasUnsavedChanges;Vara.transcription.hasUnsavedChanges=()=>true;});
 await page.locator('[data-action=update-install]').first().click();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length),0);
 await page.evaluate(()=>Vara.transcription.hasUnsavedChanges=realUnsavedChanges);
 await page.locator('[data-action=update-install]').first().click();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length),1);
 assert.equal(await page.locator('#app-window').evaluate(e=>e.inert),false);
 // The shared notice must arrive on every main page, including a live file editor.
 await page.evaluate(()=>emitUpdate({status:'available',version:'1.0.1',downloaded:false}));
 for(const section of ['history','apps','transcription']){
  await page.locator(`[data-action=navigate][data-page=${section}]`).click();
  assert.ok((await page.locator('#sidebar-update').textContent()).includes(locale==='da'?'tilgængelig':'available'));
 }
 await page.locator('[data-action=tx-select][data-id=update-editor]').click();
 await page.locator('[data-action=tx-edit]').click();
 await page.locator('#tx-editor-text').fill('Unsaved updater test draft');
 await page.locator('#tx-editor-text').evaluate(e=>{window.liveUpdateEditor=e;e.setSelectionRange(4,9);});
 await page.evaluate(()=>emitUpdate({status:'available',version:'1.0.1',downloaded:false}));
 await page.locator('#sidebar-update [data-action=update-download]').click();
 assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_download').length),1);
 assert.equal(await page.locator('#tx-editor-text').inputValue(),'Unsaved updater test draft');
 assert.equal(await page.evaluate(()=>document.getElementById('tx-editor-text')===liveUpdateEditor),true);
 assert.deepEqual(await page.locator('#tx-editor-text').evaluate(e=>[e.selectionStart,e.selectionEnd]),[4,9]);
 assert.equal(await page.evaluate(()=>Vara.transcription.hasUnsavedChanges()),true);
 await page.evaluate(()=>emitUpdate({status:'ready',downloaded:true}));
 const installs=await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length);
 await page.locator('#sidebar-update [data-action=update-install]').click();
 assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length),installs);
 assert.equal(await page.locator('#tx-editor-text').inputValue(),'Unsaved updater test draft');
 assert.deepEqual(errors,[]);await page.close();
 }
 console.log('PASS: current/remote versions, DA/EN shared notices on all views, real editor/selection retention, manual check, opt-out and install guards');
}finally{await browser?.close();await new Promise(r=>server.close(r));}})().catch(e=>{console.error(e);process.exitCode=1;});
