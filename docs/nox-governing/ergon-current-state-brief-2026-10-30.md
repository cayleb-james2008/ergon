# Ergon — current state of the project

*Context brief prepared 30 Oct 2026 by Nox (Cayleb's agent) for handoff to GLM. This supersedes stale documentation in the ergon repo. Where any repo doc conflicts with this brief, this brief is correct. Also seeded into the repo as the current-state doc in the weekend consolidation (Order A, first step).*

## 1. What Ergon is

Ergon is one project: a company-agnostic template for a complete company software suite, installed per company. It gives every employee a personalised agent that learns how they work, adapts to their workflows, proactively offers tasks to take over, and recursively improves and extends the suite. Around the agents sit shared, company-wide suites (tools/apps, sub-agents, skills) with a review gate, and a management piece (management apps). First company: RHL (Robert H. Lord Company, a furniture company in Manchester, Connecticut), where Cayleb Alvarez-James works as an installer and leads the AI-integration effort. First app: a warehouse lines board for RHL (see §6), installable on its own.

## 2. The decisions that changed the design (29–30 Oct 2026 — these supersede older docs)

- **Harness:** Ergon runs on DeepSeek Harness ("dsh"; plugin framework: Cordis), with Ergon delivered as a plugin. **OpenClaw is OUT** — removed 29 Oct after hands-on setup pain. Any doc naming OpenClaw as the harness/implementation is stale.
- **Model:** the build runs **Haiku 5.5 through Claude (Anthropic provider)**. **DeepSeek R1 is definitively not used. The old "MiniMax is the suite's only model" lock is retired for this platform.** Any doc naming MiniMax as the model route is stale.
- **One project:** **"Ergon Lite" was scrapped as a separate product on 30 Oct** and merged into Ergon. Ergon = suite + personalised agent + management piece. The warehouse board is Ergon's first app. Any doc treating Ergon Lite as a separate product, or a board titled "Lite", is stale.
- **Management:** **Nox manages the library/host side.** There is no "Outis management layer" in the current design — Outis is a build lane (a Codex-based agent Cayleb delegates construction work to), not a management layer and not part of the product. Any doc describing Outis as the management layer is stale.

## 3. Architecture (current)

- One main agent per employee; sub-sessions are its children.
- Sub-agents are ephemeral by default; persistent only as deliberate artefacts in a company-wide sub-agent library, each carrying the same recursive self-improvement loop.
- Three shared suites per company: (1) a combined tool/app suite — one library, one review queue, tool-vs-app is a surface tag; (2) a sub-agent suite; (3) a skills suite.
- Tools and apps are dual-surface by design: agent-friendly (an agent surface + a human UI over shared state) — an agent can drive the whole back end from an employee simply asking, and the human gets a strong front end.
- Review gate (ReviewDesk pattern): agent-built tools and apps stage, get reviewed, then promote into the shared library/suite. Since 30 Oct the same gated admission applies to apps joining the shared suite (functional proof, database wired, dual-surface proof, design conformance, security baseline, recorded admission).
- Memory: a knowledge-graph layer (Kem pattern) over a notes substrate — the notes are the source of truth, every claim carries a citation, the graph is a view/search layer, not the retrieval engine. Session search was rejected; the graph is the memory.
- Front end: familiar Claude/ChatGPT-style chat, built for everyday users; layout: home = the one main agent, sidebar carries everything else. Generative UI (A2UI, pinned v0.9.1) reshapes the full-Ergon front end into the single-agent conversation. Mobile: the phone is a window onto the desktop-resident agent (one agent; the client is phone-attachable from the start).
- DSH plugin rulings: persistent terminals adopted; automatic authorisation review is the default permission mode; Agent Teams plugin rejected.
- The Nox desktop endpoint (full computer-use control of Cayleb's home desktop over Tailscale, token-authenticated, full action log, local kill switch) becomes a downloadable component of the management software only — owner opt-in at install, off by default. It is a management roadmap item, not part of the employee product.

## 4. App doctrine (30 Oct 2026)

- Every app built into a company is fully functional end to end — no stubs.
- One company-wide database; every app connects under its own subsection, organised under categories (an "apps" category), legible to a non-technical browser.
- Dual-surface always (see §3).
- ONE house design style for all Ergon apps (per-company styling was dropped 30 Oct). Baseline component/motion library: **Annnimate** (GSAP-based; components in React, Vue and plain HTML). Design skill stack: Impeccable, the Taste skill, frontend-design, and Emil Kowalski's motion skills (motion only as feedback/state). UX and visuals are a first-order requirement, part of the build, never a follow-up.

## 5. Voice (current)

- Speech is a shared hosted service, not a per-app feature: transcription (STT) and speech (TTS) models hosted centrally — preferably on Cayleb's own desktop / the management server — behind one OpenAI-compatible endpoint; every app calls that one service and employees install nothing. Chosen build (verified 30 Oct): **faster-whisper large-v3-turbo (STT) + Kokoro-82M (TTS)**; Groq's free tier (Whisper large-v3, ~8 audio-hours/day) is the cloud stopgap with the same API shape. Dictation in every app sits behind a single configurable transcription endpoint.
- Dictation is local-first (SenseVoice bundle adopted for local dictation); a live voice call is a cascade: local STT → Claude API → local TTS. There is NO Claude voice API in this design.
- LAN mic pattern: phone browsers block microphones on untrusted LAN pages. The productised fix is an optional in-app one-time setup button (installs trust for a local certificate) after which a badge in the app reads "Voice control activated". Optional; the app is fully usable without it.

## 6. First app — the warehouse board (RHL)

- A spreadsheet-style table of warehouse lines A–S (19 lines), one row per line, holding the job(s) on each line. Jobs are entered/erased by voice (talk to the agent) or typing; each line holds just the job(s) — no customer/time fields.
- ONE control: a command bubble floating at centre-bottom of the screen — tap to type or talk; search, add and edit all run through it; the chart updates in real time. No per-item dictate buttons.
- Colour coding from the real board's sticker system: Jessica (JA) blue, Janice (JS) pink, Joe (JG) orange, Anna (AK) green, Linda (LB) yellow; an initials→colour legend is shown on the board.
- It prints cleanly on one landscape page (the chart is posted by the door). Physical floor end-state: painted lines, one row per letter — the grid mirrors that.
- Data: hosted in the "Ergon" database project, schema `ergon_warehouse`, tables `lines`/`entries` (19 lines, 12 seeded entries from a photo of the paper board). One crossed-out number on line S in the photo is deliberately excluded everywhere.
- Deployment ruling: the board runs open on the work Wi-Fi with no sign-in (the Wi-Fi boundary is a security layer; nothing reachable from outside; no port forwarding). Current demo host: a coworker's Windows PC (RHLCO-WKS-09) on the warehouse LAN; Cayleb tests from his Android phone (a Nothing Phone 3a) over that Wi-Fi.

## 7. Security baseline

- "Amazon-level, Apple-level" security with zero added friction for employees — security lives under the floorboards; any design adding a human-facing authentication burden is non-compliant.
- Human credential decided: passkeys, with a one-time recovery code as lost-device backup.
- Unique per-person/machine credentials in proper storage; least privilege; revocable per-machine tokens; private-network-only access; private repositories; automatic patching via the GitHub update flow; encryption, full logging, backups. Roadmap: a 24/7 always-on cybersecurity agent on the management host.

## 8. Install and distribution

- Packaging: a simple setup script run in a terminal (no EXE wrapper). Per-user install, no admin elevation, no vendor sign-in ever presented, harness ships preconfigured. The installer closes itself on success and launches the intro screen.
- Updates come from GitHub: an in-app "click to restart and update" button, plus a check on start.
- Distribution: private GitHub Releases / a link Cayleb sends personally to named people (Roy first). No code-signing certificate yet (ships unsigned; a real certificate is a pre-broad-rollout purchase).
- API provisioning: one Anthropic team account owned by Cayleb; he personally assigns each install its own key.
- Standard setup on a new machine: install ChatGPT desktop, connect Outis (the build lane), and the lane performs the whole setup.
- Machine lesson (standing): Windows Controlled Folder Access can block writes to Documents — reroute to non-protected folders, never weaken a machine's protections.

## 9. Business model

- One company at a time: full setup for every employee, a 1–2 month settle-in, then a flat monthly retainer per company plus bookable help sessions. Support intake is by email to Cayleb.
- Naming rule: nothing in the product layer is named around RHL; RHL specifics live only in the client layer.

## 10. Repo and current state (30 Oct 2026)

- Repo: cayleb-james2008/ergon (private), main. Current head includes the Parts Handbook (suite/parts-handbook) merged via PR #6. The legacy suite pieces from the first build (library, onboarding, baseline, review-gate, build loop, shell) are in the tree — treat their architecture docs as historical where they conflict with this brief (they predate the DSH pivot).
- PR #5 (an OpenCode 2.0 port draft) ruling: audit against the DSH design, port anything harness-agnostic still needed, close as superseded with reasoning.
- Reviews on record: a GLM full-scope review (46 findings; the five Highs are X-1 — the test runner silently excludes ~180 tests, fix first — X-2 bare JSON.parse at store loads, O-1 onboarding filename-sanitisation collision, L-2 socket destroyed on oversized body, L-3 decodeURIComponent pre-auth 500s) and a second GLM pass (98 findings: 3 critical, 27 high, 44 medium, 24 polish, with a phased implementation plan).
- This weekend (31 Oct–1 Nov): a completion build on Cayleb's home desktop — done means ready to install and set up in any company, end-to-end tested multiple times by fresh-context checkers, and fully code-reviewed. The repo documentation is being brought current as the first step of that build, seeded by this brief.
- The desktop endpoint (see §3) was assessed NOT install-ready on 30 Oct; its remaining gates are part of the desktop work.

## 11. Stale-doc warnings — if you read any of these in the repo, they are WRONG

- "OpenClaw" as the harness or implementation route → replaced by DeepSeek Harness (dsh) + Cordis plugins.
- "MiniMax" as the suite's model, or any pay-as-you-go MiniMax plan → replaced by Haiku 5.5 via the Anthropic provider; DeepSeek R1 is not used either.
- An "Outis management layer" / Outis as management software → Nox manages the library/host side; Outis is a build lane only.
- "Ergon Lite" as a separate product or a separate install → merged into Ergon on 30 Oct; the board is an app of Ergon.
- Per-company visual styles / a "taste matrix" → replaced by one house style (Annnimate baseline).
- Paper-first or "Lite demo" scoping language → the demo constraints were for one morning only; the product bar is end-to-end and installable anywhere.

*End of brief. Work from this; where the repo disagrees, the repo is out of date, not this brief.*
