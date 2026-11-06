/* Bounded DOM geometry/scroll checks against the browser proposal. Not a native or WCAG audit. */
const fs=require('fs'),path=require('path'),{createRequire}=require('module');
const base=path.resolve(__dirname,'..'),req=createRequire(path.join(base,'capture.cjs'));
const selected=Object.keys(JSON.parse(fs.readFileSync(path.join(__dirname,'frames.json'))));
const {webkit}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const results=[];
(async()=>{
 const browser=await webkit.launch({headless:true,...(process.env.PLAYWRIGHT_WEBKIT?{executablePath:process.env.PLAYWRIGHT_WEBKIT}:{})});
 try{
 for(const file of ['capture.cjs','capture-transcription.cjs']){
  const page=await browser.newPage({deviceScaleFactor:2,locale:'en-US',timezoneId:'Europe/Copenhagen'});
  await page.clock.install({fixedTime:new Date('2026-11-06T12:00:00Z')});
  const go=page.goto.bind(page);page.goto=async(...a)=>{const r=await go(...a);await page.addStyleTag({path:path.join(__dirname,'proposal.css')});return r};
  const check=async(name)=>{
   if(!selected.includes(name))return;
   for(const size of [{width:1280,height:820},{width:1000,height:760}]){
    await page.setViewportSize(size);
    const record=await page.evaluate(({name,size})=>{
     const box=e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height,right:r.right,bottom:r.bottom}};
     const vis=e=>{const r=e.getBoundingClientRect();return r.width&&r.height&&getComputedStyle(e).visibility!=='hidden'&&e.closest('[hidden]')===null};
     const failures=[],measurements={};
     const lab=document.querySelector('label:has(#update-automatic)'),button=document.querySelector('[data-action="update-check"]');
     if(lab&&button){const a=box(lab),b=box(button);const gap=b.x>=a.right?b.x-a.right:b.y-a.bottom;measurements.updateLabelToButton=gap;if(gap<12-0.1)failures.push('update control gap');}
     const fields=[...document.querySelectorAll('.field')].filter(vis);
     measurements.fields=fields.map(f=>{const label=f.querySelector('label,.field-label'),c=f.querySelector('input:not([type=hidden]),select,textarea,.shortcut-control');if(!label||!c)return null;const a=box(label),b=box(c);const gap=b.y-a.bottom;if(gap<7.9)failures.push('field gap '+c.id);return {id:c.id,gap}}).filter(Boolean);
     const toggle=document.querySelector('#app-form > .toggle-row'),prompt=document.querySelector('#app-prompt')?.closest('.field');if(toggle&&prompt){const a=box(toggle),b=box(prompt);measurements.toggleToPrompt=b.y-a.bottom;if(b.y-a.bottom<23.9)failures.push('toggle prompt group gap');}
     const close=document.querySelector('dialog[open] [data-action="close-modal"]'),title=document.querySelector('dialog[open] h2');
     if(close&&title){const a=box(close),range=document.createRange();range.selectNodeContents(title);const r=range.getBoundingClientRect(),b={x:r.x,y:r.y,right:r.right,bottom:r.bottom};measurements.closeTarget=[a.w,a.h];if(a.w<32||a.h<32)failures.push('close target');if(a.x<b.right&&a.right>b.x&&a.y<b.bottom&&a.bottom>b.y)failures.push('header close overlap');}
     const structuredRows=[...document.querySelectorAll('.model-row,.provider-connection .provider-row')].filter(vis);measurements.structuredRowsChecked=structuredRows.length;for(const row of structuredRows){const children=[...row.children].filter(vis).map(box);for(let i=0;i<children.length;i++)for(let j=i+1;j<children.length;j++){const a=children[i],b=children[j];if(a.x<b.right-.1&&a.right>b.x+.1&&a.y<b.bottom-.1&&a.bottom>b.y+.1)failures.push('structured row overlap');}}
     const row=document.querySelector('.history-row');if(row){measurements.historyListVisible=!!vis(row);measurements.historyDisposition=vis(row)?'visible list with persistent direct actions':'inherited responsive list-hidden selected-detail state';measurements.historyRowHeight=vis(row)?box(row).h:null;measurements.directActions=[...row.querySelectorAll('.icon-button')].map(e=>({action:e.dataset.action,...box(e)}));}
     const direct=[...document.querySelectorAll('.detail-pane .result-actions .icon-button')].filter(vis);measurements.visibleResultActions=direct.map(e=>({action:e.dataset.action,...box(e)}));if(name==='01-history'&&!vis(row)&&(!direct.length||direct.some(e=>box(e).w<32||box(e).h<32)))failures.push('visible result direct actions');
     return {name,size,measurements,failures};
    },{name,size});
    results.push(record);
    if(size.width===1000&&['04-dictation','03-apps','13-who-download'].includes(name))await page.screenshot({path:path.join(__dirname,'render-evidence',name+'-1000.png')});
   }
   if(['04-dictation','03-apps','13-who-download'].includes(name)){
    await page.evaluate(()=>{window.draftOriginals=[];for(const e of document.querySelectorAll('label:has(#update-automatic),#app-form .toggle-row label,dialog[open] h2')){window.draftOriginals.push([e,e.innerHTML]);if(e.matches('label:has(#update-automatic)'))e.append(' — Kontrollér automatisk efter tilgængelige opdateringer på denne computer');else e.append(' — Indstillinger for automatisk tekstbehandling og transskription');}});
    const style=await page.addStyleTag({content:'body *{line-height:1.5!important;letter-spacing:.12em!important;word-spacing:.16em!important}p{margin-bottom:2em!important}'});
    const stress=await page.evaluate(name=>{
     const d=document.querySelector('dialog[open]'),root=d||document.querySelector('#workspace');
     const lab=document.querySelector('label:has(#update-automatic)'),b=document.querySelector('[data-action="update-check"]');let gap=null;
     if(lab&&b){const a=lab.getBoundingClientRect(),r=b.getBoundingClientRect();gap=r.x>=a.right?r.x-a.right:r.y-a.bottom;}
     root.scrollTop=root.scrollHeight;const endControl=d?.querySelector('[data-action=tx-who-download]'),end=endControl?.getBoundingClientRect(),viewport=root.getBoundingClientRect();
     return {legalActionReachable:end?end.bottom<=viewport.bottom&&end.top>=viewport.top:null,name,kind:'1000×760 Danish-label + WCAG text-spacing override stress',updateLabelToButton:gap,scroll:{clientHeight:root.clientHeight,scrollHeight:root.scrollHeight,canReachEnd:root.scrollTop+root.clientHeight>=root.scrollHeight-1},legalCopyPreserved:d?d.textContent.includes('Desert Ant Labs'):null,failures:gap!==null&&gap<11.9?['update overlap stress']:[]};
    },name);results.push(stress);
    if(name==='13-who-download')await page.screenshot({path:path.join(__dirname,'render-evidence',name+'-stress-end.png')});
    await page.evaluate(()=>{const d=document.querySelector('dialog[open]');if(d)d.scrollTop=0;else document.querySelector('#workspace').scrollTop=0;document.querySelector('#update-automatic')?.closest('.settings-section').scrollIntoView({block:'start'});});
    await page.screenshot({path:path.join(__dirname,'render-evidence',name+'-stress.png')});
    if(name==='04-dictation'){const t=page.locator('.settings-form input:not([disabled]):visible').last();await t.focus();await t.evaluate(e=>e.scrollIntoView({block:'center'}));results.push(await page.evaluate(()=>{const a=document.activeElement.getBoundingClientRect(),f=document.querySelector('.sticky-save').getBoundingClientRect();return {name:'04-dictation',kind:'stress focused visible control',focusedId:document.activeElement.id,clearance:f.top-a.bottom,failures:a.bottom>f.top?['stress focused control obscured']:[]}}));}
    await style.evaluate(e=>e.remove());await page.evaluate(()=>{for(const [e,html]of window.draftOriginals)e.innerHTML=html});
   }
   if(['04-dictation','03-apps','13-who-download'].includes(name)){
    await page.evaluate(()=>{window.draftFontStyles=[...document.querySelectorAll('body *')].map(e=>[e,e.getAttribute('style'),parseFloat(getComputedStyle(e).fontSize)]);for(const[e,,size]of window.draftFontStyles)e.style.fontSize=(size*1.5)+'px';const root=document.querySelector('dialog[open]')||document.querySelector('#workspace');root.scrollTop=0;document.querySelector('#update-automatic')?.closest('.settings-section').scrollIntoView({block:'start'});});
    const large=await page.evaluate(name=>{
     const root=document.querySelector('dialog[open]')||document.querySelector('#workspace'),failures=[];
     const labels=[...document.querySelectorAll('.field')].filter(e=>e.getBoundingClientRect().height>0);for(const f of labels){const label=f.querySelector('label,.field-label'),control=f.querySelector('input:not([type=hidden]),select,textarea,.shortcut-control');if(label&&control&&control.getBoundingClientRect().top<label.getBoundingClientRect().bottom)failures.push('large text field overlap');}
     const label=document.querySelector('label:has(#update-automatic)'),b=document.querySelector('[data-action="update-check"]');let updateGap=null;if(label&&b){const a=label.getBoundingClientRect(),r=b.getBoundingClientRect();updateGap=r.x>=a.right?r.x-a.right:r.y-a.bottom;if(updateGap<11.9)failures.push('large text update overlap');}
     const d=document.querySelector('dialog[open]'),c=d?.querySelector('[data-action=close-modal]'),t=d?.querySelector('h2');if(c&&t){const r=document.createRange();r.selectNodeContents(t);const a=r.getBoundingClientRect(),b=c.getBoundingClientRect();if(a.x<b.right&&a.right>b.x&&a.y<b.bottom&&a.bottom>b.y)failures.push('large text title close overlap');}
     return {name,kind:'150% computed font size at1000×760',actualFontSize:parseFloat(getComputedStyle(document.querySelector('.field label')||t).fontSize),updateGap,failures};
    },name);results.push(large);
    await page.screenshot({path:path.join(__dirname,'render-evidence',name+'-large-text.png')});
    await page.evaluate(()=>{const r=document.querySelector('dialog[open]')||document.querySelector('#workspace');r.scrollTop=r.scrollHeight;});
    results.push(await page.evaluate(name=>{const sidebar=document.querySelector('.sidebar');sidebar.scrollTop=sidebar.scrollHeight;const dictate=sidebar.querySelector('.sidebar-dictation-controls').getBoundingClientRect(),side=sidebar.getBoundingClientRect(),sidebarActionReachable=dictate.bottom<=side.bottom&&dictate.top>=side.top;const root=document.querySelector('dialog[open]')||document.querySelector('#workspace'),last=document.querySelector('dialog[open] [data-action=tx-who-download]')||document.querySelector('#app-form [data-action=save-app]')||document.querySelector('.sticky-save [data-action=save-settings]'),r=last.getBoundingClientRect(),v=root.getBoundingClientRect();return{name,kind:'150% final action reachability',sidebarActionReachable,reachable:r.bottom<=v.bottom&&r.top>=v.top,failures:r.bottom>v.bottom||r.top<v.top||!sidebarActionReachable?['large text final action unreachable']:[]}},name));
    await page.screenshot({path:path.join(__dirname,'render-evidence',name+'-large-text-end.png')});
    if(name==='04-dictation'){const t=page.locator('.settings-form input:not([disabled]):visible').last();await t.focus();await t.evaluate(e=>e.scrollIntoView({block:'center'}));results.push(await page.evaluate(()=>{const a=document.activeElement.getBoundingClientRect(),f=document.querySelector('.sticky-save').getBoundingClientRect();return{name:'04-dictation',kind:'150% focused visible control',focusedId:document.activeElement.id,clearance:f.top-a.bottom,failures:a.bottom>f.top?['large text focus obscured']:[]}}));}
    await page.evaluate(()=>{for(const[e,style]of window.draftFontStyles){if(style===null)e.removeAttribute('style');else e.setAttribute('style',style)}});
   }
   if(['04-dictation','37-recording-controls','36-privacy-audio'].includes(name)){
    const target=page.locator('.settings-form input:not([disabled]):visible,.settings-form select:not([disabled]):visible,.settings-form textarea:not([disabled]):visible').last();
    if(await target.count()){
     await target.focus();await target.evaluate(e=>e.scrollIntoView({block:'center'}));
     results.push(await page.evaluate(name=>{const a=document.activeElement.getBoundingClientRect(),footer=document.querySelector('.sticky-save').getBoundingClientRect();return {name,kind:'last visible control focused after scroll',focusedId:document.activeElement.id,controlBottom:a.bottom,footerTop:footer.top,clearance:footer.top-a.bottom,failures:a.bottom>footer.top?['focused control obscured']:[]}},name));
    }
   }
   await page.setViewportSize({width:1280,height:820});
  };
  let source=fs.readFileSync(path.join(base,file),'utf8');const a=source.indexOf(' const capture=async(name)=>{'),b=source.indexOf('\n };',a);
  source=source.slice(0,a)+' const capture=async(name)=>{await check(name);'+source.slice(b);
  const run=new Function('require','check','return ('+source+')')(req,check);await run(page,__dirname);await page.close();
 }
 fs.writeFileSync(path.join(__dirname,'checks.json'),JSON.stringify({browserVersion:browser.version(),checks:results},null,2)+'\n');
 const failures=results.flatMap(x=>x.failures);console.log(results.length+' bounded checks; failures: '+JSON.stringify(failures));if(failures.length)process.exitCode=1;
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
