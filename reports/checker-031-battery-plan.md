# Fleet 0.3.1 acceptance battery — independent checker plan (PREP)

**Seat:** `checker-031` — fresh-context independent checker for `@cayleb/dsh-fleet` 0.3.1 (reused team slot; zero
involvement in the fleet build). **Maker never checker.**
**Authority:** Nox `20261102T002000Z` §1/§3 (strike owns the 0.3.1 build, tarball, derived-count install, installed
identity readback and the fresh checker over installed output) + the Lead's battery brief.
**Subject:** `@cayleb/dsh-fleet` **0.3.1** built from `tb/roster-0.3.1` (`7d4eb95` base → `e98f87a` fix round →
`70ee748` patch set; branch tip `70ee7480a89027078fb054147190c78b2d51ed0e`).
**Status: PREP ONLY — 2026-11-02T00:53Z.** The 0.3.1 artifact does not exist yet. Nothing in this plan has been run
against the installed service: no install, no restart, no service call, no credential value read, no image read, no
host input, $0. This document is the battery to execute **when the Lead opens the candidate phase**.

Accepted 0.3.0 bars this plan builds on (cited, **not re-litigated**):
`strike-checker-s4a-package-alone.md` (63/63 package-alone), `strike-checker-s4b-stage2-installed.md` (installed output
`pass=60 fail=0`, `STAGE 2: PASS`), `strike-checker-verify-s5-installed.md` (MCP battery 85 checks / 83 pass / 2 named
strict stub-text failures accepted under the Lead's OR), `strike-rails-identity.md` (11 rail areas byte-identical
0.2.2→0.3.0).

Evidence root (layout defined, currently empty): [`board/probes/checker-031/`](../probes/checker-031/README.md).

---

## 0. Candidate preconditions (staging gates) — checked first; unmet gate ⇒ **BLOCKED**, no artifact verdict

| Gate | Check (exact) | Pass bar | Evidence |
|---|---|---|---|
| **G0** | Strike announces: tarball path + sha256, release-tree path, install-log path, merge commit(s), installed version | all five present, in writing | `00-preflight/G0-candidate-announcement.txt` |
| **G1** | `sha256sum "$TGZ" "$DIST"`; compare with the published digest (`SHA256SUMS.strike` / release note) | three values equal | `00-preflight/G1-hashes.txt` |
| **G2** | `ls -l /var/lib/fleet-drop/cayleb-dsh-fleet-0.3.1.tgz.sha256`; `sha256sum -c` it | sidecar exists and matches the tarball (stage2 leg 2.1a **hard-requires** it; S4B had to restore a missing 0.3.0 sidecar) | `00-preflight/G2-sidecar.txt` |
| **G3** | `git -C "$REL" log -1 --format='%H %s'`; `git -C "$REL" status --porcelain`; shipped-file equality tarball↔release tree (§P6) | HEAD is the documented 0.3.1 release commit, tree **clean**, tarball members byte-identical to `$REL/package/` shipped files | `00-preflight/G3-release-tree.txt` |
| **G4** | `sha256sum /usr/local/bin/dsh-fleet-admin` | `75ab346e8b314c25b1da630c78ded154efa5669aff62c474afa7da2d53c01209` (the derived-count normalise wrapper; already installed for 0.3.0 — if it differs, the owner relay act is outstanding ⇒ BLOCKED) | `00-preflight/G4-wrapper.txt` |
| **G5** | `sha256sum "$CHK"` | `7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95` — the **unmodified** stage-2 check accepted in S4B; any change ⇒ BLOCKED until re-reviewed | `00-preflight/G5-check-hash.txt` |
| **G6** | Install log from strike: `grep -E "INSTALL-0\.3\.1: PASS|normalise-archive|READBACK|status ok" <log>` | `INSTALL-0.3.1: PASS`; `normalise-archive: @cayleb/dsh-fleet 0.3.1 <N> regular members`; `READBACK version=0.3.1 files=<N> tarball=<N> missing=0 extra=0`; `status ok/integrity ok after restart` | `00-preflight/G6-install-log.txt` |
| **G7** | `systemctl show fleet-dsh -p ActiveState,MainPID,InvocationID`; `node "$REL/package/bin/dsh-fleet.mjs" first-run --status` | unit `active`; status `ok:true`; installed `version=0.3.1`; `N` files | `00-preflight/G7-post-install-identity.txt` |
| **G8** | Tarball member scan: `src/bus/` absent, `src/roster/*.js` present (19), member count `N = 123` (104 + 19) | **no `src/bus/**`** (the Stage-5 checkpoint Team B says is *not mergeable*; its `files[]`/exports would ship 10 unwired files ⇒ 133 members), roster present, `N=123` | `00-preflight/G8-member-shape.txt` |

Environment block used by every command below:

```bash
EVID=/home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/checker-031
REL=<strike's 0.3.1 release tree>            # e.g. /home/cayleb/workspace/harness-build/strike-release-031
TGZ=/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.1.tgz
DIST=$REL/dist/cayleb-dsh-fleet-0.3.1.tgz
OLD=/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.0.tgz
OLD_SHA=8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070
CHK=$REL/checks/stage2-check.sh
CHK_SHA=7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95
WRAPPER=75ab346e8b314c25b1da630c78ded154efa5669aff62c474afa7da2d53c01209
DSH=/home/cayleb/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js
CONTROL=/run/fleet/control.sock ; ADMIN=/run/fleet/mcp-admin.sock
# run as cayleb (uid 1000); /var/lib/fleet is 0750 fleet:fleet — the installed tree, token and log are NOT readable
# from this seat by design. Static checks read the readable tarball; installed-tree equality is proven by the host's
# own `first-run-status` readback (sha256+mode per file), not by reading the installed tree.
```

Run order: **G** → **P** (read-only) → **R** static/composed (read-only) → **M** read-only battery → **I** installed
output (service-touching; the check's own 2.11 restarts the unit) → **R7/M5/M7** post-stage2 → **X** → verdict.
Every command is logged with cwd + UTC timestamp + exit code.

---

## 1. PROVENANCE (P) — read-only

| ID | Command (exact) | Expected / pass bar | Evidence file | Why it matters |
|---|---|---|---|---|
| **P1** | `sha256sum "$TGZ" "$DIST"` | equal, and equal to the published digest; version `0.3.1` | `01-provenance/P1-tarball-hash.txt` | The artifact under test must be the bytes strike released, not a rebuild. |
| **P2** | `python3 "$EVID/scripts/tar-members.py" "$TGZ"` (tarfile walk) | regular members `N=123` (G8); modes **only** `0644`/`0755`; no symlinks, no absolute/`..` names, no dirs-with-files anomalies | `01-provenance/P2-members-modes.txt` | Archive hygiene + the normalise contract (modes are the identity binding). |
| **P3** | `tar -xzOf "$TGZ" package/package.json \| jq -r .version` | `0.3.1` | `01-provenance/P3-version.txt` | Version is a release gate, not a comment. |
| **P4** | `node "$REL/package/bin/dsh-fleet.mjs" first-run --status > "$EVID/01-provenance/frstatus-before.json"` (exit 0) | host readback: `name=@cayleb/dsh-fleet`, `version=0.3.1`, `files=123` | `01-provenance/P4-frstatus-before.json` | The running host's own view of the installed tree; the only permitted installed-tree read. |
| **P5** | `python3 "$EVID/scripts/compare-frstatus.py" "$EVID/01-provenance/frstatus-before.json" "$TGZ" 0.3.1` | `status_files=123 tarball_members=123 missing=0 extra=0 sha/mode_mismatch=0` → `compare-frstatus: PASS` | `01-provenance/P5-compare-frstatus.txt` | **Installed tree == tarball** file-for-file, hash and mode included. |
| **P6** | Derived-count identity: grep the install log (`normalise-archive:` line, `normalise: … single-link` line, `READBACK` line) and compare the three numbers with P2/P4 | `normalise-archive: @cayleb/dsh-fleet 0.3.1 123 regular members`; `normalise: 123 files single-link with archive modes; package dirs 0750`; `READBACK version=0.3.1 files=123 tarball=123 missing=0 extra=0`; all four counts equal | `01-provenance/P6-derived-count.txt` | The **derived-count install path**: count is derived from the hash-verified archive, never pinned; identity readback is the release's own proof. |
| **P7** | Release-tree provenance: `git -C "$REL" log --oneline -3`; confirm `e98f87a` and `70ee748` are in history (or strike's documented merge commit) and `7d4eb95` is the base; `git status --porcelain` empty; shipped-file equality tarball↔`$REL/package/` (`P5` pattern with a worktree walk) | clean tree; documented commit chain; 0 mismatched shipped files | `01-provenance/P7-release-tree.txt` | Tarball must be built from the reviewed merge, not hand-assembled. |
| **P8** | MCP server version string (also M7): `tar -xzOf "$TGZ" package/src/mcp/server.js \| grep SERVER_VERSION`; initialize `serverInfo.version` | expect `0.3.1-mcp`. **Prep risk:** branch source hardcodes `'0.3.0-mcp'` (`src/mcp/server.js:15`, `src/mcp/tools.js:86`) and the patch set does not bump it — if strike did not, record as version-string FAIL (smallest fix: bump both, rebuild) | `01-provenance/P8-mcp-version.txt` | "version 0.3.1" must hold on every surface a client sees. |

---

## 2. INSTALLED-OUTPUT ACCEPTANCE (I) — the ruling's required leg; service-touching, run only after G passes

| ID | Command (exact) | Expected / pass bar | Evidence file | Why it matters |
|---|---|---|---|---|
| **I1** | Identity **before**: `systemctl show fleet-dsh -p ActiveState,MainPID,InvocationID`; `node "$REL/package/bin/dsh-fleet.mjs" first-run --status > "$EVID/02-installed-output/frstatus-before.json"`; control `{"op":"status"}` | unit active; `version=0.3.1`, `files=123`; `ok:true integrity.ok:true failures:[] lanes_allowed:true selftest:true` | `02-installed-output/I1-identity-before.txt` | Freeze the state the acceptance starts from; no checker restart may happen later. |
| **I2** | `mkdir -p "$EVID/02-installed-output/OUT"; OUT="$EVID/02-installed-output/OUT" FLEET_CLI="$REL/package/bin/dsh-fleet.mjs" bash "$CHK" >"$EVID/02-installed-output/console.out" 2>"$EVID/02-installed-output/console.err"; echo "EXIT=$?" >"$EVID/02-installed-output/exit-code.txt"` | `EXIT=0`; `console.err` **0 bytes**; `summary: pass=60 fail=0`; `STAGE 2: PASS`; re-derived `^PASS ` = **60**, `^FAIL ` = **0**, no `NOT-RUN`; `stage2-check.out` byte-identical to `console.out` | `02-installed-output/OUT/`, `console.out`, `console.err`, `exit-code.txt`, `pass-table.txt` | The plugin's own **UNMODIFIED** acceptance check against the installed 0.3.1 output — the ruling's installed-output bar. 0.3.0 was 60/60; the rails are unchanged (§X1), so 60/60 is the bar. |
| **I3** | Identity **after**: same reads as I1 + `ss -ltn \| grep :8789`; `grep -E "2.11-kill" console.out` | `version=0.3.1`, `files=123`, sha/mode 0 mismatch vs I1 (installed identity identical); unit active; MainPID transition matches the check's own `2.11-kill` line (`A -> B`); no other restart; listeners still exactly the three addresses | `02-installed-output/I3-identity-after.txt` | Proves the check's documented kill/restart is the only state change, and the installed artifact survived it identically. |
| **I4** | Stage-2 side effects: `grep -cE "^PASS " console.out`; grep trap line; `board.op order-list` readback for a `real_lane`/fixture label | fixture orders cancelled by the EXIT trap (append-only rows remain by design); **no real (non-fixture) order created**; `real_lane:"not-built (Stage 4 roster)"` where present | `02-installed-output/I4-side-effects.txt` | The acceptance must not have created a real order (that is a Stage-4/owner matter, not certified here). |
| **I5** | Post-run control `{"op":"status"}` | `ok:true`, `integrity.ok:true`, `failures:[]`, `rows.ok:true pinned=8 disabled=12`, `lanes_allowed:true`, `selftest:true` | `02-installed-output/I5-status-after.json` | The rails still pass after the acceptance; feeds R6. |

---

## 3. ROSTER / DOCTRINE (R) — static from the tarball + composed config + one MCP call

Static checks read the **tarball** (readable); the tarball==installed equality is P5, so the shipped bytes are the
installed bytes.

| ID | Command (exact) | Expected / pass bar | Evidence file | Why it matters |
|---|---|---|---|---|
| **R1** | `tar -tzf "$TGZ" \| grep '^package/src/roster/.*\.js$' \| sort`; `tar -xzOf "$TGZ" package/package.json \| jq -r '.exports["./roster"], .files[]'` | exactly the **19** roster `.js` files (brief, checker, checkflow, confined, control, engines, fake, goldfish, host, index, invalid, kit, lanes, roster, runcheck, sensing, store, table, tree); `exports["./roster"]="./src/roster/index.js"`; `files[]` includes `src/roster/*.js` | `03-roster-doctrine/R1-roster-present.txt` | The 0.3.1 patch set's item (a): the roster module must ship in the package export. |
| **R2** | `node --input-type=module -e "import('$EXTRACT/package/src/roster/table.js').then(m=>console.log(JSON.stringify(m.SHIPPED_MODELS)))"` | exactly `lead=anthropic/claude-sonnet-5-5`, `builder=opencode-go/deepseek-v4.1-flash`, `checker=anthropic/claude-haiku-5-5`, `researcher=opencode-go/deepseek-v4.1-flash`; no extra/missing role | `03-roster-doctrine/R2-doctrine.json` | Nox owner go `20261102T003300Z`: the doctrine table, not a design-doc default. |
| **R3** | Control `{"op":"policy-get","key":"rates"}` via `node "$EVID/scripts/admin-op.mjs" --socket "$CONTROL"` | `ok:true`, `source:"default"`, value contains `opencode-go/deepseek-v4.1-flash` = `{input_per_mtok:0.15, output_per_mtok:0.6, cache_read_per_mtok:0.003, cache_write_per_mtok:0}`; the three Anthropic rows unchanged | `03-roster-doctrine/R3-rates.json` | The working engine must be **priced**, or the ledger refuses `unknown-model`; this is the patch-set item (c). |
| **R4** | `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin dump-config > "$EVID/03-roster-doctrine/composed.yml"`; extract the `- id: llm-pi-ai` block | row present+enabled; `providers.opencode-go.apiKeyEnv: OPENCODE_GO_API_KEY`; `displayName` present; **no `baseURL`/`baseUrl`/`base_url` key** in that block (apiKeyEnv-only; the pi-ai catalog supplies the endpoint) | `03-roster-doctrine/R4-route.txt` | Patch-set item (b): the route shape must not restate an invented endpoint. |
| **R5** | same `composed.yml`; extract `- id: subagent` | `maxActiveSubagents: 32` (not the 8 default) | `03-roster-doctrine/R5-subagent.txt` | Patch-set item (d): headroom ceiling. |
| **R6** | `composed.yml` row count/ids + control `status.integrity.rows`; `sha256sum` the tarball `package/rows.manifest.json` | `rows.ok:true`, `pinned=8`, `disabled=12`, `failures:[]`, `manifest_sha256 == sha256(tarball rows.manifest.json)`; all 8 pinned ids present/enabled in the composed dump (incl. the `fleet-mcp` insert row that 0.3.0's stale 7-pin manifest omitted); the 12 disabled ids absent/disabled | `03-roster-doctrine/R6-rows.txt` | Patch-set item (d): manifest regen must **compose**, not just ship. |
| **R7** | MCP `order.stage_real` with the door payload (`~/.dsh/strike-door/staged-order.json` fields: title/goal/done_when/example/est_minutes) via `mcp-selftest --script` or the battery; then `board.op {"op":"order-list"}` readback | **record the exact refusal**: `lanes-unavailable` (roster installed but **not mounted** — expected on this merge) or the exact new named refusal; staged path + `payload_sha256` equal to the checker's own recomputation; **no order row created**; roster-mounted state recorded from `composed.yml` (`fleet-roster` row present/absent). If it returns `ok` **and creates an order**, stop: Stage-4 activation detected — **not certified** here (separate Stage-4 battery/ruling) | `03-roster-doctrine/R7-stage-real.json` + `.log` | The Lead's explicit "no longer `lanes-unavailable` — or exact new refusal" leg; and it must not silently activate Stage 4. |

---

## 4. MCP SURFACE (M) — read-only battery against the installed listeners

| ID | Command (exact) | Expected / pass bar | Evidence file | Why it matters |
|---|---|---|---|---|
| **M1** | `ss -ltn \| grep ":8789" > "$EVID/04-mcp/listeners.txt"`; `systemctl show fleet-dsh -p IPAddressAllow -p IPAddressDeny` | exactly **three** listeners: `127.0.0.1:8789`, `[::1]:8789`, `100.114.17.67:8789`; **no** `0.0.0.0`/`[::]` catch-all; every bind inside the unit's allowlist | `04-mcp/M1-listeners.txt` | Surface exposure is part of the security contract. |
| **M2** | `node "$REL/package/bin/mcp-selftest.mjs" --url http://127.0.0.1:8789/mcp --admin-socket "$ADMIN" --skip-roundtrip --json > "$EVID/04-mcp/mcp-selftest.json"` (read-only proof: no board rows mutated) | tools/list advertises **exactly nine** tools: `fleet.status, session.create, session.prompt, session.status, board.op, order.drive_roundtrip, order.stage_real, fleet.selftest, actions.status`; `initialize` 200 + session id; every non-strict step PASS; the strict `session.prompt` `S3STUB:` text check is the accepted S5 installed-host deviation (§M6) — record exactly which step fails | `04-mcp/M2-mcp-selftest.json` | The nine-tool contract. |
| **M3** | unchanged accepted battery: `node "$EVID/scripts/mcp-verify.mjs" --mode real --url http://127.0.0.1:8789/mcp --tailnet-url http://100.114.17.67:8789/mcp --admin-socket "$ADMIN" --control-socket "$CONTROL" --out "$EVID/04-mcp/battery"` | auth refusals **pre-dispatch**: missing token → HTTP 401 `-32001`; wrong token → 401; `WWW-Authenticate: Bearer`; admin `log_tail` shows `refused` rows `missing-token`/`invalid-token` with `principal:null` and **zero intent rows** for them; both loopback and tailnet legs initialize + 9 tools | `04-mcp/battery/` (`mcp-verify.json`, raw/) | Refusals must be before any effect, and logged. |
| **M4** | `actions.status` + admin `{"op":"log_tail","n":100}` (readable scope only; the raw file is fleet-owned and unreadable by uid 1000 — named limit) | `open:true`, `broken:null`, chain `jsonl-hash-chain`; every tool call has intent-before-outcome (`ref_seq`), 9/9 paired; args stored as **names + sha256 digest only** (never values); seq/head advance | `04-mcp/M4-action-log.json` | The audit trail is the non-repudiation evidence. |
| **M5** | session checks (inside M2/M3): `session.create {mode:"stub"}` → `model:"deterministic-stub (fleet-selftest-stub)"`; `session.prompt` → `transport`/`turn_agent`/`ledger_gate` named; `session.status` found | labelled stub route only; **this is NOT provider activation and will not be reported as such** | `04-mcp/M5-stub-session.json` | Distinguish "session works" from "model route works". |
| **M6** | Compare the unchanged accepted battery result with the accepted S5 baseline (`85 checks, 83 pass, 2 named strict stub-text failures, 4 informational`) | **no NEW failure** beyond the accepted named installed-host stub-text deviation; if the deviation disappeared, record it (better, not required); diff recorded check-by-check | `04-mcp/M6-vs-s5-baseline.txt` | 0.3.0 acceptance is not re-litigated: same battery, same bar, delta only. |
| **M7** | `initialize` → `serverInfo.version`; `fleet.status` version field | `0.3.1-mcp` (see P8 risk); if `0.3.0-mcp`, FAIL this sub-check, smallest fix = bump `SERVER_VERSION` + the `tools.js` default and rebuild | `04-mcp/M7-server-version.txt` | Client-visible version must match the release. |
| **M8** | Whole-evidence scan: `grep -rInE 'Bearer [A-Za-z0-9]{16,}\|token[\"=: ]+[A-Za-z0-9]{16,}' "$EVID"` → 0 hits; token never printed/stored (probes read it from the admin socket in memory) | 0 token/credential hits in the evidence root | `04-mcp/M8-token-scan.txt` | Fence: no credential values anywhere. |

Owner-act-dependent sub-leg (BLOCKED until the owner acts; **not** part of the pre-owner verdict):

| ID | Command | Expected | Status rule |
|---|---|---|---|
| **M9a** (before owner key) | `mcp-selftest --script '[{"rpc":"initialize"},{"tool":"session.create","args":{"mode":"model","prompt":"state your role in one line"},"expect":{"error":"model-credential-not-entered"}}]' --json` | named refusal `model-credential-not-entered`, no session, no provider call | **PASS as fail-closed**; records the owner act as outstanding |
| **M9b** (after owner enters `OPENCODE_GO_API_KEY` — owner act; I never enter/read the value) | same with `mode:"model"`, then `session.prompt`, `session.status`, `ledger-get`/`usage-feed` | real route session; model/route names `opencode-go/deepseek-v4.1-flash`; `provider_calls > 0`; non-empty assistant text; spend recorded in the ledger | **BLOCKED-ON-OWNER** until the key is entered; then run and certify only this turn |

---

## 5. RAILS & ROLLBACK (X) — 0.3.0 acceptance not re-litigated

| ID | Command (exact) | Expected / pass bar | Evidence file | Why it matters |
|---|---|---|---|---|
| **X1** | Extract both tarballs to `/tmp/checker031/{new,old}`; for each of the **11 rail areas** run `diff -rq old/package/<area> new/package/<area>`: `src/core src/orders src/policy src/ledger src/lessons src/sandbox src/egress src/fsguard src/housekeeper src/client presets` | 10 areas **empty**; `src/ledger` differs **only** in `ledger.js`, and the diff is exactly the rates row (+2 lines, 0 removed); presets byte-identical; no other rail file changes | `05-rails-rollback/X1-rail-diff.txt` | The board/lease/ledger/containment code the Stage-2 acceptance exercises is the accepted code; the only rail change is the priced working-engine row. |
| **X2** | Full shipped member-set diff vs 0.3.0 (`diff` of `tar -tzf` lists + per-member sha256) | added = exactly the 19 `src/roster/*.js`; changed ⊆ {`package.json`, `cordis.patch.yml`, `rows.manifest.json`, `src/ledger/ledger.js`, optionally `src/mcp/server.js`/`src/mcp/tools.js` **only** for the M7 version bump}; removed = 0; all other members byte-identical; **no `src/bus/**`** | `05-rails-rollback/X2-member-diff.txt` | No unplanned shipped delta; catches the Stage-5 bus checkpoint leaking into the release. |
| **X3** | Cite accepted 0.3.0 evidence; do **not** re-run package-alone/rails review | S4A 63/63, S4B 60/60, S5 83/85+named, rails 11 areas — cited as baseline only | `05-rails-rollback/X3-baseline-citations.txt` | The Lead's instruction: 0.3.0 acceptance is not re-litigated. |
| **X4** | Rollback path (verify, do **not** execute): `sha256sum "$OLD" /var/lib/fleet-drop/cayleb-dsh-fleet-0.3.0.tgz.sha256`; `ls -l /usr/local/bin/dsh-fleet-admin.pre-0.3.0-20261101`; `grep -nE "OLD_TGZ\|OLD_SHA\|reinstalling 0\.3\.0\|0\.2\.2 restored" "$REL/release/install-0.3.1.sh"` | 0.3.0 tarball + sidecar present and hash `8173d13e…`; wrapper backup present; the install script contains the fail-closed branch that reinstalls the accepted previous release and restarts, plus the documented one-command rollback | `05-rails-rollback/X4-rollback.txt` | Rollback must be a real, hash-verified path — verified statically, **not exercised** (would be a second disruptive install; named limit). |

---

## 6. What I will NOT certify (plain)

1. **Owner credential / real-provider turn before it happens.** `OPENCODE_GO_API_KEY` is not entered in the fleet
   profile at prep time. M9b stays **BLOCKED-ON-OWNER**; I never enter or read the key value.
2. **First real (non-fixture) order** — not driven by this battery. `order.drive_roundtrip` (if run at all) is
   fixture-validated only.
3. **Stage-4 orders if the roster is installed-but-not-activated.** If `fleet-roster` is not mounted, `order.stage_real`
   correctly refuses; that is the expected state and is recorded, **not** certified as Stage-4 readiness.
4. **Roster activation from module presence, stub route, or `order.stage_real` staging.** The stub route is labelled
   `deterministic-stub`; it is not model activation.
5. **Rollback execution** — artifacts and script logic verified only.
6. **0.3.0 acceptance re-litigation** — S4A/S4B/S5/rails cited, not re-certified; package-alone scratch install is not
   repeated (installed-tree==tarball + installed-output stage2 are this line's acceptance).
7. **The Stage-5 `src/bus` checkpoint** — if it ships, that is a REJECT, not a "minor extra".
8. **RUN/other gates, model spend, credentials, images** — none touched. No image reads, no host input, $0.

---

## 7. Verdict format (per-leg ACCEPT/REJECT table)

Statuses: **PASS** / **FAIL** / **BLOCKED-ON-OWNER** / **BLOCKED-ON-STAGING** / **NOT-RUN (named)**.
Overall rule: **ACCEPT** iff every mandatory leg (P, I, R, M, X) is PASS and no FAIL; **REJECT** if any mandatory leg
FAILs — name the smallest concrete fix (e.g. "exclude `src/bus` from `files[]`/`exports` and rebuild", "bump
`SERVER_VERSION` to `0.3.1-mcp` and rebuild", "add the missing `.tgz.sha256` sidecar"); **BLOCKED** if a staging gate
(G0–G8) is unmet (no artifact verdict). Owner-act legs are recorded as BLOCKED-ON-OWNER and never counted as PASS.

`06-verdict/REPORT.md` will carry this table, then the machine lines:

```
LEG P provenance        PASS|FAIL   (tarball <sha> · members <N> · installed==tarball 0/0/0)
LEG I installed-output  PASS|FAIL   (EXIT=0 · pass=60 fail=0 · STAGE 2: PASS · identity before==after)
LEG R roster/doctrine   PASS|FAIL   (roster 19 · doctrine 4/4 · rates row · apiKeyEnv-only · subagent 32 · rows 8/12)
LEG M mcp-surface       PASS|FAIL   (9 tools · 3 listeners · auth refusals · action log · no new failures vs S5)
LEG X rails/rollback    PASS|FAIL   (11 areas · ledger rates row only · 0 unplanned deltas · rollback artifacts)
LEG O owner-act         BLOCKED-ON-OWNER (M9b real provider turn; Stage-4 real order)
VERDICT_031: ACCEPT|REJECT|BLOCKED · TGZ_SHA256: <sha> · INSTALLED_FILES: <N> · CHECKED_AT: <ISO8601Z> · CHECKER: checker-031
```

---

## 8. Prep-time facts already established (read-only; no service runs)

- **0.3.0 accepted baselines** (cited): S4A 63/63 on final `8173d13e…`; S4B `EXIT=0`, `pass=60 fail=0`,
  `STAGE 2: PASS`; S5 85 checks/83 pass/2 named strict stub-text fails (accepted OR); rails 11 areas byte-identical
  0.2.2→0.3.0; final 0.3.0 tarball = 104 members (101×0644 + 3×0755).
- **0.3.1 expected shape from `70ee748`** (Team B branch): `package.json` version `0.3.1`, `exports["./roster"]`, `files`
  includes `src/roster/*.js`; `src/roster/` = 19 shipped `.js`; `cordis.patch.yml` +23 lines (`llm-pi-ai` providers
  opencode-go apiKeyEnv-only, `subagent` maxActiveSubagents 32); `src/ledger/ledger.js` +2 lines (the rates row);
  `rows.manifest.json` regenerated to **8 pinned / 12 disabled** with the `fleet-mcp` insert row; expected tarball
  **123 members = 104 + 19**.
- **Candidate risks to verify at candidate time** (not claims): (i) `SERVER_VERSION` still `'0.3.0-mcp'` in branch
  source — likely stale unless strike bumps it (P8/M7); (ii) `files[]`/exports on the branch include
  `src/bus/*.js` + `./bus` (Stage-5 checkpoint, not mergeable) — if strike merges `70ee748` wholesale the tarball would
  ship 10 unwired bus files (133 members) ⇒ REJECT unless deliberately excluded; (iii) the 0.3.0 drop tarball was
  missing its conventional `.sha256` sidecar (S4B lesson) — G2 checks it before stage2; (iv) known installed-host
  deviation: `stub-turn` returns empty `assistant_text`/`provider_calls:0` (accepted S5; expect the same, do not treat as
  a 0.3.1 regression and do not claim model activation); (v) installed profile composition has a known 2-row delta
  (`ui-context-menu`, `ui-file-actions` only in the shipped template scratch) — the rows facet checks pinned/disabled
  only, so record it as an environment note.
- **Access model:** `/var/lib/fleet` is `0750 fleet:fleet`; installed tree, token and action log are unreadable by uid
  1000 by design. Static checks read the tarball; installed equality comes from the host's `first-run-status` readback;
  `dump-config` is a sanctioned `sudo -n -u fleet dsh-fleet-admin` read (sudoers NOPASSWD, unchanged verb list); stage2
  runs as cayleb and uses its own sanctioned sudo/polkit verbs.
- **Fences for the candidate phase:** checker never installs/restarts; the only restarts are the stage-2 check's own
  documented 2.11 kill and strike's install; token read in-memory only; no credential values; no image reads; no host
  input; $0; read-only on origin; verdict written to `board/reports/` and evidence under `board/probes/checker-031/`.

---

**Standing by for the candidate.** When the Lead opens the phase I will run G0–G8 first; if they hold, the battery in
this order, writing the per-leg verdict and machine line above. Plan of record for the evidence root:
[`board/probes/checker-031/README.md`](../probes/checker-031/README.md).
