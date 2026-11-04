// Sanitised in-memory host. This exercises the actual transcription UI without app data or services.
const {chromium} = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs'), http = require('node:http'), path = require('node:path'), assert = require('node:assert/strict');
const desktop = path.resolve(__dirname, '..');
const fixture = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/ui-state.json')));
const shots = process.env.VARA_I18N_SHOTS || '/tmp/vara-i21-screens'; fs.mkdirSync(shots, {recursive:true});
const server = http.createServer((req,res) => {
  const file = path.resolve(desktop, '.' + new URL(req.url, 'http://localhost').pathname);
  if (!file.startsWith(desktop + path.sep)) return res.writeHead(403).end();
  fs.readFile(file, (err,data) => { if(err) return res.writeHead(404).end(); if(req.headers.range && file.endsWith('.wav')) { const start=Number(req.headers.range.match(/bytes=(\d+)/)?.[1] || 0); res.writeHead(206,{'Content-Type':'audio/wav','Accept-Ranges':'bytes','Content-Range':`bytes ${start}-${data.length-1}/${data.length}`,'Content-Length':data.length-start}); return res.end(data.subarray(start)); } res.setHeader('Content-Type', ({'.js':'text/javascript','.css':'text/css','.html':'text/html','.json':'application/json','.wav':'audio/wav'})[path.extname(file)] || 'application/octet-stream'); res.end(data); });
});
(async () => {
  await new Promise(resolve => server.listen(0,'127.0.0.1',resolve)); let browser;
  try {
    browser = await chromium.launch({headless:true, ...(process.env.PLAYWRIGHT_CHROMIUM ? {executablePath:process.env.PLAYWRIGHT_CHROMIUM} : {})});
    const page = await browser.newPage({viewport:{width:1100,height:800}}), errors=[];
    page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(data => {
      const NativeAudio=window.Audio; window.Audio=function(...args){ window.testAudio=new NativeAudio(...args); return window.testAudio; };
      data.settings.ui_locale='da'; data.resolved_ui_locale='da'; data.ui_locale_revision=0;
      window.calls=[]; window.events={};
      const engine=data.settings.engines.find(e=>e.id==='local.parakeet');
      data.settings.engines.push({...structuredClone(data.settings.engines.find(e=>e.id==='openai.speech')),id:'custom.speech',label:'Custom Æ'});
      window.docs=[{id:'doc',title:'Interview <A>',source_name:'original.wav',source_path:'/original.wav',created_at:1,engine,language:'da',stage:'ready',error:null,duration:9,completed_seconds:9,revision:4,edited_text:null,ai_text:null,ai_engine:null,speakers:{'legacy':'Taler 1'},transcript:{text:'Original Æ text',timing:'word',timing_source:'asr',segments:[{id:'seg',text:'Original Æ text',start:0.1,end:2,speaker:'legacy',words:[{id:'word',text:'Original Æ text',start:0.1,end:2,speaker:'legacy'}]}]}}];
      const emit=()=>events['vara:files']?.({payload:null});
      window.__TAURI__={core:{convertFileSrc:()=>'/tests/fixtures/player.wav',invoke:async(_,{action,payload})=>{
        calls.push({action,payload}); const d=docs.find(x=>x.id===payload?.id);
        switch(action){
          case 'snapshot': return structuredClone(data);
          case 'devices': return [];
          case 'application_icons': return {};
          case 'file_list': { const snapshot={documents:structuredClone(docs),media:{doc:'/player.wav'},who_download_busy:!!window.whoDownloadBusy}; const delay=window.nextListDelay||0, fail=window.nextListFailure; window.nextListDelay=0; window.nextListFailure=false; if(delay) await new Promise(resolve=>setTimeout(resolve,delay)); if(fail) throw Error('stale failure sentinel'); return snapshot; }
          case 'file_pick': return [{path:'/f1.wav',name:'f1.wav',bytes:1500000},{path:'/f2.wav',name:'f2.wav',bytes:2000000}];
          case 'file_add': docs.push({...structuredClone(docs[0]),id:'queued',title:'queued',stage:'transcribing',completed_seconds:2,transcript:null}); emit();return ['queued'];
          case 'file_cancel': d.stage='cancelled';emit();return;
          case 'file_retry': d.stage='queued';emit();return;
          case 'file_save': d.edited_text=payload.text;d.revision++;emit();return {revision:d.revision};
          case 'file_ai_cancel': d.ai_busy=false;emit();return;
          case 'file_who_cancel': window.whoDownloadBusy=false;emit();return;
          case 'file_ai': await new Promise(resolve=>window.finishAI=resolve);d.ai_text='AI result Æ';emit();return;
          case 'file_who_status': return {available:true,ready:!!window.whoReady};
          case 'file_who_download': await new Promise(resolve=>window.finishDownload=resolve);window.whoReady=true;return;
          case 'file_who': await new Promise(resolve=>window.finishWho=resolve);d.diarization={turns:[{speaker:'who-0',start:0,end:2},{speaker:'who-uncertain',start:2,end:4}]};d.speakers['who-0']='Taler 1';d.auto_speakers={'who-0':1};d.speaker_transcript=structuredClone(d.transcript);d.speaker_transcript.segments[0].speaker='who-0';emit();return;
          case 'file_speaker': d.speakers[payload.speaker]=payload.name;delete d.auto_speakers?.[payload.speaker];emit();return;
          case 'file_export': return true;
        }
        return null;
      }},event:{listen:async(name,fn)=>{events[name]=fn;return()=>{};}}};
    },fixture);
    await page.goto(`http://127.0.0.1:${server.address().port}/ui/index.html`); await page.locator('[data-page="transcription"]').click();
    const action = name => page.locator(`[data-action="${name}"]:visible`).first();
    const locale = async lang => { await page.evaluate(lang=>Vara.i18n.setLocale(lang),lang); await page.waitForTimeout(30); };
    await action('tx-pick').click(); assert.match(await page.locator('.tx-staged').innerText(),/1,5 MB/); assert.match(await page.locator('.tx-import').innerText(),/2 filer klar/);
    const importsBefore=await page.evaluate(()=>calls.filter(c=>c.action==='file_add').length); await locale('en');
    assert.match(await page.locator('.tx-import').innerText(),/2 files ready/); assert.match(await page.locator('.tx-staged').innerText(),/1\.5 MB/);
    assert.equal(await page.evaluate(()=>calls.filter(c=>c.action==='file_add').length),importsBefore);
    await page.locator('#tx-engine').selectOption('openai.speech'); assert.match(await page.locator('.tx-locality').innerText(),/OpenAI · speech/);
    await locale('da'); assert.match(await page.locator('.tx-locality').innerText(),/OpenAI · tale/); await page.locator('#tx-engine').selectOption('custom.speech'); assert.match(await page.locator('.tx-locality').innerText(),/Custom Æ/);
    await locale('en'); assert.match(await page.locator('.tx-locality').innerText(),/Custom Æ/); await page.locator('#tx-engine').selectOption('local.parakeet');
    await action('tx-start').click(); await page.getByRole('button',{name:/Cancel queued/}).waitFor(); assert.match(await page.locator('.tx-detail').innerText(),/0:02 of 0:09 audio processed/); const beforeJobSwitch=await page.evaluate(()=>calls.length);
    await locale('da'); assert.equal(await page.evaluate(()=>calls.length),beforeJobSwitch); assert.match(await page.locator('.tx-detail').innerText(),/0:02 af 0:09 lyd behandlet/); await page.locator('.tx-detail [data-action="tx-cancel"]').click(); await action('tx-retry').click();
    await page.locator('[data-action="tx-select"][data-id="doc"]').click();
    const sourceBefore=await page.evaluate(()=>JSON.stringify(docs[0]));
    assert.equal(await page.locator('[data-action="tx-speaker"][data-id="legacy"]').first().textContent(),'Taler 1');
    await locale('en'); assert.equal(await page.locator('[data-action="tx-speaker"][data-id="legacy"]').first().textContent(),'Taler 1'); assert.equal(await page.locator('#tx-speed option[value="0.75"]').textContent(),'0.75×');
    await action('tx-play').click(); await page.waitForFunction(()=>window.testAudio.readyState >= 1); await page.evaluate(()=>{window.testAudio.currentTime=2}); await page.waitForTimeout(100);
    await action('tx-edit').click(); const edit=page.locator('#tx-editor-text'); await edit.fill('Draft Æ unchanged'); await edit.evaluate(el=>{el.focus();el.setSelectionRange(2,7)});
    const state=await page.evaluate(()=>({editor:document.querySelector('#tx-editor-text'),audio:window.testAudio.currentTime}));
    await page.evaluate(()=>{window.retainedEditor=document.querySelector('#tx-editor-text');window.retainedAudio=window.testAudio;window.beforeCalls=calls.length;});
    await locale('da');
    assert.equal(await edit.inputValue(),'Draft Æ unchanged'); assert.deepEqual(await edit.evaluate(el=>[el===window.retainedEditor,document.activeElement===el,el.selectionStart,el.selectionEnd]),[true,true,2,7]);
    assert.equal(await page.evaluate(()=>calls.slice(window.beforeCalls).filter(c=>['file_add','file_ai','file_who','file_export','file_save'].includes(c.action)).length),0);
    assert.ok(state.audio >= 2); const audioAfter=await page.evaluate(()=>({same:window.testAudio===window.retainedAudio,time:window.testAudio.currentTime,paused:window.testAudio.paused})); assert.ok(audioAfter.same && !audioAfter.paused && audioAfter.time>=state.audio-0.1 && audioAfter.time<state.audio+1); assert.equal(await page.locator('#tx-speed option[value="0.75"]').textContent(),'0,75×'); assert.match(await page.locator('[data-action="tx-play"]').innerText(),/Pause/);
    await action('tx-save').click(); await action('tx-original').click();
    await page.evaluate(()=>window.getSelection().removeAllRanges()); await action('tx-selection').click();
    assert.match(await page.locator('#tx-errors').innerText(),/Markér først en passage/); await locale('en'); assert.match(await page.locator('#tx-errors').innerText(),/First select a passage/); await locale('da');
    await action('tx-rename').click(); const rename=page.locator('#tx-title-input'); await rename.fill('Unchanged title draft'); await locale('en'); assert.equal(await rename.inputValue(),'Unchanged title draft'); assert.match(await page.locator('dialog').innerText(),/Rename transcription/); await page.locator('dialog [data-action="close-modal"]').first().click();
    await action('tx-export').click(); await page.locator('#tx-export-format').selectOption('srt'); await page.locator('#tx-export-original').selectOption('original'); await locale('da'); assert.equal(await page.locator('#tx-export-format').inputValue(),'srt'); assert.equal(await page.locator('#tx-export-original').inputValue(),'original'); assert.match(await page.locator('dialog').innerText(),/Eksportér transskription/); await action('tx-export-save').click();
    assert.deepEqual(await page.evaluate(()=>calls.filter(c=>c.action==='file_export').at(-1).payload),{id:'doc',format:'srt',original:true,ai:false,draft:undefined});
    for (const format of ['txt','md','json','srt','vtt']) {
      await action('tx-export').click(); await page.locator('#tx-export-format').selectOption(format);
      await page.locator('#tx-export-original').selectOption('original'); await action('tx-export-save').click();
      const sent=await page.evaluate(()=>calls.filter(c=>c.action==='file_export').at(-1).payload);
      assert.equal(sent.format,format); assert.equal(sent.original,true); assert.equal(sent.draft,undefined);
    }
    await action('tx-tab').click(); // transcript tab stays selected; select AI explicitly below
    await page.locator('[data-action="tx-tab"][data-id="ai"]').click();
    // A new UI has no local pending promise but must display and cancel backend work.
    await page.evaluate(()=>{docs[0].ai_busy=true;events['vara:files']({payload:null});});
    await action('tx-ai-cancel').waitFor(); assert.equal(await action('tx-ai').isDisabled(),true);
    await action('tx-ai-cancel').click(); await page.waitForFunction(()=>!docs[0].ai_busy);
    await page.waitForFunction(()=>!document.querySelector('[data-action="tx-ai-cancel"]'));
    await page.locator('#tx-ai-prompt').fill('User prompt Æ'); await action('tx-ai').click({timeout:2000});
    await page.waitForFunction(()=>!!window.finishAI); const beforeAI=await page.evaluate(()=>calls.length); await locale('en'); assert.match(await page.locator('.tx-ai').innerText(),/Processing/); assert.equal(await page.evaluate(()=>calls.length),beforeAI); await page.evaluate(()=>finishAI()); await page.waitForSelector('.tx-ai-result'); assert.match(await page.locator('.tx-ai-result').innerText(),/AI result Æ/);
    await page.locator('[data-action="tx-tab"][data-id="transcript"]').click();
    await page.evaluate(()=>{window.whoDownloadBusy=true;events['vara:files']({payload:null});});
    await page.waitForTimeout(400); await action('tx-who-open').click();
    await action('tx-who-download-cancel').waitFor(); await action('tx-who-download-cancel').click();
    await page.waitForSelector('[data-action="tx-who-download"]');
    await action('tx-who-download').click(); await page.waitForFunction(()=>!!window.finishDownload); const beforeDownload=await page.evaluate(()=>calls.length); await locale('da'); assert.match(await page.locator('dialog').innerText(),/Henter og klargør Who/); assert.equal(await page.evaluate(()=>calls.length),beforeDownload); await page.evaluate(()=>finishDownload()); await page.locator('#tx-who-count').waitFor(); await page.locator('#tx-who-count').selectOption('1'); await locale('en'); await locale('da'); assert.equal(await page.locator('#tx-who-count').inputValue(),'1'); assert.match(await page.locator('dialog').innerText(),/Taleropdeling med Who/); await action('tx-who-run').click(); await page.waitForFunction(()=>!!window.finishWho); const beforeWho=await page.evaluate(()=>calls.length); await locale('en'); assert.match(await page.locator('#tx-who-tools').innerText(),/Finding speakers/); assert.equal(await page.evaluate(()=>calls.length),beforeWho); await page.evaluate(()=>finishWho()); await page.waitForSelector('.tx-who-turns'); await page.locator('.tx-who-turns summary').click(); assert.match(await page.locator('#tx-who-tools').innerText(),/1 speaker · 2 seconds with uncertain speaker/); assert.match(await page.locator('.tx-who-turns').innerText(),/Speaker 1/); await locale('da'); assert.match(await page.locator('.tx-who-turns').innerText(),/Taler 1/);
    await page.locator('.tx-who-turns [data-action="tx-speaker"]').click(); await page.locator('#tx-speaker-input').fill('Person Æ'); await locale('en'); assert.equal(await page.locator('#tx-speaker-input').inputValue(),'Person Æ'); await action('tx-speaker-save').click(); assert.match(await page.locator('.tx-who-turns').textContent(),/Person Æ/);
    const after=await page.evaluate(()=>docs[0]); const original=JSON.parse(sourceBefore); assert.deepEqual(after.transcript,original.transcript); assert.equal(after.source_path,original.source_path); assert.equal(after.revision,original.revision+1); assert.equal(after.edited_text,'Draft Æ unchanged'); assert.equal(after.speakers.legacy,'Taler 1'); assert.equal(after.speakers['who-0'],'Person Æ'); assert.equal(after.ai_text,'AI result Æ');
    // A slower pre-restart snapshot must not overwrite a newer reload.
    await page.evaluate(()=>{docs[0].title='Old snapshot';window.nextListDelay=900;events['vara:files']({payload:null});});
    await page.waitForTimeout(350);
    await page.evaluate(()=>{docs[0].title='New snapshot';events['vara:files']({payload:null});});
    await page.waitForTimeout(1200);
    assert.match(await page.locator('.tx-title').innerText(),/New snapshot/);
    await page.evaluate(()=>{window.nextListDelay=900;window.nextListFailure=true;events['vara:files']({payload:null});});
    await page.waitForTimeout(350);
    await page.evaluate(()=>events['vara:files']({payload:null}));
    await page.waitForTimeout(1200);
    assert.doesNotMatch(await page.locator('body').innerText(),/stale failure sentinel/);
    await page.evaluate(()=>{docs[0].title='Interview <A>';events['vara:files']({payload:null});});
    await page.waitForTimeout(400);
    await page.evaluate(() => {
      window.restoreText = () => { for (const [el,size] of window.scaledText || []) if (el.isConnected) el.style.fontSize=size; window.scaledText=[]; };
      window.doubleText = () => { window.restoreText(); const els=[...document.querySelectorAll('body *')]; const sizes=els.map(el=>parseFloat(getComputedStyle(el).fontSize)); window.scaledText=els.map(el=>[el,el.style.fontSize]); els.forEach((el,i)=>{el.style.fontSize=`${sizes[i]*2}px`}); };
    });
    const restore = () => page.evaluate(()=>window.restoreText());
    const scale = () => page.evaluate(()=>window.doubleText());
    const visibleBounds = async (selector,label) => {const el=page.locator(selector).filter({visible:true}).first(); await el.scrollIntoViewIfNeeded(); const box=await el.boundingBox(); assert.ok(box && box.x>=0 && box.x+box.width<=720 && box.y>=0 && box.y+box.height<=560,`${label}: action bounds ${JSON.stringify(box)}`);};
    for(const lang of ['da','en']) {
      await restore(); await locale(lang); await page.setViewportSize({width:1280,height:820});
      await page.screenshot({path:path.join(shots,`${lang}-editor-1280x820.png`)});
      await action('tx-who-open').click(); await page.screenshot({path:path.join(shots,`${lang}-who-1280x820.png`)}); await page.locator('dialog [data-action="close-modal"]').first().click();
      await action('tx-export').click(); await page.screenshot({path:path.join(shots,`${lang}-export-1280x820.png`)}); await page.locator('dialog [data-action="close-modal"]').first().click();
      await page.setViewportSize({width:720,height:560}); await scale();
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),true,`${lang}: editor page overflow`);
      for(const control of ['tx-rewind','tx-play','tx-forward']) await visibleBounds(`[data-action="${control}"]`,`${lang} ${control}`); await page.evaluate(()=>document.getElementById('toast').hidden=true); await page.screenshot({path:path.join(shots,`${lang}-editor-720-200.png`)});
      await restore(); await action('tx-who-open').click(); await scale();
      assert.ok(await page.locator('dialog').evaluate(el=>el.scrollWidth<=el.clientWidth+1),`${lang}: Who dialog overflow`);
      await visibleBounds('[data-action="tx-who-run"]',`${lang} Who`); const countSpacing=await page.evaluate(()=>{const l=document.querySelector('[for=tx-who-count]').getBoundingClientRect(),s=document.querySelector('#tx-who-count').getBoundingClientRect();return s.top>=l.bottom}); assert.ok(countSpacing,`${lang}: count label and control overlap`); await page.screenshot({path:path.join(shots,`${lang}-who-720-200.png`)});
      await visibleBounds('dialog #dialog-title',`${lang} Who title`);
      await visibleBounds('dialog [data-action="close-modal"]',`${lang} Who close`);
      await page.locator('dialog [data-action="close-modal"]').first().click(); await restore();
      await action('tx-export').click(); await scale();
      assert.ok(await page.locator('dialog').evaluate(el=>el.scrollWidth<=el.clientWidth+1),`${lang}: export dialog overflow`);
      await visibleBounds('[data-action="tx-export-save"]',`${lang} export`); await page.screenshot({path:path.join(shots,`${lang}-export-720-200.png`)});
      await visibleBounds('dialog #dialog-title',`${lang} export title`);
      await visibleBounds('dialog [data-action="close-modal"]',`${lang} export close`);
      await page.locator('dialog [data-action="close-modal"]').first().click(); await restore();
    }
    assert.equal(await page.evaluate(()=>JSON.stringify(calls).includes('__vara_')),false,'no binding tokens in host payloads');
    assert.deepEqual(errors,[]); console.log(`Import, jobs, editor/player, AI, Who, dialogs, speaker provenance and export locale checks passed. Screenshots: ${shots}`);
  } finally {if(browser) await browser.close(); await new Promise(resolve=>server.close(resolve));}
})().catch(error=>{console.error(error);process.exitCode=1});
