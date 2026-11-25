// probe-settings.mjs: Settings-aware theme + SVG probe matching ultra-design/kit/probe.mjs rules
// (opposite prefers-color-scheme, opens Settings dialog to toggle Light/Dark, verifies <html data-theme>
// & darker dark background, verifies Light/Dark is NOT on main chrome, and validates Work Graph SVG labels >= 12px).
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const argv = process.argv.slice(2);
const url = argv.find((a) => /^https?:\/\//.test(a));
const clicks = [];
const opt = { svg: "", fallback: "", "min-px": "12", widths: "1440,1100,900,768", out: "probe-shots" };
for (let i = 0; i < argv.length; i++) {
  const k = argv[i].replace(/^--/, "");
  if (k === "click") clicks.push(argv[++i]);
  else if (k in opt) opt[k] = argv[++i];
}
if (!url) {
  console.error("usage: node probe-settings.mjs <url> [--click <sel>]... [--svg <sel>] [--fallback <sel>]");
  process.exit(2);
}
const req = createRequire(path.resolve("x"));
const found = ["playwright", "playwright-core"].map((m) => { try { return req.resolve(m); } catch { return ""; } }).find(Boolean);
const pw = await import(pathToFileURL(process.env.PLAYWRIGHT || found).href);
const browser = await (pw.chromium || pw.default.chromium).launch({ headless: true, executablePath: process.env.CHROME_PATH || undefined });
fs.mkdirSync(opt.out, { recursive: true });

const lum = ([r, g, b]) => 0.2126 * r + 0.7152 * g + 0.0722 * b;
let fails = 0;

for (const w of opt.widths.split(",").map(Number)) {
  const bg = {};
  for (const theme of ["light", "dark"]) {
    const p = await browser.newPage({ viewport: { width: w, height: 900 }, colorScheme: theme === "dark" ? "light" : "dark" });
    const why = [];
    await p.goto(url);
    await p.waitForTimeout(800);

    // Enforce owner ruling: Appearance must NOT sit exposed on main rail/bar before opening Settings.
    const exposedBefore = await p.locator('[aria-label="Appearance"]').count();
    if (exposedBefore > 0) why.push("Appearance control exposed on main chrome instead of inside Settings");

    // Open Settings dialog, toggle theme, and close Settings.
    if (!(await p.locator('[data-thing="settings-btn"]').isVisible().catch(() => false))) {
      await p.locator('button[aria-label="Toggle conversations rail"]:visible, button[aria-label="Open menu"]:visible').first().click().catch(() => {});
      await p.waitForTimeout(250);
    }
    await p.locator('[data-thing="settings-btn"]').first().click({ timeout: 3000 }).catch(() => why.push("no visible Settings trigger"));
    await p.waitForSelector("#settings-panel", { timeout: 3000 }).catch(() => why.push("Settings dialog did not open"));
    const label = theme === "dark" ? "Dark" : "Light";
    await p.locator('[role="dialog"]').getByRole("button", { name: label, exact: true }).first().click({ timeout: 3000 }).catch(() => why.push(`no visible "${label}" switch inside Settings`));
    await p.keyboard.press("Escape");
    await p.locator('[role="dialog"]').waitFor({ state: "hidden", timeout: 3000 }).catch(() => {});

    for (const c of clicks) await p.locator(c).first().click({ timeout: 3000 }).catch(() => why.push(`could not click ${c}`));
    await p.waitForTimeout(900);

    const r = await p.evaluate(({ svgSel, fallback, minPx }) => {
      const de = document.documentElement;
      const cx = document.createElement("canvas").getContext("2d");
      const srgb = (c) => { cx.clearRect(0, 0, 1, 1); cx.fillStyle = c; cx.fillRect(0, 0, 1, 1); return [...cx.getImageData(0, 0, 1, 1).data]; };
      const painted = [document.body, de].map((e) => getComputedStyle(e).backgroundColor);
      const bgc = painted.find((c) => srgb(c)[3] > 0) || painted[0];
      const visible = (e) => { const b = e.getBoundingClientRect(); return b.width > 0 && b.height > 0 && getComputedStyle(e).visibility !== "hidden"; };
      const svgs = [...document.querySelectorAll(svgSel || "svg")].filter((s) => visible(s) && (svgSel || s.querySelector("text")));
      const out = [], overlap = [], small = [];
      let labels = 0, minSeen = Infinity;
      for (const s of svgs) {
        const box = s.getBoundingClientRect();
        const ts = [...s.querySelectorAll("text")].filter((t) => t.textContent.trim() && visible(t));
        const rects = ts.map((t) => t.getBoundingClientRect());
        ts.forEach((t, i) => {
          labels++;
          const m = t.getScreenCTM();
          const px = parseFloat(getComputedStyle(t).fontSize) * (m ? Math.hypot(m.a, m.b) : 1);
          minSeen = Math.min(minSeen, px);
          if (px < minPx) small.push(`"${t.textContent.slice(0, 24)}" ${px.toFixed(1)}px`);
          const q = rects[i];
          if (q.left < box.left - 1 || q.right > box.right + 1 || q.top < box.top - 1 || q.bottom > box.bottom + 1) out.push(`"${t.textContent.slice(0, 24)}"`);
          for (let j = i + 1; j < ts.length; j++) {
            const o = rects[j];
            if (Math.min(q.right, o.right) - Math.max(q.left, o.left) > 1 && Math.min(q.bottom, o.bottom) - Math.max(q.top, o.top) > 1) {
              overlap.push(`"${t.textContent.slice(0, 16)}" x "${ts[j].textContent.slice(0, 16)}"`);
            }
          }
        });
      }
      return {
        theme: de.dataset.theme || "",
        bg: bgc,
        rgb: srgb(bgc),
        transition: document.querySelectorAll(".transition").length,
        svgs: svgs.length,
        labels,
        minPx: labels ? +minSeen.toFixed(1) : null,
        out,
        overlap,
        small,
        fallback: fallback ? [...document.querySelectorAll(fallback)].filter(visible).length : 0,
      };
    }, { svgSel: opt.svg, fallback: opt.fallback, minPx: +opt["min-px"] });

    bg[theme] = lum(r.rgb);
    if (r.theme !== theme) why.push(`clicked ${theme} but <html data-theme="${r.theme}">`);
    if (r.transition) why.push(`${r.transition} element(s) with class "transition"`);
    if (opt.svg && !r.svgs && !r.fallback) why.push(`no ${opt.svg} drawn`);
    if (opt.svg && r.svgs && !r.labels) why.push(`${opt.svg} has no <text> labels`);
    if (r.out.length) why.push(`labels outside svg: ${r.out.join(", ")}`);
    if (r.overlap.length) why.push(`labels overlap: ${r.overlap.join(", ")}`);
    if (r.small.length) why.push(`labels under ${opt["min-px"]}px: ${r.small.join(", ")}`);
    await p.screenshot({ path: path.join(opt.out, `probe-${w}-${theme}.png`), fullPage: true });
    await p.close();
    if (why.length) fails++;
    console.log(`${why.length ? "FAIL" : "ok  "} ${w} ${theme} [data-theme=${r.theme}] bg ${r.bg}; svg ${r.svgs ? `${r.svgs} drawn, ${r.labels} labels, smallest ${r.minPx}px` : "not drawn"}${why.length ? "  " + why.join("; ") : ""}`);
  }
  if (!(bg.dark < bg.light)) {
    fails++;
    console.log(`FAIL ${w} dark background is not darker than light (${bg.dark.toFixed(0)} vs ${bg.light.toFixed(0)})`);
  }
}

await browser.close();
console.log(fails ? `FAIL ${fails}` : "PASS");
process.exit(fails ? 1 : 0);
