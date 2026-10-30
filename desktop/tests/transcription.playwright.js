async function run(page) {
  const root = process.env.VARA_TEST_OUTPUT_ROOT || ".";
  const context = await page.context().browser().newContext();
  page = await context.newPage();
  const fixture = await (
    await page.request.get(
      "http://127.0.0.1:18770/tests/fixtures/ui-state.json",
    )
  ).json();
  const errors = [],
    checks = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const assert = (v, m) => {
    if (!v) throw Error(m);
  };
  await page.addInitScript((data) => {
    window.txMainState = data;
    window.txFixtureCalls = [];
    window.txFixtureEvents = {};
    const speech = data.settings.engines.find((e) => e.id === "local.parakeet");
    const document = {
      id: "file-1",
      title: "Interview med Anna",
      source_name: "interview-anna.m4a",
      source_path: "/original/interview-anna.m4a",
      created_at: 1792953000,
      engine: speech,
      language: "da",
      stage: "ready",
      error: null,
      duration: 24.18,
      completed_seconds: 24.18,
      revision: 0,
      edited_text: null,
      ai_text: null,
      ai_engine: null,
      speakers: { speaker_1: "Anna", speaker_2: "Mikkel" },
      transcript: {
        text: "Vi skal gøre det nemt at komme i gang. Når jeg klikker på et ord, vil jeg høre passagen.",
        timing: "word",
        timing_source: "asr",
        segments: [
          {
            id: "s1",
            text: "Vi skal gøre det nemt at komme i gang.",
            start: 0.1,
            end: 1.8,
            speaker: "speaker_1",
            words: [
              {
                id: "w1",
                text: "Vi skal gøre det nemt at komme i gang.",
                start: 0.1,
                end: 1.8,
                speaker: "speaker_1",
              },
            ],
          },
          {
            id: "s2",
            text: "Når jeg klikker på et ord, vil jeg høre passagen.",
            start: 2,
            end: 4,
            speaker: "speaker_2",
            words: [
              {
                id: "w2",
                text: "Når jeg",
                start: 2,
                end: 2.4,
                speaker: "speaker_2",
              },
              {
                id: "w3",
                text: "klikker",
                start: 2.4,
                end: 2.8,
                speaker: "speaker_2",
              },
              {
                id: "w4",
                text: "på et ord, vil jeg høre passagen.",
                start: 2.8,
                end: 4,
                speaker: "speaker_2",
              },
            ],
          },
        ],
      },
    };
    window.txFixtureDocs = [
      document,
      {
        ...structuredClone(document),
        id: "file-2",
        title: "Workshop.mp4",
        stage: "transcribing",
        transcript: null,
        completed_seconds: 5,
      },
      {
        ...structuredClone(document),
        id: "file-3",
        title: "Noter.m4a",
        stage: "failed",
        transcript: null,
        error: "Intet understøttet lydspor.",
      },
    ];
    const emit = () =>
      window.txFixtureEvents["vara:files"]?.({ payload: null });
    window.__TAURI__ = {
      core: {
        convertFileSrc: () => "/tests/fixtures/player.wav",
        invoke: async (name, { action, payload }) => {
          window.txFixtureCalls.push({ action, payload });
          const d = window.txFixtureDocs.find((d) => d.id === payload?.id);
          switch (action) {
            case "snapshot":
              return structuredClone(data);
            case "devices":
              return [];
            case "application_icons":
              return {};
            case "file_list":
              return {
                documents: structuredClone(window.txFixtureDocs),
                media: window.txMissingAudio
                  ? {}
                  : { "file-1": "/private/player.wav" },
              };
            case "file_who_status":
              return { available: true, ready: !!window.txWhoReady };
            case "file_who_download":
              if (!payload.acknowledge_license)
                throw Error("Læs licensen først.");
              if (window.txWhoDownloadWait)
                await new Promise(
                  (resolve) => (window.txWhoFinishDownload = resolve),
                );
              window.txWhoReady = true;
              return { ready: true };
            case "file_who":
              if (window.txWhoFails)
                throw Error("Who kunne ikke finde talere. Teksten er bevaret.");
              d.diarization = {
                model: "Who by Desert Ant Labs",
                version: "v0.3.4",
                turns: [
                  { speaker: "who-0", start: 0.1, end: 1.8 },
                  { speaker: "who-1", start: 2, end: 4 },
                ],
              };
              if (payload.known_speaker_count) {
                d.diarization.known_speakers = { count: payload.known_speaker_count, method: "fixture", cosine_threshold: 0.65 };
                d.diarization.raw_turns = structuredClone(d.diarization.turns);
                d.diarization.turns.push({ speaker: "who-uncertain", start: 5, end: 7 });
              }
              d.speaker_transcript = structuredClone(d.transcript);
              d.speaker_transcript.segments.forEach((s, i) => {
                s.speaker = "who-" + i;
                s.words.forEach((w) => (w.speaker = s.speaker));
              });
              d.speakers["who-0"] = "Taler 1";
              d.speakers["who-1"] = "Taler 2";
              emit();
              return d.diarization;
            case "file_who_cancel":
              if (d) d.who_busy = false;
              emit();
              return null;
            case "file_pick":
              return [
                {
                  path: "/original/Interview.m4a",
                  name: "Interview.m4a",
                  bytes: 18000000,
                },
                {
                  path: "/original/Workshop.mp4",
                  name: "Workshop.mp4",
                  bytes: 84000000,
                },
              ];
            case "file_add":
              window.txFixtureDocs.push({
                ...structuredClone(document),
                id: "new-file",
                title: "Ny fil",
                stage: "queued",
                transcript: null,
              });
              emit();
              return ["new-file"];
            case "file_save":
              if (window.txSaveFails)
                throw Error("Disken er fuld. Din kladde er bevaret.");
              if (payload.revision !== d.revision)
                throw Error("Dokumentet er ændret.");
              d.edited_text =
                payload.text === d.transcript.text ? null : payload.text;
              d.revision++;
              emit();
              return { revision: d.revision };
            case "file_rename":
              d.title = payload.title;
              emit();
              break;
            case "file_speaker":
              d.speakers[payload.speaker] = payload.name;
              emit();
              break;
            case "file_cancel":
              d.stage = "cancelled";
              emit();
              break;
            case "file_retry":
              d.stage = "queued";
              emit();
              break;
            case "file_delete":
              window.txFixtureDocs = window.txFixtureDocs.filter(
                (d) => d.id !== payload.id,
              );
              emit();
              break;
            case "file_ai":
              d.ai_text = "Et kort resumé af interviewet.";
              emit();
              break;
            case "file_export":
              if (
                ["srt", "vtt"].includes(payload.format) &&
                ((!payload.original && d.edited_text !== null) ||
                  d.transcript.timing === "none")
              )
                throw Error("Undertekster kræver tidsjusteret tekst.");
              return true;
          }
          return null;
        },
      },
      event: {
        listen: async (name, fn) => {
          window.txFixtureEvents[name] = fn;
          return () => {};
        },
      },
    };
  }, fixture);
  await page.setViewportSize({ width: 1586, height: 992 });
  await page.goto("http://127.0.0.1:18770/ui/index.html");
  await page.locator('[data-page="transcription"]').click();
  const act = (a) =>
    page.locator(`[data-action="${a}"]`).filter({ visible: true }).first();
  assert(
    await page
      .getByRole("heading", { name: "Tilføj lyd eller video" })
      .isVisible(),
    "Import primary task",
  );
  await act("tx-pick").click();
  assert(
    (await page.locator(".tx-staged li").count()) === 2,
    "Multiple files staged",
  );
  assert(
    await page.locator("#tx-language").isDisabled(),
    "Redux language automatic",
  );
  await page.screenshot({
    path: root + "/gdd/reports/transcription/import.png",
  });
  await page.locator("#tx-engine").selectOption("openai.speech");
  assert(
    (await page.locator(".tx-locality").innerText()).includes("Lyd sendes"),
    "Cloud destination before start",
  );
  await page.locator("#tx-engine").selectOption("local.parakeet");
  await act("tx-start").click();
  assert(
    await page.locator(".tx-detail").isVisible(),
    "Explicit start reaches queued file",
  );
  assert(
    (await page.evaluate(
      () =>
        window.txFixtureCalls.filter((c) => c.action === "file_add").at(-1)
          .payload.paths.length,
    )) === 2,
    "Native job receives batch",
  );
  checks.push("Import, batching, local/cloud destinations and explicit start");
  await page.locator('[data-action="tx-select"][data-id="file-1"]').click();
  assert((await page.locator(".tx-word").count()) === 4, "Word layer present");
  assert(
    (await page.locator(".tx-provenance").innerText()).includes("Ordtidskoder"),
    "Timing level visible",
  );
  assert(
    (await page.locator(".tx-segment-meta").first().innerText()).includes(
      "Anna",
    ),
    "Speaker label",
  );
  await page.locator('[data-word="w3"]').click();
  await page.waitForFunction(() =>
    document
      .querySelector('[data-word="w3"]')
      ?.classList.contains("is-playing"),
  );
  await page.screenshot({
    path: root + "/gdd/reports/transcription/editor.png",
  });
  await act("tx-play").click();
  checks.push("Editor, actual HTML audio playback, word seek and highlight");
  // Select a real timed passage and verify playback stops at its final word.
  await page.evaluate(() => {
    const first = document.querySelector('[data-word="w2"]');
    const last = document.querySelector('[data-word="w3"]');
    const range = document.createRange();
    range.setStartBefore(first);
    range.setEndAfter(last);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
  });
  await act("tx-selection").click();
  await page.waitForFunction(
    () =>
      document.querySelector('[data-action="tx-play"]').textContent === "Pause",
  );
  await page.waitForFunction(
    () =>
      document.querySelector('[data-action="tx-play"]').textContent ===
      "Afspil",
    { timeout: 5000 },
  );
  assert(
    (await page.locator("#tx-time").innerText()).startsWith("0:02"),
    "Passage stops at selected word boundary",
  );
  await page.evaluate(() => window.getSelection().removeAllRanges());
  checks.push("Timed passage selection plays and stops at its boundary");
  await page.locator('[data-action="tx-speaker"][data-id="speaker_1"]').click();
  await page.locator("#tx-speaker-input").fill("Interviewer");
  await act("tx-speaker-save").click();
  assert(
    (await page.locator(".tx-segment-meta").first().innerText()).includes(
      "Interviewer",
    ),
    "Speaker rename retained",
  );
  checks.push(
    "Provider speaker names can be changed independently of transcript text",
  );
  await page.locator('[data-word="w1"]').focus();
  await page.keyboard.press("ArrowRight");
  assert(
    (await page.evaluate(() => document.activeElement.dataset.word)) === "w2",
    "Roving word navigation uses arrow keys",
  );
  await page.evaluate(() => {
    window.txMainState.session = {
      stage: "recording",
      plan: window.txMainState.history[0].plan,
    };
    window.txFixtureEvents["vara:state"]({
      payload: structuredClone(window.txMainState),
    });
  });
  await act("tx-play").click();
  assert(
    (await page.locator(".tx-reading").innerText()).includes(
      "Afslut dikteringen",
    ),
    "File playback cannot overlap microphone dictation",
  );
  await page.evaluate(() => {
    window.txMainState.session = null;
    window.txFixtureEvents["vara:state"]({
      payload: structuredClone(window.txMainState),
    });
  });
  await act("tx-dismiss").click();
  checks.push("Keyboard word navigation and recording/playback isolation");

  await act("tx-edit").click();
  await page.locator("#tx-editor-text").fill("Min rettede tekst");
  await page.waitForFunction(
    () => window.txFixtureDocs[0].edited_text === "Min rettede tekst",
  );
  assert(
    (await page.locator(".tx-edit-notice").innerText()).includes(
      "kræver ny tidsjustering",
    ),
    "Edit does not inherit verified times",
  );
  await page.locator("#tx-editor-text").press("End");
  await page.locator("#tx-editor-text").pressSequentially("!");
  await page.waitForFunction(
    () => window.txFixtureDocs[0].edited_text === "Min rettede tekst!",
  );
  await page.locator("#tx-editor-text").press("ControlOrMeta+z");
  assert(
    (await page.locator("#tx-editor-text").inputValue()) ===
      "Min rettede tekst",
    "Native undo survives autosave",
  );
  await page.waitForFunction(
    () => window.txFixtureDocs[0].edited_text === "Min rettede tekst",
  );
  await act("tx-original").click();
  assert(
    (await page.locator(".tx-word").count()) === 4,
    "Original word data preserved",
  );
  await act("tx-original").click();
  assert(
    (await page.locator("#tx-editor-text").inputValue()) ===
      "Min rettede tekst",
    "Edited version preserved",
  );
  checks.push("Autosave, original and explicit alignment status");
  await page.evaluate(() => {
    window.txSaveFails = true;
  });
  await page.locator("#tx-editor-text").fill("Kladde der ikke kunne gemmes");
  await page.waitForFunction(() =>
    document
      .querySelector(".tx-reading")
      ?.textContent.includes("Disken er fuld"),
  );
  await act("tx-copy").click();
  assert(
    (await page.evaluate(
      () =>
        window.txFixtureCalls.filter((c) => c.action === "file_copy").at(-1)
          .payload.draft,
    )) === "Kladde der ikke kunne gemmes",
    "Failed save copies the actual recoverable draft, never stale stored text",
  );
  assert(
    (await page.locator("#tx-editor-text").inputValue()) ===
      "Kladde der ikke kunne gemmes",
    "Failed save retains draft",
  );
  await page.evaluate(() => {
    window.txSaveFails = false;
  });
  await act("tx-save").click();
  await page.locator("#tx-editor-text").fill("Min rettede tekst");
  await page.waitForFunction(
    () => window.txFixtureDocs[0].edited_text === "Min rettede tekst",
  );
  checks.push("Failed autosave preserves and copies the recoverable draft");
  await act("tx-export").click();
  await page.locator("#tx-export-format").selectOption("srt");
  await act("tx-export-save").click();
  assert(
    (await page.locator("#dialog-error").innerText()).includes("tidsjusteret"),
    "Unsafe subtitle export rejected",
  );
  await page.locator("#tx-export-original").selectOption("original");
  await act("tx-export-save").click();
  assert(
    (await page.locator("dialog[open]").count()) === 0,
    "Original subtitle export allowed",
  );
  checks.push("Export format/version and alignment guard");
  await act("tx-rename").click();
  await page.locator("#tx-title-input").fill("Dansk interview");
  await act("tx-rename-save").click();
  assert(
    (await page.locator(".tx-title").innerText()).includes("Dansk interview"),
    "Rename persists",
  );
  await page.locator('[data-action="tx-tab"][data-id="ai"]').click();
  await page.locator("#tx-ai-prompt").fill("Lav et kort resumé.");
  await act("tx-ai").click();
  assert(
    (await page.locator(".tx-ai-result").innerText()).includes(
      "Et kort resumé",
    ),
    "Separate AI result",
  );
  await page.locator('[data-action="tx-tab"][data-id="transcript"]').click();
  assert(
    (await page.locator("#tx-editor-text").inputValue()) ===
      "Min rettede tekst",
    "AI does not replace corrected transcript",
  );
  checks.push("Rename and separate optional AI output");
  await page.locator('[data-action="tx-select"][data-id="file-3"]').click();
  assert(
    (await page.locator(".tx-detail").innerText()).includes(
      "Intet understøttet lydspor",
    ),
    "Actionable file error",
  );
  await act("tx-retry").click();
  assert(
    (await page.evaluate(
      () => window.txFixtureDocs.find((d) => d.id === "file-3").stage,
    )) === "queued",
    "Retry preserves document identity",
  );
  await page.locator('[data-action="tx-cancel"][data-id="file-2"]').click();
  assert(
    (await page.evaluate(
      () => window.txFixtureDocs.find((d) => d.id === "file-2").stage,
    )) === "cancelled",
    "Per-file cancellation",
  );
  checks.push("Failed import, retry and cancellation");
  await page.locator('[data-action="tx-select"][data-id="file-1"]').click();
  await page.evaluate(() => {
    window.txMissingAudio = true;
    window.txFixtureEvents["vara:files"]({ payload: null });
  });
  await page.waitForSelector(".tx-missing");
  assert(
    (await page.locator(".tx-missing").innerText()).includes(
      "Teksten er bevaret",
    ),
    "Missing audio preserves text",
  );
  checks.push("Missing playback source");
  await page.evaluate(() => {
    window.txMissingAudio = false;
    window.txFixtureEvents["vara:files"]({ payload: null });
  });
  const originalBeforeWho = await page.evaluate(() =>
    JSON.stringify(window.txFixtureDocs[0].transcript),
  );
  await act("tx-who-open").click();
  assert(
    (await page.locator("dialog").innerText()).includes("installations-id"),
    "Who usage reporting disclosed before download/use",
  );
  assert(
    (await page.locator("dialog").innerText()).includes(
      "Lyd og tekst sendes ikke",
    ),
    "Who data boundary explicit",
  );
  await act("tx-who-download").click();
  await act("tx-who-run").waitFor();
  await act("tx-who-run").click();
  await page.waitForSelector('.tx-segment-meta [data-id="who-0"]');
  assert(
    (await page.locator("#tx-who-tools").innerText()).includes("2 talere"),
    "Who result visible",
  );
  assert(
    (await page.evaluate(() =>
      JSON.stringify(window.txFixtureDocs[0].transcript),
    )) === originalBeforeWho,
    "Who never mutates source ASR",
  );
  assert(
    (await page.evaluate(() => window.txFixtureDocs[0].edited_text)) !== null,
    "Who preserves corrections",
  );
  await page.locator(".tx-who-turns summary").click();
  assert(
    (await page.locator(".tx-who-turn").count()) === 2,
    "Independent speaker intervals accessible",
  );
  await page.locator('.tx-segment-meta [data-id="who-0"]').click();
  await page.locator("#tx-speaker-input").fill("Vært");
  await act("tx-speaker-save").click();
  assert(
    (await page.locator(".tx-segment-meta").first().innerText()).includes(
      "Vært",
    ),
    "Who speaker can be renamed",
  );
  checks.push(
    "Optional Who setup, usage disclosure, speaker turns, rename and immutable source/corrections",
  );
  await act("tx-who-open").click();
  assert((await page.locator("#tx-who-count").inputValue()) === "", "Automatic is default");
  assert((await page.locator("dialog").innerText()).includes("krydstjekker med forskudte tidsvinduer"), "Known-count cross-check is explained");
  await page.locator("#tx-who-count").selectOption("2");
  await act("tx-who-run").click();
  await page.waitForFunction(() => document.querySelector("#tx-who-tools").textContent.includes("usikker"));
  assert((await page.locator("#tx-who-tools").innerText()).includes("2 talere · 0:02 med usikker taler"), "Known count excludes and measures uncertainty");
  await page.locator(".tx-who-turns summary").click();
  assert((await page.locator(".tx-who-turns").innerText()).includes("Usikker taler"), "Uncertain interval visible");
  assert((await page.locator('[data-action="tx-speaker"][data-id="who-uncertain"]').count()) === 0, "Uncertainty cannot be renamed");
  await act("tx-who-open").click();
  assert((await page.locator("#tx-who-count").inputValue()) === "2", "Known count survives reopening");
  assert(await page.evaluate(() => window.txFixtureCalls.some(c => c.action === "file_who" && c.payload.known_speaker_count === 2)), "Known count sent to host");
  await page.locator('dialog button[aria-label="Luk dialog"]').click();
  checks.push("Known speaker count, uncertainty and persisted choice");
  await page.evaluate(() => (window.txWhoFails = true));
  await act("tx-who-open").click();
  await act("tx-who-run").click();
  await page.waitForFunction(() =>
    document
      .querySelector(".tx-reading")
      .textContent.includes("Who kunne ikke"),
  );
  assert(
    (await page.locator('.tx-segment-meta [data-id="who-0"]').count()) === 1,
    "Failed rerun retains previous labels",
  );
  await act("tx-dismiss").click();
  await page.evaluate(() => {
    window.txWhoFails = false;
    window.txFixtureDocs[0].who_busy = true;
    window.txFixtureEvents["vara:files"]({ payload: null });
  });
  await act("tx-who-cancel").click();
  assert(
    (await page.evaluate(() =>
      window.txFixtureCalls.some((c) => c.action === "file_who_cancel"),
    )) === true,
    "Who cancellation uses separate job",
  );
  checks.push(
    "Who failure/cancellation preserve the document and previous successful result",
  );
  await page.waitForFunction(
    () =>
      !document
        .querySelector("#tx-who-tools")
        .textContent.includes("Finder talere"),
  );
  await page.evaluate(() => {
    window.txWhoReady = false;
    window.txWhoDownloadWait = true;
  });
  await act("tx-who-open").click();
  await act("tx-who-download").click();
  await page.locator('dialog button[aria-label="Luk dialog"]').click();
  await page.evaluate(() => window.txWhoFinishDownload());
  await page.waitForFunction(() => window.txWhoReady === true);
  assert(
    !(await page.locator("dialog").evaluate((n) => n.open)),
    "Late download completion must not reopen a dismissed dialog",
  );
  checks.push(
    "Who model setup does not reclaim focus after its dialog is dismissed",
  );

  for (const width of [1586, 1280, 1040, 720, 320]) {
    await page.setViewportSize({ width, height: 992 });
    assert(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
      "No viewport overflow " + width,
    );
    assert(
      await page
        .locator("#workspace")
        .evaluate((n) => n.scrollWidth <= n.clientWidth + 1),
      "No workspace overflow " + width,
    );
  }
  checks.push("Desktop and narrow reflow");
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.locator('.tx-detail [data-action="tx-delete"]').click();
  assert(
    (await page.locator("dialog").innerText()).includes(
      "originalfil ændres ikke",
    ),
    "Deletion explains source safety",
  );
  await act("tx-delete-confirm").click();
  assert(
    !(await page.evaluate(() =>
      window.txFixtureDocs.some((d) => d.id === "file-1"),
    )),
    "Deleted file disappears",
  );
  checks.push("Deletion scope and library update");
  assert(errors.length === 0, errors.join("\n"));
  await context.close();
  return checks;
}
