/* Draft-only adapter: reuses baseline fictional host, serializer and state transitions. */
const fs=require('fs'), path=require('path'), {createRequire}=require('module');
const base=path.resolve(__dirname,'..'), localRequire=createRequire(path.join(base,'capture.cjs'));
const selected=['01-history','03-apps','04-dictation','37-recording-controls','39-local-models','29-api-providers','36-privacy-audio','11-transcription-import','12-transcription-editor','24-transcription-ai','22-key-dialog','13-who-download'];
const {webkit}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
(async()=>{
 const browser=await webkit.launch({headless:true,...(process.env.PLAYWRIGHT_WEBKIT?{executablePath:process.env.PLAYWRIGHT_WEBKIT}:{})});
 const metadata={engine:'WebKit',browserVersion:browser.version(),viewport:{width:1280,height:820},deviceScaleFactor:2,locale:'en-US',timezone:'Europe/Copenhagen',fixedTime:'2026-11-06T12:00:00Z',captures:[]};
 try {
  for(const filename of ['capture.cjs','capture-transcription.cjs']) {
   const page=await browser.newPage({deviceScaleFactor:2,locale:'en-US',timezoneId:'Europe/Copenhagen'});
   await page.clock.install({fixedTime:new Date(metadata.fixedTime)});
   const go=page.goto.bind(page);page.goto=async(...args)=>{const r=await go(...args);if(!process.env.DRAFT_CONTROL)await page.addStyleTag({path:path.join(__dirname,'proposal.css')});return r};
   let source=fs.readFileSync(path.join(base,filename),'utf8').replace('const capture=async(name)=>{','const capture=async(name)=>{ if(!'+JSON.stringify(process.env.DRAFT_CONTROL?['01-history','12-transcription-editor']:selected)+'.includes(name)) return;').replaceAll("out+'/references/'","out+'/render-evidence/'");
   if(process.env.DRAFT_CONTROL) source=source.replaceAll("out+'/render-evidence/'","out+'/render-evidence/control-'");
   const run=new Function('require','return ('+source+')')(localRequire);
   const outputs=await run(page,__dirname);
   metadata.systemMedia=await page.evaluate(()=>({colorScheme:matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light',forcedColors:matchMedia('(forced-colors: active)').matches,devicePixelRatio,platform:document.documentElement.dataset.platform}));
   for(const [name,data] of Object.entries(outputs)){metadata.captures.push(name);if(process.env.DRAFT_CONTROL)continue;fs.writeFileSync(path.join(__dirname,'svg',name+'.svg'),data.main);fs.writeFileSync(path.join(__dirname,'svg',name+'.assets.json'),JSON.stringify(data.assets));}
   console.log(filename+': '+Object.keys(outputs).length+' draft captures');
   await page.close();
  }
  fs.writeFileSync(path.join(__dirname,process.env.DRAFT_CONTROL?'control-metadata.json':'capture-metadata.json'),JSON.stringify(metadata,null,2)+'\n');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
