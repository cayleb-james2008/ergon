# Ergon — current state of the project (v2)

*Context brief prepared by Nox (Cayleb's agent), Fri 30 Oct 2026, 15:13 ET. This v2 supersedes the v1 brief of the same morning — several rulings landed after v1 was written, and GLM's implementation run was lost before it was ever pushed (see §12). Attach this brief to the initial prompt of any build run. Where this brief conflicts with any repo document, any older brief, or the build plan being executed, THIS BRIEF GOVERNS.*

## 1. What Ergon is

Ergon is one project: a company-agnostic template for a complete company software suite, installed per company. It gives every employee a personalised agent that learns how they work, adapts to their workflows, proactively offers tasks to take over, and recursively improves and extends the suite. Around the agents sit shared, company-wide suites (tools/apps, sub-agents, skills) with a review gate, and a management piece (management apps; §2). First company: RHL (Robert H. Lord Company, a furniture company in Manchester, Connecticut), where Cayleb Alvarez-James works as an installer and leads the AI-integration effort. First app: a warehouse lines board for RHL (§7), installable on its own.

## 2. The product structure — three pieces (locked 30 Oct 2026)

Ergon is a THREE-PIECE PACKAGE:
1. The **management ticket and support software** (the management piece — management apps; the RHL warehouse lines board is its first app).
2. The **DeepSeek Harness baseline**.
3. The **Ergon plugin** — which takes the harness baseline and turns it into the fully functional Ergon app.

**Ergon Lite = pieces 2+3 without piece 1.** "Ergon Lite" is NOT a separate product, repo or install — it is the same project minus the management piece. Any document treating Lite as its own product is stale.

**Ergon IS the installer.** Running Ergon on a PC downloads all three pieces and sets everything up, easily — the experience bar is an official Windows installer / how OpenCore setup feels. The installer is the product's front door. (Mechanics underneath may be scripts; the experience may not feel like one.)

## 3. Platform decisions (locked 29–30 Oct 2026)

- **Harness:** DeepSeek Harness ("dsh", MIT, v0.1 preview; plugin framework: **Cordis**), with Ergon delivered as a plugin. The name is **DeepSeek Harness** — "Deep Sea" is a voice-to-text artefact and is never correct. **OpenClaw is OUT** (removed 29 Oct); any doc naming OpenClaw as the implementation is stale.
- **Model:** the build runs **Haiku 5.5 through Claude (Anthropic provider)**. **DeepSeek R1 is definitively not used. The old "MiniMax is the suite's only model" lock is retired for this platform.** Any doc naming MiniMax or R1 as the model route is stale.
- **Management layer:** there is no "Outis management layer". Outis is a build lane (a Codex-based agent Cayleb delegates construction to), not part of the product. Nox manages the library/host side.
- **One project, per-company repos:** Ergon ships NO pre-packaged apps — it ships the formula (app-building template + creation blueprints). Each company gets its own separate private GitHub repo (its apps, skills, tools, personalised agents) as its cloud home/backup; the Ergon project repo holds the platform/formula only.
- **Naming rule:** nothing in the platform/product layer is named around a client. App instances at a company ARE client-branded (§8).

## 4. Architecture (current)

- One main agent per employee; sub-sessions are its children. Sub-agents are ephemeral by default; persistent only as deliberate artefacts in a company sub-agent library, each carrying the same recursive self-improvement loop.
- Three shared suites per company: (1) a combined tool/app suite — one library, one review queue, tool-vs-app is a surface tag; (2) a sub-agent suite; (3) a skills suite.
- Every app is **dual-surface**: an agent-drivable back end (the whole back end operable from a plain ask) plus a strong human front end, over shared state. **Agent pairing:** opening an app pairs the user's personalised agent with it; the pairing state is ALWAYS VISIBLE; unpaired, manual use works and speech stays mere text. Pairing and all standing app requirements are built into the **app-building template**; admission gates verify template provenance.
- Review gate (ReviewDesk pattern) extended to apps: agent-built tools/apps stage, get reviewed against stated acceptance criteria, then promote. A maker's self-report never closes a piece of work — a fresh-context checker runs the actual output.
- Memory: a knowledge-graph layer (Kem pattern) over a notes substrate — notes are the source of truth, every claim carries a citation, the graph is a view/search layer. Session search was rejected.
- Front end: familiar Claude/ChatGPT-style chat for everyday users; home = the one main agent, sidebar carries everything else (NO top bar — a sidebar dashboard shell). The Ergon plugin's front end is one long-running session with the user's one personal agent; the plugin adds its own sidebar section: click-through apps plus a live view of the brain (the full knowledge graph, live). Generative UI (A2UI, pinned v0.9.1) applies to the full-Ergon front end only.
- DSH plugin rulings: persistent terminals adopted; automatic authorisation review is the default permission mode; Agent Teams rejected.

## 5. Hosting, data, residency, access (locked 30 Oct 2026)

- **Hosted access:** apps are hosted on ONE machine and everyone uses the same apps through a shareable URL — **nobody installs anything to use an app**. The URL is never `localhost` — a real name on the network. Agent profiles and keys live on the host only. (This is how the RHL board demo ran: hosted on a coworker's desktop, opened from phones on the work Wi-Fi.)
- **Dedicated cloud host per company:** the production shape is a single-tenant cloud host per company (standard: **Hetzner CPX22 class, ~€8/mo, US Ashburn**), never a shared multi-company service — client data never pools. Nox holds native control of the host so issues can be fixed remotely. **Fleet end-state:** once a company approves, the Ergon harness is installed on every computer there and the whole company lives inside the suite; the hosted-URL model is the pre-approval phase of the same product.
- **Agent residency:** the personalised agent's home is the **company cloud host** — the agent must work EVEN IF THE DESKTOP IS OFF. Phone and desktop are both windows onto it. (Any doc describing the agent as desktop-resident is stale.)
- **Data:** everything on **Supabase — one Pro org of ours, one project per company** (each project is its own isolated database). Every app connects to the one official company-wide database under its own subsection, legible to a non-technical browser. **The database provider stays swappable by design** — RHL may have its own database hosting; if so, the wiring adapts. Budgets: hosting ≤ $100/mo per company; API credits are the biggest cost line by design; planning anchor ~$2,000/mo API credits per company as a comfortable envelope the company's own call can set lower.
- **Per-device enrolment:** access is bound to the installed device — install Ergon on a device and only that device connects to that company's server, from any network; unenrolled devices get nothing even with the address; per-device revocation from the management side.
- **Security baseline:** "Amazon/Apple-level" security with **zero added human friction** — security lives under the floorboards; any design adding an employee-facing authentication burden is non-compliant. Human credential: passkeys + one-time recovery code. Per-machine revocable tokens, least privilege, encryption, full logging, automatic patching, backups, private repos. (The baseline's private-network-only clause was written for LAN hosting; the cloud host is internet-facing — revisit that clause deliberately in design, never silently drop it.)

## 6. Voice (locked 30 Oct 2026 — supersedes all earlier voice sections)

- **The DeepSeek Harness voice input plugin, ported with the personalised agent, IS the voice input ability of every app the agent pairs with. It replaces the hosted voice service entirely — there is NO hosted voice service anywhere in the stack.** (v1 of this brief described a hosted faster-whisper/Kokoro service; that model is superseded. Phonon-2 is on file as a fallback engine candidate only if the plugin needs an external STT engine.)
- **Voice calls:** local small-model STT + local TTS edges (Kokoro-class; ~1 CPU core in bursts, ~1 GB RAM — sized for resource-poor work desktops) with the API model as the brain. No large local STT and no local LLM anywhere.
- Known gap the build must close: the pinned DSH plugin transcribes into its OWN unsent draft — the port must bridge plugin output into paired apps, and the call edges are built, not assumed. There is NO Claude voice API in this design.

## 7. App doctrine (30 Oct 2026; full text in the repo, team branch, docs/nox-governing/)

- Every app fully functional end to end — no stubs. Built exactly as it would go live for the client, production-shaped in every way — except no client's native information is ever invented (we can't have it; seed only from provided material).
- **The command layer IS the agent** (§13 of the doctrine, plus its addendum): the user can say whatever they want, in any phrasing, and the agent interprets and executes (add/remove/search/edit). Preset intent matchers and confidence-gated clarification are BANNED. Clarification is asked only when information is genuinely missing. Acceptance uses the owner's own fresh phrasings, never a drafted acceptance set.
- **Brain-backed retrieval:** clicking a job (or asking) pulls everything the brain knows about it — pictures and all relevant information, at creation and on ask, via the paired agent. The per-job data SOURCE is an open question (undecided; do not invent one); the principle and the click-through breakdown stand.
- No clutter chrome (no suggestion chips, no Refresh/Print buttons), no explanatory sections in apps, voice setup appears only when voice is invoked, manual entry is first-class (click any item → a form with all its fields, brain retrieval assisting inside that flow).
- Motion and finish are first-order: the house baseline is **Annnimate** (GSAP-based; React/Vue/plain HTML), design stack Impeccable + Taste skill + frontend-design + Emil Kowalski's motion skills. UX is part of the build, never a follow-up.
- Person/colour mappings ship ONLY from a confirmed mapping, never an agent's guess. The confirmed RHL board mapping: **JA Jessica blue, JS Janice pink, JG Joe orange, AK Anna green, LB Linda PURPLE** (not yellow).

## 8. Branding (locked 30 Oct 2026)

- Every app in the suite carries the **client's actual name/logo in the top corner**. The shell keeps its baseline templatised look; the "Ergon" wordmark becomes a small simple logo mark. **Ergon is the architecture, not the face** — the suite presents as the client's own custom suite of apps and agent fleet; Ergon adapts to them and disappears underneath.

## 9. Management fleet improver (Cayleb, 30 Oct 2026)

- The management software runs a **daily pass over every employee's agent** hunting inefficiency — above all token waste — because the suite must stay budget-friendly. Cayleb wants the management layer to be the main recursive improver of suite + agents + harness.
- Nox's recorded recommendation (Cayleb's explicit ruling on the shape is still pending — do not treat the shape as decided): ONE improver (Harness²) at two altitudes — a local loop per agent (own tools/workflows, ReviewDesk-gated promotion) and a fleet loop (daily audit, harvest-and-promote proven local improvements through the same gate, PROPOSE suite/harness changes, never silently apply; budget enforced via per-employee fair-share quotas). Never build two competing improvement engines.

## 10. The first app — the RHL warehouse board (state at 30 Oct 2026)

- Spreadsheet-style table of warehouse lines A–S (19 lines); jobs entered/erased by typing or the paired agent; ONE control — a command bubble, floating, draggable, translucent; the board fits the viewport (no page scroll, no trailing white space); per-job boxes size to each line's actual jobs; the RHL logo sits on the board; it prints cleanly on one landscape page.
- **Demo status: CLOSED CLEAN (30 Oct, noon).** The accepted build (V11) passed 15/15 acceptance cases; 19 lines / 12 jobs preserved; second-device proof passed (opened on Cayleb's phone over the work Wi-Fi). Roy (the RHL stakeholder) saw it in person and liked it. Carry-overs for the next pass: remove the bulky move selector from the command bar; expand the job click-through breakdown categories (the delivery-ticket field model — order ID, customer vs ship-to, ticket/due dates, sales rep, PM, repeatable items with description + qty — is the source set); voice dictation as a home update.
- **SOURCE WARNING:** the accepted V11 source lives on the work PC (RHLCO-WKS-09), which is OFFLINE. The repo's board copy is OLDER. Do not "fix" or rebuild the board from the repo copy; board source edits wait for the work PC's next window.
- Board data: hosted copy in the "Ergon" Supabase project, schema `ergon_warehouse`, tables `lines`/`entries`. One crossed-out number in the source photo is deliberately excluded everywhere — never reproduce it.
- Deployment ruling for the demo: ran open on the work Wi-Fi, no sign-in (the Wi-Fi boundary is a security layer; nothing reachable from outside).

## 11. Install and distribution

- Ergon itself is the installer (§2). Per-user install, no admin elevation, no vendor sign-in ever presented, harness ships preconfigured; the installer self-closes on success and launches the intro. Updates from GitHub via an in-app restart-and-update. Distribution: private GitHub Releases / a link Cayleb sends personally. No code-signing certificate yet. API provisioning: one Anthropic team account owned by Cayleb; he personally assigns each install its own key — no build run ever handles key values.
- Machine lesson (standing): Windows Controlled Folder Access can block writes to Documents — reroute to non-protected folders, never weaken a machine's protections.

## 12. Repo state and the GLM history (read carefully — this is where runs die)

- Repo: **cayleb-james2008/ergon (private)**. **main = 9d937e83** (30 Oct 13:17): the sanitised CURRENT-STATE doc (Order A step 0), landed standalone ahead of the consolidation merge. Governing documents live on the **team** branch under `docs/nox-governing/` (app doctrine with all addenda, weekend mission orders with Amendments 1–22, the current-state brief, the harness spec v2). Where a repo doc conflicts with this brief, this brief is correct.
- A consolidation merge (Order A: one project, PR #5 port-and-close-as-superseded) is IN PROGRESS on another lane against this main. **Do not touch main. Do not rebase, merge into, or overwrite it.**
- Findings registers on record: (a) GLM full-scope review — 46 findings (5 High: X-1 test runner silently excludes ~180 tests — fix FIRST — X-2 bare JSON.parse at store loads, O-1 onboarding filename-sanitisation collision, L-2 socket destroyed on oversized body, L-3 decodeURIComponent pre-auth 500s); (b) a second GLM pass — 98 findings (3 critical, 27 high, 44 medium, 24 polish) with a phased plan; (c) GLM's code audit report — 48 findings (10 critical, 13 high, 19 medium, 6 polish) with a Completion Build Plan (4 phases).
- **The GLM implementation run (30 Oct):** GLM executed its Completion Build Plan in its own workspace and reported full completion (11 local commits; its own fresh-clone proof: 139 tests discovered / 117 passed / 0 failed / 22 counted skips; 43 of 48 findings closed). **None of it was ever pushed, and the workspace was then wiped — the implementation no longer exists.** The audit report, build plan and implementation log documents survive (Cayleb holds them). Every number in that report is the maker's own grading and is UNVERIFIED.
- The redo therefore executes the same plan from the surviving documents, under this brief's governance, and its output faces Nox's alignment audit before any of it counts.

## 13. Working rules for any build run under this brief (non-negotiable)

1. **Branch only.** Work in Cayleb's ergon repo on a dedicated branch (name it for the run, e.g. `redo/completion-2026-10-30`). NEVER commit to main, NEVER force-push, NEVER rewrite shared history.
2. **Push incrementally.** Commit and push at every phase boundary at minimum — ideally every workstream. The previous run lost EVERYTHING because 11 commits lived only in a local workspace that got wiped. A run that cannot show its work on the remote does not have the work.
3. **Maker ≠ checker.** A maker's self-report — including a "fresh-context proof" rerun of its own suite — is a reproducibility check, not a verdict. Report test numbers only from runs actually executed, with the command and the counts; name failures plainly.
4. **Conflicts:** where the build plan and this brief conflict, this brief wins. Where anything is ambiguous, record the assumption in the run's report and keep moving — do not stall, and do not silently choose the stale option.
5. **Naming discipline:** DeepSeek Harness (never "Deep Sea"); Ergon / Ergon Lite per §2; client data never invented; the excluded board number never reproduced.
6. **The audit gate:** on handback, Nox audits everything landed against this brief, the doctrine and the findings registers. Nothing counts, merges, or ships until the audit passes it. Build so the audit is easy: small traceable commits, findings referenced in commit messages, a handback report that maps every claim to its evidence.

## 14. Stale-doc warnings — if you read any of these anywhere, they are WRONG

- "OpenClaw" as harness/implementation → DeepSeek Harness (dsh) + Cordis.
- "MiniMax" as the suite's model, or DeepSeek R1 → Haiku 5.5 via the Anthropic provider.
- An "Outis management layer" → Outis is a build lane only.
- "Ergon Lite" as a separate product/install → §2.
- A hosted voice service → §6 (the DSH plugin replaces it entirely).
- The agent as desktop-resident → §5 (cloud-host resident; works with the desktop off).
- Installing apps per user to use them → §5 (hosted; shareable URL).
- LB yellow → LB is PURPLE. "JAN" as a person → JAN was an error; the mapping is §7.
- Per-company visual styles → one house style, client-branded per §8.
- Paper-first / demo-scoped language → the product bar is end-to-end, installable in any company, tested multiple times by fresh-context checkers, code-reviewed to the moon.

*End of brief v2. Work from this; where anything else disagrees, it is out of date, not this brief.*
