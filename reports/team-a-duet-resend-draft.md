# DRAFT — Duet post-reset resend pack: decisions list + Pack 20 findings

**STATUS: DRAFT ONLY — NOT SENT. NOTHING HAS BEEN TRANSMITTED.** Prepared by Team A (`ergon-lead`, Lane 5) on
2026-11-01 for **Nox's Monday review** under capacity order `20261101T182200Z` pull item 3, which follows Nox's
2026-10-31 `224700Z` §2: "Duet's Pack 20 resend (decisions list + findings text)" is genuinely Monday-bound because the
Claude lanes are dark until the Anthropic 01:00 EST reset. **No send, no email, no post, no external message has been
made from this draft.** Claude lanes are dark; this is preparation only.

## How to use this file (read before sending)

- The body below (§Message) is written as **one self-contained message** — a catch-up pack Duet can act on without
  re-reading the dark period. Nox reviews it, fixes the addressee/channel if needed, and only then may it be sent after
  the reset.
- **Nox-marked slots (`<NOX: …>`) are places where his record carries text this seat cannot see.** They are not to be
  filled by invention. The Pack 20 narrative report (`P20-F1` to `P20-F22`) and the formal "decisions list" document
  live Nox-side; this seat holds the committed evidence and the board record, not those two documents.
- Provenance for every claim is in §Sources below. Where an F-number is referenced by the committed docs, the evidence
  is named; where it is not recoverable here, it is NOT invented — it is listed as a Nox slot.
- No credential values, no secrets, no money figures outside the already-board-recorded caps, no private content.

## §Sources (for Nox's review, not necessarily sent)

| Source | What it gives |
|---|---|
| Nox `20261031T224700Z` §2; Nox `20261101T182200Z` §3 | The resend is Monday-bound; content = decisions list + Pack 20 F-items; no sending before the reset. |
| Workspace board `board/BOARD.md`, `board/reports/team-a-*.md`, `board/PROGRAMME.md` | Decisions/rulings and job states cited in Part A. |
| Nox board inbox (read-only) `2026-11/20261101T{050327Z,170100Z,170800Z,171200Z,181500Z,182800Z}-nox-to-delegator.md` | The rulings cited in Part A. |
| Ergon repo at `origin/main` `6a455f9` (`docs/acceptance/demo-company/**`) | Pack 20 artefacts, probes evidence, and every F-number reference used in Part B. |
| **Not reachable from this seat** | The Pack 20 narrative report (the `P20-F1`–`P20-F22` text itself); the goal records (`GOAL.md`); the formal "decisions list" document. Confirmed by searching this host 2026-11-01 (the expected Nox-side path `~/workspace/research_notes/...` does not exist here; no Pack 20 report file exists in any local clone/branch). |

## §Message (the draft body — self-contained)

**To:** Duet (the Claude-side lane/agent that receives the post-reset resend — `<NOX: confirm the exact addressee and
channel; the fleet's phase-1 Duet path is a read-only status digest, so if this is that channel, adapt the framing>`).
**From:** the programme delegator, via Nox. **When:** after the Anthropic 01:00 EST reset. **Purpose:** post-reset
catch-up — the decisions you missed while dark, and the findings your Pack 20 work produced, in one place.

### Part A — Decisions and rulings you missed (as of 2026-11-01)

A1. **Jobs 01 and 02 are CLOSED and checker-proven.** Pack 20 merged to `main` as `1c71898`; Job 02 (Pack 21 +
template-hardening scale-up + two residuals) merged as `c196c25`; the three low J2 follow-ups landed as `686fca8`.
Each step: maker evidence → fresh-checker PASS → merge → merged-tree proof with 0 branch-attributable failures. The
Ergon code base on `main` is checker-proven; do not rebuild it.

A2. **Job 03 (dress rehearsal + RHL deployment) is PREP-ONLY and Team A owns it.** Nox ruling `20261031T050327Z`:
Team A is maker end to end (candidate assembly, the rehearsal run, the fix loop); a fresh-context rehearsal checker who
built no part of the candidate runs acceptance on the installed output; a handback that has not run live against the
installed package is a progress report, never a completion. The R0–R7 step labels, run sheet and package index are
filed on `main` (PR #10 `6bcd4eb`). **The RUN is unscheduled** until the installed output exists (gate G1).

A3. **The three-piece package contract C1–C6 is recorded as PENDING Nox, and the client-shape arbitration is open.**
C1 freeze at DSH `0.2.2-alpha.2` (+ one recorded delta review when 0.2.2 lands); C2 pin alpha.2 until Job 06's release
is installed and checker-proven; C3 only the shell's declared slots + `locale` inject — no plugin-manager, add your own
keyed `main[key]` view, never replace the stock conversation view without arbitration; C4 client-contract page still
owed; C5 Linux-only bundle + separately installed Windows companions; C6 private pinned tarball + sha256, no public
registry publish. **Nothing is in force until Nox confirms**, and nothing is built on the open arbitration (ADD-only
`main[key]` W1 view vs a documented reshape of the chat view).

A4. **Open Duet verdict ADOPTED as the mobile benchmark (ruling `20261031T170100Z`).** Identity settled as
CopilotKit/OpenDuet (MIT, created 21 Oct 2026); the mobile endpoint's governing benchmark is the Duet shape (this
supersedes "almost copy the Muse UI" for mobile only; desktop benchmarks unchanged). Merge shape: **re-tokenised code**
for the tenant-package pattern, the shell chrome (floating three-tab Chat / Tasks / Make, composer docked above the bar)
and the motion system; **reimplement** the product features (brief, triage, tasks, Make, approvals UX, channels,
routines) inside the Ergon/DSH plugin on DSH sessions and the fleet's guard/grant-ledger. **Neither codebase is a runtime
dependency** — the thread/memory layer's hosted CopilotKit Intelligence is never called; components are pinned and
vendored only. Their approvals gateway's "Always allow" **never** applies to Tier B classes; the grant model governs.
The ported shell must pass the unslop gate before it is called done; the orb-glow and frosted-glass bar are ported as
flat-token equivalents under Midnight Slate (a state disc, not a glowing orb).

A5. **Desktop model settled (ruling `20261031T170800Z`).** Desktop = a task-spawning surface over the SAME one personal
agent: multiple tasks visible, every task sent to the one agent, which decomposes and delegates invisibly; the extra
room goes to tasks, apps and the live view — not to more agents. The agent has a character pinned at the **top of the
sidebar**, one click into chat from anywhere, carrying the live-state readout (working / idle / needs-you). UI design
itself is delegated to Nox + the DeepSeek team inside the standing constraints; what comes back to the owner is a
finished design to grade, not questions.

A6. **Character and motion settled (ruling `20261031T171200Z`).** Character system is light: bubbly Grokbot-esque
defaults, one pinned library (DiceBear-class fallback; library identification is a small research item), and the
employee's entire customisation surface is **shape + colour + name** — nothing else. The character carries live state,
rendered flatly (a state disc/ring), not as effects. Motion is a required acceptance property, with the Duet budget as
the floor: **tap 120 / swap 200 / enter 250 / sheet 350 ms, one ease-out curve**; no pulse/ping loops, no glow; motion
serves state and continuity and the unslop gate checks it.

A7. **The one front-end design brief exists.** Team A filed `board/reports/team-a-ergon-frontend-design-brief.md`
(19,930 B, sha256 `6a0ac87543a90a54…`, 215 lines) turning A4–A6 into a build-from brief. It is the starting document
for the front-end builders; surface builds stay behind the package/rehearsal gates.

A8. **Authorisation and staging patterns ADOPTED as design (relayed 2026-11-01).** Three patterns: (1) a
gate → eval → mandate authorisation kernel for agent-initiated work; (2) observe-by-default initiative (the agent
records what it *would* accept and does nothing until rules promote it; these records are reviewable); (3) versioned
prompt-config staging with a true separate promotion/approver step (the approver is never the author). **Design adopted,
code not** — no prototype code imported, vendored or run (it is a single-commit alpha coupled to alpha DSH internals);
each pattern names the gap it fills or it does not land. Landed as a design record:
`docs/acceptance/ERGON-PACKAGE-GOVERNANCE-PATTERNS.md` on `main` (`6a455f9`, PR #11).

A9. **Visual system locked: Midnight Slate.** Dark-only, one indigo accent, no gradients/glass/shine/reflections/shadows;
every colour and size comes from `design/tokens.json` through the theme generator (no hand-typed hex); one accent
forward action per view, with Back/Close/Cancel secondary (shipped principle `848a41b`); the unslop gate must pass on
every built surface. Recent merges on `main`: unslop gate pass `3c4411a`, Midnight Slate retheme `29cb921`,
text-button accent strictness `5c2eaed`, primary-action discipline `848a41b`.

A10. **Engine and provider state changed — do not route as the old plan said.** ChatGPT is OUT (Cayleb's subscription
lapsed and he will not renew); Outis, Dots and Codex are RETIRED — never route work through them, never wait on them.
The plugin's Codex-engine provider is CANCELLED (its credential was the ChatGPT login); preserve Codex WIP in place, do
not delete it. The volume engine is DeepSeek V4.1 Flash; the Claude lanes return at the 01:00 EST reset (which is why
this message waited); Nox has Anthropic API access meanwhile for judgement/checker work.

A11. **Fleet/install state that your dependency touches.** Installed package is `@cayleb/dsh-fleet` **0.2.1**; the
integrated 0.2.2 tree `cf93107` / tree `e15c17a3` exists but is **not installed** (owner install Wait, release-identity
row 18). Nox ruling `20261101T181500Z` §2 sets V3 = **FIX, not amend**; Team B's wrapper-change review package is
ready for the owner's ONE combined ask, and no install and no V3 run happens before that ask is answered. Stage 2 is
not closed. The Ergon dress rehearsal runs on that install — not before.

A12. **Hard holds unchanged** (these are never anyone's to cross): no spend beyond a stated envelope (unstated = zero);
no credential values asked for, printed, copied, logged or committed; no public posts/sends/publication; no new
accounts; no live orders/wallet ops; single-writer Lane 1 for the installed fleet service and Cayleb's personal DSH
profile; maker never checker. Claude-gated items (this resend, Money Desk resumption) wait on the provider reset, not
on our scheduling.

A13. **Repo state you will see:** `ergon` `origin/main` = `6a455f9` (after PR #10 `6bcd4eb` Job 03 prep docs and
PR #11 run-evidence + R0 checklist + F1 correction + governance record); earlier merges as in A1/A9. The demo-company
fixture set, acceptance harness (`docs/acceptance/order-e|order-f`) and the rehearsal step labels are all on `main`.

A14. `<NOX: the one or two decisions specific to Duet's own lane that this seat cannot reconstruct — e.g. the exact
"decisions list" wording/ordering you hold, and whether Duet is expected to resume any particular Ergon work item>`

### Part B — Pack 20 findings text (the items your work produced)

B0. **Provenance limit stated plainly:** the Pack 20 narrative report (`P20-F1`–`P20-F22`) is **not on this seat**; the
list below is reconstructed from the committed evidence on `main` (`docs/acceptance/demo-company/`), where every F-number
it knows about is cited. `<NOX: replace or confirm against your held report; where an F-number is missing here it is
because the committed docs do not reference it, not because it did not exist.>` The Pack 20 return headline, quoted
from the committed README: **no genuine recursive self-improvement instance exists on current code**; the gate mechanics
around it are real, but they were driven by fixture code standing in for missing steps — that run is labelled a
gate-mechanics rehearsal and is not counted as the instance.

B1. **The Pack 20 deliverables that were accepted** (merged `1c71898`, checker PASS + merged-tree proof): P20-1 the
demo company (Lanternfield Moving & Storage, RHL's shape); P20-2 the cast of six with distinct written voices; P20-3
scenarios (a)–(f) and seven multi-hop interaction chains scored at the end; P20-4 the seeded past with re-runnable
setup/teardown; P20-5 finding probes and captured output; P20-6 the simulated week in cast voices with stack
artefacts; P20-7 zero-to-competent battery + scorer; P20-8 the self-improvement verdict; P20-9 PACKAGE-PENDING per act;
plus the run config/report template (Addendum 2) and the honestly-labelled stand-in mind-run evidence.

B2. **Findings referenced by the committed docs** (each row is what the evidence says; F-numbers as the docs use them):

| # | Finding | Evidence in the committed tree |
|---|---|---|
| P20-F1 | The Get Help starter configuration refuses the fictional company's name (`Lanternfield`); `RHL` is accepted by `support/app/core.mjs:20` | `COMPANY.md`; probe PR-01 (`evidence/probes-output.txt`) |
| P20-F2 | No host-resident personalised-agent runtime on main; acts that need it are PACKAGE-PENDING, not failures | `SELF-IMPROVEMENT.md`, `WEEK.md`, `fixtures/dry-run-week.mjs` |
| P20-F3 | The agent path's ticket-worker credential cannot act as expected — replies are refused `403 worker_route_denied` (worker route denied) | probe PR-04 |
| P20-F4 | Agent-side ticket resolution is blocked by `409 remote_binding_required`; only escalation succeeds | probe PR-03 |
| P20-F5 | A phone cannot use the fleet join path; it must be hand-registered in the private device registry — the phone/voice surface is a package piece | `CAST.md`, probe PR-05 |
| P20-F6 | One shared operator token is all main offers; every operator-cast member shares that one credential | `COMPANY.md`, probe PR-06 |
| P20-F7 | The manager queue list is capped (returns 100) and hides open tickets (store held 107 tickets, 28 open, 2–9 hidden across the week) — the oldest are invisible to the manager | `SEED.md`, probe PR-07, `WEEK.md` §6 |
| P20-F8 | A ticket raised from the shared North Yard kiosk has no per-person attribution | probe PR-08 |
| P20-F9 | The warehouse board is not on main — scripted state only, PACKAGE-PENDING | `SEED.md`, `WEEK.md` |
| P20-F10 | Onboarding records friction only while `observing`/`interviewing`; after activation `recordObservation` is refused `invalid_transition` | `SELF-IMPROVEMENT.md` (quotes `suite/onboarding/src/protocol.js:701-703`), probe PR-10 |
| P20-F11 | No product/runtime code calls the build loop (`evaluateCandidates`, `buildProposal`, `signOffProposal`, `submitProposal`); only the labelled fixture rehearsal does | `SELF-IMPROVEMENT.md`, probe PR-11 |
| P20-F12 | The employee-profile writer gives every agent the same gateway/voice port pair instead of its own pair | `CAST.md`, `fixtures/setup.mjs:99`, probe PR-12 |
| P20-F13 | The budget proxy accepts only an `https://*.ts.net:8443` origin and refuses any other non-loopback URL; the stand-in reaches the proxy only through a loopback fake upstream redirect | `COMPANY.md`, `WEEK.md` §6, probe PR-13 |
| P20-F14 | Hindsight company memory is unprovisioned on main — no company memory in any run | `COMPANY.md`, `COMPETENCY.md`, probe PR-14 |
| P20-F15 | The review gate and inventory read the wall clock; the state dump has to normalise the stamps and report the count | `SEED.md` (wall-clock dependency), probe PR-15 |
| P20-F16 | Duplicate and burst tickets from the same people are accepted as separate tickets / not de-duplicated by the product | probe PR-16 |
| P20-F17 | A resolution does not reach the raiser's device status read | probe PR-17 |
| P20-F18–F20 | `<NOX: not referenced by the committed docs; supply from the Pack 20 report if these exist>` | — |
| P20-F21 | The two product cost figures disagree by an order of magnitude; the fixture uses the per-person figure as the cap and records the disagreement | `COMPANY.md` |
| P20-F22 | The daily fleet audit is a package piece that does not exist; its acts are PACKAGE-PENDING (with P20-F23 the person-to-person message surface the chains use) | `COMPANY.md`, `WEEK.md` (README says F1–F22; `WEEK.md` cites F23 — the numbering discrepancy is recorded, not smoothed) |

B3. **What the findings mean for the work** (no product defect is claimed beyond what the evidence shows):
- The **PACKAGE-PENDING** class (F2, F5 phone/voice, F9 board, F22 audit, F23 messaging) is missing package surface, not
  a broken existing one — those acts are reported `pending`, never `failed`, and they are exactly what the DSH-era
  package build supplies.
- The **contract/credential shape gaps** (F3, F4, F6, F8, F12) are small product-boundary defects worth folding into the
  package/contract work rather than patching fixture-side.
- The **observability/queue gaps** (F7, F15, F16, F17) are the ones that bite a real manager on day one (hidden backlog,
  clock-dependent state, duplicates, resolution not reaching the raiser).
- The **self-improvement chain** (F2, F10, F11 + P20-8 verdict) is the honest top-line finding: three named missing
  steps (post-activation friction record, runtime caller, agent-authored staging) — no fixture theatre substitutes for
  them.
- F1 and F21 are fixture-realism records: the starter-name refusal is kept true to the company; the cost-figure
  disagreement is recorded with the fixture's chosen cap.

### Part C — Send gate (for Nox)

1. Nox reviews: fix the addressee/channel and A14 + F18–F20 slots against his record.
2. After the Anthropic 01:00 EST reset: send through the ruled Claude route only; **no email/post/public channel**, no
   secrets (none are present in this draft), no money figures outside the board-recorded caps.
3. If Nox prefers the message shorter, the natural cut is Part B's table to the B1 headline + the five grouped
   implications in B3; A1–A13 are the catch-up Duet needs.

## §Draft housekeeping

- File: `board/reports/team-a-duet-resend-draft.md` (this file). Author: Team A (`ergon-lead`).
- Nothing sent; no external action taken; spend $0.
- Successor: Nox's review → send after reset (owner of the send: Nox / the ruled Claude route) → record the send in
  Nox's board. If Nox rules the Pack 20 report must be quoted verbatim, the report needs relaying to this seat first
  (it is not here); that relay is the only input this draft cannot produce itself.
