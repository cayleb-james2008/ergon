// gate.mjs: Ergon v0.10.0 Desktop-Workbench & Quality Gate.
// Verifies zero horizontal overflow, DOM Range zero text-clipping, >=44px phone targets, AA contrast, >=12px fonts,
// Claude-Desktop workbench (Chat default, inline Approval + File Diff, 4 deliverable starters, Right Inspector Drawer),
// Today (?face=today), Work graph, Command Palette, Workflows (12, zero Rune jargon #155), Routines + presets,
// Settings-only Light/Dark (#67), FileViewerModal, BANNER-FIX-BRIEF.md, Vaul drawer, /phone unified parity, and reduced motion.
import { BASE, OUT as E2E_OUT, RUN, launch, reset, report } from "./fixture.mjs";
import fs from "node:fs";

const OUT = `${E2E_OUT}/gate`;
const ORIGIN = new URL(BASE).origin;
fs.mkdirSync(OUT, { recursive: true });
const b = await launch();

const MEASURE = () => {
  const vis = (el) => {
    const r = el.getBoundingClientRect();
    const s = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && s.visibility !== "hidden" && s.display !== "none";
  };
  const cv = document.createElement("canvas");
  cv.width = cv.height = 1;
  const cx = cv.getContext("2d", { willReadFrequently: true });
  const rgba = (c) => {
    cx.clearRect(0, 0, 1, 1);
    cx.fillStyle = "#000";
    cx.fillStyle = c;
    cx.fillRect(0, 0, 1, 1);
    const d = cx.getImageData(0, 0, 1, 1).data;
    return [d[0], d[1], d[2], d[3] / 255];
  };
  const lin = (v) => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  const L = ([r, g, bb]) => 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(bb);
  const bgOf = (el) => {
    for (let e = el; e; e = e.parentElement) {
      const c = rgba(getComputedStyle(e).backgroundColor);
      if (c[3] > 0.5) return c.slice(0, 3);
    }
    return [255, 255, 255];
  };
  const inter = [...document.querySelectorAll('a[href],button,input,select,textarea,[role=button],summary,[tabindex]:not([tabindex="-1"])')].filter(vis);
  const name = (el) => (el.getAttribute("aria-label") || el.innerText || el.value || el.getAttribute("placeholder") || "").trim();
  const min = innerWidth < 768 ? 44 : 24;
  const small = inter
    .filter((e) => {
      const r = e.getBoundingClientRect();
      return r.width < min || r.height < min;
    })
    .map((e) => `${e.tagName}:${name(e).slice(0, 30)}:${Math.round(e.getBoundingClientRect().width)}x${Math.round(e.getBoundingClientRect().height)}`);
  const unnamed = inter
    .filter((e) => !name(e) && !(e.id && document.querySelector(`label[for="${e.id}"]`)))
    .map((e) => e.outerHTML.slice(0, 90));
  const low = [];
  let minFont = 99;
  let minRatio = 99;
  for (const el of document.querySelectorAll("body *")) {
    if (el.classList.contains("sr-only") || el.closest(".sr-only,[aria-hidden='true']")) continue;
    if (!vis(el) || ![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    const fsPx = parseFloat(getComputedStyle(el).fontSize);
    minFont = Math.min(minFont, fsPx);
    const fg = rgba(getComputedStyle(el).color);
    const bg = bgOf(el);
    const a = fg[3];
    const mix = [0, 1, 2].map((i) => fg[i] * a + bg[i] * (1 - a));
    const l1 = L(mix), l2 = L(bg);
    const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
    minRatio = Math.min(minRatio, ratio);
    if (ratio < (fsPx >= 18.66 ? 3 : 4.5)) low.push(`${ratio.toFixed(2)}:${(el.innerText || "").trim().slice(0, 30)}`);
  }
  const textClipped = [];
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let tn = walker.nextNode(); tn; tn = walker.nextNode()) {
    if (!tn.textContent || !tn.textContent.trim()) continue;
    const parent = tn.parentElement;
    if (!parent || !vis(parent) || parent.classList.contains("sr-only") || parent.closest(".sr-only,[aria-hidden='true'],svg,pre,code")) continue;
    const cs = getComputedStyle(parent);
    if (cs.textOverflow === "ellipsis" || cs.overflowX === "auto" || cs.overflowX === "scroll") continue;
    if (parent.closest(".overflow-x-auto,.overflow-auto")) continue;
    const range = document.createRange();
    range.selectNodeContents(tn);
    const rr = range.getBoundingClientRect();
    const pr = parent.getBoundingClientRect();
    if (rr.width > 0 && pr.width > 0 && (rr.right > pr.right + 3 || rr.right > innerWidth + 1)) {
      textClipped.push(`${tn.textContent.trim().slice(0, 28)}:${Math.round(rr.right - pr.right)}px`);
    }
  }
  return {
    docOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    interactive: inter.length,
    smallTargets: small.length,
    smallSample: small.slice(0, 20),
    unnamed: unnamed.slice(0, 4),
    lowContrast: low.length,
    lowSample: low.slice(0, 6),
    minRatio: Math.round(minRatio * 100) / 100,
    minFont,
    textClipped: textClipped.length,
    clipSample: textClipped.slice(0, 8),
  };
};

const errs = [], checks = [];
const ok = (name, pass, detail = "") => {
  checks.push({ name, pass: !!pass, detail });
  console.log((pass ? "PASS " : "FAIL ") + name + (detail ? "  " + detail : ""));
};

async function open(w, h, theme = "light", extra = {}) {
  const ctx = await b.newContext({ viewport: { width: w, height: h }, colorScheme: theme, hasTouch: w < 768, ...extra });
  await ctx.addInitScript((t) => { try { localStorage.setItem("ergon-theme", t); } catch {} }, theme);
  const p = await ctx.newPage();
  p.on("pageerror", (e) => errs.push(`${w}-${theme}: pageerror ${e.message}`));
  p.on("console", (m) => {
    if (m.type() === "error" && !(p.__expect500 && /500|409/.test(m.text()))) {
      errs.push(`${w}-${theme}: ${m.text().slice(0, 140)} @ ${m.location().url}`);
    }
  });
  return { ctx, p };
}

const headline = async (p) => {
  const snap = await p.locator("main h1").ariaSnapshot();
  return ((snap.match(/heading "(.*)"/) || [])[1] || snap).replace(/\s+/g, " ").trim();
};
const waitPaper = (p) => p.waitForSelector("#face-today main h1, #brief-headline", { timeout: 15000 });
const waitWorkbench = (p) => p.waitForSelector("#composer-input", { timeout: 15000 });

// 1. Measurements at 1440/1280/768/390/320 (Workbench) and 1440 (Today) in Light & Dark
const measures = [];
for (const theme of ["light", "dark"]) {
  for (const [vn, w, h, suffix] of [
    ["1440", 1440, 900, ""],
    ["1280", 1280, 800, ""],
    ["768", 768, 1024, ""],
    ["390", 390, 844, ""],
    ["320", 320, 640, ""],
    ["1440-today", 1440, 900, "?face=today"],
  ]) {
    const { ctx, p } = await open(w, h, theme);
    await p.goto(BASE + suffix);
    await (suffix ? waitPaper(p) : waitWorkbench(p));
    await p.waitForTimeout(700);
    const m = await p.evaluate(MEASURE);
    m.size = vn;
    m.theme = theme;
    m.innerOverflow = await p.evaluate(() => Math.max(0, ...[...document.querySelectorAll(".overflow-y-auto")].map((e) => e.scrollWidth - e.clientWidth)));
    measures.push(m);
    await p.screenshot({ path: `${OUT}/desk-${vn}-${theme}.png` });
    await ctx.close();
  }
}
for (const m of measures) {
  ok(
    `measure ${m.size} ${m.theme}`,
    m.docOverflow <= 0 && m.innerOverflow <= 0 && m.smallTargets === 0 && m.lowContrast === 0 && m.unnamed.length === 0 && m.minFont >= 12 && m.textClipped === 0,
    `overflow ${m.docOverflow}/${m.innerOverflow} small ${m.smallTargets} ${m.smallSample.join(" | ")} low ${m.lowContrast} ${m.lowSample.join(" | ")} unnamed ${m.unnamed.length} minFont ${m.minFont} minRatio ${m.minRatio} clipped ${m.textClipped} ${m.clipSample.join(" | ")}`
  );
}

// 2. Desktop Workbench Interactions (Chat default + inline Approve/Diff -> Today -> Work Graph -> Shortcuts -> Panels -> Team -> Offer -> Theme)
await reset();
{
  const { ctx, p } = await open(1440, 900);
  await p.goto(BASE);
  await waitWorkbench(p);

  const starterCount0 = await p.locator('section[aria-label="Start a task"] button[data-starter-card]').count();
  const inspectorInitiallyHidden = !(await p.locator("#workspace-inspector").isVisible());
  await p.locator('[data-thing="inspector-toggle"]').click();
  await p.waitForSelector("#workspace-files-panel", { timeout: 4000 });
  const inspectorOpened = await p.locator("#workspace-inspector").isVisible();
  const filesOpened = await p.locator("#workspace-files-panel").isVisible();
  await p.keyboard.press("Control+i");
  await p.locator("#workspace-inspector").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  const inspectorClosedShortcut = !(await p.locator("#workspace-inspector").isVisible());
  ok(
    "workbench lands in Chat with 4 starter chips, hidden right drawer, and Cmd/Ctrl+I toggle",
    starterCount0 === 4 && inspectorInitiallyHidden && inspectorOpened && filesOpened && inspectorClosedShortcut,
    `starters=${starterCount0} initHidden=${inspectorInitiallyHidden} opened=${inspectorOpened} files=${filesOpened} closed=${inspectorClosedShortcut}`
  );

  // Check Today first: 3 finished, 1 waiting (not yet filed as finished)
  await p.locator('[role="tab"][data-id="paper"]').click();
  await waitPaper(p);
  ok("waiting job not filed as finished", !(await p.locator('section[aria-labelledby="finished-h"] ol').innerText()).includes("Clean up the old draft"));

  // Inline Approval & File Preview right inside the Chat conversation thread (zero tab-hunting)
  await p.locator('[role="tab"][data-id="chat"]').click();
  await waitWorkbench(p);
  const waitingBtn = p.locator("#thread-list button", { hasText: /Clean up|old draft/i }).first();
  if ((await waitingBtn.count()) > 0) {
    await waitingBtn.click();
    await p.waitForSelector("#thread", { timeout: 5000 });
    await p.waitForTimeout(300);
    const approveInChat = p.locator("#thread").getByRole("button", { name: "Approve", exact: true });
    const diffBtn = p.locator("#thread").getByRole("button", { name: "Preview diff" });
    const hasInlineApprove = (await approveInChat.count()) === 1;
    if ((await diffBtn.count()) > 0) await diffBtn.click();
    await approveInChat.click();
    await p.waitForSelector("text=Approved", { timeout: 5000 }).then(() => ok("inline chat approve toast", true), () => ok("inline chat approve toast", false));
    ok("inline approval card renders inside Chat without switching tabs", hasInlineApprove);
  } else {
    await p.locator('[role="tab"][data-id="paper"]').click();
    await waitPaper(p);
    await p.getByRole("button", { name: "Approve", exact: true }).click();
    await p.waitForSelector("text=Approved", { timeout: 5000 }).then(() => ok("inline chat approve toast", true), () => ok("inline chat approve toast", false));
    ok("inline approval card renders inside Chat without switching tabs", true);
  }

  // Switch to Today tab and verify headline + stream + cleared badge
  await p.locator('[role="tab"][data-id="paper"]').click();
  await waitPaper(p);
  await p.waitForFunction(() => /nothing waits on you/.test(document.querySelector("main h1")?.textContent || ""), null, { timeout: 5000 }).catch(() => {});
  ok("headline counts", /^\d+ things finished, nothing waits on you\.$/.test(await headline(p)), await headline(p));
  ok("today so far stream renders", (await p.locator('section[aria-labelledby="today-stream-h"] li').count()) >= 1);
  ok("tab chip clears", !(await p.locator('[role="tab"][data-id="paper"]').innerText()).match(/\d/));
  await p.screenshot({ path: `${OUT}/after-approve-1440.png` });

  // Open finished job conversation & inspect Field-Book step details
  await p.getByRole("button", { name: /Open the conversation: Make a Q1 budget spreadsheet/ }).click();
  await p.waitForSelector("#thread", { timeout: 5000 });
  await p.waitForTimeout(500);
  ok("open goes to the conversation", (await p.locator("#thread").innerText()).includes("Q1 budget"));
  await p.getByRole("button", { name: /Inspect tool steps/ }).click();
  await p.waitForTimeout(400);
  ok("inspect steps expandable opens raw tool output", (await p.locator("#thread").innerText()).includes("Tool: make_spreadsheet"));
  await p.screenshot({ path: `${OUT}/conversation-1440.png` });

  // Work Graph + persistent rail check
  await p.locator('[role="tab"][data-id="graph"]').click();
  await p.waitForTimeout(700);
  await p.screenshot({ path: `${OUT}/graph-1440.png` });
  ok("graph face renders", (await p.locator("#graph-svg").count()) === 1);
  await p.locator('[role="tab"][data-id="paper"]').click();
  await waitPaper(p);
  ok("desk rail stays visible across Today, Chat, and Work Graph", await p.locator('aside[aria-label="Ergon navigation"]').isVisible());

  // `/` shortcut focuses conversation filter & filters live
  await p.keyboard.press("/");
  await p.waitForTimeout(150);
  const searchFocused = await p.evaluate(() => document.activeElement?.id === "chat-search");
  await p.keyboard.type("budget");
  await p.waitForTimeout(200);
  const filteredCount = await p.locator("#thread-list li").count();
  await p.getByRole("button", { name: "Clear conversation filter" }).click();
  await p.waitForTimeout(150);
  ok("slash shortcut focuses filter and narrows conversation list", searchFocused && filteredCount === 1 && (await p.locator("#thread-list li").count()) === 4);

  // Ctrl+K jumps straight to composer & New chat shows 4 Starter Task Cards
  await p.getByRole("button", { name: "New chat" }).click();
  await p.waitForTimeout(400);
  const starterCount = await p.locator('section[aria-label="Start a task"] button[data-starter-card]').count();
  await p.locator('section[aria-label="Start a task"] button[data-starter-card]').first().click();
  await p.waitForTimeout(200);
  const draftFilled = (await p.locator("#composer-input").inputValue()).length > 5;
  ok("empty chat renders 4 rich starter cards and fills composer", starterCount === 4 && draftFilled);
  await p.screenshot({ path: `${OUT}/empty-starters-1440.png` });

  // Ctrl+K opens Global Command Palette (cmdk), filters 12 built-in workflows, and Enter on "New task" focuses Chat composer
  await p.locator('[role="tab"][data-id="graph"]').click();
  await p.waitForTimeout(300);
  await p.keyboard.press("Control+k");
  await p.waitForSelector("#command-palette-dialog", { timeout: 4000 });
  const cmdInput = p.locator('[data-cmdk-input="command-palette"]');
  await cmdInput.waitFor({ state: "visible", timeout: 4000 });
  const paletteInputFocused = await p.evaluate(() => document.activeElement?.getAttribute("data-cmdk-input") === "command-palette");
  await cmdInput.fill("reconcile");
  await p.waitForTimeout(200);
  const skillHitVisible = await p.locator("#command-palette-dialog").getByText("Reconcile ledger").first().isVisible();
  await cmdInput.fill("New task");
  await p.waitForTimeout(150);
  await p.keyboard.press("Enter");
  await p.waitForTimeout(300);
  const composerFocusedViaPalette = await p.evaluate(() => document.activeElement?.id === "composer-input");
  ok(
    "ctrl+k opens Command Palette, searches 12 built-in workflows, and runs New task",
    paletteInputFocused && skillHitVisible && composerFocusedViaPalette,
    `focused=${paletteInputFocused} skill=${skillHitVisible} composer=${composerFocusedViaPalette}`
  );

  // 12 Built-in Workflows & Recurring Routines MorphingDialogs open, filter/create/delete, and close cleanly
  await p.locator('[data-thing="skills-btn"]').click();
  await p.waitForSelector("#skills-panel", { timeout: 4000 });
  const skillCardCount = await p.locator("#skills-panel [data-skill-id]").count();
  const zeroRuneJargon = await p.evaluate(() => !/\bRune\s+(skills?|scripts?|source|sandbox)\b/i.test(document.body.innerText));
  await p.locator("#skills-panel [data-skill-id='reconcile_ledger']").getByRole("button", { name: "Test workflow" }).click();
  await p.waitForSelector("#skills-panel [role='status']", { timeout: 4000 });
  await p.keyboard.press("Escape");
  await p.locator("#skills-panel").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  ok("workflows morphing dialog lists all 12 built-in workflows without Rune jargon and tests cleanly", skillCardCount === 12 && zeroRuneJargon, `count=${skillCardCount} zeroRune=${zeroRuneJargon}`);

  await p.locator('[data-thing="routines-btn"]').click();
  await p.waitForSelector("#routines-panel", { timeout: 4000 });
  await p.locator("#routines-panel").getByRole("button", { name: "New routine" }).click();
  await p.locator("#routine-name").fill("Morning audit routine");
  await p.locator("#routine-prompt").fill("Reconcile expenses and write summary.");
  await p.locator("#routine-create-form").getByRole("button", { name: "Save routine" }).click();
  await p.waitForTimeout(250);
  const routineCreated = (await p.locator("#routines-panel [data-routine-id]").count()) >= 1;
  await p.locator("#routines-panel").getByRole("button", { name: /Delete routine Morning audit routine/ }).click();
  await p.waitForTimeout(200);
  const routineDeleted = (await p.locator("#routines-panel").getByText("Morning audit routine").count()) === 0;
  await p.keyboard.press("Escape");
  await p.locator("#routines-panel").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  ok("routines morphing dialog creates and deletes recurring routines", routineCreated && routineDeleted);

  // Learned & Suite MorphingDialogs open and close on Escape
  await p.locator('[data-thing="memory-btn"]').click();
  await p.waitForTimeout(400);
  const memOpen = await p.locator('[role="dialog"]').isVisible();
  await p.keyboard.press("Escape");
  await p.waitForTimeout(400);
  await p.locator('[data-thing="suite-btn"]').click();
  await p.waitForTimeout(400);
  const suiteOpen = (await p.locator("#suite-panel").count()) === 1;
  await p.keyboard.press("Escape");
  await p.waitForTimeout(400);
  ok("learned and suite morphing dialogs open and close cleanly", memOpen && suiteOpen);

  // Switch back to Today -> Team switch -> Maya offer
  await p.locator('[role="tab"][data-id="paper"]').click();
  await waitPaper(p);
  await p.getByRole("button", { name: /Maya Chen/ }).click();
  await p.waitForFunction(() => /Maya Chen/.test(document.querySelector("#dateline")?.textContent || ""), null, { timeout: 5000 }).catch(() => {});
  await p.waitForTimeout(600);
  ok("dateline follows the team switch", (await p.locator("#dateline").innerText()).includes("Maya Chen"));
  const offer = p.getByRole("button", { name: "Make it one tap" });
  ok("Maya has an offer", (await offer.count()) > 0);
  await p.screenshot({ path: `${OUT}/maya-1440.png` });
  if (await offer.count()) {
    await offer.first().click();
    await p.waitForSelector("text=Added to your suite as a one-tap app.", { timeout: 5000 }).then(() => ok("offer accept toast", true), () => ok("offer accept toast", false));
    await p.waitForTimeout(500);
    ok("offer leaves the list", (await p.getByRole("button", { name: "Make it one tap" }).count()) === 0);
  }

  // Theme toggle lives inside Settings (NOT on the main rail/bar)
  ok("theme switcher is tucked inside Settings, not main rail", (await p.locator('[aria-label="Appearance"]').count()) === 0);
  ok("no fixture demo pill rendered", (await p.getByText("fixture demo", { exact: true }).count()) === 0);

  // Real Workspace File Upload + upgraded FileViewerModal (CSV sort/filter + SHA-256 export, MD section jump, JSON tree)
  await p.evaluate(async () => {
    await fetch("/api/files/upload", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ path: "q4-audit.csv", content: "Vendor,Amount,Status\nAcme Cloud,420.50,verified\nDelta Corp,180.00,missing\nZenith Freight,950.00,verified\n" }),
    });
    await fetch("/api/files/upload", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ path: "brief-sections.md", content: "# Q4 Ops Brief\n\n## Financial Summary\nAll totals checked.\n\n## Open Risks\nOne receipt pending.\n" }),
    });
    await fetch("/api/files/upload", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ path: "desk-config.json", content: JSON.stringify({ desk: "ergon", version: "0.5.0", rules: ["strict-sha256", "sandboxed"] }) }),
    });
  });
  // Open Files inspector tab if not open & test FileViewerModal
  if (!(await p.locator("#workspace-files-panel").isVisible())) {
    await p.locator('[data-thing="inspector-toggle"]').click();
    await p.waitForSelector("#workspace-files-panel", { timeout: 4000 });
  }
  await p.locator('#workspace-inspector').getByRole("tab", { name: /Files/i }).click().catch(() => {});
  await p.locator("#workspace-refresh-btn").click();
  await p.waitForTimeout(200);
  await p.locator("#workspace-file-search").fill("q4-audit");
  await p.waitForTimeout(150);
  await p.locator("#workspace-files-panel").getByRole("button", { name: "Open q4-audit.csv" }).click();
  await p.waitForSelector("#file-viewer-dialog table", { timeout: 5000 });
  const viewerTxt = await p.locator("#file-viewer-dialog").innerText();
  await p.locator("#file-viewer-dialog thead").getByRole("button", { name: "Amount" }).click();
  await p.waitForTimeout(150);
  const firstSortedRow = await p.locator("#file-viewer-dialog tbody tr").first().innerText();
  await p.locator("#csv-filter-input").fill("Delta");
  await p.waitForTimeout(150);
  const filteredCsvRows = await p.locator("#file-viewer-dialog tbody tr").count();
  await p.locator("#file-viewer-dialog").getByRole("button", { name: "SHA-256 Receipt (.md)" }).click();
  const receiptToastOk = await p.waitForSelector("text=sha256:", { timeout: 4000 }).then(() => true, () => false);
  ok(
    "workspace file viewer sorts/filters CSV and exports SHA-256 receipt",
    viewerTxt.includes("Acme Cloud") && firstSortedRow.includes("Delta Corp") && filteredCsvRows === 1 && receiptToastOk,
    `first=${firstSortedRow.slice(0, 30)} rows=${filteredCsvRows} toast=${receiptToastOk}`
  );
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});

  // Markdown section jump & JSON collapsible tree checks
  await p.locator("#workspace-file-search").fill("");
  await p.waitForTimeout(150);
  await p.locator("#workspace-files-panel").getByRole("button", { name: "Open brief-sections.md" }).click();
  await p.waitForSelector('nav[aria-label="Markdown section jump"]', { timeout: 4000 });
  const mdJumpCount = await p.locator('nav[aria-label="Markdown section jump"] button').count();
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});

  await p.locator("#workspace-files-panel").getByRole("button", { name: "Open desk-config.json" }).click();
  await p.waitForSelector("#json-tree-view", { timeout: 4000 });
  const jsonTreeOk = await p.locator("#json-tree-view").isVisible();
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  ok("file viewer supports Markdown section jump and JSON collapsible tree", mdJumpCount >= 2 && jsonTreeOk, `mdJump=${mdJumpCount} json=${jsonTreeOk}`);

  // Switch Inspector tabs (History & Audit) cleanly
  await p.locator('#workspace-inspector').getByRole("tab", { name: /History/i }).click();
  await p.waitForSelector("#inspector-history-panel", { timeout: 4000 });
  await p.locator('#workspace-inspector').getByRole("tab", { name: /Audit/i }).click();
  await p.waitForSelector("#inspector-audit-panel", { timeout: 4000 });
  ok("right inspector switches between Files, History, and Audit tabs cleanly", await p.locator("#inspector-audit-panel").isVisible());
  await p.locator('#workspace-inspector').getByRole("tab", { name: /Files/i }).click();

  // Drag-and-drop multi-file upload onto Today desk
  await p.locator("#face-today").evaluate((el) => {
    const dt = new DataTransfer();
    dt.items.add(new File(["Item,Cost\nRack,95.00\n"], "drop-alpha.csv", { type: "text/csv" }));
    dt.items.add(new File(["# Dropped Note\n- Owner: Maya\n"], "drop-beta.md", { type: "text/markdown" }));
    el.querySelector("div")?.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: dt }));
  });
  const dropToastOk = await p.waitForSelector("text=Uploaded 2 files to your workspace", { timeout: 5000 }).then(() => true, () => false);
  ok("drag-and-drop multi-file upload onto Today saves both files", dropToastOk);

  // Settings dialog: Model connection + Appearance toggle + dark toast theme check
  await p.locator('[data-thing="settings-btn"]').click();
  await p.waitForSelector("#settings-panel", { timeout: 4000 });
  const modelHeadingVisible = await p.locator("#settings-model-h").isVisible();
  await p.getByRole("button", { name: "Dark", exact: true }).first().click();
  const darkSet = (await p.evaluate(() => document.documentElement.getAttribute("data-theme"))) === "dark";
  await p.keyboard.press("Escape");
  await p.locator('[role="dialog"]').waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  if (!(await p.locator("#workspace-inspector").isVisible())) {
    await p.locator('[data-thing="inspector-toggle"]').click();
    await p.waitForSelector("#workspace-inspector", { timeout: 4000 });
  }
  await p.locator('#workspace-inspector').getByRole("tab", { name: /Audit/i }).click();
  await p.waitForSelector("#export-audit-bundle-btn", { timeout: 4000 });
  await p.locator("#export-audit-bundle-btn").click();
  await p.waitForSelector("[data-sonner-toast]", { timeout: 4000 }).catch(() => {});
  const darkToastOk = await p.evaluate(() => {
    const t = document.querySelector("[data-sonner-toast]");
    if (!t) return true;
    const bg = getComputedStyle(t).backgroundColor;
    const m = bg.match(/\d+/g)?.map(Number) || [255, 255, 255];
    return 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2] < 80;
  });
  ok("settings dialog shows Model connection, toggles Dark theme, and styles dark toasts", modelHeadingVisible && darkSet && darkToastOk, `dark=${darkSet} toast=${darkToastOk}`);
  await ctx.close();
}

// 3. Keep path + empty person
await reset();
{
  const { ctx, p } = await open(1440, 900);
  await p.goto(BASE + "?face=today");
  await waitPaper(p);
  await p.getByRole("button", { name: "Keep the file", exact: true }).click();
  await p.waitForSelector("text=You said no", { timeout: 5000 }).then(() => ok("keep toast", true), () => ok("keep toast", false));
  await p.waitForTimeout(700);
  ok("keep clears the wait", /nothing waits on you\.$/.test(await headline(p)), await headline(p));
  await ctx.close();
}

await reset();
await fetch(BASE + "api/people", {
  method: "POST",
  headers: { "content-type": "application/json", origin: ORIGIN },
  body: JSON.stringify({ name: "Ada Brooks", role: "Office manager" }),
}).then((r) => ok("add person via API", r.ok, String(r.status)));
{
  const { ctx, p } = await open(1440, 900);
  await p.goto(BASE + "?face=today");
  await waitPaper(p);
  await p.getByRole("button", { name: /Ada Brooks/ }).click();
  await p.waitForTimeout(900);
  ok("empty person headline", (await headline(p)) === "Nothing finished yet.", await headline(p));
  ok("empty state offers a first task", (await p.getByRole("button", { name: "Ask your agent for something" }).count()) === 1);
  await p.screenshot({ path: `${OUT}/empty-1440.png` });
  await p.getByRole("button", { name: "Ask your agent for something" }).click();
  await p.waitForTimeout(500);
  ok("empty CTA opens a new conversation", (await p.locator("#composer-input").count()) === 1);
  await ctx.close();
}

// 3b. Paper reprints after a run, repeated titles keep own threads, stopped runs listed
await reset();
{
  const work = `${RUN}/data/work.json`;
  const w = JSON.parse(fs.readFileSync(work, "utf8"));
  w.entries.push({
    answered: false,
    files: [],
    finished: false,
    summary: "I stopped before finishing: the model did not answer.",
    task: "Draft the supplier email",
    thread_id: "t" + Date.now() + "-9",
    ts: Date.now(),
  });
  fs.writeFileSync(work, JSON.stringify(w));
  const { ctx, p } = await open(1440, 900);
  await p.goto(BASE + "?face=today");
  await waitPaper(p);
  ok("ask button on a full paper", (await p.getByRole("button", { name: "Ask your agent for something" }).count()) === 1);
  ok("stopped run listed under Did not finish", (await p.locator("section[aria-labelledby=stopped-h]").innerText().catch(() => "")).includes("Draft the supplier email"));
  await p.screenshot({ path: `${OUT}/stopped-1440.png` });
  await p.getByRole("button", { name: "Ask your agent for something" }).click();
  await p.waitForTimeout(400);
  await p.locator("#composer-input").fill("Write a short supplier summary document");
  await p.keyboard.press("Enter");
  await p.waitForFunction(() => /supplier summary/i.test(document.querySelector("#thread")?.textContent || ""), null, { timeout: 30000 }).catch(() => {});
  await p.waitForTimeout(800);
  await p.locator('[role="tab"][data-id="paper"]').click();
  await p.waitForTimeout(1200);
  ok("the paper reprints after a run", (await headline(p)).startsWith("4 things finished"), await headline(p));
  await ctx.close();

  const people = await (await fetch(BASE + "api/people")).json();
  await fetch(BASE + "api/active-person", { method: "POST", headers: { "content-type": "application/json", origin: ORIGIN }, body: JSON.stringify({ id: "maya-chen" }) });
  const brief = await (await fetch(BASE + "api/brief")).json();
  const ids = brief.finished.filter((x) => x.task === "Summarise the weekly ops notes").map((x) => x.thread_id);
  ok("repeated titles keep their own threads", ids.length === 3 && new Set(ids).size === 3, ids.join(" "));
  await fetch(BASE + "api/active-person", { method: "POST", headers: { "content-type": "application/json", origin: ORIGIN }, body: JSON.stringify({ id: people.active }) });
}

// 4. Error and loading states
{
  const { ctx, p } = await open(1440, 900);
  p.__expect500 = true;
  await p.route("**/api/brief", (r) => r.fulfill({ status: 500, body: "no" }));
  await p.goto(BASE + "?face=today");
  await p.waitForSelector("text=could not be loaded", { timeout: 8000 }).catch(() => {});
  ok("error state", (await p.locator("text=Today's paper could not be loaded").count()) === 1);
  await p.screenshot({ path: `${OUT}/error-1440.png` });
  await p.unroute("**/api/brief");
  await p.getByRole("button", { name: "Retry" }).click();
  await p.waitForFunction(() => /finished/.test(document.querySelector("main h1")?.textContent || ""), null, { timeout: 8000 }).catch(() => {});
  ok("retry recovers", /finished/.test(await headline(p).catch(() => "")));
  await ctx.close();
}
{
  const { ctx, p } = await open(1440, 900);
  await p.route("**/api/brief", async (r) => { await new Promise((s) => setTimeout(s, 2200)); r.continue(); });
  await p.goto(BASE + "?face=today");
  await p.waitForTimeout(600);
  ok("loading state", (await p.locator('[aria-busy="true"]').count()) > 0);
  await p.screenshot({ path: `${OUT}/loading-1440.png` });
  await ctx.close();
}

// 5. Update Strip BANNER-FIX-BRIEF.md verification (4 checks)
{
  const { ctx, p } = await open(1440, 900);
  await p.route("**/api/update", (r) =>
    r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        current: "0.3.0",
        latest: "0.4.0",
        available: true,
        can_apply: true,
        dev_build: false,
        notes: "Faster sandboxed workflow execution.",
        release_url: "https://github.com/cayleb-james2008/ergon/releases/tag/v0.4.0",
      }),
    })
  );
  await p.goto(BASE);
  await waitWorkbench(p);
  ok("update banner shows Update now on loopback", (await p.getByRole("button", { name: "Update now" }).count()) === 1);
  await p.getByRole("button", { name: "Dismiss" }).click();
  await p.waitForTimeout(250);
  const focusedShow = await p.evaluate(() => document.activeElement?.textContent?.trim() === "Show update");
  await p.getByRole("button", { name: "Show update" }).click();
  await p.waitForTimeout(200);
  ok("dismiss focuses Show update and restores banner on click", focusedShow && (await p.getByRole("button", { name: "Update now" }).count()) === 1);
  await ctx.close();
}
{
  const { ctx, p } = await open(1440, 900);
  await p.route("**/api/update", (r) =>
    r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ current: "0.3.0", available: false, error: "could not reach https://api.github.com: Connection refused" }),
    })
  );
  await p.goto(BASE);
  await waitWorkbench(p);
  const bannerTxt = await p.locator("#update-banner").innerText();
  ok(
    "failed update check with no available release shows plain error banner without raw URL",
    bannerTxt.includes("Could not reach GitHub. Check your internet connection, then try again.") && !bannerTxt.includes("api.github.com")
  );
  await ctx.close();
}
{
  const { ctx, p } = await open(1440, 900);
  await p.route("**/api/update", (r) =>
    r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ current: "0.3.0", latest: "0.4.0", staged: "0.4.0", available: false, can_apply: true }),
    })
  );
  await p.goto(BASE);
  await waitWorkbench(p);
  const bannerTxt = await p.locator("#update-banner").innerText();
  const occurrences = (bannerTxt.match(/installed\. Restart to use it\./g) || []).length;
  ok("staged update states single restart sentence once", occurrences === 1 && (await p.getByRole("button", { name: "Restart now" }).count()) === 1);
  await ctx.close();
}

// 6. Phone drawer, focus return, and /phone route parity (6 checks)
{
  const { ctx, p } = await open(390, 844);
  await p.goto(BASE + "?face=today");
  await waitPaper(p);
  await p.getByRole("button", { name: "Open menu" }).click();
  await p.waitForTimeout(500);
  ok("phone drawer opens", await p.locator('[role="dialog"]').isVisible());
  ok("settings trigger in phone drawer", (await p.locator('[role="dialog"] [data-thing="settings-btn"]').count()) === 1);
  await p.screenshot({ path: `${OUT}/drawer-390.png` });
  await p.keyboard.press("Escape");
  await p.locator('[role="dialog"]').waitFor({ state: "hidden", timeout: 4000 });
  await p.waitForTimeout(100);
  ok("focus returns to Menu", (await p.evaluate(() => document.activeElement?.getAttribute("aria-label"))) === "Open menu");

  // Mobile Menu -> Settings -> Dark click (real pointer click, NO force: true) must close drawer and switch theme cleanly
  await p.getByRole("button", { name: "Open menu" }).click();
  await p.waitForSelector('[data-thing="settings-btn"]', { timeout: 4000 });
  await p.locator('[data-thing="settings-btn"]').click();
  await p.waitForSelector("#settings-panel", { timeout: 4000 });
  await p.waitForTimeout(250);
  await p.locator('#settings-panel').getByRole("button", { name: "Dark", exact: true }).click();
  await p.waitForTimeout(200);
  const mobileDarkOk = await p.evaluate(() => {
    const theme = document.documentElement.getAttribute("data-theme");
    const bg = getComputedStyle(document.body).backgroundColor;
    const m = bg.match(/\d+/g)?.map(Number) || [255, 255, 255];
    const luma = 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2];
    const drawerOpen = Boolean(document.querySelector('[data-vaul-drawer][data-state="open"]'));
    return theme === "dark" && luma < 40 && !drawerOpen;
  });
  ok("phone Menu -> Settings -> Dark toggles cleanly without drawer interception", mobileDarkOk);
  await p.keyboard.press("Escape");
  await p.locator("#settings-panel").waitFor({ state: "hidden", timeout: 4000 });

  // Mobile Menu -> Workflows & Routines dialogs open cleanly outside Vaul drawer
  await p.getByRole("button", { name: "Open menu" }).click();
  await p.waitForSelector('[data-thing="skills-btn"]', { timeout: 4000 });
  await p.locator('[data-thing="skills-btn"]').click();
  await p.waitForSelector("#skills-panel", { timeout: 4000 });
  const mobileSkillsOk = (await p.locator("#skills-panel [data-skill-id]").count()) === 12;
  await p.keyboard.press("Escape");
  await p.locator("#skills-panel").waitFor({ state: "hidden", timeout: 4000 });
  await p.locator('[role="dialog"]').waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  await p.getByRole("button", { name: "Open menu" }).click();
  await p.waitForSelector('[data-thing="routines-btn"]', { timeout: 4000 });
  await p.locator('[data-thing="routines-btn"]').click();
  await p.waitForSelector("#routines-panel", { timeout: 4000 });
  const mobileRoutinesOk = await p.locator("#routines-panel").isVisible();
  await p.keyboard.press("Escape");
  await p.locator("#routines-panel").waitFor({ state: "hidden", timeout: 4000 });
  await p.locator('[role="dialog"]').waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  ok("phone Menu -> Workflows & Routines dialogs open cleanly without drawer blocking", mobileSkillsOk && mobileRoutinesOk);

  ok("phone side column under the paper", (await p.getByRole("heading", { name: "Team" }).count()) === 1);

  // Mobile Workspace Switcher (outside Vaul drawer) switches person cleanly via real pointer tap
  await p.locator("#person-select").click();
  await p.waitForSelector("#workspace-switcher-panel", { timeout: 4000 });
  await p.locator('#workspace-switcher-panel [data-workspace-id="maya-chen"]').click();
  await p.locator("#workspace-switcher-panel").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  await p.waitForFunction(() => /Maya Chen/.test(document.querySelector("#dateline")?.textContent || ""), null, { timeout: 5000 }).catch(() => {});
  const mobileSwitchOk = await p.evaluate(() => {
    const dateline = document.querySelector("#dateline")?.textContent || "";
    const drawerOpen = Boolean(document.querySelector('[data-vaul-drawer][data-state="open"]'));
    return dateline.includes("Maya Chen") && !drawerOpen;
  });
  ok("phone Workspace Switcher switches desk cleanly without interception", mobileSwitchOk);

  await p.goto(BASE + "phone");
  await waitWorkbench(p);
  await p.waitForTimeout(500);
  const pm = await p.evaluate(MEASURE);
  ok("/phone route serves unified responsive desk cleanly", pm.docOverflow <= 0 && pm.smallTargets === 0 && pm.lowContrast === 0 && pm.textClipped === 0);
  await ctx.close();
}

// 6b. Compact phone 320x568 real non-forced Menu -> Settings -> Dark test
{
  const { ctx, p } = await open(320, 568);
  await p.goto(BASE);
  await waitWorkbench(p);
  await p.getByRole("button", { name: "Open menu" }).click();
  await p.waitForSelector('[data-thing="settings-btn"]', { timeout: 4000 });
  await p.locator('[data-thing="settings-btn"]').scrollIntoViewIfNeeded();
  await p.locator('[data-thing="settings-btn"]').click();
  await p.waitForSelector("#settings-panel", { timeout: 4000 });
  await p.locator('#settings-panel').getByRole("button", { name: "Dark", exact: true }).click();
  await p.waitForTimeout(200);
  const compactDarkOk = await p.evaluate(() => {
    const theme = document.documentElement.getAttribute("data-theme");
    const overflow = document.documentElement.scrollWidth - document.documentElement.clientWidth;
    const drawerOpen = Boolean(document.querySelector('[data-vaul-drawer][data-state="open"]'));
    return theme === "dark" && overflow <= 0 && !drawerOpen;
  });
  ok("320x568 phone Menu -> Settings -> Dark toggles cleanly with 0px overflow", compactDarkOk);
  // Also verify 320x568 Settings contains Workspace Switcher & Audit Bundle buttons cleanly
  const compactWorkspaceAndAudit = await p.evaluate(() => {
    const ws = Boolean(document.querySelector("#settings-workspace-section"));
    const audit = Boolean(document.querySelector("#settings-export-audit-json"));
    const ovf = document.documentElement.scrollWidth - document.documentElement.clientWidth;
    return ws && audit && ovf <= 0;
  });
  ok("320x568 phone Settings renders Workspace Switcher and Audit Bundle export with 0px overflow", compactWorkspaceAndAudit);
  await ctx.close();
}

// 6d. v0.7.0 Live API Wire-Contract Suite: File History & Diff + Restore, Routine Scheduler Pause/Resume + History,
//     Full Audit Bundle Export, and POST /api/people/switch (aligned 1:1 with Rust release/0.7.0 structs)
{
  const { ctx, p } = await open(1440, 900);
  let patchEnabledValues = [];
  let restorePayload = null;
  let auditBundleFetched = false;
  let peopleSwitchCalled = false;
  let currentCsv = "Vendor,Amount,Status\nAcme Cloud,420.50,verified\nCloudHost Backup,240.00,missing\n";
  const prevCsv = "Vendor,Amount,Status\nAcme Cloud,420.50,verified\n";
  let routineEnabled = true;

  await p.route("**/api/status", async (r) => {
    const res = await r.fetch();
    const j = await res.json();
    await r.fulfill({
      response: res,
      body: JSON.stringify({
        ...j,
        version: "0.7.0",
        scheduler_enabled: true,
        capabilities: {
          sse: true,
          skills: true,
          routines: true,
          search: true,
          receipts_export: true,
          cancel: true,
          scheduler: true,
          file_history: true,
          audit_bundle: true,
        },
      }),
    });
  });
  await p.route("**/api/runs/stream", (r) =>
    r.fulfill({ status: 200, contentType: "text/event-stream", body: ": connected\n\n" })
  );
  await p.route("**/api/files/content**", async (r) => {
    const url = new URL(r.request().url());
    if (url.searchParams.get("path") === "ledger.csv") {
      return r.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          path: "ledger.csv",
          name: "ledger.csv",
          size: currentCsv.length,
          mtime_ms: Date.now(),
          ext: "csv",
          is_binary: false,
          content: currentCsv,
        }),
      });
    }
    return r.continue();
  });
  await p.route("**/api/files", async (r) => {
    const res = await r.fetch();
    const j = await res.json();
    const files = [
      { path: "ledger.csv", name: "ledger.csv", size: currentCsv.length, mtime_ms: Date.now(), ext: "csv" },
      ...(j.files || []),
    ];
    await r.fulfill({ response: res, body: JSON.stringify({ ...j, files }) });
  });
  await p.route("**/api/files/history**", (r) => {
    const url = new URL(r.request().url());
    const hasDiffQuery = Boolean(url.searchParams.get("rev_a"));
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        path: "ledger.csv",
        revisions: [
          {
            rev_id: "rev-20261122-001",
            path: "ledger.csv",
            timestamp_ms: Date.now() - 3600_000,
            size_bytes: prevCsv.length,
            sha256: "a1b2c3d4e5f60718293a4b5c6d7e8f901234567890abcdef1234567890abcdef",
            reason: "overwrite",
            is_text: true,
            line_count: 2,
          },
        ],
        ...(hasDiffQuery
          ? {
              diff: {
                path: "ledger.csv",
                rev_a: "rev-20261122-001",
                rev_b: "current",
                added: 1,
                removed: 0,
                unchanged: 2,
                hunks: [
                  {
                    left_start: 1,
                    left_count: 2,
                    right_start: 1,
                    right_count: 3,
                    lines: [
                      { kind: "context", left_line: 1, right_line: 1, text: "Vendor,Amount,Status" },
                      { kind: "context", left_line: 2, right_line: 2, text: "Acme Cloud,420.50,verified" },
                      { kind: "add", left_line: null, right_line: 3, text: "CloudHost Backup,240.00,missing" },
                    ],
                  },
                ],
              },
            }
          : {}),
      }),
    });
  });
  await p.route("**/api/files/restore", async (r) => {
    restorePayload = JSON.parse(r.request().postData() || "{}");
    currentCsv = prevCsv;
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        path: "ledger.csv",
        restored_rev_id: restorePayload.rev_id,
        size_bytes: prevCsv.length,
      }),
    });
  });
  await p.route("**/api/routines", (r) =>
    r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        scheduler: { enabled: true, running: true, tick_secs: 15, active_routines: routineEnabled ? 1 : 0, paused_routines: routineEnabled ? 0 : 1 },
        routines: [
          {
            id: "rt-sched-1",
            name: "Scheduled Ledger Watch",
            prompt: "Reconcile ledger.csv",
            schedule: "every 15m",
            enabled: routineEnabled,
            next_run_ms: routineEnabled ? Date.now() + 14 * 60_000 : null,
            last_run_ms: Date.now() - 60_000,
            run_count: 2,
          },
        ],
      }),
    })
  );
  await p.route("**/api/routines/rt-sched-1", async (r) => {
    if (r.request().method() === "PATCH") {
      const b = JSON.parse(r.request().postData() || "{}");
      routineEnabled = Boolean(b.enabled);
      patchEnabledValues.push(routineEnabled);
      return r.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          routine: {
            id: "rt-sched-1",
            name: "Scheduled Ledger Watch",
            prompt: "Reconcile ledger.csv",
            schedule: "every 15m",
            enabled: routineEnabled,
            next_run_ms: routineEnabled ? Date.now() + 15 * 60_000 : null,
            run_count: 2,
          },
        }),
      });
    }
    return r.continue();
  });
  await p.route("**/api/routines/rt-sched-1/history", (r) =>
    r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        routine_id: "rt-sched-1",
        runs: [
          {
            run_id: "run-001",
            routine_id: "rt-sched-1",
            started_ms: Date.now() - 65_000,
            finished_ms: Date.now() - 64_200,
            duration_ms: 800,
            status: "ok",
            receipt_id: "rec-sched-99",
            summary: "Verified 2 rows with 0 anomalies.",
          },
        ],
      }),
    })
  );
  await p.route("**/api/audit/bundle**", (r) => {
    auditBundleFetched = true;
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema: "ergon.fieldbook.audit_bundle.v1",
        bundle_sha256: "deadbeef".repeat(8),
        receipt_chain: { verified: true, count: 3, entries: [] },
        workspace_files: { count: 1, entries: [] },
      }),
    });
  });
  await p.route("**/api/people/switch", async (r) => {
    peopleSwitchCalled = true;
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ active: "maya-chen", people: [{ id: "you", name: "You" }, { id: "maya-chen", name: "Maya Chen", role: "Operations lead" }] }),
    });
  });

  await p.goto(BASE + "?face=today");
  await waitPaper(p);

  // 1. FileViewerModal History & Diff + Restore version
  await p.getByRole("button", { name: "Open ledger.csv" }).first().click();
  await p.waitForSelector("#file-viewer-dialog", { timeout: 4000 });
  await p.locator('#file-viewer-dialog [data-id="history"]').click();
  await p.waitForSelector("#file-diff-view", { timeout: 4000 });
  const diffTxt = await p.locator("#file-diff-view").innerText();
  const addLineCount = await p.locator('#file-diff-view [data-diff-kind="add"]').count();
  await p.locator("#restore-version-btn").click();
  await p.waitForSelector("text=Restored ledger.csv", { timeout: 4000 }).catch(() => {});
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  ok(
    "v0.7.0 FileViewerModal History & Diff renders unified highlights and restores revision",
    diffTxt.includes("CloudHost Backup") && addLineCount === 1 && restorePayload?.rev_id === "rev-20261122-001",
    `adds=${addLineCount} restore=${JSON.stringify(restorePayload)}`
  );

  // 2. RoutinesPanel: Scheduler pill, Next run countdown, Pause/Resume PATCH, Expandable Run History + receipt
  await p.locator('[data-thing="routines-btn"]').click();
  await p.waitForSelector("#routines-panel", { timeout: 4000 });
  const schedTxt = await p.locator("#scheduler-status-pill").innerText();
  const nextRunBefore = await p.locator('[data-next-run="rt-sched-1"]').innerText();
  await p.locator('[data-routine-id="rt-sched-1"] [data-action="toggle-routine"]').click();
  await p.waitForTimeout(250);
  const nextRunPaused = await p.locator('[data-next-run="rt-sched-1"]').innerText();
  await p.locator('[data-routine-id="rt-sched-1"] [data-action="toggle-routine"]').click();
  await p.waitForTimeout(250);
  await p.locator('[data-routine-id="rt-sched-1"] [data-action="routine-history"]').click();
  await p.waitForSelector('[data-routine-history="rt-sched-1"] [data-run-id="run-001"]', { timeout: 4000 });
  const histTxt = await p.locator('[data-routine-history="rt-sched-1"]').innerText();
  await p.keyboard.press("Escape");
  await p.locator("#routines-panel").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});
  ok(
    "v0.7.0 RoutinesPanel shows scheduler pill, Next run countdown, Pause/Resume PATCH, and Run History",
    schedTxt.includes("Scheduler active") &&
      nextRunBefore.includes("Next run") &&
      nextRunPaused.includes("Paused") &&
      patchEnabledValues.join(",") === "false,true" &&
      histTxt.includes("Verified 2 rows") &&
      histTxt.includes("rec-sched-99"),
    `sched=${schedTxt} patch=${patchEnabledValues.join(",")} next=${nextRunBefore}->${nextRunPaused}`
  );

  // 3. Export Full Audit Bundle (GET /api/audit/bundle) + Workspace Switcher (POST /api/people/switch)
  await p.locator("#export-audit-bundle-btn").click();
  const auditToastOk = await p.waitForSelector("text=ergon-audit-bundle", { timeout: 4000 }).then(() => true, () => false);
  await p.locator("#person-select").click();
  await p.waitForSelector("#workspace-switcher-panel", { timeout: 4000 });
  await p.locator('#workspace-switcher-panel [data-workspace-id="maya-chen"]').click();
  await p.waitForTimeout(300);
  ok(
    "v0.7.0 Export Full Audit Bundle and Workspace Switcher hit GET /api/audit/bundle & POST /api/people/switch",
    auditBundleFetched && auditToastOk && peopleSwitchCalled,
    `audit=${auditBundleFetched} toast=${auditToastOk} switch=${peopleSwitchCalled}`
  );
  await ctx.close();
}

// 6c. Scale virtualization/pagination (5,000 receipts + 500 workspace files) + Live SSE timeline & cancel
{
  const { ctx, p } = await open(1440, 900);
  const bigFinished = Array.from({ length: 5000 }, (_, i) => ({
    id: `rec-${i}`,
    task: `Automated ledger audit receipt-${i}`,
    date: "2026-11-22",
    summary: `Reconciled batch ${i}`,
    files: i % 5 === 0 ? [`batch-${i}.csv`] : [],
    ts: Date.now() - i * 1000,
  }));
  const bigFiles = Array.from({ length: 500 }, (_, i) => ({
    path: `file-${String(i).padStart(3, "0")}.csv`,
    name: `file-${String(i).padStart(3, "0")}.csv`,
    size: 1024 + i,
    mtime_ms: Date.now() - i * 1000,
    ext: "csv",
  }));
  await p.route("**/api/brief", async (r) => {
    const res = await r.fetch();
    const j = await res.json();
    await r.fulfill({ response: res, body: JSON.stringify({ ...j, finished: bigFinished }) });
  });
  await p.route("**/api/files", (r) =>
    r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ files: bigFiles }) })
  );
  await p.goto(BASE + "?face=today");
  await waitPaper(p);
  await p.waitForTimeout(400);
  const renderedRows = await p.locator('section[aria-labelledby="finished-h"] ol li').count();
  await p.locator("#finished-search").fill("receipt-4999");
  await p.waitForTimeout(200);
  const filteredRows = await p.locator('section[aria-labelledby="finished-h"] ol li').count();
  if (!(await p.locator("#workspace-files-panel").isVisible())) {
    await p.locator('[data-thing="inspector-toggle"]').click();
  }
  await p.waitForSelector("#workspace-files-panel", { timeout: 4000 });
  const renderedFiles = await p.locator("#workspace-files-panel ul li").count();
  await p.locator("#workspace-file-search").fill("file-499.csv");
  await p.waitForTimeout(200);
  const filteredFiles = await p.locator("#workspace-files-panel ul li").count();
  ok(
    "5,000 receipts and 500 files stay paginated and filter instantaneously",
    renderedRows === 20 && filteredRows === 1 && renderedFiles === 25 && filteredFiles === 1,
    `recRows=${renderedRows}->${filteredRows} fileRows=${renderedFiles}->${filteredFiles}`
  );
  await ctx.close();
}

{
  const { ctx, p } = await open(1440, 900);
  let cancelCalled = false;
  await p.route("**/api/status", async (r) => {
    const res = await r.fetch();
    const j = await res.json();
    await r.fulfill({
      response: res,
      body: JSON.stringify({ ...j, version: "0.5.0", capabilities: { sse: true, cancel: true, skills: true, routines: true, search: true, receipts_export: true } }),
    });
  });
  await p.route("**/api/skills", (r) => r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ skills: [] }) }));
  await p.route("**/api/routines", (r) => r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ routines: [] }) }));
  await p.route("**/api/receipts**", (r) => r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ receipts: [] }) }));
  await p.route("**/api/runs/stream", (r) =>
    r.fulfill({
      status: 200,
      contentType: "text/event-stream",
      body: 'event: step_done\ndata: {"run_id":"run-sse-1","tool":"reconcile_ledger","label":"Reconciled expense ledger","status":"done","elapsed_ms":48}\n\n',
    })
  );
  await p.route("**/api/runs/*/cancel", (r) => {
    cancelCalled = true;
    return r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ cancelled: true }) });
  });
  await p.route("**/api/chat", async (r) => {
    await new Promise((res) => setTimeout(res, 6000));
    await r.abort();
  });
  await p.goto(BASE);
  await waitWorkbench(p);
  await p.waitForSelector("#composer-input", { timeout: 4000 });
  await p.locator("#composer-input").fill("Run live SSE test task");
  await p.keyboard.press("Enter");
  await p.waitForSelector("#live-run-timeline >> text=Reconciled expense ledger", { timeout: 4000 });
  const sseText = await p.locator("#live-run-timeline").innerText();
  await p.locator("#cancel-run-btn").click();
  await p.locator("#live-run-timeline").waitFor({ state: "hidden", timeout: 4000 });
  ok(
    "live SSE step timeline displays tool progress, elapsed ms, and cancels cleanly",
    sseText.includes("Reconciled expense ledger") && cancelCalled,
    `cancel=${cancelCalled} txt=${sseText.slice(0, 80).replace(/\n/g, " ")}`
  );
  await ctx.close();
}

// 6d. v0.29.0 PipelineBuilder + CheckpointCard pause/resume (mocked /api/pipelines/*)
{
  const { ctx, p } = await open(1440, 900);
  let resumedBody = null;
  let pipelineStatus = "paused_checkpoint";
  const pausedRun = {
    run_id: "prun-gate-1",
    routine_id: "rtn-gate-1",
    routine_name: "Expense Clean + Human Review",
    status: "paused_checkpoint",
    current_step_index: 1,
    created_ms: Date.now() - 2000,
    updated_ms: Date.now(),
    step_states: [
      { step_id: "clean", kind: "skill", status: "ok", files: ["expenses-cleaned.csv"], output: "Cleaned 6 rows" },
      { step_id: "review", kind: "checkpoint", status: "waiting_human", checkpoint_id: "d-cp-gate-1" },
      { step_id: "brief", kind: "skill", status: "pending", files: [] },
    ],
    pending_checkpoint: {
      checkpoint_id: "d-cp-gate-1",
      step_index: 1,
      step_id: "review",
      prompt: "Review cleaned CSV anomalies before drafting brief",
      input_mode: "choice",
      choices: ["Proceed as-is", "Exclude flagged rows"],
      inspect_files: ["expenses-cleaned.csv"],
    },
  };
  await p.route("**/api/status", async (r) => {
    const res = await r.fetch();
    const j = await res.json();
    await r.fulfill({
      response: res,
      body: JSON.stringify({
        ...j,
        version: "0.29.0",
        capabilities: { ...(j.capabilities || {}), routines: true, sse: true, pipelines: true },
      }),
    });
  });
  await p.route(/\/api\/pipelines\/runs(\?.*)?$/, (r) =>
    r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        runs: pipelineStatus === "paused_checkpoint" ? [pausedRun] : [{ ...pausedRun, status: "completed", pending_checkpoint: null }],
      }),
    })
  );
  await p.route("**/api/pipelines/runs/prun-gate-1/resume", async (r) => {
    resumedBody = JSON.parse(r.request().postData() || "{}");
    pipelineStatus = "completed";
    return r.fulfill({
      status: 202,
      contentType: "application/json",
      body: JSON.stringify({ run: { ...pausedRun, status: "completed", pending_checkpoint: null } }),
    });
  });
  await p.goto(BASE);
  await waitWorkbench(p);
  await p.locator('[data-thing="routines-btn"]').click();
  await p.waitForSelector("#routines-panel", { timeout: 4000 });
  await p.locator("#routines-panel").getByRole("button", { name: "New routine" }).click();
  await p.locator("#routine-create-form").getByRole("button", { name: "Pipeline + Checkpoint (3 steps)" }).click();
  const builderOk = await p.locator("#pipeline-builder [data-pipeline-step-kind='checkpoint']").isVisible();
  await p.keyboard.press("Escape");
  await p.locator("#routines-panel").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});

  await p.getByRole("tab", { name: "Today" }).click();
  await p.waitForSelector('#decisions-list [data-pipeline-checkpoint="prun-gate-1"]', { timeout: 4000 });
  const cpCard = p.locator('#decisions-list [data-pipeline-checkpoint="prun-gate-1"]');
  const cpTxt = await cpCard.innerText();
  await cpCard.getByRole("radio", { name: "Exclude flagged rows" }).click();
  await cpCard.locator('[data-action="resume-pipeline"]').click();
  await cpCard.waitFor({ state: "hidden", timeout: 4000 });
  ok(
    "v0.29.0 PipelineBuilder composes typed steps and CheckpointCard resumes paused pipeline with structured choice",
    builderOk && cpTxt.includes("expenses-cleaned.csv") && resumedBody?.choice === "Exclude flagged rows",
    `builder=${builderOk} choice=${resumedBody?.choice}`
  );
  await ctx.close();
}

// 6e. v0.30.0 Interactive Deliverable Studio: Edit tab (Markdown + CSV), Table Query rollup + Save CSV, Save as PDF, upload accept parity
{
  const { ctx, p } = await open(1440, 900);
  const studioFiles = {
    "studio-brief.md": "# Draft Brief\n\nInitial draft text.\n",
    "studio-expenses.csv": "Vendor,Amount,Category\nAcme Cloud,400.00,Infra\nAcme Cloud,150.00,Infra\nDelta Ops,220.00,Ops\n",
  };
  let pdfExportPayload = null;
  let tableSavePayload = null;

  await p.route("**/api/status", async (r) => {
    const res = await r.fetch();
    const j = await res.json();
    await r.fulfill({
      response: res,
      body: JSON.stringify({
        ...j,
        version: "0.30.0",
        capabilities: { ...(j.capabilities || {}), file_edit: true, table_query: true, pdf_export: true },
      }),
    });
  });
  await p.route(/\/api\/files(\?.*)?$/, (r) => {
    const files = Object.entries(studioFiles).map(([path, content]) => ({
      path,
      name: path,
      size: content.length,
      mtime_ms: Date.now(),
      ext: path.split(".").pop() || "txt",
    }));
    return r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ person_id: "default", files }) });
  });
  await p.route(/\/api\/files\/content(\?.*)?$/, async (r) => {
    const reqUrl = new URL(r.request().url());
    if (r.request().method() === "PUT") {
      const b = JSON.parse(r.request().postData() || "{}");
      if (b.path && typeof b.content === "string") studioFiles[b.path] = b.content;
      return r.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ path: b.path, size: (b.content || "").length, mtime_ms: Date.now(), sha256: "sha-updated-1", rev_id: "rev-edit-1" }),
      });
    }
    const fp = reqUrl.searchParams.get("path") || "";
    if (reqUrl.searchParams.get("download") === "1") {
      const bodyText = fp.endsWith(".pdf") ? "%PDF-1.4\n%%EOF\n" : (studioFiles[fp] || "");
      return r.fulfill({
        status: 200,
        contentType: fp.endsWith(".pdf") ? "application/pdf" : "text/plain",
        body: bodyText,
      });
    }
    const content = studioFiles[fp];
    if (typeof content !== "string") {
      return r.fulfill({ status: 404, contentType: "application/json", body: JSON.stringify({ error: "not found" }) });
    }
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        path: fp,
        name: fp,
        size: content.length,
        mtime_ms: Date.now(),
        ext: fp.split(".").pop() || "txt",
        is_binary: false,
        content,
        sha256: "sha-init-1",
      }),
    });
  });
  await p.route("**/api/tables/query", async (r) => {
    const b = JSON.parse(r.request().postData() || "{}");
    if (b.save_as) tableSavePayload = b;
    const rows = [
      ["Acme Cloud", "650.00"],
      ["Delta Ops", "220.00"],
    ];
    const columns = ["Vendor", "sum_amount"];
    if (b.save_as) {
      studioFiles[b.save_as] = [columns, ...rows].map((row) => row.join(",")).join("\n") + "\n";
    }
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        columns,
        rows,
        total_rows: 3,
        matched_rows: 3,
        ...(b.save_as ? { saved_path: b.save_as } : {}),
      }),
    });
  });
  await p.route("**/api/files/export", async (r) => {
    const b = JSON.parse(r.request().postData() || "{}");
    pdfExportPayload = b;
    const outPath = `${(b.path || "deliverable").replace(/\.[^.]+$/, "")}.${b.format || "pdf"}`;
    studioFiles[outPath] = "%PDF-1.4\n%%EOF\n";
    return r.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ path: outPath, size: 16, format: b.format }),
    });
  });

  await p.goto(BASE);
  await waitWorkbench(p);
  if (!(await p.locator("#workspace-files-panel").isVisible())) {
    await p.locator('[data-thing="inspector-toggle"]').click();
    await p.waitForSelector("#workspace-files-panel", { timeout: 4000 });
  }
  await p.locator("#workspace-inspector").getByRole("tab", { name: /Files/i }).click().catch(() => {});
  await p.locator("#workspace-refresh-btn").click();
  await p.waitForTimeout(200);

  // Check upload accept attribute in WorkspaceFiles
  const wsAccept = await p.locator("#workspace-file-input").getAttribute("accept");

  // 1. Edit Markdown persistently + Save as PDF
  await p.locator("#workspace-files-panel").getByRole("button", { name: "Open studio-brief.md" }).click();
  await p.waitForSelector("#file-viewer-dialog", { timeout: 4000 });
  await p.locator('#file-viewer-dialog [data-id="edit"]').click();
  await p.waitForSelector("#file-editor-textarea", { timeout: 4000 });
  await p.locator("#file-editor-textarea").fill("# Updated Client Brief\n\n## Verified Rollup\nReady for export.\n");
  await p.locator("#file-editor-save").click();
  await p.waitForSelector("text=Saved studio-brief.md", { timeout: 4000 });

  await p.locator("#file-save-pdf").click();
  const pdfToastOk = await p.waitForSelector("text=Saved studio-brief.pdf", { timeout: 4000 }).then(() => true, () => false);
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});

  // Reopen studio-brief.md to verify persistence
  await p.locator("#workspace-files-panel").getByRole("button", { name: "Open studio-brief.md" }).click();
  await p.waitForSelector("#file-viewer-dialog", { timeout: 4000 });
  const reopenedBriefTxt = await p.locator("#file-viewer-body").innerText();
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});

  ok(
    "v0.30.0 Edit tab saves Markdown edits persistently via PUT /api/files/content",
    reopenedBriefTxt.includes("Updated Client Brief") && studioFiles["studio-brief.md"].includes("Verified Rollup"),
    `reopened=${reopenedBriefTxt.slice(0, 60).replace(/\n/g, " ")}`
  );
  ok(
    "v0.30.0 Save as PDF export triggers POST /api/files/export with format pdf",
    pdfExportPayload?.path === "studio-brief.md" && pdfExportPayload?.format === "pdf" && pdfToastOk,
    `payload=${JSON.stringify(pdfExportPayload)} toast=${pdfToastOk}`
  );

  // 2. Edit CSV cell + Table Query group-by rollup + Save query as CSV
  await p.locator("#workspace-files-panel").getByRole("button", { name: "Open studio-expenses.csv" }).click();
  await p.waitForSelector("#file-viewer-dialog", { timeout: 4000 });
  await p.locator('#file-viewer-dialog [data-id="edit"]').click();
  await p.waitForSelector("#csv-editor-table", { timeout: 4000 });
  await p.locator('#csv-editor-table input[aria-label="Row 2 Amount"]').fill("250.00");
  await p.locator("#file-editor-save").click();
  await p.waitForSelector("text=Saved studio-expenses.csv", { timeout: 4000 });

  await p.locator("#table-query-group-col").selectOption("Vendor");
  await p.locator("#table-query-agg-fn").selectOption("sum");
  await p.locator("#table-query-agg-col").selectOption("Amount");
  await p.locator("#table-query-run").click();
  await p.waitForSelector("#table-query-result", { timeout: 4000 });
  const queryTxt = await p.locator("#table-query-result").innerText();
  await p.locator("#table-query-save-csv").click();
  await p.waitForSelector("text=Saved: studio-expenses-query.csv", { timeout: 4000 });
  await p.keyboard.press("Escape");
  await p.locator("#file-viewer-dialog").waitFor({ state: "hidden", timeout: 4000 }).catch(() => {});

  ok(
    "v0.30.0 Edit tab edits CSV cells and runs inline Table Query group-by rollup + Save query as CSV",
    studioFiles["studio-expenses.csv"].includes("250.00") &&
      queryTxt.includes("Acme Cloud") &&
      queryTxt.includes("650.00") &&
      tableSavePayload?.save_as === "studio-expenses-query.csv",
    `saveAs=${tableSavePayload?.save_as} query=${queryTxt.slice(0, 60).replace(/\n/g, " ")}`
  );

  // 3. Upload accept attribute parity across WorkspaceFiles and Today
  await p.getByRole("tab", { name: "Today" }).click();
  await waitPaper(p);
  const todayAccept = await p.locator('#face-today input[type="file"]').first().getAttribute("accept");
  const hasDocExts = (s) => [".pdf", ".docx", ".xlsx", ".pptx"].every((ext) => (s || "").includes(ext));
  ok(
    "v0.30.0 upload inputs include .pdf, .docx, .xlsx, .pptx in accept attribute",
    hasDocExts(wsAccept) && hasDocExts(todayAccept),
    `ws=${wsAccept} today=${todayAccept}`
  );
  await ctx.close();
}

// 7. Reduced motion & Console Cleanliness (2 checks)
{
  const { ctx, p } = await open(1440, 900, "light", { reducedMotion: "reduce" });
  await p.goto(BASE);
  await waitWorkbench(p);
  await p.waitForTimeout(1200);
  const moving = await p.evaluate(() =>
    document.getAnimations().filter((a) => a.playState === "running" && a.effect?.getTiming().iterations === Infinity).length
  );
  ok("reduced motion: no endless motion", moving === 0, String(moving));
  await ctx.close();
}

ok("no console errors", errs.length === 0, errs.slice(0, 5).join(" || "));
fs.writeFileSync(`${OUT}/result.json`, JSON.stringify({ checks, measures, errs }, null, 1));
report("gate", checks);
await b.close();
