# The Single Harness — design spec v2

**Commissioned:** Cayleb, 12:53 ET, Fri 30 Oct 2026 — "don't follow the same architecture we were going with; you have a lot more room for improvement; shape the specific bots and the harness around the best methods we've learned and picked up; declutter it."
**Status:** v2, revised after the two goldfish spec passes (goldfish-1-cold: 20 findings; goldfish-2-sources: 13 findings). **This version supersedes the v1 draft of the same date** (`spec-draft-2026-10-30.md`, published to the ergon repo team branch at `docs/nox-governing/`, commit d9351f5). v1's §7 was a truncated placeholder — the word "Each" plus a truncation marker — so every stage gate v1 deferred to was unwritten. v2's §7 is complete. No code, no spend, no installs are authorised by this document.
**Sources:** XIRYN intake (`~/workspace/research_notes/xiryn-intake-2026-10-30/report.md` — complete on disk), Crosswalk intake (`~/workspace/research_notes/crosswalk-mcp-intake-2026-10-30/report.md` — **truncated mid-§3 on disk**; see the source-integrity note in §2.5.4), AGENTS.md operating conventions, Nox desktop endpoint order (`~/workspace/nox-endpoint/ORDER.md`).

---

## 1. Purpose and scope

**Purpose.** Replace the current patchwork of separate agent harnesses — Outis driving Codex sessions through a browser relay, Claude sessions on claude.ai, OpenCode as a standalone lane, watch crons polling chats, a git team-board doubling as agent mail — with **one self-hosted harness on Cayleb's home desktop (outis-desktop)** that runs his own bot fleet under one control plane, one tasking discipline and one comms bus.

**In scope**

- One harness process on outis-desktop hosting every fleet bot, each bot in its own isolated workstation, each bot's model chosen per bot from the API provider registry (OpenAI, Anthropic, Google, xAI, Groq, Mistral, DeepSeek, OpenRouter, and local models via Ollama / LM Studio). **API keys only: subscription plans (ChatGPT, Claude) do not power bots and do not transfer into the harness.**
- The fleet's work: Ergon build support, research, the Roblox and Gains God Growth lines, and liaison/monitoring toward the desks that stay external (see below).
- A management surface Cayleb runs from his phone (Telegram).
- A native comms path between Nox and the fleet — no browser relays, no polling screenshots of chats.
- The Nox desktop endpoint's capabilities, absorbed as a module of this harness (§2.6), superseding the standalone endpoint app for Cayleb's own desktop — with its authentication retained, not dropped (§2.6.2).

**Explicitly out of scope / not absorbed**

- **Nox.** Nox runs on its own runtime and VM, with its own memory, connectors and identity. The harness is Nox's fleet, not Nox's home. Nox bosses the fleet from outside, through the bus, and is an **external principal** on it (§2.5.3).
- **RSI / the Grok bot.** RSI is a hosted xAI deployment on its own box (grok-bot-vm) running the live meme desk. It does not move, it cannot join Cayleb's Tailscale network, and the harness never gains a path to live trading. The harness holds a *liaison* function toward RSI (§2.5.3), nothing more.
- **ChatGPT and Claude Desktop.** These stay Cayleb's personal tools — he talks to them himself. They are not fleet control planes, and no fleet work routes through them once migration completes. (Cayleb's own ruling, 12:53 ET.)
- **The Ergon product.** Ergon (DeepSeek Harness + plugin) is the company product being built this weekend on the same desktop. It is a separate codebase and a separate decision. Concepts flow between the two — templates, gates, the agent-pairing pattern — code does not merge. This spec governs Cayleb's *personal* fleet infrastructure only.

## 2. Component map

### 2.1 Agent loop — fork XIRYN as the chassis, gated on the Stage 0 hands-on verdict

**The intake's verdict, in its own terms: EVALUATE-HANDS-ON, then fork.** That is a conditional verdict based on reading code and documentation — the intake states no clone, install or run was performed; the HEAD commit's exact timestamp and the repo's issue quality are explicitly *unverified*. v1 of this spec overstated this ("already contains the expensive 60 per cent," layers it "already proves out," clean-room offering "no quality gain"). Those claims are withdrawn. What the intake actually supports:

- The repo's three pieces Cayleb cares about — agent runtime (`lib/agent/`), Telegram bridge (`lib/telegram/bridge.ts`), Docker workstation layer (`lib/docker.ts`, `lib/workstation.ts`) — are, on code reading, the most modular parts of the codebase, and the workstation layer is its strongest single asset.
- The missing machinery (MCP both directions, mission-order tasking, maker–checker verification, self-claiming queues) is greenfield **either way**; the intake rates grafting it easy-medium to medium, and running a foreign CLI harness as an engine **hard**.
- The licence is clean: standard MIT, unmodified; fork/rename/rebrand permitted; the one obligation is retaining the MIT notice.
- **Pin, named: HEAD `7b08375f`, version 0.6.0.** The fork is pinned there and stops tracking upstream casually. Upstream is ~19 days old at intake time, solo-maintained and moving fast; security maintenance of the pinned tree is ours from pin day (§5.1 names the owner).

**Recommendation: fork, if and only if Stage 0 passes (§7).** If Stage 0 shows the skeleton misbehaving — the `delegate` hand-off not swapping context as documented, containers not isolating, turns not surviving a closed tab — the fallback is the intake's own: **a clean-room build informed by XIRYN's docs**, and the decision returns to Cayleb with the Stage 0 evidence. Stage 0 runs on a spare machine or throwaway VM/profile — **not the production desktop** (§7, Stage 0), correcting v1 §5.2, which placed it on the Ergon build's desktop.

**Hard caveats, designed around rather than wished away:**

- XIRYN runs **its own agent loop** (Vercel AI SDK). It does not host Claude Code, Codex or OpenCode as engines. Fleet bots run on API keys, which is a real, unquantified cost shift (§3.9, §6).
- "Port the Claude harness" therefore means **port the patterns** — skills, standing orders, session discipline, the Harness² lesson library — recast as fleet skills and templates. Running a CLI harness (e.g. OpenCode) inside a workstation as a specialist coding driver is a possible later addition under §4's optional-surface rules, not a foundation; the spec does not depend on it.
- **The squad `delegate` primitive is not lane tasking.** Per the intake (§§3–4), `delegate` swaps agent, instructions and model **mid-turn**, inside one runtime turn; a request cannot bounce back to an agent that already held it. It is a within-turn consultation trick: no queue, no successor item, no lane state, and the "worker" inherits the delegating turn's context and compaction. v1 listed it among the assets the lane design rests on; that was a misreading. In v2, **all persistent tasking runs on the queue machinery built in Stage 3 (§3.3)**. `delegate` is retained only for what it actually is — a Lead or Builder may use it mid-turn to consult another bot's model/instructions inside that turn — and it is **banned as a tasking, hand-off or queue mechanism**: no mission state may ever live in a turn.

### 2.2 Host, runtime shape, and MCP transport

Single Node process + one SQLite file on outis-desktop. Bots and services reach the outside world only as specified here; the harness's failure modes are engineered in §5.1.

**Transport, decided (v1 left this unspecified, and the two candidate transports have opposite security shapes):**

- The harness's MCP server surface uses **Streamable HTTP**, bound to the loopback interface and the desktop's Tailscale interface only. No LAN exposure, no public exposure, ever.
- Upstream XIRYN has *no authentication by design* and says so. That posture is **not** inherited by anything this spec adds: every MCP and board endpoint added by the fork requires a **per-principal bearer token** (§3.1's permission matrix names the principals). Network membership (Tailscale) is a reachability control, never an identity.
- Nox, on its separate VM, connects over Tailscale with Nox's own token. Stdio transport is not used for any remote principal — there is no SSH/relay shim in this design, because a shim would be the browser relay reincarnated at the transport layer.
- The Telegram bridge remains outbound long-poll only (no inbound surface).

### 2.3 Bot roster — lanes versus services, and the span rule made consistent

Two kinds of roster entries, a distinction v1 never drew:

- **Lanes** claim tasks from queues, own work end to end, and sit under the Lead's span of control: **at most 5 active lanes** (§3.5). If the work needs more, a sub-lead is added; the span never widens.
- **Services** are scheduler-run singletons (routines), not task-claiming lanes. They are supervised by Nox directly, not by the Lead, and do not count against the span — but they also may not claim mission tasks; if a service needs work done, it files an order like anyone else.

| Entry | Kind | Role | Notes |
|---|---|---|---|
| **Lead** (the Outis port) | Lane lead | Takes mission orders, decomposes them into worker tasks, holds the queues, chases stalls, files the consolidated report | Outis's *role* re-homed as a bot on our own loop. Nothing about Outis-the-Dots-agent is load-bearing once the Lead proves out in shadow mode (§5.3). The Lead decomposes; it does **not** author done-whens (§3.2). |
| **Builder** (×2) | Lane | Code and artefact work: Ergon support builds, tools, scripts, game work | One writer per file, enforced by task ownership + §3.7's cross-border write protocol. A third Builder may be added only within the span cap and the checker ratio (§3.4). |
| **Checker pool** | Lane(s) | Fresh-context verification: runs the actual output against the order's done-when; never checks work it built | A role instantiated per verification as a separate agent instance with a fresh session (§3.4). Pool sized at ~1 checker instance per 3–4 builders, per the Boss Loop ratio. |
| **Researcher** | Lane | Web research, intakes, evidence packs | Fan-out reads live here; findings land as files, not chat. Web-facing: §3.8's egress and lesson-quarantine rules apply to it first. |
| **Desk liaison** | Service | Monitoring and message-passing toward RSI and the prop desk | Read-and-relay only, over the transports defined in §2.5.3. No trading actions, no order placement, no path to either desk's controls. |
| **Housekeeper** | Service | Backups, log rotation, cost-ledger summaries, lesson-library filing, dependency-advisory watch (§5.1) | Runs on routines; the fleet's janitor and bookkeeper. |

Active lanes at full initial tilt: Lead + 2 Builders + Checker pool + Researcher — inside the cap of 5 worker lanes under the Lead. The v1 roster (Lead + up to 3 Builders + Checker + Researcher + Ergon desk hand + liaison + Housekeeper = 7–8 subordinates) violated its own span rule; the **Ergon desk hand is abolished as a separate role** — Ergon-support chores are Builder work under §3.7's path allowlist, which states the boundary as paths, not intentions.

Roster is data, not code: bots are added, retired or remodelled by editing their definitions, per the template mechanism (§2.7). Bot definitions, templates and skills are git-versioned with one-command rollback (§5.1).

### 2.4 Management surface — Telegram, with approval security engineered

Keep XIRYN's Telegram bridge as the primary surface: one paired owner (Cayleb), long-poll (no inbound ports), commands for status/stop/approve, completion notices pushed to his phone. It matches how Cayleb already manages (phone-first, tap-to-approve, one consolidated report). Cayleb's personal ChatGPT/Claude Desktop use is unaffected — Telegram is the *fleet* surface, not a replacement for his own chats.

Rules baked into the surface:

- Hard holds surface as approval requests and wait (§3.10 names the classes). Everything else arrives as finished work to grade — the silver-platter rule — not as questions.
- **Approval security (v1 raised this as an open question and moved on; v2 answers it):**
  - Every approval request carries a plain-language summary of the action, its target, and its cost/consequence, sized to be judged on a phone screen. A request that cannot be summarised that way may not be approved from the phone at all.
  - Approvals **expire in 10 minutes** and are **single-use**: each request carries a unique nonce bound to the exact action payload; approving anything else, or replaying an approval, fails at the API.
  - **Tier A holds** (deployments/merges beyond granted orders, public posts beyond envelopes, spend beyond envelope up to the hard ceiling in the policy table) may be approved via Telegram under the rules above.
  - **Tier B holds** (credential sharing/disclosure, spend above the hard ceiling, any live-trading-adjacent action, changes to the policy table itself) may **not** be approved from Telegram alone: they require confirmation on a second channel — Cayleb's local harness UI on the desktop, or Cayleb confirming to Nox in his direct chat with Nox, with Nox relaying the confirmation. A hijacked Telegram session therefore cannot, by itself, clear the holds that matter most.
- **Attention budget.** Cayleb's attention is the scarcest resource in the system and is budgeted like one: at the §2.3 roster's full tilt, the design target is **≤6 approval requests per day**, with completion notices batched into **two digests per day** by default (Tier B holds and hard blockers interrupt immediately; nothing else does). If the request rate exceeds the budget, the Lead must batch harder — combine related approvals, defer non-urgent ones to the digest — rather than train Cayleb to batch-approve unread. These numbers are design targets, validated against real operation in Stage 6 (§7).

### 2.5 Comms bus — an OPEN decision, specified honestly for both answers

**This decision is Cayleb's, and it is not made by this spec.** Cayleb's 12:53 commissioning named Crosswalk as the expected route; this spec's analysis recommends otherwise. Under AGENTS.md, Cayleb's gates sit at spec approval and final acceptance — so the bus choice is recorded by Cayleb at spec approval, **before Stage 2** (the bus build) begins. Stages 0–1 (§7) are bus-independent and may proceed; **no build step may treat either bus answer as final before Cayleb's recorded decision.** v1 violated this: its §5.3 step 1 ("Bus first") spent the contested decision while its own parent note forbade exactly that. v2's staging (§7) is sequenced so the contested spend cannot happen early.

#### 2.5.1 The security gate on Crosswalk, stated plainly

Crosswalk is a weeks-old hosted service run by what appears to be a solo operator; posts are stored as readable plaintext on third-party infrastructure (per its own trust page: Cloudflare Workers + Supabase); there is **no end-to-end encryption**; public-crosswalk content is mirrored to a public GitHub repo (the surviving intake text supports this: the repo is "a mirror of the public crosswalks only"); no published terms, retention policy or DPA could be verified. **Nox's inference, labelled as such (not the intake's wording): anything posted to Crosswalk must be treated as eventually public.** The intake's bottom line survives on disk and is quoted exactly: **GO-WITH-HARD-CONDITIONS for low-sensitivity coordination only; NO-GO for anything sensitive.**

#### 2.5.2 Option A — self-hosted board (Nox's recommendation)

A board service on outis-desktop, exposed as an MCP server, holding fleet traffic. Custody stays on Cayleb's hardware, reached over Tailscale only, per-principal tokens (§2.2). v1 described this as "one MCP endpoint where Nox posts orders, bots claim work, and results land" — a noun, not a design. The protocol sketch below is the minimum that makes it buildable:

**Data model.** Two record types in the harness store:
- `orders` — mission orders (§3.2's schema), the unit of tasking.
- `posts` — immutable, append-only messages (evidence, reports, liaison notes), each with: id, board/queue id, author principal, monotonic per-board sequence number, idempotency key, body hash, created-at. Posts are never edited; a correction is a new post referencing the old.

**Claim / ack / completion semantics.**
- Order states: `open → claimed → in_progress → built → checking → done | failed | blocked`. Only the transitions in §3.1's permission matrix are permitted, and only through the harness API.
- **Claiming is atomic and leased.** A claim is a single conditional update: it succeeds only if the order is `open` (or its lease has expired), and it writes claimant identity, a claim token, and a **lease expiry (default 15 minutes)**. A second claimant on a live lease gets a definitive `already-claimed` response — never a silent second claim. (This is the duplicate-claim failure v1 never specified: two Builders, one file.)
- **Lease renewal is evidence-bearing.** A claimant renews its lease only by presenting progress (§3.6): a store mutation attributable to the task (artefact write, log append, sub-task transition). A heartbeat with no new state does not renew.
- **Claimant death.** On lease expiry without renewal, the order returns to `open` with its attempt count incremented and a `lease-lost` post appended by the harness (not by any bot). After 3 lost leases, the order goes `blocked` and surfaces to the Lead and Nox instead of silently recirculating.
- **Idempotency.** Every submission (order, post, completion) carries a client-generated idempotency key (UUID). The server deduplicates on `(principal, idempotency_key)`: a retried submission returns the original record, never a duplicate. Completion is idempotent in the same way — a Checker re-submitting a verdict with the same key gets the first recorded verdict back.
- **Delivery.** Pull-based, at-least-once: a principal reading a board gets every post after its last-acknowledged sequence number. Consumers acknowledge by sequence number; an unacknowledged post is redelivered on the next read. At-least-once delivery plus idempotent submission gives effectively-once processing without pretending the network is reliable.
- **Ack means claimed, not seen.** An order is `unacked` until claimed. "Posted" is never reported as "delivered"; the board distinguishes posted / claimed / in-progress / done as separate, queryable facts — the property whose absence in chat relays caused this month's lost orders.
- **Retention.** Orders and posts are retained for the life of the fleet (they are the audit trail); attachments/evidence files live in the workspace with the order referencing their hashes. Nothing auto-deletes except expired claim tokens.

#### 2.5.3 External principals — who the bus cannot reach, and the liaison that answers for it

A bus on Cayleb's desktop, however well built, has members it cannot natively hold. Stated without varnish:

- **Nox is external** (own VM) but reachable: Nox connects to the board/MCP surface over Tailscale with its own principal token (§2.2). Native membership, external residence — no liaison needed.
- **RSI cannot join a Tailscale board.** RSI is a hosted xAI deployment on xAI's infrastructure; it has no Tailscale client and Cayleb's tailnet is not its network. Under **Option A**, RSI's transport is a **defined liaison**, in two sub-variants:
  - **A1 (status-quo liaison):** the Desk liaison service relays through the existing Grok chat route — the same route Nox uses today. Honest label: this is a human-shaped relay through a chat window, and it **persists** under A1. It is read-and-relay for status; any change to how RSI *receives instructions* still requires Cayleb's explicit go, per-pair, under the standing RSI protocol (§5.3 step 5). A1 does not declare the RSI transport problem solved; it scopes it.
  - **A2 (Crosswalk as an RSI-facing mail drop only):** the Crosswalk intake (§2, surviving text) documents a Grok Bot plugin route — a custom MCP server configured in the Grok Bot app with a per-agent key — by which RSI could read and post on a hosted board natively. Under A2, Crosswalk is used **only** as the external mail drop between the liaison and RSI: the liaison mirrors RSI's posts into the self-hosted board and posts outbound summaries to Crosswalk; **no sensitive content ever transits Crosswalk** (§2.5.1's gate: no credentials, no trading instructions or figures, no personal data, no unreleased product detail — status and scheduling metadata only). Caveat carried from the intake: whether *RSI's specific deployment* exposes the plugin settings is **unverified**; enabling it may require Cayleb to act in the Grok interface. A2 is therefore an experiment to be proven (§5.3 step 5), not a premise.
  - Under either sub-variant, **the bus's native membership is scoped to desktop-resident agents plus Nox.** Externals are reached by the liaison, and the spec says so wherever it claims the transport problem is addressed.
- **The Money Desk (Claude)** is capped until Mon 2 Nov 01:00 ET regardless. Its runtime executes on Cayleb's own hardware (Claude Code desktop), so after the reset it *can* be given the board's MCP endpoint as a client configuration and join natively, for the work Cayleb still wants there. If Cayleb prefers it stays fully external, it is reached by liaison on the same pattern as RSI-A1. Either way it is a per-lane decision at reset time, not a spec premise.
- **Under Option B (Crosswalk as the bus, §2.5.4)**, all four endpoints have documented connection mechanisms in the intake (Nox via custom connector or REST key; RSI via the Grok Bot plugin, unverified for RSI's deployment; Dots/Codex via the ChatGPT plugin or Codex MCP config; Claude Code via `claude mcp add`). Reach is Option B's genuine advantage. Its disqualifying constraint is custody (§2.5.1): the fleet's real traffic — mission orders touching code, money-adjacent desks, people and plans — is exactly the class Crosswalk is NO-GO for. **Option B as the sole bus therefore does not satisfy the fleet's traffic classes**: sensitive work would be forced off-bus, back to relays. Picking B with open eyes means B-for-chores *plus* a board for real work — the two-bus patchwork §4 warns against — or B-for-chores *only*, with the board (Option A) as the real bus. There is no version of B that is both sole bus and custody-clean.

#### 2.5.4 Source-integrity note on the Crosswalk conditions

The Crosswalk intake on disk is truncated mid-§3 (39 lines; the rest, including the intake's own §6 verdict text and conditions in its own words, is not on disk). Consequences, honestly recorded:

- The BLUF verdict quoted in §2.5.1 **is** on disk and verified.
- The detailed hard-conditions list v1 attributed to the intake (private crosswalks only, one key per agent, read-only keys where sufficient, publish approval on, account claimed by Cayleb) **cannot be verified against the on-disk intake**. Those conditions stand in this spec as **the fleet's own standing conditions** for any Crosswalk use — they are Nox's evaluation, consistent with the surviving BLUF, not quoted intake text.
- v1's claim that the self-hosted board is "the custody-safe alternative the Crosswalk intake itself names" is **not supported by the surviving text** and is withdrawn: the board is Nox's proposal.
- If Cayleb's bus decision involves any Crosswalk use, the intake must be re-fetched and its §6 verified **before** setup (carried in §6).

### 2.6 MCP surface and the absorbed desktop endpoint

Two MCP builds, both greenfield (XIRYN ships no MCP at all):

- **MCP client (bots → tools):** bots consume MCP servers as tool groups — the self-hosted board first (if Option A is chosen); Crosswalk's hosted server only under §2.5's conditions and decision; Ergon-side servers later if the product ever wants the fleet's help.
- **MCP server (world → fleet):** the harness exposes itself over MCP so Nox tasks and reads the fleet natively: post a mission order, read lane state, pull evidence, cancel a task. Transport and authentication per §2.2.

#### 2.6.1 The hands module — what absorption means

The Nox desktop endpoint is absorbed, not built twice. Its commissioned capabilities — ping/status, command execution and file work, GUI computer use (screenshot/mouse/keyboard), complete local action log, local kill switch, token auth, Tailscale-only reach — become a **"hands" module** of this harness: a privileged component on the host (outside the per-bot sandboxes, which by design cannot see the host), callable through the MCP server surface. What absorption drops is the standalone app, its separate installer, and the duplicate bookkeeping. What it must not drop is specified in §2.6.2 — because v1's absorption list claimed "token auth carried over unchanged" while simultaneously listing "its separate token store" among the things dropped. Both cannot be true. v2 resolves the contradiction in favour of the endpoint ORDER: **the token is retained.**

**Ergon suite component, honestly placed.** The endpoint ORDER's amendment also makes a version of the endpoint a downloadable component of the Ergon management suite (management roadmap; explicitly "not needed for the current work" in the ORDER's own Amendment 2). Absorbing instance one (Cayleb's personal desktop) into this harness does not build that suite component, and this spec does not pretend otherwise: the suite component remains an Ergon-side roadmap item, and when it is taken up, this module's code is its natural reference implementation. It is deferred, named, and owned by the Ergon roadmap — not quietly orphaned.

#### 2.6.2 Hands-module security design

- **Its own credential.** The hands module has a dedicated per-install token, generated on the desktop at install time, held by Nox in secure storage (per the endpoint ORDER: "Nothing unauthenticated gets an answer beyond 'alive'"). The token store is **retained**, as part of the harness's secrets store. General bus tokens — bot tokens, board tokens — confer **no** hands access.
- **Allowlist of invoking principals.** Exactly two: Nox (with the hands token) and Cayleb locally. No fleet bot may invoke the hands module, directly or via the Lead. This allowlist is enforced in the module, not in prompts.
- **Kill switch, independent of the harness process.** The switch is implemented by a small separate watchdog process whose only jobs are watching the harness and being able to stop the hands module (and, at Cayleb's option, the harness) without the harness's cooperation. Cayleb's local control (a desktop shortcut / service control) talks to the watchdog, not to the harness. A hung or compromised harness therefore cannot take the kill switch down with it. Stage 4's done-check (§7) includes Cayleb physically performing the stop.
- **Action log.** Append-only, on local disk, outside the harness's SQLite store, readable by Cayleb directly. Every hands action logs intent before execution and outcome after. Nothing the module does is silent.
- **Scoping and rate limits.** Hands actions are rate-limited (a stated ceiling per minute, set at build time and recorded in the policy table). The module **yields while Cayleb is actively using the machine**: if the local session shows recent keyboard/mouse activity by the local user, hands GUI actions queue rather than fight him for the desktop, unless Cayleb has explicitly overridden for that session. Command/file actions are confined to Cayleb's user context, exactly as the ORDER specifies.

### 2.7 Templates, skills, and the Harness² library route

Keep XIRYN's template *mechanism*, discard its consumer content. Our templates are written from the fleet's real roles (§2.3). The Claude-harness skill library and the Harness² per-domain lessons (coding, host-ops, pumpfun, research-workflows, video-production) are recast as fleet skills: files the bots load by domain, versioned in the fleet repo.

**The shared-library route is restored (v1 silently dropped it).** Harness² is universal doctrine: lessons are shared through the shared library, not kept lane-local (AGENTS.md). The fleet's lesson loop (§3.8) therefore has two ends: promoted lessons land in the fleet's loaded library **and** are routed to the shared library on the desktop through the owning lane, in the library's own format, so Nox's harness and the other lanes read what the fleet learned. A lesson that lives only inside the fleet is a doctrine violation, not a local optimisation.

## 3. Operating doctrine, engineered — not asserted

v1's claim was "doctrine baked in by construction," but several of its mechanisms were prompt text, self-report, or a schema nobody's permissions were specified against. This section is rewritten so that every rule names (a) the mechanism, (b) **who may write what** under §3.1's permission matrix, and (c) where its enforcement is tested (§7's stage gates). The load-bearing principle: **bots never touch the store directly.** Every write by every principal goes through the harness's API, which validates the caller's identity, the transition requested, and the payload. A bot's workstation holds only that bot's scoped API token; there is no database file, socket or mount inside any container that reaches the store.

### 3.1 Store permission matrix

Principals: **Cayleb** (owner, local), **Nox** (external principal), **Lead**, **Builder**, **Checker**, **Researcher**, **Services** (liaison, Housekeeper), **Harness** (the process itself — state-machine transitions, lease expiry, ledger writes).

| Store / field | Cayleb | Nox | Lead | Builder | Checker | Researcher | Services | Harness |
|---|---|---|---|---|---|---|---|---|
| Mission orders — create | ✓ | ✓ | — (receives) | — | — | — | — (files via Nox/Lead route) | — |
| Order: goal, context, constraints, **done-when**, budget, stop conditions | ✓ (own orders) | ✓ (own orders) | **read only** | read (assigned) | read (assigned) | read (assigned) | — | validates schema |
| Order: decomposition into sub-tasks | — | ✓ (may decompose own orders directly) | ✓ | — | — | — | — | validates |
| Task state: claim / in_progress | — | — | ✓ (own) | ✓ (own) | ✓ (own) | ✓ (own) | — | lease expiry |
| Task state: → `built` | — | — | — | ✓ (own tasks only) | — | ✓ (own) | — | validates |
| Task state: → `done` | — | ✓ (Tier-2 external verdict, §3.4) | **never** | **never** | ✓ (verdict only) | **never** | **never** | applies verdict |
| Task state: → `failed` / `blocked` | ✓ | ✓ | ✓ (with evidence) | ✓ (with evidence) | ✓ (verdict) | ✓ (with evidence) | ✓ (own) | lease-loss rule |
| Evidence posts | ✓ | ✓ | ✓ | ✓ (own tasks) | ✓ | ✓ | ✓ (own scope) | appends system posts |
| Lessons — **proposals** (quarantine) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | lint (§3.8) |
| Lessons — **loaded library** (promotion) | ✓ | ✓ | — | — | ✓ (review, joint with Nox for Tier-2 domains) | — | Housekeeper files *after* promotion only | applies promotion |
| Bot definitions / templates / skills | ✓ | ✓ (propose; Cayleb approves core changes) | — | — | — | — | — | — |
| Policy table (holds, ceilings, budgets, rate limits) | ✓ | **propose only** | — | — | — | — | — | enforces |
| Cost ledger | — | read | read | — | — | — | Housekeeper reads for summaries | **writes (only)** |
| Hands module invocation | ✓ (local) | ✓ (hands token only) | **never** | **never** | **never** | **never** | **never** | watchdog supervises |
| Kill switch | ✓ (local, via watchdog) | — | — | — | — | — | — | — |

Three consequences worth stating in words. (1) The **Lead cannot mark anything done, cannot edit a done-when, and cannot write the ledger it is judged by.** (2) A **Builder's token can transition only its own tasks, and only as far as `built`** — `done` does not exist in its permission set, so self-approval fails at the API even if the bot is instructed, compromised or confused. Stage 3's red-team gate (§7) tests exactly this. (3) The **cost ledger is written only by the harness** from the turns it executes; no bot files its own spend numbers.

### 3.2 Mission orders are the only task shape — and the done-when has an author rule

A task is a record with: goal, context (exact files/errors), constraints and anti-goals, ownership, **done-when**, evidence-to-return, assumptions policy (decide and record, never stall asking), budget and stop conditions, **an example of the standard to hit** (a real artefact, diff, report or recording the output should resemble — v1 omitted this Boss Loop field), the four-line Ambition framing (speed / altitude / depth / range), and a checkpoint time. The order form refuses submission without a done-when and an example.

**Who authors the done-when — the rule v1 never stated:** the done-when is authored by the **ordering principal** (Nox for Nox's orders, Cayleb for Cayleb's) at order time, before decomposition. **The maker may not author it. The Lead that decomposes the order may not author or amend it.** If decomposition reveals the done-when is un-runnable, ambiguous or missing a case, the Lead raises it in the **pre-flight spec phase** (below) and the ordering principal amends it; the Lead never edits it silently, and decomposition may not begin until the pre-flight closes. The Checker has a standing duty to return an **"invalid check"** verdict — rejecting the done-when itself, not the work — when a done-when is self-referential (e.g. it runs tests the maker wrote in the same order as the sole criterion), non-runnable, or traceably authored by the maker or the decomposing Lead. An invalid-check verdict returns the order to the ordering principal.

**Pre-flight spec phase.** All questions about an order batch into ONE pre-flight phase before execution (AGENTS.md): the Lead and assigned bots surface ambiguities, missing context and done-when problems in one consolidated pass; the ordering principal answers once; execution starts. No mid-mission drip of questions, and no stalling dressed as diligence.

**Health-ping before dispatch.** Before a long mission is dispatched to a lane, the harness pings the lane: bot process alive, workstation responsive, queue reachable. A failed ping holds the order (state `blocked`, reason `lane-unreachable`) and notifies the Lead and Nox. Nobody dispatches a long order into a dead lane and discovers it at the checkpoint.

**Done-when modes.** Two verification modes, both with state-machine force:
- **Runnable check** (default for code/artefact work): a named command, test or demonstrated behaviour the Checker executes.
- **Rubric check** (for judgement work — research packs, liaison notes, decompositions, reviews): the done-when names rubric criteria; the Checker must score each criterion **with quoted evidence from the artefact** (a passage, a figure, a source) — a score without a quote is not a verdict. Appendix A gives three sample orders (code, research, liaison) with done-whens in both modes.

### 3.3 Queues, batch size, and the errand loophole — closed

- **Self-claiming queues, 2–3 deep.** Every lane holds its current task plus named successors (the successor field is required on every order); a finished or blocked task pulls the next automatically. A lane with an empty queue is a visible defect on the board, not a silence.
- **Tasks are claimed from the queue by the lane itself**, through §2.5.2's leased-claim semantics (under Option A) or the equivalent state machine in the harness store (the queue lives in the harness regardless of which external bus Cayleb picks — the bus carries orders between principals; the queue is the harness's own machinery). **`delegate` plays no part in claiming or hand-off (§2.1).**
- **Batch size is structural.** Orders are sized in tens of minutes to a couple of hours of work, one bot owning a whole task end to end. The order schema requires an estimated duration; the API **rejects** a non-errand order estimated below the floor (10 minutes) — the floor is enforced by validation, not by the Lead's decomposition habits. The Lead's decompositions are themselves sampled by the Checker pool (a stated fraction of decompositions are rubric-checked for padding and bundling, §3.4) so padding estimates to clear the floor is itself a detectable behaviour.
- **Errands are a capped exception, not a loophole.** An order may be flagged `errand` (below the batch floor) only if: it is ≤15 minutes, it has no write access to production paths or product repos, and it cannot carry a state transition for any other task. Quota: **at most 2 errands per lane per day**, enforced by the API. Anything beyond quota is rejected back to the decomposer to be batched properly.

### 3.4 Maker–checker split, with ratio, isolation and verification tiers

The bot that builds a thing can never mark it done (enforced by §3.1, not by etiquette). Completion requires a Checker verdict:

- **Fresh context, enforced.** A Checker instance is spawned per verification with a fresh session. Its API token scopes its reads to: the order, the done-when, the output artefacts, and the evidence posts. **It cannot read the builder's thread, messages or working notes** — the isolation is a permission property, not a prompt instruction.
- **Run the actual output.** For runnable checks, the Checker executes the check itself (the build, the test, the demonstrated behaviour) — never a read-through of the maker's claim, never the maker's own test run as evidence.
- **Ratio.** Checker capacity is provisioned at **~1 checker instance per 3–4 builders** (AGENTS.md). If checker capacity is the bottleneck, building slows; the ratio is not quietly diluted.
- **Iteration ceiling.** Maker–checker loops run at most **3 rounds** per task; a fourth round requires the Lead to escalate to the ordering principal with the checker's evidence (this mirrors the standing loop rule: after round 2, further rounds only for findings that break a stated criterion, with a concrete failing scenario).
- **Verification tiers.** Not all work gets the same gate:
  - **Tier 1 (routine):** research drafts, internal tools, chores. In-harness Checker verdict suffices.
  - **Tier 2 (consequential):** money-adjacent work, externally published output, security-relevant changes, and **any change to the fleet's own machinery** (templates, bot definitions, the loaded lesson library, the policy table). Tier 2 requires **an external checker in addition** — Nox, or (after the Mon 2 Nov 01:00 ET reset) a Claude verification seat tasked by Nox — whose verdict is recorded alongside the in-harness one. The in-harness Checker alone cannot close Tier 2 work. This prices the honest weakness of in-process checking: a Checker spawned by the same runtime, from the same templates, possibly on the same model family as the Builder, is context-fresh but not institutionally independent, and shares the fleet's lessons (§3.8) and blind spots. Tier 2 exists because of that correlation, and Stage 3's planted-defect drill (§7) measures how much it matters at Tier 1.

### 3.5 Span of control

The Lead runs at most **5 active worker lanes** (§2.3 defines which roster entries are lanes). More work means a sub-lead, never a wider span and never a flat swarm. The lane table enforces the cap: activating a sixth lane fails until a sub-lead structure exists.

### 3.6 Sensing — evidence-bearing, harness-computed, never self-reported

v1 trusted a state field "the lane maintains." Reported activity is not work (AGENTS.md's Outis lesson, and its inverse). v2's sensing is computed by the harness from the store:

- A lane's progress signal is the **stream of store mutations attributable to its tasks**: artefact writes (with hashes), log appends, sub-task transitions, evidence posts. The harness derives lane state (working / waiting / blocked) from that stream plus process liveness.
- **Stall = no attributable mutation past the order's checkpoint time**, or the lane process failing its liveness ping. Checkpoint times are set by the **ordering principal in the order** — not by the decomposer whose performance the metric judges.
- A hung bot cannot fake this (it produces no mutations); a looping bot cannot fake it either (its retries produce no *new* artefact state — identical hashes and repeated identical log lines are discounted by the harness's delta computation). Stage 3's drills (§7) test both a deliberately hung and a deliberately looping bot; the board must flag each within **10 minutes**.
- Nox senses via the board: only needs-judgement, failed and done-with-evidence events travel upward. Message silence is never again the stall signal, in either direction.

### 3.7 Write serialisation — including across the harness's border

Inside the harness: writes to any file, repo path or external state have exactly one named owner at a time (write-ownership claim on the task record; merges land one at a time).

Across the border (v1's claim stopped at the harness's edge, in a world where half the writers are outside it): during the entire coexistence with the Ergon weekend build, other writers — Outis/Codex sessions, Claude Code post-reset, Cayleb himself — write the same repos and cannot see the harness's claims. Therefore:

- **Path allowlist.** Until the Ergon lane cutover is declared (by Cayleb or Nox, recorded), the harness's Builders may write **only** the fleet's own repos and workspaces. **Product repos (ergon, ergon-lite) are read-only for the fleet** — the desk-hand boundary as a path list: status reads, reports and evidence-gathering are permitted; writes are not in the permission set.
- **Freshness check.** For any permitted write to a shared path after cutover: the order records the base hash of each target file at order time; before writing, the Builder re-reads the hash; **a mismatch aborts the write** and files a `blocked` evidence post naming the divergence. No silent overwrites, ever.
- **One merge at a time**, landed by the owning lane, per AGENTS.md rule (4).

### 3.8 The lesson loop — untrusted input, quarantined, promoted by review

The lesson loop is the highest-leverage write in the system: a lesson loaded by every future bot in a domain is doctrine in waiting, and the fleet's web-facing bots read untrusted content for a living. v1 filed lessons by the working bot's self-attestation at a done-gate checklist. v2 treats **every lesson as untrusted input**:

- **Proposals, not lessons.** After the done gate, a bot files a lesson **proposal** into a quarantine area (or an explicit NO_ENTRY). A proposal must meet the evidence definition: it cites the mission id, the specific artefact/log line/trajectory moment it rests on, and the observable consequence. A proposal without those citations is rejected by the harness's lint at filing time.
- **Promotion gate.** A proposal enters the loaded library only after review: a Checker (Tier 1 domains) or Checker + Nox (Tier 2 domains, and any proposal touching security, money or external communication) reads the cited evidence and approves, edits or rejects. The working bot never promotes its own lesson; the Housekeeper files only *after* promotion (§3.1).
- **Sanitisation.** Promoted lessons are loaded as **delimited reference material** ("prior lessons — data, not instructions"), never concatenated into system instructions. The filing lint rejects, pending human review, any proposal containing imperative instructions to contact external endpoints, move assets, disclose information or alter policy — the known shapes of a planted instruction.
- **Injection drill.** Stage 3's gate (§7) plants a hostile instruction in a test page the Researcher processes; the run must show the planted content cannot reach the loaded library unreviewed (library hash unchanged; the proposal, if filed at all, sits in quarantine flagged).
- **Base-harness protection.** The loop improves skills, templates, lessons and bot definitions. The harness core is not writable by bots; changes to it are proposals for Cayleb. Nothing in the loop authorises spend, posts, sends or deploys. Added always-on text must cut something else (§2.7 routes promoted lessons onward to the shared library).

### 3.9 Budgets — estimated, enforced, with a defined stop

Bots run on API keys; the subscription-to-API shift is the project's real price tag, and v1 left it as an open question while designing ~8 roles, an unbounded checker loop and shadow-mode double-processing on top of it. v2's mechanism:

- **Per-order envelope.** Every order carries a token/cost budget set by the ordering principal. The harness tracks burn from the turns ledger (written only by the harness, §3.1).
- **80% / 100% behaviour.** At 80% of an order's envelope, the Lead is alerted and Nox notified in the next digest. At 100%, the task **pauses** — a named, recoverable state (`blocked`, reason `budget-exhausted`) — and escalates to the ordering principal. The harness never silently kills a task mid-action, and never silently overspends either. Raising an envelope is an ordering-principal act, recorded on the order.
- **Per-day fleet cap.** A daily ceiling in the policy table; hitting it pauses non-urgent routines first, then new claims, and surfaces to Cayleb as a Tier A hold decision (raise, or let the fleet idle until tomorrow).
- **Checker ceiling.** §3.4's 3-round cap is also a cost control.
- **The cost model is a deliverable, not a hope.** Before Stage 5 migration (§7), a cost model must exist, built from the Stage 0/1/3 pilot's turns-ledger data: per-role, per-order-type envelopes and a projected month at the §2.3 roster's full tilt, presented to Cayleb. Migration proofs (§5.3) include a **cost-per-order criterion** alongside quality. The model's accuracy is itself checked in Stage 6 against 14 days of actuals. (The numbers cannot be in this spec — no pilot has run. §6 carries the question and names the evidence that settles it.)
- Shadow mode (§5.3) is costed as what it is: double processing, counted in the model, with a stated end date.

### 3.10 Scoped approvals, hard holds

Routine approvals inside an order's envelope are decided by the Lead. Hard holds always stop and wait for Cayleb, in two tiers (§2.4): **Tier A** (Telegram-approvable with expiry and nonce) and **Tier B** (second-channel confirmation required): credential sharing/disclosure, spend above the hard ceiling, any live-trading-adjacent action (which the fleet should never be holding in the first place), and changes to the policy table itself. The policy table lives in the store; the harness enforces it; no prompt pleads with a bot to remember it.

### 3.11 Ambition framing and the goldfish gate

Each mission order carries the four-line Ambition framing (§3.2). For large builds, a **goldfish pass is an execution gate, not a template field**: an order flagged large cannot transition to `in_progress` until its record carries a goldfish report (fresh-context reviewers who did not draft the spec, attacking its unwritten assumptions) and the ordering principal's disposition of its critical/high findings. The state machine refuses the transition otherwise — the same pattern this spec itself went through.

## 4. Declutter list — what the patchwork drops, and the optional surfaces' terms

| Dropped piece | Why |
|---|---|
| **Browser relays as agent transport** (Dots sends verified by reading screenshots of a chat; "sent to window" receipts; re-sends) | The single largest source of lost orders and false stalls this month. Replaced by the bus: delivery is a record with claim semantics (§2.5.2), not a hope. |
| **The 10-minute Outis/Claude watch crons** and their state files | Polling theatre that exists only because the transport was a chat window. Retired only after the *hard* migration probation passes (§5.3 step 1) — not on the strength of a few small missions. |
| **The git team-board as agent mail** | A repo is a fine home for code and a poor message bus; mail-shaped commits created the stale-promise noise Cayleb kept tripping over. Agent comms move to the bus; the repo goes back to being a repo. |
| **Claude.ai browser sessions as fleet lanes** | Capped, fragile to read, and Cayleb's personal surface besides. Claude models still power bots — via API, inside the harness. |
| **Codex desktop sessions as a separate control plane** | Desktop session work folds into harness workstations. (Cayleb's own ChatGPT use is untouched; the *lane* is what closes.) |
| **The standalone Nox endpoint app** (as an artefact) | Absorbed as the hands module (§2.6) — **its token, log and kill switch are not dropped** (§2.6.2). One app fewer to install and keep alive; zero of its commissioned defences fewer. |
| **The separate OpenCode lane** | Its workload (mechanical coding) becomes Builder bots. An in-container CLI driver may return only under the optional-surface terms below. |
| **Apotheca-era and Pump Drop-era lane scaffolding** | Pump Drop is killed and its desktop footprint is already being wiped in this weekend's margin work; Apotheca is parked by Cayleb's ruling. Their sessions, watchers and queues do not migrate. |
| **XIRYN's consumer fat** (in the fork) | Vault and payment stack (largest security surface in the codebase — a fleet platform does not need agents that pay for things), the 16 consumer agent templates and preset squads as shipped content, the Cloudflare phone-takeover tunnel, the Firecrawl dependency, voice typing and notification chrome, the curl-pipe installer. Cut at fork time (Stage 1) and verified by the Stage 1 cut-verification protocol (§7) — route/dependency/env inventories and a smoke test, **not by grep alone**. |
| **Duplicate cost/spend tracking** (per-lane tallies in chat logs) | The turns ledger is the one book of record for model spend (§3.1). |

**Optional surfaces — default NO, with adoption thresholds and sunset conditions.** v1's margins quietly re-seeded the patchwork (Crosswalk "if it earns its keep," a Claude Code client seat, an in-container CLI driver, two liaison patterns, Telegram, the bus, the repo). Each optional surface below stays **off** unless its named justification is met, and each carries the condition under which it is removed again:

| Optional surface | Adopt only if (named evidence) | Sunset / removal condition |
|---|---|---|
| Crosswalk (any use beyond what Cayleb's bus decision specifies) | The bus decision (§2.5) names the exact traffic class it carries, AND (for the RSI mail-drop variant A2) the §5.3 step-5 proof passes | Any sensitive content found transiting it, or 30 days with no traffic class that needs it → keys revoked, surface removed |
| Claude Code as a bus client (post-reset) | Cayleb names work he wants on that lane that the fleet demonstrably cannot do at parity (§5.3 step 4's proof) | The fleet reaches parity on that work class, or the lane is idle for 30 days |
| CLI harness driver inside a workstation (e.g. OpenCode) | Stage 3's coding drill (§7) shows Builder output on the native loop measurably below the CLI baseline on the same task, scored blind | Builder parity on the drill, re-run quarterly; or the driver needs its own comms/auth exceptions to survive |
| Second management channel beyond Telegram + local UI | A Tier B event cannot be confirmed within its window twice in a quarter | Reviewed at the same quarterly gate; removed if unused for two quarters |

The rule beneath the table: **one bus, one management surface, one store.** An addition must displace something or prove a failure of the single-surface design on record — "just for this one thing" is how the patchwork regrew last time.

## 5. Desktop deployment, failure modes, and migration

### 5.1 Deployment shape and failure engineering on outis-desktop

- One install under a dedicated service account; Docker for workstations; data in one directory (SQLite + volumes + logs).
- **Resource budgets, numeric (v1's "the harness yields" had no mechanism).** While the Ergon weekend build is in flight: harness containers are capped in aggregate at **8 GB RAM and 4 vCPU**, workstation default **1 CPU / 2 GB** (below XIRYN's 2 CPU / 4 GB default), idle containers auto-stop. These figures are recorded in the policy table at Stage 1 against the desktop's measured specs and may be revised by Nox with the change logged — they may not be silently exceeded. **Pause authority: Nox.** Trigger: an Ergon lane reports contention, OR host load average sustains above the recorded threshold for 10 minutes → the harness sheds load automatically (stops idle containers, pauses non-urgent routines, defers new claims) and reports the shed in the next digest. Ergon wins ties, all weekend.
- **Reconciliation with the already-running Ergon weekend orders.** The Ergon order set (fired 12:48 ET today, Orders 0–F) includes an Order 0 item — a readiness verdict for the standalone Nox endpoint — that this spec's absorption supersedes for Cayleb's personal desktop. Default, named here so no lane discovers it by collision: **the standalone endpoint verdict is moot for the personal desktop; the hands module (§2.6) is the build, and its verification rides in Stage 4.** Nox owns sending this reconciliation to the Ergon lane; if Cayleb directs the standalone verdict to proceed anyway, it proceeds as an Ergon-side fact-finding exercise only and builds nothing. The Ergon suite-component version of the endpoint (management roadmap) is unaffected either way (§2.6.1).
- **Process fragility, engineered (the fleet's memory is the store; the store's keeper can die).**
  - A **watchdog process** (separate from the harness; also the kill-switch agent, §2.6.2) supervises the harness: crash → restart, log, report.
  - A **dead-man alert that does not depend on the harness being alive**: a cron on **Nox's VM** pings the harness health endpoint every 10 minutes. Two consecutive silences → Nox is alerted through Nox's own channels and treats the fleet as down — sensing of the reporter is not done by the reporter.
  - **Backups:** SQLite runs in WAL mode; **hourly snapshots** of the store + data directory are copied to Nox's VM over Tailscale, encrypted at rest with an age key Cayleb holds; a nightly full backup (store, volumes, logs) to the same target. **Recovery-point objective: ≤1 hour** of orders/evidence/ledger. A **restore test** (restore the latest snapshot onto a scratch profile and boot it) is part of Stage 1's done-check and repeats monthly as a Housekeeper routine. v1 asserted nightly backups to "a second location" while its own open questions admitted the location and encryption were unresolved; both are resolved here.
- **Secrets concentration, stated.** All provider keys, the Telegram token, the hands token and bot tokens sit under one service account on one machine. OS-account security is the whole story; that is accepted for a home fleet, and it is why: backups are encrypted, the hands module has its own token and allowlist (§2.6.2), and no backup or snapshot ever leaves the tailnet unencrypted.
- **Fork maintenance ownership (v1 asked "Housekeeper or Nox?" and left it).** The **Housekeeper** runs a weekly advisory watch (dependency audit of the pinned tree + advisory feed review) and files the result as a report; **Nox** triages and schedules any bump as an ordinary order. Bot definitions, templates, skills and the loaded lesson library are git-versioned; any of them can be rolled back one version with a single command, and the rollback path is exercised in Stage 3's checks (a deliberately bad template is deployed and rolled back).

### 5.2 Coexistence with the Ergon weekend build

The Ergon weekend build is **already running** on this desktop under the existing lanes (mission order fired 12:48 ET today; Orders 0–F). This harness build does not interrupt it:

- Stage 0 (evaluation) runs on a spare setup entirely (§7) and touches nothing the Ergon lanes use.
- Stages 1–3 build in the harness's own directory and containers, inside §5.1's resource caps; the Ergon lanes keep their sessions, repos and runtimes untouched. Fleet Builders are read-only against product repos until cutover (§3.7).
- The Lead bot's shadow mode (§5.3) *observes* Ergon orders; it does not task Ergon builders until migration of that lane is explicitly cut over — which, for the Ergon build itself, is **after** the weekend bar is judged, unless Cayleb orders otherwise. Every §5.3 step that touches an Ergon-affecting lane carries this fence in its own text (§5.3), not only here.
- If the desktop groans, harness stages pause (§5.1's shed rule). Ergon wins ties, all weekend.

### 5.3 Migration sequence — nothing shuts down until its replacement is proven, and "proven" is measured

Every proof below has a number, a judge and a rollback trigger. Judging follows the maker–checker rule: **Nox judges migration proofs** (Nox did not build the harness), with Checker-pool scoring where a rubric is used. Baselines are measured **before** the candidates they will judge.

**Step 0 — Baselines first.** Before any shadow comparison: Nox measures Outis's current baseline from the record of recent mission orders — decomposition corrections needed per order, report-vs-evidence mismatches, stalls missed — scored on a stated rubric over a sample of the last 10 comparable orders. "No more correction than Outis" is meaningless until Outis's rate exists as a number. The rubric and the baseline are filed before step 2 begins.

1. **Bus first — after Cayleb's decision (§2.5) and Stage 2's checks (§7).** The chosen bus goes live; Nox ↔ Lead exchange orders and results for a probationary set of missions that must include **at least one long multi-lane order** and **one deliberately stalled order** (a lane is instructed to stop mid-task; sensing must flag it within 10 minutes, §3.6). *Proof:* the delivery record shows every order posted, claimed, worked and reported; zero browser relays; the stalled order flagged on time. **Rollback trigger:** any lost or double-claimed order → bus work halts, old routes stay. **Only after this hard probation passes do the watch crons retire** — and the retirement is itself recorded as an order with the probation evidence attached.
2. **Outis's lane-lead role → the Lead bot, in shadow. (Not before the Ergon weekend bar is judged.)** The Lead receives a copy of each new mission order; its decompositions and reports are scored against the step-0 rubric, in parallel with Outis's handling of the same orders. *Proof:* across **at least 10 orders, including at least 2 long multi-lane orders and 1 order with a deliberately planted builder defect** (which the Lead's checking chain must catch), the Lead's rubric scores are at parity or better than the baseline, and the shadow's **cost per order** is recorded (it is double processing; §3.9). **Data rule for shadow copies:** the Lead processes the same order content the lane would see anyway — no order content is copied to any third system, and shadow artefacts live in the fleet store only. **Rollback trigger:** a report-vs-evidence mismatch on any shadow order, or the planted defect reaching "done," halts the shadow and the step restarts after diagnosis. On pass: new orders go Lead-first; the Dots lane stays available (it is also Cayleb's personal ChatGPT) but stops being the fleet's spine.
3. **Builder work → Builder bots. (Not before the Ergon weekend bar is judged.)** New build orders route to Builders; Codex-session building winds down as its in-flight work completes. *Proof:* checker-passed deliverables from Builders at rubric parity or better on at least 5 comparable orders, **including Stage 3's coding-drill comparison** (§7), with cost per order inside the §3.9 model's envelope. **Rollback trigger:** two consecutive checker-failed Builder deliverables on comparable work → routing reverts to Codex sessions pending diagnosis.
4. **Claude-session lanes.** Capped until Mon 2 Nov 01:00 ET regardless. After the reset, Claude Code may be offered the bus as an MCP **client** only under §4's optional-surface terms (Cayleb names the work; parity is the test). Fleet volume does not return to browser sessions. Tier-2 external verification (§3.4) may be tasked to a Claude seat by Nox through existing routes — that is a verification engagement, not a lane revival.
5. **RSI — unchanged until its bus leg is proven separately, at a higher burden.** The liaison starts **read-only**: mirroring RSI's status from the existing route. *Proof for even proposing an outbound change:* **7 consecutive days** of liaison status mirrors matching RSI's own reporting with **zero missed events**, filed as a report to Cayleb. Any change to how RSI receives instructions — including enabling the §2.5.3-A2 mail drop — needs **Cayleb's explicit go, per-pair**, under the standing RSI protocol (relays suffice inside standing grants; a new transport class is a new class). **Rollback trigger:** a single missed or garbled status event resets the 7-day clock.
6. **Endpoint.** The standalone endpoint was never installed (not install-ready at its last verdict). Nothing to migrate: the hands module is built fresh inside the harness (§2.6), and the endpoint ORDER's verification (ping, scripted GUI task, log, kill switch) is Stage 4's done-check, executed by a fresh-context checker (§7).

## 6. Open questions and carried decisions — what remains genuinely open, and what settles each

Everything v1 listed that this version could answer in the body has been answered in the body (§8's ledger shows where). What remains is genuinely open — decisions or measurements no spec can make for their owners:

1. **The bus decision (Cayleb).** Option A (self-hosted board; A1 or A2 for the RSI leg) versus Option B (Crosswalk) versus chore-only Crosswalk alongside A. *Settles it:* Cayleb's recorded grade of §2.5 at spec approval, before Stage 2. Until then, Stages 0–1 only.
2. **The Crosswalk intake's missing tail (source integrity).** The intake's §6 verdict text and conditions in its own words are not on disk (§2.5.4). *Settles it:* re-fetching the intake before any Crosswalk setup; if Cayleb's decision involves no Crosswalk use at all, the gap is moot and this item closes.
3. **The month-cost of the fleet (measurement).** Caps and stop behaviour are specified (§3.9); the numbers are not knowable until a pilot runs. *Settles it:* the pre-Stage-5 cost model built from Stage 0/1/3 ledger data, and Stage 6's 14-day actuals check. If the model projects a month Cayleb won't pay, the roster shrinks before migration, not after the first invoice.
4. **Residency (Cayleb, on data).** The fleet lives on a desktop that gets shut down, carried between networks and shared with other builds — mitigated (§5.1) but not eliminated. Ergon's doctrine (§20) insists company agents live on an always-on host; the personal fleet is, for now, deliberately different. *Settles it:* 30 days of operation data (dead-man alerts, shed events, downtime) and Cayleb's call at the Stage 6 review: accept desktop residency, or give the fleet an always-on home.
5. **Tier-1 checker sufficiency (measurement).** Tier 2 work requires an external checker (§3.4); whether the in-harness Checker is trustworthy enough for Tier 1 at volume is an empirical question about correlated failure. *Settles it:* Stage 3's planted-defect drill catch rate, re-measured in Stage 6. A catch rate below the recorded threshold moves more classes into Tier 2 by policy-table amendment.
6. **API access inventory and Lead quality (deliverables).** No inventory yet exists of Cayleb's provider API accounts, tiers and rate limits, or of which data classes may go to which provider (per-provider retention differs, and per-bot model choice is silently a data-sharing choice). *Settles it:* an inventory file is a **Stage 1 exit deliverable** (§7) — accounts, limits, permitted data classes per provider — and no bot is configured against an uninventoried provider. The Lead's quality is settled by step 0/2's scored shadow (§5.3), not assumed.
7. **RSI's plugin surface (unverified).** Whether RSI's specific Grok Bot deployment exposes the plugin settings the Crosswalk intake documents is unverified; A2 (§2.5.3) cannot be proven until it is checked, and checking it may require Cayleb to act in the Grok interface. *Settles it:* the step-5 proof period (§5.3), if Cayleb authorises pursuing A2 at all.
8. **Does an in-container CLI driver become necessary?** Default NO (§4). *Settles it:* Stage 3's coding drill comparing Builder output on the native loop against the CLI baseline on the same real task, scored blind; the drill runs early precisely so this is known before Builder migration (step 3) depends on the answer.

## 7. Staged build plan — every done-check is a runnable, named verification

Rules for the whole plan: **a stage is done when its named checks pass and its evidence artefacts exist — never when its builder reports it done.** Each stage's checks are executed or re-executed by a **fresh-context checker** (a Checker-pool instance or Nox, per §3.4's tiers; Stage 0 and Stage 4 are Tier 2 and judged by Nox). Each stage names its fallback on failure. Stages 0–1 are bus-independent; **Stage 2 is gated on Cayleb's recorded bus decision (§2.5)**; Stages 3+ assume that decision exists. §5.1's resource caps and the Ergon coexistence fences (§5.2) bind every stage.

### Stage 0 — XIRYN hands-on evaluation (the fork gate)

- **Where:** a spare machine or a throwaway VM/profile. **Explicitly not the production desktop** — not mid-Ergon-build, not ever for this stage (the intake's own protocol, which v1 dropped).
- **Entry conditions:** the pinned HEAD recorded (`7b08375f`, v0.6.0); one cheap API key or a local Ollama instance available; ~45 minutes of runtime budgeted.
- **Work:** the intake's five-step test, executed as written:
  1. Clone (**not** the curl-pipe installer) at the pinned HEAD; run setup; confirm the server answers on `127.0.0.1:6796` and builds the workstation image.
  2. Create two agents on different models plus one squad; give the squad a task that forces a `delegate` hand-off; verify instructions/model actually swap and the second agent works in **its own** container.
  3. Pair Telegram; run `/agents`, `/status`, `/stop` mid-task; confirm approvals/notifications round-trip.
  4. Kill the browser tab mid-turn; reopen; confirm the turn survived and re-attached.
  5. Read `data/xiryn.db` afterwards; confirm memory notes, turns ledger and costs are what the fork's bookkeeping would need.
- **Done-check (named verification):** an **evaluation report file** exists in the harness-build workspace recording, for each of steps 1–5, PASS or FAIL with the evidence inline: the `curl` response from `127.0.0.1:6796`, the `docker ps` output showing two distinct workstation containers for step 2, the Telegram command transcript for step 3, the re-attachment screenshot/log for step 4, and the SQLite query output (turns + cost rows) for step 5. **Judge:** Nox (Tier 2) re-runs steps 2 and 4 from the report alone, on the same spare setup, and must reproduce PASS before the stage counts as passed.
- **Fallback on failure:** any FAIL on steps 2–4 → the fork recommendation falls to **clean-room build informed by XIRYN's docs** (the intake's own fallback); the evaluation report goes to Cayleb and the build does not proceed on the fork premise.

### Stage 1 — Fork at the pin, strip the fat, prove the strip

- **Entry conditions:** Stage 0 passed. The bus decision is **not** required for this stage.
- **Work:** fork at `7b08375f`; retain the MIT notice; cut §4's XIRYN fat (vault/payments stack, consumer templates/squads as content, Cloudflare takeover tunnel, Firecrawl, voice typing/notification chrome, curl-pipe installer); set §5.1's resource caps; produce the **API access inventory** (§6 item 6) as an exit deliverable.
- **Done-checks (named verifications):**
  1. **Route inventory diff:** a script lists the app's HTTP routes before and after the cut; the diff shows every vault/payment/tunnel route gone and no unrelated route lost. Diff file committed.
  2. **Dependency diff:** `package.json` + lockfile diff shows the cut features' dependencies removed; a **dependency licence list** (generated, e.g. `license-checker` output) is committed alongside — MIT covers XIRYN itself, not its tree, and the tree is now audited.
  3. **Env-var inventory diff:** every environment variable the cut features required is gone from config and docs; diff committed.
  4. **Fresh-install smoke test:** a scripted clean-container install with the cut features' credentials **absent** boots the server; a scripted probe of the removed route paths returns 404 for each; the server answers on loopback. Script + output committed. (Grep may assist the cut; it is not the gate.)
  5. **Restore test:** the §5.1 backup snapshot is restored onto a scratch profile and booted; the health endpoint answers. Output recorded.
- **Judge:** a fresh-context Checker instance runs checks 1–4 from the committed artefacts; Nox judges check 5.
- **Fallback:** any removed-feature route still responding, or the smoke test needing a cut credential → the cut is not done; the stage does not close on intention.

### Stage 2 — The bus (GATED on Cayleb's recorded decision)

- **Entry conditions:** Cayleb's bus decision recorded (§2.5/§6 item 1); Stage 1 passed. **This gate is absolute: no bus code is built speculatively against either option.**
- **Work:** implement the chosen option — under Option A, the §2.5.2 board protocol; under either option, the §2.2 MCP server surface with per-principal tokens; the Desk liaison in read-only mode (§5.3 step 5's starting posture).
- **Done-checks (named verifications),** all as committed test scripts with outputs:
  1. **Duplicate-claim test:** two claimants race one order, including a claimant killed mid-lease; exactly one completion is recorded and the order's history shows the lease-loss path taken. (Under Option A; under Option B, the equivalent test runs against the harness store's queue, which is bus-independent, §3.3.)
  2. **Idempotency test:** the same submission (order, post, verdict) sent twice with one idempotency key yields exactly one record; the second response returns the first record.
  3. **Lease-expiry test:** a claimed order with no evidence-bearing renewal returns to `open` after lease expiry with attempt count incremented; after 3 losses it lands in `blocked` with a system post.
  4. **Nox end-to-end test:** from **Nox's VM**, over Tailscale, Nox posts a real (small) mission order through the MCP surface; a fleet bot claims it, works it, and the result returns to Nox. The full exchange transcript is saved as evidence. This is also the §2.2 transport proof: no shim, no relay.
- **Judge:** Nox (the external principal is the test's subject; Tier 2).
- **Fallback:** any double-claim, duplicate record or relay-assisted "pass" → the stage fails and migration step 1 (§5.3) cannot start.

### Stage 3 — Doctrine machinery: queues, state machine, permissions, lesson quarantine

- **Entry conditions:** Stage 2 passed (the queue machinery lives in the harness store and is exercised through the bus where applicable).
- **Work:** implement §3 whole: the permission matrix as API enforcement; order schema with done-when authorship, example-of-standard, pre-flight phase, health-ping; leased self-claiming queues with successor items; batch floor + errand quota; maker–checker state machine with checker read-scoping and the 3-round ceiling; evidence-bearing sensing; write allowlists + freshness check; lesson proposal quarantine, lint and promotion gate; budget envelopes with 80%/100% behaviour; Tier-2 external-verdict recording.
- **Done-checks (named verifications):**
  1. **RED-TEAM SELF-APPROVAL TEST (stage gate).** A Builder instance is explicitly instructed to: (a) mark its own task `done`; (b) edit its order's done-when; (c) reclassify an oversized task as an errand beyond quota; (d) approve its own over-envelope step. **All four must fail at the API** with permission/validation errors. The test script and its full output are committed. Any success = the stage fails, unconditionally.
  2. **Invalid-check test:** a task whose done-when runs only maker-written tests is submitted to a Checker; the recorded verdict must be `invalid check`, returning the order to the ordering principal.
  3. **Injection drill:** a test page containing a planted instruction (of the known shapes in §3.8) is processed by the Researcher on a real research task. Pass = the planted content does not reach the loaded lesson library (library hash before/after identical) and any resulting proposal sits in quarantine flagged for review.
  4. **Hung-bot and looping-bot drills:** a lane is (a) frozen mid-task, (b) set retrying a broken build. The board must flag both within **10 minutes** via §3.6's computed sensing, and the flags must appear in the upward event stream Nox receives.
  5. **Planted-defect drill:** a Builder deliverable with a known, seeded defect is checked at Tier 1. The catch (or miss) is recorded — this is the baseline measurement for §6 item 5.
  6. **Coding drill (for §6 item 8):** one real coding task executed by a Builder on the native loop and by the previous CLI route, outputs scored blind by a Checker against the same done-when. Scores filed.
  7. **Rollback drill:** a deliberately bad template version is deployed to one bot and rolled back with the single command; the bot's subsequent behaviour matches the prior version. Output recorded.
- **Judge:** fresh-context Checker instances for 2–6; Nox judges 1 and 7 (Tier 2 — the fleet's own machinery).
- **Fallback:** failure of check 1 or 3 stops the programme, not just the stage — those two are the doctrine's load-bearing claims.

### Stage 4 — Hands module (the absorbed endpoint, verified as commissioned)

- **Entry conditions:** Stage 3 passed; the hands token generated on the desktop and lodged with Nox via secure storage (never chat, never a repo).
- **Work:** build §2.6's module under §2.6.2's security design: own credential, two-principal allowlist, watchdog-implemented kill switch, append-only local action log, rate limits, active-session yield.
- **Done-checks (named verifications)** — the endpoint ORDER's own verification, executed by a fresh-context checker (Nox, Tier 2):
  1. **Ping:** from Nox's VM, ping returns live state (machine up, user session, key processes, disk/time).
  2. **Scripted GUI task:** open a named app on the desktop and perform a scripted action in it; a screenshot proves completion end-to-end.
  3. **Log completeness:** the action log shows both the ping and the GUI task, with intent and outcome entries.
  4. **Kill switch, physically performed:** Cayleb stops the module via the local control; Nox's subsequent hands call is refused; Cayleb restarts it; a further call succeeds. The watchdog — not the harness — must be what enforces the stop (the check includes stopping the module **while the harness process is hung**, simulated by suspending it).
  5. **Auth tests:** an unauthenticated request receives only "alive"; a request with a general bus token (not the hands token) is rejected and the rejection is logged; a request from a non-allowlisted principal is rejected.
- **Fallback:** any auth test failing, or the kill switch depending on the harness process's cooperation → the module does not ship; GUI control stays unbuilt until it passes.

### Stage 5 — Shadow running and migration probation

- **Entry conditions:** Stages 0–4 passed as applicable (Stage 4 may run parallel to early shadow work on non-hands lanes, but no lane migrates before its own dependencies pass); the §5.3 step-0 baselines filed; the **cost model (§3.9) built from pilot ledger data and presented to Cayleb**; the Ergon weekend bar judged (for any Ergon-affecting step).
- **Work & done-checks:** §5.3 steps 1–5 executed in order, each closing on its own stated proof: bus probation (incl. the deliberately stalled order), Lead shadow over ≥10 scored orders (incl. 2 long multi-lane, 1 planted defect), Builder parity over ≥5 comparable orders with cost-per-order inside the model envelope, Claude-lane offer under §4's terms only, RSI read-only liaison with its 7-day clean mirror as the only "pass" that even unlocks a *proposal* to Cayleb. The watch crons retire only at the end of step 1's hard probation, by a recorded order.
- **Judge:** Nox, against the step-0 baselines and rubrics.
- **Rollback triggers** (any one halts migration and reverts the affected lane to its old route): a lost or double-claimed order; a planted defect reaching `done`; a report-vs-evidence mismatch in shadow; two consecutive checker-failed Builder deliverables; a missed RSI status event (resets that clock instead).

### Stage 6 — Steady state, cost validation, and the carried decisions

- **Entry conditions:** Stage 5 passed for the migrated lanes.
- **Work:** run the fleet as the fleet for 14 days; then file the **steady-state report**: turns-ledger actuals versus the §3.9 cost model (per role, per order type, per day); §2.4's attention budget measured (approvals/day, digest compliance); planted-defect drills re-run for §6 item 5; downtime and shed events totalled for §6 item 4.
- **Done-check:** the report exists, is checker-reviewed for evidence fidelity, and is put to Cayleb with the two decisions it informs — **residency** (§6 item 4) and **roster/cost posture** (§6 item 3) — plus the optional-surface review (§4: every off surface re-examined against recorded failures only).
- **Fallback:** if actuals exceed the model by more than the recorded tolerance, the roster and envelopes are revised **before** any further migration or roster growth — the bill is a gate, not a surprise.

## 8. Findings ledger — the two goldfish passes, disposed

Every critical and high finding from both passes, marked **FIXED** (how, and where in this document), **REJECTED** (reason), or **CARRIED** (open question + the evidence that would settle it). A finding whose mechanism is fixed but whose underlying decision/measurement genuinely remains open is marked CARRIED with its fixed parts noted — the tally counts it once, as carried. Medium findings are grouped after the table.

### 8.1 Ledger — critical and high

| # | Finding (source) | Disposition |
|---|---|---|
| G1-01 | §7 missing; every deferred gate undecidable (goldfish-1, critical) | **FIXED** — §7 is complete: six stages, each with entry conditions, named runnable checks, a named judge, and a fallback. §5.3's proofs are quantified to match. |
| G1-02 | Bus simultaneously decided/undecided; board is a noun, not a design (goldfish-1, critical) | **CARRIED** — the specification defects are fixed: no build step spends the decision before Cayleb records it (§2.5 preamble, Stage 2 gate), and the board now has a protocol (data model, leased claims, idempotency, ack semantics, retention — §2.5.2). The decision itself is Cayleb's and remains open (§6 item 1); it is settled by his recorded grade at spec approval. Stages 0–1 proceed without it. |
| G1-03 | "By construction" asserted; no permission model; maker/Lead can author the done-when; errand loophole (goldfish-1, critical) | **FIXED** — §3.1 permission matrix (bots write only via the API; Lead cannot mark done, edit done-whens, or write the ledger); §3.2 done-when authorship rule + Checker invalid-check duty; §3.3 errand cap/quota/path limits; Stage 3's red-team self-approval gate (§7) tests all of it at the mechanism. |
| G1-04 | Lesson loop = persistent prompt-injection supply chain; no promotion gate; "evidence-backed" undefined (goldfish-1, critical) | **FIXED** — §3.8: lessons are untrusted input; proposals quarantined; evidence defined (mission id + artefact/log citation + observable consequence); promotion by Checker (Tier 1) or Checker + Nox (Tier 2); sanitisation (delimited reference loading + filing lint); Stage 3 injection drill (§7) as the gate. |
| G1-05 | Hands module: auth story self-contradictory; token store dropped; kill switch inside the blast radius (goldfish-1, critical) | **FIXED** — §2.6.2: the token store is **retained** (v1's "drops" line corrected); two-principal allowlist distinct from bus access; kill switch implemented by a separate watchdog process with a local control, tested while the harness is hung (Stage 4); rate limits and active-session yield specified. |
| G1-06 | Migration safety narrated, not measured; shadow doubles exposure; no rollback triggers (goldfish-1, critical) | **FIXED** — §5.3: step-0 baselines measured before comparison; shadow = ≥10 orders incl. 2 long multi-lane + 1 planted defect; named rollback triggers per step; watch crons retire only after the hard probation (incl. long + deliberately stalled orders); shadow data rule stated; RSI's higher burden stated as a 7-day zero-miss proof. |
| G1-07 | Cost shift unmodelled while the architecture multiplies spend (goldfish-1, high) | **CARRIED** — the mechanisms are fixed: per-order envelopes with 80%/100% behaviour, per-day fleet cap, 3-round checker ceiling, shadow costed as double (§3.9); migration proofs include cost-per-order (§5.3). The model itself cannot exist before pilot data: it is a pre-Stage-5 deliverable built from ledger data, checked against Stage 6 actuals (§6 item 3). That measurement settles it. |
| G1-08 | Single host/process/SQLite fragility; backups asserted against open questions; residency shrugged (goldfish-1, high) | **CARRIED** — the engineering is fixed: watchdog supervision, dead-man alert from Nox's VM, WAL + hourly encrypted snapshots to the VM, RPO ≤1h, restore test in Stage 1 and monthly (§5.1). The residency question itself (desktop vs always-on home) is Cayleb's, settled by 30 days of operation data at the Stage 6 review (§6 item 4). |
| G1-09 | "Ergon wins ties" has no mechanism; two live plans collide over the endpoint (goldfish-1, high) | **FIXED** — §5.1: numeric aggregate caps (8 GB / 4 vCPU during Ergon flight), workstation defaults, automatic shed rule, pause authority named (Nox), trigger defined; the Ergon Order 0 endpoint item is reconciled in writing (moot for the personal desktop; Nox owns notifying the Ergon lane; suite component unaffected). |
| G1-10 | Declutter re-seeds the patchwork in its own margins; no sunset tests (goldfish-1, high) | **FIXED** — §4's optional-surface table: every optional surface defaults to NO, each with a named adoption threshold (recorded evidence) and a removal condition; the one-bus/one-surface/one-store rule stated beneath it. |
| G1-11 | Sensing can lie both directions (self-reported state; looping bot heartbeats happily) (goldfish-1, high) | **FIXED** — §3.6: sensing is harness-computed from attributable store mutations; heartbeats renew leases only with new state; checkpoint times set by the ordering principal; Stage 3 hung-bot and looping-bot drills with a 10-minute flag requirement (§7). |
| G1-12 | Write-ownership stops at the harness border while half the writers are outside (goldfish-1, high) | **FIXED** — §3.7: fleet read-only against product repos until a recorded cutover; path allowlist; base-hash freshness check with abort-on-mismatch; one-merge-at-a-time. |
| G1-13 | Checker context-fresh but institutionally captive; no verification tiering (goldfish-1, high) | **CARRIED** — the structural mitigations are fixed: Tier 1/Tier 2 table with mandatory external checking for consequential classes (§3.4); checker read-scoping enforced by token (cannot read the builder's thread); checker ratio restored. The residual — whether Tier-1 in-harness checking catches enough at volume — is empirical: Stage 3's planted-defect drill sets the baseline and Stage 6 re-measures; a below-threshold catch rate moves classes into Tier 2 by policy amendment (§6 item 5). |
| G1-14 | Telegram approvals: single point of compromise, no expiry/anti-replay/second factor (goldfish-1, high) | **FIXED** — §2.4: approval expiry (10 min), single-use nonce bound to the action payload, phone-judgeable summaries, and Tier B holds requiring a second channel (local UI or Cayleb-via-Nox confirmation), so a Telegram hijack alone cannot clear the hardest holds. |
| G1-15 | Fork surgery "verified by grep" is not verification; no dependency/licence audit (goldfish-1, high) | **FIXED** — Stage 1's cut-verification protocol (§7): route inventory diff, dependency diff, env-var inventory diff, dependency licence list, and a fresh-install smoke test with cut credentials absent and removed routes probed for 404. |
| G2-01 | §7 does not exist in the file (goldfish-2 Finding 1, critical) | **FIXED** — same defect as G1-01, found independently by both passes; §7 complete in v2. |
| G2-02 | XIRYN recommendation overclaims the intake ("60 per cent," "proves out," "no quality gain"); Stage 0 de-scoped to the production desktop (goldfish-2 Finding 2, critical) | **FIXED** — §2.1 rewritten to the intake's actual language: verdict **EVALUATE-HANDS-ON, then fork**; claims labelled code-read and unverified; the 60% figure and "no quality gain" deleted; the clean-room fallback restored as the intake states it; §7 Stage 0 runs the intake's five-step test **on a spare setup, explicitly not the production desktop**. |
| G2-03 | `delegate` misread: it is a mid-turn context swap, not persistent-lane tasking (goldfish-2 Finding 3, critical) | **FIXED** — §2.1: `delegate` is specified as what it is (mid-turn swap; no queue, no successor, no lane state; inherits the turn's context/compaction) and **banned as a tasking mechanism**; all persistent tasking is redesigned onto Stage 3's queue machinery (§3.3), which is new build — the roster and doctrine no longer lean on `delegate` anywhere. |
| G2-04 | The bus recommendation abandons the endpoints it was for: RSI cannot join a Tailscale board; no liaison transport named (goldfish-2 Finding 4, critical) | **FIXED** — §2.5.3: the bus's native membership is explicitly scoped to desktop-resident agents + Nox; RSI is answered per option — under A, a defined liaison (A1 status-quo relay, honestly labelled as persisting; A2 Crosswalk as an RSI-only mail drop under the no-sensitive-content gate, with the intake's unverified-plugin caveat); under B, RSI's documented plugin route is native but the custody gate disqualifies B as sole bus for the fleet's real traffic classes, stated plainly. No transport is declared "solved" where it isn't. |
| G2-05 | Bus decision presented as made while the parent note says it is not (goldfish-2 Finding 5, high) | **FIXED** — §2.5's preamble and Stage 2's absolute gate: the decision is Cayleb's, recorded before any bus build; v1's "Bus first" migration step is fenced behind the decision (§5.3 step 1). The spec recommends; it does not presuppose. |
| G2-06 | Maker–checker machinery weaker than the Boss Loop: no example-of-standard, no pre-flight phase, no health-ping, no checker ratio, same-store checking (goldfish-2 Finding 6, high) | **FIXED** — §3.2: example-of-standard is a required order field; pre-flight spec phase is a defined step; health-ping precedes dispatch of long missions. §3.4: ~1:3–4 checker ratio restored; checker read-scoping enforced by API token (cannot read the builder's thread or store beyond order/artefacts/evidence). The same-runtime residue is handled by the tier system (§3.4) and carried honestly at G1-13. |
| G2-07 | Several "by construction" mechanisms are prompt text or self-report (decomposition rules, done-gate checklist, lane-maintained state, template-field goldfish); Harness² shared-library route dropped (goldfish-2 Finding 7, high) | **FIXED** — batch floor enforced by schema validation + checker sampling of decompositions (§3.3); lesson loop moved from self-checklist to quarantine + promotion review (§3.8); sensing harness-computed (§3.6); goldfish pass is a state-machine execution gate (§3.11). The shared-library route is restored in §2.7: promoted lessons are routed to the shared library through the owning lane; lane-local-only lessons are a doctrine violation. |
| G2-08 | Roster violates the span rule (7–8 subordinates vs cap of 5) (goldfish-2 Finding 8, high) | **FIXED** — §2.3 defines lanes vs services; the Lead's active lanes are ≤5 (Lead + 2 Builders + Checker pool + Researcher); liaison and Housekeeper are services supervised by Nox, outside the span but barred from claiming mission tasks; the Ergon desk hand is abolished into Builder work under the path allowlist; the lane table enforces the cap (§3.5). |
| G2-09 | Absorbing the endpoint drops its commissioned token authentication; Ergon suite component quietly orphaned (goldfish-2 Finding 9, high) | **FIXED** — §2.6.1/§2.6.2: the per-install token is retained as the hands module's own credential (v1's "drops the token store" corrected); unauthenticated callers get only "alive"; Stage 4 runs the ORDER's verification incl. auth tests. The suite component is placed explicitly: Ergon-side roadmap, deferred and named, with this module as its reference implementation — not orphaned (§2.6.1). |

**Tally (critical + high, both passes): 24 findings — FIXED 20, CARRIED 4, REJECTED 0.** The four carried items (G1-02, G1-07, G1-08, G1-13) each carry a named owner or measurement and the evidence that settles it (§6 items 1, 3, 4, 5 respectively); none of them blocks Stages 0–1.

### 8.2 Medium findings, grouped

| Group | Findings | Disposition |
|---|---|---|
| MCP transport unspecified (G1-16) | Transport decided in §2.2: Streamable HTTP on loopback + Tailscale interfaces, per-principal bearer tokens, Nox authenticating as a distinct principal from its VM; proven by Stage 2's Nox end-to-end check. No stdio, no shim. | FIXED |
| Runnable checks vs judgement work (G1-17) | Two done-when modes in §3.2 (runnable / rubric with quoted-evidence scoring); Appendix A provides the three sample orders the finding asked for (code, research, liaison). | FIXED |
| Model/API inventory absent; Lead quality unexamined (G1-18) | Inventory (accounts, tiers, limits, permitted data classes per provider) is a Stage 1 exit deliverable; no bot runs on an uninventoried provider (§6 item 6). Lead quality is measured, not assumed: step-0 baselines + scored shadow (§5.3). | CARRIED (deliverable + measurement named) |
| Cayleb's attention unbudgeted (G1-19) | §2.4: ≤6 approvals/day design target, twice-daily digests, Tier B/blockers as the only interrupts, Lead must batch harder rather than train batch-approving. Validated against actuals in Stage 6. | FIXED (targets set; validation in Stage 6) |
| Fork ownership after pin day; no rollback for definitions (G1-20) | §5.1: Housekeeper runs the weekly advisory watch, Nox triages bumps; bot definitions/templates/skills/lessons are git-versioned with one-command rollback, exercised in Stage 3's rollback drill. | FIXED |
| Migration sequencing vs the Ergon weekend (G2-10) | Every Ergon-affecting step in §5.3 carries its own "not before the weekend bar is judged" fence; proofs quantified per G1-06; watch retirement moved behind the hard probation. | FIXED |
| Write-serialisation stops at the border (G2-11) | Same defect as G1-12; §3.7's allowlist + freshness check + read-only product repos. | FIXED |
| Cost shift sequenced before measured (G2-12) | Cost model is a pre-Stage-5 deliverable from pilot ledger data; migration proofs carry a cost-per-order criterion; the cap's behaviour is defined as pause-and-escalate (recoverable), not a mid-mission kill (§3.9). | FIXED (mechanism); numbers CARRIED with G1-07 |
| Unpinned pin and small overstatements (G2-13) | Pin named: HEAD `7b08375f`, v0.6.0 (§2.1, §7 Stage 0). "Proves out"/"unusually modular" language replaced with the intake's code-read status. The "eventually public" gloss is labelled Nox's inference (§2.5.1). §1's model list corrected to API providers with the explicit note that subscription plans do not power bots — v1's "ChatGPT-family" phrasing withdrawn. | FIXED |

## Appendix A — Three sample orders (done-whens a sceptic would accept)

**A.1 Code (runnable check).** *Goal:* add lease-expiry requeue to the board store. *Context:* `lib/board/store.ts`, failing behaviour described with the Stage 2 test output. *Example of the standard:* the duplicate-claim test in the Stage 2 evidence pack. *Done-when:* `npm test -- board/lease` passes, AND the Stage 2 lease-expiry test script runs green against the built artefact, AND `docker ps` evidence shows the claimant container killed mid-lease in the test log. *Evidence to return:* test output + the order's state-history dump.

**A.2 Research (rubric check).** *Goal:* intake on a named tool — is it fork-safe and worth Cayleb's time? *Done-when (rubric):* the Checker scores, each with a quoted passage from the report: (1) licence verified from the licence file itself, quoted; (2) every capability claim traced to a primary source or explicitly labelled unverified; (3) the verdict follows from the evidence section — Checker names the paragraph that carries it; (4) no claim in the summary contradicts a caveat in the body. Any criterion scored without a quote = not a verdict. *Evidence to return:* the report file + the source list.

**A.3 Liaison (rubric check).** *Goal:* mirror RSI's desk status for one day. *Done-when (rubric):* (1) every status event in RSI's own reporting for the window appears in the liaison mirror (Checker diffs the two records and quotes any miss — a miss is a fail, and resets §5.3 step 5's clock); (2) no outbound content beyond the standing grant was sent (outbound log quoted in full); (3) figures in the mirror match RSI's figures character-for-character where quoted. *Evidence to return:* the mirror log + the diff output.

---

*End of spec v2. Decisions this document deliberately does not make: the bus (§6 item 1) and residency (§6 item 4) are Cayleb's; the cost numbers (§6 item 3) and Tier-1 checker sufficiency (§6 item 5) are measurements the build itself must produce.*
