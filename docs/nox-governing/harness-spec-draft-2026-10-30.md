# The Single Harness — design spec (DRAFT)

**Commissioned:** Cayleb, 12:53 ET, Fri 30 Oct 2026 — "don't follow the same architecture we were going with; you have a lot more room for improvement; shape the specific bots and the harness around the best methods we've learned and picked up; declutter it."
**Status:** Draft for a goldfish spec pass (fresh-context review) before any build. No code, no spend, no installs are authorised by this document.
**Sources:** XIRYN intake (`~/workspace/research_notes/xiryn-intake-2026-10-30/report.md`), Crosswalk intake (`~/workspace/research_notes/crosswalk-mcp-intake-2026-10-30/report.md`), AGENTS.md operating conventions, Nox desktop endpoint order (`~/workspace/nox-endpoint/ORDER.md`).

---

## 1. Purpose and scope

**Purpose.** Replace the current patchwork of separate agent harnesses — Outis driving Codex sessions through a browser relay, Claude sessions on claude.ai, OpenCode as a standalone lane, watch crons polling chats, a git team-board doubling as agent mail — with **one self-hosted harness on Cayleb's home desktop (outis-desktop)** that runs his own bot fleet under one control plane, one tasking discipline and one comms bus.

**In scope**

- One harness process on outis-desktop hosting every fleet bot, each bot in its own isolated workstation, each bot's model chosen per bot (Claude, ChatGPT-family, Gemini, DeepSeek, local — as the engines and API access allow).
- The fleet's work: Ergon build support, research, the Roblox and Gains God Growth lines, and liaison/monitoring toward the desks that stay external (see below).
- A management surface Cayleb runs from his phone (Telegram or equivalent).
- A native comms path between Nox and the fleet — no browser relays, no polling screenshots of chats.
- The Nox desktop endpoint's capabilities, absorbed as a module of this harness (§2.6), superseding the standalone endpoint app for Cayleb's own desktop.

**Explicitly out of scope / not absorbed**

- **Nox.** Nox runs on its own runtime and VM, with its own memory, connectors and identity. The harness is Nox's fleet, not Nox's home. Nox bosses the fleet from outside, through the bus.
- **RSI / the Grok bot.** RSI is a hosted xAI deployment on its own box (grok-bot-vm) running the live meme desk. It does not move, and the harness never gains a path to live trading. The harness holds a *liaison* function toward RSI (§2.3), nothing more.
- **ChatGPT and Claude Desktop.** These stay Cayleb's personal tools — he talks to them himself. They are not fleet control planes, and no fleet work routes through them once migration completes. (This is Cayleb's own ruling, 12:53 ET.)
- **The Ergon product.** Ergon (DeepSeek Harness + plugin) is the company product being built this weekend on the same desktop. It is a separate codebase and a separate decision. Concepts flow between the two — templates, gates, the agent-pairing pattern — code does not merge. This spec governs Cayleb's *personal* fleet infrastructure only.

## 2. Component map

### 2.1 Agent loop — recommendation: fork XIRYN as the chassis, port patterns, never wrap foreign harnesses

**Recommendation: fork, gated on the Stage 0 hands-on test (§7).** XIRYN (MIT, licence clean for fork/rename/rebrand) already contains the expensive 60 per cent of this build, in unusually modular form:

- the agent runtime pattern (durable in-process turns, per-agent instructions and memory injection, per-step context compaction);
- the Docker workstation layer (one container per agent, persistent home volume, fingerprinted image, lifecycle janitor) — the single strongest asset in the repo;
- the provider registry (11 providers, per-agent model choice, BYOK);
- the Telegram bridge (long-poll, single paired owner, no inbound surface);
- the routines scheduler and the per-turn cost ledger (the fleet's bookkeeping spine);
- the squad `delegate` primitive.

The missing 40 per cent (MCP, queues, checker machinery — §2.5, §3) is greenfield **either way**; a clean-room rebuild would spend its first weeks re-deriving the workstation and runtime layers XIRYN already proves out, for no quality gain. Clean-room is the fallback if, and only if, the Stage 0 test shows the skeleton misbehaving (delegation not swapping context, containers not isolating, turns not surviving a closed tab).

**Hard caveats, designed around rather than wished away:**

- XIRYN runs **its own agent loop** (Vercel AI SDK). It does not host Claude Code, Codex or OpenCode as engines, and subscription entitlements do not transfer: Dots/Outis and Claude-plan value stay outside; fleet bots run on API keys, which is a real cost shift (§6).
- Therefore "port the Claude harness" means **port the patterns** — skills, standing orders, session discipline, the Harness² lesson library — recast as fleet skills and templates. Running a CLI harness (e.g. OpenCode) inside a workstation as a specialist coding driver is a possible later addition, not a foundation; the spec does not depend on it.
- Pin the fork at the evaluated HEAD and stop tracking upstream casually. Upstream is 19 days old, solo-maintained and moving fast; the fork is ours from the pin onward. The MIT notice is retained, per the licence's one condition.

### 2.2 Host and runtime shape

Single Node process + one SQLite file, loopback-bound, reached from outside the desktop **only over Tailscale** — Nox's VM and Cayleb's phone are already on that network. No LAN exposure, no public exposure, ever: the upstream design has *no authentication by design* and says so; Tailscale plus loopback is the auth story, and it is only acceptable kept that way.

### 2.3 Bot roster

Bots are roles with standing orders, a chosen model, and their own workstation. Initial roster:

| Bot | Role | Notes |
|---|---|---|
| **Lead** (the Outis port) | Lane lead: takes mission orders, decomposes them into worker tasks, holds the queues, chases stalls, files the consolidated report | This is Outis's *role* re-homed as a bot on our own loop. Nothing about Outis-the-Dots-agent is load-bearing once the Lead proves out in shadow mode (§5). |
| **Builder** (×2–3) | Code and artefact work: Ergon support builds, tools, scripts, game work | One writer per file, enforced by task ownership (§3). |
| **Checker** | Fresh-context verification: runs the actual output against the order's done-when; never checks its own lane's building | A role, instantiated per verification with no builder context (§3). |
| **Researcher** | Web research, intakes, evidence packs | Fan-out reads live here; findings land as files, not chat. |
| **Ergon desk hand** | Ergon-repo and build-support chores that do not need the product codebase touched by the personal fleet | Boundary guard: personal fleet *supports*, the weekend lanes *build*. |
| **Desk liaison** | Monitoring and message-passing toward RSI and the prop desk | Read-and-relay only. No trading actions, no order placement, no path to either desk's controls. RSI stays on its box; the Money Desk lane stays Claude's until its cap resets Mon 2 Nov 01:00 ET, and stays external after. |
| **Housekeeper** | Backups, log rotation, cost-ledger summaries, lesson-library filing | Runs on routines; the fleet's janitor and bookkeeper. |

Roster is data, not code: bots are added, retired or remodelled by editing their definitions, per the template mechanism (§2.7).

### 2.4 Management surface — Telegram

Keep XIRYN's Telegram bridge as the primary surface: one paired owner (Cayleb), long-poll (no inbound ports), commands for status/stop/approve, completion notices pushed to his phone. It matches how Cayleb already manages (phone-first, tap-to-approve, one consolidated report). Cayleb's personal ChatGPT/Claude Desktop use is unaffected — Telegram is the *fleet* surface, not a replacement for his own chats.

Rules baked into the surface:

- Hard holds surface as approval requests and wait: spend beyond envelope, credential sharing/disclosure, deployments/merges beyond granted orders, public posts beyond envelopes, any live-trading-adjacent action (which the fleet should never be holding in the first place).
- Everything else arrives as finished work to grade — the silver-platter rule — not as questions.

### 2.5 Comms bus — recommendation: a self-hosted board is the fleet bus; Crosswalk is chore traffic only

The security gate, stated plainly: **Crosswalk fails the gate for sensitive traffic.** It is a weeks-old hosted service run by what appears to be a solo operator; posts are stored as readable plaintext on third-party infrastructure (Cloudflare Workers + Supabase); there is **no end-to-end encryption**; public-crosswalk content is mirrored to a public GitHub repo; no published terms, retention policy or DPA could be verified. Anything on Crosswalk must be treated as eventually public.

**Recommendation:**

1. **Primary bus: a self-hosted board on outis-desktop**, exposed as an MCP server, holding all real fleet traffic — mission orders, evidence, desk liaison notes, anything touching code, money, people or plans. Custody stays on Cayleb's hardware, reached over Tailscale only. The harness's MCP server surface (§2.6 of the build, below) and the board are the same build: one MCP endpoint where Nox posts orders, bots claim work, and results land. This is the custody-safe alternative the Crosswalk intake itself names.
2. **Crosswalk: permitted for low-sensitivity chore traffic only**, under the intake's hard conditions, if it earns its keep at all: private crosswalks only, one key per agent, read-only keys where read-only suffices, publish approval on, the account claimed by Cayleb himself, and a standing content rule — nothing sensitive ever: no credentials, no trading instructions or figures, no personal data, no unreleased product detail. First pair, if used: Nox ↔ a fleet bot, for scheduling/hand-off chores. If the self-hosted board covers the need (it should), Crosswalk is dropped entirely — one bus is the decluttered answer, two buses is the old patchwork wearing a new coat.

Delivery is pull-based in both designs (an agent sees work when it next reads); the harness's routines give the fleet a short polling cadence so this is minutes, not the current watch-cron archaeology.

### 2.6 MCP surface and the absorbed desktop endpoint

Two MCP builds, both greenfield (XIRYN ships no MCP at all):

- **MCP client (bots → tools):** bots consume MCP servers as tool groups — the self-hosted board first; Crosswalk's hosted server only under §2.5's conditions; Ergon-side servers later if the product ever wants the fleet's help.
- **MCP server (world → fleet):** the harness exposes itself over MCP so Nox tasks and reads the fleet natively: post a mission order, read lane state, pull evidence, cancel a task. This endpoint is the front door the endpoint order's "job model" described — Nox sends a task, gets output and evidence back.

**The Nox desktop endpoint is absorbed, not built twice.** Its commissioned capabilities — ping/status, command execution and file work, GUI computer use (screenshot/mouse/keyboard), complete local action log, local kill switch, token auth, Tailscale-only reach — become a **"hands" module** of this harness: a privileged workstation class on the host (outside the per-bot sandboxes, which by design cannot see the host), callable by Nox through the MCP server surface, with the endpoint order's audit log and kill switch carried over unchanged. What the absorption *drops* is the standalone app, its separate installer and its separate token store. What it must *not* drop: the kill switch is a local control Cayleb can hit without any agent's cooperation, and the action log stays local, complete and readable by him directly. (The endpoint's second life as an opt-in component of the Ergon management suite is a product-roadmap item on the Ergon side; this spec neither builds nor blocks it — the personal instance is what gets absorbed here.)

### 2.7 Templates and skills

Keep XIRYN's template *mechanism*, discard its consumer content. Our templates are written from the fleet's real roles (§2.3). The Claude-harness skill library and the Harness² per-domain lessons (coding, host-ops, research workflows, video production) are recast as fleet skills: files the bots load by domain, versioned in the fleet repo, improved through the lesson loop (§3).

## 3. Operating doctrine, baked in by construction

The failure this section exists to prevent: doctrine that lives in prompt text gets forgotten; doctrine that lives in the data model and the state machine cannot be skipped. Each rule names its mechanism.

1. **Mission orders are the only task shape.** A task is a record with: goal, context (exact files/errors), constraints and anti-goals, ownership, **done-when as a runnable check**, evidence-to-return, assumptions policy (decide and record, never stall asking), budget and stop conditions. The order form refuses submission without a done-when. *(Mechanism: order schema + validation.)*
2. **Batch size is structural.** Orders are sized in tens of minutes to a couple of hours of work, one bot owning a whole task end to end. The queue rejects micro-tasks below a floor size unless they are explicitly marked errands. *(Mechanism: order schema + Lead's decomposition rules.)*
3. **Self-claiming queues, 2–3 deep.** Every lane holds its current task plus named successors; a finished or blocked task pulls the next automatically. A lane with an empty queue is a visible defect on the board, not a silence. *(Mechanism: per-lane queue table + successor field required on every order.)*
4. **Maker–checker split.** The bot that builds a thing can never mark it done. Completion requires a Checker instance with fresh context (spawned for the verification, holding the order and the output, not the builder's thread) to **run the actual output** against the done-when and record a verdict. Fail → back to the maker with the checker's evidence; loop until pass. *(Mechanism: task state machine — `built` is not `done`; only a checker verdict transitions to `done`.)*
5. **Span of control 3–5.** The Lead runs at most five active worker lanes; more work means a sub-lead, never a wider span and never a flat swarm. *(Mechanism: lane table constraint.)*
6. **Fan out reads, serialise writes.** Read/research tasks parallelise freely; writes to any file, repo path or external state have exactly one named owner at a time. *(Mechanism: write-ownership claim on the task record.)*
7. **State on disk, sessions disposable.** Plan, progress log, task list and assumptions live in the harness's store and in workspace files — never only in a bot's context. A bot can be killed and replaced mid-mission without losing the mission. *(Mechanism: the SQLite store + per-mission files; the transcript is not memory.)*
8. **Sensing, not silence.** Lane state (working / waiting / blocked / done) is a field the lane maintains, surfaced on the board and in the Lead's report. Nox never again infers a stall from a quiet chat. *(Mechanism: state field + heartbeat timestamps; stall = state unchanged past the order's own checkpoint time.)*
9. **Harness² lesson loop.** Before substantive work in a domain, the working bot loads that domain's lessons file; after the done gate, it files one evidence-backed lesson or an explicit NO_ENTRY. **The base harness is never rewritten by the loop** — improvements land in skills, templates, lessons and bot definitions; changes to the harness core itself are proposals for Cayleb, never self-enacted. Nothing in the loop authorises spend, posts, sends or deploys. *(Mechanism: lesson files + a done-gate checklist item; the core repo is not writable by bots.)*
10. **Ambition framing at order time.** Each mission order carries the four-line framing (speed / altitude / depth / range) and, for large builds, a goldfish pass — fresh-context reviewers attack the *spec* before execution, as this draft itself will get. *(Mechanism: order template fields; goldfish is a task type.)*
11. **Scoped approvals, hard holds.** Routine approvals inside an order's envelope are decided by the Lead. Hard holds (§2.4) always stop and wait for Cayleb. *(Mechanism: policy table on the order; the harness enforces, the prompt doesn't plead.)*

## 4. Declutter list — what the patchwork drops, and why

| Dropped piece | Why |
|---|---|
| **Browser relays as agent transport** (Dots sends verified by reading screenshots of a chat; "sent to window" receipts; re-sends) | The single largest source of lost orders and false stalls this month. Replaced by the MCP bus: delivery is a record, not a hope. |
| **The 10-minute Outis/Claude watch crons** and their state files | Polling theatre that exists only because the transport was a chat window. Retired once bus delivery is proven (§5), not before. |
| **The git team-board as agent mail** | A repo is a fine home for code and a poor message bus; mail-shaped commits created the stale-promise noise Cayleb kept tripping over. Agent comms move to the board; the repo goes back to being a repo. |
| **Claude.ai browser sessions as fleet lanes** | Capped, fragile to read, and Cayleb's personal surface besides. Claude models still power bots — via API, inside the harness. |
| **Codex desktop sessions as a separate control plane** | Desktop session work folds into harness workstations. (Cayleb's own ChatGPT use is untouched; the *lane* is what closes.) |
| **The standalone Nox endpoint app** | Absorbed as the hands module (§2.6). One app fewer to install, secure and keep alive. |
| **The separate OpenCode lane** | Its workload (mechanical coding) becomes Builder bots. An in-container CLI driver may return later as a tool, not as a lane with its own comms problem. |
| **Apotheca-era and Pump Drop-era lane scaffolding** | Pump Drop is killed and its desktop footprint is already being wiped in this weekend's margin work; Apotheca is parked by Cayleb's ruling. Their sessions, watchers and queues do not migrate. |
| **XIRYN's consumer fat** (in the fork) | Vault and payment stack (largest security surface in the codebase — a fleet platform does not need agents that pay for things), the 16 consumer agent templates and preset squads as shipped content, the Cloudflare phone-takeover tunnel, the Firecrawl dependency, voice typing and notification chrome, the curl-pipe installer. Cut at fork time (Stage 1), verified by grep, not by intention. |
| **Duplicate cost/spend tracking** (per-lane tallies in chat logs) | The turns ledger is the one book of record for model spend. |

## 5. Desktop deployment and migration

### 5.1 Deployment shape on outis-desktop

- One install under a dedicated service account; Docker for workstations; data in one directory (SQLite + volumes + logs) that the Housekeeper backs up nightly to a second location on the home network/VM.
- Resource discipline: the desktop is also Cayleb's machine and, this weekend, the Ergon build's home. Workstation containers get conservative caps by default (below XIRYN's 2 CPU / 4 GB default where the host is busy), idle containers auto-stop, and the whole harness yields: Ergon weekend work has the machine's priority until Cayleb says otherwise.
- Exposure: loopback + Tailscale only (§2.2). The Telegram bridge is outbound-only. The hands module (§2.6) is the only host-privileged component; bot workstations have no host mounts.

### 5.2 Coexistence with the Ergon weekend build

The Ergon weekend build is **already running** on this desktop under the existing lanes (mission order fired 12:48 ET today; Orders 0–F). This harness build does not interrupt it:

- Stage 0 (evaluation) runs in a throwaway profile and touches nothing the Ergon lanes use.
- Stages 1–3 build in the harness's own directory and containers; the Ergon lanes keep their sessions, repos and runtimes untouched.
- The Lead bot's shadow mode (§5.3) *observes* Ergon orders; it does not task Ergon builders until migration of that lane is explicitly cut over — which, for the Ergon build itself, is **after** the weekend bar is judged, unless Cayleb orders otherwise.
- Shared-machine contention is a named risk (§6): if the desktop groans, harness stages pause. Ergon wins ties, all weekend.

### 5.3 Migration sequence — nothing shuts down until its replacement is proven

1. **Bus first.** Self-hosted board live; Nox ↔ Lead bot exchange orders and results over MCP for a probationary set of real (small) missions. *Proof:* delivery log shows every order acknowledged, worked and reported with zero browser relays. Only then do the watch crons retire.
2. **Outis's lane-lead role → the Lead bot, in shadow.** The Lead receives a copy of each new mission order; its decompositions and reports are compared against Outis's on the same orders. *Proof:* across an agreed run of orders, the Lead's plans need no more correction than Outis's, and its consolidated reports match the evidence. Then new orders go Lead-first; the Dots lane stays available (it is also Cayleb's personal ChatGPT) but stops being the fleet's spine.
3. **Builder work → Builder bots.** New build orders route to Builders; Codex-session building winds down as its in-flight work completes. *Proof:* checker-passed deliverables from Builders at parity or better on comparable orders.
4. **Claude-session lanes.** Capped until Mon 2 Nov 01:00 ET regardless. After the reset, Claude Code may join the bus as an MCP *client* for the work Cayleb still wants there (e.g. final verification seats); fleet volume does not return to browser sessions.
5. **RSI.** Unchanged until its bus leg is proven separately — and the proof burden is higher, because RSI is live-money adjacent. The liaison bot starts read-only (mirroring status), and any change to how RSI receives instructions needs Cayleb's explicit go, per-pair, under the standing RSI protocol.
6. **Endpoint.** The standalone endpoint was never installed (not install-ready at its last verdict). Nothing to migrate: the hands module is built fresh inside the harness, and the endpoint order's verification (ping, scripted GUI task, log, kill switch) becomes part of Stage 4's done-checks.

## 6. Open questions and risks — for the goldfish pass

Listed honestly; the goldfish reviewers should attack these first.

1. **The subscription-to-API cost shift is unquantified.** Bots run BYOK. Work that today rides Cayleb's ChatGPT and Claude subscriptions will bill per token. The turns ledger can *measure* it, but nobody has yet modelled a month of fleet operation. Is the convenience worth a new, variable API bill — and does a budget cap (XIRYN supports one) become a hard platform rule?
2. **It cannot absorb Nox or RSI** (intake §9). Cayleb's "all of our models and agents together" is therefore *not* literally achievable: the fleet is one harness, but Nox and RSI remain external principals on the bus, not bots in it. Is Cayleb's mental model updated to that, or does he expect a unity this design doesn't deliver?
3. **Single-host single point of failure.** Desktop off, asleep, or busy = fleet down. Cayleb's Ergon doctrine (§20) insists agents live on an always-on host so devices don't matter; this design puts his *personal* fleet on a desktop that also gets shut down, carried between networks and fought over by the Ergon build. Accepted risk, or does the fleet eventually want the same residency treatment?
4. **"Fresh context" for the Checker, inside one process.** A checker bot spawned in the same runtime, reading the same SQLite, is context-fresh but not infrastructure-independent. Is that separation strong enough for the maker–checker doctrine, or does high-stakes verification still need a genuinely external checker (Nox, or Claude post-reset)?
5. **Prompt injection is declared out of scope upstream.** Fleet bots browse and read the world from containers that have internet access and in-container sudo. The blast radius is bounded to a container — but a container holds that bot's keys-in-context, memory and queue. Does the fleet need egress rules or per-bot network policy before Researcher-class bots roam?
6. **Secrets concentration.** All provider keys, the Telegram token and (if ever enabled) vault material sit under one OS account on one machine. OS-account security is the whole story. Acceptable for a home fleet — but it raises the value of the desktop as a target, and the backup copies multiply the exposure. Where do backups live, encrypted how?
7. **Telegram as the single management channel.** One paired account controls approvals. If Cayleb's Telegram is compromised, the fleet takes orders from an attacker — mitigated only by the hard-hold list. Is a second factor (a local-only approval for the hardest holds) warranted?
8. **Queue and sensing machinery is unbuilt.** XIRYN has squads and a delegate tool, not self-claiming queues, successor items or lane state machines. §3's mechanisms are the real build cost of this project, and the intake rates them "medium" — medium has a way of becoming large. What is the smallest queue/state implementation that still makes the doctrine structural rather than decorative?
9. **Fork maintenance burden.** Pinned at a 0.6.0 HEAD from a solo maintainer on a fast-moving Next.js: security bumps become *our* job from day one. Who watches the dependency advisories — a Housekeeper routine, or Nox?
10. **Desktop contention.** Ergon build lanes, the fleet, and Cayleb's own use share one machine, whose Outis/Codex connection is exclusive and whose resources are finite (the laptop can't even be wired at the same time). Realistic bot counts may be 3–5, not a swarm. Does the roster in §2.3 survive contact with the actual hardware?
11. **Does OpenCode-in-a-container turn out to be necessary for code quality?** If Builder bots on the native loop produce noticeably worse code than the CLI harnesses did, the "later driver" becomes a second runtime to build. Stage 3's drill should include one real coding task to find out early.
12. **The Crosswalk temptation.** Once a bus exists, adding Crosswalk "just for chores" reintroduces a second bus and a plaintext-custody surface for near-zero benefit. The spec recommends dropping it if the board suffices — the goldfish pass should stress-test whether any real use case survives that recommendation.

## 7. Staged build plan — deterministic done-checks

Each
...[truncated 1534 chars]

---

## Parent note (Nox, 30 Oct 2026 12:56 ET) — a named divergence for the goldfish pass

Cayleb's 12:53 commissioning named Crosswalk as Nox's expected comms route ("the MCP server connection or whatever that was, Crosswalk I think") while simultaneously delegating the design judgement in full ("you have a lot more room for improvement... you know what to do"). This draft recommends the opposite: self-hosted board as the single bus, Crosswalk demoted or dropped. The divergence is deliberate and reasoned (Crosswalk's custody gate fails for sensitive traffic; two buses recreate the patchwork), but it is NOT settled by this draft: it goes to the goldfish spec pass as Open Question #1, and Cayleb sees it named in the design when he grades it. No build step may treat either bus answer as final until then.
