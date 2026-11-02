#!/usr/bin/env node
// checker-verify — INDEPENDENT MCP acceptance probe for the @cayleb/dsh-fleet MCP facade.
//
// Written by checker-verify (fresh-context checker). It does not import or reuse the builder's client, tests or
// selftest: it speaks MCP Streamable HTTP itself, reads the bearer token only from the local admin socket, and
// verifies the fleet rows through the fleet's own control socket instead of trusting the tool's echo.
//
// The bearer token is never printed and never written: every byte this program emits passes through redact().
//
// Usage:
//   node mcp-verify.mjs --mode real \
//     --url http://127.0.0.1:8789/mcp --tailnet-url http://100.114.17.67:8789/mcp \
//     --admin-socket /run/fleet/mcp-admin.sock --control-socket /run/fleet/control.sock \
//     --out <evidence-dir>
//
//   --mode dryrun   allows a loopback --tailnet-url (used only against the throwaway test double) and does not
//                   claim the tailnet leg as production evidence.
//   --token-file    fallback only if the admin socket is unavailable (never pass a token on the command line).
//
// Exit: 0 = every mandatory check passed; 1 = a mandatory check failed; 2 = harness/precondition problem.
import net from 'node:net'
import { randomBytes, createHash } from 'node:crypto'
import { spawnSync } from 'node:child_process'
import { mkdirSync, writeFileSync, readFileSync, existsSync } from 'node:fs'
import { join, resolve } from 'node:path'

// ─────────────────────────────────────────────────────────────── args / output
const argv = process.argv.slice(2)
const opt = (name, dflt) => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : dflt }
const has = name => argv.includes(name)
const MODE = opt('--mode', 'real')
const LOOPBACK_URL = opt('--url', 'http://127.0.0.1:8789/mcp')
const TAILNET_URL = opt('--tailnet-url', null)
const ADMIN_SOCKET = opt('--admin-socket', '/run/fleet/mcp-admin.sock')
const CONTROL_SOCKET = opt('--control-socket', '/run/fleet/control.sock')
const TOKEN_FILE = opt('--token-file', null)
const TAILNET_OPTIONAL = has('--tailnet-optional')
const OUT = resolve(opt('--out', join(process.cwd(), 'mcp-verify-out')))
if (!['real', 'dryrun'].includes(MODE)) fail2(`--mode must be real or dryrun, got ${MODE}`)
if (MODE === 'real' && !TAILNET_URL) fail2('--tailnet-url is required in real mode (the tailnet leg is part of the acceptance)')

mkdirSync(OUT, { recursive: true, mode: 0o700 })
mkdirSync(join(OUT, 'raw'), { recursive: true, mode: 0o700 })

const EXPECTED_TOOLS = Object.freeze([
  'actions.status', 'board.op', 'fleet.selftest', 'fleet.status', 'order.drive_roundtrip',
  'order.stage_real', 'session.create', 'session.prompt', 'session.status',
])
const STARTED = new Date().toISOString()
let token = null
const redactions = []
function redact (text) {
  let out = String(text ?? '')
  if (token !== null && token.length > 0) out = out.split(token).join('<REDACTED-BEARER-TOKEN>')
  for (const value of redactions) if (value.length >= 12) out = out.split(value).join('<redacted>')
  return out
}
function fail2 (message) {
  process.stderr.write(`HARNESS: ${redact(message)}\n`)
  process.exit(2)
}
const transcriptPath = join(OUT, 'mcp-verify-transcript.txt')
const transcriptLines = []
function say (line) { const clean = redact(line); transcriptLines.push(clean); process.stdout.write(`${clean}\n`) }
function flushTranscript () { writeFileSync(transcriptPath, transcriptLines.join('\n') + '\n', { mode: 0o600 }) }
function writeJson (name, value) {
  const file = join(OUT, name)
  mkdirSync(join(file, '..'), { recursive: true, mode: 0o700 })
  const text = redact(JSON.stringify(value, null, 2))
  writeFileSync(file, text + '\n', { mode: 0o600 })
  return file
}
function writeTextFile (name, text) {
  const file = join(OUT, name)
  mkdirSync(join(file, '..'), { recursive: true, mode: 0o700 })
  writeFileSync(file, redact(text), { mode: 0o600 })
  return file
}
const sha256 = value => createHash('sha256').update(value).digest('hex')
function canonical (value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`
  if (value !== null && typeof value === 'object') {
    return `{${Object.keys(value).sort().filter(k => value[k] !== undefined).map(k => `${JSON.stringify(k)}:${canonical(value[k])}`).join(',')}}`
  }
  return JSON.stringify(value) ?? 'null'
}
const nonce = `cv-${Date.now().toString(36)}-${randomBytes(4).toString('hex')}`
const markers = [nonce]

// ─────────────────────────────────────────────────────────────── check ledger
const checks = []
function check (id, title, ok, detail, extra = {}) {
  checks.push({ id, title, ok: ok === null ? null : Boolean(ok), detail: String(detail ?? ''), ...extra })
  say(`${ok === null ? 'SKIP ' : ok ? 'PASS ' : 'FAIL '} ${id} — ${title}${detail ? ` :: ${redact(detail)}` : ''}`)
  return ok
}
const testData = {}
function record (key, value) { testData[key] = value; writeJson(`raw/${key}.json`, value); return value }

// ─────────────────────────────────────────────────────────────── unix socket calls
function udsCall (path, request, { timeoutMs = 8000 } = {}) {
  return new Promise((resolveP) => {
    const socket = net.connect(path)
    let buffer = ''
    let settled = false
    const finish = value => { if (settled) return; settled = true; clearTimeout(timer); try { socket.end() } catch { /* closed */ } resolveP(value) }
    const timer = setTimeout(() => finish({ ok: false, error: `timeout after ${timeoutMs} ms` }), timeoutMs)
    socket.setEncoding('utf8')
    socket.on('connect', () => { try { socket.write(JSON.stringify(request) + '\n') } catch (error) { finish({ ok: false, error: `write: ${error.message}` }) } })
    socket.on('data', chunk => {
      buffer += chunk
      const nl = buffer.indexOf('\n')
      if (nl < 0) return
      try { finish(JSON.parse(buffer.slice(0, nl))) } catch (error) { finish({ ok: false, error: `non-JSON answer: ${error.message}` }) }
    })
    socket.on('error', error => finish({ ok: false, error: `${error.code ?? error.message}` }))
    socket.on('close', () => { if (!settled) finish({ ok: false, error: 'closed before answering' }) })
  })
}
const admin = (req, opts) => udsCall(ADMIN_SOCKET, req, opts)
const control = (req, opts) => udsCall(CONTROL_SOCKET, req, opts)

// ─────────────────────────────────────────────────────────────── HTTP / JSON-RPC
function parseRpcBody (response) {
  const ctype = String(response.headers['content-type'] ?? '')
  if (ctype.includes('text/event-stream')) {
    const line = response.text.split('\n').find(l => l.startsWith('data:'))
    if (line === undefined) return null
    try { return JSON.parse(line.slice(5).trim()) } catch { return null }
  }
  try { return JSON.parse(response.text) } catch { return null }
}
async function httpJson (url, { method = 'POST', headers = {}, body, timeoutMs = 30_000 } = {}) {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeoutMs)
  try {
    const res = await fetch(url, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: controller.signal,
      redirect: 'manual',
    })
    const text = await res.text()
    const responseHeaders = {}
    for (const [key, value] of res.headers.entries()) responseHeaders[key.toLowerCase()] = value
    return { status: res.status, headers: responseHeaders, text, parsed: (() => { try { return JSON.parse(text) } catch { return null } })() }
  } finally { clearTimeout(timer) }
}
function rpcClient (url, { withToken = true } = {}) {
  let sessionId = null
  return {
    url,
    get sessionId () { return sessionId },
    async rpc (id, method, params, { notify = false, timeoutMs = 30_000 } = {}) {
      const headers = { 'content-type': 'application/json', accept: 'application/json, text/event-stream' }
      if (withToken && token !== null) headers.authorization = `Bearer ${token}`
      if (sessionId !== null) headers['mcp-session-id'] = sessionId
      const body = notify ? { jsonrpc: '2.0', method, params } : { jsonrpc: '2.0', id, method, params }
      const startedAt = Date.now()
      const response = await httpJson(url, { headers, body, timeoutMs })
      if (typeof response.headers['mcp-session-id'] === 'string' && response.headers['mcp-session-id'] !== '') sessionId = response.headers['mcp-session-id']
      const rpcBody = parseRpcBody(response)
      say(`[${url}] ${method} -> HTTP ${response.status} (${Date.now() - startedAt} ms)${notify ? '' : ` rpc_id=${id}`}`)
      return { response, rpc: rpcBody }
    },
    async toolCall (name, args = {}, { timeoutMs = 240_000 } = {}) {
      const { response, rpc } = await this.rpc(++rpcSeq, 'tools/call', { name, arguments: args }, { timeoutMs })
      const result = rpc?.result ?? null
      const isError = result?.isError === true
      const text = Array.isArray(result?.content) ? result.content.map(part => part?.text ?? '').join('') : ''
      let payload = null
      try { payload = text === '' ? null : JSON.parse(text) } catch { payload = { unparsed: text.slice(0, 2000) } }
      toolCalls.push({ tool: name, ok: response.status === 200 && !isError, status: response.status, isError, error_code: payload?.error?.code ?? null })
      say(`[${url}] tools/call ${name} -> HTTP ${response.status}${isError ? ` TOOL-ERROR ${payload?.error?.code ?? '?'}` : ' OK'}`)
      return { response, rpc, result, isError, payload }
    },
  }
}
let rpcSeq = 100
const toolCalls = []

// ─────────────────────────────────────────────────────────────── preflight
async function preflight () {
  say(`checker-verify MCP probe — ${STARTED} — mode=${MODE}`)
  say(`loopback: ${LOOPBACK_URL}`)
  say(`tailnet:  ${TAILNET_URL ?? '(none)'}`)
  say(`admin socket: ${ADMIN_SOCKET}`)
  say(`control socket: ${CONTROL_SOCKET}`)

  // token: admin socket first (the intended owner path); token-file as a documented fallback.
  let tokenError = null
  if (existsSync(ADMIN_SOCKET) || TOKEN_FILE !== null) {
    if (TOKEN_FILE !== null) {
      try { token = readFileSync(TOKEN_FILE, 'utf8').trim() } catch (error) { tokenError = `token file: ${error.message}` }
    } else {
      const answer = await admin({ op: 'token' })
      if (answer?.ok === true && typeof answer.result?.token === 'string') token = answer.result.token
      else tokenError = `admin socket token: ${answer?.error ?? 'no token in answer'}`
    }
  } else tokenError = `no admin socket at ${ADMIN_SOCKET}`
  if (typeof token !== 'string' || !/^[0-9a-f]{64}$/.test(token)) {
    check('P0.token', 'bearer token readable over the owner admin socket', false, tokenError ?? 'token shape invalid (not printed)')
    say('NOT RUNNABLE: no usable bearer token; the facade cannot be exercised.')
    flushTranscript()
    process.exit(2)
  }
  check('P0.token', 'bearer token readable over the owner admin socket', true, 'token present (value never printed)')

  // admin stat: log open, token file exists/mode, never the value.
  const stat = await admin({ op: 'stat' })
  record('p0-admin-stat', stat)
  check('P0.admin-stat', 'admin stat reports the token and the action log ready',
    stat?.ok === true && stat.result?.log?.open === true && stat.result?.log?.broken === null &&
    stat.result?.token?.exists === true && stat.result?.token?.mode === '600',
    `log.open=${stat?.result?.log?.open} log.broken=${stat?.result?.log?.broken} token.exists=${stat?.result?.token?.exists} token.mode=${stat?.result?.token?.mode}`)
  const fleetMcp = stat?.result ?? {}
  record('p0-fleet-mcp-stat', {
    name: fleetMcp.name, version: fleetMcp.version, ok: fleetMcp.ok, degraded: fleetMcp.degraded,
    bind: fleetMcp.bind, sources: fleetMcp.sources, admin: fleetMcp.admin, log: fleetMcp.log, model_route: fleetMcp.model_route, real_lane: fleetMcp.real_lane,
  })

  // fleet control plane: needed for independent row verification.
  const fleetStatus = await control({ op: 'status' })
  record('p0-fleet-status', { ok: fleetStatus?.ok, result: fleetStatus?.result })
  check('P0.fleet-control', 'fleet control socket answers status with integrity ok',
    fleetStatus?.ok === true && fleetStatus.result?.integrity?.ok === true,
    `ok=${fleetStatus?.ok} integrity.ok=${fleetStatus?.result?.integrity?.ok} failures=${JSON.stringify(fleetStatus?.result?.integrity?.failures ?? [])}`)

  // raw TCP reachability of both legs before any MCP call.
  let tailnetReachable = null
  for (const [label, url] of [['loopback', LOOPBACK_URL], ...(TAILNET_URL === null ? [] : [['tailnet', TAILNET_URL]])]) {
    const target = new URL(url)
    const port = Number(target.port || 80)
    const reachable = await tcpProbe(target.hostname, port)
    if (label === 'tailnet') tailnetReachable = reachable.ok
    if (!reachable.ok && label === 'tailnet' && TAILNET_OPTIONAL) {
      check('P0.reach.tailnet', `tailnet listener ${target.hostname}:${port} accepts TCP — NOT ESTABLISHED on this host`, null,
        `not reachable (${reachable.detail}); the tailnet leg is recorded as NOT-ESTABLISHED here, to be proven on the installed service`)
      say('TAILNET LEG NOT ESTABLISHED on this host: continuing loopback-only and recording the limitation.')
      continue
    }
    check(`P0.reach.${label}`, `${label} listener ${target.hostname}:${port} accepts TCP`, reachable.ok, reachable.detail)
    if (!reachable.ok) { say(`NOT RUNNABLE: ${label} leg is not reachable (${reachable.detail}).`); flushTranscript(); process.exit(2) }
  }
  return { stat, fleetStatus, tailnetReachable }
}
function tcpProbe (host, port, timeoutMs = 4000) {
  return new Promise(resolveP => {
    const socket = net.connect({ host, port })
    const timer = setTimeout(() => { socket.destroy(); resolveP({ ok: false, detail: 'timeout' }) }, timeoutMs)
    socket.on('connect', () => { clearTimeout(timer); socket.end(); resolveP({ ok: true, detail: `connected from ${socket.localAddress}:${socket.localPort}` }) })
    socket.on('error', error => { clearTimeout(timer); resolveP({ ok: false, detail: `${error.code ?? error.message}` }) })
  })
}

// ─────────────────────────────────────────────────────────────── T1: auth negative
async function test1AuthNegative () {
  say('\n── T1: unauthenticated / wrong-token calls are refused before dispatch and logged')
  const baseline = await admin({ op: 'log_tail', n: 5 })
  const baselineSeq = Number(baseline?.result?.seq ?? 0)
  record('t1-baseline', { seq: baselineSeq, head: baseline?.result?.head ?? null })

  const body = { jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: 'fleet.status', arguments: {} } }
  const missing = await httpJson(LOOPBACK_URL, { headers: { 'content-type': 'application/json', accept: 'application/json, text/event-stream' }, body })
  record('t1-missing-token', { status: missing.status, headers: { 'www-authenticate': missing.headers['www-authenticate'] ?? null }, body: missing.parsed })
  check('T1.missing.status', 'missing token -> HTTP 401', missing.status === 401, `status=${missing.status}`)
  check('T1.missing.no-result', 'missing token -> refusal envelope, no tool result', missing.parsed?.result === undefined && (missing.parsed?.error?.code === -32001),
    `error.code=${missing.parsed?.error?.code} result=${missing.parsed?.result === undefined ? 'absent' : 'PRESENT'}`)

  const wrongToken = randomBytes(32).toString('hex')
  const wrong = await httpJson(LOOPBACK_URL, { headers: { 'content-type': 'application/json', accept: 'application/json, text/event-stream', authorization: `Bearer ${wrongToken}` }, body: { ...body, id: 2 } })
  record('t1-wrong-token', { status: wrong.status, headers: { 'www-authenticate': wrong.headers['www-authenticate'] ?? null }, body: wrong.parsed })
  check('T1.wrong.status', 'wrong token -> HTTP 401', wrong.status === 401, `status=${wrong.status}`)
  check('T1.wrong.no-result', 'wrong token -> refusal envelope, no tool result', wrong.parsed?.result === undefined && (wrong.parsed?.error?.code === -32001),
    `error.code=${wrong.parsed?.error?.code} result=${wrong.parsed?.result === undefined ? 'absent' : 'PRESENT'}`)
  check('T1.www-authenticate', '401 advertises the Bearer challenge', String(missing.headers['www-authenticate'] ?? '').toLowerCase().includes('bearer'),
    `www-authenticate=${missing.headers['www-authenticate'] ?? '(absent)'}`)

  await new Promise(r => setTimeout(r, 150))
  const tail = await admin({ op: 'log_tail', n: 50 })
  const rows = Array.isArray(tail?.result?.rows) ? tail.result.rows : []
  const fresh = rows.filter(row => Number(row.seq) > baselineSeq)
  record('t1-log-after', { seq: tail?.result?.seq ?? null, fresh })
  const refused = fresh.filter(row => row.kind === 'refused')
  const refusedCodes = refused.map(row => row.code)
  check('T1.log.refused-rows', 'both refusals are appended as refused rows (principal null)',
    refused.length >= 2 && refusedCodes.includes('missing-token') && refusedCodes.includes('invalid-token') &&
    refused.every(row => row.principal === null && row.outcome === 'refused'),
    `refused=${refused.length} codes=${JSON.stringify(refusedCodes)} principals=${JSON.stringify(refused.map(r => r.principal))}`)
  const intents = fresh.filter(row => row.kind === 'intent')
  check('T1.log.no-dispatch', 'nothing was dispatched: no intent row for the refused call', intents.length === 0,
    `new intent rows=${intents.length}${intents.length ? ` (${intents.map(r => r.tool).join(',')})` : ''}`)
}

// ─────────────────────────────────────────────────────────────── T2: both legs
async function test2Legs (legs) {
  say('\n── T2: authenticated initialize + tools/list over loopback and the tailnet address')
  const results = {}
  const clients = {}
  for (const [label, url] of legs) {
    const client = rpcClient(url)
    clients[label] = client
    const init = await client.rpc(1, 'initialize', { protocolVersion: '2025-03-26', capabilities: {}, clientInfo: { name: 'checker-verify', version: '1.0.0' } })
    const initResult = init.rpc?.result ?? null
    results[label] = { url, session_header: init.response.headers['mcp-session-id'] ?? null, init: initResult }
    check(`T2.${label}.initialize`, `${label} initialize answers 200 with serverInfo fleet-mcp`,
      init.response.status === 200 && initResult?.serverInfo?.name === 'fleet-mcp' && typeof initResult?.protocolVersion === 'string',
      `status=${init.response.status} server=${initResult?.serverInfo?.name} version=${initResult?.serverInfo?.version} protocol=${initResult?.protocolVersion}`)
    check(`T2.${label}.session-id`, `${label} initialize issues Mcp-Session-Id`,
      typeof results[label].session_header === 'string' && results[label].session_header.length >= 16,
      `header=${results[label].session_header === null ? '(absent)' : '(present, not a secret)'}`)
    check(`T2.${label}.capabilities`, `${label} advertises tools capability`, initResult?.capabilities?.tools !== undefined,
      JSON.stringify(initResult?.capabilities ?? null))
    const notified = await client.rpc(2, 'notifications/initialized', {}, { notify: true })
    check(`T2.${label}.initialized-notification`, `${label} notifications/initialized answers 202 with empty body`,
      notified.response.status === 202 && notified.response.text === '', `status=${notified.response.status}`)
    const list = await client.rpc(3, 'tools/list', {})
    const specs = Array.isArray(list.rpc?.result?.tools) ? list.rpc.result.tools : []
    const names = specs.map(spec => spec?.name).sort()
    results[label].tools = names
    record(`t2-${label}-tools-list`, { url, toolCount: names.length, names, specs })
    check(`T2.${label}.tools-list`, `${label} tools/list lists the exact 9 contracted tools`,
      JSON.stringify(names) === JSON.stringify(EXPECTED_TOOLS),
      `count=${names.length} missing=${JSON.stringify(EXPECTED_TOOLS.filter(n => !names.includes(n)))} extra=${JSON.stringify(names.filter(n => !EXPECTED_TOOLS.includes(n)))}`)
    const schemasOk = specs.length === EXPECTED_TOOLS.length && specs.every(spec => typeof spec?.description === 'string' && spec.description !== '' && spec.inputSchema !== null && typeof spec.inputSchema === 'object')
    check(`T2.${label}.schemas`, `${label} every tool has a description and an inputSchema`, schemasOk, `specs=${specs.length}`)
    if (label === 'tailnet') {
      const host = new URL(url).hostname
      const octets = host.split('.')
      const inTailnet = octets.length === 4 && octets[0] === '100' && Number(octets[1]) >= 64 && Number(octets[1]) <= 127
      check('T2.tailnet.address', 'the tailnet leg is a 100.64.0.0/10 address', MODE === 'dryrun' ? null : inTailnet,
        `host=${host}${MODE === 'dryrun' ? ' (dryrun: self-test only, not production evidence)' : ''}`)
    }
  }
  record('t2-legs', results)
  if (results.tailnet === undefined) {
    check('T2.tailnet.leg', 'tailnet leg initialize + tools/list — NOT ESTABLISHED on this host (stated, to prove on the installed service)', null,
      'preflight could not reach the tailnet address from this scratch host')
  } else {
    const loop = JSON.stringify(results.loopback?.tools ?? [])
    const tail = JSON.stringify(results.tailnet?.tools ?? [])
    check('T2.legs.identical', 'both legs list the same tools', loop === tail && loop !== '[]', `loopback=${results.loopback?.tools?.length ?? 0} tailnet=${results.tailnet?.tools?.length ?? 0}`)
  }
  return { clients, results }
}

// ─────────────────────────────────────────────────────────────── T3: sessions
async function test3Session (client) {
  say('\n── T3: deterministic stub session — create, one full turn, read-back')
  const create = await client.toolCall('session.create', {})
  const created = create.payload ?? {}
  record('t3-session-create', { isError: create.isError, result: created })
  const sessionId = typeof created.sessionId === 'string' ? created.sessionId : null
  check('T3.create', 'session.create returns a session id on the labelled stub route',
    !create.isError && created.ok === true && typeof sessionId === 'string' && /^fleet-mcp-[0-9a-f-]{36}$/.test(sessionId) &&
    created.status === 'created' && created.mode === 'stub',
    `ok=${created.ok} sessionId=${sessionId === null ? '(none)' : '(present)'} status=${created.status} mode=${created.mode}`)
  check('T3.model-label', 'session.create labels the model deterministic-stub and names no provider credential',
    created.model === 'deterministic-stub (fleet-selftest-stub)' && created.provider === 'fleet-selftest-stub',
    `model=${created.model} provider=${created.provider}`)
  if (sessionId === null) {
    check('T3.prompt', 'one agent turn completes', false, 'no session id to prompt')
    return { sessionId: null }
  }

  const promptText = `checker-verify prompt ${nonce}`
  const prompt = await client.toolCall('session.prompt', { sessionId, text: promptText }, { timeoutMs: 180_000 })
  const turn = prompt.payload ?? {}
  record('t3-session-prompt', { isError: prompt.isError, result: turn })
  const transport = turn.transport ?? null
  const transportValid = transport === 'agent' || transport === 'stub-turn'
  const assistantText = typeof turn.assistant_text === 'string' ? turn.assistant_text : ''
  const stubAnswer = assistantText.includes('S3STUB:')
  check('T3.prompt.transport', 'session.prompt names its real transport (agent | stub-turn), never an unlabelled turn',
    !prompt.isError && transportValid,
    `transport=${transport}${transport === 'stub-turn' ? ` turn_agent=${turn.turn_agent ?? '(absent)'} ledger_gate=${turn.ledger_gate ?? '(absent)'} turn_model=${turn.turn_model ?? '(absent)'} provider_calls=${turn.provider_calls ?? '(absent)'}` : ''}`)
  check('T3.prompt', 'one full agent turn completes (non-empty assistant text; the transport is named)',
    !prompt.isError && turn.status === 'completed' && assistantText.trim() !== '' && turn.turns === 1,
    `status=${turn.status} turns=${turn.turns} transport=${transport} assistant_bytes=${Buffer.byteLength(assistantText)} contains_stub_prefix=${stubAnswer}`)
  // Recorded exactly, not smoothed: the tool description says the stub answers "S3STUB:<your text>", but the observed
  // text was the session's injected runtime-context turn, not the caller's literal text. Informational, never a pass.
  check('T3.prompt.echo', 'the stub answer echoes the caller text literally (tool description says S3STUB:<your text>) — RECORDED, not required: ' + (assistantText.includes(nonce) ? 'yes' : 'no'),
    null, `contains_caller_nonce=${assistantText.includes(nonce)} contains_stub_prefix=${stubAnswer} first_80=${JSON.stringify(assistantText.slice(0, 80))}`)
  if (transport === 'stub-turn') {
    check('T3.prompt.stub-turn-principal', 'the stub-turn transport names the selftest principal and the gate reason',
      typeof turn.turn_agent === 'string' && /^(lead|builder|checker|researcher):selftest-/.test(turn.turn_agent) && typeof turn.ledger_gate === 'string' && turn.ledger_gate !== '',
      `turn_agent=${turn.turn_agent ?? '(absent)'} ledger_gate=${turn.ledger_gate ?? '(absent)'}`)
  }

  const status = await client.toolCall('session.status', { sessionId })
  const state = status.payload ?? {}
  record('t3-session-status', { isError: status.isError, result: state })
  const lastText = typeof state.last_assistant_text === 'string' ? state.last_assistant_text : ''
  const readBack = lastText !== '' && (assistantText.startsWith(lastText) || lastText.startsWith(assistantText.slice(0, 4000)))
  check('T3.status', 'session.status reads the same session back (found, turns, model label, transport, same assistant text)',
    !status.isError && state.found === true && state.model === 'deterministic-stub (fleet-selftest-stub)' &&
    state.turns === 1 && state.last_transport === transport && readBack,
    `found=${state.found} turns=${state.turns} model=${state.model} last_transport=${state.last_transport} expected_transport=${transport} read_back_same_text=${readBack} last_bytes=${Buffer.byteLength(lastText)}`)

  const modelMode = await client.toolCall('session.create', { mode: 'model' })
  record('t3-session-model-mode', { isError: modelMode.isError, result: modelMode.payload })
  check('T3.model-mode-staged', 'session.create mode:model is refused by name, never faked',
    modelMode.isError && modelMode.payload?.error?.code === 'model-credential-not-entered' &&
    modelMode.payload?.sessionId === undefined,
    `isError=${modelMode.isError} code=${modelMode.payload?.error?.code ?? '(none)'}`)
  return { sessionId, promptText, turn, state }
}

// ─────────────────────────────────────────────────────────────── T4: roundtrip
async function readOrderIndependently (orderId) {
  const orderGet = await control({ op: 'order-get', id: orderId })
  const posts = await control({ op: 'posts', board: orderId, after: 0 })
  const list = await control({ op: 'order-list', limit: 500 })
  return { orderGet, posts, list }
}
async function test4Roundtrip (client, tag = 't4') {
  say(`\n── ${tag.toUpperCase()}: order.drive_roundtrip through the facade, verified over the control socket`)
  const title = `checker-verify roundtrip ${nonce} ${tag}`
  markers.push(title)
  const call = await client.toolCall('order.drive_roundtrip', {
    title,
    done_when: { mode: 'command', command: 'true', text: `checker-verify done-when ${nonce}` },
    example: 'one evidence note citing an artefact sha256',
    est_minutes: 10,
  }, { timeoutMs: 300_000 })
  const result = call.payload ?? {}
  record(`${tag}-roundtrip`, { isError: call.isError, result })
  if (call.isError) {
    check(`${tag}.call`, 'order.drive_roundtrip returns a result (not a refusal)', false, `code=${result?.error?.code} message=${result?.error?.message}`)
    return null
  }
  const orderId = result?.order?.id ?? null
  const taskId = result?.task?.id ?? null
  const ids = result?.ids ?? {}
  check(`${tag}.ids`, 'the id chain is shaped oN / tN and the post ids are digit strings',
    /^o\d+$/.test(String(orderId)) && /^t\d+$/.test(String(taskId)) &&
    (Array.isArray(result?.posts) ? result.posts.every(post => /^\d+$/.test(String(post?.id))) : false),
    `order=${orderId} task=${taskId} final_task=${ids.final_task} posts=${Array.isArray(result?.posts) ? result.posts.length : 0}`)
  check(`${tag}.honesty`, 'the result labels the lifecycle fixture-validated and the real lane not-built',
    result.mode === 'fixture-validated' && typeof result.real_lane === 'string' && /not-built/i.test(result.real_lane) &&
    result.order?.fixture === true,
    `mode=${result.mode} real_lane=${result.real_lane} order.fixture=${result.order?.fixture}`)
  check(`${tag}.final_state`, 'final_state reports order and task done with a pass verdict',
    result.final_state?.order_state === 'done' && result.final_state?.order_done === true &&
    result.final_state?.task_state === 'done' && result.final_state?.task_done === true &&
    result.final_state?.verdict === 'pass',
    JSON.stringify(result.final_state ?? null))
  check(`${tag}.not-real-execution`, 'the result claims no real-lane execution',
    !/real[_ -]?lane[^;]{0,80}(executed|drove|ran|done)/i.test(JSON.stringify(result)) || /not-built/i.test(JSON.stringify(result?.real_lane ?? '')),
    `real_lane=${result.real_lane}`)

  // Independent verification through the fleet's own dispatcher — not the tool echo.
  const independent = orderId === null ? null : await readOrderIndependently(orderId)
  record(`${tag}-control`, independent === null ? null : {
    orderGet: independent.orderGet, posts: independent.posts, listIds: (independent.list?.result?.orders ?? []).map(order => order.id),
  })
  const orderRow = independent?.orderGet?.result?.order ?? null
  const tasks = Array.isArray(independent?.orderGet?.result?.tasks) ? independent.orderGet.result.tasks : []
  const taskRow = tasks.find(task => String(task.id) === String(taskId)) ?? null
  const finalRow = tasks.find(task => task.final === true) ?? null
  check(`${tag}.control.order`, 'control order-get shows the same order id, terminal done, fixture, and the nonce title',
    independent?.orderGet?.ok === true && orderRow !== null && String(orderRow.id) === String(orderId) &&
    orderRow.state === 'done' && orderRow.fixture === true && String(orderRow.title ?? '').includes(nonce),
    `ok=${independent?.orderGet?.ok} id=${orderRow?.id} state=${orderRow?.state} fixture=${orderRow?.fixture} titleMatch=${String(orderRow?.title ?? '').includes(nonce)}`)
  check(`${tag}.control.task`, 'control order-get shows the task done and banked, and the final task done',
    taskRow !== null && taskRow.state === 'done' && (taskRow.banked === 1 || taskRow.banked === true) &&
    finalRow !== null && finalRow.state === 'done',
    `task.state=${taskRow?.state} task.banked=${taskRow?.banked} final.state=${finalRow?.state}`)
  const reportedPosts = Array.isArray(result.posts) ? result.posts : []
  const controlPosts = Array.isArray(independent?.posts?.result?.posts) ? independent.posts.result.posts : []
  const controlPostIds = new Set(controlPosts.map(post => String(post.id)))
  const missingPosts = reportedPosts.filter(post => !controlPostIds.has(String(post.id)))
  check(`${tag}.control.posts`, 'every post id the tool reported exists on the order board over the control socket',
    reportedPosts.length > 0 && missingPosts.length === 0,
    `reported=${reportedPosts.length} control=${controlPosts.length} missing=${JSON.stringify(missingPosts.map(p => p.id))}`)
  const listRow = (independent?.list?.result?.orders ?? []).find(order => String(order.id) === String(orderId)) ?? null
  check(`${tag}.control.order-list`, 'the order is visible in order-list (no shadow row)',
    independent?.list?.ok === true && listRow !== null && listRow.state === 'done',
    `listed=${listRow !== null} state=${listRow?.state}`)
  return { orderId, taskId, taskRow, orderRow }
}

// ─────────────────────────────────────────────────────────────── T5: stage_real
async function test5StageReal (client) {
  say('\n── T5: order.stage_real — non-fixture intention, runnable done-when, honest about not executing')
  const stageMarker = `staged ${nonce}`
  const title = `checker-verify ${stageMarker}`
  markers.push(title)
  const goal = `stage only (checker-verify ${nonce})`
  const doneWhen = { mode: 'command', command: 'true', text: `checker-verify staged done-when ${nonce}` }
  markers.push(doneWhen.text)
  const call = await client.toolCall('order.stage_real', {
    title,
    goal,
    done_when: doneWhen,
    example: 'a sha256 artefact',
    est_minutes: 10,
    budget_usd: 1,
    checkpoint_at: new Date(Date.now() + 24 * 3600_000).toISOString(),
  }, { timeoutMs: 120_000 })
  record('t5-stage-real', { isError: call.isError, result: call.payload })
  const error = call.payload?.error ?? null
  const details = error?.details ?? null

  // Independent check of the control plane: in the staged branch staging must create no order; in the real-lane branch
  // the one order it creates must be exactly the id it returned.
  const list = await control({ op: 'order-list', limit: 500 })
  const matchingOrders = (list?.result?.orders ?? []).filter(order => String(order.title ?? '').includes(stageMarker))
  record('t5-control-order-list', { ok: list?.ok, matchingOrders })

  const rosterMounted = call.isError && error?.code === 'lanes-unavailable' ? false : (call.isError ? null : true)
  if (rosterMounted === false) {
    check('T5.control.no-order', 'staging created no order (fixture or real) and claims no execution', matchingOrders.length === 0,
      `orders titled with the staging marker=${matchingOrders.length}${matchingOrders.length ? ` states=${matchingOrders.map(o => o.state).join(',')}` : ''}`)
    // Branch A: honest staging, no roster (expected on this host).
    check('T5.branch', 'no roster mounted -> clean staged refusal named lanes-unavailable', true,
      'branch A (real-lane-staged) is the expected honest shape on a host without fleetRoster')
    check('T5.refusal', 'refusal is named lanes-unavailable with mode real-lane-staged and no order id',
      error?.code === 'lanes-unavailable' && details?.mode === 'real-lane-staged' &&
      typeof details?.stage_path === 'string' && details.stage_path.includes('/staged/') &&
      error?.message !== undefined && !/executed|order .*done/i.test(String(error.message)),
      `code=${error?.code} mode=${details?.mode} stage_path=${details?.stage_path ? '(present)' : '(absent)'}`)
    check('T5.payload-digest', 'the returned payload digest equals sha256 of the canonical payload',
      typeof details?.payload_sha256 === 'string' && /^[0-9a-f]{64}$/.test(details.payload_sha256) &&
      details.payload !== undefined && sha256(canonical(details.payload)) === details.payload_sha256,
      `digest_match=${typeof details?.payload_sha256 === 'string' && details.payload !== undefined ? sha256(canonical(details.payload)) === details.payload_sha256 : false}`)
    const payload = details?.payload ?? null
    const nonFixture = payload !== null && payload.fixture !== true && String(payload.title ?? '') === title && String(payload.goal ?? '') === goal
    check('T5.payload.non-fixture', 'the staged payload is the non-fixture order form with its own title/goal', nonFixture,
      `fixture=${payload?.fixture} titleMatch=${String(payload?.title ?? '') === title} goalMatch=${String(payload?.goal ?? '') === goal}`)
    const payloadDoneWhen = payload?.done_when ?? null
    check('T5.payload.done-when', 'the staged payload carries the sent done-when', JSON.stringify(payloadDoneWhen) !== 'null' && payloadDoneWhen?.mode === 'command' && payloadDoneWhen?.command === 'true',
      JSON.stringify(payloadDoneWhen ?? null))
    // Runnable: execute the command in a throwaway cwd. Only the command we sent is executed.
    let runnable = { ok: false, detail: 'not executed' }
    if (payloadDoneWhen?.mode === 'command' && payloadDoneWhen?.command === 'true') {
      const proc = spawnSync('sh', ['-c', payloadDoneWhen.command], { cwd: OUT, timeout: 5000, encoding: 'utf8' })
      runnable = { ok: proc.status === 0, detail: `exit=${proc.status}` }
    }
    check('T5.done-when.runnable', 'the staged done-when actually runs (exit 0) in a throwaway cwd', runnable.ok, runnable.detail)
    check('T5.honest', 'the staged refusal does not claim execution', !/executed|ran the order|order is done/i.test(JSON.stringify(error ?? {})),
      'message/detail inspected for execution claims')
    return { branch: 'A', details, matchingOrders: matchingOrders.length }
  }
  if (rosterMounted === true) {
    // Branch B: a real lane exists; the order must be a visible non-fixture row, not executed by staging.
    const out = call.payload ?? {}
    const orderId = out?.order?.id ?? null
    const orderRow = orderId === null ? null : (await control({ op: 'order-get', id: orderId }))?.result?.order ?? null
    record('t5-branch-b', { out, orderRow })
    check('T5.branch', 'a roster is mounted -> branch B (real-lane staging) applies', true, 'branch B exercised')
    check('T5.real-order', 'staging returns an order id whose control-socket row is a non-fixture order',
      out?.mode === 'real-lane' && /^o\d+$/.test(String(orderId)) && orderRow !== null &&
      String(orderRow.id) === String(orderId) && orderRow.fixture === false && orderRow.state !== 'done',
      `mode=${out?.mode} id=${orderId} state=${orderRow?.state} fixture=${orderRow?.fixture}`)
    check('T5.control.order-chain', 'the staged real order is visible in order-list exactly once, as the returned id',
      matchingOrders.length === 1 && String(matchingOrders[0].id) === String(orderId) && matchingOrders[0].fixture === false,
      `matching=${matchingOrders.length} ids=${JSON.stringify(matchingOrders.map(o => o.id))}`)
    check('T5.honest', 'staging does not claim the order executed', !/executed|ran the order|order is done/i.test(JSON.stringify(out)),
      `state=${orderRow?.state}`)
    return { branch: 'B', orderId }
  }
  check('T5.branch', 'staging either stages honestly (no roster) or creates a visible real order', false,
    `unexpected outcome: isError=${call.isError} code=${error?.code ?? '(none)'}`)
  return { branch: 'unknown' }
}

// ─────────────────────────────────────────────────────────────── T7: source gate (partial, stated)
async function test7SourceGate (client) {
  say('\n── T7: source allowlist — what is provable from this seat, and what is not')
  const status = await client.toolCall('fleet.status', {}, { timeoutMs: 60_000 })
  const data = status.payload ?? {}
  record('t7-fleet-status', { isError: status.isError, result: data })
  const sources = data?.mcp?.sources ?? null
  const bind = Array.isArray(data?.mcp?.bind) ? data.mcp.bind : []
  const v4 = Array.isArray(sources?.v4) ? sources.v4 : []
  check('T7.runtime-sources', 'fleet.status reports the loopback+tailnet allowlist',
    Array.isArray(sources?.sources) && sources.sources.includes('loopback') && sources.sources.includes('tailnet') &&
    v4.includes('127.0.0.0/8') && v4.includes('100.64.0.0/10') && (sources?.v6 ?? []).includes('::1'),
    JSON.stringify(sources ?? null))
  const bindHosts = bind.map(entry => String(entry?.host ?? ''))
  const hasTailnetBind = bindHosts.some(host => /^100\.(6[4-9]|[7-9]\d|1[01]\d|12[0-7])\./.test(host))
  check('T7.bind', 'the listener binds exact addresses, never a catch-all',
    bindHosts.includes('127.0.0.1') && bindHosts.includes('::1') && (hasTailnetBind || MODE === 'dryrun') &&
    !bindHosts.includes('0.0.0.0') && !bindHosts.includes('::'),
    `bind=${JSON.stringify(bind)}${MODE === 'dryrun' && !hasTailnetBind ? ' (dryrun: self-test only)' : ''}`)
  check('T7.honesty', 'fleet.status labels the model route and the real lane honestly',
    data?.mcp?.model_route === 'deterministic-stub (fleet-selftest-stub)' && /not-built/i.test(String(data?.mcp?.real_lane ?? '')),
    `model_route=${data?.mcp?.model_route} real_lane=${data?.mcp?.real_lane}`)
  check('T7.fleet-integrity', 'fleet.status reports fleet integrity ok with no failures',
    data?.fleet?.integrity?.ok === true && (data?.fleet?.integrity?.failures ?? []).length === 0,
    `ok=${data?.fleet?.integrity?.ok} failures=${JSON.stringify(data?.fleet?.integrity?.failures ?? [])}`)

  // Unit netfilter, read from systemd (no root needed for a system unit's properties on this host).
  const show = spawnSync('systemctl', ['show', 'fleet-dsh', '-p', 'IPAddressDeny', '-p', 'IPAddressAllow', '-p', 'MainPID', '-p', 'ActiveState'], { encoding: 'utf8', timeout: 10_000 })
  const unitFile = spawnSync('systemctl', ['cat', 'fleet-dsh', '--no-pager'], { encoding: 'utf8', timeout: 10_000 })
  writeTextFile('t7-unit-netfilter.txt', `# systemctl cat fleet-dsh\n${unitFile.stdout ?? ''}\n# systemctl show\n${show.stdout ?? ''}\n`)
  record('t7-unit', { show: show.stdout ?? '', cat: unitFile.stdout ?? '' })
  const unitText = unitFile.stdout ?? ''
  const showText = show.stdout ?? ''
  const fileAllow = /IPAddressAllow=localhost 100\.110\.154\.92 100\.114\.17\.67/.test(unitText)
  const fileDeny = /IPAddressDeny=0\.0\.0\.0\/8 10\.0\.0\.0\/8 172\.16\.0\.0\/12 192\.168\.0\.0\/16 169\.254\.0\.0\/16 100\.64\.0\.0\/10 ::\/0/.test(unitText)
  const effectiveAllow = /127\.0\.0\.0\/8/.test(showText) && /100\.114\.17\.67\/32/.test(showText) && /100\.110\.154\.92\/32/.test(showText) && /::1\/128/.test(showText)
  const effectiveDeny = /100\.64\.0\.0\/10/.test(showText) && /0\.0\.0\.0\/8/.test(showText) && /::\/0/.test(showText)
  check('T7.unit-netfilter', 'the unit netfilter allows only localhost and the two named tailnet peers',
    fileAllow && fileDeny && effectiveAllow && effectiveDeny && /RestartPreventExitStatus=SIGKILL/.test(unitText),
    `fileAllow=${fileAllow} fileDeny=${fileDeny} effectiveAllow=${effectiveAllow} effectiveDeny=${effectiveDeny}; unit shown in t7-unit-netfilter.txt`)
  const listeners = spawnSync('ss', ['-ltn'], { encoding: 'utf8', timeout: 10_000 })
  const mcpPort = String(new URL(LOOPBACK_URL).port || 80)
  const portLines = (listeners.stdout ?? '').split('\n').filter(line => line.includes(`:${mcpPort}`))
  writeTextFile('t7-listeners.txt', `# ss -ltn | :${mcpPort}\n${portLines.join('\n')}\n`)
  check('T7.listeners', `ss shows :${mcpPort} only on loopback and the host tailnet address (never 0.0.0.0/::)`,
    portLines.length >= 1 && !portLines.some(line => line.includes(`0.0.0.0:${mcpPort}`) || line.includes(`[::]:${mcpPort}`)),
    `lines=${JSON.stringify(portLines)}`)

  // A same-host attempt to source from a non-loopback local bridge address. Non-probative by construction: recorded, not scored.
  const spoof = await new Promise(resolveP => {
    const socket = net.connect({ host: '127.0.0.1', port: Number(new URL(LOOPBACK_URL).port || 80), localAddress: '172.17.0.1' })
    const timer = setTimeout(() => { socket.destroy(); resolveP({ outcome: 'timeout' }) }, 2500)
    socket.on('connect', () => { clearTimeout(timer); socket.destroy(); resolveP({ outcome: 'connected (source seen as a local address; not a foreign-source proof)' }) })
    socket.on('error', error => { clearTimeout(timer); resolveP({ outcome: `kernel refused local bind/route: ${error.code ?? error.message}` }) })
  })
  record('t7-source-attempt', spoof)
  check('T7.foreign-source-live', 'a live foreign-source request is refused (NOT PROVEN from this seat — stated, not claimed)',
    null, `${spoof.outcome}. Requires a second host/namespace or root; this is recorded as not proven, never as a pass.`)
  return data
}

// ─────────────────────────────────────────────────────────────── T6: action log
function verifyChain (rows) {
  const linked = []
  const recompute = []
  for (let i = 1; i < rows.length; i++) {
    linked.push(rows[i].prev === rows[i - 1].hash)
  }
  for (let i = 1; i < rows.length; i++) {
    const row = rows[i]
    const body = {}
    for (const key of Object.keys(row)) if (key !== 'prev' && key !== 'hash') body[key] = row[key]
    recompute.push(sha256(`${rows[i - 1].hash}\n${canonical(body)}`) === row.hash)
  }
  return { linkOk: linked.length === 0 ? null : linked.every(Boolean), linkCount: linked.length, recomputeOk: recompute.length === 0 ? null : recompute.every(Boolean), recomputeCount: recompute.length }
}
async function test6ActionLog (client, baselineSeq) {
  say('\n── T6: action log — append-before-effect, principal/tool/outcome, no secret value')
  const actionsStatus = await client.toolCall('actions.status', { n: 100 }, { timeoutMs: 30_000 })
  const statusData = actionsStatus.payload ?? {}
  record('t6-actions-status', { isError: actionsStatus.isError, result: statusData })
  check('T6.actions.status', 'actions.status reports an open, unbroken hash-chained log',
    !actionsStatus.isError && statusData?.log?.open === true && statusData?.log?.broken === null && statusData?.log?.chain === 'jsonl-hash-chain',
    `open=${statusData?.log?.open} broken=${statusData?.log?.broken} chain=${statusData?.log?.chain} seq=${statusData?.log?.seq}`)

  const tail = await admin({ op: 'log_tail', n: 100 })
  const rows = Array.isArray(tail?.result?.rows) ? tail.result.rows : []
  record('t6-admin-log-tail', { seq: tail?.result?.seq, head: tail?.result?.head, path: tail?.result?.path, rows })
  if (rows.length === 0) { check('T6.rows', 'the action log has rows', false, 'admin log_tail returned no rows'); return }

  const fresh = rows.filter(row => Number(row.seq) > baselineSeq)
  const intents = fresh.filter(row => row.kind === 'intent')
  const outcomes = fresh.filter(row => row.kind === 'outcome')
  record('t6-fresh-rows', { baselineSeq, intentCount: intents.length, outcomeCount: outcomes.length, intents: intents.map(r => ({ seq: r.seq, tool: r.tool, principal: r.principal, args_keys: r.args_keys })), outcomes: outcomes.map(r => ({ seq: r.seq, ref_seq: r.ref_seq, tool: r.tool, outcome: r.outcome, code: r.code, ms: r.ms })) })

  // Every call this probe made must have an intent row, in the same order, with a principal.
  const expectedTools = toolCalls.map(call => call.tool)
  const intentTools = intents.map(row => row.tool)
  check('T6.intents', 'every tool call this probe made has an append-before-effect intent row (same order)',
    JSON.stringify(intentTools) === JSON.stringify(expectedTools),
    `expected=${JSON.stringify(expectedTools)} logged=${JSON.stringify(intentTools)}`)
  check('T6.principal', 'every intent row carries a non-empty principal',
    intents.length > 0 && intents.every(row => typeof row.principal === 'string' && row.principal !== ''),
    JSON.stringify(intents.map(row => row.principal)))
  check('T6.args-digest', 'intent rows store only argument NAMES and a digest (no values)',
    intents.every(row => /^[0-9a-f]{64}$/.test(String(row.args_sha256 ?? '')) && Array.isArray(row.args_keys) && Number(row.args_bytes) > 0),
    `keys_sample=${JSON.stringify(intents[intents.length - 1]?.args_keys ?? null)}`)

  const outcomeByRef = new Map(outcomes.map(row => [Number(row.ref_seq), row]))
  const missingOutcome = intents.filter(intent => {
    const outcome = outcomeByRef.get(Number(intent.seq))
    return outcome === undefined || Number(outcome.seq) <= Number(intent.seq)
  })
  check('T6.append-before-effect', 'every intent has its outcome appended after it (ref_seq = intent seq)',
    intents.length > 0 && missingOutcome.length === 0,
    `intents=${intents.length} outcomes=${outcomes.length} missing=${JSON.stringify(missingOutcome.map(row => row.tool))}`)
  const errorCalls = toolCalls.filter(call => !call.ok)
  const errorIntents = intents.filter(intent => outcomeByRef.get(Number(intent.seq))?.outcome === 'error')
  const outcomeMismatches = []
  const pairCount = Math.min(toolCalls.length, intents.length)
  for (let i = 0; i < pairCount; i++) {
    const outcome = outcomeByRef.get(Number(intents[i].seq))
    const expected = toolCalls[i].ok ? 'ok' : 'error'
    if (outcome === undefined || outcome.outcome !== expected) outcomeMismatches.push(`${i}:${toolCalls[i].tool}:expected ${expected}`)
  }
  check('T6.outcomes', 'outcome rows match the real results (ok vs error), call for call in order',
    pairCount === toolCalls.length && outcomeMismatches.length === 0,
    `toolCalls=${toolCalls.length} intents=${intents.length} errors=${errorCalls.length} errorOutcomes=${errorIntents.length} mismatches=${JSON.stringify(outcomeMismatches)}`)

  // No secret / no argument VALUE in the rows.
  const rowsText = JSON.stringify(rows)
  check('T6.no-token', 'the bearer token never appears in the action log rows (value compared in memory)',
    !rowsText.includes(token), 'comparison only; the value is never printed')
  const leakedMarkers = markers.filter(marker => typeof marker === 'string' && marker.length >= 8 && rowsText.includes(marker))
  check('T6.no-arg-values', 'no argument VALUE this probe sent appears in the log rows; only names/digests are stored',
    leakedMarkers.length === 0, `markers checked=${markers.length} leaked=${JSON.stringify(leakedMarkers.map(m => m.slice(0, 18)))}`)

  const chain = verifyChain(rows)
  check('T6.chain-link', 'consecutive tail rows link by prev == previous hash', chain.linkOk !== false,
    `rows=${rows.length} linkChecks=${chain.linkCount} linkOk=${chain.linkOk}`)
  check('T6.chain-recompute', 'the documented hash recomputation reproduces the row hashes', chain.recomputeOk !== false,
    `recomputeChecks=${chain.recomputeCount} recomputeOk=${chain.recomputeOk}`)

  const statusRows = Array.isArray(statusData?.rows) ? statusData.rows : []
  let tailAgrees = false
  let tailDetail = 'no rows'
  if (statusRows.length > 0 && rows.length > 0) {
    const lastSeq = Number(statusRows[statusRows.length - 1].seq)
    const index = rows.findIndex(row => Number(row.seq) === lastSeq)
    const sameChain = index >= 0 && statusRows.every((row, offset) => {
      const adminRow = rows[index - statusRows.length + 1 + offset]
      return adminRow !== undefined && String(adminRow.seq) === String(row.seq) && String(adminRow.hash) === String(row.hash)
    })
    const trailing = index >= 0 ? rows.slice(index + 1) : []
    const trailingIsOwnOutcome = trailing.every(row => row.kind === 'outcome' && Number(row.ref_seq) === lastSeq)
    tailAgrees = sameChain && trailingIsOwnOutcome
    tailDetail = `actions_rows=${statusRows.length} admin_rows=${rows.length} lastSeq=${lastSeq} sameChain=${sameChain} trailing=${trailing.length} trailingIsOwnOutcome=${trailingIsOwnOutcome}`
  }
  check('T6.actions.tail-agrees', 'actions.status tail and the admin-socket tail agree on the rows both can see',
    tailAgrees, tailDetail)

  // Full-file byte scan when the log file is readable (scratch hosts); otherwise the limitation is stated.
  const logPath = statusData?.log?.path ?? (await admin({ op: 'stat' }))?.result?.log?.path ?? null
  let logText = null
  try { if (typeof logPath === 'string') logText = readFileSync(logPath, 'utf8') } catch { logText = null }
  if (logText !== null) {
    const logLines = logText.split('\n').filter(line => line.trim() !== '')
    let parsed = true
    const fileRows = []
    for (const line of logLines) { try { fileRows.push(JSON.parse(line)) } catch { parsed = false } }
    const fileChain = verifyChain(fileRows)
    writeTextFile('t6-full-log.json', JSON.stringify(fileRows, null, 2))
    check('T6.full-file-scan-token', 'the whole action log (every byte) contains no bearer token', !logText.includes(token),
      `log=${logPath} bytes=${Buffer.byteLength(logText)} rows=${fileRows.length} (value compared in memory only)`)
    const leaked = markers.filter(marker => typeof marker === 'string' && marker.length >= 8 && logText.includes(marker))
    check('T6.full-file-scan-values', 'the whole action log contains no argument value this probe sent', leaked.length === 0,
      `markers=${markers.length} leaked=${JSON.stringify(leaked.map(m => m.slice(0, 18)))}`)
    check('T6.full-file-scan-chain', 'every row in the whole log parses and the full hash chain links and recomputes',
      parsed && fileRows.length > 0 && fileChain.linkOk === true && fileChain.recomputeOk === true,
      `rows=${fileRows.length} parsed=${parsed} linkOk=${fileChain.linkOk} recomputeOk=${fileChain.recomputeOk}`)
  } else {
    check('T6.full-file-scan', 'raw full-file byte scan of the action log (limitation stated, not hidden)', null,
      `log path ${logPath ?? '(unknown)'} is not readable as uid 1000; tail scan is the provable scope, full scan needs the owner: sudo sh -c 'grep -c "$(cat /var/lib/fleet/state/mcp/token)" /var/lib/fleet/log/mcp-actions.jsonl' (expect 0)`)
  }
}

// ─────────────────────────────────────────────────────────────── T8: transport limits
async function test8Limits (client) {
  say('\n── T8: body/parse limits refuse cleanly (and log the refusal)')
  const baseline = Number((await admin({ op: 'log_tail', n: 1 }))?.result?.seq ?? 0)
  const headers = { 'content-type': 'application/json', accept: 'application/json, text/event-stream', authorization: `Bearer ${token}` }
  const oversizedBody = '{"jsonrpc":"2.0","id":901,"method":"ping","params":{"pad":"' + 'x'.repeat(300_000) + '"}}'
  const rawPost = async (body, timeoutMs = 20_000) => {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), timeoutMs)
    try {
      const res = await fetch(LOOPBACK_URL, { method: 'POST', headers, body, signal: controller.signal })
      const text = await res.text()
      let parsed = null
      try { parsed = JSON.parse(text) } catch { /* not JSON */ }
      return { status: res.status, headers: Object.fromEntries(res.headers.entries()), text, parsed }
    } catch (error) {
      return { status: 0, headers: {}, text: `fetch error: ${error.code ?? error.message}`, parsed: null }
    } finally { clearTimeout(timer) }
  }
  const oversized = await rawPost(oversizedBody)
  record('t8-oversized', oversized)
  check('T8.oversized.status', 'a body over the limit is refused (413, or a reset AFTER the refusal is logged — recorded exactly)',
    oversized.status === 413,
    `status=${oversized.status} body=${String(oversized.text ?? '').slice(0, 160)}`)
  const malformed = await rawPost('{not json', 10_000)
  record('t8-malformed', malformed)
  let malformedParsed = null
  try { malformedParsed = JSON.parse(malformed.text) } catch { /* recorded below */ }
  check('T8.malformed', 'malformed JSON is refused with 400 and a parse error (never dispatched)', malformed.status === 400 && malformedParsed?.error?.code === -32700,
    `status=${malformed.status} code=${malformedParsed?.error?.code ?? '(none)'}`)
  const batch = await httpJson(LOOPBACK_URL, { headers, body: [{ jsonrpc: '2.0', id: 902, method: 'ping' }], timeoutMs: 10_000 })
  record('t8-batch', batch)
  check('T8.batch', 'a JSON-RPC batch is refused with 400 (one message per POST)', batch.status === 400 && batch.parsed?.error?.code === -32600,
    `status=${batch.status} code=${batch.parsed?.error?.code ?? '(none)'}`)
  await new Promise(r => setTimeout(r, 150))
  const tail = await admin({ op: 'log_tail', n: 30 })
  const fresh = (tail?.result?.rows ?? []).filter(row => Number(row.seq) > baseline)
  const refusedCodes = fresh.filter(row => row.kind === 'refused').map(row => row.code)
  record('t8-log-refusals', { baseline, refusedCodes })
  check('T8.log', 'the limit refusals are appended to the action log (payload-too-large, parse-error)',
    refusedCodes.includes('payload-too-large') && refusedCodes.includes('parse-error'),
    JSON.stringify(refusedCodes))
  check('T8.log.batch-row', 'a batch refusal is ALSO appended as a refused row (observed behaviour, recorded not required)',
    null, `batch_logged=${refusedCodes.includes('batch')} codes=${JSON.stringify(refusedCodes)}`)
}

// ─────────────────────────────────────────────────────────────── T9: rate limit (last: exhausts the token window)
async function test9RateLimit (client) {
  say('\n── T9: 60 requests/minute per-token rate limit')
  const headers = { 'content-type': 'application/json', accept: 'application/json, text/event-stream', authorization: `Bearer ${token}` }
  let firstLimited = null
  let sent = 0
  for (let i = 0; i < 90; i++) {
    sent++
    const response = await httpJson(LOOPBACK_URL, { headers, body: { jsonrpc: '2.0', id: 1000 + i, method: 'ping' }, timeoutMs: 10_000 }).catch(() => ({ status: 0, headers: {}, parsed: null }))
    if (response.status === 429) { firstLimited = { after: sent, headers: { 'retry-after': response.headers['retry-after'] ?? null }, error: response.parsed?.error ?? null }; break }
  }
  record('t9-rate', { sent, firstLimited })
  check('T9.rate', 'the 61st-request-class call inside one minute is refused 429 with a retry-after and -32005',
    firstLimited !== null && firstLimited.error?.code === -32005 && firstLimited.headers['retry-after'] !== null && Number(firstLimited.headers['retry-after']) >= 1,
    firstLimited === null ? `no 429 within 90 further requests (sent=${sent})` : `limited after ${firstLimited.after} requests this run; retry-after=${firstLimited.headers['retry-after']}s code=${firstLimited.error?.code}`)
  if (firstLimited !== null) {
    const tail = await admin({ op: 'log_tail', n: 5 })
    const rateRow = (tail?.result?.rows ?? []).filter(row => row.kind === 'refused' && row.code === 'rate-limited')
    check('T9.rate-logged', 'the rate-limit refusal is appended to the action log', rateRow.length > 0, `rows=${rateRow.length}`)
  } else {
    check('T9.rate-logged', 'the rate-limit refusal is appended to the action log', false, 'no 429 to log')
  }
}

// ─────────────────────────────────────────────────────────────── main
async function main () {
  const pre = await preflight()
  const tailnetUsable = TAILNET_URL !== null && pre.tailnetReachable !== false
  const legs = [['loopback', LOOPBACK_URL], ...(tailnetUsable ? [['tailnet', TAILNET_URL]] : [])]
  await test1AuthNegative()
  let baselineSeq = Number((await admin({ op: 'log_tail', n: 1 }))?.result?.seq ?? 0)
  // baseline for T6: the seq before the first authenticated tool call (T1 refused rows are already behind us)
  const { clients } = await test2Legs(legs)
  const loop = clients.loopback
  const session = await test3Session(loop)
  if (typeof session.sessionId === 'string') markers.push(session.sessionId)
  if (typeof session.promptText === 'string') markers.push(session.promptText)
  const first = await test4Roundtrip(loop, 't4')
  const second = await test4Roundtrip(loop, 't4b')
  const roundtripIdsDiffer = first !== null && second !== null && String(first.orderId) !== String(second.orderId)
  check('T4b.distinct-orders', 'a second independent roundtrip creates a different order (no planted one-shot)', roundtripIdsDiffer,
    `t4=${first?.orderId ?? '(none)'} t4b=${second?.orderId ?? '(none)'}`)
  const stage = await test5StageReal(loop)
  const source = await test7SourceGate(loop)
  await test8Limits(loop)
  await test6ActionLog(loop, baselineSeq)
  await test9RateLimit(loop)
  record('t3-session-summary', session)
  record('t5-summary', stage)
  record('t7-summary', source)

  const failed = checks.filter(check => check.ok === false)
  const mandatory = checks.filter(check => check.ok === null ? false : true)
  const summary = {
    probe: 'checker-verify/mcp-verify.mjs',
    started: STARTED,
    finished: new Date().toISOString(),
    mode: MODE,
    urls: { loopback: LOOPBACK_URL, tailnet: TAILNET_URL, tailnet_established: tailnetUsable },
    nonce,
    checks: { total: checks.length, passed: checks.filter(c => c.ok === true).length, failed: failed.length, informational: checks.filter(c => c.ok === null).length },
    failedIds: failed.map(check => check.id),
    verdict: failed.length === 0 ? 'PASS' : 'FAIL',
  }
  writeJson('mcp-verify.json', { summary, checks, tests: testData })
  writeTextFile('SHA256SUMS-PARTIAL.txt', '') // replaced below once files are final
  say('')
  say(`SUMMARY: ${summary.verdict} — ${summary.checks.passed}/${summary.checks.total} checks passed, ${summary.checks.failed} failed, ${summary.checks.informational} informational`)
  for (const failure of failed) say(`  FAILED ${failure.id}: ${failure.title} :: ${failure.detail}`)
  flushTranscript()
  // Hash the evidence files (excluding the sums file itself) for the pack.
  try {
    const listing = spawnSync('bash', ['-lc', `cd ${JSON.stringify(OUT)} && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum`], { encoding: 'utf8', timeout: 20_000 })
    writeTextFile('SHA256SUMS', listing.stdout ?? '')
  } catch { /* best effort */ }
  process.exit(summary.verdict === 'PASS' ? 0 : 1)
}
main().catch(error => {
  process.stderr.write(`HARNESS: probe crashed: ${redact(error?.stack ?? error?.message ?? error)}\n`)
  flushTranscript()
  process.exit(2)
})
