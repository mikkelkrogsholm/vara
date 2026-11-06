/* Verify production CSS using unchanged approved fixture flows and bounded checks.
 * Never loads proposal.css. All new output stays in implementation/. */
const fs=require('fs'),path=require('path'),{createRequire}=require('module');
const polish=path.resolve(__dirname,'..'),base=path.resolve(polish,'..');
const req=createRequire(path.join(base,'capture.cjs'));
const selected=Object.keys(JSON.parse(fs.readFileSync(path.join(polish,'frames.json'))));
const {webkit}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
(async()=>{
 const browser=await webkit.launch({headless:true,...(process.env.PLAYWRIGHT_WEBKIT?{executablePath:process.env.PLAYWRIGHT_WEBKIT}:{})});
 const metadata={engine:'WebKit',browserVersion:browser.version(),viewport:{width:1280,height:820},deviceScaleFactor:2,locale:'en-US',timezone:'Europe/Copenhagen',fixedTime:'2026-11-06T12:00:00Z',styles:'canonical production CSS, no injected proposal',captures:[]};
 try{
  for(const file of ['capture.cjs','capture-transcription.cjs']){
   const page=await browser.newPage({deviceScaleFactor:2,locale:'en-US',timezoneId:'Europe/Copenhagen'});
   await page.clock.install({fixedTime:new Date(metadata.fixedTime)});
   let source=fs.readFileSync(path.join(base,file),'utf8');
   const a=source.indexOf(' const capture=async(name)=>{'),b=source.indexOf('\n };',a);
   if(a<0||b<0)throw Error('Fixture capture boundary changed');
   source=source.slice(0,a)+` const capture=async(name)=>{if(!${JSON.stringify(selected)}.includes(name))return;await page.screenshot({path:out+'/render-evidence/'+name+'.png'});outputs[name]=true;`+source.slice(b);
   const run=new Function('require','return ('+source+')')(req);
   metadata.captures.push(...Object.keys(await run(page,__dirname)));
   await page.close();
  }
  fs.writeFileSync(path.join(__dirname,'capture-metadata.json'),JSON.stringify(metadata,null,2)+'\n');
 }finally{await browser.close()}
 // Reuse approved geometry, focus and stress checks with production styles only.
 let check=fs.readFileSync(path.join(polish,'check.cjs'),'utf8');
 check=check.replace("path.resolve(__dirname,'..')",JSON.stringify(base));
 check=check.replace("path.join(__dirname,'frames.json')",JSON.stringify(path.join(polish,'frames.json')));
 check=check.replace("  const go=page.goto.bind(page);page.goto=async(...a)=>{const r=await go(...a);await page.addStyleTag({path:path.join(__dirname,'proposal.css')});return r};",'');
 if(check.includes('proposal.css'))throw Error('Proposal injection was not removed');
 await new Function('require','__dirname',check.replace('(async()=>{','return (async()=>{'))(req,__dirname);
})().catch(e=>{console.error(e);process.exitCode=1});
