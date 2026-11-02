# 0.3.1 PRE-INSTALL verification — independent checker (`checker-031`)

**Subject:** `@cayleb/dsh-fleet` 0.3.1 candidate tarball
`/home/cayleb/workspace/harness-build/candidate-031/dist/cayleb-dsh-fleet-0.3.1.tgz`
sha256 **`b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891`** (337,149 B, 123 members:
120×0644 + 3×0755). Branch `candidate/0.3.1` tip `aa7415eb0d60de8da85036e577e7cf03308b7ff8`, built
`7d4eb95 → 0653f82 → e98f87a → 70ee748 → 6dc1348 → aa7415e`.
**Seat:** `checker-031` (fresh-context independent checker; built none of this). Maker never checker.
**Checked:** 2026-11-02T00:56:06Z → 01:03:17Z, all runs by this seat.
**Mode:** PRE-INSTALL, package-alone in a throwaway `DSH_HOME` (`/tmp/checker-031-pre.ohS0RG`), S4A-style.
**Live fleet untouched:** no install, no `fleet-dsh` restart, no fleet-profile write; unit still `active`,
`MainPID 518634`, `InvocationID bc98f82dc22a4053b41f75f3a9e10231` at the end (same as the S5 baseline).
No credential values, no image reads, no host input, $0.

**VERDICT: FAIL (do not install this artifact as-is)** — 1 blocking defect (D1) + 1 minor defect (D2). Every other leg
passes. Smallest fix is named in §6; the tarball must be rebuilt (its sha will change) and this battery re-run.

`PREINSTALL_VERDICT: FAIL` · `D1: BLOCKING` · `D2: MINOR` · `TGZ_SHA256: b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891` ·
`CHECKED_AT: 2026-11-02T01:03:17Z` · `CHECKER: checker-031` · `LIVE_SERVICE_TOUCHED: no`

---

## 1. Per-leg table

| Leg | Status | Evidence (in `board/probes/checker-031/preinstall/`) |
|---|---|---|
| **P — provenance** (sha, members/modes, sidecar, tree↔tarball, manifest, stage2 hash, version) | **PASS** | `00-provenance/`, `02-roster/P1-*` |
| **I — package-alone install** (rc=0, normalise 123/123, boot, status, first-run==tarball, version) | **PASS** | `logs/10–22`, `logs/19-compare-frstatus.txt` |
| **R — roster/doctrine** (roster 19, mounted row, doctrine, rates, route, subagent 32, rows 8/12) | **PASS** | `02-roster/` |
| **M — MCP surface** (9 tools, auth, action log, stub session, stage_real, roundtrip) | **FAIL** — 1 step (`order.drive_roundtrip` → `no-check-run`, defect **D1**) | `03-mcp/` |
| **X — rails/rollback** (rail deltas, no bus, additivity, rollback artifacts) | **PASS** | `04-rails-rollback/` |
| **D1 — verdict gate breaks fixture verdicts** | **BLOCKING DEFECT** | `03-mcp/P8-*`, `P9-*`, `D1-verdict-gate-stage2-impact.txt` |
| **D2 — MCP `serverInfo.version` still `0.3.0-mcp`** | **MINOR DEFECT** | `03-mcp/P7-mcp-full.log`, `P11-admin-stat-logtail.json` |
| **Owner-act legs** (real-provider turn, first real order, Stage-4 activation) | **UNVERIFIED — owner-act** (not in this phase; see §7) | — |

---

## 2. PROVENANCE — PASS

| Check | Result |
|---|---|
| Tarball sha256 recomputed | `b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891` — equals the Lead's digest, the `.tgz.sha256` sidecar content and `SHA256SUMS.candidate`; 337,149 B |
| Members / modes | **123** regular members, modes exactly **120×0644 + 3×0755** (no other mode, no symlink/dir members) |
| Roster / bus | **19** `src/roster/*.js` shipped; `exports["./roster"]="./src/roster/index.js"`; **0** `src/bus/**`; no `./bus` export |
| Version | tarball `package.json` = **0.3.1**; installed version = 0.3.1; first-run readback version = 0.3.1 |
| `rows.manifest.json` | sha256 **`1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba`**, **8 pinned / 12 disabled**; running the package's own `bin/rows-manifest.mjs` against its own patch files regenerates it **byte-for-byte identical** |
| `checks/stage2-check.sh` | sha256 **`7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95`** = the S4B/S1-accepted script (unmodified) |
| Tarball == candidate tree | `tree_version=0.3.1 tree_files=123 tarball_members=123 missing=0 extra=0 sha_mode_mismatch=0` (shipped-file walk vs tarball extract) |
| Branch/tree | tip `aa7415e`; tree clean except untracked `release/install-0.3.1.sh` (not shipped) |

## 3. PACKAGE-ALONE INSTALL — PASS (throwaway DSH_HOME, umask 027, pnpm 11.28.5)

- Profile init from the default web template: rc=0, 196 rows.
- `dsh plugin --profile fleet add --save-exact --ignore-scripts <tgz>`: **rc=0**, `@cayleb/dsh-fleet 0.3.1`,
  bundles `[dsh-base, dsh-web-app, @cayleb/dsh-fleet]`.
- Pre-normalise pnpm state: 123 files, modes masked (`0640`×120, `0750/0751`), nlink=1.
- Reviewed normalise step (the installed wrapper's heredoc, hash `75ab346e…`): rc=0,
  `normalise-archive: @cayleb/dsh-fleet 0.3.1 123 regular members`,
  `normalise: 123 files single-link with archive modes; package dirs 0750`.
- Independent assert: `archive=123 files; installed=123 files; missing=0; extra=0; problems=0; dirs_wrong_mode=0`.
- Composed profile (`--dump-config` with scratch overlay): **215 rows = 196 + 19** bundle rows, 0 duplicate ids;
  `fleet-roster` mounted; `llm-pi-ai` patched; `subagent` patched.
- Scratch host boot: `ok:true`, `integrity.ok:true`, `failures:[]`, `rows.ok:true pinned=8 disabled=12`,
  `lanes_allowed:true`, `selftest:true` (see the scratch-overlay note below).
- `dsh-fleet first-run --status` (host readback) vs tarball:
  `status_files=123 tarball_members=123 missing=0 extra=0 sha/mode_mismatch=0`, version 0.3.1, `dshVersion 0.2.2-alpha.2`
  → **installed tree == tarball file-for-file**.

**Scratch-overlay note (expected, not an artifact defect):** the 0.3.1 manifest now pins the `fleet-mcp` insert row's
config; a scratch overlay must redirect its paths/port, so the rows facet reports `rows fleet-mcp: config changed`
(integrity refuses lanes). For the scratch MCP run this seat used a **scratch-only** accommodation: it rewrote only the
scratch install's `rows.manifest.json` fleet-mcp config digest to the overlay value (shipped manifest untouched,
`1970f995…`; scratch copy `e1849887…`, recorded in `02-roster/P5-*`). The shipped manifest's correctness is proven
statically (regen byte-identical) and will be judged on the live install with the shipped config.

## 4. ROSTER / DOCTRINE — PASS

- `policy-get roster.models` (live scratch control socket): exactly
  `lead anthropic/claude-sonnet-5-5`, `builder opencode-go/deepseek-v4.1-flash`, `checker anthropic/claude-haiku-5-5`,
  `researcher opencode-go/deepseek-v4.1-flash`.
- `policy-get rates`: contains `opencode-go/deepseek-v4.1-flash` = `{input 0.15, output 0.6, cache_read 0.003,
  cache_write 0}`; the three Anthropic rows unchanged.
- Composed `llm-pi-ai` row: `providers.opencode-go {apiKeyEnv: OPENCODE_GO_API_KEY, displayName: …}` — **no
  `baseURL`/`baseUrl` restated** (apiKeyEnv-only; catalog supplies the endpoint). `providers-get` shows the
  `opencode-go` llm route with `credential.present:false` and `base_url_override:null` (no owner key — expected).
- Composed `subagent` row: `maxActiveSubagents: 32`.
- Rows: manifest regen identical (8 pinned / 12 disabled, incl. the `fleet-mcp` insert row); scratch rows facet computed
  the same counts and manifest sha.
- `order.stage_real` on the roster-mounted scratch host: **`ok:true mode:"real-lane"`, order created**, staged path +
  `payload_sha256` returned — i.e. **no longer refuses `lanes-unavailable`**; the exact behaviour is recorded in
  `03-mcp/P6-*`. This is expected with `fleet-roster` mounted; see observation O2 (door/MCP text now stale).

## 5. MCP SURFACE — FAIL (one step)

Against the scratch host (nine-tool facade, loopback, dynamic port, scratch admin socket; token read in-memory by the
probe, never printed):

| Step | Result |
|---|---|
| initialize / notifications / tools/list | PASS — `fleet-mcp 0.3.0-mcp`, exactly the **nine** contracted tools |
| auth.refuse | PASS — wrong token → HTTP **401**; admin log has `refused`/`auth-invalid` rows with `principal:null` |
| fleet.status / actions.status | PASS — action log `open:true`, `broken:null`, mode `jsonl-hash-chain`, seq advancing |
| session.create / prompt / status (stub route) | PASS — labelled stub route, transport named; **not** model activation |
| board.op order-list / task-cancel refused | PASS |
| fleet.selftest board_read | PASS |
| order.stage_real | PASS — `ok mode:"real-lane"` (roster mounted; see §4) |
| **order.drive_roundtrip** | **FAIL — `refused no-check-run: roundtrip step verdict failed: verdict: the verdict is refused by the run record: no-check-run`** (defect **D1**) |
| **serverInfo.version** | **`0.3.0-mcp`, not `0.3.1-mcp`** (defect **D2**) |

**D1 differential proof:** the same scratch host with `fleet-roster` disabled (no verdict gate) runs
`order.drive_roundtrip` to **PASS** (`03-mcp/P9-roundtrip-no-roster-differential.json`); with the roster mounted it fails
at the `verdict` step. Causation is the roster's gate, not the scratch environment.

## 6. DEFECTS and smallest fix

**D1 — BLOCKING: the mounted `fleet-roster` verdict gate refuses every fixture-order command-mode verdict (`no-check-run`).**
The gate is installed globally on fleet-orders (`src/roster/index.js:145–148` → `src/roster/runcheck.js:61–68`): any
`done_when.mode === "command"` verdict needs a `run_check` record for (task, checker, round) or it is refused.
- **Observed:** MCP `order.drive_roundtrip` FAIL (a documented nine-tool tool; the door's live-proof step 3) —
  `no-check-run`.
- **Predicted blocking consequence for the live install:** the required **unmodified** `checks/stage2-check.sh`
  acceptance drives exactly this path — fixture orders with `done_when:{mode:"command",command:"true"}`
  (stage2-check.sh lines 690/710) and checker verdicts with `exit_code` (line 216) — so its verdict legs will be
  refused `no-check-run` and the required **`pass=60 fail=0 / STAGE 2: PASS`** bar cannot be met on a roster-mounted
  0.3.1 host. (Not run here: stage2 needs the live unit/sudo/polkit; the code path is identical and quoted in
  `03-mcp/D1-verdict-gate-stage2-impact.txt`.)
- **Smallest fix:** exempt fixture orders from the roster gate. In the gate wrapper (`src/roster/index.js`), after
  `const t = orders.getTask(v.task)`, look up its order (`orders.getOrder(t.order)`; the view already exposes
  `fixture: row.fixture === 1`) and `return null` when `order.fixture === true`. Keep the N4 gate for real (non-fixture)
  lanes. Then rebuild the tarball (sha changes) and re-run this pre-install battery.

**D2 — MINOR: MCP `serverInfo.version` is still `0.3.0-mcp`** (`src/mcp/server.js:15`, `src/mcp/tools.js:86`; not bumped by
the patch set). Client-visible version string mismatched with 0.3.1. **Smallest fix:** bump both to `0.3.1-mcp` (or derive
from `package.json`) in the same rebuild.

## 7. Observations (not defects)

- **O1 — scratch rows artifact:** redirecting the pinned `fleet-mcp` config fails the rows facet on a scratch overlay
  (expected; accommodation documented in §3). The live install uses the shipped config and must be judged by the
  post-install battery.
- **O2 — `order.stage_real` behaviour change:** with the roster mounted it now **creates a real (fixture:false) order**
  and returns `ok mode:"real-lane"` instead of the 0.3.0 `lanes-unavailable` refusal. `docs/MCP-FACADE.md` and
  `NOX-DOOR.md` still say it refuses; the door's step 3 will create a real order on the live host — the Lead should know
  before the door is re-issued (doc drift, not a code defect).
- **O3 —** the candidate tree has an untracked `release/install-0.3.1.sh` (the Lead's install script; not shipped).
- **O4 —** 0.3.0 acceptance is not re-litigated here: S4A 63/63, S4B 60/60, S5 83/85+named, rails 11 areas are cited as
  the accepted baseline only.

## 8. What remains for the live install (UNVERIFIED now)

1. **Blocked until D1/D2 are fixed and the tarball is rebuilt:** strike/Lead install (wrapper `plugin-add`, restart) —
   not run by this seat.
2. Post-install battery (on the Lead's go): `first-run --status` 123/0/0/0 on the installed host; rows **8 pinned /
   12 disabled / failures [] / manifest sha `1970f995…`** with the **shipped** config; unit identity before/after; MCP
   listeners exactly `127.0.0.1/[::1]/100.114.17.67:8789` (no catch-all); installed-unit MCP battery
   (`mcp-verify.mjs`, unchanged) with no new failure vs S5; auth refusals/action-log on the installed listeners;
   evidence token scan.
3. **Owner-act legs (never certified by this seat):** `OPENCODE_GO_API_KEY` entry and the real-provider
   `session.create {mode:"model"}` turn; the first real order; Stage-4 activation.
4. Rollback **not executed** (verified statically): wrapper `75ab346e…` installed, 0.3.0 tarball `8173d13e…` + sidecar
   in the drop, wrapper backup present, and `release/install-0.3.1.sh` has a hash-guarded fail-closed rollback branch
   (re-add 0.3.0, restart, status). Additivity verified from schema/code: roster migrations add **new tables only**
   (`roster`, `roster_trees`, `roster_runs`, `roster_goldfish`, `roster_stalls` + indexes/triggers); orders
   `schema.js` byte-identical 0.3.0→0.3.1; ledger schema statements unchanged; no destructive migration.

## 9. Evidence

`board/probes/checker-031/preinstall/` — 45 files, `SHA256SUMS` verified (`sha256sum -c` OK), token/PII scan **0 hits**
(ephemeral scratch web tokens redacted). Key files: `00-provenance/` (P1–P5), `logs/11-plugin-add.log`,
`logs/13-normalise.txt`, `logs/14-assert-normalised.txt`, `logs/18-status.json`, `logs/19-compare-frstatus.txt`,
`02-roster/P1–P5`, `03-mcp/P5–P12` + `D1-verdict-gate-stage2-impact.txt`, `04-rails-rollback/R1–R4`.
Scratch root kept at `/tmp/checker-031-pre.ohS0RG` (throwaway; the scratch host is stopped).

`PREINSTALL_VERDICT: FAIL` · `D1_BLOCKING: roster verdict gate refuses fixture verdicts (no-check-run) → MCP roundtrip
FAIL and the required stage2 installed-output acceptance is predicted to fail` · `D2_MINOR: MCP serverInfo 0.3.0-mcp` ·
`SMALLEST_FIX: exempt fixture orders in src/roster/index.js gate wrapper + bump SERVER_VERSION to 0.3.1-mcp, rebuild,
re-run this battery` · `TGZ_SHA256: b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891` ·
`CHECKED_AT: 2026-11-02T01:03:17Z` · `CHECKER: checker-031`
