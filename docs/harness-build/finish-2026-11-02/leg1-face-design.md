# Leg 1: the visible face. Spec, feasibility, build plan

Written 2 Nov 2026 (about 09:45 ET) on outis-desktop for the FINISH-THE-PLUGIN mission. Scope is the agent-activity view, the shared-workspace shape and the contained Roblox Studio instances. The sidebar section, phone-VM panel and Windows-VM monitor belong to order o1 and are only referenced where something plugs into them.

Evidence labels: **VERIFIED** means I read the file or ran a read-only command today. **REPORTED** means it comes from the task brief and I did not re-check it. **INFERENCE** is my reasoning. I started no process, installed nothing, made no network call, and touched no Studio, no live unit and no profile. Source base is the worktree `finish-int/package` (candidate 0.3.1 lineage), DSH checkout `~/.local/opt/deepseek-harness-official`.

## 0. Reconciliation with Legs 2, 3 and 4 (2 Nov, binding; supersedes anything below that disagrees)

Leg 2 is now BUILT (`finish/comms` b10e35f, fresh-checker PASS). Where this document used to rely on Leg 2's design it now relies on the built code, read today and not run (VERIFIED-BUILT). The decisions:

1. **No re-derived active-work logic.** The activity view consumes Leg 2's outputs: `fleetRoster.rosterGet()` (per lane `session`, `state`, `idle_since`, `last_event_at`, `wake_count`, plus the `staffing` block), `fleetRoster.watchdog.status().open_episodes` (the watchdog's own record of a live lane idle with work: lane-keyed, so it covers the Lead's episode and never the unstaffed ones), `fleetRoster.staffing.episodes()` (the open `unstaffed:<role>` episodes: Leg 2 amendment C6, this leg's gap G6), and `fleetComms.who()` (3.4). `order-unworked` and `lead-idle-holding` are projections of those, not new predicates. There is one grace key, Leg 2's `roster.unstaffed-grace-ms`; `face.unworked-grace-ms` is deleted (3.3).
2. **Presence schema is Leg 2's built one** (3.4): exact fields, and `silent-while-running` is a ping detail on a `working` lane, not an `unreachable` state.
3. **Owner-present is one signal**, defined in Leg 2 section 13.3: this leg's `present_until` and `roster.ownerPresent(lane, at)`, set and cleared by this leg's owner ops `session-join` and `session-leave` (Leg 2 13.3 uses these names; there is no `lane-join`/`lane-leave`). The four hooks this leg adds are listed in 4.3 item 4. Leg 4's desktop lease is independent and gates nothing.
4. **Elevation is read-only here.** The page renders Leg 4's `elevations[]` as cards with no button and no argv (3.2, 3.5). No browser write for any `elevate-*` op, policy, grant or ticket op is ever on the flag 2 allowlist. The remote-authority gate (2, decision 5) applies to the cards.
5. **Owner-origin messages are not authenticated in this install.** `lane-message` is a control-socket op (`authenticated 0`); the structured `from:'owner'` source is set only on an authenticated channel (4.3 item 6).
6. **The status pill belongs to o1.** This leg supplies `attention` (including `elevation-pending`) as its single source; Leg 4 does not touch the pill.
7. **Shared files** (`package.json`, `rows.manifest.json`, `cordis.patch.yml`, the `client.js` bundle) follow Leg 2 section 13.2 and are edited once in the integration commit.
8. **The Tier B confirm path** is Leg 4's item L4-A. The one approval statement is Leg 4 section 9.
9. **The Studio login display** is a residual risk against Leg 4's baseline (5.4 rule 8).

## 1. Summary

| Item | Verdict |
|---|---|
| (a) Agent-activity view | Buildable and checkable today in a scratch DSH home. One gap decides it: no browser-to-fleet data path exists yet (section 2). The proposed path is DSH's own `ctx.connection` RPC, which I read in source but nobody has run for a third-party plugin; spike G0 proves it first. |
| (b) Shared workspace | A narrow v1 is buildable now: the owner watches, joins and messages an agent lane's session, under a care fence that makes the plugin unable to close or take over a session. Owner-origin messages need new owner ops and one host change (a structured owner source). An agent joining the owner's own sessions is not buildable and not wanted. |
| (c) Contained Studio instances | Conditionally feasible for one extra instance on this Linux desktop, unproven for two. No probe has run. The registry, supervisor, guards and panel can be built and tested today with a fake launcher. A real instance needs the owner for the login and a go for the first probe. |
| Token conflict | Build on Midnight Slate v1.0.0 now, with a semantic alias layer so a pack v2 is a token-file change. Needs a one-line ruling from Nox (section 7). |
| Rendering | A fresh checker renders in a scratch DSH home with Playwright Chromium and grades by measurement. DSH-seat models cannot read images, so the owner (and optionally an independent Claude session) grades the look (section 8). |

## 2. How the browser reads fleet state (the keystone)

**VERIFIED.** Nothing in 0.3.1 serves fleet state to a browser:

- fleet-core's only owner surface is a unix socket with a uid-1000 peer check (`src/core/control.js`). A page cannot open it. Design 4.11 assumes "a fleet-core read route on the loopback listener"; I grepped `src/core` and no such route exists.
- The MCP facade (port 8790) is bearer-token and tool-shaped, and GET returns 405. Wrong credential and wrong shape for a page.
- DSH's `ctx.remote` namespaces are fixed at DSH build time ("the Client does not discover the Host's active Services", `packages/api/remotes/README.md`, Known Limitations). A plugin cannot add one.

**VERIFIED in source, never run for a third party.** `packages/client/connection/src/rpc.ts:153-215, 295-309`: the host half can call `ctx.connection.rpc.handle('/fleet', handler)` for JSON endpoints and `ctx.connection.fetch.register({ path: '/api/fleet/...', methods: ['GET'], requestBody: 'buffered', fetch })` for binary responses. The browser half calls `ctx.connection.rpc.call('/fleet', 'activity', payload, signal)`. The doc comment on exact routes says only owner Peers reach them; the browser's shared Peer is the owner principal and authenticates with DSH's launch token and cookie, so no new token is needed. The consolidated unit serves the fleet profile on 127.0.0.1:3092 (unit file).

**Design decisions**

1. The host half goes in `src/index.js`, the existing no-op root plugin behind the bare-name `fleet` row that already carries the client half. No new row for this leg, so this leg changes no entry of `rows.manifest.json` (pinned 8, disabled 12); the integrated tree's manifest is regenerated once for the other legs' rows (Leg 2 section 13.2). Other services (`fleetCore`, `fleetOrders`, `fleetRoster`, later the bus) are read in-process, with `ctx.get` for the optional ones, as `src/roster/host.js` does.
2. Every endpoint is read-only. Nothing here exposes a control op.
3. Endpoints: `activity` (snapshot), `lane` (one lane, last N messages), `studios` (registry and state). Binary: `GET /api/fleet/studio/<id>/frame` (PNG).
4. **Trust statement.** Every authenticated browser connection shares one owner `browserPeer`, so a `peer.principal` re-check only proves "is a browser" and is kept as defence in depth, not as a boundary. The browser peer is owner-equivalent, and so is any uid-1000 process holding the launch token (design decision 4: hands run as uid 1000). Because of that, every write allowlist stays narrow (section 4.4, flag 2) and no endpoint returns a secret. Every browser write RPC records `channel: browser-peer, authenticated: 0` (Leg 2 section 13.5).
5. **Remote-authority gate.** `lane`, `studios` and `frame` refuse unless `ctx.connection.allowsRemoteAuthorities === false` (`rpc-host.ts:79,101`, VERIFIED present); `activity` serves only the redacted summary otherwise. A tailnet-exposed fleet profile therefore never serves lane transcripts or frames. The `elevations[]` list is gated the same way (it is served only as the redacted summary otherwise). Test: a connection stub with `allowsRemoteAuthorities: true` gets the refusal code on each gated endpoint.
6. **Refresh is polling:** 2 s while the page is visible, 15 s hidden, backoff to 10 s on error. There is no `since`/`unchanged` shortcut keyed on `head.seq`. `events.seq` does not cover the page: roster state changes and flags only save the lane row (`roster/lanes.js` `setState`, `addFlag`; no `appendEvent` call exists under `src/roster`, VERIFIED by grep), and `since`, `lease-near-expiry` and `lane-silent` change with the clock. Rule: the server returns the full snapshot with an `etag` that is a hash of the snapshot with every time-derived field removed; a matching `If-None-Match` returns `{unchanged:true}`. The client recomputes `since`, remaining lease and silence from `at` and the raw timestamps on every render tick, so those never depend on the server's answer. Test: flip a lane `running` to `idle` through `book.setState` with no event appended and assert the next poll's etag differs and the lane row changes. SSE through a streaming exact route is possible later and not needed.

**Gate G0 (first build task, scratch home):** prove an RPC round trip from a client component to the host half, prove it is refused without the browser's credential, prove one PNG from a fetch route, confirm the client service is spelled `connection` in `inject`, prove the `allowsRemoteAuthorities` refusal (section 2, decision 5), and prove that panel id `fleet-activity` opens `main[fleet-activity]` and the row appears and disappears with the page (section 6). If G0 fails, the fallback is a plain `node:http` listener on a second loopback port with its own random token handed to the page by the host (more code, same snapshot contract). The rest of this document does not change either way.

## 3. (a) The agent-activity view

### 3.1 Which sessions are "the agents'"

A DSH session is an agent session if and only if it has a row in the `roster` table (`src/roster/store.js`) and is not stopped, plus stopped rows from the last 24 hours shown dimmed. Everything else in DSH's stock session list is the owner's and never appears in this view. Engine lanes (`virtual`, no DSH session) show as engine cards. Participants with no DSH session (Nox, Duet, delegator) come from leg 2's registry.

Lanes are plugin-created root agents (`src/roster/host.js:createAgent`), and a root agent carries no `origin` (only subagents do; `session-controller/src/list.ts:366`), so they most likely also show up in the stock sidebar list, one workspace per lane directory. That clutter is why this panel is the curated view. Unverified until probe S1 (section 4.4).

### 3.2 Data contract: `GET fleet/activity`

| Field | Content | Source today |
|---|---|---|
| `v`, `at` | contract version 1, server time | face |
| `head` | `{seq, hash}` of the events chain, shown for the feed cursor only (not a change detector, see section 2, decision 6) | `events` max row (`core/store.js`) |
| `etag` | hash of the snapshot without time-derived fields | face |
| `host` | integrity ok, lanes allowed, selftest flag | `fleetCore.core.status()` |
| `lanes[]` | `id, role, principal, state, session, domain, provider, model, engine, order, task, flags[], started_at, stopped_at` | `fleetRoster.book` through `view()` (`roster.js`) |
| per lane, derived | `holding {kind, id, lease_until, via}`, `since` (state entered), `last_mutation_at` (memory-only in the roster; null after a restart), `spend_usd`. `session`, `idle_since`, `last_event_at` and `wake_count` are read from `roster-get`'s lane rows and are not re-derived or duplicated here | `holding` per 3.3a; `spend_usd` from `taskView.spent_usd`; `last_mutation_at` needs gap G1 |
| `orders[]` | `id, title, state, claimant, lease_until, attempts, block_reason, waiting_on, tier, size, budget_usd, fixture, facts, tasks{state:count}, goldfish` | `orders.listOrders`, `orderView` |
| `tasks[]` | `id, order, seq, title, needs_role, state, claimant, lease_until, rounds/rounds_allowed, after[], block_reason, waiting_on, final, spent_usd` | `orders.orderTasks(order)` |
| `queue` | `claimable{builder,researcher,lead}`, `stranded[]`: open tasks whose order is not `in_progress`, each with `why` (`order open: lease lost`, `order blocked`, `deps pending`) | the read-only queue join in 3.2a |
| `stalls[]` | lane, task, reason, at | `roster_stalls` (the store has only a point lookup; gap G2) |
| `feed[]` | last 50 rows of `events` (seq, ts, kind, actor, short data), cursor by `seq`, passed through the redaction pass in 3.5 | read-only SELECT on `events` |
| `holds[]` | pending Tier B tickets, read-only. The confirm command is shown only once Leg 4's L4-A has made it the working path; before that the card says the confirm path is unavailable in this install | `policy_tickets` (`policy/tickets.js`) |
| `elevations[]` | read-only: `id, state, requester {agent, order}, age, expires_at, binding12` (the first 12 hex of the binding hash), `classification, remote_decision`. Never the argv, never an outcome, stdout or stderr tail. The card names the command to run at the machine (`dsh-fleet elevate show <id>`) | Leg 4 `fleetElevate.list()`; `null` until it exists, and the panel tolerates `null` |
| `participants[]` | the `participants` of Leg 2's built `who()`, plus the lane rows' `ping` and `state` (3.4) | Leg 2 built; the panel tolerates `null` |
| `attention[]` | derived flags, see 3.3 | `src/face/attention.js` |

Construction rule: no raw SQL on orders tables. Use the orders read API (`listOrders`, `orderTasks`, `getTask`, plus gap G5 below), the roster book, and a read-only SELECT only on `events` and `roster_stalls`. A test asserts the module never calls a write member.

### 3.2a Read-only queue join, fixture partition

`orders.queue(actor)` needs an actor and partitions by `K.fixtureOf(actor)` (`orders/tasks.js:240`). The face has no actor and must not forge a write-capable one. Gap G5 adds to the orders service one read-only member, `queueView({role, fixture})`, that runs the same two SELECTs as `queue` with the fixture partition as an explicit parameter and `depsPending` applied, and `decomposers(orderId)`, a SELECT of `author` from `posts` where `order_id = ? and kind = 'decomposition'` (the board row `tasks.js:155` writes). Both are read-only and covered by the no-write test. The snapshot runs each flag twice, once per partition, and the page shows non-fixture orders by default and fixture orders only behind an explicit "show fixtures" toggle. Sensing's own `if (order.fixture) continue` (`sensing.js:59`) is unchanged and is not the face's rule. Leg 2's staffing detector (`roster/stuff.js`) reads open work through `orders.queue` with a `builder:0` or `lead` actor: a read-only call, shipped as built and not changed here. `queueView` is used only for the stranded list and the fixture partition; the checker's seeded incident asserts the two reads agree. `decomposers(orderId)` is dropped from G5 (3.3a). Test: the same seeded incident in the fixture partition and in the live partition yields identical flags, so a fixture-seeded pass proves behaviour on a live order.

### 3.3 Attention flags, and the o1 failure they would have shown

`deriveAttention(snapshot, now)` is one pure function. Its o1 fixture is not a hand-made snapshot: a seed script in a scratch home drives the real orders API (`file`, `claim`, `decompose` with five tasks, then an advanced clock past the lease so the sweep emits `orders.lease-lost`) and the roster book (Lead lane `idle`, no builder lanes), then calls the face's `snapshot()`. The expected shape comes from filing 134400Z and the brief (order o1 `open` after `lease-lost` at 13:11:04Z, five open builder tasks, Lead idle since about 12:57Z, `roster_stalls` empty); I did not open `fleet.db`.

**3.3a How an idle Lead is tied to its order (reconciled).** VERIFIED-BUILT: the roster sets `lane.order` for a Lead on an order claim (`sensing.js:35`) and clears it when the order's claimant is no longer that Lead (`:60`); after `orders.lease-lost` the claimant is cleared. The idle watchdog covers that state itself: it reads the orders the Lead holds or is owed (an open order in its queue after a lease loss) and records an episode in `roster_idle`. The face reads the episode and the `holding` field and re-derives nothing, so the decomposition-post fallback of the first draft is not needed and `decomposers()` is dropped. An episode row has `lane, ref, episode, started_at, nudges, last_nudge_at, lease_at_nudge, outcome, sig`; the reason is not a column, and `sig` is opaque to the face.

| Flag | Fires when | Threshold, hysteresis, policy key | On the o1 fixture |
|---|---|---|---|
| `order-unworked` (high) | `staffing` (from `roster-get`) shows a role with `live: 0` and open work (`open_tasks` or `open_orders` above 0) **and** a roster idle episode with `lane` `unstaffed:<role>` is open for that order, **read through `fleetRoster.staffing.episodes()` (G6, Leg 2 C6) and not from `watchdog.status().open_episodes`**: VERIFIED-BUILT that `open_episodes` is `book.liveLanes().flatMap(l => store.idleOpen(l.principal))` (`roster/watchdog.js:196`) and so carries live-lane principals only, while the unstaffed rows are written under the virtual lane `unstaffed:<role>` (`roster/stuff.js:31`) and appear neither there nor in the `roster-get` `staffing` block. Leg 2 opens it only after `roster.unstaffed-grace-ms` (default 120000) and closes it when the work is staffed or gone. Before the episode exists the row shows as an info line "no <role> lane, inside the grace" | one grace key, Leg 2's; clears when the episode closes. Tasks whose `after` dependencies are pending never count (Leg 2's `depsMet`) | fires: "o1: 5 tasks open, no builder lane running" |
| `lead-idle-holding` (high) | `watchdog.status().open_episodes` has an open episode for the Lead lane (`lead`). Leg 2's predicates (`roster/active.js`) have already decided the Lead holds or is owed work only it can do, so a Lead whose order waits on someone else (`waiting_on`) or whose tasks are staffed has no episode and raises nothing. The line shows the order (`ref`), `started_at`, and nudges used against `roster.idle-nudge-max` | clears when the episode closes (progress, work gone, or the nudge cap) | fires, with `owed-order` work after the lease loss: "Lead idle since 12:57, o1 not progressing" |
| `lease-near-expiry` | an order or task with a claimant and `lease_until - now < face.lease-warn-ms` (default 180000), and `lease_until` unchanged | clears when `lease_until` moves. The renewal rule is a changed content hash (`orders/leases.js`), so "no renewal" is simply `lease_until` not having moved | would have fired about 13:08 |
| `claims-lost` | the order has one or more `orders.lease-lost` events | high at `orders.max-lost-leases` minus 1 (a third loss blocks); clears only when the order reaches `done` or `blocked` | warns after the first loss |
| `lane-silent` | the comms lane row has `state: 'working'` and `ping: {ok: false, why: 'silent-while-running'}` (Leg 2's `lanePing`, key `roster.silence-ms`, default 900000) | Leg 2's computation and key; the face recomputes nothing and adds no threshold. Clears on the next event. Wording is "no session event for N min", never "stuck", since a long generation may emit nothing. **It never raises `lane-error`** | uses `last_event_at` from `roster-get` for the age |
| `lane-error` (high) | the comms lane row has `state: 'error'` or `state: 'unreachable'` (Leg 2 sets `unreachable` for `no-agent` and `agent-error` only) | clears when the state leaves them. `parked` shows as a state word. Vocabulary: the comms `working` is the roster's `running` | none on o1 |
| `participant-stale` | a pull-only participant row with `pending.pending > 0` and `state` `stale` or `absent` | Leg 2's per-kind TTLs verbatim. A `nox` row with `unmeasured: true` renders as "unmeasured: nothing reads the bus as Nox yet", never as absent | Leg 2 built |
| `elevation-pending` | `elevations[]` has a request in state `pending` | clears on any other state. It counts into `attention` and so into o1's status pill; there is no browser action | needs Leg 4 `fleetElevate.list()` |
| `host` | integrity not ok or lanes not allowed | none | uses `status()` |

Tests: one per flag and one per lane state (`working`, `idle`, `parked`, `stopped`, `error`, `unreachable`), each asserting fires, does not fire just under the threshold, and clears.

This is display only. Waking the Lead and spawning builders is leg 2 and leg 5. Showing the stranded state within one poll is this leg's contribution, and it is what `sensing.js` cannot do today because `pass()` skips every lane whose task is null.

### 3.4 What leg 2 supplies (consumed, not redefined)

Leg 1 defines no participant shape. It consumes Leg 2's built `who()` (the one function behind `bus.who`, `comms-presence` and the comms block of `fleet.status`; `leg2-comms-design.md` 3.3, VERIFIED-BUILT). The exact fields:

- `{at, lanes[], participants[]}`.
- **Lane row:** `id` (the lane principal), `kind: 'lane'`, `role`, `state` (`working`, `idle`, `parked`, `stopped`, `error`, `unreachable`), `ping` (`{ok: true}` or `{ok: false, why: 'no-agent' | 'silent-while-running' | 'agent-error'}`), `idle_since` (only in `idle`), `at`. There is **no session, `last_event_at` or `wake_count`** in it: those come from `roster-get`'s lane rows. **Join on `id` = the lane principal**, unique among live lanes (not on `session`).
- **Participant row:** `id`, `kind`, `state` (`fresh`, `stale`, `absent`, `never`), `ping: 'n/a'`, `ttl_ms`, `last_seen_at`, `last_pull_at`, `last_ack_at` (raw epoch ms or null), `display`, `pending` (null or `{pending, oldest_age_s, head}`), and `unmeasured: true` on a `nox` row never seen.
- The face's etag removes time-derived fields; it keeps the raw timestamps and `ttl_ms` and recomputes fresh, stale and absent from `at` on every render tick, as Leg 2's state rule defines them (`fresh` within `ttl_ms`, `stale` to six times it, then `absent`).

For Duet, `last_seen_at` (the delivery leg's pull) and `last_ack_at` (read evidence) render as two separate ages, "last pull 4 min ago, last acknowledged 2 h ago", and are never folded into one word. The strip uses no "nudged", "woken" or "present" wording for pull-only participants. If the output is absent the strip is omitted and nothing else changes.

### 3.5 The page

`main` key `fleet-activity` with a `sidebar.panellist` entry registered in the same generator, so the row never exists without its page.

- **Header strip:** host health word, lanes live out of 5, open tasks, attention count. Words, not colour alone.
- **Agents:** one row per lane: role and principal, state word with a shape glyph, one line of what it works on (task title, else order title, else "no work"), time in state, model. A row opens a detail drawer: the last 20 messages as plain text (never HTML, capped, treated as untrusted), flags, spend, an "Open session" link (route confirmed in S1). Transcripts include tool results that can hold file contents, env dumps and credentials a lane read, and passwords and tokens never travel through chat, board or logs. So `tail()` (G3) returns assistant and user text only; tool results are replaced by a one-line stub (tool name, byte count) and expand on request after a redaction pass that masks key-shaped strings (`sk-`, `ghp_`, `Bearer `, long base64 or hex runs, `KEY=`/`TOKEN=`/`PASSWORD=` lines, PEM blocks) and caps each item at 2 KB. The same pass runs on `feed[]` data. Test: a transcript fixture with a planted fake key, an env dump and a PEM block never appears in a response body.
- **Queue:** claimable by role now, then stranded tasks with the reason. The two lists make the o1 state obvious.
- **Feed:** recent events grouped by kind prefix.
- **Pending holds and elevation requests** appear as read-only cards. A hold card shows the confirm command only once Leg 4's L4-A has made it the working path; before that it says the confirm path is unavailable in this install. An elevation card shows `elevations[]` (id, state, requester, age, expiry, the 12-hex binding hash, classification) and names the command to run at the machine. It has no argv, no outcome and no button, and the remote-authority gate applies to it. There is no approve control anywhere in this view (section 7, flag 5).

Every state in the pack checklist B must render: default, hover, focus (2 px accent ring, 2 px gap, never animated), pressed, disabled, loading (static placeholder plus a worded label, no pulse), error (plain words, nothing lost, one next step, a Retry), empty ("No agents running." plus the exact command to start the Lead), reduced motion, Windows High Contrast. No emoji, no sparkle, no glow, no pulse or ping loops, no uppercase tracked labels, no `transition: all`.

### 3.6 Small changes in existing modules (all read-only or additive)

| # | Change | Why |
|---|---|---|
| G1 | `roster`: a one-line read accessor `fleetRoster.lastMutationAt(principal)` over `sensing.mutated` (in memory, not persisted, null after a restart). `last_event_at` is not added here: it is already in `roster-get` | "doing what" freshness |
| G2 | `roster/store.js`: `stalls()` | stall list |
| G3 | `roster/host.js`: `tail(session, n)` with the redaction in 3.5 | the drawer. `readSession` calls `tools.resolve` and is too heavy for a poll |
| G5 | `orders` service: read-only `queueView({role, fixture})` (3.2a) | the stranded-queue join without forging an actor or raw SQL on orders tables |
| G6 | `roster`: read accessor `fleetRoster.staffing.episodes()` returning `[{role, order, episode, started_at}]` for the open `unstaffed:<role>` rows. **This is Leg 2 amendment C6** (Leg 2 section 14); it is built once, in the comms amendments branch (Leg 2 section 13.1 step 1), and this leg consumes it. It is a separate accessor, not extra `open_episodes` rows | the data source of `order-unworked`; without it the flag and the o1 fixture assertion have no sanctioned input |
| G4 | `package.json` `files`: add `src/face/*.js`, `src/face/studios/*.js` | whitelist; edited in the integration commit and the installed-tree file count is recomputed once (Leg 2 section 13.2) |

## 4. (b) The shared workspace

### 4.1 Facts

- **VERIFIED.** Lanes are driven by `agent.followup` through `host.send` (`host.js`). The owner ops that exist are `roster-get`, `lane-spawn`, `lane-get`, `lane-stop`, `session-get`, `tree-get`, `run-records`, `providers-get`, `engine-probe`, `goldfish-status` (`roster/control.js`). There is no op to message a lane.
- **VERIFIED.** `host.js:user()` stamps every followup `source: {kind: 'user'}`, and the roster's own briefs use it. A transcript therefore cannot tell the owner's message from a roster brief. An owner message needs an explicit owner source (4.3 item 6).
- **VERIFIED.** `roster.dispatch()` and `start()` brief any idle lane, and `lane-stop` disposes the agent. Nothing knows an owner is present.
- **VERIFIED.** Lanes work in their own directory under `workRoot/lanes/` (`laneCwd`, `ensureLaneDir`). The consolidated unit is a user unit, so it runs as cayleb and those files are owner-readable.
- **VERIFIED.** The Lead and Researcher presets include `tool-session-query` (`presets/lead.patch.yml:37`, `researcher.patch.yml:38`). Whether that indexes sessions the owner opens in the same DSH home is unverified and is a care-fence risk (probe S2).
- DSH LiveShare (`docs/subsystems/liveshare.md`) is the closest stock feature. It shares one session with an external viewer through a separate Liveshare server and a network relay, with `view` and `write` tiers. It is the wrong tool (network, external viewer, experimental). Its rule that confinement applies to the viewer's turn and never to the owner's is the model for the fence below.

### 4.2 Definition

A **shared session** in this plugin is an agent lane's own DSH session plus an owner presence record. Three owner relations:

| Relation | Meaning | Effect on the lane |
|---|---|---|
| `watching` (default) | the owner reads the transcript in the fleet panel or the stock session view | none |
| `present` | the owner joined: lane field `present_until` (a server-side timestamp) is in the future | dispatch does not brief it, sensing does not flag it stalled, `roster.wake` refuses `owner-present` (which covers the idle watchdog, handoff wakes and rewakes), nothing disposes it (4.3). Auto-spawn does not consult it, and an explicit owner `lane-wake` is allowed |
| `left` | `present_until` cleared or elapsed | lane resumes normal dispatch |

A **shared workspace** is the lane's directory, or an optional `workRoot/shared/<name>` directory the owner and a lane both use, which the owner opens as an ordinary DSH workspace. When the owner edits a file a task has write-claimed, the existing `freshnessCheck` (sha256 against the claimed base) makes the agent see a stale base instead of silently overwriting it (`orders/API.md`, freshness rows).

### 4.3 Join protocol and care fence

1. **Join is additive.** It never changes the lane's preset, permission, model, cwd or task. Owner text enters as a normal followup, queued behind the running turn, carrying the owner source (item 6). It never cancels a turn. Interrupting a running turn is a separate explicit action, not in v1.
2. **No harness path closes a present session.** The refusal lives inside `roster.stop(id, why, {ownerOverride})` (`roster/roster.js:100`, exported on `FleetRosterService`, `roster/index.js:230`), not in the `lane-stop` op, so every caller is covered. `stop` on a lane with `present_until > now` returns the refusal `owner-present` unless `ownerOverride === true`, which only the `lane-stop` owner op sets and only when the owner passed the explicit flag himself. Housekeeping, lease loss, span-cap, budget and watchdog paths may block work and raise a flag and never dispose a present lane. Test: call `roster.stop` directly with each `why` the code uses and assert the refusal.
3. **Eligible lanes only.** Join, message and read accept only a lane-book session whose role is `lead`, `builder` or `researcher`, that is not `kind` set, and that has a live host entry. They refuse with a stable code: `not-an-agent-session` (unknown id), `lane-not-joinable` for role `checker`, for `kind` goldfish and for virtual or engine lanes (no DSH session), and `lane-not-live` for parked or stopped lanes. Reason: a Checker is a fresh, seedless, isolated session whose independence is the verdict gate (design 4.2; `checker.js` `assertFreshOptions` runs only at creation), and an owner message after creation would break it. Leg 2 also says checkers and goldfish reviewers are not participants. One test per refused kind. No path points an agent at an owner session, and the owner's own sessions and apps are never touched.
4. **Presence is stored server-side, and consulted at four places only** (Leg 2 section 13.3 defines the signal once). `present_until` is a timestamp on the lane row, not a flag that a page renews, so a crash or restart leaves nothing permanent: it simply elapses. One predicate `roster.ownerPresent(lane, at)` is read by: (1) `roster.wake`, as the new guard `owner-present` (the watchdog, the handoff wake and the rewake sweeper reach a lane only through it); (2) `roster.dispatch` and `dispatchAll`; (3) sensing's stall flags; (4) `roster.stop`. A test per hook asserts it does nothing to a present lane. Auto-spawn does not consult it (it creates new lanes and disturbs no present session); an explicit owner `lane-wake` is the owner's own act and is allowed. A suppressed handoff wake is not counted as a signal, so `dead: unacked` cannot be reached while the owner is present (built `wake.signal` counts a signal only when the wake happened); the message wakes the lane after `session-leave`.
5. **Renewal is by CLI until flag 2 is ruled.** `session-join` takes a TTL (default 30 minutes) and `dsh-fleet op session-join` again extends it; `session-leave` clears it. Page renewal needs a browser write RPC, which section 7 flag 2 does not yet allow, so before that ruling the page shows presence read-only and does not renew. After the ruling, renewal is one allowlisted call.
6. **The lane learns of presence, and the owner marker is structured.** At join the host sends one presence note through `host.send` ("the owner joined this session and may edit files in your directory; his edits are not drift"), and a leave note at `session-leave`. Auto-briefing is suppressed while present, so these notes and the owner's own messages are the only carriers. The owner marker is a structured field: `host.send(text, {from})` builds `createUserMessage` with `source: {kind: 'user', from}` (the field `host.js:user()` already builds, extended), never a text prefix. **`from` is `'owner'` only when the call arrived on an authenticated channel.** `lane-message` is a control-socket op, which is `authenticated 0` (Leg 2 section 13.5): in case W (Leg 4 G40) any lane can call it, so until G40 shows case C the source is `from: 'control-socket'` and the lane's presence note says the message came over an unauthenticated owner channel and is a request from a colleague, not owner authority. This leg never presents `lane-message` to a lane as the owner while that is so. Inbound text from any non-owner path has any literal owner-marker string stripped or escaped before it is sent. Owner messages grant no approval: no policy, Tier B, ticket or approval code path reads the owner source, and the design v1.2 section 8 decision 4 rule (relayed approvals never count) is unchanged. Tests: a lane output, tool result or relayed text containing the marker string from a non-owner path is not delivered with the owner source; a `lane-message` over the control socket is delivered with `from: 'control-socket'`; the presence note arrives at join; an owner-sourced message does not satisfy a pending policy ticket.

### 4.4 Buildable now versus not

| Piece | Today in scratch? | Notes |
|---|---|---|
| Read-only lane list, transcript tail, "Open session" link | yes | link route confirmed by probe S1 |
| `present_until`, `ownerPresent()`, suppression of dispatch, sensing and leg 2's wake paths, `roster.stop` refusal | yes | fakes cover the roster; tests include the refusals and every wake path |
| `lane-message`, `session-join`, `session-leave` owner ops and `host.send(..., {from})` with the structured source | yes | about 80 lines plus tests; the browser calling them needs the ruling in section 7 (flag 2); until then the panel shows the exact `dsh-fleet op` command and presence is CLI-renewed only |
| Typing straight into a lane session in DSH's stock chat box | unknown | probe S1. If the stock UI allows it, it works but bypasses the fence and leaves the roster's idle bookkeeping stale (`onStatus` only fires for turns the roster started). A listener on `agent/status` for fleet sessions closes the bookkeeping part |
| Shared folder under `workRoot/shared/` | yes | check ownership and mode in S1 |
| An agent joining or reading the owner's own sessions or apps | no, and refused by design | the owner invites an agent by giving it a lane and a shared folder, never the reverse |
| Nox, Duet and delegator inside a shared session | no | they have no DSH session; they appear as participants (3.4) and act through bus handoffs |
| Join or message on a Checker, goldfish or engine lane | no, refused by design | 4.3 item 3 |
| LiveShare for a remote viewer | no | needs a relay server and network |

**Probes S1 and S2 (scratch home, stub provider, no spend):** spawn a stub-route lane through the facade's `session.create`, then (S1) check in Playwright that it appears in the sidebar list, what its workspace is called, its URL route, and whether the stock chat box accepts a prompt; (S2) check what the Lead's `session-query` returns when a second, non-lane session exists in the same home.

## 5. (c) Contained Roblox Studio instances

### 5.1 Facts

| # | Fact | Label |
|---|---|---|
| F1 | Exactly one Vinegar flatpak instance runs (`flatpak ps`: instance 2729219394, `org.vinegarhq.Vinegar`). One `RobloxStudioBeta.exe` (pid 995852, about 1 GB RSS) started 04:07:54 ET today with the argument `.../data/usg/usg-desktop.rbxlx`; pid 995801 is its wineserver. No qemu process runs. I treat all of it as the owner's; it is the process set every probe must prove unchanged. | VERIFIED |
| F2 | `~/.local/bin/roblox-studio-mcp` hard-codes the prefix `.../vinegar/prefixes/studio` and enters the first flatpak instance of the Vinegar application listed by `flatpak ps` (`next(...)` in the script). The current `~/.codex/config.toml` defines it as the MCP server `roblox_studio`, so any Codex session that starts it runs it. | VERIFIED |
| F3 | Only one prefix exists (`prefixes/studio`, 2.2 GB; `versions` 1.2 GB; runner 298 MB). Disk and RAM are ample (511 GB free, 32 GB available, 32 cores). | VERIFIED |
| F4 | A second `vinegar <file>` is handed to the already-running Studio and the child exits (run `goal-studio-cli-20260927`, second pass). A second Studio in the same prefix is not possible; another prefix is required. | VERIFIED in prior evidence |
| F5 | `StudioMCP.exe` binds a WebSocket host at `ws://127.0.0.1:13469` and waits for Studio to connect (same run; its debug output). Port 13469 is not bound right now, so no MCP client is attached to the owner's Studio at the moment. | VERIFIED |
| F6 | Strings in the shipped `StudioMCP.exe` (in `version-7062aec9dbe74b14`, not run) show a host and proxy mode ("port already in use; this process is not the host"), a `list_studios` control message, `StudioInfo {studio_mcp_id, studio_name, studio_count, client_count}`, and an `MCP_PROXY_HTTP_PORT` setting. | INFERENCE from `strings` |
| F7 | I found no literal 13469 and no port flag in the binary. Whether the port is configurable is unknown. | VERIFIED absence, meaning unknown |
| F8 | Two Studios on one login revoked it on both profiles. | REPORTED |
| F9 | No MCP tool opens a place; the Studio tool list is empty until Studio connects (`tools/list` returned `[]` in the 27 Sep run). | REPORTED, partly VERIFIED there |
| F10 | `pasta` (rootless network namespace with outbound), `Xvfb`, `Xwayland`, ImageMagick `import`, `bwrap` and `socat` are installed. Unprivileged user namespaces are available (`max_user_namespaces` 255399). | VERIFIED |
| F11 | The Windows guest is not running (no qemu process, no libvirt). The cause of the 05:05 SIGTERM is unestablished. | REPORTED; absence VERIFIED |
| F12 | Wine crashes are intermittent. | REPORTED |

### 5.2 Why F5 and F6 shape the design

The Studio MCP host already fans several Studios out to several MCP clients and keeps a list. Two Studios sharing the default loopback port would land in one host, and the owner's Studio would be one of the entries an agent can pick. That is the opposite of contained. The strongest isolation that needs no root is a private network namespace per instance: each instance has its own `127.0.0.1:13469`, which the owner's Studio cannot see and which cannot see his.

### 5.3 Architecture of one contained instance

| Layer | Mechanism | Purpose |
|---|---|---|
| Wine prefix | own directory under `<studio root>/studio-<n>/prefix`; Studio binaries copied (not shared) from the owner's `versions/` (1.2 GB, no download needed) | no file overlap; owner updates cannot break instances |
| Sandbox | its own `flatpak run` instance of Vinegar with `--filesystem=<instance dir>` for that run only, started by the supervisor | a new instance, never `flatpak enter` into the owner's |
| Display | its own `Xvfb :9<n>` | no window ever appears on the owner's desktop; frames come from `import -window root`, except while `login_state` is `needs-login` (5.6, rule 7) |
| Network | `pasta -- ...` wraps the run so the instance has a private loopback and outbound access | private bridge port; Roblox login still reaches the internet |
| Lifecycle | one transient `systemd-run --user` unit per instance (`fleet-studio-<n>`) | stop means stopping that unit's cgroup and nothing else |
| MCP | an instance launcher (a new file, parameterised by instance) runs `StudioMCP.exe` inside the same namespace and flatpak instance | one MCP connection per instance |
| Binding | `studio_instances` table: `id, label, state, place_path (a local .rbxlx), game_label, team (roster domain), prefix_dir, display, unit, account_label, login_state, last_probe_at, last_exit` | each instance is tied to one game and one agent team (lanes with `domain = studio-<n>`); the place opens by passing its file path at launch, the way the owner's did |

The registry stores an account label and a login state (`unknown`, `owner-confirmed`, `revoked`), never a credential.

### 5.4 Safety rules (code and tests, not intentions)

1. The plugin signals only processes inside a unit it recorded in the registry. It never matches by name. The unit and the flatpak instance id are recorded at launch (rule 4).
2. **Never `flatpak kill org.vinegarhq.Vinegar` and never `pkill vinegar` or `wine`.** Either kills the owner's Studio. Stop is `systemctl --user stop fleet-studio-<n>`.
3. Never run, wrap or edit `~/.local/bin/roblox-studio-mcp`, and never `flatpak enter` the owner's instance (`2729219394`).
4. **Instance identity comes from the launch, never from a diff of `flatpak ps`** (a diff races with the owner restarting his Studio, whose new instance id would look like ours). The supervisor starts `flatpak run --instance-id-fd=<fd>` inside its transient unit and records the id it reads, with the unit's cgroup; any later signal must match the recorded id and cgroup. The guard compares realpaths (symlinks resolved, no relative paths, bind-mount sources resolved through `/proc/<pid>/root` and `mountinfo`) against a registry allowlist of instance directories, and refuses anything that resolves under the owner's `prefixes/studio` or whose instance id or cgroup is not in the registry; a substring match is not used. Tests feed it the owner's real command lines plus symlinked and relative variants.
5. **Baseline is taken at the start of each probe, not hard-coded.** Each probe step records the owner's Studio pids, start times and flatpak instance id at its start and compares at its end. The values seen today (pids 995852 and 995801, instance 2729219394) are evidence for the baseline in 5.1 and are not constants in code. A change in pid or start time is treated as an owner restart: the probe aborts, re-baselines, reports, and does not resume unless the owner says so. A change in the registry's own instances is a fault.
6. Agents never enter a password. Login is an owner act (5.6), and no agent injects keystrokes into the instance's display at any point.
7. **No frame capture or serving while `login_state` is `needs-login`.** Login-screen frames can show an account label or a one-time code. In that state the frame route returns `login-in-progress`, no frame is captured, and nothing login-related is written to disk or logs. Frames are held in memory only (never written to scratch) and are excluded from render packs; the render pack uses fixture frames. Test: with the registry in `needs-login`, the frame endpoint returns no bytes and the pack runner has no path that reads the instance display.
8. **Display exposure (a residual, stated against Leg 4's baseline).** Xvfb and any x11vnc run as uid 1000 on the host, so in Leg 4's case W any lane can capture or keylog the display, including the login the owner types in P5; Leg 4 holds the sudo prompt to a stricter standard against the same attacker. Each instance runs Xvfb and any x11vnc inside its own mount and network namespace with a private X socket and auth file, `-nolisten tcp`, and x11vnc on a unix socket. That removes the network and abstract-socket routes; it does not remove file access by a co-resident uid-1000 process. So P5 uses only the dedicated non-owner account already required (5.7), Quick Login if offered, and the owner's own time-limited view, and **is not run until G40 is known or the owner accepts this residual** (decision file R1).

**Risk to flag before the first probe (F2).** While a second Vinegar instance exists, the owner's `roblox-studio-mcp` wrapper may select it, because `flatpak ps` ordering is not defined, and the current Codex config starts that wrapper for any Codex session. Run the first probes when no such session is active, and tell the owner. A one-line change that makes his wrapper pick the instance whose command line names `prefixes/studio` would remove the risk; that edit is his to make or refuse.

### 5.5 Feasibility verdict

**One contained instance on this machine: conditional yes. Two or more at once: unproven. Owner-independent: no.** The three unknowns, in the order they would kill it:

1. **Does Vinegar run with a different data directory or prefix?** Its prefix path looks fixed to `$XDG_DATA_HOME/vinegar/prefixes/studio`. Whether `flatpak run --env=XDG_DATA_HOME=...` takes effect inside the sandbox is untested. Fallback: `flatpak run --command=...` into the Vinegar runtime and run the bundled Wine (`kombucha/bin/wine`) directly against our own `WINEPREFIX` with Studio's installed files. That is more manual and probably workable.
2. **Does the instance stay logged in and reach Roblox from the private namespace, with Studio's login flow working without a host browser?** The login window has historically offered a Quick Login code that the owner enters on his phone; I have not confirmed this for this Studio build. If it is absent, the only compliant route is the owner-visible view in P5, and if he declines that too, P5 and everything after it is blocked.
3. **Does Studio connect to the instance's own host on its private port, and does Wine stay up?** Crashes are intermittent (F12), so the supervisor needs a restart budget (3 in 10 minutes, then `crashed` plus an attention flag). Unsaved work is lost on a crash, so agents must save through the MCP tool on a schedule.

If unknown 1 or 3 fails, the fallback is the Windows guest. It isolates fully, but it is down, needs its own Roblox accounts per Windows user session, and is a separate decision.

### 5.6 Probe plan that cannot touch the owner's Studio

All probes run under `<scratch>/studio-probe/`, started only by us, observed from outside.

| Step | Action | Pass | Touches owner? |
|---|---|---|---|
| P0 | Read-only: record pids, `flatpak ps`, listening ports, window list, owner Studio start time | baseline recorded (done today, section 5.1) | no |
| P1 | `pasta -- sh -c '...'` runs a listener on 127.0.0.1:13469; the host `ss` must show nothing on 13469 | private loopback proven | no |
| P2 | `flatpak run --command=sh org.vinegarhq.Vinegar` as a new instance with `--filesystem=<probe dir>` and `--env=XDG_DATA_HOME=<probe dir>/data`: print the environment, `id`, `ls`. No Studio, no Wine | answers unknown 1, first half; owner pids unchanged | only through the F2 ordering |
| P3 | Same instance, bundled Wine with `WINEPREFIX=<probe dir>/prefix` running `wineboot -u` | a second wineserver with a different prefix appears and the owner's wineserver is unchanged | no |
| P4 | Copy `versions/version-7062aec9dbe74b14` into the probe prefix and register it (offline); launch Studio under `Xvfb :90` inside `pasta`, with a local place file, no login. Window presence is checked with `xwininfo`/window list, not by capturing a frame | a Studio window exists on Xvfb, nothing on the owner's desktop, `login_state` is `needs-login`, no frame captured | no |
| P5 | Owner act: complete login with a non-owner account. Owner-visible input route, chosen by the owner, with no agent input: a time-limited view he opens himself (for example his own VNC client on a loopback-only `x11vnc` bound to `:90` that the supervisor starts on request and stops at timeout, password set by him), or Quick Login if this Studio build offers it (unknown 2). If neither is available, **P5 is blocked** and the registry stays `needs-login`; the plugin does not fall back to an agent typing into the display | `login_state` becomes `owner-confirmed`; the owner confirms his own Studio is still signed in | no |
| P6 | Start `StudioMCP.exe` in the same namespace; `tools/list` and `list_studios` | exactly one studio listed, tools populated, host `ss` still shows nothing on 13469 | no |
| P7 | Kill only the probe unit; the supervisor restarts it; login survives | restart works, owner pids unchanged | no |
| P8 | Second instance with a second account, concurrently | both listed separately in their own namespaces, neither revoked | no |

P2 onward needs the owner's go (the F2 ordering risk and the first real launch). P4 needs no network if the Studio files are copied, but P5 onward does, and needs the owner physically.

### 5.7 What the owner must supply

1. A go for P2, with the F2 note understood.
2. One non-owner Roblox account per contained instance, created and held by him. Credentials never go through chat, board, logs or an agent. He performs each login himself.
3. The game or games, as local `.rbxlx` files in a repo path, one per instance. Cloud-published places are out of scope for v1 since no tool opens a place.
4. How many instances (recommend 1 until P7 passes, then 2).
5. Whether to keep the Windows guest as a fallback (a separate decision).

### 5.8 Panel and plug-in

A page `main[fleet-studios]` plus its panellist entry. One tile per instance: label, game, state (`stopped, starting, running, needs-login, crashed`), login state, MCP connected (host tool count), the team's lane principals, last crash, and a frame (PNG from the fetch route, with its age). It reuses o1's viewport tile if one exists (an image or live-view source, a state, an age); if not, the tile component in this leg is the one o1 can adopt. Start, stop and restart are owner ops (`studio-start`, `studio-stop`), shown as exact commands in v1 (browser writes need the ruling in section 7, flag 2).

### 5.9 Studio tools for the agent team

Lanes get no MCP tools today (role presets are fixed). A small broker, `studio-broker`, speaks MCP stdio to the instance's `StudioMCP.exe` and exposes an allow-listed set of tools to lanes whose domain is `studio-<n>`, the same shape as the existing facade. It cannot be built before P6 (the tool list is unknown until a Studio connects), and it changes the role matrix, so it needs Lane 6 review.

## 6. Client contract

**VERIFIED against `src/client/index.js` and `src/client/NOTES.md`.**

- The client exports only `inject` and `apply` (C4). Components never see `ctx`; they receive props. Services reach components through `ctx.slots.register({ name, key, locale, inject: () => face }, Component)`, the pattern `packages/experimental/cua/src/client/mount.ts` uses.
- The page is a `main` entry with its own key (`fleet-activity`, `fleet-studios`) and a `sidebar.panellist` entry `{ id, order, label }`. The sidebar draws a row for every panellist entry whether or not a page exists (`SidebarRoot.tsx:67-92`), and the row renders only a glyph and a label. Register both in one generator via `ctx.slots.inject` so they install and roll back together (`packages/client/AGENTS.md:150`). The glyph is inline SVG using `currentColor`, no emoji.
- Data: `inject = ['slots', 'locale', 'connection']`. `apply` creates one feed object (poller, visibility pause, backoff) inside `ctx.effect` so its timers are cleared on unload. `packages/client/AGENTS.md` rule 2 forbids `useSyncExternalStore` and manual subscribe wiring in business components, and rule 7 says a registrant-private reactive fact goes through the inject `hooks` compartment (bare observables the renderer binds to `use<Name>`). So the feed is delivered as a `hooks` compartment entry and components stay prop-only: they receive plain JSON data and callbacks and never see the observable. The cua plugin's `face` with `hooks` is the pattern.
- **Panel id to page key (unverified).** `PanelRow` calls `selectPanel(id)`, and nothing read so far shows that the `main` key `fleet-activity` is what a panel id opens. G0 therefore also proves that a `sidebar.panellist` entry with id `fleet-activity` opens `main[fleet-activity]`, that the row appears when the plugin loads and disappears when it unloads, and the same check is repeated in R2.
- **Writes in v1: none from the elevation cards.** The browser peer is owner-equivalent and `authenticated 0`; elevation is therefore app notice plus the machine-side secure prompt, and the cards are read-only. What a writable elevation panel would need is listed in Leg 4 section 8.1. No `elevate-*` op is ever on the flag 2 allowlist.
- No JSX transform. Source uses `createElement`; the bundle is built with `node build/build.mjs` and `client.js` is committed. Only `react` and DSH platform modules are external. New modules live under `src/client/face/` and are inlined.
- Copy lives in the existing `dshFleet` locale namespace, English only.
- **Build prerequisite, VERIFIED:** `finish-int/package/src/client/build/node_modules` does not exist, so both client test files fail with `Cannot find module 'react'`. The same files pass 8 of 8 in the `dsh-fleet` tree, which has the directory. The first build step is a git-ignored symlink to it (no network).

**Tokens.** Midnight Slate v1.0.0, `design/tokens.json` on ergon `origin/main` (598da57e), sha256 `c3bc149eb2ca67bf06b7f2f0693674108f4b46be4cf7dc74fbfa662354a0bc0a`.

- A vendored copy is pinned by that hash. `build/tokens.mjs` generates `src/client/face/tokens.generated.js`, which exports one CSS string of custom properties (`--fleet-color-base`, `--fleet-space-1` and so on, with the wash alpha 31/255 computed there) scoped to `[data-fleet-surface]`. It is injected as a single `<style data-plugin="@cayleb/dsh-fleet">`, added and removed in `ctx.effect`.
- Component CSS refers only to `var(--fleet-*)`. A node test scans `src/client/face/*.js` (generated file excluded) and fails on a hex colour, `rgb(`, `hsl(`, any gradient, `box-shadow`, `text-shadow`, `backdrop-filter`, a blur filter, an emoji, `transition: all`, or an infinite animation.
- Use the `it` type scale (14, 22, 12.5), radius 8 panel and 6 control, spacing 8, 12, 16, 24, 32, and 44 px minimum targets. The pack says the theme is dark-only (`DESIGN.md`). DSH has light, dark and system themes (`ui-theme`), so the panels are a self-contained dark surface. The checker must render them inside DSH light and dark and look at the seam.
- Motion: 120, 200, 250 and 350 ms budgets and one ease-out curve (`design/motion-spec.md` section 2). The pack names the curve but `tokens.json` holds no value for it. The generator defines one constant for it, and the checker verifies only one curve is used. Reduced motion turns decorative motion off and leaves state changes immediate.
- The font tokens are Segoe UI and Consolas, which Linux lacks. The generator appends system fallbacks as a build setting, not as tokens.

**The design-token and unslop conflict.** `DESIGN.md` on ergon main (v1.0.0, lines 96 and 302) bans gradients, glass, shine and drop shadows, with elevation from the colour steps only. The Ergon Lumen v2 comps use translucent planes with 14, 18 and 24 px backdrop blur and low-alpha radial washes. REPORTED in Nox filings 023306Z and 063400Z: the checker gave the re-frozen v2 set FINAL ACCEPT, the rendered grade is the owner's, and I found no pack-v2 change merged to ergon main. **Decision for this leg:** build on v1 now. Components read semantic aliases (`--fleet-surface-1/2/3`, `--fleet-border-1/2/3`) that v1 maps to `base`, `panel`, `raise` with no blur, so a later pack 2.0.0 is a token-file change that goes through the pack gate, never Lumen material hand-coded. Ruling needed from Nox: is the fleet pack v1 or v2 (section 7, flag 1).

## 7. Flags and rulings needed

| # | Flag | Recommendation |
|---|---|---|
| 1 | Fleet pack: v1 now or wait for Lumen v2 | v1 now with the alias layer; switch only on a pack 2.0.0 merge through the pack gate |
| 2 | Design 4.11 says the client is read-only. Join, message, studio start and stop are owner actions. The browser peer is owner-equivalent (section 2, decision 4) | allow a small allowlisted write RPC from the browser (`session-join/leave`, `lane-message`, `studio-start/stop`); never policy, grants, tickets or any `elevate-*` op. Needs Lane 6. Until ruled, the panel shows commands only and presence is CLI-renewed (4.3 item 5) |
| 3 | F2: the owner's `roblox-studio-mcp` wrapper may pick a contained Vinegar instance | the owner decides: tolerate the window, or change his wrapper |
| 4 | Lead and Researcher carry `session-query` with unknown scope | Lane 6 checks it in probe S2 before any shared-session claim |
| 5 | Tier B confirm path and relayed approval | Settled. Leg 4 owns the confirm-path fix and the forge-path closure (L4-A); Leg 4 section 9 is the one approval statement. The hold card shows the working command once L4-A lands and has no approve control |
| 6 | o1's page shell and status pill | o1 owns the sidebar section, `main[fleet]` and the status pill, and reads this leg's `attention` as the pill's single source; this leg adds its own `main` keys and panellist entries and exports its tile and feed modules for o1 to reuse. If o1 instead wants tabs inside one page, the page modules already export `{id, label, order, Component}` and can be listed |
| 7 | DSH-seat models cannot read images (the "image-read crash class", Nox 010400Z, and the standing "no image reads inside the app" fence) | the DSH checker grades from measurements (section 8), the owner grades the look, and an independent Claude session may read the PNG pack if the fleet routes it to one |

## 8. How a fresh checker renders and grades

The checker is a different seat from the maker, re-derives every number from `tokens.json`, and does not trust the maker's tests.

**Environment (scratch only).** A throwaway home like `scratch-tb-l2/s2home` (`DSH_HOME` and `HOME` under scratch, `--profile fleet --patch overlay.yml --port 0`), with this package loaded from the worktree. A seed script recreates the o1 incident through the real orders API and roster book (3.3), a healthy case, an empty case and an error case, each seeded in both the fixture and the non-fixture partition (3.2a). Playwright Chromium (`~/.cache/ms-playwright/chromium-1243`; `playwright-core` is present at several paths) opens the scratch URL using that scratch process's own launch token. The live profile on :3092 is never opened or written.

**Two render tiers**

- **R1, component harness:** a static page with a minimal fake context that mounts each component in every state from a JSON fixture (`?state=...`). Fast, and it covers states a live host will not produce.
- **R2, real shell:** the real DSH web UI with the plugin loaded, sidebar wide and collapsed, at 1440, 1024 and 768 px, DSH light and dark themes, at 200% zoom, with `prefers-reduced-motion` and forced-colors emulated.

**Checks that need no image reading**

- Token lint (section 6) and the unslop sign table D from `probes/team-a-pack-conformance-checklist.md`, each sign marked pass or fail.
- In-browser computed-style assertions: text contrast against the resolved background (ink at least 7:1; muted, subtle and accent at or above the floors recomputed from `tokens.json`), interactive targets at least 44 px, focus ring 2 px accent with 2 px gap, `getAnimations().length === 0` under reduced motion, no animation with infinite iterations, every transition within the four budgets, and every padding, margin and gap value on the 4 px grid.
- Accessibility tree: every interactive element has an accessible name, Tab order follows reading order, Escape closes the drawer, and state is conveyed by text and shape as well as colour.
- State coverage B: each state present with a screenshot path. A missing state is a fail.
- Behaviour: the o1 fixture produces the `order-unworked` and `lead-idle-holding` items with the right ids in both partitions; a lane state flip with no event appended appears within two poll intervals (section 2, decision 6); a refused join on a Checker, goldfish and virtual lane; stopping the host data source shows the error state with Retry and keeps the last data; an unknown session id is refused; a join followed by `lane-stop` is refused.
- Pixel sampling by script (a PNG decoder, no model look) for background and ink at known points, to confirm the rendered colours equal the tokens.

**What the owner sees.** The checker produces a render pack (every state, width and theme, plus a contact sheet); it contains fixture frames only and no captured Studio frame (5.4 rule 7). After a PASS, the scratch instance is left running on a side port for the owner to open in his browser. The live consolidated install is not touched; install happens only after the checker PASS and Nox's order, through the guarded install script. The owner grades what actually renders. Nothing is called done on measurements alone.

## 9. Ordered build list

"Scratch" means it can be built and checked today in a scratch DSH home without touching the live install.

| # | Step | Scratch today? | Depends on |
|---|---|---|---|
| 0 | Symlink `build/node_modules`, confirm client tests 8 of 8, record the baseline | yes | none |
| 1 | G0 transport spike: RPC round trip, auth refusal, one PNG, the `connection` inject name | yes | 0 |
| 2 | Tokens: vendor, hash pin, generator, lint test | yes | 0 |
| 3 | Read model (`snapshot.js`, `attention.js`) consuming Leg 2's `roster-get`, `staffing`, `watchdog.status()` and `who()` (3.3, not re-deriving), o1 incident fixture (its `order-unworked` assertion reads `staffing.episodes()`, so it needs G6 = Leg 2 C6 from the amendments branch), G1, G2, tests | yes | none (C6 from Leg 2 13.1 step 1) |
| 4 | Host half in `src/index.js` (read-only endpoints), G3, G4, tests with fakes and in the scratch host | yes | 1, 3 |
| 5 | Client feed, tile, drawer, all states, R1 harness | yes | 2, 4 |
| 6 | Activity page and panellist, R2 in the scratch shell | yes | 5 |
| 7 | Render-pack runner and measurement checks (section 8) | yes | 6 |
| 8 | Shared session v1: `present_until`, `ownerPresent()` and its four hooks (4.3 item 4), `roster.stop` refusal, ops, structured owner source (unauthenticated until G40), fence tests, probes S1 and S2 | yes | 3; ruling 2 for browser writes |
| 9 | Studio registry, supervisor with a fake launcher, guards and their tests, panel with fixture instances | yes | 2, 4 |
| 10 | Probes P1 to P3 | needs the owner's go | 9, flag 3 |
| 11 | Probes P4 to P8 | needs the owner physically (account, login, game files) | 10 |
| 12 | `studio-broker` tools for lanes | no | 11, Lane 6 |
| 13 | Participants strip, and the read-only elevation cards | yes against Leg 2's built `who()`; the cards need Leg 4's `fleetElevate.list()` or fixtures | Leg 2 built; Leg 4 core (Leg 2 section 13.1) |
| 14 | Hand the tile and feed modules to o1; coordinate the page shell | yes | o1 state |
| 15 | Pack v2 switch | no | Nox ruling, pack gate |
| 16 | Install into the consolidated profile | no | checker PASS and Nox's order; not part of this leg |

**Done-when for this leg (each demonstrable by a checker who did not build it):** the activity page renders fixture and scratch-host data in all pack states and shows the o1 failure within one poll; the shared-session fence refuses every case in 4.3, including each refused lane kind and every wake path; the Studio page renders instances from the registry with a working supervisor on a fake launcher and the guards refusing the owner's real process; probes P1 to P3 pass or name the exact blocker; the unslop and conformance tables are filed with a measured pass; and the owner has seen the rendered pages.

## 10. What I did not verify

- That a browser component can call a third-party host RPC in DSH 0.2.2-alpha.2 (G0). Source reading only.
- That lane sessions appear in, and accept prompts from, the stock session list (S1), and what `session-query` reaches (S2).
- Everything under 5.5 and 5.6: a second prefix, a different data directory, private-namespace networking with Studio, the Quick Login flow, a second login. No probe has run.
- F8 (login revocation) and F12 (Wine crashes) are the brief's statements.
- Whether the owner's Ergon v2 grade has been given. The brief's Nox filing 133300Z lists "the owner's Ergon v2 grade PASS" among things the wake-up report lacks; I did not find the owner's own words.
- Leg 2's output is consumed from the built tree (b10e35f, VERIFIED-BUILT, read and not run), as stated in 3.4.
- Whether `flatpak run --instance-id-fd` works through Vinegar's wrapper (5.4 rule 4).
- Whether a panel id opens a `main` key (section 6, G0).

## 11. Disputed

- Review point on `peer.principal`: I agree the re-check proves only "is a browser" and say so in section 2, decision 4. I keep it as defence in depth, not as a boundary; the real controls are the remote-authority gate, the narrow allowlists and the redaction.
- Review point on `lane-silent`: adopted, but the threshold is leg 2's `roster.silence-ms` (900000), which leg 2 itself calls a guess to be tuned from a week of data. This leg reuses it and carries the same caveat.
