# Phase A · Mission item 2 — Comms layer: design + gap report

**Status:** design only; nothing built, nothing installed. **Owner:** Team B leads design + build. **Inputs:** `duet-wake-evidence.md` (2026-11-02T06:31 EDT), Nox 113000Z directive (handoffs through the MCP; agents stay alive; wake-on-message; idle watchdog on active work), Nox 123400Z item 2, live 0.3.3-consolidated read at 09:09 ET.
**Proof target (mission):** a handoff to a parked participant wakes them and returns an ack through the bus.

## 1. What exists today (verified)

| Fact | Evidence |
|---|---|
| The installed plugin has **no bus**: `src/` = client, core, egress, firstrun, fsguard, housekeeper, ledger, lessons, mcp, orders, policy, roster, sandbox. `package.json` still exports `./git-bridge` and `./bus` → missing files | `ls` live package; `package.json` |
| **No server→client stream**: `GET /mcp` → 405, `notifications/` handled = only `notifications/initialized`, no timers pushing to clients; live `curl` GET → 405 | wake evidence 06:31; live probe 09:09 |
| Board handoffs are pull-only ops (`posts`, `pull`, `ack`, `board-get`, `order-list`) through `board.op` / control socket | `src/mcp/tools.js`, `src/orders/control.js` |
| One shared bearer token for the facade (single file `state/mcp/token`); auth = token match only | `src/mcp/auth.js`; `fleet.status` |
| No participant registry, no presence, no heartbeat, no idle watchdog anywhere in the package (housekeeper = retention/backups only) | `grep` across `src/` |
| Roster keeps a live-agent map (`host.js`: `live` Map, `readSession`), lanes are plugin-created real DSH sessions, and `lane-spawn`/`lane-stop`/`lane-get` exist | `src/roster/*` |
| **Roster liveness is already broken in live state**: all lanes `stopped`; `builder:1` appears several times with different sessions/cwds (id reuse, no registry) | `roster-get` 09:09 ET |
| The one real order stalled with **no wake**: lead claim `lease-lost` after its first turn, no builder spawned, all tasks open | `claims`/events, o1 board |
| The unshipped bus (0.2.0) already has: hashed per-principal tokens (`bus_tokens`, principals `nox`, `service:git-bridge`; **`cayleb-local`,`duet` reserved**), 8 JSON routes + MCP route, two supervised listeners, funnel-header refusal, no-CORS posture, hash-chained action log | `~/workspace/harness-build/dsh-fleet-bus/package/src/bus/{auth,routes,http,listener}.js`, `docs/API.md` |

**Verdict:** mission item 2 is a **build from primitives**, not an extension: bus + registry + wake + watchdog + tokens + presence. It is also
the keystone for items 3 (Duet bridge is a bus transport), 1 (agent-activity view reads the registry) and 5 (o1 stalled because nothing woke it).

## 2. Design

### 2.1 Participant registry (one table, one owner op surface)

`bus_participants` (migration `bus/002-participants`; SQLite in the fleet DB, hash-chained events alongside):

| Column | Meaning |
|---|---|
| `id` | stable participant id: `lane:builder-1`, `seat:lead`, `service:git-bridge`, `external:duet`, `owner:cayleb` |
| `kind` | `lane` \| `seat` \| `service` \| `external` \| `owner` |
| `role` | lead/builder/checker/researcher/service/external |
| `session` | DSH session id when plugin-hosted (null for external) |
| `state` | `alive` \| `busy` \| `parked` \| `stopped` \| `stale` \| `offline` |
| `presence_at` | last heartbeat/observation (monotonic + wall clock) |
| `wake` | `{mode: inject\|start\|transport, bound_ms, last_wake_at, prods}` |
| `ack_cursor` | per-board/library read cursor (reuses bus ack shape) |
| `token_hash` | sha256 of the participant's bearer token (never plaintext) |
| `created_at`,`revoked_at` | lifecycle |

Owner ops (control socket, `cayleb-local` only): `participant-add`, `participant-revoke`, `participant-list`, `participant-state` (read).
Lanes/seat register **through the plugin**, never self-declare: the registry is written by roster/core hooks, so a lane cannot forge a peer.
External participants (`external:duet`) are registered by an owner/ruling act after the bridge exists.

### 2.2 Per-participant tokens (replaces one shared bearer)

- One token per participant, minted by owner op; only `sha256` stored (same discipline as `bus_tokens`); plaintext shown once at mint.
- Scope matrix per principal: `lane:*` → board/handoff ops for its own principals; `owner:cayleb` → everything on its socket; `service:git-bridge`
  → evidence only; `external:duet` → `handoff.pull`/`handoff.ack`/status only (nothing that creates orders or verdicts).
- Revocation is a row update read by the next request (inherited bus property); attempts are audited (`args_sha256`, never values).
- The **MCP facade** keeps a token but now resolves it to a participant; `cayleb-local` stays peer-credential on the control socket, not a token.
- Duet's token, if the bridge lands, lives **only in the courier**, never in Duet's hands (see B).

### 2.3 Handoff envelope + wake-on-message

New bus routes (JSON + MCP tools, one operation per gap):

| Op | Who | Behaviour |
|---|---|---|
| `handoff.send` | owner, lanes (to self-team), services | `{to, kind, body, ack_required, ttl_ms, idempotency_key}` → `{handoff_id, delivered: 'waking'\|'queued'\|'refused', transport}` |
| `handoff.pull` | the addressee only | move own cursor; returns pending handoffs; content labelled `bot-authored-data` |
| `handoff.ack` | the addressee only | `{handoff_id, note?}`; idempotent; writes the ack row and wakes the sender's queue |
| `handoff.wait` | internal | stream/queue wait for ack with TTL, for the proof and for blocking senders |

Wake path is **explicit per kind** — the plugin never claims a wake it cannot perform:

- **T1 plugin-hosted, running** (`lane`/`seat` alive): the host injects one turn into that session (`ctx.agents` handle → session prompt path,
  same pipeline as `session.prompt`), with the handoff in the turn's context. This is a true wake (Nox rule 3).
- **T2 parked/stopped lane**: roster `start` the lane (its budget/authority check applies) then inject. If `start` is refused (budget, role,
  lane cap) the handoff is `queued` + the sender is told the refusal, never "waking".
- **T3 external (Duet later)**: bus enqueues and the transport delivers on its own schedule; state is `queued`/`delivered`, and the ack is the
  only proof. No push claim (see B2 gap).
- Wake is allowed only to participants in the sender's scope (lead→its team; owner→all; lane→same team); cross-team handoffs route through the
  lead seat, recorded as a hop.
- Ack timeout = `ttl_ms` → the sender gets `overdue` and the face shows it; no silent loss.

### 2.4 Idle watchdog (scoped to active work — the owner's qualifier)

- **Active work** = the participant holds any of: an open claim/lease, an open task assigned to it, a non-empty handoff queue, an order it leads
  with a non-final state.
- A scheduler tick (fleet-core timer, default 15 s, config `comms.watchdog.tick_ms`) computes idle-since-last-activity per participant and compares
  with the role bound (`comms.watchdog.bounds`: lead 5 min, builder 10 min, checker 10 min, researcher 10 min; config, owner-tunable).
- On breach: one prod handoff (`kind: 'idle-prod'`, body names the open queue) through the same wake path; `prods` increments; exponential backoff
  (bound, 2×, 4×, cap 4 prods per hour) then `stale` + face alert. **No prod for a participant with no active work** (no waking idle lanes).
- Every prod is an audited event (`watchdog.prod`, participant, reason, queue size); the face shows it as activity.

### 2.5 Presence / liveness

- Heartbeat: plugin timer writes `presence_at` for `alive` participants; lane session status transitions from roster events; external participants
  are `stale` by definition until a transport delivery/ack updates them.
- Derived states: `alive` (heartbeat within bound), `busy` (turn running), `parked` (no turn, wakeable), `stopped` (session gone, startable),
  `stale` (missed beats), `offline` (revoked/transport down).
- Liveness probes: control socket + facade + DB chain head, surfaced as the fleet status pill (D face) and the `fleet.status` MCP tool extension.
- Presence is read-only truth for the face: **the UI shows last-observed state and its timestamp, never a "live" label without a heartbeat.**

## 3. Gap report (named, against the proof target)

| ID | Gap | Severity | Close |
|---|---|---|---|
| G-A1 | Bus is not in the installed 0.3.3 package (`src/bus` absent, stale `exports`) | critical | ship bus modules + bundle rows + manifest regen (version bump), variant pipeline proven yesterday |
| G-A2 | Facade cannot push (GET 405); a handoff cannot be delivered *to* a participant through MCP itself | critical | wake is in-process (T1/T2); MCP only sends; document honestly for external |
| G-A3 | No participant registry; lane ids reused (`builder:1` ×N) and liveness wrong (`stopped` while the plan believed live) | critical | §2.1 + roster hooks; migration + audit |
| G-A4 | Single shared bearer token; no per-participant scopes; `duet` reserved so no token can exist for it | high | §2.2 |
| G-A5 | No handoff/ack envelope: board posts are order-bound (`readableBoard` → `mustRow('order')`); a mailbox board family is new | high | new bus routes + posts table already allows order-less rows (`order_id` nullable) |
| G-A6 | Wake of a stopped lane needs `roster.start`, which needs budget/role authority and can be refused | high | T2 semantics + refusal surfaced; test with refused start |
| G-A7 | No watchdog scheduler; housekeeper timer is retention only | high | §2.4 + config + tests (fake clock) |
| G-A8 | No liveness data in the MCP tool surface: `fleet.status` has no participants/presence section | medium | extend `fleet.status`; add `participants.list` tool |
| G-A9 | `board.op` acts as the owner principal; a lane-scoped token must not inherit that | high | scope matrix §2.2; cross-principal refusal test |
| G-A10 | Token minting is an owner act; without an owner present a new participant can't get a token — needs a rebindable-per-person design or a session-token bound to the plugin's own host | medium | design choice to rule: long-lived hashed tokens (owner mints once per role) vs short-lived session tokens minted by the plugin per wake |

**Named blocks (need a ruling, not code):**
- **B0** o1 is `blocked` on the dead work tree + commit gate — the wake fix alone does not unblock it (Lead/Nox: work tree (a) or per-lane artefacts (b)).
- **B2/B3** belong to the Duet leg but constrain A: the only external participant is `external:duet`; its wake is transport-bound, and no Claude
  seat can start without a credential/engine.
- **Ruling asked:** token lifetime model (G-A10), watchdog bounds/defaults, and whether a lane may hand off to another team directly or only via its lead.

## 4. Build plan (batch 2, after rulings)

1. Cut `finish/comms` from the live 0.3.3 source tree (source of truth: the installable variant tree used for 0.3.3; the unshipped bus 0.2.0 modules
   are the base, adapted to the 0.3.3 row wiring). Maker: Team B; fresh checker (not the maker) runs the proof.
2. Migrations `bus/002-participants` (+ tokens backfill for `service:git-bridge`), registry hooks from roster/core, handoff routes, watchdog,
   presence in `fleet.status`, MCP tools `handoff.*` + `participants.list`.
3. Tests: registry lifecycle; token scope matrix (each forbidden cross-principal call refused); handoff idempotency; T1 wake (fake host records one
   injected turn); T2 wake (fake roster start) + refused start; ack path; watchdog fake-clock (active vs inactive participant); presence staleness.
4. Proof harness (the mission's proof): a **real parked builder lane** in a scratch DSH home; `handoff.send` to it; observe wake (lane turn starts,
   its own pick-up recorded), it calls `handoff.ack`; sender receives the ack through the bus; captured as a transcript file. Feeds a fresh checker.
5. Then (and only then) the install variant + mission proof on the live consolidated setup; item 5's o1 re-stage follows the ruling.
