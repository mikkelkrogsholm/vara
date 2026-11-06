// Run from any directory with a local Playwright installation and Chromium.
const path = require('path');
const fs = require('fs');
const { webkit } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const dir = __dirname;
const metadata={engine:'WebKit',viewport:{width:1280,height:820},deviceScaleFactor:2,locale:'en-US',timezone:'Europe/Copenhagen',fixedTime:'2026-11-06T12:00:00Z',captures:[]};
(async () => {
  const browser = await webkit.launch({headless: true,...(process.env.PLAYWRIGHT_WEBKIT?{executablePath:process.env.PLAYWRIGHT_WEBKIT}:{})});
  try {
    for (const file of ['capture.cjs', 'capture-transcription.cjs']) {
      const page = await browser.newPage({deviceScaleFactor:2,locale:'en-US',timezoneId:'Europe/Copenhagen'});
      await page.clock.install({fixedTime:new Date('2026-11-06T12:00:00Z')});
      const run = eval('(' + fs.readFileSync(path.join(dir, file), 'utf8') + ')');
      const errors=[];page.on('pageerror',e=>errors.push(e.message));
      const captures = await run(page, dir);
      for (const [name, data] of Object.entries(captures)) {
        metadata.captures.push(name);
        fs.writeFileSync(path.join(dir, 'svg', name + '.svg'), data.main);
        fs.writeFileSync(path.join(dir, 'svg', name + '.assets.json'), JSON.stringify(data.assets));
      }
      metadata.browserVersion=browser.version();
      if(errors.length)throw Error(errors.join('\n'));
      console.log(file+': '+Object.keys(captures).length+' fresh frames; '+browser.version());
      await page.close();
    }
    fs.writeFileSync(path.join(dir,'capture-metadata.json'),JSON.stringify(metadata,null,2)+'\n');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
