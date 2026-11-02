# TEAM B — BATCH 8: Stage-4 ROSTER FIX ROUND + 0.3.1 PATCH SET (HANDOVER TO STRIKE)

Author: `fleet-lead` (Team B), 2026-11-02 ~00:20–01:05Z. Worktree only: **no install, no service action, no profile
write, no tarball packed** (Nox 20261102T002000Z §1: strike owns the roster build and the 0.3.1 release line; Team B
supplies the fix round). Spend $0. No credential values handled.

## 0. HANDOVER BOTTOM LINE

| Item | Value |
|---|---|
| Branch | **`tb/roster-0.3.1`** in repo `cayleb-james2008/dsh-fleet` (local; not pushed) |
| Worktree | `/home/cayleb/workspace/harness-build/scratch-tb-roster031` |
| Base | `7d4eb95` (strike/0.3.0 release tree; contains merged main history through `4a8a1d3`) |
| Rebased content commit | `0653f82` — the `bus` checkpoint `e8c8a4c` replayed onto `7d4eb95` (only `package/package.json` conflicted: files list kept both sides) |
| Fix-round commit | **`e98f87a`** — roster F1/F2/F3/F4 + regression tests (merge this) |
| Patch-set commit | **`70ee748`** — 0.3.1 items (a)–(e) below (strike reviews/merges; version/tarball/install are strike's) |
| Adaptation commit | `695644f` — rebase adaptation onto 0.3.0 (`RESERVED_PRINCIPAL`→`reservedKind`) + `.gitignore` |
| Branch tip | **`70ee7480a89027078fb054147190c78b2d51ed0e`** |
| Bus warning | `package/src/bus` on this branch is the Stage-5 builder checkpoint and is **NOT mergeable as-is**: 13 tests, 5 pass / 8 fail (needs its own wiring onto 0.3.0). Nox/strike should merge `src/roster` + the patch-set files, not `src/bus`. |

**Maker-side proof (run on the clean tip, this worktree):** roster suite **102/102 pass, 0 fail**; mutation suite
**8/8 mutants killed** (freshness + lane cap; unmutated baseline passes first); core **99/99**; ledger **12/12**;
policy **65/65**; orders **196/196**. Commands in §4.

## 1. FIX ROUND (commit `e98f87a`) — what changed and why

### F1 CRITICAL — `src/roster/tree.js` (review `review_stage4_roster.txt` F1, line 169)
Old: blobs written first, then the tree hash recomputed; a crafted tree could name a symlink `d` and an entry `d/x`
(valid to `git hash-object -t tree --literally`) so the host wrote through the symlink (e.g. into
`/var/lib/fleet/skills`), and `gitwork/<id>.git` + `.home` (with the maker's absolute object path in
`objects/info/alternates`) were left readable by lanes.
New:
- `assertNoPathCollisions(entries)` refuses duplicate names and any path that is both an entry and an ancestor of
  another entry (`tree-unsupported`) — this is the symlink-prefix class;
- the tree identity is computed from the `ls-tree` entries and compared to the commit's tree **before the first
  filesystem write** (`tree-mismatch`);
- directories first, regular files next, **symlinks last**, so no write can follow a link this extractor made;
  the symlink parent is lstat-checked as a plain directory;
- new `extractTree({ cleanupGit })`: the production path (`index.js` extract) removes `gitDir` + `home` on success
  **and** on every error path, so no lane can read another lane's paths.
- Regression test: `test/tree.test.mjs` "F1 (critical): a hand-crafted tree that names a path and a path under it is
  refused before anything is written, and no file escapes" — real git plumbing (`hash-object --literally`), asserts
  refusal, that no file appears in the outside target dir, and that the refused tree was removed.

### F2 HIGH — `src/roster/checkflow.js` + `index.js`
Old: `const insp = host.inspect(session)` (no `await`; the real `host.inspect` is `async`) → every real Checker start
failed closed `not-fresh: missing board_read; missing verdict; missing run_check`. The suite passed only because the
fake host was synchronous.
New: `await host.inspect(session)`; the test fake is `async` so the suite mirrors production; the level-triggered
`tick()` now re-runs `flow.kick()` so a `built` task with no live Checker (checker died/lease expired) is retried
without a restart.
Regression test: `test/checkflow.test.mjs` "F2: the Checker freshness inspection is awaited" (delayed async inspector;
before the fix this refused not-fresh).

### F3 HIGH — `src/roster/tree.js` + `checkflow.js` + `invalid.js` (N6)
Old: the "maker-authored check" changed set was diffed against the built commit's **first parent**, so a maker could
commit `check.sh` (exit 0) in commit 1 and the implementation in commit 2 and submit commit 2 — `check.sh` was never
in `changed`.
New: `readCommit({ since })` computes the changed set as the built commit's changes **plus every ancestor commit whose
committer time is at or after the task's `claimed_at` minus a 10-minute skew tolerance** (max 500 ancestors). `checkflow`
passes `since: t.facts.claimed_at`; without a claim time the old first-parent semantics stand unchanged. The set only
ever grows, so the rule stays fail-closed. `invalid.js` doc updated.
Regression test: `test/tree.test.mjs` "F3 (N6): with the task claim time, a script committed earlier in the same build
is maker-authored" — shows first-parent alone misses `check.sh` and the claim baseline catches it.
Limit recorded: a commit whose committer timestamp was rewritten to before the claim escapes this rule (rebase/clock
skew); the Checker still judges the build, and a claim-time workspace snapshot would close it fully (needs an orders
touchpoint, named as remaining work, not smoothed).

### F4 HIGH — `src/roster/engines.js` + `table.js`
Old (review F4): usage unrecorded on timeout/kill; Codex `turn.completed` usage events dropped past an 8 MiB head cap;
no vendor budget flag for Codex; envelope spent => Claude ran uncapped; day cap not reserved; `sourceSeq` was an
in-memory counter that restarted at 1 after a restart; engine lanes took the **builder role's** model id.
New:
- `TailCapture` class keeps the **tail** (64 KiB–8 MiB cap) of stdout, so a killed/truncated run's last usage events
  are still parsed and **recorded on non-zero/timeout exits** (the record call already preceded the exit check; now the
  parse sees the tail);
- `room <= 0` ⇒ `failTask('budget-exhausted', …)` — an exhausted task never launches an engine uncapped;
- the engine run sequence is persisted on the lane row (`book.update(session, { runs })`), so a restart cannot restart
  it at 1 and make the ledger drop spend;
- `ENGINE_MODELS` (`table.js`): `claude-code` prices/launches as `anthropic/claude-haiku-5-5`; `codex` remains
  `openai` (unpriced ⇒ `unknown-model` fails closed) — the Codex vendor-model gap is **named**, not fixed by name
  invention (Job 10's Codex decision is still owed).
- Named remaining limits (not smoothed): the **day cap** is still asked once at launch and not reserved across
  concurrently starting engine lanes (needs the ledger reserve API, Stage-4 touchpoint 3d); Codex has no vendor
  budget flag, so it is bounded by the envelope refusal + timeout, not a flag.
- Regression tests: `test/engines.test.mjs` "F4: the stream capture keeps the TAIL…" (>9 MiB noise then usage) and
  "F4: an exhausted task envelope refuses the launch…".

## 2. PATCH SET (commit `70ee748`) — items (a)–(e) for strike to take, not release-owned by Team B

- **(a) Roster module included in the package export** — already present from the `bus` checkpoint and kept:
  `package.json` `exports["./roster"] = "./src/roster/index.js"`, `files` includes `src/roster/*.js`.
  After merge, `dsh plugin add` ships it; the fleet bundle patch does **not** mount `fleet-roster` yet — strike must add
  the bundle row (`- id: fleet-roster / name: '@cayleb/dsh-fleet/roster'`) and any Stage-4 touchpoints
  (`setLaneRoots` in fleet-core, `setVerdictGate`/`invalidCheck`/`harnessPost`/`releaseClaims`/`harnessBlock`/views in
  fleet-orders, engine rates in fleet-ledger) per the builder report's touchpoint list — those were **never applied**
  in `bus` and are NOT in this branch.
- **(b) `llm-pi-ai` provider route** — `package/cordis.patch.yml` appends:
  `- id: llm-pi-ai / config: { providers: { opencode-go: { apiKeyEnv: OPENCODE_GO_API_KEY, displayName: … } } }`.
  **Endpoint ruling (Nox 20261102T002000Z §2): cloud OpenCode Go base URL; the local `:49374` instance is NOT a fleet
  route** (unverified ghost from the Lane 6 marker; a local route is a separate migration ruling). The actual base URL is
  strike's to pick/own; this patch deliberately restates **no URL** — the pi-ai catalog supplies it: evidence
  `@earendil-works/pi-ai@1.3.0/dist/providers/data/opencode-go.json` carries per-model
  `baseUrl: "https://opencode.ai/inference/go"` (and `/go/v1` for `deepseek-v4.1-flash`), model id
  **`deepseek-v4.1-flash`** exists in that catalog (29 models), cost 0.15 in / 0.6 out / 0.003 cacheRead per Mtok,
  context 1,000,000, maxTokens 384,000. If strike prefers an explicit `baseURL`, use the catalog value above —
  do not take the URL from this report as a ruling, it is catalog evidence.
- **(c) `rates` row** — `src/ledger/ledger.js` default rates gains
  `'opencode-go/deepseek-v4.1-flash': { input_per_mtok: 0.15, output_per_mtok: 0.6, cache_read_per_mtok: 0.003,
  cache_write_per_mtok: 0 }` (catalog cost above), so `policy-get rates` no longer refuses the working engine with
  `unknown-model`.
- **(d) `subagent` row config** — `package/cordis.patch.yml` appends
  `- id: subagent / config: { maxActiveSubagents: 32 }` (Nox 141900Z: headroom, not working shape; sub-lead tiers
  span 3–5 remain the design). `rows.manifest.json` **regenerated** with `bin/rows-manifest.mjs`
  (`DSH_RUNTIME_NM=<dsh checkout node_modules>`): now **8 pinned / 12 disabled** — 0.3.0 shipped 7 pinned and omitted
  the `fleet-mcp` insert row that its own bundle patch (`mcp.row.yml`) lists. Strike should keep the regenerated file;
  if strike disagrees, the fallback is to re-add `pinMounts` handling deliberately, but 7-pinned was stale.
  **Integration risk named:** the installed 0.3.0 profile will boot 0.3.1 with the new manifest; `fleet-mcp`'s row
  config must match its composed value (it does if the 0.3.0→0.3.1 install composes the same `mcp.row.yml`).
- **(e) version + release** — `package.json` bumped to **0.3.1** in this branch (proposed). **Strike owns the tarball,
  a derived-count/identity install, the fresh checker over installed output, and the identity readback** (Nox
  20261102T002000Z §3). Team B did **not** run `npm pack`, did not write `/var/lib/fleet-drop`, did not touch the unit.
- Doctrine table — `src/roster/table.js` `SHIPPED_MODELS`: lead `anthropic/claude-sonnet-5-5`, builder
  `opencode-go/deepseek-v4.1-flash`, checker `anthropic/claude-haiku-5-5`, researcher
  `opencode-go/deepseek-v4.1-flash` (Nox owner go 20261102T003300Z; overrides the design doc's Anthropic-only table).
  `control.js` `providers-get` now also shows the `opencode-go` llm route so the providers view cannot contradict the
  table.
- Tests updated to the doctrine/manifest: roster 102/102, core 99/99, ledger 12/12, policy 65/65, orders 196/196.

## 3. FILES TOUCHED (vs `7d4eb95`; `git diff --stat 7d4eb95 70ee748`)

Roster module added (from `bus`): `package/src/roster/**` (src + 15 test files + API/NOTES), `checks/stage4-check.sh`,
`checks/fixtures/s4-fake-llm.py`, `src/bus/**` + `checks/stage5-check.sh` (STAGE-5 checkpoint, **not mergeable** — see
§0). Team-B edits in these commits: `package/src/roster/tree.js`, `checkflow.js`, `index.js`, `engines.js`,
`invalid.js`, `table.js`, `control.js`, `test/tree.test.mjs`, `test/checkflow.test.mjs`, `test/engines.test.mjs`,
`test/helpers.mjs`, `test/lanes.test.mjs`, `test/control.test.mjs`, `test/roster.test.mjs`, `test/plugin.test.mjs`;
`package/src/ledger/ledger.js`; `package/src/core/test/cli.test.mjs`, `plugin.test.mjs`, `rows.test.mjs`;
`package/cordis.patch.yml`, `package/rows.manifest.json`, `package/package.json`, `.gitignore`.

## 4. CHECKER HANDOFF — exact commands

```bash
cd /home/cayleb/workspace/harness-build/scratch-tb-roster031
git log --oneline -4                      # 70ee748 0.3.1 patch set | e98f87a fix round | 695644f adaptation | 0653f82 rebased bus
git rev-parse HEAD                        # 70ee7480a89027078fb054147190c78b2d51ed0e
# roster suite (102 expected)
cd package/src/roster && TMPDIR=<disk dir> /usr/bin/node --import ../core/test/_register.mjs --test test/*.test.mjs
# mutant proof (8 expected, needs the check fixtures dir at package/../checks/fixtures)
cd package/src/roster && TMPDIR=<disk dir> /usr/bin/node test/mutate.mjs
# other suites touched by the patch set
cd package/src/core   && /usr/bin/node --import ./test/_register.mjs --test test/*.test.mjs      # 99/99
cd package/src/ledger && /usr/bin/node --import ../core/test/_register.mjs --test test/*.test.mjs # 12/12
cd package/src/policy && /usr/bin/node --import ../core/test/_register.mjs --test test/*.test.mjs # 65/65
cd package/src/orders && /usr/bin/node --import ../core/test/_register.mjs --test test/*.test.mjs # 196/196
# manifest identity (should print 8 pinned / 12 disabled and match the shipped file)
cd package && DSH_RUNTIME_NM=/home/cayleb/.local/opt/deepseek-harness-official/node_modules node bin/rows-manifest.mjs
```
Fresh checker for the release is the Lead's; maker never checker — this file is maker evidence only.

## 5. LIMITS / NOT DONE (honest)

- **`src/bus` is not wired** (Stage 5): 13 tests, 5 pass / 8 fail on this base; do not merge it as-is.
- **Stage-4 touchpoints are not applied**: `fleet-roster` is not mounted in the bundle patch, and fleet-core/fleet-orders
  changes from the builder report (setLaneRoots, verdict gate, invalid-check, harness post kinds, releaseClaims, views,
  ledger reserve API) are still owed. Without them the roster cannot run on the installed host even after merge.
- **F3 residual**: rewritten-committer-date commits before the claim are not caught; claim-time workspace snapshot is
  the full fix (needs the orders touchpoint).
- **F4 residuals**: day-cap reservation across concurrent engine lanes (needs ledger reserve API); Codex vendor-model id
  + no vendor budget flag (Job 10 decision owed); `reportedCostUsd` from Claude is not yet used to reconcile the priced
  row (ledger prices from tokens; recorded as-is).
- **No install/tarball/service action was performed** (Nox 20261102T002000Z §1/§3); the 0.3.1 release line, the actual
  OpenCode Go base URL choice, the fresh checker over installed output and the identity readback are strike's.
