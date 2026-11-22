// probe-graph.mjs: Work Graph SVG map at 1440 / 1100 / 900 / 768 (light & dark): every label inside the map,
// no label over the next column's pins, rendered label size >= 12px, opaque label backings, plus List-only toggle.
import { BASE, OUT as E2E_OUT, launch, report } from "./fixture.mjs";
import fs from "node:fs";
const OUT = `${E2E_OUT}/graph`;
fs.mkdirSync(OUT, { recursive: true });
const b = await launch();
const checks = [];

for (const [w, scheme] of [[1440, "light"], [1440, "dark"], [1100, "light"], [900, "light"], [768, "light"]]) {
  const p = await b.newPage({ viewport: { width: w, height: 900 }, colorScheme: scheme });
  await p.goto(BASE);
  await p.waitForTimeout(600);
  await p.getByRole("button", { name: scheme === "dark" ? "Dark" : "Light", exact: true }).first().click();
  await p.locator('[role="tab"][data-id="graph"]').click();
  await p.waitForTimeout(900);
  const theme = await p.evaluate(() => document.documentElement.dataset.theme || "");
  const r = await p.evaluate(() => {
    const svg = document.querySelector("svg#graph-svg");
    if (!svg || !svg.getBoundingClientRect().width) {
      return {
        hidden: true,
        note: /map is drawn on wider screens/.test(document.body.innerText),
        list: document.querySelectorAll('ol[aria-label="Work graph as a list"] li').length,
      };
    }
    const vb = svg.viewBox.baseVal;
    const scale = svg.getBoundingClientRect().width / vb.width;
    const pins = [...svg.querySelectorAll("g[data-pin]")].map((g) => {
      const box = g.querySelector("circle,polygon,rect").getBBox();
      return { x: +box.x, box };
    });
    const bandXs = [...new Set(pins.map((pt) => Math.round(pt.box.x + pt.box.width / 2)))].sort((a, bb) => a - bb);
    const out = [], over = [];
    for (const g of svg.querySelectorAll("g[data-pin]")) {
      const t = g.querySelector("text");
      const tb = t.getBBox();
      const right = tb.x + tb.width;
      if (right > vb.width - 2) out.push(`${t.textContent} right=${right.toFixed(0)}`);
      const next = bandXs.find((x) => x > tb.x);
      if (next !== undefined && right > next - 8) over.push(`${t.textContent} right=${right.toFixed(0)} next=${next}`);
    }
    const backs = [...svg.querySelectorAll("g[data-pin] rect")].filter((rc) => rc.getAttribute("rx")).map((rc) => rc.getAttribute("opacity") || "1");
    return {
      svgW: Math.round(svg.getBoundingClientRect().width),
      scale: +scale.toFixed(3),
      fontPx: +(13 * scale).toFixed(1),
      bandXs,
      out,
      over,
      backs: [...new Set(backs)],
      labels: svg.querySelectorAll("g[data-pin] text").length,
    };
  });
  const bad = r.hidden || theme !== scheme || r.out.length || r.over.length || r.fontPx < 12 || r.backs.some((o) => o !== "1") || r.labels === 0;
  checks.push({ name: `graph ${w} ${scheme}`, pass: !bad });
  console.log(`${w} ${scheme} [${theme}]`, JSON.stringify(r), bad ? "FAIL" : "ok");
  await p.screenshot({ path: `${OUT}/graph-${w}-${scheme}.png` });
  await p.close();
}

// Structured list toggle check
{
  const p = await b.newPage({ viewport: { width: 1440, height: 900 } });
  await p.goto(BASE);
  await p.waitForTimeout(500);
  await p.locator('[role="tab"][data-id="graph"]').click();
  await p.waitForTimeout(700);
  await p.getByRole("button", { name: "List only", exact: true }).click();
  await p.waitForTimeout(300);
  const listOnlyOk = (await p.locator("svg#graph-svg").count()) === 0 && (await p.locator("#graph-list li").count()) > 0;
  await p.getByRole("button", { name: "Map & list", exact: true }).click();
  await p.waitForTimeout(300);
  const mapBackOk = (await p.locator("svg#graph-svg").count()) === 1;
  checks.push({ name: "graph list-only toggle", pass: listOnlyOk && mapBackOk });
  console.log(`list-only toggle: ${listOnlyOk && mapBackOk ? "ok" : "FAIL"}`);
  await p.close();
}

await b.close();
report("graph", checks);
