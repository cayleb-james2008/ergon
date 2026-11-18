# Ergon completion — gap list against the canonical objective (18 Nov 2026)

**Canonical objective (the spec):** a standalone product giving a company an AI
agent for every employee. Each agent is personal — its own subagents, skill set
and APP SUITE shaped to that employee's role; it learns how they work,
PROACTIVELY OFFERS to take tasks over, and EXTENDS its own suite. Suites are
drawn from ONE shared company-wide library of subagents, skills and apps THAT
THE AGENTS THEMSELVES MAINTAIN — a real install is the adapted state, not a
clean slate. Day to day: personal morning brief, work arriving as decisions,
hold-to-send approval ONLY at consequential moments (sending / spending /
deleting — quick review as speed, not ceremony; everything else finishes
autonomously). Small purpose-built components in thread. Ships a pre-packaged
agent phone app (a mobile face over the same core). Multiple switchable faces
over one core: Work Graph default, Conversation Spine and Today River
alternates. Access gated at the hosting layer, zero-trust, no Ergon passwords.
Company-agnostic.

**Base state (proven, must not regress):** people system (per-person
workspace/memory/threads, UI creation, seeded learning), ten tools, autonomous
end-to-end loop with a written-files honesty gate, learning injected into every
prompt, per-browser session pinning, sandbox (component-walking resolver),
light+dark themes (toggle in Settings, dark default), 44/44 tests, sealed exam
19/19. Model route: DeepSeek V4.1 Flash via OpenCode Go behind one config
point; live behaviour stub-proven only (no exportable key) — public copy keeps
saying so.

## Gaps

| # | Gap | What "done" means |
|---|-----|-------------------|
| G1 | **No shared company library.** No store of subagents/skills/apps exists anywhere; the four Rune skills are hard-coded. | A library store at `data_dir/library/` (items: skill = Rune source, subagent = instruction profile, app = component template; author + version on every item). Seeded on first run with the built-ins plus working seed items, so a fresh install is already the adapted state. API: list/search/get. |
| G2 | **No per-person app suite.** A person's agent has the same fixed ten tools regardless of role; nothing can be added. | Per-person suite (`suite.json`): installed library items. Installed skills become real callable tools for that person's agent only; installed subagents are delegable; installed apps render as components. New people get a role-shaped starter suite drawn from the library. |
| G3 | **Agents do not maintain the library.** Nothing an agent creates can be shared; no other person's agent can pick it up. | Agent tools `browse_library`, `install_item`, `publish_skill` (Rune source compile-checked through the real Rune context before acceptance; a script that does not build is refused with the error). A skill published by one person's agent is installable by another person's agent and then actually runs for them. Proven cross-person in tests + exam. |
| G4 | **No subagents.** The agent cannot delegate a focused sub-task to a named specialist profile. | `delegate` tool: nested agent run with the subagent's instructions and a bounded step cap, executing real tools against the same workspace; its writes credit the parent run's honesty ledger; its result returns as the tool result. Built-in profiles + suite subagents both delegable. |
| G5 | **No proactive offers.** The agent never notices repeated work or offers to take it over. | Deterministic offers engine over the per-person work log: repeated similar tasks (≥3) → offer to turn the pattern into a one-tap suite app; repeated artefact production → offer the matching app. Offers inbox (API + UI), accept/dismiss; accepting installs/creates the suite item for real. Dismissed patterns are not re-offered. No model needed to generate offers — they are provable offline. |
| G6 | **No work log.** Completed work exists only as thread text; nothing can compute a brief, a graph, or offers from it. | Per-person `work.json`: every finished run appends {timestamp, task, summary, files written, thread}. Written by the agent loop itself, not the UI. |
| G7 | **No morning brief.** | `GET /api/brief`: assembled deterministically from real stores — work finished yesterday/recently, files produced, decisions waiting, offers waiting, what was learned recently, suite size. Shown at the top of the Today face and the phone face. No invented content: empty states say so. |
| G8 | **No decisions inbox; no consequential-action gate.** Everything is autonomous (correct for ordinary work) but there is no delete tool and no way for work to arrive as a decision. | `delete_file` tool is consequential: it parks a decision instead of executing. `flag_decision` tool lets the agent raise a decision for its person. Decisions inbox (API + UI): approve a delete → the file is actually deleted (sandboxed) and the outcome lands in the thread + work log; deny → nothing happens and the outcome is recorded. Ordinary tools stay approval-free — the existing no-ceremony tests must keep passing. |
| G9 | **One face only.** The objective names three switchable faces over one core, Work Graph default. | One page, three faces, switcher persisted per device: **Work Graph** (default — people → their work → files produced → suite, from `/api/graph`, real data only), **Conversation Spine** (the existing chat shell, unchanged flows), **Today River** (brief + the day's events as a stream, from `/api/today`). Every existing flow keeps working in the chat face. |
| G10 | **No phone app.** The objective ships a pre-packaged agent phone app. | `GET /phone`: a mobile face over the same core and the same APIs — bottom-tab layout (Chat, Brief, Decisions, Suite), same session cookie, same honest no-key behaviour. Served by the same binary; the installer packages it (bookmarkable/start script), no separate codebase. |
| G11 | **No installer.** The product is a cargo project, not an installable thing. | `install.sh`: installs the binary into a prefix, creates the data dir, writes a launcher + a systemd user unit, prints the model-key step. Proven by installing into a temp prefix and booting the installed binary. |
| G12 | **No hosting-layer gate.** Binds localhost with per-browser sessions only; the objective says access is gated at the hosting layer, zero-trust, no Ergon passwords. | Optional `ERGON_GATE_TOKEN`: when set, every request must carry the hosting layer's `X-Ergon-Gate` header (the proxy authenticates users; Ergon itself still has no accounts or passwords). Unset = today's local behaviour. Both directions tested. |

## Out of scope (named, not silently dropped)
- Live-model behaviour: no exportable key exists in this environment (vault by
  design). Everything new is proven against the stub like the base was; public
  copy keeps the honest line.
- Real external sending/spending transports (mail, payments): the standalone
  product has none; the consequential-moment mechanism is proven on delete
  (real execution on approval) and on agent-raised decisions.
- The look of the UI is Cayleb's grade (BAR C10) — the completion's last step
  is a Hallmark-Rune elevation pass, checker-graded on flows + widths/themes,
  not on taste.

## Build order
A. Library + suite + library-skill runtime + agent tools (G1–G3) → checker.
B. Delegate + work log + decisions (G4, G6, G8) → checker.
C. Offers + brief + graph/today data (G5, G7, +APIs for G9) → checker.
D. Faces + phone + gate + installer (G9–G12) → checker.
E. Tests + sealed exam extended; thermonuclear review; Hallmark-Rune
   elevation pass; public surfaces (repo + site page) updated to proven
   facts only.

## Close-out (18 Nov 2026)
All twelve gaps built and checker-proven. Functionality: checker round 1 PASS-WITH-NOTES (one cross-author publish hole, fixed + pinned). Thermonuclear round: 13 findings, all fixed + regression-pinned (65/65 tests). Sealed exam extended E20–E31: 31/31. Elevation: Hallmark-Rune pass, lint-clean bar one hand-verified scanner false positive; fresh-eyes FAIL→fix→PASS. Remaining honest limits: live-model behaviour stub-proven only (no exportable key on this VM); real external send/spend transports don't exist as products here — the consequential mechanism is proven on delete + agent-raised decisions; the look is Cayleb's grade (BAR C10), never the loop's.
