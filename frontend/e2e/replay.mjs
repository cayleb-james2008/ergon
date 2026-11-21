// replay.mjs: Ergon's share of the step 9 worst cases, fed through browser page.route mocks only (odd, long and
// malformed data must not blank or break the page). Run through `npm run check:ui` on fresh fixture data; shots to .out/replay.
import { BASE, OUT as E2E_OUT, launch, report } from './fixture.mjs'
import fs from 'node:fs'
const OUT = `${E2E_OUT}/replay`
fs.mkdirSync(OUT, { recursive: true })
const b = await launch()
const res = []
const ok = (n, p, d = '') => { res.push({ n, p: !!p, d }); console.log((p ? 'PASS ' : 'FAIL ') + n + (d ? '  ' + d : '')) }
const LONG = 'x'.repeat(260)
const clipped = p => p.evaluate(() => [...document.querySelectorAll('body *')].filter(e => { const r = e.getBoundingClientRect(); if (!r.width || r.right <= innerWidth + 1) return false; for (let q = e.parentElement; q; q = q.parentElement) { const o = getComputedStyle(q).overflowX; if (o !== 'visible') return false } return true }).length)
const textOverflow = (p, sel) => p.evaluate(s => [...document.querySelectorAll(s)].filter(e => e.scrollWidth > e.clientWidth + 1).length, sel)
async function page(w, url, routes, theme = 'light') {
  const ctx = await b.newContext({ viewport: { width: w, height: w < 768 ? 844 : 900 } })
  const p = await ctx.newPage(); p.errs = []
  p.on('pageerror', e => p.errs.push(e.message))
  for (const [pat, fn] of routes) await p.route(pat, fn)
  await p.goto(url); await p.waitForTimeout(2200)
  return { ctx, p }
}
const json = (r, body, status = 200) => r.fulfill({ status, contentType: 'application/json', body: typeof body === 'string' ? body : JSON.stringify(body) })
const patch = (fn) => async r => { const res = await r.fetch(); const j = await res.json(); r.fulfill({ response: res, body: JSON.stringify(fn(j)) }) }

for (const w of [1440, 390]) {
  { const { ctx, p } = await page(w, BASE, [['**/api/people', patch(j => ({ ...j, people: [...j.people, { id: 'n1', name: null, role: null }, { id: 'n2', name: 42, role: '' }] }))]])
    ok(`ergon ${w}: odd person names do not blank the page`, !p.errs.length && await p.locator('main h1').count() === 1, p.errs.join('|')); await ctx.close() }
  { const { ctx, p } = await page(w, BASE, [['**/api/decisions', patch(j => ({ decisions: j.decisions.map(d => ({ ...d, summary: 'Delete ' + LONG + '.txt' })) }))], ['**/api/brief', patch(j => ({ ...j, finished: j.finished.map(f => ({ ...f, task: 'Task ' + LONG })) }))]])
    ok(`ergon ${w}: long unbroken decision and titles stay in their boxes`, (await clipped(p)) === 0 && (await textOverflow(p, '#lead-h, main ol p')) === 0); await p.screenshot({ path: `${OUT}/ergon-long-${w}.png` }); await ctx.close() }
  { const { ctx, p } = await page(w, BASE, [['**/api/brief', r => r.fulfill({ status: 200, contentType: 'application/json', body: '{"finished": [tr' })]])
    ok(`ergon ${w}: malformed brief shows the error`, await p.locator("text=Today's paper could not be loaded").count() === 1); await ctx.close() }
  { const { ctx, p } = await page(w, BASE, [['**/api/decisions', patch(j => ({ decisions: j.decisions.map(d => ({ ...d, kind: 'question', summary: 'Use the new supplier list?' })) }))]])
    const band = await p.locator('section[aria-labelledby=lead-h]').innerText()
    ok(`ergon ${w}: a question decision says the answer is recorded; Decline`, /recorded/.test(band) && await p.getByRole('button', { name: 'Decline', exact: true }).count() === 1 && !/went ahead/.test(band)); await ctx.close() }
  // A 200 the reader cannot parse turns into {} (call() in lib/api.ts): the list must fail into its own error, not crash the page.
  { const { ctx, p } = await page(w, BASE, [['**/api/threads', r => r.request().method() === 'GET' ? json(r, '{"threads": [tr') : r.continue()]])
    await p.getByRole('tab', { name: /Conversations/ }).click().catch(() => {}); await p.waitForTimeout(600)
    ok(`ergon ${w}: malformed conversation list shows its error`, !p.errs.length && (w < 768 || await p.locator('text=Could not load your conversations').count() === 1), p.errs.join('|')); await ctx.close() }
  { const { ctx, p } = await page(w, BASE, [['**/api/people', r => r.request().method() === 'GET' ? json(r, '{"people": [tr') : r.continue()]])
    // The paper renders its side column twice (wide and phone layouts, one hidden): count the visible one.
    ok(`ergon ${w}: malformed people list shows its error on the paper`, !p.errs.length && await p.locator('#team-h ~ p[role=alert]:visible', { hasText: 'Could not load the people list' }).count() === 1, p.errs.join('|')); await ctx.close() }
}
await b.close()
report('replay', res)
