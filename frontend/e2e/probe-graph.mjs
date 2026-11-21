// probe-graph.mjs: Work graph map at 1440 / 1100 / 900 / 768, light: every label inside the map, no label over the
// next column's pins, rendered label size, and the label backing fully opaque. Shots to .out/graph/graph-*.png.
import { BASE, OUT as E2E_OUT, launch, report } from './fixture.mjs'
import fs from 'node:fs'
const OUT = `${E2E_OUT}/graph`
fs.mkdirSync(OUT, { recursive: true })
const b = await launch()
const checks = []
for (const [w, scheme] of [[1440, 'light'], [1440, 'dark'], [1100, 'light'], [900, 'light'], [768, 'light']]) {
  const p = await b.newPage({ viewport: { width: w, height: 900 }, colorScheme: scheme })
  await p.goto(BASE); await p.waitForTimeout(600)
  await p.getByRole('button', { name: scheme === 'dark' ? 'Dark' : 'Light', exact: true }).first().click().catch(() => p.getByRole('radio', { name: scheme === 'dark' ? 'Dark' : 'Light' }).first().click())
  await p.locator('[role="tab"][data-id="graph"]').click(); await p.waitForTimeout(900)
  const theme = await p.evaluate(() => document.documentElement.dataset.theme || document.documentElement.className)
  const r = await p.evaluate(() => {
    const svg = document.querySelector('svg[role="group"]'); if (!svg || !svg.getBoundingClientRect().width) return { hidden: true, note: /map is drawn on wider screens/.test(document.body.innerText), list: document.querySelectorAll('ol[aria-label="Work graph as a list"] li').length }
    const vb = svg.viewBox.baseVal, scale = svg.getBoundingClientRect().width / vb.width
    const pins = [...svg.querySelectorAll('g[role="button"]')].map(g => ({ x: +(g.querySelector('circle,polygon,rect').getBBox().x), box: g.querySelector('circle,polygon,rect').getBBox() }))
    const bandXs = [...new Set(pins.map(p => Math.round(p.box.x + p.box.width / 2)))].sort((a, b) => a - b)
    const out = [], over = []
    for (const g of svg.querySelectorAll('g[role="button"]')) {
      const t = g.querySelector('text'), bg = g.querySelector('rect:not(:first-of-type)') || g.querySelectorAll('rect')[g.querySelectorAll('rect').length - 1]
      const tb = t.getBBox(), right = tb.x + tb.width
      if (right > vb.width - 4) out.push(`${t.textContent} right=${right.toFixed(0)}`)
      const next = bandXs.find(x => x > tb.x)
      if (next !== undefined && right > next - 10) over.push(`${t.textContent} right=${right.toFixed(0)} next=${next}`)
    }
    const backs = [...svg.querySelectorAll('g[role="button"] rect')].filter(r => r.getAttribute('rx')).map(r => r.getAttribute('opacity') || '1')
    const truncated = [...svg.querySelectorAll('g[role="button"] text')].filter(t => t.textContent.endsWith('…')).length
    return { svgW: Math.round(svg.getBoundingClientRect().width), scale: +scale.toFixed(3), fontPx: +(13 * scale).toFixed(1), bandXs, out, over, backs: [...new Set(backs)], truncated, labels: svg.querySelectorAll('g[role="button"] text').length }
  })
  const bad = r.hidden ? !(r.note && r.list > 0) : (r.out.length || r.over.length || r.fontPx < 11 || r.backs.some(o => o !== '1'))
  checks.push({ name: `graph ${w} ${scheme}`, pass: !bad })
  console.log(`${w} ${scheme} [${theme}]`, JSON.stringify(r), bad ? 'FAIL' : 'ok')
  await p.screenshot({ path: `${OUT}/graph-${w}-${scheme}.png` })
  await p.close()
}
await b.close(); report('graph', checks)
