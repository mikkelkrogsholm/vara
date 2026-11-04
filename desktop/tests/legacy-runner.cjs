// Runs the existing browser regressions against a local, isolated fixture server.
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const fs=require('node:fs'),http=require('node:http'),path=require('node:path');
const root=path.resolve(__dirname,'../..'),desktop=path.join(root,'desktop');
const output=process.env.VARA_TEST_OUTPUT_ROOT||'/tmp/vara-22b-legacy';fs.mkdirSync(path.join(output,'gdd/reports'),{recursive:true});process.env.VARA_TEST_OUTPUT_ROOT=output;
const server=http.createServer((req,res)=>{const url=new URL(req.url,'http://localhost').pathname;let file;
 if(url==='/tests/runtime-catalog.json')file=path.join(root,'crates/vara-core/messages/runtime.json');
 else if(url.startsWith('/ui/design/'))file=path.join(root,'gdd/design',url.slice('/ui/design/'.length));
 else file=path.resolve(desktop,'.'+url);
 if(!file.startsWith(root+path.sep))return res.writeHead(403).end();
 fs.readFile(file,(err,data)=>{if(err)return res.writeHead(404).end();res.setHeader('Content-Type',({'.js':'text/javascript','.css':'text/css','.html':'text/html','.json':'application/json','.wav':'audio/wav'})[path.extname(file)]||'application/octet-stream');res.end(data);});});
(async()=>{await new Promise(r=>server.listen(18770,'127.0.0.1',r));let browser;try{
 browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM?{executablePath:process.env.PLAYWRIGHT_CHROMIUM}:{})});
 for(const name of ['ui.playwright.js','transcription.playwright.js']){const page=await browser.newPage();const run=eval('('+fs.readFileSync(path.join(__dirname,name),'utf8')+')');const checks=await run(page);console.log(`${name}: ${checks.length} checks passed`);await page.close();}
 }finally{if(browser)await browser.close();await new Promise(r=>server.close(r));}})().catch(e=>{console.error(e);process.exitCode=1});
