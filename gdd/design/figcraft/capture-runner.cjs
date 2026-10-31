// Run from any directory with a local Playwright installation and Chromium.
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const dir = __dirname;
(async () => {
  const browser = await chromium.launch({headless: true, ...(process.env.PLAYWRIGHT_CHROMIUM ? {executablePath: process.env.PLAYWRIGHT_CHROMIUM} : {})});
  try {
    for (const file of ['capture.cjs', 'capture-transcription.cjs']) {
      const page = await browser.newPage();
      const run = eval('(' + fs.readFileSync(path.join(dir, file), 'utf8') + ')');
      const captures = await run(page, dir);
      for (const [name, data] of Object.entries(captures)) {
        fs.writeFileSync(path.join(dir, 'svg', name + '.svg'), data.main);
        fs.writeFileSync(path.join(dir, 'svg', name + '.assets.json'), JSON.stringify(data.assets));
      }
      await page.close();
    }
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
