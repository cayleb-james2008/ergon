# Team A — component-layer admission proposal SKELETON (S1–S8) — BLOCKED UNTIL THE OWNER GRADES v2

**STATUS: SKELETON ONLY — NOT THE PROPOSAL, NOT AN ADMISSION, NOTHING VENDORED.** This is the fillable structure for the
component-layer admission proposal that becomes live **only after the owner grades the Lumen v2 direction**. The parts-bin
curation gate is **FROZEN** (not discarded) until that happens; this file exists so the proposal can be filled in one pass
from graded deltas. Prepared 2026-11-02T06:41Z by `ergon-lead` (Team A). No presentation, no owner contact, no image
reads, $0.

**Frozen set this sits behind:** `board/probes/team-a-design-v2/` — 17 content files; freeze digest
**`2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`**; re-verified 2026-11-02T14:58Z
(`sha256sum -c` 17/17 OK · `build-tokens --check` CURRENT · `check-comps` PASS).

**Entry conditions (all must be true before a row can be filled):**
1. The owner's v2 grade is recorded in `team-a-v2-grading-intake.md` (direction accepted or accepted-with-deltas).
2. The graded deltas that touch components are classified and applied (v2 re-frozen; checker re-run ACCEPT).
3. The pack-change gate (`team-a-pack-change-gate-checklist.md`) is at least at the "owner direction accepted" step.
4. The row's licence line says clean for the intended use, and the **curation gate** (G1 proposal → G2 all-states → G3
   screenshot compare → G4 fresh checker) is run for that row — maker never checker.

**Standing constraints:** Animate UI **in-app only** (Lead ruling R1) — never the substance of a distributed components
bundle; Uiverse **per-element provenance record** required at admission (R2); Forge UI/Uilora are **patterns only** (R3/R4,
not in S1–S8); W1 is **ADD-only** (C-W1) — components live in Ergon's own `main[key]` view / sidebar panels, stock chat is
not replaced; the unslop gate and all accessibility floors bind every admission.

## How to fill a row

For each S-row: link the **owner delta(s)** that authorise it (or mark *direction-level, no delta* if he accepted the
direction wholesale and the component follows directly), then walk G1→G4 and record evidence per step. A row may end
**ADMIT**, **HOLD** (named blocker), or **DROP** (licence/fit mismatch) — no "looks close" admissions.

## Admission record skeleton (duplicate per row)

### S1 — Bottom-navigation minimise-to-icons · Ergon pack
- **Owner delta link:** `<delta id(s) from intake form, or "direction-level">`
- **Licence line:** L2 Animate UI — MIT + Commons Clause, **in-app only** (R1); copyright + licence notice retained;
  evidence URL: `https://raw.githubusercontent.com/imskyleen/animate-ui/main/LICENSE.md`; fetch date: `<yyyy-mm-dd>`
- **TAKE (interaction pattern):** tab bar that gives content room back; minimise-to-icons is a state, not decoration.
- **LEAVE (fence):** state-disc glow, uppercase tracked labels, looping attention cues, `transition-all`, icon shift
  without a word.
- **All-states checklist (G2):** expanded · collapsed/minimised · selected · unselected · hover · focus-visible
  (2 px accent ring + 2 px gap) · pressed · disabled tab · loading · error · empty · reduced-motion · High Contrast ·
  keyboard order + Enter/Space · SR tab name + "selected".
- **Screenshot-compare criteria (G3):** fills only `panel/raise/base/hover`; selection via accent ring or `raise`; text
  `ink`/`muted`; no hex; radius 6 control / 8 panel; targets 44 (56 employee); type IT 14/22/12.5, employee 16/24/14;
  swap 200 ms, one ease-out. Differences named (token, spacing, type, motion), never "looks close".
- **Provenance block:** library@version `<...>` · element id/author/URL `<...>` · fetch date `<...>` · restyle plan `<...>` ·
  leave-list `<...>` · licence notice retained `<Y/N>`
- **Gate state:** G1 proposal `<date/state>` · G2 all-states `<owner/evidence>` · G3 compare `<seat/evidence>` ·
  G4 fresh checker `<checker/verdict/date>`
- **Admission decision:** `<ADMIT / HOLD / DROP>` — reason: `<...>` — all-states evidence path: `<...>`

### S2 — Accordion / collapsible rows · Ergon pack
- **Owner delta link:** `<...>`
- **Licence line:** L2 Animate UI — MIT + Commons, in-app only (R1); evidence `…/animate-ui/main/LICENSE.md`; fetch `<...>`
- **TAKE:** task-detail / approvals rows expand in place; honest state without a modal.
- **LEAVE:** glow open/close chrome, bounce, `transition-all`, rotating chevron as spectacle.
- **All-states (G2):** collapsed · expanded · hover · focus-visible (2 px ring) · pressed · disabled row · loading row ·
  error row · empty group · reduced-motion (instant expand) · HC (dividers system colours) · keyboard Enter/Space +
  arrow nav · `aria-expanded` + region association.
- **Compare (G3):** `line` 1 px dividers only; no shadow; expand ≤200 ms swap; chevron 16 px `muted` → accent on focus
  only; type per platform.
- **Provenance / Gate state / Decision:** `<fill>`

### S3 — Carousel mechanics (embla) · Ergon pack
- **Owner delta link:** `<...>`
- **Licence line:** L2 Animate UI in-app (R1) + embla upstream MIT — record both notices; evidence `<...>`
- **TAKE:** Make-artefact gallery without writing a slider; keyboard- and reduced-motion-aware.
- **LEAVE:** autoplay, looping, glowing dots, parallax cards, drag inertia outside the budget.
- **All-states (G2):** first · middle · last (prev/next disabled correctly) · single item · overflow · hover ·
  focus-visible · pressed · loading · error · empty · reduced-motion (no autoplay ever) · HC.
- **Compare (G3):** dots 2 px accent / `subtle`, no glow; arrows 44 DIP; swap 200 ms; images in a `raise` frame, 8 px
  radius; no gradient/reflection.
- **Provenance / Gate state / Decision:** `<fill>`

### S4 — Dialog / sheet composition · Ergon pack (**hosts C-REM**)
- **Owner delta link:** `<...>` (comp 02's affordance is approved only in the scoped Remember form; any wider form → Nox)
- **Licence line:** L2 Animate UI in-app (R1); evidence `<...>`
- **TAKE:** approvals sheet; enter/exit within the 350 ms sheet budget; flat scrim.
- **LEAVE:** glass/blur scrim, sheet bounce, drag-handle glow, decorative depth.
- **All-states (G2):** closed · entering ≤350 ms · open · exiting · hover/focus on actions · pressed · disabled primary
  while working · loading · error (plain words + next step) · empty (no pending approvals) · reduced-motion (instant) ·
  HC · focus trap + Escape · focus returns to trigger · one accent forward action.
- **C-REM check (must pass in this surface):** wording exactly "Remember for \<subject\> for \<action\>" (both named);
  no standing-permission wording; never cross-subject/cross-action; revocable; remembered grants visible in the approvals
  record; Tier B never offers it.
- **Compare (G3):** flat token scrim (no blur); panel fill + `line` hairline; 8 px radius; approve = single accent action,
  deny/close secondary; 44/56 DIP; 350 ms one ease-out.
- **Provenance / Gate state / Decision:** `<fill>`

### S5 — Heading / entry reveal · Ergon pack
- **Owner delta link:** `<...>`
- **Licence line:** L1 Vengeance UI **MIT** (CLEAN); evidence `…/VengeanceUI/main/LICENSE`; notice retained; fetch `<...>`
- **TAKE:** onboarding/empty-state heading arrives once, 250 ms, purposeful.
- **LEAVE:** gradient text, aurora/glow meshes, particles, looping hero demos, 800 weights, uppercase tracked labels,
  greeting headlines/hype copy.
- **All-states (G2):** final/static end-state readable with motion off · entering 250 ms · hover (if interactive) ·
  focus-visible · pressed/disabled (if a control) · reduced-motion (instant) · HC.
- **Compare (G3):** heading semibold, IT 22 / employee 24; `ink`; no gradient text/`background-clip`; one accent action;
  250 ms enter, one curve.
- **Provenance / Gate state / Decision:** `<fill>`

### S6 — Shared-element nav transition · Ergon pack
- **Owner delta link:** `<...>`
- **Licence line:** L1 Vengeance UI MIT; evidence `<...>`
- **TAKE:** sidebar → content continuity at 200 ms.
- **LEAVE:** `transition-all`, 3D/tilt stretch, spectacle hover, motion outliving the budget.
- **All-states (G2):** no-transition path (reduced motion instant) · enter 200 ms · focus moves to destination heading ·
  back restores focus · keyboard-only path · HC (decorative motion off) · SR announces new view.
- **Compare (G3):** transform/opacity only; 200 ms swap; destination surface `base` with `panel` chrome; no shadow;
  **ADD-only** surface (own `main[key]` view; does not assume replacing stock chat).
- **Provenance / Gate state / Decision:** `<fill>`

### S7 — Switch/toggle + field focus · Ergon pack
- **Owner delta link:** `<...>`
- **Licence line:** L3 Uiverse **MIT per element** + **R2 provenance record required**; evidence
  `https://uiverse.io/elements` (site-wide MIT) + element author/URL recorded; fetch `<...>`
- **TAKE:** employee settings controls that read flat and stateful; 44 DIP targets.
- **LEAVE:** glow track, neon/neumorphic skins, 3D press, animated glow.
- **All-states (G2):** off · on · hover · focus-visible · pressed · disabled · loading (save pending) · error (save failed,
  plain words) · reduced-motion (no spring overshoot) · HC (state not colour-only) · keyboard Space/Enter ·
  `role="switch"` + `aria-checked`.
- **Compare (G3):** track `base`/`raise` + `accent` when on; label always present; 44/56 DIP; 120 ms tap; `fieldLine` →
  `accent` focus treatment; no shadow/glow.
- **Provenance / Gate state / Decision:** `<fill>` — note: no admission without the R2 provenance record.

### S8 — Input / field focus treatment · Ergon pack
- **Owner delta link:** `<...>`
- **Licence line:** L3 Uiverse MIT per element + R2 provenance record; evidence `<...>`
- **TAKE:** forms alive via focus and honest error states, not decoration.
- **LEAVE:** animated glow, shimmer placeholder, gradient border, colour-only error signalling.
- **All-states (G2):** empty · filled · hover · focus (`fieldLine` → accent border + 2 px ring/2 px gap) · disabled ·
  read-only · error (`danger` line + plain words) · loading/validating · max length · reduced-motion · HC · tab order.
- **Compare (G3):** `base` fill, `fieldLine` 1 px border → `accent` focus, `ink` text, 6 px radius, 44 DIP; error text
  `danger` **and** words; no glow/shimmer.
- **Provenance / Gate state / Decision:** `<fill>`

## Rows deliberately NOT in this skeleton

- **S9–S12 (fleet pack)** — provisional, gated on the fleet pack's own definition + S9's R1 boundary; separate file when
  that pack is defined.
- Any new component the owner's deltas introduce — a new row is created from the intake form, not improvised.

## Handoff to the fresh checker (per row, at G4)

The checker receives: the row's G1 proposal (this skeleton + the delta link + licence line), G2 all-states evidence set,
G3 named-difference comparison, and the provenance block — and must have built none of it. Only G4 PASS appends the
admission record and updates the pack. **The parts-bin gate stays frozen until the owner's v2 grade exists; this skeleton
authorises nothing.**
