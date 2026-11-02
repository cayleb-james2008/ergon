# 0.3.1 INSTALLED-OUTPUT acceptance — independent checker (`checker-031`)

**Subject:** `@cayleb/dsh-fleet` **0.3.1** installed on `outis-desktop` (fleet-dsh unit, live host).
Artifact `cayleb-dsh-fleet-0.3.1.tgz` sha256 **`5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4`** (123 members,
r2 candidate), installed by the Lead's guarded script via the reviewed wrapper (`plugin-add` after `systemctl stop`).
**Seat:** `checker-031` (fresh-context independent checker; built none of this). Maker never checker.
**Checked:** 2026-11-02T01:16:35Z → 01:21:40Z, all runs by this seat.
**Verdict evidence:** `board/probes/checker-031/installed/` (130 evidence files + `SHA256SUMS`, verified; token scan 0 hits).

**VERDICT: ACCEPT (installed-output acceptance PASS)** — the authoritative unmodified `checks/stage2-check.sh`
acceptance returns **`EXIT=0`, `summary: pass=60 fail=0`, `STAGE 2: PASS`** on the installed 0.3.1 host; installed tree ==
tarball 123/123 with 0 missing/extra and 0 sha/mode mismatch; rows 8/12 `failures:[]` with the shipped manifest
`1970f995…`; unit healthy before/after. Two **named, non-blocking** items are recorded, neither a product defect: (1) the
*unchanged* MCP battery has one new failed check (`T7.honesty`) because it hard-codes the 0.3.0 "not-built" label while
0.3.1 truthfully reports `real_lane:"fleetRoster mounted"`; (2) a minor doc/behaviour mismatch: `MCP-FACADE.md` says the
roundtrip's `real_lane` label follows the host, but the shipped roundtrip result still prints the conservative
`"not-built (Stage 4 roster)"`. Owner-activation legs remain uncertifiable by this seat (§7).

`INSTALLED_VERDICT: ACCEPT` · `STAGE2: EXIT=0 pass=60 fail=0 STAGE 2: PASS` · `TGZ_SHA256: 5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4` ·
`UNIT_AFTER: active MainPID 669437 InvocationID 999771adc0a84fa7b07d3e56fe97bc28` · `CHECKED_AT: 2026-11-02T01:21:40Z` · `CHECKER: checker-031`

---

## 1. Per-leg table

| Leg | Status | Evidence (`board/probes/checker-031/installed/`) |
|---|---|---|
| **P — provenance / installed identity** (install log, drop sha+sidecar, tree==tarball, version, CLI identity) | **PASS** | `01-installed-output/`, `00-identity/`, `logs/10-install-log.txt` |
| **I — installed-output acceptance** (unmodified `stage2-check.sh`) | **PASS** — `EXIT=0`, `pass=60 fail=0`, `STAGE 2: PASS` | `01-installed-output/` (`console.out`, `console.err`, `exit-code.txt`, `P4-summary.txt`, `OUT/`) |
| **R — rows/status, shipped config** (8/12, failures [], manifest `1970f995…`, guard/chain, lanes) | **PASS** | `02-rows/` |
| **M — MCP surface, installed unit** (nine tools, listeners, auth, action log, stub session, roundtrip) | **PASS** (one named stale-battery expectation) | `03-mcp/` |
| **D — roster/doctrine/patch-set, installed** (fleet-roster mounted, doctrine, rates, route, subagent 32, stage_real) | **PASS** | `04-roster/` |
| **A — audit/fences** (token scan, no credentials/images/spend/input) | **PASS** | `05-audit/` |
| **Owner-act legs** (real-provider turn, first real order, Stage-4 activation) | **UNVERIFIED — owner-act** (§7) | — |

## 2. INSTALLED-OUTPUT ACCEPTANCE — the authoritative S4B-equivalent

Pre-run: `stage2-check.sh` sha256 **`7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95`** (unmodified
S4B script); `FLEET_CLI` = `candidate-031/package/bin/dsh-fleet.mjs` sha256 `3d839901…` = the **installed-tree readback
sha** = the tarball member sha (so the check ran the shipped/installed CLI); unit before: active,
`MainPID 663330`, `InvocationID cd1208c35d68470b8ba84e32f44db50a`.

```
$ OUT=<evidence>/01-installed-output/OUT FLEET_CLI=<candidate>/package/bin/dsh-fleet.mjs \
  bash <candidate>/checks/stage2-check.sh          # 2026-11-02T01:16:58Z
EXIT=0
summary: pass=60 fail=0 (evidence: .../board/probes/checker-031/installed/01-installed-output/OUT)
STAGE 2: PASS
trap: cancelled 70 of 1082 fixture orders (append-only rows remain by design)
```

- Re-derived from the log: `^PASS ` = **60**, `^FAIL ` = **0**, `NOT-RUN` = **0**; `console.err` = **0 bytes**;
  `OUT/stage2-check.out` byte-identical to `console.out`.
- The check's own 2.11 leg SIGKILLed and restarted the unit — `PASS 2.11-kill — fleet-dsh SIGKILLed mid-burst and back
  up (MainPID 663330 -> 669437)`; the unit came back active (`InvocationID 999771adc0a84fa7b07d3e56fe97bc28`). That
  restart is the check's documented behaviour, not a checker action; the battery's MCP leg afterwards left the unit
  unchanged.
- **Why it matters:** this is the required installed-output bar, and it is the exact path the r1 defect broke (fixture
  command-mode verdicts). 60/60 confirms the D1 fix works on the live installed host.

## 3. Provenance / identity — PASS

- Install log (`~/workspace/harness-build/evidence/strike/s3-install-0.3.1/install-20261102T011524Z.log`):
  preconditions ok (wrapper `75ab346e…`, tarball `5ede7484…`, rollback `8173d13e…`, status ok) → stop → plugin-add
  0.3.0→0.3.1 → **`normalise-archive: @cayleb/dsh-fleet 0.3.1 123 regular members`** →
  `normalise: 123 files single-link with archive modes; package dirs 0750` → start → `status ok/integrity ok` →
  `READBACK version=0.3.1 files=123 tarball=123 missing=0 extra=0 badModes=0` → **`INSTALL-0.3.1: PASS`**.
- Drop: `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.1.tgz` sha256 = `5ede7484…`; conventional `.sha256` sidecar present
  and matching (leg 2.1a satisfied); 0.3.0 rollback tarball `8173d13e…` + sidecar still present.
- Installed tree readback (`first-run --status` from the running host) vs tarball: **before** stage2 and **after**
  stage2 both `status_files=123 tarball_members=123 missing=0 extra=0 sha/mode_mismatch=0`, name `@cayleb/dsh-fleet`,
  version **0.3.1** → installed tree == tarball file-for-file.
- Identity timeline: install `518634 / bc98f82d…` → `663330 / cd1208c3…`; stage2's own 2.11 kill → `669437 /
  999771ad…`; after MCP battery and final read: **active, `669437 / 999771ad…`** (no unplanned restart).
- Listeners at every read: exactly `127.0.0.1:8789`, `[::1]:8789`, `100.114.17.67:8789`; no `0.0.0.0`/`::` catch-all.

## 4. Rows / status with the SHIPPED config — PASS

Control `{"op":"status"}` before and after stage2: `ok:true`, `integrity.ok:true`, `failures:[]`,
`rows {ok:true, pinned:8, disabled:12, failures:[], manifest_sha256:"1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba"}`,
`chain.ok:true`, `sandbox.ok:true`, `guard.ok:true` (`fleet-policy`, `fleet-ledger`), `lanes_allowed:true`,
`policy.selftest:true`, WAL. (The scratch-overlay accommodation used pre-install does **not** apply here — the live
host composes the shipped manifest and passes.)

## 5. MCP surface on the INSTALLED unit — PASS (one named stale-battery expectation)

- **Nine tools** exactly (`T2` loopback+tailnet: `fleet.status, session.create, session.prompt, session.status,
  board.op, order.drive_roundtrip, order.stage_real, fleet.selftest, actions.status`); initialize
  `server fleet-mcp 0.3.1-mcp`; admin stat `version:"0.3.1-mcp"`, binds exactly the three addresses.
- **Auth refusals pre-dispatch:** missing token → HTTP **401** `-32001` with `WWW-Authenticate: Bearer`; wrong token →
  401; action log gained `refused` rows `auth-missing`/`auth-invalid` with `principal:null` (readable tail: 19 refused
  rows incl. `missing-token`, `invalid-token`, `payload-too-large`, `parse-error`, `rate-limited`).
- **Action log:** `open:true`, `broken:null`, `mode:"jsonl-hash-chain"`, seq 97; T6 chain recompute + intent-before-
  outcome + tail agreement all PASS; full-file scan remains a stated uid-1000 limitation (SKIP, as in S5).
- **Stub route only, no activation claim:** `session.create` = `mode:"stub"`, model
  `deterministic-stub (fleet-selftest-stub)`; `session.prompt` = `transport:"stub-turn"`, `ledger_gate:
  "stub-provider-selftest-only"`, `provider_calls:0`. `session.create {mode:"model"}` still refuses
  **`model-credential-not-entered`** (owner key not entered) — recorded, not faked.
- **Roundtrip:** `order.drive_roundtrip` PASS (T4 and T4b): fixture orders `o5461`/`o5462` driven to `done`,
  `mode:"fixture-validated"`, verified over the control socket — the r1 defect is fixed live.
- **Unchanged accepted battery** (`mcp-verify.mjs` sha256 `8449913ac8186825e39004333818a9974f6893dc6cd129abef7c5575c6e7c1e0`,
  copied unchanged): **81 checks, 74 passed, 3 failed, 4 informational**.
  - `T3.prompt` / `T3.status` — the **same two failures as the accepted S5-installed baseline** (installed-host
    stub-turn returns empty assistant text, transport/ledger gate named); unchanged condition, not a 0.3.1 regression.
  - **`T7.honesty` — new failed check, named:** the unchanged battery asserts `/not-built/i` on
    `fleet.status.mcp.real_lane`; the roster-mounted 0.3.1 host truthfully reports
    `real_lane:"fleetRoster mounted"`. The label is honest and matches 0.3.1; the **battery expectation is stale**
    (0.3.0-era). Not a product defect.
  - Check count 81 vs S5's 85: T5 branch A (6 staged-refusal checks) is replaced by branch B (2 real-lane checks)
    because the roster is mounted — expected, not missing coverage.

## 6. Roster / patch-set facts on the INSTALLED host — PASS

- `sudo -n -u fleet dsh-fleet-admin dump-config` → 213 composed rows; **`fleet-roster` mounted** (`@cayleb/dsh-fleet/roster`,
  paths under `/var/lib/fleet`); **`llm-pi-ai`** = `providers.opencode-go {apiKeyEnv: OPENCODE_GO_API_KEY, displayName}`
  with **no `baseURL` restated**; **`subagent`** = `maxActiveSubagents: 32`.
- `policy-get roster.models` = lead `anthropic/claude-sonnet-5-5`, builder `opencode-go/deepseek-v4.1-flash`, checker
  `anthropic/claude-haiku-5-5`, researcher `opencode-go/deepseek-v4.1-flash`; `policy-get rates` has
  `opencode-go/deepseek-v4.1-flash` = `{input 0.15, output 0.6, cache_read 0.003, cache_write 0}`.
- `providers-get`: `opencode-go` llm route present, **credential `present:false`** (owner key not entered — expected),
  `base_url_override:null`; engines `claude-code`/`codex` not installed (expected).
- **`order.stage_real` observed behaviour (exact):** `ok:true`, `mode:"real-lane"`,
  `real_lane:"fleetRoster mounted"`, created real order **`o5463`** (`fixture:false`, state `open`, principal
  `cayleb-local`), staged path + `payload_sha256` returned — **no `lanes-unavailable` refusal**. It was **not fired**;
  this seat then cancelled the acceptance-created order via `order-cancel` → `state:"failed"` (cleanup recorded in
  `04-roster/P8-cancel-o5463.txt`).

## 7. What remains uncertifiable (plainly)

1. **Owner credential / real-provider turn** — `OPENCODE_GO_API_KEY` is not entered; `session.create {mode:"model"}`
   still refuses `model-credential-not-entered`. No activation is claimed from the stub route.
2. **First real order** — not fired. The one real order `o5463` created by the battery's `stage_real` check was
   cancelled (not driven).
3. **Stage-4 activation** (real lanes, engine CLIs, `stage4-check.sh`) — not exercised; engines absent, egress Tier-B
   act untouched.
4. **Rollback not executed** — path verified as armed only: 0.3.0 tarball `8173d13e…` + sidecar in the drop, wrapper
   backup, install script's hash-guarded fail-closed rollback branch.
5. **Named non-blocking observations:** (a) the unchanged MCP battery's `T7.honesty` expectation is stale for a
   roster-mounted host (product label truthful); (b) `docs/MCP-FACADE.md` says the roundtrip's `real_lane` label
   follows the host, but shipped `board.js` still hard-codes `"not-built (Stage 4 roster)"` in the roundtrip result
   (conservative — it understates, never claims a real lane); (c) `NOX-DOOR.md` (outside the package) still describes
   `stage_real` refusing `lanes-unavailable` — update before the door is re-issued; (d) installed-host
   `first-run --status.dshVersion` reads `null` (informational; the acceptance bar does not use it).

## 8. Audit / fences — PASS

Evidence token/secret scan: **0 hits**; no image files (0), no screenshots, no credential values read (providers show
`present:false`; the MCP bearer token was read in memory by the unchanged probe and never printed/stored); `$0` spend;
no host input injection; the checker performed no install and no restart — the only restart was the plugin's own stage2
2.11 leg. Evidence root: 130 evidence files + `SHA256SUMS`, verified.

`INSTALLED_VERDICT: ACCEPT` · `STAGE2_LINE: summary: pass=60 fail=0 (evidence: board/probes/checker-031/installed/01-installed-output/OUT) / STAGE 2: PASS / EXIT=0` ·
`INSTALLED_TREE: 123/123 missing=0 extra=0 sha_mode_mismatch=0 version=0.3.1` ·
`ROWS: pinned 8 disabled 12 failures [] manifest 1970f995…` ·
`UNIT: active MainPID 669437 InvocationID 999771adc0a84fa7b07d3e56fe97bc28` ·
`MCP_BATTERY: 81 checks / 74 pass / 3 failed (T3.prompt+T3.status = S5 baseline; T7.honesty = stale battery expectation) / 4 informational` ·
`TGZ_SHA256: 5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4` · `CHECKED_AT: 2026-11-02T01:21:40Z` · `CHECKER: checker-031`
