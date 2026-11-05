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
  fixture.settings.ui_locale=locale;fixture.resolved_ui_locale=locale;window.calls=[];window.update={status:'disabled',automatic:true,version:null,downloaded:false};
  window.__TAURI__={core:{invoke:async(_,{action,payload})=>{calls.push({action,payload});
   if(action==='runtime_catalog')return runtime;if(action==='snapshot')return fixture;if(action==='file_list')return {documents:[],media:{}};if(action==='devices')return [];if(action==='application_icons')return {};
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
 for(const status of ['checking','current','error','available','downloading','ready','blocked']){
  await page.evaluate(status=>emitUpdate({status,version:['available','downloading','ready','blocked'].includes(status)?'1.0.1':null,downloaded:['ready','blocked'].includes(status)}),status);
  assert.equal(await page.locator('[data-action=update-check]').isDisabled(),['checking','downloading'].includes(status),status);
  if(status==='available')assert.ok(await page.locator('[data-action=update-download]').count());
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
 await page.evaluate(()=>Vara.transcription.hasUnsavedChanges=()=>true);
 await page.locator('[data-action=update-install]').first().click();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length),0);
 await page.evaluate(()=>Vara.transcription.hasUnsavedChanges=()=>false);
 await page.locator('[data-action=update-install]').first().click();assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='update_install').length),1);
 assert.equal(await page.locator('#app-window').evaluate(e=>e.inert),false);
 assert.deepEqual(errors,[]);await page.close();
 }
 console.log('PASS: update states, localization, manual check, opt-out, dirty Settings/editor refusal and explicit install action');
}finally{await browser?.close();await new Promise(r=>server.close(r));}})().catch(e=>{console.error(e);process.exitCode=1;});
