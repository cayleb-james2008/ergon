# calls.js hotfix review: task_decompose `tasks` string coercion

Reviewer: independent fresh-context pass, 2026-11-02. Read-only on the live fleet package; all runs in /tmp/claude-1000/callsreview/.

## Verdict: PASS (no high or medium defect; two low defects worth fixing in 0.3.4)

Recommendation: carry into 0.3.4 with fixes (move the decode before key derivation, add a length cap, ship the test below). Keeping it on the live plugin meanwhile is safe.

## 1. Exact diff

Live file: `.../home/profiles/fleet/node_modules/@cayleb/dsh-fleet/src/orders/calls.js` (sha256 e01dace0...). Backup `calls.js.bak-pre-tasks-coerce-20261102T1427Z` and baseline `finish-lanefix/package/src/orders/calls.js` have the same sha256 (783ea6c0...), so the backup is byte-identical to baseline.

`diff backup live` and `diff baseline live` are the same, and the only difference in the file:

```
147c147,151
<         const r = orders.decompose(actor, a.order, { claimToken: a.token, tasks: a.tasks, idempotencyKey: k })
---
>         // HOTFIX 2026-11-02 (o1 re-plan): a model sometimes sends `tasks` as a JSON string; coerce it so the same
>         // decomposition the lead intended is validated as a list (nothing else changes).
>         let tasksArg = a.tasks
>         if (typeof tasksArg === 'string') { try { tasksArg = JSON.parse(tasksArg) } catch { /* leave it for the validator's message */ } }
>         const r = orders.decompose(actor, a.order, { claimToken: a.token, tasks: tasksArg, idempotencyKey: k })
```
Nothing else in calls.js differs (confirmed by diff exit and identical hashes of backup and baseline). The stated change matches the diff exactly.

## 2. Analysis

How the argument reaches the handler:
- `tasks` is declared `{ type: 'json', required: true, description: 'Array of task objects (1 to 500).' }` (calls.js, task_decompose). `checkArgs` (calls.js) only type-checks names in `TYPES` (string/integer/number/boolean/array/object); `json` is absent, so it is unchecked and a string DOES reach the handler, both on the `./store` path and the DSH path (tools.js hands the same `parameters` to defineTool). I could not read dsh-tools here (package not present under node_modules), so the host-side schema step for `type: json` is inferred from the declaration and from existing plugin tests, not run directly. The declared schema does not lie: `json` means any JSON value, only the prose says array. Before the hotfix a string died in the validator (`tasks-invalid`); now it is decoded first.
- Validation semantics: after decode the value goes through the identical `validateTasks` (array of 1..500, plain objects, `unknownKeys`, text caps, sizing, budget). Tests confirm: valid string equals array (same stored `spec` rows), 501-item string refused, an item carrying `final` refused (`final-is-harness-owned`), `[]`, `null`, `5`, `"x"`, `{"goal":..}`, `""`, malformed JSON are all refused `tasks-invalid`, nothing created. No size/shape/type limit of the array path is bypassed.
- Prototype pollution: `JSON.parse` makes `__proto__` an own data property; `validateTasks` refuses it (`tasks[0].__proto__:unknown-field`, `constructor:unknown-field`) and `Object.prototype` stayed clean. Same exposure as the array path (the host already JSON.parses the model's tool-call arguments).
- Size/DoS: no length cap before `JSON.parse` (defect L2). Measured: 61 MB valid array of 8M objects parses in ~0.87 s synchronously, ~370 MB heap, 599 MB RSS; 64 MB of garbage fails in 38 ms; 1M-deep nesting parses in 67 ms (V8 parser is not recursive). The array path pays the same cost when the host parses the call arguments, and a model tool call is bounded by its output limit, so this is parity, not a new bypass. Deep nesting that does parse then overflows the stack in `canonical()` (kit.js) and returns uncoded `tool-error: Maximum call stack size exceeded` on BOTH paths (pre-existing, informational).
- Idempotency (defect L1, confirmed): `runSpec` derives the key from the ORIGINAL `args` (string) in calls.js, while `decompose` hashes the DECODED array inside `K.idempotent` (tasks.js:71, kit.js:117-129). Consequences, all tested: same string retried replays (1 task set); explicit `idempotency_key` with string then array replays (request hash is over the decoded array, so this even works across formats); but WITHOUT an explicit key, string first then the same content as an array gets a different derived key and decomposes twice (2 task rows vs 1, test failure below). The array path cannot do this because `canonical()` normalises key order, so formatting never changes its derived key. Needs a lost response plus a format flip, hence Low.
- Audit/board post: the `decomposition` post body is `json({tasks: refs})` of created task ids and `S.progressOrder` hashes `canonical(tasks)` of the decoded array (tasks.js:155-156), so the board records what was actually decomposed, never the raw string. The task `spec` rows hold the validated fields only. Observed: stored specs for the string and array orders are deep-equal.
- Error quality (info): a malformed string yields `tasks-invalid: the decomposition was refused: tasks: tasks-invalid`; the explanatory text ("tasks must be an array of 1..500 task objects") is only in `details.problems[0].message`, and nothing tells the model its string was not JSON. A model that keeps sending bad JSON gets little guidance.
- Other (info): only `task_decompose` is coerced; other `json`-typed args (e.g. `preflight_raise.questions`) still reject a string, so models that stringify there are not helped.

## 3. Test runs

Scratch: `cp -r finish-lanefix/package /tmp/claude-1000/callsreview/pkg`, node_modules replaced by a symlink to `/home/cayleb/.local/opt/deepseek-harness-official/node_modules`, live calls.js copied over `pkg/src/orders/calls.js` (and a second copy `pkgbase` with baseline calls.js). Node v26.7.0.

| run | command | result |
|---|---|---|
| live-patched | `node --test "src/orders/test/*.test.mjs"` in pkg | 185 tests, 182 pass, 3 fail, exit 1 |
| baseline | same in pkgbase | 185 tests, 182 pass, 3 fail, exit 1 |

The set of ok/not-ok lines is identical between the two runs. The 3 failures are the known environmental ones in both: `finisher.test.mjs` (file), plugin.test.mjs "selftest with `agent`: distinct ids..." and plugin.test.mjs "R5-6 ... bad-args" (actual `tool-error`). No regression from the hotfix.

New tests (proposed snippet below), file `string-tasks.test.mjs`, six tests:

| package | result |
|---|---|
| pkg (live-patched calls.js) | 4 pass, 2 fail: the oversized-string test (no cap) and the string-then-array retry test (double decompose, `2 !== 1`) |
| pkgfix (scratch fix, diff in section 5) | 6 pass, 0 fail |
| pkgfix full suite | 191 tests, 188 pass, 3 fail (same 3 environmental failures; the 6 new tests all pass) |

Probes: `dos.mjs` (parse timings above), `proto.mjs` (prototype pollution probe) in the scratch folder.

## 4. Defects

| id | severity | defect | evidence |
|---|---|---|---|
| L1 | Low | Derived idempotency key is computed over the original string, the inner dedupe over the decoded array, so a retry in a different format without an explicit key decomposes twice. | test "retries ..." fails on pkg: `2 !== 1`; passes on pkgfix. |
| L2 | Low | No length cap before `JSON.parse` of a model-written string (parity with the array path, but the new path adds an explicit unbounded parse in a synchronous in-process handler). | 61 MB: 0.87 s block, 370 MB heap; test "oversized" fails on pkg, passes on pkgfix with a 2 MiB cap. |
| I1 | Info | No regression test shipped with the hotfix. | no string-tasks test in the live package tests. |
| I2 | Info | Invalid-JSON string gives the bare `tasks-invalid` message, no hint it was not JSON. | error text above. |
| I3 | Info | Deeply nested input gives uncoded `tool-error: Maximum call stack size exceeded` on both paths (pre-existing, not introduced). | deep-nested test output. |
| I4 | Info | Coercion is only in `task_decompose`; other json args are inconsistent. | calls.js specs. |

No high or medium defect found; the live change is correct for its purpose and does not weaken validation.

## 5. Proposed fix (verified in scratch only, NOT applied to live or any worktree)

Decode before the key is derived, via a spec-level `normalize` hook, and cap the string. Replaces the inline hotfix; diff against baseline calls.js:

```diff
20a21,28
> /** Models sometimes send `tasks` as a JSON string. Decoded BEFORE the idempotency key is derived, so a string and its array are one request. */
> const MAX_TASKS_TEXT = 2 * 1024 * 1024
> export function tasksFromText(tasks) {
>   if (typeof tasks !== 'string') return tasks
>   if (tasks.length > MAX_TASKS_TEXT) throw new OrdersError('tasks-invalid', `tasks is too large as text (over ${MAX_TASKS_TEXT} characters); send fewer or smaller tasks`)
>   try { return JSON.parse(tasks) } catch { return tasks } // not JSON: the validator refuses the string as it always did
> }
> 
110a119
>   if (spec.normalize !== undefined) args = spec.normalize(args)
145a155
>       normalize: a => ({ ...a, tasks: tasksFromText(a.tasks) }),
```
The cap (2 MiB) is a policy choice: the validator's own per-field limits allow a theoretically larger valid array, but no model tool call approaches that; pick the number you prefer.

## 6. Proposed test (verified: 4/6 pass on live-patched, 6/6 on the fixed scratch copy). Place at `src/orders/test/string-tasks.test.mjs`

```js
// Proposed regression test for the task_decompose `tasks`-as-JSON-string coercion (calls.js hotfix 2026-11-02).
// Place as src/orders/test/string-tasks.test.mjs. Runs over the `./store` (same specs/runSpec/checkArgs as the DSH tool path).
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { join } from 'node:path'
import { openStore } from '../standalone.js'
import { tmp } from './helpers.mjs'

const quiet = { info() {}, warn() {}, error() {}, debug() {} }
const drain = async () => { for (let i = 0; i < 10; i++) await new Promise(resolve => setImmediate(resolve)) }
let k = 0
const key = () => `str-${process.pid}-${++k}`
const item = (extra = {}) => ({ key: 'a', goal: 'g', done_when: { mode: 'command', command: 'true' }, est_minutes: 15, budget_usd: 0.01, ...extra })

async function world(t) {
  const dir = tmp('str')
  const store = await openStore({ path: join(dir, 'fleet.db'), workRoots: [dir], log: quiet })
  t.after(() => store.close())
  const w = { store }
  w.c = async req => { const r = await store.control(req); await drain(); return r }
  w.t = async (agent, tool, args) => { const r = await store.tool(agent, tool, args); await drain(); return r }
  const doc = { title: 'live', goal: 'g', context: 'c', constraints: ['c'], anti_goals: ['a'], done_when: { mode: 'command', command: 'true', text: 'exits 0' }, example: 'e', evidence_required: ['h'],
    assumptions_policy: 'log-and-proceed', budget: { usd: 1 }, stop_conditions: ['x'], checkpoint_at: new Date(Date.now() + 7 * 86_400_000).toISOString(), est_minutes: 30, size: 'standard', successors: [], ambition: ['go'], base_hashes: {}, tier: 1 }
  w.fresh = async n => {                       // a claimed fixture order past its pre-flight, one lane per call
    const lead = `lead:selftest-s${n}`
    const o = (await w.c({ op: 'order-create', idempotency_key: key(), order: doc, fixture: true })).result.id
    const token = (await w.t(lead, 'order_claim', { order: o })).result.token
    await w.t(lead, 'preflight_raise', { order: o, token, questions: [] })
    return { o, token, lead, decompose: (tasks, extra = {}) => w.t(lead, 'task_decompose', { order: o, token, tasks, ...extra }),
      taskRows: () => store.db.prepare('SELECT * FROM tasks WHERE order_id = ? AND final = 0').all(Number(o.slice(1))) }
  }
  return w
}

test('tasks sent as a JSON string decompose exactly like the same array', async (t) => {
  const w = await world(t)
  const items = [item({ key: 'a' }), item({ key: 'b', after_index: [0] })]
  const A = await w.fresh(1), S = await w.fresh(2)
  const ra = await A.decompose(items)
  const rs = await S.decompose(JSON.stringify(items))
  assert.equal(ra.ok, true, JSON.stringify(ra)); assert.equal(rs.ok, true, JSON.stringify(rs))
  assert.deepEqual(rs.result.tasks.map(x => x.key), ['a', 'b'])
  assert.equal(S.taskRows().length, 2)
  assert.deepEqual(S.taskRows().map(r => r.spec), A.taskRows().map(r => r.spec), 'same stored specs')
  assert.equal(w.store.db.prepare("SELECT COUNT(*) n FROM task_deps").get().n >= 2, true)
})

test('a string that is not valid JSON is refused with the validator\'s own refusal, and nothing is created', async (t) => {
  const w = await world(t)
  const S = await w.fresh(3)
  for (const bad of ['not json', '[{"goal": ', '', 'null', '"x"', '5', '{"goal":"g"}', '[]']) {
    const r = await S.decompose(bad)
    assert.equal(r.ok, false, bad); assert.match(r.error, /^tasks-invalid: the decomposition was refused: tasks: tasks-invalid/, bad)
  }
  assert.equal(S.taskRows().length, 0)
})

test('the validator still enforces the array limits on a string: 501 items refused, a string item carrying `final` refused', async (t) => {
  const w = await world(t)
  const S = await w.fresh(4)
  const many = Array.from({ length: 501 }, (_, i) => item({ key: `k${i}`, budget_usd: 0.0001 }))
  assert.match((await S.decompose(JSON.stringify(many))).error, /^tasks-invalid/)
  assert.match((await S.decompose(JSON.stringify([item(), { goal: 'g', est_minutes: 15, final: true }]))).error, /^final-is-harness-owned/)
  assert.equal(S.taskRows().length, 0)
})

test('an oversized tasks string is refused before it is parsed (no JSON.parse of a string over 2 MiB)', async (t) => {
  const w = await world(t)
  const S = await w.fresh(5)
  const realParse = JSON.parse; let biggest = 0
  JSON.parse = (text, ...rest) => { if (typeof text === 'string') biggest = Math.max(biggest, text.length); return realParse(text, ...rest) }
  t.after(() => { JSON.parse = realParse })
  const items = Array.from({ length: 120 }, (_, i) => item({ key: `k${i}`, budget_usd: 0.0001, context: 'x'.repeat(20_000) }))   // every item valid, ~2.4 MB of JSON
  const text = JSON.stringify(items)
  assert.ok(text.length > 2 * 1024 * 1024)
  const r = await S.decompose(text)
  JSON.parse = realParse
  assert.equal(r.ok, false); assert.match(r.error, /^tasks-invalid: .*too large/)
  assert.equal(S.taskRows().length, 0, 'nothing created')
})

test('retries: the same string, or the same content as an array, never decomposes twice', async (t) => {
  const w = await world(t)
  const items = [item()]
  const S = await w.fresh(6)                                                         // derived key (no idempotency_key)
  assert.equal((await S.decompose(JSON.stringify(items))).ok, true)
  assert.equal((await S.decompose(JSON.stringify(items))).ok, true)
  assert.equal(S.taskRows().length, 1, 'same string retried: de-duplicated')
  const R = await w.fresh(7)                                                         // string first, then the array form, no explicit key
  assert.equal((await R.decompose(JSON.stringify(items))).ok, true)
  await R.decompose(items)
  assert.equal(R.taskRows().length, 1, 'string then array with the same content must not decompose twice')
  const E = await w.fresh(8)                                                         // explicit key
  const idem = key()
  assert.equal((await E.decompose(JSON.stringify(items), { idempotency_key: idem })).ok, true)
  assert.equal((await E.decompose(items, { idempotency_key: idem })).ok, true)
  assert.equal(E.taskRows().length, 1, 'explicit key: string then array replays')
})

test('deeply nested input fails the same way as an array and creates nothing (parity, not a new path)', async (t) => {
  const w = await world(t)
  const S = await w.fresh(9), A = await w.fresh(10)
  const depth = 200_000
  const rs = await S.decompose('['.repeat(depth) + ']'.repeat(depth))
  const nested = JSON.parse('['.repeat(depth) + ']'.repeat(depth))
  const ra = await A.decompose(nested)
  assert.equal(rs.ok, false); assert.equal(ra.ok, false)
  assert.equal(S.taskRows().length + A.taskRows().length, 0)
  console.log('# deep nested: string path ->', rs.error.slice(0, 80), '| array path ->', ra.error.slice(0, 80))
})
```
