# TEAM B — 0.3.1 CANDIDATE r2 (checker-031 fixes) — NOT INSTALLED

Response to `board/reports/checker-031-preinstall.md`. Same rails: scratch worktree only; **nothing installed, no
service/profile touch, no credential value, no publication**, `$0`. Fresh checker re-runs the whole pre-install
battery on this sha before anything installs.

## 1. Commits / branch

| | |
|---|---|
| Worktree / branch | `/home/cayleb/workspace/harness-build/candidate-031`, **`candidate/0.3.1`** |
| New tip | **`741db66747672f5549204749a9ae1eea239cfef7`** (r2 fix commit) |
| Parent | `aa7415e` (r1 touchpoints) ← `6dc1348` (bus out) ← `70ee748` (patch set) ← `e98f87a` (fix round) ← `0653f82` (rebased bus content) ← `7d4eb95` (0.3.0) |
| Diff of r2 | 5 files: `package/src/roster/index.js`, `package/src/roster/test/gate.test.mjs` (new), `package/src/mcp/server.js`, `package/src/mcp/tools.js`, `docs/MCP-FACADE.md` |

## 2. What changed

### D1 (BLOCKING) — fixture orders exempt from the roster N4 verdict gate
`src/roster/index.js` now exports `makeVerdictGate(orders, rc)` and installs it via `orders.setVerdictGate(...)`:

```js
export function makeVerdictGate(orders, rc) {
  return verdict => {
    const task = orders.getTask(verdict.task)
    const order = task === null || task === undefined ? null : orders.getOrder(task.order)
    if (order?.fixture === true) return null          // D1: fixtures have no Checker lane, so no run_check can exist
    return rc.verdictGate({ ...verdict, doneWhenMode: task?.done_when?.mode, briefHashes: task?.built?.artefact_hashes ?? [] })
  }
}
```
Real (non-fixture) lanes keep the full N4 gate; rubric and `invalid-check` paths are unchanged. This is exactly the
checker's smallest fix (fixture test after `getTask`/`getOrder`, before `rc.verdictGate`).

**Proof (unit, at the same seam the MCP roundtrip and `stage2-check.sh` drive):** new
`package/src/roster/test/gate.test.mjs` (3 tests) uses the **real** `runcheck` (`createRunChecks`) over a real store:
1. fixture order + command done-when + pass/fail verdict → gate returns `null` (was `no-check-run` before r2);
2. real (non-fixture) lane, no run record → still `no-check-run`;
3. real lane rubric pass / `invalid-check` → still `null`.
This is the in-process path behind `order.drive_roundtrip` (fixture order) and the stage2 fixture verdict legs; the
full MCP roundtrip scratch-boot re-run is the checker's own P9 harness (I did not re-run the scratch MCP battery in
this window — named, not claimed).

### D2 — MCP version
`package/src/mcp/server.js` `SERVER_VERSION` and `package/src/mcp/tools.js` default: `0.3.0-mcp` → **`0.3.1-mcp`**
(tarball readback below proves it in the shipped bytes).

### O2 — doc truth for the roster-mounted door (checked-in doc updated; NOX-DOOR is outside this package)
`docs/MCP-FACADE.md` updated: `order.stage_real` now **CREATES** the real non-fixture order and answers
`{mode:"real-lane", real_lane:"fleetRoster mounted"}` when the 0.3.1 roster is mounted (it does not refuse
`lanes-unavailable` as 0.3.0 did); the roundtrip stays fixture-only and its `real_lane` label follows the host;
fixture verdicts are exempt from the gate; sample server version `0.3.1-mcp`; the refusal table gained the
roster-mounted row. **Flag for Lead/Nox:** `NOX-DOOR.md` lives outside the package — its step 3 will now create a
real order on a live roster host; update it before the door is run.

## 3. Rebuild (same way: `rows-manifest.mjs` + `npm pack`)

| Fact | r2 value |
|---|---|
| Tarball | `/home/cayleb/workspace/harness-build/candidate-031/dist/cayleb-dsh-fleet-0.3.1.tgz` |
| **sha256** | **`5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4`** |
| Sidecar / sums | `cayleb-dsh-fleet-0.3.1.tgz.sha256` + `SHA256SUMS.candidate`, both carry the r2 sha |
| Size / members | 337,454 B; **123 members** (unchanged: fix files replace shipped files, new test/doc are not in `files[]`) |
| Modes | **120×0644 + 3×0755** (0.3.0 discipline; no other mode) |
| Version readback | tarball `package/package.json` version **0.3.1**; tarball `src/mcp/server.js` **`SERVER_VERSION = '0.3.1-mcp'`** |
| `rows.manifest.json` | sha256 `1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba`, **8 pinned / 12 disabled** (unchanged) |
| `package.json` | sha256 `631eaeb18a257173765ed3fd7334d78fd213697bb52cbb219ef148386b87de81` (unchanged) |
| `checks/stage2-check.sh` | unchanged `7df6344c…` — S4B command reusable unmodified |

## 4. Suites + mutants (r2, clean tree)

roster **105/105** (102 + 3 D1 tests), core **100/100**, orders **200/200**, ledger **13/13**, policy **65/65**,
mcp **78/78**; roster mutation suite **8/8 mutants killed** (unmutated baseline passes first). Commands as in the r1
report (`package/src/<c>`, `--import ../core/test/_register.mjs --test test/*.test.mjs`; mutants:
`cd package/src/roster && node test/mutate.mjs`).

## 5. Install command (NOT RUN — for Lead approval after the fresh checker re-runs)

```bash
install -m 0644 -o root -g root dist/cayleb-dsh-fleet-0.3.1.tgz        /var/lib/fleet-drop/
install -m 0644 -o root -g root dist/cayleb-dsh-fleet-0.3.1.tgz.sha256 /var/lib/fleet-drop/
systemctl --no-ask-password stop fleet-dsh
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin plugin-add \
  5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4 cayleb-dsh-fleet-0.3.1.tgz
systemctl --no-ask-password start fleet-dsh
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin status && sudo -n -u fleet /usr/local/bin/dsh-fleet-admin dump-config
```
Rollback unchanged: stop → `plugin-add 8173d13e… cayleb-dsh-fleet-0.3.0.tgz` (still in `/var/lib/fleet-drop`) →
start. Migrations are additive.

## 6. Expected pre-install battery outcome on r2 (what the fresh checker should now see)

- `P9` roundtrip on a roster-mounted scratch host: **passes** (fixture order; gate exempts fixtures) — the r1
  differential `no-check-run` is the defect fixed here.
- Unmodified `checks/stage2-check.sh` on the installed host: `summary: pass=60 fail=0`, `STAGE 2: PASS`, `EXIT=0` —
  stage2's fixture command verdicts (script lines 690/710, verdict `exit_code` at 216) are no longer refused
  `no-check-run` by the roster. **This is still the checker's run to make**, not a claim of mine.
- `initialize` → `server fleet-mcp 0.3.1-mcp`; `dump-config` → `fleet-roster` row, `subagent.maxActiveSubagents: 32`,
  `llm-pi-ai` `providers.opencode-go {apiKeyEnv OPENCODE_GO_API_KEY}` (no value); `policy-get rates` →
  `opencode-go/deepseek-v4.1-flash`; rows 8/12; listeners and nine MCP tools unchanged.
- Still **must not certify**: activation (owner key), real-provider turns, `stage4-check.sh`, engines, hands/bus.

## 7. Residuals carried (unchanged from r1 except where noted)

verdicts CHECK stores the harness `invalid-check` row with source `checker`/principal `harness` (widening needs a
table rebuild); Codex model id + `openai/<model>` rates row owed (Job 10); day-cap reservation across concurrent
engine launches open; `lessons-list` op not added (roster needs only `pending`); `egress.allow` for engine CLIs is an
owner Tier-B act; D1's full scratch MCP battery re-run and all live readbacks remain the fresh checker's (nothing was
installed).
