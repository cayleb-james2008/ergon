// fixture.mjs: build Ergon, run it on free ports against seeded fixture data and a stub model, stop it.
// Everything it writes (data, logs, pid files, shots) goes to frontend/e2e/.out.
//   node e2e/fixture.mjs start   start on fresh data and print the URL      node e2e/fixture.mjs stop
import { chromium } from 'playwright-core'
import { execFileSync, spawn } from 'node:child_process'
import fs from 'node:fs'
import net from 'node:net'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export const E2E = path.dirname(fileURLToPath(import.meta.url)), ROOT = path.resolve(E2E, '../..')
export const OUT = path.join(E2E, '.out'), RUN = path.join(OUT, 'run')
const STATE = path.join(RUN, 'ports.json'), FIX = path.join(E2E, 'fixtures')
const BIN = path.join(process.env.CARGO_TARGET_DIR || path.join(ROOT, 'target'), 'debug/ergon')
// Playwright comes from the frontend's own devDependencies; the browser is the installed Chrome for Testing build.
export const CHROME = process.env.E2E_CHROME || path.join(os.homedir(), '.cache/ms-playwright/chromium-1234/chrome-linux64/chrome')
export const ZERO_GPU_ARGS = ['--disable-gpu', '--disable-gpu-compositing', '--disable-gpu-sandbox', '--disable-accelerated-2d-canvas', '--disable-accelerated-video-decode', '--num-raster-threads=1']
export const launch = (opts = {}) => chromium.launch({ headless: true, executablePath: CHROME, ...opts, args: [...ZERO_GPU_ARGS, ...(opts.args || [])] })

let st = fs.existsSync(STATE) ? JSON.parse(fs.readFileSync(STATE, 'utf8')) : null
export let BASE = st && `http://127.0.0.1:${st.app}/`

export const results = []
export function report(name, checks) {
  const pass = checks.filter(c => c.pass ?? c.p).length
  results.push({ name, pass, total: checks.length })
  console.log(`\n${name}: ${pass}/${checks.length} pass`)
  if (pass !== checks.length) process.exitCode = 1
}

export function build() {
  const run = (cmd, args, cwd) => execFileSync(cmd, args, { cwd, stdio: 'inherit' })
  run('npm', ['run', 'build'], path.join(ROOT, 'frontend'))
  // the binary embeds assets/index.html with include_str: touch the sources so cargo rebuilds
  const now = new Date()
  for (const f of fs.readdirSync(path.join(ROOT, 'src'))) if (f.endsWith('.rs')) fs.utimesSync(path.join(ROOT, 'src', f), now, now)
  run('cargo', ['build', '--offline'], ROOT)
}

const sleep = ms => new Promise(r => setTimeout(r, ms))
const listening = port => new Promise(res => { const s = net.connect(port, '127.0.0.1'); s.once('connect', () => { s.destroy(); res(true) }); s.once('error', () => res(false)) })
const freePort = () => new Promise((res, rej) => { const s = net.createServer().once('error', rej).listen(0, '127.0.0.1', () => { const { port } = s.address(); s.close(() => res(port)) }) })
const tryPort = port => new Promise(res => { const s = net.createServer().once('error', () => res(0)).listen(port, '127.0.0.1', () => s.close(() => res(port))) })
async function pickPort(taken) {
  for (let p = 18780; p <= 18789; p++) if (!taken.includes(p) && await tryPort(p)) return p
  for (;;) { const p = await freePort(); if (p !== 9230 && !taken.includes(p)) return p }
}

// Only ever signal a pid this fixture wrote, and only while its command line is still the one it started.
function alive(name, mark) {
  const f = path.join(RUN, `${name}.pid`)
  if (!fs.existsSync(f)) return 0
  const pid = Number(fs.readFileSync(f, 'utf8'))
  try { return fs.readFileSync(`/proc/${pid}/cmdline`, 'utf8').includes(mark) ? pid : 0 } catch { return 0 }
}
async function kill(name, mark) {
  const pid = alive(name, mark)
  if (pid) {
    try { process.kill(-pid, 'SIGTERM') } catch {}
    for (let i = 0; i < 30 && alive(name, mark); i++) await sleep(100)
    if (alive(name, mark)) try { process.kill(-pid, 'SIGKILL') } catch {}
    for (let i = 0; i < 20 && alive(name, mark); i++) await sleep(100)
  }
  fs.rmSync(path.join(RUN, `${name}.pid`), { force: true })
}
function launchProc(name, cmd, args, env) {
  const log = fs.openSync(path.join(RUN, `${name}.log`), 'w')
  const child = spawn(cmd, args, { cwd: ROOT, env, detached: true, stdio: ['ignore', log, log] })
  fs.closeSync(log); child.unref()
  fs.writeFileSync(path.join(RUN, `${name}.pid`), String(child.pid))
}

// The seed was written once; move each person's times up to just now so both the 48-hour brief window and Today's stream hold it.
function shiftGroup(files) {
  const docs = files.map(f => [f, JSON.parse(fs.readFileSync(f, 'utf8'))])
  let min = Infinity, max = 0
  const walk = (v, fn) => { if (Array.isArray(v)) v.forEach(x => walk(x, fn)); else if (v && typeof v === 'object') for (const k of Object.keys(v)) { if (k === 'ts' && typeof v[k] === 'number') v[k] = fn(v[k]); else walk(v[k], fn) } }
  for (const [, d] of docs) walk(d, t => (min = Math.min(min, t), max = Math.max(max, t), t))
  if (!max) return
  const now = Date.now(), utcDayStart = Math.floor(now / 86_400_000) * 86_400_000, span = Math.max(0, max - min)
  const targetMax = now - utcDayStart > span + 10_000 ? Math.max(now - 2 * 60_000, utcDayStart + span + 2_000) : now - 2_000
  const delta = targetMax - max
  for (const [f, d] of docs) { walk(d, t => t + delta); fs.writeFileSync(f, JSON.stringify(d)) }
}
function shiftTimes(dir) {
  const rootFiles = fs.readdirSync(dir, { withFileTypes: true }).filter(e => e.isFile() && e.name.endsWith('.json')).map(e => path.join(dir, e.name))
  shiftGroup(rootFiles)
  const peopleDir = path.join(dir, 'people')
  if (fs.existsSync(peopleDir)) {
    for (const p of fs.readdirSync(peopleDir, { withFileTypes: true })) {
      if (!p.isDirectory()) continue
      const pFiles = fs.readdirSync(path.join(peopleDir, p.name)).filter(n => n.endsWith('.json')).map(n => path.join(peopleDir, p.name, n))
      shiftGroup(pFiles)
    }
  }
}

export async function start({ fresh = false } = {}) {
  fs.mkdirSync(RUN, { recursive: true })
  if (!st) { const app = await pickPort([]); st = { app, stub: await pickPort([app]) }; fs.writeFileSync(STATE, JSON.stringify(st)) }
  BASE = `http://127.0.0.1:${st.app}/`
  await kill('app', BIN)
  for (let i = 0; i < 20 && await listening(st.app); i++) await sleep(200)
  if (await listening(st.app)) throw new Error(`port ${st.app} is still held`)
  if (fresh || !fs.existsSync(path.join(RUN, 'data'))) {
    fs.rmSync(path.join(RUN, 'data'), { recursive: true, force: true })
    fs.cpSync(path.join(FIX, 'data'), path.join(RUN, 'data'), { recursive: true })
    shiftTimes(path.join(RUN, 'data'))
    await kill('stub', RUN) // the stub walks its replies in order: a fresh run starts it again
  }
  for (const d of ['home', 'tmp']) fs.mkdirSync(path.join(RUN, d), { recursive: true })
  const env = { PATH: '/usr/bin:/bin', HOME: path.join(RUN, 'home'), TMPDIR: path.join(RUN, 'tmp'), TZ: 'UTC', LANG: 'C.UTF-8' }
  if (!alive('stub', RUN)) {
    fs.copyFileSync(path.join(FIX, 'replies.json'), path.join(RUN, 'replies.json'))
    launchProc('stub', process.execPath, [path.join(FIX, 'stub-model.mjs'), String(st.stub), path.join(RUN, 'replies.json')], env)
    for (let i = 0; i < 30 && !await listening(st.stub); i++) await sleep(100)
  }
  launchProc('app', BIN, [], { ...env, ERGON_PORT: String(st.app), ERGON_DATA_DIR: path.join(RUN, 'data'), ERGON_MODEL_BASE_URL: `http://127.0.0.1:${st.stub}`, ERGON_MODEL_API_KEY: 'stub', ERGON_UPDATE_MODE: 'off' })
  for (let i = 0; i < 50; i++) {
    if (await fetch(BASE + 'api/status').then(r => r.ok, () => false)) return BASE
    await sleep(300)
  }
  throw new Error('Ergon did not come up:\n' + fs.readFileSync(path.join(RUN, 'app.log'), 'utf8').slice(-800))
}
export const reset = () => start({ fresh: true })

export async function stop() {
  await kill('app', BIN); await kill('stub', RUN)
  fs.rmSync(STATE, { force: true }); st = null; BASE = null
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const cmd = process.argv[2]
  if (cmd === 'start') console.log(await start({ fresh: true }))
  else if (cmd === 'stop') await stop()
  else { console.error('usage: node e2e/fixture.mjs start|stop'); process.exit(2) }
}
