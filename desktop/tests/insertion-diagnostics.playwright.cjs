// Deterministic browser/IPC fixtures only: never invokes native permissions or insertion.
const {chromium} = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs'), path = require('node:path'), http = require('node:http'), assert = require('node:assert/strict');
const desktop = path.resolve(__dirname, '..');
const fixture = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/ui-state.json')));
const catalog = JSON.parse(fs.readFileSync(path.join(__dirname, '../../crates/vara-core/messages/runtime.json')));
const shots = process.env.VARA_INSERTION_SHOTS || '/tmp/vara-insertion-ui-screens';
fs.mkdirSync(shots, {recursive:true});
const server = http.createServer((req,res) => {
  const file = path.resolve(desktop, '.' + new URL(req.url, 'http://localhost').pathname);
  if (!file.startsWith(desktop + path.sep)) return res.writeHead(403).end();
  fs.readFile(file, (err,data) => {
    if (err) return res.writeHead(404).end();
    res.setHeader('Content-Type', ({'.js':'text/javascript','.html':'text/html','.css':'text/css'})[path.extname(file)] || 'application/octet-stream');
    res.end(data);
  });
});
(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await chromium.launch({headless:true, ...(process.env.PLAYWRIGHT_CHROMIUM ? {executablePath:process.env.PLAYWRIGHT_CHROMIUM} : {})});
    const page = await browser.newPage({viewport:{width:1440,height:950}}), errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.route('**/*', route => new URL(route.request().url()).hostname === '127.0.0.1' ? route.continue() : route.abort());
    await page.addInitScript(({fixture,catalog}) => {
      window.state = fixture; window.calls = []; window.events = {};
      state.settings.ui_locale = 'en'; state.resolved_ui_locale = 'en'; state.ui_locale_revision = 0;
      state.message = null; state.session = null; state.recoveries = [];
      window.__TAURI__ = {
        core:{invoke:async (_, {action,payload}) => {
          calls.push({action,payload});
          switch (action) {
            case 'runtime_catalog': return catalog;
            case 'snapshot': return structuredClone(state);
            case 'devices': return [];
            case 'file_list': return {documents:[],media:{}};
            case 'application_icons': return {};
          }
          return null;
        }},
        event:{listen:async (name,callback) => {events[name] = callback; return () => {};}}
      };
      window.emit = () => events['vara:state']({payload:structuredClone(state)});
    }, {fixture,catalog});
    await page.goto(`http://127.0.0.1:${server.address().port}/ui/index.html`);
    await page.locator('.history-row').first().waitFor();
    const mutate = async changes => {
      await page.evaluate(changes => {
        Object.assign(state.history[0], changes);
        if (changes.insertion_diagnostic === null) delete state.history[0].insertion_diagnostic;
        emit();
      }, changes);
    };
    const locale = language => page.evaluate(language => {state.resolved_ui_locale=language;state.settings.ui_locale=language;Vara.i18n.setLocale(language);}, language);
    const text = () => page.locator('.insertion-feedback').innerText();
    const noTokens = async () => assert.equal(await page.evaluate(() => /__vara_|insertion\.(?:reason|row|access)\./.test(document.body.innerText)), false);
    const reasonPatterns = {
      accessibility_missing: [/Accessibility access/, /Tilgængelighed/],
      capture_failed: [/could not capture/, /kunne ikke registrere/],
      target_gone: [/no longer available/, /ikke længere tilgængeligt/],
      field_unavailable: [/could not accept/, /kunne ikke modtage/],
      app_changed: [/active app changed/, /aktive app blev ændret/],
      focus_changed: [/focused text field changed/, /aktive tekstfelt blev ændret/],
      unsupported_role: [/does not support automatic/, /understøtter ikke automatisk/],
      post_event_access_missing: [/Keyboard insertion was blocked/, /Tastaturindsættelse blev blokeret/],
      dispatch_failed: [/could not send/, /kunne ikke sende/],
      not_attempted: [/was not attempted/, /blev ikke forsøgt/],
      platform_unsupported: [/unavailable on this platform/, /ikke tilgængelig på denne platform/],
    };
    for (const [languageIndex,language] of ['en','da'].entries()) {
      await locale(language);
      for (const [reason,patterns] of Object.entries(reasonPatterns)) {
        await mutate({delivery:'manual_ready',insertion_diagnostic:{reason,method:'accessibility',os_error:-25204,token:'PRIVATE_TOKEN',text:'PRIVATE_TEXT',window_title:'PRIVATE_TITLE'}});
        assert.match(await text(), patterns[languageIndex], `${language}/${reason}`);
        assert.equal(await page.locator('.insertion-feedback [data-action=permission]').count(), ['accessibility_missing','post_event_access_missing'].includes(reason) ? 1 : 0);
        assert.doesNotMatch(await text(), /PRIVATE_/);
        await noTokens();
      }
      // The delivery wins over every diagnostic, including revoked access or changed focus.
      for (const reason of [...Object.keys(reasonPatterns), 'handoff_unconfirmed', 'confirmed']) {
        await mutate({delivery:'unknown',insertion_diagnostic:{reason,method:'keyboard',os_error:null}});
        assert.match(await page.locator('.insertion-feedback .notice--warning').innerText(), language === 'en' ? /may already have been sent.*Check.*before pasting again/s : /kan allerede være sendt.*Tjek.*før du indsætter igen/s);
        const row = await page.locator('.row-select').first().innerText();
        assert.match(row, language === 'en' ? /Insertion unconfirmed/ : /Kontrollér indsættelse/);
        assert.doesNotMatch(row, /Ready to insert|Klar til manuel/);
      }
      await mutate({delivery:'inserted',insertion_diagnostic:{reason:'confirmed',method:'accessibility',os_error:0}});
      assert.equal(await page.locator('.insertion-feedback').count(), 0);
      await mutate({delivery:'pending',insertion_diagnostic:null});
      assert.match(await text(), language === 'en' ? /still pending/ : /afventer stadig/);
      await mutate({delivery:'manual_ready',insertion_diagnostic:null});
      assert.match(await text(), language === 'en' ? /ready to copy.*No insertion details/s : /klar til at blive kopieret.*ingen oplysninger/s);
      assert.equal(await page.locator('.insertion-details').count(), 0);
      await mutate({delivery:'manual_ready',insertion_diagnostic:{reason:'PRIVATE_REASON',method:'PRIVATE_METHOD',os_error:'PRIVATE_ERROR',token:'PRIVATE_TOKEN'}});
      assert.equal(await page.locator('.insertion-details').count(), 0);
      assert.doesNotMatch(await text(), /PRIVATE_/);
      // Metadata never enters the DOM, even attributes. Only closed codes and i32 values can appear.
      await mutate({delivery:'manual_ready',insertion_diagnostic:{reason:'field_unavailable',method:'accessibility',os_error:-25204,window_title:'PRIVATE_TITLE'}});
      await page.locator('.insertion-details summary').click();
      assert.equal(await page.locator('.insertion-details p').innerText(), 'field_unavailable · accessibility · AX -25204');
      assert.equal(await page.locator('.insertion-details summary').innerText(), language === 'en' ? 'Insertion details' : 'Oplysninger om indsættelse');
      assert.doesNotMatch(await page.content(), /PRIVATE_/);
      await page.screenshot({path:path.join(shots, `${language}-manual.png`)});
      await mutate({delivery:'unknown',insertion_diagnostic:{reason:'focus_changed',method:'keyboard',os_error:null}});
      await page.screenshot({path:path.join(shots, `${language}-unknown.png`)});
      await noTokens();
    }
    // Copy and text selection retain their existing behavior; no insertion retries occur.
    await page.evaluate(() => {
      const range = document.createRange(); range.selectNodeContents(document.querySelector('.detail-pane .transcript'));
      getSelection().removeAllRanges(); getSelection().addRange(range);
      window.selectedText = getSelection().toString();
    });
    await locale('en');
    assert.equal(await page.evaluate(() => getSelection().toString() === selectedText), true);
    const selected = await page.locator('.row-select[aria-pressed=true]').getAttribute('data-id');
    await page.locator('.history-row [data-action=copy]').last().click();
    assert.equal(await page.locator('.row-select[aria-pressed=true]').getAttribute('data-id'), selected);
    await page.locator('.copy-bar [data-action=copy]').click();
    assert.equal(await page.evaluate(() => calls.filter(c => c.action === 'copy').length), 2);
    assert.equal(await page.evaluate(() => calls.some(c => c.action === 'permission')), false);
    assert.equal(await page.evaluate(() => calls.some(c => /retry|insert|start|clipboard/.test(c.action))), false);
    // Recovery only requests the exact permission after an explicit click.
    await mutate({delivery:'manual_ready',insertion_diagnostic:{reason:'accessibility_missing'}});
    await page.locator('.insertion-feedback [data-action=permission]').click();
    await mutate({delivery:'unknown',insertion_diagnostic:{reason:'post_event_access_missing'}});
    await page.locator('.insertion-feedback [data-action=permission]').click();
    assert.deepEqual(await page.evaluate(() => calls.filter(c => c.action === 'permission').map(c => c.payload)), [{kind:'accessibility'},{kind:'post_event'}]);
    await page.locator('.sidebar [data-page=settings]').click();
    const insertion = page.locator('.permission-grid .permission-row').last();
    const permissions = async change => page.evaluate(change => {Object.assign(state.permissions,change);emit();}, change);
    await permissions({automatic_insertion:true,accessibility:false,post_event_access:false});
    assert.equal(await insertion.locator('[data-kind=accessibility]').count(), 1);
    await insertion.locator('[data-action=permission]').click();
    await permissions({accessibility:true,post_event_access:false});
    assert.match(await insertion.innerText(), /Direct insertion through Accessibility is available.*Insertion by keyboard is blocked/s);
    assert.equal(await insertion.locator('[data-kind=post_event]').count(), 1);
    await page.screenshot({path:path.join(shots,'en-partial-permissions.png')});
    await insertion.locator('[data-action=permission]').click();
    await locale('da');
    assert.match(await insertion.innerText(), /Direkte indsættelse.*Tastaturindsættelse er blokeret/s);
    await page.screenshot({path:path.join(shots,'da-partial-permissions.png')});
    await locale('en');
    for (const missing of [undefined,null]) {
      await permissions({post_event_access:missing});
      assert.match(await insertion.innerText(), /version of Vara does not report access for insertion by keyboard/);
      assert.equal(await insertion.locator('[data-action=permission]').count(), 0);
    }
    await permissions({post_event_access:true});
    assert.match(await insertion.innerText(), /Accessibility and keyboard insertion access are granted/);
    await permissions({automatic_insertion:false,accessibility:true,post_event_access:null});
    assert.match(await insertion.innerText(), /ready to insert manually/);
    assert.equal(await insertion.locator('[data-action=permission]').count(), 0);
    assert.equal(await insertion.locator('.permission-granted').count(), 0);
    // The same insertion row is shared by onboarding and leaves the microphone step available.
    await permissions({automatic_insertion:true,accessibility:false,post_event_access:false});
    await page.locator('.sidebar [data-page=history]').click();
    await page.evaluate(() => {state.history=[];emit();});
    await page.locator('[data-action=setup]').last().click();
    await page.locator('[data-action=setup-next]').click();
    assert.equal(await page.locator('.setup-page [data-kind=accessibility]').count(), 1);
    assert.equal(await page.locator('.setup-page [data-action=setup-next]').isEnabled(), true);
    assert.deepEqual(await page.evaluate(() => calls.filter(c => c.action === 'permission').map(c => c.payload)), [{kind:'accessibility'},{kind:'post_event'},{kind:'accessibility'},{kind:'post_event'}]);
    assert.equal(await page.evaluate(() => calls.some(c => /retry|insert|start|clipboard/.test(c.action))), false);
    await noTokens();
    assert.deepEqual(errors, []);
    console.log(`PASS: both languages, all closed reason families, authoritative Unknown, legacy/invalid metadata, partial/old-host permissions, explicit recovery actions, copy/selection and onboarding. Screenshots: ${shots}`);
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => {console.error(error);process.exitCode=1;});
