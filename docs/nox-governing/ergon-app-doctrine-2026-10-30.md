# Ergon app doctrine — Cayleb, Fri 30 Oct 2026 (06:53–06:57 ET)

Standing rules for every app Ergon builds into a company. Set by Cayleb while grading the warehouse board live; they bind all suite builds from here on. This morning's 08:00 board test is NOT re-scoped by them — the board keeps its proven data path for the demo, and the doctrine lands on it as the first post-demo consolidation pass.

## 1. One house design style

One design style for every Ergon app, everywhere — no per-company styling (supersedes the 29 Oct taste matrix). Stack:

- **Annnimate** (Cayleb's pick, 30 Oct) — the baseline component/motion library: 129 components, each in React, Vue and plain HTML, one GSAP engine; agents pull real source via its MCP server; pasted code stays ours. Solo €240/yr when Cayleb chooses to buy; free Starter Pack (17 components) needs no card.
- **Skill stack on top:** Impeccable (front-end structure, Operate mode for working tools), the Taste skill (quiet borders, small labels, no gradients/glows/vanity numbers), frontend-design ("claw design"), Emil Kowalski's skills as the motion standard (motion only as feedback/state; nothing on high-frequency actions).
- **Known gap:** Annnimate is motion components (reveals, menus, sliders, preloaders, heroes). The plain workhorse primitives — buttons, inputs, data tables, dialogs — are suite-built basics held to the same style, built once and reused.

## 2. End to end, or it isn't an app

Every app built into a company is fully functional end to end. No demo-grade stubs, no dead buttons, no features that only work on the builder's machine.

## 3. One company database, per-app subsections

Every app connects to the one official company-wide Ergon database, under its own subsection, organised under categories (an **apps** category) so the structure stays clean and legible — for Nox and for an average Joe who just wants to take a look.

- The warehouse board's data already sits this way in embryo: Nox's Supabase "Ergon" project, schema `ergon_warehouse`, tables `lines`/`entries` (verified 19 lines / 12 entries).
- The board app itself still runs on its local copy; wiring it live to the company DB is post-demo consolidation item #1, alongside the one-project merge.

## 4. Dual-surface by design

Every app ships two surfaces over the same state:

- **Agent surface** — an agent can drive the whole back end from the user or employee simply asking (voice or text).
- **Human front end** — a strong, polished UI the person uses directly.

Balancing both is expected; neither is an afterthought. (Extends the 23 Oct ruling that tools are agent-friendly — dual human UI + agent surface, shared state — from tools to apps.)

## 5. Gated admission to the shared suite

No autonomously created app joins the shared tool/app suite until it passes a strict launch workflow — the ReviewDesk promotion model extended from tools to apps. Draft gate list (Nox's draft, for Cayleb's grade):

1. End-to-end functional proof — the app's core flows run clean, fresh-context checker, actual output not self-report.
2. Database wired — the app reads/writes its own subsection of the company DB; no local-only state.
3. Dual-surface proof — an agent drives it end to end from a plain ask, and the human front end renders and works on a second device.
4. Design conformance — house style (Annnimate + skill stack) verified by looking at renders.
5. Security baseline — zero-friction rules hold (no new human auth burden; per-machine credentials; private-network only).
6. Admission recorded — app entered in the suite under its category with owner, purpose and data subsection named.

## Sequencing

- 08:00 demo: unchanged bar (table, voice, search/typing, phone on Wi-Fi).
- Post-demo consolidation, in order: (a) one-project merge (Ergon/Ergon Lite), (b) board wired live to the company DB, (c) Annnimate re-skin of the board, (d) gate workflow ratified and applied to the next app.


## 6. Voice setup pattern (Cayleb, 30 Oct 2026 ~07:01 ET)

Where a phone browser blocks dictation on a LAN page (the secure-context rule), the remedy ships inside the app as a product feature, never as instructions handed to a user:

- An **optional one-time setup button** in the app starts the local-CA trust install for that device (the button serves the certificate and guides the OS's own confirm steps — Android does not allow a web page to install a CA silently, so this is as close to one tap as the OS permits).
- After setup, a persistent **badge at the bottom of the app reads "Voice control activated"**.
- Strictly optional: the app is fully usable without it (viewing, searching, typing); no prompts or nagging.
- **Default for every Ergon voice-dictation app** that hits the same browser limit.

First instance: the warehouse board — LAN-only HTTPS + local CA (approved by Cayleb 06:59 ET 30 Oct; his phone is Android), in the build he tests at 08:00.


## 7. Hosted voice service (Cayleb, 30 Oct 2026 ~07:01–07:02 ET)

Speech is a shared suite service, not a per-app feature:

- The transcription (STT) and voice (TTS) models are hosted centrally — preferably on Cayleb's own desktop / the management server, cloud only if needed — and every Ergon app calls that one service. Employees install nothing; voice just works in every app.
- Free-first: a free high-quality model with free usage via API key is the preference for now (options check commissioned 30 Oct: Groq free tier vs self-hosted Parakeet v3 / faster-whisper + Kokoro).
- Boundary: hosting does not remove the browser's secure-context microphone gate — an untrusted LAN page gets no mic wherever transcription runs. The §6 button is the per-device unlock; the hosted service is what listens afterwards.
- Build rule: dictation in every app sits behind a single configurable transcription endpoint, so moving from built-in dictation to the hosted service is a config change, not a rework. The warehouse board follows this from its first build.


## 8. Agent pairing (Cayleb, 30 Oct 2026 ~07:25 ET)

An app is never just a screen. The reason: voice dictation only becomes *action* — "add this job to line B" turning into a job actually added on line B, rather than text sitting in a box — when an agent is on the other end interpreting and executing.

- **Every app pairs with the user's own personalised agent.** Opening an app pairs the agent to that app session (the agent effectively logs on to the app); the user can unpair when done, or stay paired.
- **Pairing state is always visible** in the app — paired (agent connected, shown by name) or unpaired — so there is never confusion about whether the agent is attached. Same family as the "Voice control activated" badge: connection state is shown, never ambiguous.
- **Paired:** voice and typed commands run through the agent, which turns them into real operations on the app (the dual-surface rule, §4, with the employee's own agent as the driver).
- **Unpaired:** the app still works manually (viewing, typing, tapping), but speech is only text — nothing interprets it into actions until the agent is paired.
- Simplicity is the bar: pairing is one obvious action, not a settings expedition.

First general implementation: the weekend build (the DSH plugin's app-integration layer). This morning's demo board keeps its shape — its command bubble already routes to the board's agent endpoint; pairing is how that generalises across the suite.


**Template rule (Cayleb, 30 Oct 2026 ~07:28 ET):** pairing — and with it the standing app requirements (company-DB subsection wiring, the configurable transcription endpoint, visible connection/voice state, the house style) — is built into the **app-building template** the agents use when building personalised apps for the app suite. Compliance is by construction: an agent building from the template produces a paired, doctrine-conforming app by default, and the admission gates (§5) verify the app came from the current template rather than re-checking every requirement from scratch.


## 9. Creation blueprints — one per suite (Cayleb, 30 Oct 2026 ~07:28 ET)

Agents do not improvise how new things get made. There is a **workflow — a blueprint — dictating how agents build** for each creation surface:

1. **Apps** → the app suite (template §8, gates §5).
2. **Skills** → the skill suite.
3. **Sub-agents** → the sub-agent suite (ephemeral by default; persistence as a deliberate artefact is itself a blueprint decision).
4. **Knowledge** → additions to the knowledge graph (the company "brain"): what counts as a claim, the per-claim citation requirement, the notes substrate as source of truth, and the personal/company visibility split all governed by the graph's intake blueprint.

Each blueprint specifies the same spine: the trigger (an employee's ask, or an agent's proposal), the template/scaffold to start from, the build rules for that surface, the review/promotion gate, and admission + registration (category, owner, purpose) once it passes. The blueprints are suite artefacts themselves — versioned, living inside the suite where agents run on them, and improved through the same lesson loop as everything else (Harness²): a build that goes wrong updates the blueprint, not just the artefact.


## §10 — Suite shell and branding (Cayleb, 30 Oct 2026 ~07:50 ET, from his first look at the live board)

- **Apps are client-branded, not Ergon-branded.** The board's title "Ergon warehouse board" is wrong: it is the **RHL Warehouse Board** — an RHL-specific app, the first app of RHL's suite. Ergon is the platform underneath; each company's app instances carry that company's name. (This refines, not contradicts, the naming rule: nothing in the *platform/product layer* is named around RHL; app instances deployed at a company are branded for that company.)
- **No "Ergon" wordmark in the shell.** Replace the text with a small, simple logo mark.
- **No top bar. The suite shell is a sidebar dashboard:** a chat-app-style sidebar (the basic chatbot-interface pattern) listing the suite's apps and sections for click-through navigation; content area carries the open app. Applied to the warehouse board first, then standing for every suite app.
- **Two launch surfaces:** every app is launchable from the suite shell sidebar AND through the DeepSeek Harness preview page.
- Sequencing (Nox, 07:50): the title correction is safe string work and must be in the rebuilt Roy package; the sidebar + logo restructure lands immediately after Cayleb's 08:00–09:00 test — the live test path is not restructured mid-window — with screenshot proof at desktop and phone widths, checker-verified, and it lands before the Roy send if that can be done without touching the test path.


## §11 — The plugin's front-end model inside DeepSeek Harness (Cayleb, 30 Oct 2026 ~07:51 ET)

Ergon is a plugin for DeepSeek Harness — and part of what the plugin IS is a set of front-end changes on top of DSH:

- **One long-running session.** The plugin makes the experience feel like one long-running session with the user's one personal agent: you just talk to your agent; it delegates tasks wherever they need to go, behind the scenes. The user should only ever really have to see that one main session — deeper sessions and delegation are there only when they deliberately go looking.
- **The plugin gets its own place on the sidebar.** Not "workspaces": the plugin adds a button/section of its own on the sidebar. From it you click through the suite's apps, and you can open the **live view of the brain** — the full knowledge graph, live.
- This is the concrete form of the standing layout doctrine (home = the main agent; the sidebar carries the rest; one main agent per employee): DSH's own shell stays the host; the Ergon plugin reshapes the front end into the single-session experience and carries the apps + brain entry points in its sidebar section.


## §12 — No pre-packaged apps; one private repo per company (Cayleb, 30 Oct 2026 ~07:52–07:53 ET)

- **Ergon ships with no pre-packaged apps.** What ships is the **formula** for how apps are set up — the app-building template and creation blueprints (§§8–9). Apps are built per company, specific to that company.
- **Each company gets its own separate private GitHub repo** holding that company's full personalised suite: their apps, their skills, their tools, and each of their personalised agents — the company's whole suite in one place, effectively its backup.
- Why (Cayleb's reasoning): the company's apps live in the cloud, so agents working in the cloud can work on them with no friction; and the separation is obvious — one company's suite never mixes with another's. Organisation and isolation are the point.
- The platform/formula lives in the Ergon project itself; company repos hold company instances. (For RHL: the warehouse board is RHL-suite content under this model — the repo move is a consolidation/weekend-orders matter, not a demo-morning change.)


## §13 — Command surface rules (Cayleb's live board test, 30 Oct 2026 ~08:02–08:03 ET)

Cayleb tested the live board himself. Visuals pass. The command layer does not — his findings, generalised to every app:

- **Free-form or it doesn't ship.** Real commands must work: the user can say whatever they want, in any phrasing, and the agent interprets and executes (add / remove / search / edit). A fixed command grammar, canned phrasing list, or a sample-only path is a defect — the board's bubble demonstrably only handled the sample path ("add sample panels to line C"), which Cayleb caught in under a minute. The bubble routes through the paired agent (§8); acceptance for any command surface is a checker-run set of varied, natural, awkward phrasings executing correctly, not the developer's own examples.
- **No clutter chrome.** Suggestion/example chips are removed. Utility buttons the agent makes unnecessary — the board carried Refresh and Print — are removed. If the agent can do it from the bubble, it doesn't get a button.
- **Voice setup is invoked, not displayed.** The voice-setup affordance appears only after the user taps the voice button (and voice is not yet activated on that device). After activation, the standing "Voice control activated" badge (§7) remains the only persistent element.

## §14 — Brain-backed retrieval (Cayleb, 30 Oct 2026 ~08:03 ET)

- The biggest coming addition to the board (and the pattern for data-carrying apps): **pictures and automatic retrieval of job information**, powered by the company back-end brain. Once the brain is working, the app pulls **everything known about the job sitting on a warehouse line** for whoever is using the tool.
- Retrieval fires **while the job is being created** (the agent enriches the new entry from the brain) **and whenever the user asks**. This works because the personalised agent is brought along inside the app (§8 pairing) — the brain is the agent's knowledge (company DB + knowledge graph), not a separate search feature bolted onto the app.
- Build home: the weekend build (company-DB wiring + brain), not the demo-morning path.


## §15 — Professional finish: motion is part of the product (Cayleb's live test, 30 Oct 2026 ~08:04 ET)

- Cayleb's grading: the board's animations feel **lacklustre**, and here that hurts more than it helps — the board must feel like a **professional website / professional app**, and it currently sits below the standard he has seen Claude (or Nox) produce. "Looks okay" is not the bar.
- Consequence: motion quality is a graded product requirement, not decoration. The front-end landing (sidebar shell + logo, §10) carries an **Annnimate-baseline motion pass** under the standing design stack (Impeccable / Taste / Kowalski motion discipline): purposeful transitions, no dead cuts, nothing that feels like a template demo. Acceptance on feel is Cayleb's grade, on top of the checker's run.
- Sequencing stands: the free-form command fix (§13) lands first; the motion pass rides with the front-end landing; Roy receives the finished article.

## §16 — Legend and label accuracy (Cayleb's live test, 30 Oct 2026 ~08:04 ET)

- The dashboard legend showed the tag **JAN as "Janice"** — an agent-side assumption from the morning's colour mapping. Cayleb struck it: **JAN is not Janice; the identification comes off the board.** (Janice's tag is JS. JAN displays as its own tag, unassigned, until Cayleb names who Jan is.)
- General rule: person mappings in legends, labels and colour keys come only from a client-confirmed mapping. An agent's guess never ships as a displayed identity — unconfirmed tags display as themselves.


## §17 — No explanatory sections (Cayleb's live test, 30 Oct 2026 ~08:05 ET)

- The board carried a little explanatory section describing its own layout ("warehouse lines — one row per line; jobs show across the row; on a narrow screen swipe the tablet sideways for more jobs…"). Cayleb: **that whole section doesn't need to be there — remove it.** The board must be self-evident; help copy explaining the layout is clutter chrome (§13's rule applied to text).
- What stays, because it was done well (his words): the legend chips — **JA, JS, JG, AK, LB showing their colours and the person each is associated with.** That is the whole explanatory apparatus the board needs.


## §18 — Manual entry stays first-class (Cayleb's live test, 30 Oct 2026 ~08:05 ET)

- Alongside the command bubble, **direct manipulation is required**: a user can click any box on the board and get a pop-up screen (a proper form) where they fill in all the job information manually. The bubble is not the only way to edit; tap-to-edit is a core path, not a fallback.
- **Automatic retrieval is part of that flow too** (§14): when the form opens / as the job is identified, the agent brain pulls what's known about the job and assists the entry — retrieval is not a bubble-only feature.
- Build split: the form itself is core board functionality (current fix cycle); its retrieval assist activates with the brain wiring (weekend build) and the form is designed with that hook in from the start.


## §16 correction (Cayleb, 30 Oct 2026 ~08:11 ET)

Two corrections to the section above. (1) **Attribution:** the JAN→Janice identification was Outis's error on the board, not a Nox assumption — Cayleb's correction was addressed to Outis. (2) **Resolution:** JAN does not remain an unassigned tag. **JAN was meant to be JA** — the JAN-tagged job is one of Jessica's jobs and is marked as Jessica's (JA, blue). The general rule in §16 stands unchanged.


## Colour mapping correction (Cayleb, 30 Oct 2026 ~08:13 ET)

**LB (Linda) is purple**, not yellow as first recorded this morning. The confirmed board legend stands as: JA Jessica blue · JS Janice pink · JG Joe orange · AK Anna green · LB Linda purple · JAN resolves to JA (§16 correction).


## 19. Hosted access — nobody installs anything to USE an app (Cayleb, 30 Oct 2026, 08:50 ET)

Apps are **hosted on one machine** (today: Gio's desktop, RHLCO-WKS-09) and **everyone on the network uses the same apps through a shareable URL**. No per-user installs for app use — Roy's "package" framing is dead; Roy gets a link. Consequences:

- **Ergon Lite, clarified in Cayleb's words:** Lite has **the full functionality of the finished Ergon project, without installing anything separate** — full functionality delivered through the hosted URL. The Ergon installer (§ structure rulings, 08:41) is how a HOST gets set up; Lite is how everyone else uses the result.
- **The URL is never localhost.** Apps are served at a real, shareable name on the network (the host's network name or an ergon.local-style name), openable by anyone on the Wi-Fi. Cloudflare Pages does NOT fit this: it is static hosting, and Ergon apps are live server apps — data and the paired agent live on the host. Public reach, if ever wanted, is a separate tunnel decision with its own security conversation, not a default.
- **Agent profiles and keys live on the host only.** They never travel to a user's machine, because there is nothing on a user's machine to hold them.


## 20. The agent lives on the company cloud host — mobile works with the desktop OFF (Cayleb, 30 Oct 2026, 09:04 ET)

**The mobile personalised agent must work even if the desktop is off.** This supersedes the earlier "phone as a window onto a desktop-resident agent" framing: the agent's home is the **company cloud host** (the dedicated per-company host, § hosting decision 09:01), and the phone and the desktop are both windows onto it. A lot of the company layer is cloud-hosted for a stated reason: **management changes and device changes must not become a problem** — a PC off, replaced, or reassigned, or a manager leaving, breaks nothing, because the agent, the brain and the apps never lived on that PC. Acceptance shape: with the desktop off, the phone agent still answers, acts, and reaches the apps end to end.


## §21 Voice is the agent's plugin, not a service (Cayleb, 30 Oct 2026 09:39 ET)

The DeepSeek Harness voice input plugin — the one Cayleb flagged on 29 Oct — is to be ported over with the personalised agent and becomes **the voice input ability and connection of every app the agent pairs with**. Voice travels with the agent; apps do not build their own voice wiring, and there is **no central hosted voice service**: this section supersedes §7's hosted-service model (self-hosted faster-whisper + Kokoro behind one endpoint, Groq stopgap) as the architecture. Standing verification duty on the weekend plugin build (Order C, first job): establish what the plugin actually does — capture only or its own transcription, what it calls if anything, input only or output too, and whether it runs outside the harness's own interface — and report any gap against this ruling immediately rather than quietly rebuilding a service behind it. Phonon-2 (intaked 30 Oct) is retained on file as the fallback engine candidate if the plugin proves to need an external speech-to-text engine; the Phonon-2 vs faster-whisper head-to-head is retired.


## Addendum — RHL may have its own database hosting (30 Oct 2026, via Roy)

From Cayleb's conversation with Roy (30 Oct, ~10:39 ET): RHL might have its own database hosting service. Not confirmed. If they do, the company-database wiring adapts to it; if they don't, Supabase stays the plan, as designed. Design consequence: the company-DB connection (weekend Order D) keeps the database provider swappable rather than hard-wired to Supabase.


## §13 addendum — the command layer IS the agent (Cayleb, 30 Oct 2026 10:45 ET)

Cayleb's first live request on the finished board — "add the Hartford PL job to line D" — was refused with "I couldn't confidently match the action and target. Please clarify the request." Cause, in his ruling: the back end matched his words against preset intents behind a confidence gate. That architecture is banned. The command path is the agent itself, wired into the app with the full board state and the brain's knowledge of the jobs, interpreting free-form language and executing directly. Clarification is permitted only when information is genuinely missing (no job and no line identifiable at all), never as a confidence fallback on a complete command. Acceptance sets must be written in the user's own fresh phrasings — a set the implementation's authors drafted in phrases the matcher already handles proves nothing, as this failure showed (25/25 + 9/10 on drafted sets; the first real command failed).


## §14 addendum — per-job information: OPEN QUESTION, nothing decided (Cayleb, 30 Oct 2026, corrected twice — final position 10:55 ET)

The board today holds only jobs, initials and notes. Where the full per-job information lives and how it would reach the job record is **not known for sure and not being set up yet**. What follows is current-practice colour, offered by Cayleb (30 Oct ~10:48–10:55 ET) as background for improving the current board work — how job information and delivery sheets currently circulate around Gio's setup: the PMs appear to hold most of it, arriving by email and on sheets, and Anna files material to an Outlook calendar for download/print. An ideal was floated in the same conversation — PMs connected directly with their own agents, filing straight to a shared database — and Cayleb ruled explicitly (10:55 ET) that this too is **not a decision**: "we don't know for sure; that was just what we heard." What stands is §14's principle — click a job and get its full breakdown from the brain — with its data source **to be determined**. One concrete artefact exists: Cayleb is sending Outis one day's Outlook sheet as an example of the per-job data's shape.


## §10 addendum — client branding placement (Cayleb, 30 Oct 2026 11:06 ET)

Every app in the suite carries the client's actual branding — for RHL, the real RHL logo and name — placed in the top corner of the app. Not a placeholder mark, not Ergon branding in the app's face: the client's own identity, top corner, on every app.


## Management piece — fleet efficiency audit + one improver at two altitudes (Cayleb, 30 Oct 2026 11:27 ET; shape = Nox's recommendation, put to Cayleb the same minute)

Cayleb's requirement: the management software runs a **daily pass over every employee's agent** hunting inefficiency — above all token waste — because the suite must stay budget-friendly; his starting anchor is **~$2,000/month in API credits** per company (credits only; final budget is the company's call). He also wants the management layer to be the **main recursive improver** of the suite, the agents and the harness, with priority over individual agents' self-improvement, and asked Nox directly whether meshing that with the existing per-agent self-improvement would work or actively worsen the system.

Nox's answer, given to Cayleb 11:27 ET: **mesh it — one improver, two altitudes; never two competing improvers.** A second independent RSI engine with runtime priority over the agents' own learning would actively hurt: conflicting rewrites of the same skills/tools (thrash, no stable baseline), double improvement spend burning the budget the auditor exists to protect, no attributable cause when something breaks, and the central layer silently overriding what an agent learned about its own employee — which fights the personalisation premise. The working shape:
- **Local loop (already designed):** each agent improves its own tools, skills and workflows; ReviewDesk gates promotion.
- **Fleet loop (this management component):** the daily pass (1) audits efficiency — token spend per agent, employee and task class, outliers flagged; (2) harvests proven local lessons/tools/skills and promotes them fleet-wide through the same ReviewDesk gate; (3) proposes suite/harness-level changes — proposals, never silent edits (Harness² rule: always-on changes need sign-off; the base harness is never rewritten by the loop); (4) enforces budget policy — per-employee fair-share quotas via the budget proxy, rebalanced from the fleet view.
- "Priority" is therefore **policy and promotion priority** (the fleet layer decides what gets shared, budgeted, retired), not edit priority over a live agent's learning. One method — Harness² — with the management app as its fleet-altitude home.


## Voice calls — no hosted voice service anywhere (Cayleb's direction + Nox's assessment, 30 Oct 2026 11:32 ET)

Cayleb restated §21 (dictation = the DSH voice plugin) and pushed on calls: he wants the cloud voice step removed entirely if efficiency allows, noting the work desktops are resource-poor. Nox's assessment, given to Cayleb the same minute: **calls need no hosted voice service either.** The split that matters: dictation quality on modern small local models is within ~1% WER of cloud large models on clean near-field English (Phonon-2 evidence, 30 Oct intake) — the gap only widens with noise, distance and multiple speakers. For calls, the perceived quality difference is latency and TTS naturalness, not transcription accuracy. The architecture that fits weak desktops: **local ears and mouth** (small-model STT + Kokoro-class TTS — a burst of one modest CPU core and ~1 GB RAM per machine) **with the API model as the brain** — and the brain is a cloud API call in every part of Ergon already, so it is not an extra voice-hosting step. What genuinely does not fit weak desktops is large-model local STT or a local LLM; neither is needed. Net doctrine: no hosted voice service exists anywhere in the stack — dictation via the plugin (§21), calls via local edges + the agent's existing API brain. The plugin-capability verification (weekend Order C, Amendment 16) remains the check this inherits from.


## §19 note — future reach beyond the work Wi-Fi (Cayleb, 30 Oct 2026 11:53 ET)

When the board's phone proof passed, Cayleb first named Tailscale-style hosting as the scale-out for off-Wi-Fi use, then **scratched it himself within the minute (11:54 ET)** in favour of the architecture already on the books: under the full setup, apps and agents live on the company cloud host (§20) and the agents are interconnected there, so **any enrolled device reaches its agent and the apps from any network — the same-Wi-Fi requirement is an artefact of today's LAN demo hosting, not of the design**. Tailscale remains a possible implementation tool, not the plan. The current artefact stays LAN-hosted per §19.


## Enrolment — access is per-device (Cayleb, 30 Oct 2026 11:54 ET)

Cayleb's explicit access model, endorsing the security baseline's per-device design ("kind of like you were saying how Amazon does it"): **Ergon is installed on a device, and that installed device — only that device — is bound to that company's server.** Access rides on the enrolled device, not on the network and not on a shared password: an enrolled device reaches its agent and the suite's apps from any network (§20 residency is what makes that possible); a device that was never installed gets nothing, even with the address. Binding is per device, and revocation is per device from the management side — a lost or replaced machine is cut off individually, which is also what makes §20's "device changes break nothing" safe to offer.


## §10 addendum 2 — Ergon is the architecture, not the face (Cayleb, 30 Oct 2026 11:58 ET)

Sharpening the branding rule: the shell keeps its baseline templatised look, and the **top corner carries the client's company logo** — not Ergon's. Cayleb's reasoning, recorded because it governs every branding choice: the suite is *the client's* custom suite of apps and agent fleet; **Ergon adapts to them — Ergon is the underlying architecture, not the brand on the glass.** Related packaging note from the same minute: the RHL warehouse app is the first app — packaged separately, installed onto the RHL-specific Ergon setup (per §12's per-company model).
