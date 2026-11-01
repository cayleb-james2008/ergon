# TEAM B — BATCH 7 (2026-11-01 ~23:35–00:15Z) — OC attestation, north-star measurement, installed stub-turn, sub-agent lever, stage readiness, ROSTER PREP

Author: `fleet-lead` (Team B). All work read-only. No spend ($0). No credential values anywhere. No fleet service
change/restart; no roster config written (ownership pinned to Nox/strike — prep only). Evidence probes:
`board/probes/team-b-batch7/`. Timestamps below are from `stat`/run output (UTC unless noted; host TZ EST=UTC-5).

Absolute times used: dump-config written 2026-11-01T23:39:08Z; policy/rates read 23:45:17Z; stub-turn repro 23:45:30Z;
unknown-model probe 23:47:08Z; status read 23:49:00Z; selftest-op preset probe 23:49:39Z.

---

## B0. Priority #0 — STAGE-4 ROSTER: READ-ONLY PREP (ownership pinned to Nox/strike; nothing installed/modified)

**State: prep package complete enough to hand over in one filing.** No fleet-profile write, no service action, no
roster config written, no MCP `order.stage_real` call executed (fleet state write refused by the Lead's fence for
this turn). Everything below is design/location/payload text ready for the seat Nox names.

### (a) Roster design (to Nox doctrine, not the pre-doctrine design doc)

| Roster element | Value | Basis |
|---|---|---|
| Working engine (builder + worker lanes) | **DeepSeek V4.1 Flash via OpenCode Go** — provider `opencode-go`, model `deepseek-v4.1-flash` | Nox owner go 20261102T003300Z/233413Z; "permanent operating model". Overrides design doc's Haiku 5.5 builder. |
| Lead seat | **Claude Sonnet 5.5** (`anthropic/claude-sonnet-5-5`) | design §4.2 roster table (lead = Sonnet 5.5); Claude seat retained per Nox ("Claude seats where the roster design calls for lead/checker roles") |
| Checker seat | **Claude Haiku 5.5** (`anthropic/claude-haiku-5-5`) | design §4.2; also preserves maker≠checker across model families (DeepSeek maker / Claude checker) |
| Researcher seat | DeepSeek V4.1 Flash via OpenCode Go (fallback: Claude Haiku 5.5 if only diary route reachable) | roster design keeps one Researcher; cheapest working engine; flagged as a design choice for Nox |
| Lanes | 5-lane cap, builders 1–5, one Lead, one Researcher, sixth worker refused; Checker is per-task, not a worker slot | roster build `src/roster/lanes.js` (97/97 tests) |
| Sub-lead tier | a **roster row with its own span (3–5)**, not a sixth worker lane, when a parent hits the ceiling | Lead's standing rule + Nox operating rule (ceiling = headroom, not working shape); design §3.5/line 330 |

**Meter/rates prerequisite (currently missing):** the installed fleet policy `rates` table carries **only
`anthropic/*` keys** — `anthropic/claude-haiku-5-5`, `claude-sonnet-5-5`, `claude-opus-5-5`. Command
`dsh-fleet op policy-get '{"key":"rates"}'` → `ok:true, source:"default"` with those three keys only
(`policy-rates-20261101T2355Z.json`). **There is no `opencode-go/deepseek-v4.1-flash` rate row, so a builder lane on
the doctrine engine would be refused `unknown-model` by `lane-spawn`/the ledger gate.** The roster build must add that
rate row (price per Mtok) before any lane on OpenCode Go can run.

**Design-vs-order conflict, recorded explicitly:** `strike-release/docs/fleet-plugin-design.md` §"Stage 4: Roster and
maker–checker on real models" (~lines 805-820) predates today's doctrine: it specifies ONE Anthropic API key and models
Haiku 5.5 builder/checker + Sonnet 5.5 lead, caps $5/day + $0.50/drill. Nox's owner go overrides the **provider plan**
(working engine = DeepSeek V4.1 Flash via OpenCode Go). The Stage-4 check's engine legs (4.10 Claude engine, 4.11 Codex
engine) and the design's credential step still assume the Anthropic key; the roster build needs a doctrine-compatible
amendment: Claude remains an **engine/seat** for lead/checker, OpenCode Go becomes the **working model route** for
builder/worker lanes. Check author must be told this is an amendment, not a defect, before stage4-check is finalised.

**Mechanism (design §4.2 / §3.5, line 321-330):** four `preset` rows already ship (`preset-fleet-lead`,
`-builder`, `-checker`, `-researcher`; `rows.manifest.json` pinned insert rows); the roster table is a fleet-core
policy row (`roster.models`) mapping role→provider/model; lane identity via `setPrincipalResolver`; lane workspaces via
`setLaneRoots` (Stage 4 touchpoint 1 in the builder report); agents are created/resumed with `ctx.agents.create/resume`
under the lane's preset; one lane workspace `<work>/lanes/<principal>-<id>`, never a selftest root (R5-12).

### (b) Exact lever for a REAL provider route on installed 0.3.0 — every file/op involved

1. **Provider route config (the missing piece).** The installed fleet profile's `llm-pi-ai` row is mounted **bare**
   (`- id: llm-pi-ai / name: '@deepseek-ai/dsh-llm-pi-ai'`, no `config` → schema default `providers: {}`).
   Evidence: `dump-config-20261101T2340Z.txt` lines 79-80; the shipped code says it in words at
   `package/src/firstrun/capabilities.mjs:104-116`: *"the fleet's llm-pi-ai row is mounted with an EMPTY providers
   config … so the fleet exposes no opencode/openrouter routes today — the provider stage is not built."*
   The route is declared by adding to the profile patch row `llm-pi-ai`:
   `config.providers.opencode-go = { apiKeyEnv: OPENCODE_GO_API_KEY, api: 'openai-completions', baseURL: <endpoint>, models: [{ id: 'deepseek-v4.1-flash', contextWindow: …, maxTokens: … }], displayName: … }`.
   Schema/shape documented in `@deepseek-ai/dsh-llm-pi-ai` (checkout
   `packages/llm/llm-pi-ai/src/index.ts:1-58`); provider profiles resolve per request over the `llm-pi-ai`
   user-settings section (`settingsPath: ['providers', provider]`, index.ts:160) → **no restart needed for a key/
   endpoint/model change once the route set exists**.
2. **File that holds it (installed):** `PROFILE_PATCH_FILENAME = cordis.patch.yml` under the profile home →
   **`/var/lib/fleet/dsh/profiles/fleet/cordis.patch.yml`** (fleet:fleet 0750; unreadable by uid 1000). Read-out is
   only via the NOPASSWD wrapper verb `dump-config` (used for this report). Write paths: fleet web UI Settings/Models
   (the settings service edits the profile patch — `packages/settings/settings/src/index.ts`, "form edits over Cordis
   profile patches"; `settings-controller` row is enabled in the fleet profile, dump line 481) **or** a package-side
   patch in a next release.
3. **Credential:** `OPENCODE_GO_API_KEY` → fleet credential store `DSH_FLEET_CREDENTIALS_FILE`, default
   `/var/lib/fleet/dsh/.credentials.yaml` (0600, fleet-owned), written by the guided first run's credential step
   (`dsh-fleet first-run --credentials`, TTY as fleet user; `src/firstrun/credentials.mjs`) or the fleet web UI Models
   page. Static field list: `src/firstrun/credentials.mjs:66-71`.
4. **Model selection:** installed default is still `agent-default-model: provider deepseek-official, model
   deepseek-flash` (dump lines 54-58). A lane/session must select `opencode-go/deepseek-v4.1-flash`; the roster table
   does that per role; the MCP facade's `session.create` mode `model` is the smoke-test surface.
5. **Roster component itself:** NOT in 0.3.0 (`package.json` exports/files have no `./roster`; `order.stage_real`
   refuses `lanes-unavailable`, proven live in S5-installed T5 with `real_lane: "not-built (Stage 4 roster)"`).
   Source exists on git branch **`bus` @ `e8c8a4c`** in `~/workspace/harness-build/dsh-fleet-bus/` (14 test files,
   97/97 pass) but is based on `8095fbc` (pre-R6/0.2.2/0.3.0) and its review found **1 CRITICAL (F1, tree.js:169
   symlink write-through), 3 HIGH (F2 checkflow async inspect, F3 invalid-check bypass, F4 engine metering gaps)**,
   7 medium, 4 low — `strike-release/evidence/review-results/review_stage4_roster.txt`. **No fix round has run.**
   → The installed 0.3.0 cannot be "given a roster" by config alone; a 0.3.1/0.4.0 release must carry the fixed roster
   + the provider route patch (with `rows.manifest.json` regenerated if any pinned row changes — there is a generator
   `package/bin/rows-manifest.mjs`), then install via `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin plugin-add
   <sha256> <tgz>` (NOPASSWD verb) with the fresh-checker + owner steps the fleet install already uses.
6. **Endpoint decision still owed:** the doctrine route's baseURL is not in any config (see B1). It must be named
   before the route can be declared. **The local `:49374` instance is not a candidate** for the fleet: it is an
   owner-run OpenCode server (pid 18797, uid 1000) outside the fleet profile; the fleet runs as uid 950 under
   containment and would have to be reviewed to reach an owner desktop service. That is a design decision for
   Nox/Lead, not something this prep makes.

### (c) Owner's ONE act — exact credential-field text (draft, for Nox to announce)

What he runs (his hand, fleet TTY): `sudo -u fleet /var/lib/fleet/npm/bin/dsh-fleet first-run --credentials`
(or the fleet web UI **Models** page; `src/firstrun/checks.mjs:177` names that alternative as the owner act).

What he sees (verbatim prompt, echo off before it prints, `credentials.mjs:355`):
```
OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:
```
- One field per provider, each presented exactly once; Enter skips (owner may enter it later); the value is written
  write-only to the fleet credential store and never echoed, logged, or returned; the summary after the run shows
  only `FIELD  provider  official/UNOFFICIAL  prompted/not-prompted`.
- Non-interactive invocation (no TTY) exits 3 with `credential-entry-required` and prints the owner-act list.
- **Decision required at announcement time:** if the doctrine route is the Cloud Go gateway, that one field is
  enough; if it is to be an OpenCode-Go-style bridge to the local `:49374` server, the field is not the only act
  (the route/baseURL review comes first). Recommend Nox announces the field only with the endpoint decided.

### (d) First real order payload — staged text, NOT driven

Visible surfaces (pre-registered Stage 5 direction, design-pack + unslop fences): the plugin's own sidebar section +
phone-VM panel + Windows-VM panel. Exact `order.stage_real` arguments drafted for the moment a real lane exists:

```json
{"title":"Stage 4 first real order — fleet visible surfaces",
 "goal":"Build the fleet plugin's own sidebar section (fleet state: lanes, meters, orders) and the phone-VM + Windows-VM sidebar panels from the Lane-4 rig, inside the approved design-pack tokens and the unslop fences; rehearsal mode, no live hands.",
 "done_when":"all three surfaces exist in the plugin client bundle, render from fleet state with no fixture data shown as real, panel transports are loopback-only per the rig contract, and the design-pack/unslop checks pass",
 "example":"sidebar shows 5 lanes with meters; phone panel drives the AVD round-trip; Windows panel is a monitor-level view",
 "est_minutes":90,"budget_usd":0.5,
 "context":["team-b-lane4-rig-status.md","team-b-lane4-mcp-facade-contract.md","team-b-scope-items-20261101.md"],
 "acceptance":["sidebar section","phone-VM panel","Windows-VM panel"]}
```
- `est_minutes` respects the ≥10 non-errand floor; budget is a placeholder envelope (no spend is authorised for this
  order until Nox/Lead set one).
- **Live staging NOT executed** (fence: no fleet-profile write while ownership is pinned). The tool's own refusal path
  would write `payload_sha256`-named payload under `/var/lib/fleet/state/mcp/staged/` and answer
  `lanes-unavailable` (S5-installed T5, evidence `checker-verify/s5-installed/...`, `real_lane: not-built`). No order
  or task ids can exist before a real lane claims the order — that is a property of the board, not a missing answer:
  `order.stage_real` creates the order only when `fleetRoster` is mounted (`src/mcp/board.js:411-441`).

### (e) Done-when proof draft (to attach to the roster order)

1. Roster installed: `order.stage_real` no longer refuses; returns `{mode:"real-lane", real_lane:"fleetRoster mounted"}`
   and a real order id; the order is **claimed and finished by a real lane** and read back `done` from
   `/run/fleet/control.sock` (board read-back, not the report).
2. Real-provider turn: MCP `session.create {mode:"model", preset:"lead"}` → session id (today refused
   `model-credential-not-entered`); `session.prompt` → `status:"completed"`, **non-empty assistant_text**,
   `provider_calls > 0`, and the result's model/transport field names the **real route** (e.g.
   `opencode-go/deepseek-v4.1-flash`) — never `deterministic-stub (fleet-selftest-stub)` and never `stub-turn`.
   The session record must label the transport honestly (the facade already has two labelled transports; a real route
   takes the `agent` path).
3. Evidence to preserve: MCP transcript (request/result), the session record, the order/task ids + final states, and
   `actions.status` chain intact. Fresh checker runs it (maker never checker).

**Roster ownership:** prep only; the strike seat owns the order until Nox rules. Handover = this section.

---

## B1. OC ATTESTATION (Nox 201800Z) — DELIVERED TEXT (URL only, no secrets)

**ATTESTATION (exact text to file):**

> OpenCode provider endpoint in the fleet provider config: **NONE — the fleet provider config contains no OpenCode
> provider endpoint at all.** The installed fleet profile mounts the `@deepseek-ai/dsh-llm-pi-ai` adapter row
> (`id: llm-pi-ai`) with an empty `providers` config, so no `opencode`/`opencode-go` route (and no other pi-ai route)
> is declared; the only model default is `agent-default-model = deepseek-official / deepseek-flash`. In the DSH
> provider catalog, the `opencode` and `opencode-go` entries declare only the credential env name
> (`OPENCODE_API_KEY`) and **no default base URL** — there is no baked cloud endpoint to fall back to.
> Therefore: **NOT the cloud Go endpoint (none is configured) and NOT the local `:49374` instance (the fleet config
> never references it).** The only OpenCode endpoint on this machine is an owner-run local OpenCode server listening
> on `127.0.0.1:49374` (owner uid 1000), which belongs to the owner's own desktop OpenCode CLI config
> (`~/.config/opencode/opencode.jsonc` — model `opencode-go/deepseek-v4.1-flash`, local llama.cpp provider block),
> not to the fleet build. Consequence for the OC timer migration: nothing in the fleet's current provider config
> points at either the cloud Go endpoint or the local instance, so the migration is fenced on the endpoint decision,
> not on a route that exists today.

**Config paths read (no secrets printed; values never quoted):**
1. In-host composed fleet profile config: `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin dump-config` (NOPASSWD
   read-only verb) → `board/probes/team-b-batch7/dump-config-20261101T2340Z.txt`, rc=0, 56,360 bytes,
   mtime 2026-11-01T23:39:08Z. The verb composes `/var/lib/fleet/dsh/profiles/fleet/cordis.patch.yml` + bundle
   patches; the file itself is fleet-owned 0750, unreadable from uid 1000.
2. Shipped template: `~/workspace/harness-build/strike-release/package/cordis.patch.yml` (no `llm`/provider patch);
   `package/src/firstrun/capabilities.mjs:104-116` states the empty-providers fact; `package/src/firstrun/
   credentials.mjs:66-71` declares the field `OPENCODE_GO_API_KEY` for provider `opencode-go`;
   `package/rows.manifest.json` (7 pinned / 12 disabled rows — no provider row).
3. DSH provider catalog: `/home/cayleb/.local/opt/deepseek-harness-official/packages/llm/llm-pi-ai/src/
   provider-data.generated.ts:35-36` → `'opencode': { env: ['OPENCODE_API_KEY'] }, 'opencode-go': { env:
   ['OPENCODE_API_KEY'] }` — **no `defaultBaseURL`** (contrast: anthropic/openai/deepseek all carry one).
4. Endpoint present on the box (not fleet config): `ss -ltnp` → `127.0.0.1:49374 LISTEN users:(("opencode",pid=18797))`
   at 23:41Z; design doc `~/workspace/harness-build/design/fleet-plugin-design-v1.md:30` records the same local
   instance as host fact. Owner's OpenCode config path only: `~/.config/opencode/opencode.jsonc` (values not printed).

**Consequence named:** Nox's 201800Z question ("cloud Go endpoint vs local :49374") has a third true answer — the
fleet config names neither. The gate for OC timer migration needs the endpoint **decided and declared**, and that is
a Nox/owner ruling, not a config reading.

---

## B2. NORTH-STAR 4-PROPERTY MEASUREMENT (Nox 200600Z) — honest, gaps named

Measured against the INSTALLED 0.3.0 build (live service, MainPID 518634, InvocationID bc98f82d…; control status
`ok:true, integrity.ok:true, lanes_allowed:true, policy.selftest:true` read 23:49:00Z).

| # | Property | Verdict | Evidence / named gap |
|---|---|---|---|
| 1 | **Shared sessions first-class** — an agent can join a session the owner is actively working in, with a care fence and no disruption | **NOT MET** | The MCP facade has exactly three session tools: `session.create`, `session.prompt`, `session.status` (`package/src/mcp/tools.js:104-135`); there is no join/attach/adopt op, and every facade session is created by the facade itself (its own `sessionRoot` under `/var/lib/fleet/work/mcp-sessions`, `src/mcp/session.js:78-92`). No care-fence/no-disruption mechanism exists in the build (no presence/yield row). Sessions are separate: owner's DSH session ≠ fleet session ≠ lane session. Gap named, not smoothed. |
| 2 | **Working set on one bus/session model** — Claude Duet + Nox + DSH harness + lanes on one bus/session model | **NOT MET** | Installed 0.3.0 exposes only the nine MCP board/session tools; the fleet-bus (Stage 5) is built but **not installed** (branch `bus` @ `e8c8a4c`, 67 tests, review found 2 high/4 medium/2 low; based on old `8095fbc`). Duet digest (`GET /v1/digest`, Stage 6) is not built; Duet remains pull-only email through Cayleb. Nox↔fleet bus 5.1 end-to-end is **not proven** (BUILD-STATE comms proof map: "None of these links is proven yet"). One token/principal/route matrix for participants does not exist yet. |
| 3 | **Full computer use for Nox** — own sessions via MCP, real work orders, complete remote desktop | **NOT MET** | Hands module has **no implementation in any branch** — only the systemd unit file `ops/systemd/fleet-hands.service` (1610/1428 bytes in `strike-release/ops/systemd/` and `dsh-fleet/ops/systemd/`; no source, no refusals-by-construction, no action log, no idle-yield). MCP now gives Nox real work orders/board ops (T4 roundtrips proven on installed host) — that sub-part is MET; full remote desktop is absent. |
| 4 | **Nothing clock-gated** | **MET at the build level; one caveat named** | The fleet build has no acceptance leg that waits on a clock: no fleet systemd timers exist (`systemctl list-timers --all` at 23:42Z lists only host timers: snapper-cleanup, shadow, plocate-updatedb, man-db, systemd-tmpfiles-clean, archlinux-keyring — **no fleet-\***). The DSH `timer` plugin is mounted (`dump-config` line 8) and used for retry/backoff and the planned housekeeper hourly snapshot; the MCP rate window is a timer but does not gate work. Caveat: the *programme* (not the build) has clock bookings (Mon 01:00 ET meter reset, market hours) and Nox's dead-man alert is a planned every-10-min timer at Stage 8; neither gates the fleet's own acceptance. |

**Summary for the Lead's filing:** 3 of 4 properties are NOT MET on the installed build; nothing is smoothed. The
single-plugin/finish-line work that DID pass (Stage 2 60/60, Lane 1, S5 board ops) is real but does not reach the
north star. The nearest true statements: "the fleet has a working board/MCP order surface and a proven containment
gate; it has no shared-session model, no bus, and no hands."

---

## B3. INSTALLED-HOST STUB-TURN DEVIATION (residual 1) — read-only diagnosis + proposed fix path (not applied)

**What happens (reproduced live, 2026-11-01T23:45:30Z):**
- `dsh-fleet op stub-turn '{"agent":"builder:selftest-batch7","model":"claude-haiku-5-5","text":"…","queue":[]}'`
  (control socket `/run/fleet/control.sock`, uid 1000, CLI = release package `bin/dsh-fleet.mjs`) →
  `{"ok":true,"result":{"status":"completed","reason":null,"assistant_text":"","inbox":[],"provider_calls":0}}`
  (file `stubturn-installed-20261101T2356Z.json`). Same shape the S5 checker saw.

**Discriminating probe — the LLM stream is never entered:** the same op with a model that is NOT in the policy rates
table (`"not-a-real-model"`, 23:47:08Z) returns the **identical** empty result (`status completed, reason null,
provider_calls 0`) instead of the expected named gate refusal `unknown-model`. The ledger gate refuses invalid models
*before* any provider call, so if the agent turn had reached `llm/stream` at all we would see `reason:"unknown-model"`.
It does not. Conclusion: on the installed host the agent created by `stub-turn` performs **no model call at all** —
this is not the stub adapter, not the gate, and not the assistant-text capture.

**Config comparison — installed profile vs shipped template: no agent/llm row difference.**
- Installed composed config: `dump-config-20261101T2340Z.txt` (23:39:08Z), 212 rows. Shipped-template scratch compose:
  `~/workspace/harness-build/evidence/strike/checker-verify/s5-rerun2/03-composed-profile.yml`, 214 rows.
- Row-set diff (script in report folder): **identical except two client-UI rows** (`ui-context-menu`,
  `ui-file-actions` — client only, cannot affect a server-side agent turn). `disabled:` flags identical.
- Specifically identical: `agent-default-model` = `deepseek-official/deepseek-flash`; `llm-pi-ai` bare in both;
  `agent-preset-registry.default = fleet-lead`; `subagent` row default config; all seven pinned fleet rows
  (`agent-preset-registry`, `permission`, 4 presets, fleet-core/orders/... ) match.
- Control status 23:49:00Z: `integrity.ok:true` (rows pinned 7 / disabled 12, failures[], manifest sha
  `1bcac986…`), `lanes_allowed:true`, `policy.selftest:true` (source default).

**Preset mounting is NOT the cause (positive control):** `dsh-fleet op selftest '{"role":"builder","tool":"bash",
"args":{"command":"echo probe"}}'` at 23:49:39Z created a real preset agent
(`session: fleet-selftest-builder-e81ccfa9-…`) and reached the real tool pipeline — the tool answered a proper
schema refusal `bad-args: invalid arguments: missing required property "description"`. So fleet presets mount and the
agent/tool path works on the installed host; only the `stub-turn`-created agent's model turn is empty.

**The real delta found — the "scratch boot of the same bytes" was not the same host runtime.** The accepted S5
scratch comparison (`s5-rerun2`, transport `stub-turn`, provider_calls 1, 730-char `S3STUB:` text) booted the plugin
with **`DSH=/home/cayleb/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js`** (the repo build, uid 1000,
fresh `DSH_HOME` in `/tmp`, plus a path overlay) — `run-s5-rerun2.sh:20,72-88`. The installed service runs
**`/var/lib/fleet/npm/bin/dsh`**, installed by `dsh-fleet-admin install-dsh 0.2.2-alpha.2` via
`npm install -g @deepseek-ai/dsh@0.2.2-alpha.2` (wrapper source lines 128-136), as uid 950 with
`HOME=/var/lib/fleet`, `DSH_HOME=/var/lib/fleet/dsh`. Both report the same version string (`0.2.2-alpha.2`) but they
are **different builds (npm release vs local repo build)** — the version string is not a build identity. The S5
verdict's own note ("installed-profile/runtime condition, not a regression") is consistent: the 0.2.2-era installed
host already showed the same empty result, and this artifact never caused it.

**Named conclusion (what is proven vs inferred):** PROVEN — config rows are identical; the turn never enters
`llm/stream`; presets/tools work; the recorded passing comparison used a different DSH host build. INFERRED, not yet
proven — the pinned npm build of DSH 0.2.2-alpha.2 is the likely locus (agent model-turn wiring for a
programmatically created agent with no preset). One decisive test remains and it is cheap + safe.

**Proposed fix path (do not apply yet; needs Lead go because it starts a second host):**
1. **Decisive reproduction (read-only, no service touch):** scratch-boot the same 0.3.0 tarball with
   `DSH=$(npm-root of installed @deepseek-ai/dsh@0.2.2-alpha.2)` (extract the published package read-only into a
   temp dir; it is the same package the wrapper installed) and the same overlay — if `stub-turn` is empty there and
   non-empty with the repo build, host-build delta is proven.
2. **Fix in the plugin (next release, additive):** make `stubTurn` self-diagnosing rather than silently empty —
   create the agent with an explicit preset and assert the created session id/agent identity; if the turn finishes
   with no `llm/stream` event, return a **named refusal** (e.g. `stub-turn-not-reached`) instead of
   `{status:"completed", assistant_text:""}`. Also pin/record the DSH host build identity in the first-run report so
   "same bytes" claims can never again compare across different host builds.
3. **Not a 0.3.0 release blocker** — S5's ACCEPT stands under the Lead's OR criterion (named gate reason present);
   this is a truthfulness/diagnosability fix for the next release, and it should be checkable by the fresh checker.

---

## B4. SUB-AGENT CEILING ON-BOX CHECK (Nox 141900Z) — lever located, package answer: NOT in shipped 0.3.0

Lead's source facts verified in the pinned checkout (all read at ~23:50Z):
- `packages/subagent/subagent/src/index.ts:186-198`: `interface Config { maxActiveSubagents: Volatile<number> /* default 8 */; maxDepth: Volatile<number> /* default 1 */ }`; schema `.default(8)` / `.default(1)`, both `.volatile()`; ranges `maxActiveSubagents` 1…MAX_SAFE_INTEGER, `maxDepth` 0…MAX_SAFE_INTEGER.
- Enforcement `packages/subagent/subagent/src/activation.ts:24-29`: `ActivationPool.reserve(capacity)` throws `SubagentError('subagent limit reached (active child limit: N)…', 'ACTIVATION_LIMIT_REACHED')`; consumed at `manager.ts:744` `pool.reserve(this.maxActiveSubagents())`.
- The user-facing lever exists: `packages/client/ui-settings-subagent/src/client/subagent-limits-card-controller.ts:48` builds the settings form with fields `maxDepth`, `maxActiveSubagents` (a "subagent limits" card); `docs/config-catalog.md:3566` documents the field.

**(1) Where the settings document is persisted for the installed fleet profile.**
The settings service edits **Cordis profile patches**: `PROFILE_PATCH_FILENAME = 'cordis.patch.yml'`
(`packages/boot/app-boot/src/profile.ts:42`), and `packages/settings/settings/src/index.ts` is explicitly
"Config-schema projection and form edits over Cordis profile patches" (it writes the row's user section through the
profile patch; secrets are redacted on read and path-addressed on write). For the installed fleet service
(`Environment=DSH_HOME=/var/lib/fleet/dsh`, `--profile fleet`, user `fleet`) that file is
**`/var/lib/fleet/dsh/profiles/fleet/cordis.patch.yml`** — fleet-owned 0750, not readable by uid 1000; the composed
result is readable only via `dsh-fleet-admin dump-config`.
Relevant row: `- id: subagent / name: '@deepseek-ai/dsh-subagent'` with **no `config`** in the installed profile
(dump-config lines 289-290) ⇒ defaults 8 / 1. The live lever is present in the profile: `settings` row (dump 66),
`config-editor` (dump 63), `settings-controller` (dump 481-482), `ui-settings-subagent` (dump 642-643), and the fleet
web UI on `127.0.0.1:3091` (unit ExecStart) — so the owner/Lead can set `maxActiveSubagents` **live (volatile, no
restart)** through that card; it persists into the profile patch. **This is a profile write: not done by this seat;
no write without Lead go.**

**(2) How a package / guided first run can set a fleet default — and whether 0.3.0 can ship it.**
- Mechanism: add the field to the **bundle patch** `package/cordis.patch.yml` as a row patch
  (`- id: subagent / config: { maxActiveSubagents: 32, maxDepth: 2 }`), which is what `init-profile` composes into a
  new profile; the guided first run (`src/firstrun/*`) has **no code path that writes row config** — `--apply` only
  performs fleet-profile **removals** of REMOVE-classified items (index.mjs modes; apply.mjs), so first-run cannot be
  the vehicle.
- **0.3.0 today: NO package-side lever.** It ships no `subagent` row patch and cannot raise the cap by config.
  Worse, the route is integrity-pinned: fleet-core verifies the composed rows against `rows.manifest.json`
  (**pinned 7 / disabled 12**, manifest sha `1bcac986…` ≡ shipped manifest — status read 23:49:00Z; S5 evidence).
  Adding a subagent row patch to the profile without the manifest would make the rows check fail → `lanes_allowed`
  false; and a package that ships it must **regenerate `rows.manifest.json`** (`package/bin/rows-manifest.mjs` exists
  for exactly this) so the new pinned row + config hash are in the manifest, then ship a new version
  (`dsh-fleet-admin plugin-add <sha256> <tgz>`), fresh checker, owner install act.
- **Named gap (no silent workaround):** the shipped 0.3.0 cannot raise or remove the >8 cap from the package. The
  only live lever is a profile write (web UI settings card / config editor, owner- or Lead-authorized) — a profile
  change, not a package change. A "ship 32 by default" requires a 0.3.1 build with the row patch + regenerated
  manifest + checker + install act.
- Operating rule recorded per Nox: the raised ceiling is **headroom, not working shape** — sub-lead tiers
  (span 3-5) remain the design; the cap raise is for a Lead with a legitimately wider fan-out, not for flat 12-wide
  spawn storms.

---

## B5. STAGE READINESS PLAN (Jobs 06-11) — what can start NOW vs blocked

Status baseline: Stage 2 gate CLOSED (installed 0.3.0 S4B ACCEPT 60/60 + S4A 63/63); finish line CLOSED (S5-installed
ACCEPT + Nox live leg; Ergon S8-030 ACCEPT 113/113). Staged 3/4/5/8/6-7/4b remain unbuilt/uninstalled.

| Stage / Job | Can start NOW (no spend, no owner act) | Blocked on | Next executable step + done-when |
|---|---|---|---|
| **Stage 3 red-team (Job 07 / Mission 2)** | YES — package v1 assembled (`evidence/stage-3/redteam-package/INDEX.md`: ten attacks, grant legs, guard ordering, ledger legs); finish the reviewer-script fixes and the dry-run of the ten attacks on scratch; author the single mechanical sequence for Cayleb's tty run | Real-install run: Lane-1-style window + **Cayleb's tty run** (`S3_CONFIRM=tty`, ~10 min, ≤9 polkit prompts); **Nox 3.1 verdict** | Run the ten attacks against the installed 0.3.0 in a read-only mode where possible + full rehearsal on scratch → done-when: package index lists each attack with command, refusal and no-exec evidence against current bytes, reviewer findings closed, and the tty sequence is one copy-paste block |
| **Stage 4 roster (Job 07 / Mission 3)** | **Prep DONE (B0)** — design, lever list, credential text, payload, proof draft | **Ownership ruling (Nox)**; then: roster fix round (F1/F2/F3/F4) on `bus`, rebase onto 0.3.0, provider route config + rate row, 0.3.1 build, manifest regen, fresh checker, owner credential act; spend caps $5/day + $0.50/drill once reached | Fix F1 (tree.js write-before-verify) + F2/F3 with regression tests on a 0.3.0-based worktree → done-when: roster suite green on the new base + mutants killed, no install |
| **Stage 5 bus/steering/Grok (Job 08)** | YES — `src/bus` exists (branch `bus`, 67 tests) and needs rebasing/wiring onto merged main; write per-participant feasibility matrix; steering slot/meter design already banked (`team-b-scope-items`) | Install/demo: service change (needs Lead/Nox); Grok lanes: **owner account acts** (handle/sign-in, API key route — Grok gap named); Cayleb tailnet tag+ACL+token | Rebase `bus` onto 0.3.0 `main` in an isolated worktree and run its suite → done-when: 67 tests green on the new base and the route matrix lists every participant with verified/unverified + fallback |
| **Stage 8 hands + Stage 9 (Job 09, TIER S)** | YES — implementation can be designed/built in a worktree: token auth, two-principal allowlist, ping/status, command/file work, **refusals by construction** (wallet/key stores/money CLIs/secret files), hash-chained action log, idle-yield, unit file already exists | **Cayleb acts**: lodge hands token, bind kill key, **physical kill test (8.4)** — his hand only; Nox Tier-2 runs 8.1/8.2/8.5/8.6/8.7; checker with veto before irreversible steps | Implement hands module + refusal test suite in a throwaway worktree → done-when: every hard refusal has a passing negative test and the kill-path code is exercised in scratch (never on the live unit), checker pre-signoff on the plan |
| **Stage 6-7 digest/phone (Job 11)** | YES — `GET /v1/digest` server-side (status-only: ids/states/counts/holds, planted content+money absent) and `stage7-check.sh` authoring are spend-free; fleet-phone builds on the vendored Telegram base | **Cayleb 6.1 probe** (phone, ~2 min); **Telegram bot token** (BotFather, secure path); duplicate-listener decision (6.2, optional) | Build the digest handler + tests in a worktree → done-when: digest returns only status fields with a planted-content/money-absence test passing; then stage7-check authored (checker runs it) |
| **Stage 4b engines (Job 10)** | NO — depends on 08; provider build needs the doctrine route decided; Claude-engine provider remains a design item | **Job 08 merged**; **provider key + stated envelope** (cap unstated ⇒ zero under hard holds); Codex decision still owed (OpenAI key vs ChatGPT login OK) | (after 08) provider route matrix build; nothing spend-bearing until key + envelope |

**Next executable step named for the Lead (Team B, this batch):** the **Stage 5 bus rebase + suite run on a
0.3.0-based throwaway worktree** — it is spend-free, owner-free, unblocks Jobs 08/09/10/11, and does not touch the
installed service. Done-when: `package/src/bus` tests (67) green against the merged 0.3.0 base in the worktree, no
install, no service change, evidence file with command+output. Successor: per-participant feasibility matrix (Grok
first). Immediately before that, if Nox rules roster ownership to us, the **roster F1 fix round** takes priority
(P0 order).

---

## Fences respected / disclosures

- $0 spend. No credential values read, printed, copied or stored — the only secret-shaped artefact is the fleet MCP
  bearer token path, which was **not** read this batch (no MCP token call); control socket reads used uid 1000 only.
- No fleet service change/restart; no profile/config write. Read-only NOPASSWD wrapper verbs used:
  `dsh-fleet-admin dump-config`, `status`, `version`.
- Control ops used (uid 1000, peer-uid allow + selftest on): `status`, `policy-get`, `selftest` (one tool-schema
  refusal, ephemeral agent), `stub-turn` twice (ephemeral in-process session created and disposed; no board rows, no
  policy/DB change). Nothing was driven; no order staged or fired.
- No image reads/screenshots; no host input injection. Roster prep adds nothing to the fleet; ownership remains
  Nox/strike until the Lead relays the ruling.
