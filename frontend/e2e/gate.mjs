// gate.mjs: Ergon Morning paper gate. Measures (overflow, 44px targets, names, contrast, min font) on the paper face
// at 1440/768/390/320 in light and dark, then every interaction, states (loading, empty, error), console, reduced motion.
// Run through `npm run check:ui`, which builds and starts the fixture first. Shots and result.json go to .out/gate.
import { BASE, OUT as E2E_OUT, RUN, launch, reset, report } from './fixture.mjs'
import fs from 'node:fs'
const OUT = `${E2E_OUT}/gate`, ORIGIN = new URL(BASE).origin
fs.mkdirSync(OUT, { recursive: true })
const b = await launch()
const MEASURE = () => {
  const vis = el => { const r = el.getBoundingClientRect(); const s = getComputedStyle(el); return r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && s.display !== 'none' }
  const cv = document.createElement('canvas'); cv.width = cv.height = 1
  const cx = cv.getContext('2d', { willReadFrequently: true })
  const rgba = c => { cx.clearRect(0, 0, 1, 1); cx.fillStyle = '#000'; cx.fillStyle = c; cx.fillRect(0, 0, 1, 1); const d = cx.getImageData(0, 0, 1, 1).data; return [d[0], d[1], d[2], d[3] / 255] }
  const lin = v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }
  const L = ([r, g, bb]) => 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(bb)
  const bgOf = el => { for (let e = el; e; e = e.parentElement) { const c = rgba(getComputedStyle(e).backgroundColor); if (c[3] > 0.5) return c.slice(0, 3) } return [255, 255, 255] }
  const inter = [...document.querySelectorAll('a[href],button,input,select,textarea,[role=button],summary,[tabindex]:not([tabindex="-1"])')].filter(vis)
  const name = el => (el.getAttribute('aria-label') || el.innerText || el.value || el.getAttribute('placeholder') || '').trim()
  const min = innerWidth < 768 ? 44 : 24; const small = inter.filter(e => { const r = e.getBoundingClientRect(); return r.width < min || r.height < min }).map(e => `${e.tagName}:${name(e).slice(0, 30)}:${Math.round(e.getBoundingClientRect().width)}x${Math.round(e.getBoundingClientRect().height)}`)
  const unnamed = inter.filter(e => !name(e) && !(e.id && document.querySelector(`label[for="${e.id}"]`))).map(e => e.outerHTML.slice(0, 90))
  const low = []; let minFont = 99; let minRatio = 99
  for (const el of document.querySelectorAll('body *')) {
    if (!vis(el) || ![...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim())) continue
    const fs = parseFloat(getComputedStyle(el).fontSize); minFont = Math.min(minFont, fs)
    const fg = rgba(getComputedStyle(el).color); const bg = bgOf(el)
    const a = fg[3]; const mix = [0, 1, 2].map(i => fg[i] * a + bg[i] * (1 - a))
    const l1 = L(mix), l2 = L(bg); const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05)
    minRatio = Math.min(minRatio, ratio)
    if (ratio < (fs >= 18.66 ? 3 : 4.5)) low.push(`${ratio.toFixed(2)}:${(el.innerText || '').trim().slice(0, 30)}`)
  }
  return { docOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth, interactive: inter.length, smallTargets: small.length, smallSample: small.slice(0, 20), unnamed: unnamed.slice(0, 4), lowContrast: low.length, lowSample: low.slice(0, 6), minRatio: Math.round(minRatio * 100) / 100, minFont }
}

const errs = [], checks = []
const ok = (name, pass, detail = '') => { checks.push({ name, pass: !!pass, detail }); console.log((pass ? 'PASS ' : 'FAIL ') + name + (detail ? '  ' + detail : '')) }
async function open(w, h, theme = 'light', extra = {}) {
  const ctx = await b.newContext({ viewport: { width: w, height: h }, colorScheme: theme, hasTouch: w < 768, ...extra })
  await ctx.addInitScript((t) => { try { localStorage.setItem('ergon-theme', t) } catch {} }, theme)
  const p = await ctx.newPage()
  p.on('pageerror', e => errs.push(`${w}-${theme}: pageerror ${e.message}`))
  p.on('console', m => { if (m.type() === 'error' && !(p.__expect500 && /500/.test(m.text()))) errs.push(`${w}-${theme}: ${m.text().slice(0, 140)} @ ${m.location().url}`) })
  return { ctx, p }
}
const headline = p => p.locator('main h1').ariaSnapshot().then(t => (t.match(/heading "(.*)"/) || [])[1] || t)
const waitPaper = p => p.waitForSelector('main h1', { timeout: 15000 })

// 1. measurements
const measures = []
for (const theme of ['light', 'dark']) for (const [vn, w, h] of [['1440', 1440, 900], ['768', 768, 1024], ['390', 390, 844], ['320', 320, 640]]) {
  const { ctx, p } = await open(w, h, theme)
  await p.goto(BASE); await waitPaper(p); await p.waitForTimeout(700)
  const m = await p.evaluate(MEASURE); m.size = vn; m.theme = theme
  // the paper scrolls inside its face: measure overflow of the scrolling column too
  m.innerOverflow = await p.evaluate(() => Math.max(0, ...[...document.querySelectorAll('.overflow-y-auto')].map(e => e.scrollWidth - e.clientWidth)))
  measures.push(m)
  await p.screenshot({ path: `${OUT}/paper-${vn}-${theme}.png` })
  await ctx.close()
}
for (const m of measures) ok(`measure ${m.size} ${m.theme}`, m.docOverflow <= 0 && m.innerOverflow <= 0 && m.smallTargets === 0 && m.lowContrast === 0 && m.unnamed.length === 0,
  `overflow ${m.docOverflow}/${m.innerOverflow} small ${m.smallTargets} ${m.smallSample.join(' | ')} low ${m.lowContrast} ${m.lowSample.join(' | ')} unnamed ${m.unnamed.length} minFont ${m.minFont} minRatio ${m.minRatio}`)

// 2. interactions (desktop)
await reset()
{
  const { ctx, p } = await open(1440, 900)
  await p.goto(BASE); await waitPaper(p)
  ok('headline counts', (await headline(p)) === '3 things finished, 1 waits on you.', await headline(p))
  ok('waiting job not filed as finished', !(await p.locator('main ol').innerText()).includes('Clean up the old draft'))
  await p.getByRole('button', { name: 'Approve', exact: true }).click()
  await p.waitForSelector('text=Approved and done', { timeout: 5000 }).then(() => ok('approve toast', true), () => ok('approve toast', false))
  await p.waitForFunction(() => /nothing waits on you/.test(document.querySelector('main h1')?.textContent || ''), null, { timeout: 5000 }).catch(() => {})
  ok('approve re-counts headline', /nothing waits on you\.$/.test(await headline(p)), await headline(p))
  ok('tab chip clears', !(await p.locator('[role="tab"][data-id="paper"]').innerText()).match(/\d/), await p.locator('[role="tab"][data-id="paper"]').innerText())
  await p.screenshot({ path: `${OUT}/after-approve-1440.png` })
  // open a finished job's conversation
  await p.getByRole('button', { name: /Open the conversation: Make a Q1 budget spreadsheet/ }).click()
  await p.waitForSelector('#thread', { timeout: 5000 })
  await p.waitForTimeout(600)
  ok('open goes to the conversation', (await p.locator('#thread').innerText()).includes('Q1 budget'), (await p.locator('#thread').innerText()).slice(0, 80))
  ok('rail shows on Conversations', await p.locator('aside[aria-label="Ergon navigation"]').isVisible())
  await p.screenshot({ path: `${OUT}/conversation-1440.png` })
  await p.locator('[role="tab"][data-id="graph"]').click(); await p.waitForTimeout(800)
  await p.screenshot({ path: `${OUT}/graph-1440.png` })
  ok('graph face renders', (await p.locator('body').innerText()).length > 50)
  await p.locator('[role="tab"][data-id="paper"]').click(); await waitPaper(p)
  ok('paper hides the rail', !(await p.locator('aside[aria-label="Ergon navigation"]').count()))
  // team switch + offer
  await p.getByRole('button', { name: /Maya Chen/ }).click()
  await p.waitForFunction(() => /Maya Chen/.test(document.querySelector('main')?.parentElement?.parentElement?.innerText || ''), null, { timeout: 5000 }).catch(() => {})
  await p.waitForTimeout(800)
  ok('dateline follows the team switch', (await p.locator('#dateline').innerText()).includes('Maya Chen'), await p.locator('#dateline').innerText())
  const offer = p.getByRole('button', { name: 'Make it one tap' })
  ok('Maya has an offer', await offer.count() > 0)
  await p.screenshot({ path: `${OUT}/maya-1440.png` })
  if (await offer.count()) {
    await offer.first().click()
    await p.waitForSelector('text=Added to your suite as a one-tap app.', { timeout: 5000 }).then(() => ok('offer accept toast', true), () => ok('offer accept toast', false))
    await p.waitForTimeout(600)
    ok('offer leaves the list', await p.getByRole('button', { name: 'Make it one tap' }).count() === 0)
  }
  // theme toggle
  await p.getByRole('button', { name: /Dark/ }).first().click()
  ok('theme toggles', (await p.evaluate(() => document.documentElement.getAttribute('data-theme'))) === 'dark')
  await ctx.close()
}
// 3. keep path + empty person
await reset()
{
  const { ctx, p } = await open(1440, 900)
  await p.goto(BASE); await waitPaper(p)
  await p.getByRole('button', { name: 'Keep the file', exact: true }).click()
  await p.waitForSelector('text=You said no', { timeout: 5000 }).then(() => ok('keep toast', true), () => ok('keep toast', false))
  await p.waitForTimeout(800)
  ok('keep clears the wait', /nothing waits on you\.$/.test(await headline(p)), await headline(p))
  await ctx.close()
}
await reset()
await fetch(BASE + 'api/people', { method: 'POST', headers: { 'content-type': 'application/json', origin: ORIGIN }, body: JSON.stringify({ name: 'Ada Brooks', role: 'Office manager' }) }).then(r => ok('add person via API', r.ok, String(r.status)))
{
  const { ctx, p } = await open(1440, 900)
  await p.goto(BASE); await waitPaper(p)
  await p.getByRole('button', { name: /Ada Brooks/ }).click(); await p.waitForTimeout(1200)
  ok('empty person headline', (await headline(p)) === 'Nothing finished yet.', await headline(p))
  ok('empty state offers a first task', await p.getByRole('button', { name: 'Ask your agent for something' }).count() === 1)
  await p.screenshot({ path: `${OUT}/empty-1440.png` })
  await p.getByRole('button', { name: 'Ask your agent for something' }).click(); await p.waitForTimeout(500)
  ok('empty CTA opens a new conversation', await p.locator('#thread').count() === 1)
  await ctx.close()
}
// 3b. the paper reprints after a run, repeated titles keep their own threads, stopped runs are listed
await reset()
{
  const work = `${RUN}/data/work.json`
  const w = JSON.parse(fs.readFileSync(work, 'utf8'))
  w.entries.push({ answered: false, files: [], finished: false, summary: 'I stopped before finishing: the model did not answer.', task: 'Draft the supplier email', thread_id: 't' + Date.now() + '-9', ts: Date.now() })
  fs.writeFileSync(work, JSON.stringify(w))
  const { ctx, p } = await open(1440, 900)
  await p.goto(BASE); await waitPaper(p)
  ok('ask button on a full paper', await p.getByRole('button', { name: 'Ask your agent for something' }).count() === 1)
  ok('stopped run listed under Did not finish', (await p.locator('section[aria-labelledby=stopped-h]').innerText().catch(() => '')).includes('Draft the supplier email'))
  await p.screenshot({ path: `${OUT}/stopped-1440.png` })
  await p.getByRole('button', { name: 'Ask your agent for something' }).click(); await p.waitForTimeout(500)
  await p.locator('#composer-input').fill('Write a short supplier summary document')
  await p.keyboard.press('Enter')
  await p.waitForFunction(() => /supplier summary/i.test(document.querySelector('#thread')?.textContent || ''), null, { timeout: 30000 }).catch(() => {})
  await p.waitForTimeout(1000)
  await p.locator('[role="tab"][data-id="paper"]').click(); await p.waitForTimeout(1500)
  ok('the paper reprints after a run', (await headline(p)).startsWith('4 things finished'), await headline(p))
  await ctx.close()
  const people = await (await fetch(BASE + 'api/people')).json()
  await fetch(BASE + 'api/active-person', { method: 'POST', headers: { 'content-type': 'application/json', origin: ORIGIN }, body: JSON.stringify({ id: 'maya-chen' }) })
  const brief = await (await fetch(BASE + 'api/brief')).json()
  const ids = brief.finished.filter(x => x.task === 'Summarise the weekly ops notes').map(x => x.thread_id)
  ok('repeated titles keep their own threads', ids.length === 3 && new Set(ids).size === 3, ids.join(' '))
  await fetch(BASE + 'api/active-person', { method: 'POST', headers: { 'content-type': 'application/json', origin: ORIGIN }, body: JSON.stringify({ id: people.active }) })
}
// 4. error and loading states
{
  const { ctx, p } = await open(1440, 900)
  p.__expect500 = true; await p.route('**/api/brief', r => r.fulfill({ status: 500, body: 'no' }))
  await p.goto(BASE); await p.waitForSelector('text=could not be loaded', { timeout: 8000 }).catch(() => {})
  ok('error state', await p.locator('text=Today\'s paper could not be loaded').count() === 1)
  await p.screenshot({ path: `${OUT}/error-1440.png` })
  await p.unroute('**/api/brief')
  await p.getByRole('button', { name: 'Retry' }).click(); await p.waitForFunction(() => /finished/.test(document.querySelector('main h1')?.textContent || ''), null, { timeout: 8000 }).catch(() => {})
  ok('retry recovers', /finished/.test(await headline(p).catch(() => '')))
  await ctx.close()
}
{
  const { ctx, p } = await open(1440, 900)
  await p.route('**/api/brief', async r => { await new Promise(s => setTimeout(s, 2500)); r.continue() })
  await p.goto(BASE); await p.waitForTimeout(700)
  ok('loading state', await p.locator('[aria-busy="true"]').count() > 0)
  await p.screenshot({ path: `${OUT}/loading-1440.png` })
  await ctx.close()
}
// 5. phone: drawer, appearance inside it, focus return
{
  const { ctx, p } = await open(390, 844)
  await p.goto(BASE); await waitPaper(p)
  await p.getByRole('button', { name: 'Open menu' }).click(); await p.waitForTimeout(600)
  ok('phone drawer opens', await p.locator('[role="dialog"]').isVisible())
  ok('appearance in the drawer', await p.locator('[role="dialog"] [aria-label="Appearance"]').count() === 1)
  await p.screenshot({ path: `${OUT}/drawer-390.png` })
  await p.keyboard.press('Escape'); await p.waitForTimeout(600)
  ok('focus returns to Menu', await p.evaluate(() => document.activeElement?.getAttribute('aria-label')) === 'Open menu')
  ok('phone side column under the paper', await p.getByRole('heading', { name: 'Team' }).isVisible().catch(() => false) || await p.getByRole('heading', { name: 'Team' }).count() > 0)
  await ctx.close()
}
// 6. reduced motion: nothing keeps moving
{
  const { ctx, p } = await open(1440, 900, 'light', { reducedMotion: 'reduce' })
  await p.goto(BASE); await waitPaper(p); await p.waitForTimeout(1500)
  const moving = await p.evaluate(() => document.getAnimations().filter(a => a.playState === 'running' && (a.effect?.getTiming().iterations === Infinity)).length)
  ok('reduced motion: no endless motion', moving === 0, String(moving))
  await ctx.close()
}
ok('no console errors', errs.length === 0, errs.slice(0, 5).join(' || '))
fs.writeFileSync(`${OUT}/result.json`, JSON.stringify({ checks, measures, errs }, null, 1))
report('gate', checks)
await b.close()
