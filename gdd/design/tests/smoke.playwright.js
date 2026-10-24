async function smoke(page) {
  const assert = (condition, message) => {
    if (!condition) throw new Error(message)
  }
  const checks = [],
    errors = []
  page.on("pageerror", (error) => errors.push(error.message))
  const act = (name) =>
    page.locator(`[data-action="${name}"]`).filter({ visible: true }).first()
  const click = (name) => act(name).click()
  const nav = (name) =>
    page.locator(`[data-action="navigate"][data-page="${name}"]`).click()
  const tab = (name) =>
    page.locator(`[data-action="settings-tab"][data-tab="${name}"]`).click()
  const openDemo = async (
    app = "mail",
    outcome = "normal",
    processing = "default",
  ) => {
    await click("open-demo")
    await page.locator("#demo-app").selectOption(app)
    await page.locator("#demo-case").selectOption(outcome)
    await page.locator("#demo-processing").selectOption(processing)
  }
  const session = async (app, outcome, processing) => {
    await openDemo(app, outcome, processing)
    await click("start-session")
    await click("stop-session")
    await page.waitForTimeout(1250)
  }
  await page.setViewportSize({ width: 1586, height: 992 })
  await page.goto("http://127.0.0.1:18769/design/index.html")
  assert(
    (await page.locator(".app-group").count()) === 3,
    "Three grouped originating apps",
  )
  assert(
    !(await page.locator(".original-disclosure").evaluate((e) => e.open)),
    "Original starts collapsed",
  )
  const columns = await page
    .locator(".sidebar,.history-pane,.detail-pane")
    .evaluateAll((nodes) =>
      nodes.map((n) => ({
        x: Math.round(n.getBoundingClientRect().x),
        width: Math.round(n.getBoundingClientRect().width),
      })),
    )
  assert(
    Math.abs(columns[1].x - 309) < 3 && Math.abs(columns[2].x - 807) < 3,
    "Reference column positions",
  )
  checks.push("Mockup column geometry and default disclosure")
  await page.locator(".original-disclosure summary").click()
  assert(await page.locator(".original-panel").isVisible(), "Original visible")
  await page.locator('[data-action="toggle-group"][data-app="mail"]').click()
  assert(
    (await page.locator(".transcript").textContent()) ===
      "Hej Anna. Tak for din mail. Jeg kan godt torsdag kl. 10. Send gerne en invitation.",
    "Collapsing group keeps selected result",
  )
  await page.locator('[data-action="toggle-group"][data-app="mail"]').click()
  await page.locator('[data-action="select-result"][data-id="4"]').click()
  assert(
    (await page.locator(".original-disclosure").count()) === 0,
    "Direct result has one representation",
  )
  assert(
    await page
      .locator('[data-action="select-result"][data-id="4"]')
      .evaluate((n) => n === document.activeElement),
    "Selection preserves keyboard focus",
  )
  checks.push(
    "Grouped history, original/direct representations and selection focus",
  )
  await click("toggle-search")
  await page.locator("#history-search").fill("zzzzzzz")
  assert(
    await page.getByRole("heading", { name: "Ingen match" }).isVisible(),
    "No match state",
  )
  await click("clear-search")
  await page.locator("#history-app").selectOption("code")
  assert((await page.locator(".app-group").count()) === 1, "Origin app filter")
  await page.locator("#history-app").selectOption("all")
  checks.push("Search, no-match and app filter")
  await nav("apps")
  await page.locator('[data-action="select-pref"][data-app="notes"]').click()
  await page.locator("#app-ai").check()
  await page.locator("#app-prompt").fill("Gør til korte noter.")
  await click("save-app")
  await page.locator("#app-prompt").fill("Ikke gemt endnu")
  await nav("settings")
  assert(await page.locator("#modal").isVisible(), "Unsaved guard")
  await click("close-modal")
  assert(
    (await page.locator("#app-prompt").inputValue()) === "Ikke gemt endnu",
    "Draft preserved",
  )
  await nav("settings")
  await click("discard-and-navigate")
  await nav("apps")
  assert(
    (await page.locator("#app-prompt").inputValue()) === "Gør til korte noter.",
    "Saved app preference preserved",
  )
  await page.locator("#app-prompt").fill("")
  await click("save-app")
  assert(
    (await page.locator("#app-prompt").getAttribute("aria-invalid")) === "true",
    "Invalid prompt marked",
  )
  await click("reset-app")
  checks.push("App AI save, unsaved edits and invalid draft")
  await nav("settings")
  await page.locator("#activation-style").selectOption("toggle")
  await click("record-shortcut")
  await page.locator("#shortcut-recorder").press("Control+q")
  await click("save-shortcut")
  assert(
    (await page.locator("#shortcut-error").textContent()) !== "",
    "Reserved shortcut rejects",
  )
  await click("close-modal")
  assert(
    (await page.locator("#nav-shortcut").textContent()) === "⌥ Space",
    "Previous shortcut retained",
  )
  await click("record-shortcut")
  await page.locator("#shortcut-recorder").press("Control+Shift+d")
  await click("save-shortcut")
  assert(
    (await page.locator("#nav-shortcut").textContent()) === "Ctrl Shift D",
    "Chosen shortcut",
  )
  checks.push("Activation mode and shortcut capture/conflict")
  await tab("privacy")
  await page.locator("#cleanup-toggle").check()
  await page.locator("#cleanup-days").fill("0")
  await click("save-retention")
  assert(
    (await page.locator("#retention-error").textContent()).includes("positivt"),
    "Retention validates",
  )
  await page.locator("#cleanup-days").fill("14")
  await click("save-retention")
  await tab("dictation")
  await tab("privacy")
  assert(
    (await page.locator("#cleanup-toggle").isChecked()) &&
      (await page.locator("#cleanup-days").inputValue()) === "14",
    "Retention saved",
  )
  checks.push("Opt-in retention and period validation")
  await tab("connections")
  await page.locator('[data-action="open-key"][data-provider="Groq"]').click()
  await page.locator("#api-key").fill("x")
  await click("save-key")
  assert(
    (await page.locator("#key-error").textContent()).includes("fiktiv"),
    "Fake key validation",
  )
  await page.locator("#api-key").fill("fake-demo-key")
  await click("close-modal")
  await page.locator('[data-action="open-key"][data-provider="Groq"]').click()
  assert(
    (await page.locator("#api-key").inputValue()) === "",
    "Secret cleared on close",
  )
  await page.locator("#api-key").fill("fake-demo-key")
  await click("save-key")
  checks.push("Key validation and secret clearing")
  await page.locator("#endpoint-name").fill("Min model")
  await page.locator("#endpoint-model").fill("model-1")
  await page.locator("#endpoint-url").fill("bad")
  await click("save-endpoint")
  assert(
    (await page.locator("#endpoint-error").textContent()).includes("gyldig"),
    "Endpoint validation",
  )
  await page.locator("#endpoint-url").fill("http://localhost:11434/v1")
  await click("save-endpoint")
  assert(
    (await page.locator(".settings-content").textContent()).includes(
      "Min model · model-1",
    ),
    "Endpoint saved",
  )
  await page.locator("#strict-local").check()
  await page.locator("#speech-choice").selectOption("cloud")
  assert(
    (await page.locator("#speech-choice").inputValue()) === "local",
    "Strict-local rejects cloud",
  )
  checks.push("Endpoint configuration and strict-local guard")
  await nav("history")
  await session("code", "normal", "default")
  assert(
    (await page.locator(".original-disclosure").count()) === 0,
    "Default direct session",
  )
  await session("mail", "normal", "direct")
  assert(
    (await page.locator(".original-disclosure").count()) === 0,
    "One-shot direct overrides app AI",
  )
  await session("code", "normal", "general")
  assert(
    (await page.locator(".original-disclosure").count()) === 1,
    "One-shot general AI",
  )
  checks.push("Direct, app-override and general-prompt sessions")
  await session("mail", "focus", "default")
  assert(
    (await page.locator(".detail-pane .notice").textContent()).includes(
      "Fokus ændrede",
    ),
    "Focus fallback",
  )
  assert(
    (await page.locator(".detail-identity h2").textContent()) === "Mail",
    "Origin remains Mail",
  )
  await session("mail", "ai", "default")
  assert(
    (await page.locator(".detail-pane .notice").textContent()).includes(
      "Originalen er bevaret",
    ),
    "AI fallback",
  )
  assert(
    (await page.locator(".original-disclosure").count()) === 0,
    "AI failure does not invent processed version",
  )
  checks.push("Focus-safe manual outcome and AI fallback")
  await session("mail", "asr", "default")
  assert((await page.locator(".recovery-task").count()) === 1, "Recovery task")
  const deadline = await page.locator(".recovery-task p").textContent()
  await click("retry")
  await click("cancel-session")
  await page.waitForTimeout(700)
  assert(
    (await page.locator(".recovery-task p").textContent()) === deadline,
    "Cancelled retry preserves original deadline",
  )
  await click("retry")
  await page.waitForTimeout(1250)
  assert(
    (await page.locator(".recovery-task").count()) === 0,
    "Successful retry removes temporary recovery",
  )
  assert(
    (await page.locator(".detail-pane .notice").textContent()).includes(
      "Fokus ændrede",
    ),
    "Retry never assumes old focus",
  )
  checks.push("Recognition recovery, retry cancellation and fixed deadline")
  const count = await page.locator(".history-row").count()
  await openDemo("mail", "normal")
  await click("start-session")
  await click("stop-session")
  await page.keyboard.press("Escape")
  await page.waitForTimeout(1250)
  assert(
    (await page.locator(".history-row").count()) === count,
    "Late cancelled result not published",
  )
  checks.push("Cancellation invalidates late provider response")
  await openDemo("mail", "normal")
  await click("start-session")
  await click("stop-session")
  await nav("apps")
  await page.locator("#app-prompt").fill("Mit igangværende arbejde")
  await page.waitForTimeout(1250)
  assert(
    (await page.locator("#app-prompt").inputValue()) ===
      "Mit igangværende arbejde",
    "Async finish preserves editor and focus context",
  )
  await click("reset-app")
  await nav("history")
  checks.push("Async completion preserves settings editing")
  const beforeDelete = await page.locator(".history-row").count()
  await page.locator('.detail-header [data-action="result-menu"]').click()
  await click("ask-delete")
  await click("confirm-delete")
  assert(
    (await page.locator(".history-row").count()) === beforeDelete - 1,
    "Delete removes selected completed result",
  )
  checks.push("Deliberate delete confirmation")
  await openDemo()
  await click("open-setup")
  await click("setup-local")
  await click("setup-next")
  await click("setup-download")
  await click("cancel-download")
  await page.waitForTimeout(700)
  assert(
    (await act("setup-download").textContent()).includes("Hent lokal"),
    "Cancelled install cannot later become ready",
  )
  await click("setup-download")
  await page.waitForTimeout(700)
  await click("setup-mic")
  await click("setup-access")
  assert(await act("setup-ready").isDisabled(), "Shortcut test is required")
  await click("setup-shortcut")
  await click("setup-ready")
  assert(await act("setup-finish").isDisabled(), "Trial required")
  await click("setup-trial")
  await click("setup-finish")
  await openDemo()
  await click("open-setup")
  await click("setup-cloud")
  await click("setup-next")
  assert(
    await act("setup-ready").isDisabled(),
    "Cloud key separate from local model",
  )
  await click("setup-key")
  await click("setup-ready")
  await click("setup-trial")
  await click("setup-finish")
  checks.push("Local/cloud onboarding, install cancellation and trial gates")
  await openDemo()
  await click("long-result")
  assert(
    (await page.locator(".transcript").textContent()).length > 10000,
    "Long content fixture",
  )
  assert(
    (await page.locator(".transcript").getAttribute("class")) ===
      "transcript is-long",
    "Compact long-result role",
  )
  checks.push("Long result retains full text")
  const layouts = []
  for (const width of [1280, 900, 600, 320]) {
    await page.setViewportSize({ width, height: 850 })
    await nav("history")
    await page.locator('[data-action="select-result"]').first().click()
    assert(
      await page.locator(".transcript").isVisible(),
      "Selected detail remains visible at " + width,
    )
    const metrics = await page.evaluate(() => ({
      width: innerWidth,
      content: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
    }))
    assert(
      metrics.scroll <= metrics.content + 1,
      "No page horizontal overflow at " + width,
    )
    layouts.push(metrics)
    if (width < 1040) {
      await click("back-history")
      assert(
        await page.locator(".history-pane").isVisible(),
        "Back to history at " + width,
      )
    }
    await nav("settings")
    await tab("connections")
    const overflow = await page.evaluate(
      () =>
        document.documentElement.scrollWidth >
        document.documentElement.clientWidth + 1,
    )
    assert(!overflow, "Settings no overflow at " + width)
  }
  checks.push("History/detail/settings reflow at 1280, 900, 600 and 320px")
  await page.emulateMedia({ reducedMotion: "reduce" })
  await openDemo("mail")
  await click("start-session")
  assert(
    (await page
      .locator(".wave i")
      .first()
      .evaluate((e) => getComputedStyle(e).animationName)) === "none",
    "Reduced motion",
  )
  await click("cancel-session")
  checks.push("Reduced-motion state feedback")
  await page.goto(
    "file:///Users/mikkelfreltoftkrogsholm/Projekter/vara/gdd/design/index.html",
  )
  await page.setViewportSize({ width: 320, height: 850 })
  await page.locator('[data-page="settings"]').focus()
  await page.keyboard.press("Enter")
  assert(
    await page.locator("#activation-style").isVisible(),
    "File URL and keyboard navigation",
  )
  checks.push("Direct file URL loading and keyboard navigation")
  await click("open-demo")
  for (let i = 0; i < 5; i++) {
    await page.keyboard.press("Tab")
    assert(
      await page
        .locator("#modal")
        .evaluate((e) => e.contains(document.activeElement)),
      "Native modal focus trap",
    )
  }
  await page.keyboard.press("Escape")
  assert(!(await page.locator("#modal").isVisible()), "Escape closes dialog")
  checks.push("Modal keyboard focus containment and Escape")
  await openDemo("mail")
  await click("start-session")
  await click("stop-session")
  let hud = await page
    .locator(".recording-hud")
    .evaluate((e) => ({ client: e.clientWidth, scroll: e.scrollWidth }))
  assert(hud.scroll <= hud.client + 1, "Speech HUD fits 320px")
  await page.waitForTimeout(650)
  hud = await page
    .locator(".recording-hud")
    .evaluate((e) => ({ client: e.clientWidth, scroll: e.scrollWidth }))
  assert(hud.scroll <= hud.client + 1, "Text HUD fits 320px")
  await click("cancel-session")
  checks.push("320px listening/processing HUD labels and controls fit")
  await page.setViewportSize({ width: 900, height: 600 })
  await nav("history")
  await page.locator("#history-scroll").evaluate((e) => (e.scrollTop = 100))
  const beforeScroll = await page
    .locator("#history-scroll")
    .evaluate((e) => e.scrollTop)
  await page.locator('[data-action="select-result"][data-id="2"]').click()
  await click("back-history")
  assert(
    Math.abs(
      (await page.locator("#history-scroll").evaluate((e) => e.scrollTop)) -
        beforeScroll,
    ) < 2,
    "List scroll restored",
  )
  checks.push("Narrow master/detail navigation preserves list scroll position")

  assert(errors.length === 0, "No JavaScript errors: " + errors.join(", "))
  await page.setViewportSize({ width: 1586, height: 992 })
  await page.goto("http://127.0.0.1:18769/design/index.html")
  return {
    result: "PASS",
    checks,
    layouts,
    columns,
    javascriptErrors: errors,
    scope:
      "Interactive static prototype only. No native microphone, focus handoff, clipboard, provider, storage or platform-runtime behavior is claimed.",
  }
}
