// replay.mjs: runs the shared worst-case cases (cases.mjs) at 1440px and 390px via browser page.route mocks.
import { BASE, OUT as E2E_OUT, launch, report } from "./fixture.mjs";
import makeCases from "./cases.mjs";
import fs from "node:fs";

const OUT = `${E2E_OUT}/replay`;
fs.mkdirSync(OUT, { recursive: true });
const b = await launch();
const res = [];
const ok = (n, p, d = "") => { res.push({ n, p: !!p, d }); console.log((p ? "PASS " : "FAIL ") + n + (d ? "  " + d : "")); };
const LONG = "x".repeat(260);

const h = {
  LONG,
  json: (r, body, status = 200) => r.fulfill({ status, contentType: "application/json", body: typeof body === "string" ? body : JSON.stringify(body) }),
  patch: (fn) => async (r) => { try { const res0 = await r.fetch(); const j = await res0.json(); await r.fulfill({ response: res0, body: JSON.stringify(fn(j)) }); } catch {} },
  malformed: (r) => r.fulfill({ status: 200, contentType: "application/json", body: '{"items": [tr' }),
  down: (r) => r.abort("connectionrefused"),
  overflowing: async (p, sel) => {
    const n = await p.evaluate((s) => {
      const els = [...document.querySelectorAll(s)];
      return els.length ? els.filter((e) => e.scrollWidth > e.clientWidth + 1).length : -1;
    }, sel);
    if (n < 0) throw new Error(`selector matched nothing: ${sel}`);
    return n;
  },
  has: async (p, text) => (await p.getByText(text).count()) > 0,
};

const clipped = (p) => p.evaluate(() =>
  [...document.querySelectorAll("body *")].filter((e) => {
    const r = e.getBoundingClientRect();
    if (!r.width || r.right <= innerWidth + 1) return false;
    for (let q = e.parentElement; q; q = q.parentElement) {
      if (getComputedStyle(q).overflowX !== "visible") return false;
    }
    return true;
  }).length
);

const cases = makeCases(h);
for (const w of [1440, 390]) {
  for (const c of cases) {
    const ctx = await b.newContext({ viewport: { width: w, height: w < 768 ? 844 : 900 } });
    const p = await ctx.newPage();
    const errs = [];
    p.on("pageerror", (e) => errs.push(e.message));
    let pendingRoutes = 0;
    let resolveIdle = null;
    const waitRoutesIdle = () =>
      pendingRoutes === 0
        ? Promise.resolve()
        : new Promise((res) => {
            const prev = resolveIdle;
            resolveIdle = () => {
              if (prev) prev();
              res();
            };
          });
    for (const [pat, fn] of c.routes) {
      await p.route(pat, async (r) => {
        pendingRoutes += 1;
        try {
          await fn(r);
        } finally {
          pendingRoutes = Math.max(0, pendingRoutes - 1);
          if (pendingRoutes === 0 && resolveIdle) {
            const done = resolveIdle;
            resolveIdle = null;
            done();
          }
        }
      });
    }
    await p.goto(BASE + (c.path || ""), { waitUntil: "domcontentloaded" });
    await p.locator("main h1, [role=alert], [role=tablist]").first().waitFor({ state: "visible", timeout: 10_000 });
    if (c.waitFor) await c.waitFor(p, h);
    if (c.act) await c.act(p, h);
    await Promise.race([waitRoutesIdle(), new Promise((r) => setTimeout(r, 2000))]);
    await p.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
    const clip = await clipped(p);
    const bodyTxt = await p.evaluate(() => document.body.innerText);
    const badWords = /\bundefined\b|\bNaN\b|\[object Object\]/.test(bodyTxt);
    const checkRes = c.check ? await c.check(p, h) : true;
    const pass = errs.length === 0 && clip === 0 && !badWords && checkRes === true;
    ok(`ergon ${w}: ${c.name}`, pass, [errs.join("|"), clip ? `clipped:${clip}` : "", badWords ? "badWords" : "", checkRes === true ? "" : String(checkRes)].filter(Boolean).join(" "));
    if (c.shot) await p.screenshot({ path: `${OUT}/${c.name.replace(/\W+/g, "-").slice(0, 48)}-${w}.png` });
    await p.unrouteAll({ behavior: "ignoreErrors" }).catch(() => {});
    await ctx.close();
  }
}

await b.close();
report("replay", res);
