# TEAM B — ROSTER ACTIVATION PACKAGE (post-install, owner's one act) + FIRST REAL ORDER STAGING + SUB-AGENT CEILING

For Nox night order 20261102T010900Z. Based on the 0.3.1 candidate r2 shipped bytes (`5ede7484…`, 123 members,
tarball tree `candidate/0.3.1` @ `741db66`) and the installed 0.3.0 host. No install, no service/profile touch, no
credential values, no image reads. Everything below is prepared; the owner's hand is only where marked.

## 1. ROSTER ACTIVATION PACKAGE

### 1a. The guided-first-run path that reaches the credential field (installed 0.3.1)

The credential CLI ships in the tarball (`package/bin/dsh-fleet.mjs` → `src/firstrun/`, 13 files; host-side op
`first-run-credential-set` in `src/firstrun/plugin.mjs`). The installed tree is under `/var/lib/fleet` (0750
fleet-owned, unreadable from uid 1000), so the owner runs the **CLI from a readable release copy** and the *installed
host* stores the value over its own control socket. Two owner paths, both "his one act":

**Path A (recommended — no root, no fleet shell):**
```bash
# as cayleb (uid 1000), in a terminal. DSH_FLEET_SOCKET defaults to /run/fleet/control.sock.
node /home/cayleb/workspace/harness-build/candidate-031/package/bin/dsh-fleet.mjs first-run --credentials
```
(any readable release/candidate tree works; the value is stored by the fleet host, not by this process). What he sees:
the field list, then exactly `OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:`, then `accepted (not echoed)`, then
the summary `status: written` / `stored: OPENCODE_GO_API_KEY via control-op`. Exit 0. A non-TTY run prints the owner
acts and `status: credential-entry-required` (exit 0), storing nothing. Ctrl-C cancels the whole flow (nothing stored).
If the control op were unusable it falls back to fleet-store write (refused from uid 1000) then a 0600 staged file +
root placement instruction — that path is **not** expected on the live host and would mean the op is down.

**Path B (native DSH, owner clicks):** fleet web UI → **Models** at `http://127.0.0.1:3091` (the fleet unit's loopback
web app) → add the same provider field. Native path: `<DSH_HOME>/.credentials.yaml` (`/var/lib/fleet/dsh/`), refs
section, mode 0600, written by the fleet process.

**"Ready for his single act" looks like (pre-key readiness):** post-install `dsh-fleet-admin status` → 0.3.1,
integrity ok/lanes allowed; `dsh-fleet first-run --status` → version 0.3.1 + **123 files**; `dump-config` shows
`fleet-roster` row, `llm-pi-ai` `providers.opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY }` with **no value**,
`subagent.maxActiveSubagents: 32`; nine MCP tools answering; stub session turn still labelled `stub-turn`.

### 1b. Exact prompt/field text and write-only property — re-confirmed in the shipped bytes

- Prompt (verbatim template, `src/firstrun/credentials.mjs`): `` `${item.field} (${item.provider}${unofficial ? ', UNOFFICIAL' : ''}) [Enter to skip]: ` ``.
  For the doctrine route the owner sees: **`OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:`** (official → no
  UNOFFICIAL suffix; cursor is only prompted with `--include-unofficial`). The 0.3.1 tarball's `credentials.mjs` is
  sha256 `f5a634666c790fe91219a5caaf6f9cb04e63874d51bc5c39e576dd4237efa06f`.
- Echo-off: `createTTYSecretReader` sets raw mode **before** printing the prompt and never echoes the value (no stars
  either); Enter skips; Ctrl-C cancels and drops the partial value.
- Write-only: over the control op the host calls `ctx.credentials.set(ref, value)` and the reply never carries the
  value; the host event log records provider + field + result only; the CLI summary prints field/provider/via only;
  `redactor` scrubs any accidental output. Store: `<DSH_HOME>/.credentials.yaml` (refs section, mode 0600, fleet-owned).

### 1c. Activation checklist — after the key is entered (honest, incl. one named blocker)

1. **Key stored:** `dsh-fleet op providers-get` (uid 1000, control socket) → the `opencode-go` route entry must show
   its credential `present: true` / `configured: true` (value never shown), models include `deepseek-v4.1-flash`.
2. **Roster table live:** `dsh-fleet op policy-get '{"key":"roster.models"}'` → doctrine table (lead sonnet-5-5,
   builder/researcher opencode-go/deepseek-v4.1-flash, checker haiku-5-5); `dsh-fleet op roster-get` → cap 5.
3. **Prove the real provider route (what is actually possible in shipped 0.3.1):**
   `dsh-fleet op lane-spawn '{"role":"builder","domain":"activation","start":true}'` → fleet-roster creates
   `builder:1` with the roster-table route and briefs it: **one REAL `llm/stream` call through the `opencode-go`
   route** (llm-pi-ai), a real `ledger.record` event (non-synthetic, priced from the new `rates` row), and a lane row.
   Evidence: `lane-get` (lane state/provider/model), `ledger-get` for `builder:1` (a real event with
   provider `opencode-go`, model `deepseek-v4.1-flash`, priced cost), `providers-get` (credential present).
   **If the provider fails** (bad key/quota/network): the lane errors (`agent/error`/lane state error or stopped), the
   ledger records nothing for that turn, and the honest report is the provider error — **never** cite the model-free
   `stub-turn` output as a real turn. `transport: "stub-turn"` / model `deterministic-stub` is the labelled stub path.
4. **NAMED BLOCKER for the Lead/Nox:** the shipped MCP facade's `session.create {mode:"model"}` is an **unconditional**
   `model-credential-not-entered` throw (`src/mcp/session.js:245`, same bytes in 0.3.0/0.3.1) — entering the key does
   **not** change it, so Nox's "a session turn through the MCP facade on a real provider route" **cannot be met by the
   shipped facade**. Either (a) strike patches facade model-mode in the release line (create the agent with an explicit
   provider/model — e.g. the roster table's working engine — and label the transport `agent`), or (b) the done-when is
   amended to the roster-lane real turn in step 3. Do not announce a facade real-route proof before one of these.
5. Also note: the default agent model stays `deepseek-official/deepseek-flash`; a fleet web-UI chat uses that route
   (needs `DEEPSEEK_API_KEY`/account auth), so it is **not** evidence for the OpenCode Go route.

### 1d. Corrected door doc

`board/reports/team-b-nox-door-corrected.md` — ready to install at `~/.dsh/strike-door/NOX-DOOR.md` by the Lead. It
updates step 2 (stub labelled), step 3 (`order.stage_real` **creates** a real `fixture:false` order on a roster host;
the roundtrip stays fixture-only), adds the real-route note + the facade blocker above, and keeps the token discipline
(no token value in the file).

## 2. FIRST REAL ORDER — STAGING PACKAGE (roster-mounted behaviour)

`order.stage_real` on the installed 0.3.1 (roster mounted) **creates** the order (`fixture:false`) instead of
refusing `lanes-unavailable`. Payload (tool schema keys only: `title, goal, done_when, example, est_minutes,
budget_usd, checkpoint_at`; required: title/goal/done_when/example/est_minutes):

```json
{"title":"Stage 4 first real order — fleet visible surfaces",
 "goal":"Build the fleet plugin's own sidebar section (fleet state: lanes, meters, orders) and the phone-VM + Windows-VM sidebar panels from the Lane-4 rig, inside the approved design-pack tokens and the unslop fences; rehearsal mode, no live hands, no spend outside the existing OpenCode Go plan.",
 "done_when":{"mode":"rubric","criteria":[
   "the plugin client bundle renders the fleet sidebar section from live fleet state (no fixture data shown as real)",
   "the phone-VM panel drives the Android AVD round-trip and the Windows-VM panel is a monitor-level view, both loopback-only per the rig contract",
   "design-pack tokens are used (no hand-typed hex), the unslop fences hold, and the client contract rules from C4 are followed (own main[key], sidebar.panellist entry)"],
  "text":"three visible surfaces, real state, contract-clean"},
 "example":"sidebar shows 5 lanes with meters; phone panel completes one AVD text round-trip; Windows panel shows the VM console; no live hands.",
 "est_minutes":90,
 "budget_usd":0.5,
 "checkpoint_at":"<now + 6h ISO>"}
```

Ops, the moment activation is green (one block, so no relay is needed):
1. `node ~/.dsh/strike-door/mcp-probe.mjs --url http://100.114.17.67:8789/mcp --token-from-admin-socket /run/fleet/mcp-admin.sock --call '{"name":"order.stage_real","arguments":<payload>}'`
   (or the MCP client's `order.stage_real`). Expected result: `{ok:true, mode:"real-lane", real_lane:"fleetRoster mounted",
   order:{id:"o<seq>", state:"open"}, stage_path, payload_sha256}`.
2. Read back: `board.op {"op":"order-get","id":"o<seq>"}` → `fixture:false`, `tasks: []` (no task ids exist until a
   Lead decomposes it — that is a board property, not a missing answer).
3. Expected ids: **one real order id `o<seq>`** (next board sequence; returned by the call, cannot be predicted), then
   after the Lead's decompose `t<seq>…` task ids incl. the harness's final task. The order's first claim comes from a
   real `lead` lane (Lead preset), builders implement, one harness-made Checker per task round (fresh by construction,
   `run_check` + verdict gate); the large-order path adds three goldfish reviewers. Maker never checker; no owner act
   in the loop after activation.
Do not drive it before activation: `order.stage_real` without a roster still stages+refuses, and the roster route
without a key refuses at the provider.

## 3. SUB-AGENT CEILING — shipped 0.3.1 confirmation

- **In the packaged bytes:** `package/cordis.patch.yml` in the 0.3.1 tarball contains
  `- id: subagent` / `config:` / `maxActiveSubagents: 32` (lines 240-242 of the shipped patch; `subagent` is a patched
  **base** row, so it is not in `rows.manifest.json`'s pinned list — the manifest stays 8 pinned/12 disabled and the
  rows integrity check is unaffected).
- **How it mounts:** the bundle patch is composed at boot by the profile Loader (the same mechanism that mounted
  `fleet-mcp` from `mcp.row.yml` in 0.3.0); after the 0.3.1 install + restart the installed host's
  `dsh-fleet-admin dump-config` must show `- id: subagent` with `config: {maxActiveSubagents: 32}`. **No profile write
  is needed for the package default.**
- **If dump-config shows 8 instead:** a profile-level user section is overriding the bundle patch. Then the exact
  profile write is: fleet web UI → Settings → **subagent limits** card (the fleet profile has `ui-settings-subagent` +
  `settings-controller` mounted) set `maxActiveSubagents = 32` — volatile, applies live, persists into
  `/var/lib/fleet/dsh/profiles/fleet/cordis.patch.yml`; or the native editor's `subagent` section. That is a
  profile write (Lead/owner), not a package change. Add to activation checklist step 0: read the dump-config line.
- Operating rule (Nox 141900Z): 32 is **headroom, not working shape**; sub-lead tiers (span 3-5) remain the design.
