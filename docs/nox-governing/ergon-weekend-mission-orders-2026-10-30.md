# Ergon weekend completion — mission orders (staged by Nox, Fri 30 Oct 2026)

Fire condition: Cayleb's home desktop (outis-desktop) reconnects AND Codex task access is live. Every order pins its environment: **outis-desktop local, never cloud**. One order per cluster, fire-and-forget; leads pull successors without waiting on Nox. Cayleb's done-bar (07:03 ET 30 Oct): installed and set up in any company, end-to-end tested multiple times, code-reviewed to the fucking moon — graded line by line in the final done-report. Plan: `ergon-weekend-build-plan-2026-10-30.md`. Doctrine: `ergon-app-doctrine-2026-10-30.md`. Design source of truth: `ergon-structure-dsh-2026-10-29.md`.

Fleet: Terra lanes at extra-high + Claude Code sessions per task (Cayleb's 29 Oct ruling). Claude's meter resets Mon 2 Nov 01:00 ET — Claude takes the final verification seat from then, not before. Goldfish spec pass runs on Orders C and D before execution (large builds).

---

## Order 0 — Desktop moment bundle

**Lead:** Outis (Dots, on outis-desktop) · **Finish line:** desktop verified live for lane work · **Report:** folded into Order A's opening report.

**Goal:** The desktop is a working build host: Codex task access enabled if still outstanding, the Nox desktop endpoint's install-readiness re-verdict run against its gates, and the swap state recorded (WKS-09 drops by the exclusive-connection rule — expected and fine on a weekend).

**Context:** Endpoint order `~/workspace/nox-endpoint/ORDER.md`; swap checklist `~/workspace/nox-endpoint/SWAP-MOMENT-CHECKLIST-2026-10-29.md`; last endpoint verdict 30 Oct 05:42 NOT INSTALL-READY (GUI/containment, enrolment/revocation, identity, installer/kill-UI gates open); nothing installed yet.

**Constraints:** No endpoint install unless the re-verdict passes its own gates. No changes to WKS-09. Cayleb's only acts are the ones he has already accepted (connect desktop; any write-only secret fields).

**Done when:** task access proven by a trivial delegated task completing on the desktop; endpoint verdict re-issued in writing (READY or the specific gates still open).

**Successor:** Order A.

---

## Order A — One-project merge and repo consolidation

**Lead:** Terra lane (fresh session) · **Finish line:** both repos consolidated on one main · **Report:** one consolidated report at the finish line.

**Goal:** Ergon is one project. `ergon-lite` folds into `ergon`; one main branch carries the whole product; the warehouse board is an app inside it, installable on its own; repo descriptions, README and releases describe the current design; nothing in the product layer is named around RHL.

**Context:** ergon main head f906d5659f8b96c4e061d9b3b7dfdc0d942ae451 (PR #6 merged). Open PR #5 (OpenCode 2.0 draft) — ruling on record: audit against the DSH design, port anything harness-agnostic still needed, close as superseded with reasoning on the PR, preserve the full patch in the report. Cayleb's merge ruling 30 Oct 06:41 supersedes the 29 Oct two-piece structure. Naming audit included.

**Constraints + anti-goals:** Pull main immediately before merging; never force-push; no branch deletes via the connector (it cannot — do not attempt). The board's local seed data (Supabase "Ergon" project, schema `ergon_warehouse`) is never committed to a repo. One writer per file; the merge is coupled work — one session, sequential.

**Done when:** a fresh clone of ergon main contains the full product incl. the board app; ergon-lite's README points at ergon and its content is fully accounted for in the merge report (file-by-file disposition); PR #5 closed with the superseded reasoning or ported with its patch preserved; test suite green on the merged tree (see Order B for the X-1 caveat — if the runner exclusion blocks a full green read, say so explicitly rather than claiming green).

**Evidence to return:** merge commit hashes, the disposition table, PR #5 outcome link.

**Successor:** Order B (fixes land on the merged tree).

---

## Order B — Review debt: GLM roadmaps worked

**Lead:** Terra lane · **Finish line:** P0s closed with tests; the 98-finding implementation verified · **Report:** one consolidated report per round under the loop rule.

**Goal:** The review debt is worked, not admired. GLM full-scope roadmap (46 findings @8218532) runs P0-first; the second GLM pass (98 findings: 3 critical, 27 high, 44 medium, 24 polish, phased plan — Cayleb is having it implemented directly) is verified on handback against its own findings list.

**Context:** Walks: `ergon-review2-glm-full-scope-walk-2026-10-30.md`, `ergon-review2-parts-handbook-walk-2026-10-29.md`. P0 order: **X-1** test runner silently excludes ~180 suite tests (fix FIRST — every later green claim depends on it), **X-2** bare JSON.parse at store loads, **O-1** onboarding filename sanitisation collision, **L-2** socket destroyed on oversized body, **L-3** decodeURIComponent pre-auth 500s. **SC-4 is already answered** by Cayleb's 29 Oct model ruling (Haiku on DSH) — doc alignment only, no debate. Note the 46-finding review read the pre-DSH tree: re-check installer/legacy-phone items against the DSH build before spending effort. Parts Handbook F-2 single-entry residual stays Cayleb's open call — do not tighten or loosen it.

**Constraints:** Loop rule binds: written finish line per round; after round 2, a further round only for findings contradicting a locked decision or breaking a stated acceptance criterion WITH a concrete failing scenario. Maker never checker — a fresh-context checker re-runs the affected tests and spot-runs the failing scenarios from the findings. No merges beyond this order's fixes to main, each with tests.

**Done when:** X-1 fixed and the full suite count is honest (run total before/after in the report); X-2/O-1/L-2/L-3 each closed by a regression test that fails on the old code and passes on the new; Mediums triaged into fixed / recorded-limitation with reasons; the 98-finding handback (when Cayleb delivers it) verified item-by-item — implemented-and-proven vs claimed — with discrepancies listed, not smoothed.

**Evidence to return:** per-finding disposition table (id, fix commit, proving test), suite totals before/after X-1.

**Successor:** Order E (its end-to-end passes assume this tree).

---

## Order C — DSH plugin build (the product core)

**Lead:** Terra lane lead (span 3–5) · **Finish line:** an employee-shaped user can install, open, and work with the agent end to end · **Report:** one consolidated report at the finish line.

**Goal:** Ergon exists as a plugin on DeepSeek Harness: one main agent per employee that learns how they work, adapts to their workflows, proactively offers tasks to take over, and extends itself through the review gate; front end in the familiar Claude/ChatGPT chat shape, built for people who barely know chatbots.

**Context:** Structure doc is the source of truth (session model, three shared suites, sub-agent library rules, Kem graph requirements in `~/workspace/research_notes/kem-knowledge-graph-harvest-20261029/report.md`, Cordis plugin mechanics and the `cordis_define` staging→promotion path = ReviewDesk). Model: **Haiku 5.5 via the Anthropic provider — DeepSeek R1 is definitively not used; MiniMax is not the route on this platform.** DSH research: `~/workspace/research_notes/deep-sea-harness-20261029/report.md` (v0.1 preview; breaking changes promised; no completed security audit — pin the version). Voice: no built-in voice messages in DSH — OS dictation interim, hosted service per Order D. Plugin rulings on record: persistent terminals adopted; auto authorisation review is the default permission mode; session search REJECTED (the knowledge graph is the memory); Agent Teams plugin rejected.

**Constraints + anti-goals:** Goldfish spec pass before execution. Security baseline is a design constraint, not a workstream: zero added human friction — any feature adding an employee-facing auth burden is non-compliant by design (`ergon-security-baseline-zero-friction-2026-10-29.md`). Nothing product-layer named around RHL. Generative UI (A2UI) belongs to the full-Ergon front end per the structure doc — scope it as the doc scopes it, no gold-plating mid-build.

**Done when:** on outis-desktop, a scripted fresh-user run passes: install → first-run onboarding conversation → the agent observes/asks and builds one small tool → the tool passes the review gate into the shared library → a second session uses it. The run is executed by a fresh-context checker from the script, not narrated by the builder.

**Evidence to return:** the run script + its output, plugin manifest, pinned DSH version, assumptions log.

**Successor:** Order D.

---

## Order D — Doctrine pass: company DB, hosted voice, house style, gates

**Lead:** Terra lane lead · **Finish line:** the board app is the doctrine's reference implementation · **Report:** one consolidated report at the finish line.

**Goal:** The doctrine is in the product, demonstrated on the board: the board reads/writes the company database live under its own subsection; dictation runs through the hosted voice service; the board wears the Annnimate house style; and the app admission gates are applied to the board as their first case.

**Context:** Doctrine file §§2–7. Company DB: Supabase "Ergon" project (ref yhpjpjeedpuvcgnvfpka), schema `ergon_warehouse`, tables `lines`/`entries` — verified 19 lines / 12 entries; the board currently serves a local copy, which this order retires. Voice service: `voice-service-options-2026-10-30.md` — build **faster-whisper large-v3-turbo (STT) + Kokoro-82M (TTS)** on the management host behind one OpenAI-compatible endpoint (`/v1/audio/transcriptions`, `/v1/audio/speech`); Groq free tier is the configured fallback (same API shape). The board's dictation already sits behind a single configurable endpoint (07:04 constraint) — point it at the service. House style: Annnimate baseline (free Starter Pack is enough to build against; the Solo subscription is Cayleb's purchase, NOT this order's) + the skill stack (Impeccable / Taste / frontend-design / Kowalski). Voice setup pattern: in-app optional button + "Voice control activated" badge, reusable across apps.

**Constraints:** The seed's crossed-out line-S number stays excluded everywhere. The board's LAN-open ruling stands (Wi-Fi boundary; no sign-in; nothing reachable from outside). Print stays clean (the chart is posted by the door — landscape, one page). One writer per shared file; DB schema changes are this order's alone.

**Done when:** with the board's local data file removed on a test copy, the board still serves 19/12 from the company DB and an edit made on one device appears on a second device; a dictated job added from a phone lands via the hosted STT endpoint (server log line proving the local service, not a cloud call, transcribed it); the admission-gate checklist (§5 of the doctrine file) is filled in for the board and passes, signed by a fresh-context checker.

**Evidence to return:** gate checklist, STT service log excerpt, before/after renders of the board.

**Successor:** Order E.

---

## Order E — Installer hardening and end-to-end passes

**Lead:** fresh-context checker lane (builders from Orders A–D do not check their own work) · **Finish line:** two consecutive clean full passes · **Report:** one consolidated report at the finish line.

**Goal:** Prove Cayleb's bar literally: Ergon can be installed and set up in any company. Two full end-to-end passes on a clean machine profile: standard setup script → intro screen → onboarding → board + one more app in real use (voice and typing, phone on the LAN) → in-app update via the GitHub flow → restart on the new version.

**Context:** Install rulings: terminal setup script (no EXE wrapper), per-user, no admin elevation, no vendor sign-in ever presented, harness preconfigured, installer closes itself on success and launches the intro screen, updates via the in-app "click to restart and update" button plus a check on start. WKS-09 lesson: Controlled Folder Access blocks writes to Documents — reroute, never defeat machine protections. API provisioning: one Anthropic team account owned by Cayleb; he assigns each install its own key — test installs use a key he has already assigned; if none is available for the test machine, that is a hard hold to report, not a workaround to invent.

**Constraints:** A pass counts only if run from a clean profile by the checker following the written setup exactly as an employee would — no builder shortcuts, no pre-seeded state. Any failure restarts the pass count after the fix; failures are reported with the exact step and output.

**Done when:** two consecutive clean passes are on record with timings, and the update flow is proven inside at least one of them (version before/after shown).

**Evidence to return:** pass scripts, per-step outputs, version evidence, failure log if any pass broke.

**Successor:** Order F.

---

## Order F — Final full-scope review and done-report

**Lead:** fresh review session (Claude from the Mon 01:00 ET reset if the build runs long; otherwise Terra extra-high) · **Finish line:** Cayleb's done-report delivered · **Report:** the done-report itself.

**Goal:** The finished tree gets the review Cayleb ordered — a full-scope pass in the GLM review's shape — its findings worked under the loop rule, and then the done-report grades the weekend against Cayleb's seven-line bar (plan §"The done-bar") line by line, with evidence links and every miss named plainly.

**Constraints:** The reviewer did not build any of what it reviews. Findings that miss the loop rule's post-round-2 bar are recorded as known limitations, not new rounds. The report does not use the word "done" for anything the evidence doesn't carry.

**Done when:** the done-report exists at `~/workspace/goals/ergon-for-rhl/files/ergon-weekend-done-report-2026-11-01.md` (date adjusted to the actual finish), every bar line carries a verdict + evidence reference, and Cayleb has it in chat.

**Successor:** none — this closes the weekend build. The lane's queue after this is the post-weekend backlog in the goal's hidden_files pickup list.


---

## Amendment (30 Oct 2026, 07:12 ET — the stale-docs incident)

Cayleb caught GLM about to implement the 98-finding plan against the repo's stale documentation (OpenClaw as the implementation, an "Outis management layer", MiniMax, Ergon Lite as a separate product). The rulings lived in Nox's memory and goal files and were never pushed into the repo. Two changes:

1. **Order A gains a step 0, before the merge:** push `ergon-current-state-brief-2026-10-30.md` into the repo as the CURRENT-STATE doc, and correct the stale architecture statements in place (harness, model, management, product structure) so the repo teaches the current design. No other weekend order starts from repo docs until step 0 lands.
2. **Order B precondition:** GLM's 98-finding implementation (Cayleb-directed) is verified against the current-state brief, not against repo documentation; any finding implemented toward the old architecture is a defect, listed as such in the verification table.


## Amendment 2 (30 Oct 2026, 07:17 ET — the testing rig)

Cayleb: proper testing of Ergon runs on **his Windows laptop when he gets home**. He connects DeepSeek or ChatGPT there himself and personally drives the laptop leg, sprints through Ergon, and the fleet moves to everything else afterwards. Order E is amended accordingly: the laptop is the acceptance rig and Cayleb is its driver. The lanes' job for Order E becomes preparation — install packages built and staged, both end-to-end passes scripted step-by-step, evidence capture ready — so Cayleb's session is install → connect → run → judge, with Nox analysing results as they land. The desktop build (Orders A–D) is unchanged and proceeds in parallel; a pass still counts only on a clean profile with no builder shortcuts, and Cayleb's verdict on the laptop passes is the acceptance evidence for the "tested multiple times" bar line alongside the checker runs.


## Amendment 3 (30 Oct 2026, 07:25 ET — agent pairing)

Cayleb's requirement: every app pairs with the user's personalised agent (doctrine file §8). Orders C and D absorb it: the DSH plugin build (Order C) includes the app-integration/pairing layer — pair on open, visible pairing state in every app, unpair at will, commands execute through the paired agent, unpaired apps run manual-only with speech as text. Order D's gate evidence for the board gains one line: pairing state visible and a dictated command shown executing through the paired agent, not merely transcribing.


## Amendment 4 (30 Oct 2026, 07:28 ET — the app template)

Cayleb: the pairing requirement (and the standing app requirements with it) must live in the **app-building template** for the app suite. Added deliverable, owned by Order C with Order D verifying: the suite's app template ships with pairing, visible connection/voice state, company-DB subsection wiring, the configurable transcription endpoint and the house style baked in, so agent-built apps conform by construction. Order D's gate evidence adds: the board (or a sample agent-built app) shown to derive from the current template version.


## Amendment 5 (30 Oct 2026, 07:28 ET — creation blueprints)

Cayleb: agents need a workflow — a blueprint — dictating how they build for every creation surface. Added deliverable (Order C): the four creation blueprints (apps, skills, sub-agents, knowledge-graph additions) drafted as versioned suite artefacts, each running the spine trigger → template → build rules → gate → admission/registration (doctrine §9). The knowledge-graph blueprint encodes the Kem intake rules (per-claim citations, notes as truth, visibility split). Order F's done-report grades whether the blueprints exist and whether at least the app blueprint was actually exercised by an agent building from it during the weekend.


## Amendment 6 (30 Oct 2026, 07:50–07:51 ET) — suite shell, branding, plugin front-end model

Cayleb's rulings from his first look at the live board (full text: `ergon-app-doctrine-2026-10-30.md` §§10–11) bind Orders C and D: (1) apps are client-branded (the board is the **RHL Warehouse Board**); the "Ergon" wordmark becomes a small logo mark; no top bar — the suite shell is a **sidebar dashboard**. (2) The Ergon plugin's front-end layer inside DSH makes the experience **one long-running session with the user's one personal agent** (delegation invisible unless the user digs deeper), and the plugin carries **its own button/section on the sidebar** — click-through apps plus the **live view of the brain** (the knowledge graph, live). Order C's UI work conforms to this model; Order D verifies the board against §10.


## Amendment 7 (30 Oct 2026, 07:52–07:53 ET) — formula-not-apps; one private repo per company

Per doctrine §12: Ergon ships the **formula** (templates + creation blueprints), never pre-packaged apps. Each company gets **its own separate private GitHub repo** holding its full personalised suite — apps, skills, tools, and each personalised agent — as its cloud home and backup, so cloud agents can work on that company's suite in isolation. Order A treats the merged Ergon project as the platform/formula repo; Orders C/D respect the split (RHL's board is RHL-suite content under this model; any repo move for it is planned inside the consolidation, never improvised mid-build).


## Amendment 8 (30 Oct 2026, 08:02–08:03 ET) — free-form commands; brain-backed retrieval

From Cayleb's live board test (doctrine §§13–14): (1) every app's command surface must pass the **free-form acceptance** — a checker-run set of varied natural phrasings executing correctly through the paired agent; fixed grammars and sample paths are defects, and suggestion chips and agent-replaceable utility buttons are stripped by template. (2) **Brain-backed retrieval** joins Order D's scope alongside the company-DB wiring: apps pull everything the brain knows about a record (pictures included) at creation time and on demand, through the paired agent.


## Amendment 9 (30 Oct 2026, 08:34 ET) — the home programme

Cayleb restated the sequencing: Ergon Lite (the board) finishes FIRST — by 11:00 ET today, with all polish additions. Then, **when home, on his laptop with ChatGPT connected to it**: the **full complete Ergon plugin finishing and testing**, and the **management app**. The laptop leg (Amendment 2) therefore carries three workstreams, not one: the plugin completion, its testing, and the management app — lanes prep all three so Cayleb's hands-on time is connecting, running and judging.


## Amendment 10 (30 Oct 2026, 08:40 ET) — three-piece packaging binds Orders C–F

Cayleb's canonical definition: **Ergon is a three-piece package** — (1) the management ticket and support software, (2) the DeepSeek Harness baseline, (3) the Ergon plugin, which is the full package that takes the DSH baseline and turns it into the fully functional Ergon app. Order C builds and Order F verifies against this packaging: the plugin is not an add-on to a finished baseline — it is the piece that makes the baseline into Ergon. Ergon Lite = pieces 2 + 3 without piece 1.


## Amendment 11 (30 Oct 2026, 08:41 ET) — Order E's target is the Ergon installer

Cayleb: **Ergon itself is the installer** — run it on a PC and it downloads everything (all three pieces, per Amendment 10) and sets everything up, easily, like an official Windows installer / OpenCore setup. Order E therefore delivers an installer-grade setup experience, not a hardened script with a wrapper: guided, preconfigured, per-user, self-closing into the intro. Acceptance: a clean-PC run goes from "run Ergon" to a fully set-up suite with no expert intervention, then passes the two consecutive end-to-end passes already specified.


## Amendment 12 (30 Oct 2026, 08:43 ET) — the expansion method: free-rein passes, Nox alignment audit, rotating thermonuclear rounds

Cayleb's method, governing Orders B and F and the whole completion build:

1. **GLM (and each rotating agent after it) gets free rein** — does everything it can, pushes everything it wants. No pre-clearance of its changes.
2. **Nox alignment audit is the gate.** Nothing an external pass lands counts as done until Nox has reviewed it against the full recorded vision — the current-state brief, doctrine §§1–18, the three-piece packaging (Amendment 10), the installer definition (Amendment 11), and Cayleb's standing rulings. Cayleb's expectation is that even a briefed GLM is not fully aligned; the audit exists to catch exactly that. Misalignments are fixed or flagged with evidence, never waved through.
3. **Then rotate: ~three back-to-back thermonuclear passes, different models, different agents, different sessions** — each told to do a thermonuclear code review and bring its areas to a ludicrous bar, expanding the app hard between audits.
4. **Then final review, tweaks and polishing** — and the whole cycle may run again.

Cayleb's evidence for the method: this prompting pattern (thermonuclear review, ludicrous bar) has already worked — the Freebuff and GLM passes are the proof.


## Amendment 13 (30 Oct 2026, 09:01 ET) — hosting target: dedicated cloud host per company

Cayleb's pick: apps host on a **dedicated cloud host per company** (single-tenant, the company's own profile), chosen because **Nox holds native control and can fix any issue remotely**. Order D's hosting work and Order C's spec pass work to this target: host provisioning per company, apps + hosted voice service on that host, company DB per company (Supabase per the cost sheet, `ergon-cost-sheet-2026-10-30.md`). The spec pass must deliberately revisit the security baseline's private-network-only clause for an internet-facing host and record what replaces it — never silently drop it.


## Amendment 14 (30 Oct 2026, 09:04 ET) — agent residency: the company cloud host

Cayleb: **the mobile personalised agent must work even if the desktop is off** — the agent's home is the company cloud host (Amendment 13), not the employee's desktop; phone and desktop are both windows onto it, and the company layer is cloud-hosted so management and device changes break nothing. This supersedes the "desktop-resident agent" framing wherever it appears. Orders C and D build to it; Order E's end-to-end passes gain an acceptance case: **desktop off → the phone agent still answers, acts, and reaches the apps end to end.**


## Amendment 15 (30 Oct 2026, 09:25 ET) — voice STT is a head-to-head, not a coronation

Cayleb intaked **Phonon-2** (Fermion Research; intake + verification: `phonon-2-intake-2026-10-30.md`) as the candidate for the voice service's STT slot, with the instruction to be sure it's truly top of the line. Order D's voice-service build therefore runs **Phonon-2 vs faster-whisper large-v3-turbo behind the same OpenAI-compatible endpoint**, judged on a warehouse command test set (spoken job numbers/codes, names, line letters, background noise) — not on leaderboard WER alone. Winner ships; the result and the measured numbers are reported either way.


## Amendment 16 (30 Oct 2026, 09:39 ET) — voice: the plugin replaces the service; Amendment 15's head-to-head is retired

Cayleb ruled: the DeepSeek Harness voice input plugin is ported with the personalised agent and **replaces the hosted voice service entirely** (doctrine §21). Consequences: Order C's first job is now to establish what the plugin actually does (capture vs its own transcription; any engine it calls; input only or output too; whether it runs outside the harness's own interface) and report any gap against the ruling at once. Order D no longer builds a central voice service; it ports the plugin as the agent-carried voice connection for paired apps. Amendment 15's Phonon-2 vs faster-whisper head-to-head is **retired** — Phonon-2 stays on file as the fallback engine candidate only if the plugin proves to need an external STT engine.


## Amendment 17 (30 Oct 2026, 10:39 ET) — database provider stays swappable

Roy told Cayleb RHL might have its own database hosting service (unconfirmed). Order D's company-database wiring therefore keeps the provider swappable: Supabase is the default and the build target; if RHL's in-house hosting is confirmed, the wiring adapts to it instead. Do not hard-wire Supabase-specific assumptions into app code.


## Amendment 18 (30 Oct 2026, 11:06 ET) — practice setup, laptop completion, then RHL final check

Cayleb's completion sequence, stated plainly: everything is set up exactly as it would be live for RHL — production-shaped in every way — except that we do not have (and never invent) their native information. Ergon is driven to completion with full testing on Cayleb's Windows laptop at home; the finished suite then comes into RHL for final checking. This sharpens Amendment 2's laptop leg from "acceptance rig" to the completion venue itself.


## Amendment 19 (30 Oct 2026, 11:27 ET) — management piece gains the fleet improver

The management-app workstream carries two linked components from Cayleb's 11:27 direction: (1) a **daily fleet efficiency audit** across all employees' agents, centred on token spend, against a starting API-credit anchor of ~$2,000/month per company; (2) the **fleet altitude of the one recursive self-improvement method** (Harness²): harvest and promote proven local improvements through ReviewDesk, propose suite/harness changes (never silently apply), enforce budget policy. Explicit anti-goal: no second, competing RSI engine layered over the agents' own improvement — one improver, two altitudes (doctrine, management-piece section 30 Oct).


## Amendment 20 (30 Oct 2026, 11:30 ET) — voice lands at home; completion testing runs in a simulated fleet on the laptop

Two rulings from Cayleb: (1) **Voice dictation on the warehouse board is a home update**, added to the otherwise-finalised board and tested at home on our own devices — the Windows laptop has the microphone the work demo lacks. It is not part of the work-PC artefact. (2) **Ergon's completion testing runs in a simulated environment on Cayleb's Windows laptop**: lightweight VMs or sandboxes standing in for the work PC and a group of work PCs, each with its own personalised agent, so the whole journey — setup through completely autonomous running — is emulated start to finish before anything returns to RHL. The home desktop is NOT the simulation host (Linux; wrong OS, Cayleb's call). The exact mechanism (VM weight, count, sandbox style) is Outis's to determine on the laptop against its real RAM and resources; full Windows VMs are heavy, so a small fleet or lighter sandboxes is the expected shape — the lane reports what the hardware supports rather than forcing the plan.


## Amendment 21 (30 Oct 2026, 11:56 ET) — home end-to-end test topology

Cayleb's home acceptance setup for the completed Ergon: **laptop** = the suite under test (completed there, end-to-end tested there, incl. the Amendment 20 simulated fleet); **his desktop** = the management software role; **his phone** = an enrolled client device — and the **phone ↔ laptop connection is explicitly in the acceptance set** (his correction: the phone tests the connection, it is not a proxy). Distinct from Amendment 20's exclusion: the desktop is not simulating Windows work PCs — it plays the management role only.


## Amendment 22 (30 Oct 2026, 12:53 ET) — laptop unavailable; desktop-first; the harness build is commissioned

Cayleb left the laptop at work: it is not available this weekend, and desktop + laptop cannot be connected simultaneously in any case. Sequence for the weekend: **harness work first on the desktop, then all desktop-doable Ergon work on the desktop**; the laptop testing leg (Amendments 2/20/21 — simulated fleet, E2E topology, Windows acceptance) is **deferred to the next laptop window**, not cancelled. In the same ruling Cayleb commissioned the single custom harness as Nox's design: built on the desktop, shaped around the fleet's learned methods and decluttered, explicitly NOT the old patchwork architecture; ChatGPT and Claude Desktop remain Cayleb's personal tools, outside the fleet control plane; Nox's comms route to the harness is the Crosswalk MCP under its standing hard conditions; Outis-the-Dots-agent does not join — his lane-lead role ports into the harness as a bot when it stands.
