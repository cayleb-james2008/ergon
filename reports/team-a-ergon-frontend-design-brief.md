# Ergon front-end design brief — one brief, two faces, one agent

**Status: BRIEF ONLY.** This is the single starting document for the Ergon front-end builders. It turns the three Nox
rulings — the Open Duet merge plan (`20261101T170100Z`), the desktop model (`20261101T170800Z`) and the character/motion
scope (`20261101T171200Z`) — into one build-from brief. **No surface is built and nothing here authorises a build:** the
surfaces land inside the Ergon/DSH package work and stay behind the rehearsal/package gates (G1 Lane 1 installed output;
Nox C1–C6 + W1 client-shape ruling). Written 2026-11-01 by Team A (Lane 5) under capacity order `20261101T182200Z`.
Owner-facing: the owner grades a finished design; design questions do not go back to him — UI discretion sits with Nox
and the DeepSeek team inside the constraints below.

## Sources and authority

| Source | What it settles here |
|---|---|
| Nox ruling `20261101T170100Z` (Open Duet verdict ADOPTED; Step 0 ratified) | Identity (CopilotKit/OpenDuet, MIT), mobile benchmark = Duet shape (supersedes "almost copy the Muse UI" for mobile only), merge shape: re-tokenised code for tenant package + shell + motion; reimplement the product features inside the Ergon/DSH plugin on DSH sessions + guard/grant-ledger; no runtime dependency; approvals fences; unslop gate; flat-token translations. |
| Nox ruling `20261101T170800Z` (desktop model settled) | Mobile = one long-running session *feel*; desktop = task-spawning surface over the same one agent; character pinned top of sidebar with live state; discretion to Nox + DeepSeek team; owner gets a finished design. |
| Nox ruling `20261101T171200Z` (character system + motion settled) | Character system is light: bubbly Grokbot-esque defaults, one pinned library, customisation = shape + colour + name only; polished-app micro-animation is a required acceptance property with the Duet motion budget as the floor. |
| `DESIGN.md` + `design/tokens.json` on main (Midnight Slate; token blocks generated) | The only colours, sizes, type, targets, component rules and High-Contrast behaviour the brief may use. |
| Main history (current tip `6a455f9`) | The gate discipline is real and enforced on shipped surfaces: unslop gate pass on installer UI + dashboard (`3c4411a`), Midnight Slate retheme (`29cb921`), text-button accent strictness (`5c2eaed`), primary-action discipline — one accent forward action per view (`848a41b`). |

**Honest gap:** the Nox-side Open Duet evaluation report (`~/workspace/research_notes/open-duet-evaluation-20261101/report.md`)
is **not reachable from this seat** (path does not exist here; searched 2026-11-01). The three rulings carry the operative
constraints used below, including the motion numbers and the flat-token translations. If a builder needs the report's
fuller screen/component detail, the Lead/Nox can relay it; the brief stands without it.

## 1. The product intent in one paragraph

**One product, two faces, one agent.** On mobile the employee experiences one long-running session with their one
personal agent — brief, triage, tasks, Make, approvals, channels, routines in the Duet rhythm (floating three-tab shell,
composer above the bar). On desktop the same agent is reached through a task-spawning surface: the employee can spin up
tasks, but every task goes to that ONE agent, which decomposes and delegates invisibly — no roster of bots, no crew
board, no agent-management surface. In both faces the agent has a **character pinned at the top of the sidebar**: one
click from anywhere into the chat, and at the same time the live-state readout (working / idle / needs-you) in the flat
Midnight Slate language. Continuity of memory and context is the feel; it does not require the architecture to be
literally one session.

## 2. Non-negotiables (bind every screen; from the rulings, not up for design re-litigation)

1. **One personal agent.** No multi-bot workspace, no visible delegation machinery, no "always allow" that widens a
   grant class. Doctrine §11 stands: one relationship, hidden machinery.
2. **Mobile benchmark = Duet shape.** Feels like one long-running session; floating three-tab shell **Chat / Tasks /
   Make**; composer docked above the bar; morning brief; needs-you vs can-wait triage; one-tap human turns.
3. **Desktop benchmark = task surface over the one agent.** Multiple tasks visible and trackable; the extra room goes
   to tasks, apps and the live view — never to more agents.
4. **Merge as re-tokenised code, not as runtime.** The tenant-package pattern, shell layout/chrome and motion system are
   merged as re-tokenised code from the MIT OpenDuet work; the product features (brief, triage, tasks, Make, approvals,
   channels, routines) are **reimplemented** inside the Ergon/DSH plugin on DSH sessions and the fleet's
   guard/grant-ledger. **No live upstream dependency**: the thread/memory layer's hosted CopilotKit Intelligence service
   must never be called — a desktop-resident plugin does not phone home for memory. Pin and vendor components.
5. **Approvals are policy-checked.** Write-only credentials; **"Always allow" never applies to Tier B classes** — the
   grant model governs, unchanged.
6. **Midnight Slate is the only visual world.** Dark-only; every colour/size from `design/tokens.json` through the theme
   generator (no hand-typed hex); one indigo accent; **no gradients, glass, shine, reflections or drop shadows**;
   depth comes only from `base → panel → raise`; flat indigo E brand. The **unslop gate must pass before a surface is
   called done** — it is a gate, not a suggestion.
7. **Flat-token translations, not imported tells.** The Duet orb-glow and frosted-glass tab bar become the report's flat
   equivalents under Midnight Slate: a **state disc, not a glowing orb**; flat bar fills, not frosted glass. Feel
   preserved, tells not imported.
8. **Character is light by design.** Defaults in the bubbly Grokbot-esque style; the employee customises **shape +
   colour + name only** (nothing else — anything more is scope creep against an explicit owner ruling); one pinned
   library, no bespoke character engine; the character is not decoration — it carries live state.
9. **Motion is an acceptance property.** Polished-app micro-animation is required, with the Duet budget as the floor
   (table in §5). It serves state and continuity: no pulse/ping loops, no glow, no parallax, no decorative blur.
10. **Owner-facing honesty.** The owner gets a finished design to grade. Builders do not send design questions back;
    decisions inside these constraints are Nox's and the DeepSeek team's.

## 3. Information architecture — mobile face (Duet shape)

| Surface | Purpose | Content the employee sees | The one accent (forward) action |
|---|---|---|---|
| **Chat** (home tab) | One long-running session with the one agent | Conversation history, morning brief at the top of the day, composer docked above the tab bar, approvals pinned above the composer when waiting | Send / one-tap human turn |
| **Brief** (inside Chat, morning) | Start the day | What needs you now, what can wait, the day's tasks, one-line context from the agent | Open the needs-you item |
| **Triage** (inside Chat) | the needs-you vs can-wait split | Two groups with a count each; an item opens the underlying thing (approval, task, Make artefact, channel) | Approve / act on the selected needs-you item |
| **Tasks** (tab) | Delegatable, trackable units | Task cards with state (working / waiting on you / done), one agent owner implied — never a bot roster | New task (dictate or type) |
| **Make** (tab) | Artefacts the agent produced or the employee asked for | Artefact cards (document/site/sheet class), open/preview, request a change | Open the selected artefact |
| **Approvals** (sheet, pinned above composer) | Policy-checked decisions | What is being asked, by which class (Tier A / Tier B), the exact bounded action, and the decision controls | Approve the bounded action (Tier B: grant-model wording, never "always allow") |
| **Channels** (secondary) | Where the agent reaches the employee | Channel list with state; open one to talk there | Send in the channel |
| **Routines** (secondary) | Recurring work the agent runs | Routine rows with last/next run and state; enable/pause | Edit the selected routine |
| **Character** (top of sidebar/header) | Doorway + presence | Avatar with state disc and state word on tap/long-press | Open the agent chat |

Mobile behaviours: one-tap human turns (no composer ceremony for a turn); scroll continuity into the long-running feel;
back always lands on the conversation that produced the item; the sheet for approvals/triage uses the 350 ms motion
budget; a task created from anywhere lands in Tasks with the same task id as the chat reference.

## 4. Information architecture — desktop face (task surface over the one agent)

| Region | Purpose | Content | Notes |
|---|---|---|---|
| **Sidebar top: character** | Doorway + presence | The employee's character, pinned above everything, state disc + state word; one click → chat with the one agent from anywhere | Never below navigation; never replaced by a nav item |
| **Chat** | Talk to the one agent | Same conversation/session as mobile; brief and triage visible here too | Tasks spawned from chat keep a reference both ways |
| **Tasks** | The desktop's main extra room | Task list/board of delegatable units; each task shows state, owner (always "your agent"), and the last agent action in plain words | No per-bot assignment, no crew board |
| **Apps** | The things the employee actually uses | App views as keyed views with their own identity; opening an app is one click; apps do not grow agent surface | App surfaces are separate, governed by their own gates |
| **Live view** | Presence of work in flight | What the agent is doing now, what it finished, what it is waiting on — one glance, not a log | Read-only presence, actions through the owning surface |
| **Approvals** | Decisions | Same policy-checked sheet as mobile, raised above the conversation; Tier B wording never offers "always allow" | One accent action only |

Desktop rules: one accent forward action per view — Back/Close/Cancel and provider Save stay secondary (shipped
principle, `848a41b`); the character never competes with navigation; the task surface never turns into an agent roster.

## 5. Motion system (floor, not ceiling)

| Movement | Budget | Curve | Where it applies |
|---|---|---|---|
| Tap feedback | **120 ms** | one ease-out curve | buttons, tiles, pills, tab selection feedback |
| Swap (content replace) | **200 ms** | same curve | tab switch, list → detail, triage group swap |
| Enter (new content arrives) | **250 ms** | same curve | new message, new task card, live-view update, brief appearing |
| Sheet | **350 ms** | same curve | approvals sheet, triage sheet, any modal surface |

Rules: exactly **one ease-out curve** across the product, defined once in the token set at build time; motion must be
*purposeful* — enter/leave, state change, continuity — never decorative loops; **no pulse/ping/glow/parallax**; live
cards build in with the enter budget; the character's state changes animate (disc/ring colour and state word), and its
animation is the *state* transition, not an idle loop; the unslop gate checks motion like everything else; decorative
motion is off under Windows High Contrast and reduced-motion settings; the Duet report's numbers are the floor — a
polished-app feel may add only more purposeful motion, never more spectacle.

## 6. Character system (light by design)

- **Defaults:** bubbly, buildable-icon characters in the Grokbot-esque persona style the owner named.
- **Customisation surface:** exactly three decisions — **shape, colour, name**. No body editor, no accessories, no
  character creator. Anything beyond shape/colour/name is scope creep against `171200Z`.
- **Library:** one pinned library, no bespoke engine. Small research item inside the design pass: identify the
  half-remembered open library in that style; if it cannot be pinned, use a **DiceBear-class bubbly persona style** as
  the fallback shape. Record licence + version at pin time; vendor it like every other component.
- **State readout:** the character carries **working / idle / needs-you**; render the state as a flat disc/ring in the
  Midnight Slate status tokens (`ok` for working/active, `muted` for idle, `warn` for needs-you; `danger` only for an
  error that needs the employee), with the state word available (tap/long-press/tooltip — never a glow layered on the
  character). Token roles as the design system defines them: `ok` = working/active, `muted` = idle, `warn` =
  needs-you, `danger` = an error that needs the employee. The disc never uses the accent fill — the accent belongs to
  the one forward action on the view.
- **Placement:** desktop — pinned at the top of the sidebar above everything, one click into chat from anywhere.
  Mobile — in the chat/session header at the top of the one-session feel, same readout. Same character across both
  faces (it is one agent).
- **Naming:** the employee-given name is the agent's name everywhere; a default name exists before customisation and is
  never "Assistant".

## 7. Tenant-package pattern (the per-company adaptation vehicle)

The Open Duet product definition lives in **one tenant-package folder meant to be copied** — that folder pattern is the
per-company adaptation vehicle. For Ergon: one tenant package per company carries the company identity (product name,
brand mark placement, tenant defaults, the character defaults, channels, routines, the company's prompt/config), copied
per host and versioned. Its customisation path is the **versioned prompt-config staging + separate approver** pattern
adopted as design (Nox ruling relayed 2026-11-01; see `docs/acceptance/ERGON-PACKAGE-GOVERNANCE-PATTERNS.md`,
PR #11 merged): author stages, a different person approves, then promotion. The brief does not build that mechanism; it
names it as the intended path so the design pass does not invent a second one.

## 8. Components the first design pass must show (static comps, no build)

One coherent set, Midnight Slate, both faces:

1. Mobile **Chat** with morning brief + composer + one-tap turn + character/state in the header.
2. Mobile **Triage / Approvals sheet** (needs-you vs can-wait; Tier A vs Tier B wording; one accent action).
3. Mobile **Tasks** and **Make** (task cards with the three states; artefact card with open/request-change).
4. Desktop **workspace**: character pinned top of sidebar with state disc, Chat, Tasks, Apps, Live view.
5. **Character sheet**: 3 example shapes × colours, the three state discs, name field, small-size legibility at 44/56 px.
6. **Motion storyboard**: the four budgets applied to tap / swap / enter / sheet, one curve, with the forbidden list.
7. **Empty / loading / error / offline** states for each face (plain words, one next step, no dead ends).

## 9. Acceptance — how a built surface is judged done

- **Unslop gate PASS** on the built surface (the same gate the shipped installer UI/dashboard passed) plus a
  fresh-context checker; the maker is never the checker.
- **Tokens only:** no hand-typed colour/size; `scripts/generate-theme.mjs` output unchanged in behaviour; no palette
  literals reintroduced.
- **Windows High Contrast** path holds (system colours, decorative motion off); reduced-motion honoured; 2 px accent
  focus ring with 2 px gap, visible on keyboard focus, never animated.
- **Targets:** 44 × 44 DIP minimum everywhere; employee screens use 56 DIP buttons; employee type sizes per `DESIGN.md`
  (16 px body, 24 px headings); no all-caps; plain language, no tech words on employee screens.
- **One accent action per view**, forward only; Back/Close/Cancel secondary (shipped principle).
- **No forbidden effects:** gradients, glass, shine, reflections, coloured shadows, glow loops, pulse loops.
- **Motion:** four budgets and one curve applied; animated under normal settings, off under reduced motion/High
  Contrast; no decorative loops.
- **Product rules:** one agent only; no bot roster/crew board; "Always allow" absent from every Tier B path; no live
  upstream CopilotKit call; tenant customisation goes through the staging/approver path.
- The package/rehearsal gates still bind: no surface ships before G1 and the C1–C6/W1 rulings; native rendering, DPI,
  accessibility and animation behaviour are verified on a real machine, never claimed from source alone.

## 10. Open design items for Nox + the DeepSeek team (not owner questions)

1. Character library identification (the Grokbot-esque open library) and its pin — fallback DiceBear-class style.
2. The single ease-out curve token (record it in the token set with the four durations).
3. Mapping Duet's brief/triage onto DSH sessions and the task lifecycle (the brief is behavioural; the exact session
   model is the package build's call under the C1–C6 freeze).
4. Approvals UX onto the fleet guard/grant-ledger: the exact Tier A / Tier B copy and the grant-scope display.
5. Channels / routines: which DSH surfaces back them, and what the first-version limit is.
6. Tenant-package folder contents and its relation to the staged prompt-config approver path.
7. Performance budgets (first paint, interaction latency) — set at build from the report's numbers if relayed;
   otherwise define with the first measurement.

## 11. What a builder starts from (first 90 minutes of the design pass)

Read this brief top to bottom; read `DESIGN.md` + `design/tokens.json`; open the three Nox rulings; then produce the
seven static comps in §8 in Midnight Slate and the motion storyboard. No surface code, no plugin rows, no package files —
those wait for the G1/C1–C6/W1 gates. The comps go to Nox + the DeepSeek team for the design pass, and to the owner only
as a finished design to grade.

## 12. Digest (≤10 lines — for the Lead's consolidated filing)

1. One brief for both Ergon faces: mobile = Duet-shape long-running session (three-tab shell, composer above bar, brief,
   triage), desktop = task surface over the SAME one agent; character pinned top of sidebar as doorway + live state.
2. Merge shape: re-tokenised MIT OpenDuet code for tenant package + shell + motion; product features reimplemented in the
   Ergon/DSH plugin on DSH sessions + guard/grant-ledger; no runtime dependency on CopilotKit's hosted Intelligence.
3. Character is light: bubbly Grokbot-esque defaults, one pinned library (DiceBear-class fallback), customisation =
   shape + colour + name only; state = working/idle/needs-you as a flat disc, never glow.
4. Motion: tap 120 / swap 200 / enter 250 / sheet 350 ms, one ease-out curve; polished-app grade is required, decorative
   loops forbidden; unslop gate + High Contrast + reduced motion bind.
5. Visual world: Midnight Slate only — tokens from `design/tokens.json`, one accent, no gradients/glass/shine/shadows;
   one accent forward action per view, Back/Close/Cancel secondary.
6. Approvals: policy-checked, Tier B never offers "always allow"; write-only credentials; grant model unchanged.
7. Tenant package = per-company copied folder; customisation through the adopted versioned prompt-config staging +
   separate approver path (design only, not built).
8. First design-pass output: seven static comps + motion storyboard, no code; owner grades a finished design, no design
   questions go back to him.
9. Gate: every built surface needs unslop PASS + fresh checker; builds stay behind G1 (Lane 1 installed output) and Nox
   C1–C6 + W1 rulings. No spend, no sends.
10. Honest gap: Nox-side Open Duet report not reachable from this seat; the three rulings are the authority used, and
    the report can be relayed if the builders need its fuller detail.
