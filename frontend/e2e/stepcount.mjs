// Step-count probe: measures the clicks/keystrokes/screens/decisions for Ergon's
// three most frequent jobs, on the fixture. Baseline for the quality loop.
// Run from frontend/: node e2e/stepcount.mjs   (fixture must be startable)
import { BASE, launch, reset } from './fixture.mjs'

const b = await launch()

async function steps(fn) {
  let clicks = 0, keys = 0, decisions = 0
  const screens = new Set()
  const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } })
  const p = await ctx.newPage()
  const click = async (loc) => { clicks++; await loc.click() }
  const key = async (k) => { keys++; await p.keyboard.press(k) }
  const track = (s) => screens.add(s)
  await fn({ p, click, key, track, dec: () => decisions++ })
  await ctx.close()
  return { clicks, keys, screens: screens.size, decisions }
}

const results = {}

// Job 1: Ask the agent to finish a task
await reset()
results['ask_task'] = await steps(async ({ p, click, key, track }) => {
  await p.goto(BASE); await p.waitForSelector('main h1'); track('paper')
  await click(p.getByRole('button', { name: 'Ask your agent for something' })); track('chat')
  await p.locator('#composer-input').fill('Write a short supplier summary document')
  await key('Enter')
  await p.waitForFunction(() => /supplier summary/i.test(document.querySelector('#thread')?.textContent || ''), null, { timeout: 30000 }).catch(() => {})
})

// Job 2: Review what's waiting (approve)
await reset()
results['review_waiting'] = await steps(async ({ p, click, track }) => {
  await p.goto(BASE); await p.waitForSelector('main h1'); track('paper')
  await click(p.getByRole('button', { name: 'Approve', exact: true }))
  await p.waitForTimeout(600)
})

// Job 3: Open a finished job's conversation
await reset()
results['open_finished'] = await steps(async ({ p, click, track }) => {
  await p.goto(BASE); await p.waitForSelector('main h1'); track('paper')
  await click(p.getByRole('button', { name: /Open the conversation: Make a Q1 budget spreadsheet/ })); track('chat')
  await p.waitForSelector('#thread', { timeout: 5000 }).catch(() => {})
})

await b.close()
console.log(JSON.stringify(results, null, 2))