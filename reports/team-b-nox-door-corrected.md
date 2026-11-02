# NOX DOOR — fleet MCP surface (item 2) — CORRECTED FOR 0.3.1 (install at `~/.dsh/strike-door/NOX-DOOR.md`)

Status: OPEN on 0.3.0; **the 0.3.1 release (roster-mounted) changes step 3 and the real-route story below**. The facade
itself is unchanged on the wire; what changed is what the board will do when the Stage-4 roster is mounted. This file
carries no token value and must never carry one.

## What you get
- MCP over Streamable HTTP (protocol 2025-03-26 / 2024-11-05), tailnet-only listener on **100.114.17.67:8789**, path `/mcp`.
- Bearer-token auth. The token is generated on the fleet host at first boot, fleet-owned 0600, never written to the board,
  a log, a report or this file.
- Nine tools: `fleet.status`, `session.create`, `session.prompt`, `session.status`, `board.op`, `order.drive_roundtrip`,
  `order.stage_real`, `fleet.selftest`, `actions.status`.
- Every call is appended to a hash-chained action log before its effect; refusals are logged with principal null. The log
  contains tool names, arg NAMES and arg digests only — never values, never the token.

## Get the token (one local command, on this desktop, as the owner's user)
    node /home/cayleb/.dsh/strike-door/token-fetch.mjs
It reads the local admin socket (uid 1000 only) and prints the token to the terminal. Paste it into your MCP client config, or
hand it to your relay; do not redirect it into any file/chat/log.

## Client config shape (Claude Desktop / any Streamable-HTTP MCP client)
    { "mcpServers": { "dsh-fleet": { "type": "http", "url": "http://100.114.17.67:8789/mcp",
      "headers": { "Authorization": "Bearer <token>" } } } }

## The three-step live proof — 0.3.1 wording
1. `fleet.status` — read integrity/lanes/board head and the MCP counters.
2. `session.create` with `{"prompt":"state your role in one line"}` — a REAL DSH agent session on the fleet's
   **deterministic stub route** (labelled `model: deterministic-stub`). This is model-free on purpose and its transport is
   named (`agent` only when a real model route runs the turn; `stub-turn` on this build). Then `session.status` with its id.
   **Do not present this turn as a model/provider proof.**
3. `order.drive_roundtrip` with `{"title":"Nox live proof"}` — a REAL order lifecycle through the fleet rails: create
   (fixture) → claim → preflight → decompose → claim → evidence → built → check → verdict → bank → report → done. Labelled
   `mode:"fixture-validated"`; with the 0.3.1 roster mounted its `real_lane` label is `"fleetRoster mounted"` (before the
   roster it read `"not-built (Stage 4 roster)"`); the lifecycle itself is unchanged. Independently read it back with
   `board.op {"op":"order-get","id":"oN"}`.
4. **`order.stage_real` with the payload in this directory (0.3.1 CHANGE):** on a roster-mounted host this now **CREATES a
   real, non-fixture order** and answers `{ok:true, mode:"real-lane", real_lane:"fleetRoster mounted",
   order:{id:"oN", state:"open"}, stage_path, payload_sha256}`. It is **not executed/driven** by the call; tasks appear only
   when a real Lead lane decomposes it. On a 0.3.0 host it still refuses `lanes-unavailable` (payload staged). The door's
   step 3 therefore creates a real order on the live 0.3.1 host — expect it, and read it back with `order-get`.

## Real-provider story after the owner's key (do not overclaim)
Entering the owner's `OPENCODE_GO_API_KEY` (guided first-run, field text `OPENCODE_GO_API_KEY (opencode-go) [Enter to
skip]:`, write-only) makes the roster's real route usable by **lanes**. Proof path: `fleet-roster` `lane-spawn` with
`{"role":"builder","domain":"activation","start":true}` → a real `llm/stream` call through the `opencode-go` route; evidence
is the lane row + the fleet ledger's real (non-synthetic) event + `providers-get` credential-present. **Known limit in the
shipped 0.3.1 facade:** `session.create {mode:"model"}` is still an unconditional `model-credential-not-entered` refusal
(`src/mcp/session.js`), so the facade itself cannot run a real-provider turn until that mode is wired in the next release.
A stub turn is never evidence of a model route; a failed provider call is reported as the provider error, not smoothed.

## Expected PASS lines (0.3.1)
    initialize OK / tools/list OK: 9 tools
    CALL fleet.status -> OK
    CALL session.create -> OK (sessionId, mode stub, model deterministic-stub)
    CALL order.drive_roundtrip -> OK (final_state.order_state=done, final_task_state=done, real_lane="fleetRoster mounted")
    CALL order.stage_real -> OK (mode real-lane, real_lane="fleetRoster mounted", order id)   # 0.3.1 roster host only
No token, no model key, no spend beyond the existing plan, no publication. If a call refuses, the refusal code and the
action-log row are the evidence.

## Live facts to re-read after the 0.3.1 install (`dsh-fleet-admin status` / `dump-config`)
installed version 0.3.1, installed-tree count = tarball members (123), rows 8 pinned / 12 disabled, MCP listeners exactly
127.0.0.1/[::1]/100.114.17.67:8789, nine tools, `fleet-roster` row mounted, `llm-pi-ai` route `providers.opencode-go`
(apiKeyEnv only, no value), `subagent.maxActiveSubagents: 32`, `rates` contains
`opencode-go/deepseek-v4.1-flash`, `roster.models` doctrine table. Token file/action-log paths unchanged
(`/var/lib/fleet/state/mcp/token` 0600; `/var/lib/fleet/log/mcp-actions.jsonl` hash-chained).
