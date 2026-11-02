# Team A — Lumen v2 grading intake template (owner's wake-up grade)

**STATUS: INTAKE TEMPLATE — EMPTY BY DESIGN. NOT A GRADE, NOT A REWORK ORDER, NO OWNER-FACING ACTION TAKEN.** This is the
form we fill the moment the owner gives his verdict on the frozen Lumen v2 direction set, so the turnaround is immediate
and **graded deltas only** (nothing wider). Prepared 2026-11-02T06:40Z by `ergon-lead` (Team A). **No presentation, no
owner contact, no image reads, $0.**

**Set under grade:** `board/probes/team-a-design-v2/` — frozen **17 content files**; freeze manifest `FREEZE.sha256`,
**freeze digest `2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`**. Re-verified at
2026-11-02T14:58Z: `sha256sum -c FREEZE.sha256` → **17/17 OK**; `node build-tokens.mjs --check` → **CURRENT**;
`node check-comps.mjs` → **PASS** (labels, self-containment, tokens, contrast floors, floors, motion tells, freeze match).

**Presentation order the fresh checker recommends (use it for the intake):** `index.html` → `motion-storyboard.html`
(Replay sequence, then ×0.25) → `comps/07-states.html` (finish + empty + material) → `comps/02-mobile-triage-approvals.html`
(Review, then Approve once) → `comps/01-mobile-chat.html` (send, character tap) → `comps/05-desktop-workspace.html`
(tab swap) → `comps/06-character-sheet.html` → `comps/03-mobile-tasks.html` → `comps/04-mobile-make.html`.

## Intake table (fill verbatim — one row already opened per screen)

| # | Screen / comp | What to look at | His verdict (accepted / rejected / mixed) | **His exact words (verbatim — copy, do not paraphrase)** | Deltas (change X → Y, one per line) | Object the delta maps to (token / component / rule) | Gate impact | New freeze needed? |
|---|---|---|---|---|---|---|---|---|
| 1 | `index.html` (grader index) | Direction reads; checklist; Floors preview; Replay motion | | | | | | |
| 2 | `motion-storyboard.html` | Replay sequence; ×0.25; one curve; reduced-motion/HC floors | | | | | | |
| 3 | `comps/07-states.html` | The finish exposed: all control states, field boundary, three translucent planes, light pool, empty state | | | | | | |
| 4 | `comps/02-mobile-triage-approvals.html` | Needs-you vs can-wait; sheet at 350 ms; scoped "Remember for … for …"; Tier B exclusion | | | | | | |
| 5 | `comps/01-mobile-chat.html` | Translucent layers; neural character + tap signal; send; approval resolve; tab swap | | | | | | |
| 6 | `comps/05-desktop-workspace.html` | Three planes; pinned character; live view; sidebar swap | | | | | | |
| 7 | `comps/06-character-sheet.html` | Node-and-connection mark; density/colour/name; states + one-shot signal | | | | | | |
| 8 | `comps/03-mobile-tasks.html` | Layered task cards; word+colour states; one accent forward action | | | | | | |
| 9 | `comps/04-mobile-make.html` | Artefact stack (no carousel); open + request-a-change; depth hierarchy | | | | | | |
| — | Whole direction | Identity: translucent/light-through-surfaces; technical-editorial taste; "finished" bar | | | | | | |

**Verdict vocabulary** (record which one he uses, do not translate it): *accepted* · *rejected* · *"use a ton of work"* ·
*"looks unfinished"* · *"this is it"* · any other words he actually says. **The exact wording wins over our vocabulary.**


## GRADE ROUND 1 RECORDED (direction + content) — 2026-11-02 04:55 ET

**Owner verdict (verbatim):** "Overall pretty impressed by the content. I think it can use another additional polish round.
With everything I've said in mind. But other than that it's really not bad at all."

- **Class:** direction-level PASS + **one polish round ordered** (not per-screen deltas).
- **Direction:** LOCKED — translucent Lumen vein, Prime Intellect restraint, neural-net agent characters (no further
  design-direction gate).
- **Order:** finish pass only (no redesign): components look finished (no unfinished edges), panels keep real depth, motion
  present and purposeful, finish never reads as AI slop — per his accumulated direction from both grades.
- **Grid:** the table above is kept for the **polish-round** deltas (if his final grade adds any); the polish round applied
  the finish pass without inventing deltas beyond his instruction.
- **Presentation:** re-present as **LIVE animated pages in his browser** (rendered visuals rule; static stills were the named
  failure). Presentation is the Lead's; this lane does not contact the owner.

## Delta classification (do this before touching anything)

| Class | What it means | Where it lands |
|---|---|---|
| **A — Token** | A change to a value in `tokens.json` (colour, surface alpha, blur, border weight, depth, motion, radius, type, target) | token file + regenerate `tokens.css`; then re-freeze + self-check |
| **B — Component** | A change to a component in `comp-kit.css` / a comp markup (density, state, spacing, structure, wording on a surface) | kit/comp; then re-freeze + self-check |
| **C — Copy / structure** | Wording, ordering, information on the page (not an aesthetic value) | the comp; re-freeze if frozen files change |
| **D — Direction objection** | "Not this" on identity/motion/character rather than a specific delta | **Nothing is reworked**: it goes back through the Lead → Nox with his words; a new direction pass is commissioned, not improvised |
| **E — Pack consequence** | Anything about Midnight Slate being replaced/kept, versioning, distribution | The pack-change gate (`team-a-pack-change-gate-checklist.md`), not the comps |

## Mapping key (fill the "object" column with one of these)

- **Tokens:** `--base`, `--ambient-accent/cool`, `--surface-1/2/3`, `--light-1`, `--line/line-strong/line-bright`,
  `--field-line`, `--blur-panel/card/sheet/deep`, `--depth-lift/lift-high/rim/well`, `--radius-*`, `--space-1..7`,
  `--body/heading/caption/emp-*`, `--target/target-employee`, `--motion-tap/swap/enter/sheet`, `--ease`,
  `--focus-ring/gap`, `--accent/on-accent`, `--ok/warn/danger` (+ washes/edges).
- **Components:** phone shell, desktop frame, sidebar, card/raise, message, composer (field boundary), tab bar, sheet +
  scrim, pills, buttons (primary/secondary/ghost/icon/employee/busy), fields, checkbox, empty state, light pool,
  neural character (`--nn-hue`, density, states), tooltip (not built), notification stack (not built).
- **Rules:** C-REM (scoped remember), C-W1 (ADD-only), C-TOK, C-MOTION (120/200/250/350, one curve, no loops),
  C-TARGET (44/56, focus 2px+2px), C-A11Y, C-PROV.

## Rules for this intake (binding)

1. **Graded deltas only.** We do not "improve" anything he did not name; we do not treat a general displeasure as
   permission for a redesign. No delta row = no change.
2. **No comp changes without his grade.** The set stays frozen until his words are captured on this form.
3. **Anything wider goes back through Nox/Lead** — direction objections (Class D), pack/theming consequences (Class E), or
   any request that changes scope (new screens, new product behaviour) are not absorbed locally.
4. **Verbatim first, mapping second.** Copy his words before interpreting them; if a delta is ambiguous, record both
   readings and ask through the Lead — never guess silently.
5. **Every accepted delta is traceable:** delta → object (token/component/rule) → change → re-freeze → self-check → (if the
   pack gate is touched) fresh checker. Maker never checker.
6. **Do not reopen checker notes by ourselves** (e.g. C5/C6-style aesthetic suggestions) without his grade or the Lead's
   instruction.
7. **No owner-facing action from this lane:** the Lead presents, the Lead relays his words; this form is filled by the
   lane and landed on the board.

## After the grade — exact sequence (to turn around immediately)

1. Capture his words into the table above; classify each delta (A–E).
2. For A/B/C deltas: apply only the named change; regenerate tokens if token-touching; re-run
   `node build-tokens.mjs` (then `--check`) and `node check-comps.mjs`; record before/after values.
3. Re-freeze: regenerate `FREEZE.sha256`, record the new digest in the report (`board/reports/team-a-design-v2.md`) and
   this form.
4. Hand the new freeze + the delta list to the Lead for the fresh checker re-run (same battery).
5. For D/E deltas: file the words + the question for Nox/Lead; do **not** act.
