/* Capture only: reuse fictional app fixture and exact polished serializer. No browser prototype. */
const fs=require('fs'),path=require('path'),{createRequire}=require('module');
fs.mkdirSync(path.join(__dirname,'references'),{recursive:true});
const dir=__dirname,base=path.resolve(dir,'../..'),localRequire=createRequire(path.join(base,'capture.cjs'));
const {webkit}=require(process.env.PLAYWRIGHT_MODULE||'/Users/mikkelfreltoftkrogsholm/.bun/install/global/node_modules/playwright');
(async()=>{
 const browser=await webkit.launch({headless:true,executablePath:process.env.PLAYWRIGHT_WEBKIT||'/Users/mikkelfreltoftkrogsholm/Library/Caches/ms-playwright/webkit-2311/pw_run.sh'});const controls={};
 const metadata={engine:'WebKit',browserVersion:browser.version(),viewport:{width:1280,height:820},deviceScaleFactor:2,captures:[]};
 try{for(const filename of ['capture.cjs','capture-transcription.cjs']){
 const page=await browser.newPage({deviceScaleFactor:2,locale:'en-US',timezoneId:'Europe/Copenhagen'});await page.clock.install({fixedTime:new Date('2026-11-06T12:00:00Z')});
 const go=page.goto.bind(page);page.goto=async(...args)=>{const r=await go(...args);await page.addStyleTag({path:path.resolve(dir,'../proposal.css')});return r};
 await page.exposeFunction('captureControls',(name,data)=>{controls[name]=data});
 let source=fs.readFileSync(path.join(base,filename),'utf8');
 source=source.replace('const capture=async(name)=>{',`const capture=async(name)=>{
 await page.evaluate(async name=>{const modal=document.querySelector('dialog[open]');const root=modal||document;
 const controls=[...root.querySelectorAll('button,select,input,textarea,summary,[data-action],[data-page],.history-row,.app-row')].filter(e=>{const r=e.getBoundingClientRect(),s=getComputedStyle(e);return r.width>0&&r.height>0&&r.bottom>0&&r.top<820&&s.visibility!=='hidden'&&s.display!=='none'&&!e.closest('[hidden]')}).map(e=>{let r=e.getBoundingClientRect();let l=Math.max(0,r.left),t=Math.max(0,r.top),right=Math.min(1280,r.right),bottom=Math.min(820,r.bottom);for(let p=e.parentElement;p&&p!==document.body;p=p.parentElement){const s=getComputedStyle(p);if(/auto|scroll|hidden|clip/.test(s.overflowY)){const q=p.getBoundingClientRect();t=Math.max(t,q.top);bottom=Math.min(bottom,q.bottom)}}return {tag:e.tagName,id:e.id,action:e.dataset.action,page:e.dataset.page,dataId:e.dataset.id,provider:e.dataset.provider,label:(e.getAttribute('aria-label')||e.getAttribute('title')||e.innerText||e.value||'').trim().slice(0,180),type:e.type,disabled:e.disabled||false,classes:e.className,rect:[l,t,right-l,bottom-t],options:e.tagName==='SELECT'?[...e.options].map(o=>({value:o.value,label:o.text,disabled:o.disabled})):undefined}}).filter(e=>e.rect[2]>0&&e.rect[3]>0);
 await window.captureControls(name,{modal:!!modal,controls});},name);`);
 if(filename==='capture.cjs')source=source.replace("await capture('04-dictation');",`await capture('04-dictation');await page.evaluate(()=>{window.testState.settings.general_ai_enabled=true;window.testEmit()});await capture('42-ai-defaults-enabled');await page.evaluate(()=>{window.testState.settings.general_ai_enabled=false;window.testEmit()});`);
 if(filename==='capture.cjs'){
 source=source.replace("await capture('02-original');",`await capture('02-original');await page.locator('[data-action="select-result"][data-id="2"]').click();await capture('43-history-selection');await page.locator('[data-action="select-result"][data-id="1"]').click();`);
 source=source.replace("await capture('03-apps');",`await capture('03-apps');await page.locator('[data-action="select-pref"][data-id="com.apple.Notes"]').click();await capture('44-app-notes');await page.locator('[data-action="select-pref"][data-id="com.apple.mail"]').click();`);
 source=source.replace("await capture('26-dictation-permissions');",`await capture('26-dictation-permissions');await page.evaluate(()=>{window.testState.settings.general_ai_enabled=true;window.testEmit()});await page.locator('#workspace').evaluate(e=>e.scrollTop=e.scrollHeight);await capture('42-ai-defaults-enabled');await page.evaluate(()=>{window.testState.settings.general_ai_enabled=false;window.testEmit()});`);
} else {
 source=source.replace("await capture('11-transcription-import');",`await capture('11-transcription-import');await page.locator('.tx-start').scrollIntoViewIfNeeded();await capture('45-transcription-import-actions');`);
 source=source.replace("await capture('24-transcription-ai');",`await capture('24-transcription-ai');await page.locator('[data-action="tx-ai"]').click();await page.locator('.tx-ai-result').waitFor();await page.locator('.tx-ai-result').scrollIntoViewIfNeeded();await capture('46-transcription-ai-result');`);
}
const run=new Function('require','return ('+source+')')(localRequire);const outputs=await run(page,dir);
 for(const [name,data]of Object.entries(outputs)){fs.writeFileSync(path.join(dir,'svg',name+'.svg'),data.main);fs.writeFileSync(path.join(dir,'svg',name+'.assets.json'),JSON.stringify(data.assets));metadata.captures.push(name)}await page.close();
 }fs.writeFileSync(path.join(dir,'controls.json'),JSON.stringify(controls,null,2));fs.writeFileSync(path.join(dir,'capture-metadata.json'),JSON.stringify(metadata,null,2));}finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
