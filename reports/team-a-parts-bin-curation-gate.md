# Team A — parts-bin curation gate work-up (S1–S12)

**STATUS: GATE PREP / PROPOSAL ONLY. NOTHING BUILT, VENDORED, PURCHASED, SCREENSHOT-READ OR MERGED.** This file works the
pack curation gate (`proposal → all-states check → screenshot compare → fresh checker`) for the 12 candidates in
`board/reports/team-a-parts-bin-shortlist.md`, as far as it can go **without the owner's comps grading**. It authors, per
row: the re-confirmed licence line, the take/leave fence against the packs' own rules, the all-states checklist, the
screenshot-comparison criteria, and the waiting list. Prepared by Team A (`ergon-lead`, Lane 5), 2026-11-02 ~00:46Z.
Spend **$0**; no purchases; no code moved; **no repo writes beyond this team report**; no image reads/screenshots in DSH
(host crash class — see §2).

## 0. Rulings of record — Nox `20261102T003600Z` (recorded, applied here where relevant)

| # | Ruling (quote / substance) | Application to this work-up |
|---|---|---|
| **R1-TA** | **Tier A "Remember" — ALLOWED, NARROWLY SCOPED.** "remembered for ONE named subject AND ONE named action type; the wording on the surface must state both (e.g. 'Remember for \<subject\> for \<action\>') — no 'always allow' wording anywhere; never cross-subject, never cross-action; revocable, and every remembered grant is visible in the approvals record. Tier B never offers remember/always-allow." Comp 02's affordance approved **only** in this scoped form; any wider form returns to Nox; **comps are not reworked** — owner grading deltas still govern any comp change. | Cross-cutting requirement **C-REM** (§4): any candidate that surfaces an approval/remember affordance (notably **S4** approvals sheet, and any approvals-record view) must carry the scoped wording and the visibility/revocability rules. The Remember affordance is **not** a new shortlist row; it is a constraint on the built approvals surface. |
| **R2-W1** | **W1 = (a) ADD-only** (arbitrated): Ergon's own `main[key]` view + sidebar panel(s) + graph resource contributions; stock chat **not** replaced in W1; the documented reshape is a **later W2 delta**, admissible only with the C4 contract page. Matches the first real order's shape (sidebar section + VM panels are ADD-only). | Cross-cutting requirement **C-W1** (§4): S1–S8 must fit an ADD-only surface (own view / sidebar panels); a pattern that only makes sense by replacing the stock conversation view is **out of W1 scope** and cannot be admitted now. |
| **R3-J03** | **Job 03:** G1's **Lane-1 leg is DISCHARGED by the proven 0.3.0 install only**; **G1 whole is NOT satisfied** (Ergon-on-DSH package artefact, fresh host, recorded `dsh` version still owed); the **RUN stays held/unscheduled**. **C1–C6 confirmed**, with **C2 re-pinned to installed 0.3.0** (0.3.1 re-pin only after install+checker, as a recorded bump); C4 page + example row is **Team B's** obligation. | Recorded; **no RUN action** and no gate confusion. This work-up does not touch Job 03 gates; the parts bin is downstream of grading, not of the RUN. |
| **R4-CAND** | **Candidate branch: NO MERGE**; Stage 2 PASS stands; stage left to tmpfs. | Recorded; nothing for this lane. |

Also for the record: the fold-in nit successor is **closed** — nit PR **#15 merged as `598da57`** (2026-11-02T00:08:11Z),
Ergon `main` at **`598da57`**; the simulation-bank proof runs remain **GATED and unwaived**.

## 1. Licence lines re-confirmed this work-up (2026-11-02 ~00:45Z)

| # | Library | Line of record | Re-check tonight | Admission consequence |
|---|---|---|---|---|
| L1 | **Vengeance UI** | **MIT** — "Permission is hereby granted … to deal in the Software without restriction, including … use, copy, modify, merge, publish, distribute, sublicense, and/or sell" (`raw.githubusercontent.com/Ashutoshx7/VengeanceUI/main/LICENSE`) | **HTTP 200**, text re-read: unchanged | **CLEAN** (app repo / pack, MIT notice retained) |
| L2 | **Animate UI** | **MIT + Commons Clause** — "distribute … as part of an application, website, or product"; "do not sell or redistribute the components themselves in their original form—whether alone or in a bundle" (`raw.githubusercontent.com/imskyleen/animate-ui/main/LICENSE.md`) | **HTTP 200**, text re-read: unchanged | **In-app only** (R1); **never** the substance of a distributed components pack/bundle; notice retained |
| L3 | **Uiverse** | **MIT per element** — site-wide "All content (UI elements) on this site are published under the MIT License"; ToS §29 licenses user content "Specifically, the MIT license." (`uiverse.io/elements`; ToS viewer `app.termly.io/…policyUUID=bdc0e7cf…`) | **Not re-readable from this seat tonight:** `https://uiverse.io/elements` → **HTTP 403** (bot block); ToS viewer returned its title only (client-rendered). The annex's primary-source fetch (2026-11-01 19:52–20:10Z) stands | **CLEAN** with attribution **and** a **per-element provenance record at admission** (R2); the licence line itself is unchanged, the open item is provenance, not licence |
| L4 | **Forge UI** (`forgeui.in`, not Roblox ForgeGUI) | **Bespoke usage-only** — "This license grants usage rights, not redistribution rights"; "You may NOT: Repackage, redistribute, or resell this code … as a UI library, component collection, template, starter, or similar product"; Pro terms bar resale/sublicence (`raw.githubusercontent.com/AmanShakya0018/forgeui/main/LICENSE`, `forgeui.in/termsofservice`) | **HTTP 200**, text re-read: unchanged | **Patterns only** (R4); no code copied; Pro unmade (purchase fence) |
| L5 | **Uilora** | **Commercial/premium, not open source** — non-transferable licence to incorporate into applications; "may not redistribute Uilora components as a standalone UI library or component pack … without our written consent"; free tier limited to personal/open-source (`uilora.com/terms`, `uilora.com/pricing`) | **HTTP 200 but client-rendered**: titles only ("Terms of Service \| Uilora", "Pricing — Uilora \| Lifetime Access to 490+ UI Components"); no body text to re-read. The annex's recorded clause stands | **Not usable — shape/pattern study only** (R3); no purchase, no redistribution |

**Net:** no licence line changed; no admission decision tonight depends on the two non-re-readable pages (L3's open item is
provenance, and L5 is pattern-only). **The only open licence-side item is the Uiverse per-element provenance record (R2)**,
which is created at admission time (author + element URL + fetch date in the pack provenance block).

## 2. Gate mechanics as this lane can execute them (and where the wall is)

| Gate step | What it requires | Can this work-up do it? | Produced here |
|---|---|---|---|
| **G1 Proposal** | Row with library, version/commit, element id + author + URL, licence line, fetch date, pack, restyle plan, leave-list | **Yes** | Shortlist (done) + this file's per-row fences (§3) |
| **G2 All-states check** | Component rendered once per state (default/hover/focus/pressed/disabled/loading/error/empty + reduced-motion + High Contrast); missing state = fail | **Checklist yes; execution no** — a component must exist first (post-grading for S1–S8; post-fleet-definition for S9–S12) | Per-row all-states checklists (§3) |
| **G3 Screenshot compare** | Rendered comparison against the pack's reference on tokens; differences **named** (tokens, spacing, type, motion) | **Criteria yes; execution no** — and **not in this DSH seat**: image reads/screenshots are the crash class, so the comparison must be performed by a seat permitted to view images (or via text/metadata evidence produced host-side), never by this session | Per-row comparison criteria (§3) |
| **G4 Fresh checker** | A checker who built nothing records PASS/FAIL; only PASS admits | **No** — Lead spawns the checker; maker never checker | The evidence contract to hand it (§6) |

**Evidence layout (proposed, not created — nothing is written this batch):**
`board/probes/team-a-parts-bin-gate/<row>/` with `G2-states/<state>/` (render + named state), `G3-compare/` (reference,
render, named-diff.md — produced by an image-permitted seat), `G4-verdict.md` (checker), and `admission-record.md` (only
after PASS). The rows are `S1`…`S12`; a row's folder is created only when its gate actually starts.

## 3. Per-candidate work-up (all rows: G1 done here; G2/G3 authored; none admitted)

Each block: re-confirmed licence · TAKE · LEAVE · all-states checklist (G2) · comparison criteria (G3 against
`design/tokens.json` + `DESIGN.md` + `design/motion-spec.md`) · what it waits on.

### S1 — Bottom-navigation minimise-to-icons (Animate UI) · **Ergon** · L2 MIT+Commons, **in-app only** (R1)
- **TAKE:** Chat/Tasks/Make tab bar that gives content room back; minimise-to-icons is a *state* of the bar, not decoration.
- **LEAVE:** state-disc glow, uppercase tracked labels, looping attention cues, `transition-all`, any label-only icon shift without a word on the surface.
- **All-states (G2):** expanded · collapsed/minimised · tab selected · tab unselected · hover · focus-visible (2 px `accent` ring + 2 px gap, never animated) · pressed · disabled tab (feature unavailable) · loading (agent started, tab pending) · error (tab content failed — named, plain words) · empty (no items in a section) · reduced-motion (minimise instant or ≤200 ms swap) · High Contrast (system brushes, selection not colour-only) · keyboard order + Enter/Space · screen-reader: tab name + "selected".
- **Compare (G3):** fills only `panel`/`raise`/`base`/`hover`; selection via `accent` ring or `raise` fill; text `ink`/`muted`; no hex; radius 6 controls / 8 panels; targets 44 DIP (56 employee); type IT 14/22/12.5, employee 16/24/14; swap **200 ms**, one ease-out.
- **Waits:** owner comps grading → then ADD-only W1 surface (Ergon's own `main[key]` view + sidebar panels; stock chat untouched).

### S2 — Accordion / collapsible rows (Animate UI) · **Ergon** · L2 in-app only (R1)
- **TAKE:** task-detail and approvals rows that expand in place; honest state without a modal.
- **LEAVE:** glow open/close chrome, bounce, `transition-all`, chevrons that rotate as spectacle.
- **All-states (G2):** collapsed · expanded · hover · focus-visible (2 px ring) · pressed · disabled row · loading row (async detail) · error row (detail failed) · empty group · reduced-motion (instant expand) · HC (dividers via system colours) · keyboard Enter/Space + arrow nav · `aria-expanded` + region association.
- **Compare (G3):** `line` 1 px dividers only; no shadow; expand motion ≤200 ms swap; text sizes per platform; chevron glyph 16 px, `muted` at rest / `accent` on focus only.
- **Waits:** owner comps grading → ADD-only W1 surface.

### S3 — Carousel mechanics (Animate UI / embla) · **Ergon** · L2 in-app only (R1); embla upstream MIT
- **TAKE:** Make-artefact gallery without writing a slider; keyboard- and reduced-motion-aware mechanics.
- **LEAVE:** autoplay, looping, glowing dots, parallax cards, drag inertia outside the motion budget.
- **All-states (G2):** first item · middle · last (prev/next disabled correctly) · single item · many items/overflow · hover · focus-visible on arrows + items · pressed · loading first item · error (artefact unavailable) · empty (no artefacts made) · reduced-motion (no autoplay ever; instant swap) · HC.
- **Compare (G3):** no dots glow (`accent` at 2 px, `subtle` otherwise); arrows 44 DIP; swap 200 ms; images inside a `raise` frame, 8 px radius; no gradients/reflection.
- **Waits:** owner comps grading → ADD-only W1 surface.

### S4 — Dialog / sheet composition (Animate UI) · **Ergon** · L2 in-app only (R1) · **hosts C-REM (Remember)**
- **TAKE:** approvals sheet; enter/exit fits the 350 ms sheet budget; plain flat scrim.
- **LEAVE:** glass/blur scrim, sheet bounce, drag-handle glow, decorative depth.
- **All-states (G2):** closed · entering (≤350 ms) · open · exiting · hover/focus on actions · pressed · disabled primary while working · loading (approval in flight) · error (approval failed — plain words + next step) · empty (no pending approvals) · reduced-motion (instant open/close; scrim no fade) · HC (system brushes, visible outline) · focus trap + Escape · focus returns to trigger · one accent forward action.
- **C-REM (must be checked in this surface):** if this sheet offers Remember, wording is exactly **"Remember for \<subject\> for \<action\>"** (both named, on the surface); **no "always allow" anywhere**; never cross-subject, never cross-action; revocable; remembered grants visible in the approvals record; Tier B never offers it. Wider form → Nox, not built.
- **Compare (G3):** scrim from a flat token wash (no blur); panel `panel` fill, `line` hairline, 8 px radius; approve = the single `accent` action, deny/close secondary (`raise`, `lineStrong`); 44/56 DIP; sheet 350 ms one ease-out.
- **Waits:** owner comps grading (comp 02 affordance in the scoped form only) → ADD-only W1 surface.

### S5 — Heading / entry reveal (Vengeance UI) · **Ergon** · L1 MIT (notice retained)
- **TAKE:** onboarding/empty-state heading that arrives once, 250 ms, purposeful.
- **LEAVE:** gradient text, aurora/glow meshes, particles, looping hero demos, 800-weight headings, uppercase tracked labels, greeting headlines/hype copy.
- **All-states (G2):** final/static end-state (must be fully readable with motion off) · entering (250 ms) · hover (if interactive — no spectacle) · focus-visible · pressed (if a control) · disabled (if a control) · reduced-motion (no transform/opacity animation; instant) · HC (system text colours, contrast preserved).
- **Compare (G3):** heading semibold, IT 22 px / employee 24 px; `ink`; no `font-weight: 800`; no gradient/`background-clip:text`; one accent forward action in the view; 250 ms enter, one ease-out.
- **Waits:** owner comps grading → ADD-only W1 surface.

### S6 — Shared-element nav transition (Vengeance UI) · **Ergon** · L1 MIT
- **TAKE:** sidebar → chat "one click into the agent" continuity at 200 ms.
- **LEAVE:** `transition-all`, 3D/tilt stretch, spectacle hovers, motion that outlives the budget.
- **All-states (G2):** no-transition path (reduced motion: instant) · enter (200 ms) · focus moves to destination heading · back/return restores focus to the source item · keyboard-only path works · HC (decorative motion off) · screen-reader announces the new view.
- **Compare (G3):** only transform/opacity; 200 ms swap; no `transition-all`; the destination surface is `base` with `panel` chrome; no shadow.
- **Waits:** owner comps grading → must fit **ADD-only** W1 (`main[key]` view); does **not** assume replacing stock chat.

### S7 — Switch/toggle + field focus (Uiverse) · **Ergon** · L3 MIT per element, **R2 provenance required**
- **TAKE:** employee settings controls that read flat and stateful; 44 DIP targets.
- **LEAVE:** glow track, neon/neumorphic skins, 3D press, animated glow on state change.
- **All-states (G2):** off · on · hover · focus-visible (2 px ring) · pressed · disabled · loading (save pending) · error (save failed — plain words) · reduced-motion (no spring overshoot) · HC (system brushes; on/off conveyed by position + label, never colour alone) · keyboard Space/Enter · `role="switch"` + `aria-checked`.
- **Compare (G3):** track `base`/`raise` + `accent` when on; label always present (no icon-only state); 44/56 DIP; 120 ms tap feedback; no shadow/glow; `fieldLine`→`accent` focus treatment per DESIGN.md Fields.
- **Waits:** owner comps grading → ADD-only W1 surface; **plus R2 provenance record** (author + element URL + fetch date).

### S8 — Input / field focus treatment (Uiverse) · **Ergon** · L3 MIT per element, **R2 provenance required**
- **TAKE:** forms feel alive via focus and honest error states, not decoration.
- **LEAVE:** animated glow, shimmer placeholder, gradient border, colour-only error signalling.
- **All-states (G2):** empty · filled · hover · focus (`fieldLine` → `accent` border + 2 px ring/2 px gap) · disabled · read-only · error (`danger` line + plain words, never colour alone) · loading/validating · max length · reduced-motion (no border animation) · HC (system brushes remain visible) · keyboard tab order.
- **Compare (G3):** `base` fill, `fieldLine` 1 px border → `accent` on focus, `ink` text, 6 px radius, 44 DIP; error text uses `danger` **and** words; no glow/shimmer.
- **Waits:** owner comps grading → ADD-only W1 surface; **plus R2 provenance record**.

### S9 — Copy-button + code-block (Animate UI) · **Fleet (provisional)** · L2 in-app only — **R1 boundary**
- **TAKE:** operator logs/diagnostics: copy action + monospace block, 120 ms feedback.
- **LEAVE:** rainbow syntax themes, shimmer text, glass panel, `transition-all`.
- **All-states (G2):** idle · copied (120 ms feedback, then back) · copy failed (named) · hover · focus-visible · pressed · disabled (nothing to copy) · code block: wrapped/unwrapped, long line, empty · reduced-motion (instant feedback) · HC (system colours, block border visible).
- **Compare (G3):** mono `Cascadia Mono`/`Consolas`; tokens only; feed the fleet pack's own tokens when that pack is defined; no rainbow syntax; 120 ms tap.
- **Waits:** fleet pack **definition** (SWEEP.md is Nox-side; rows provisional) **and** the **R1 boundary decision**: if the fleet pack is distributed as a components bundle, Animate UI code is barred → **clean-room reimplementation or drop**; if the fleet pack stays in-app/internal-pattern-only, the in-app line applies.

### S10 — Tooltip (Uiverse) · **Fleet (provisional)** · L3 MIT per element, R2 provenance
- **TAKE:** status-pill/queue help without clutter; 120 ms.
- **LEAVE:** arrow glow, drop shadow, gradient background, tooltip-as-only-label.
- **All-states (G2):** hidden · hover-shown · focus-visible-shown (keyboard users) · Escape-dismissed · long text/wrap · reduced-motion (appears instantly) · HC (visible border, system text) · screen-reader association (`aria-describedby`).
- **Compare (G3):** `raise` surface + `line` hairline, no shadow; 120 ms; never the only label for an icon.
- **Waits:** fleet pack definition; R2 provenance record.

### S11 — Notification-stack pattern (Forge UI) · **Fleet (provisional)** · L4 **patterns only** (R4)
- **TAKE:** queue/alert stacking that reads like a night desk; enter 250 ms; severity tokens only for real severity.
- **LEAVE:** shimmer, glow cards, gradient borders, pulse loops, "toast theatre".
- **All-states (G2):** empty · one · many/overflow · entering (250 ms) · hover · focus-visible · dismiss (mouse + keyboard) · severity variants ok/warn/danger (words + colour) · paused on hover (no loop) · reduced-motion (instant) · HC.
- **Compare (G3):** `panel`/`raise` surfaces, `line` dividers, matching `okWash`/`warnWash`/`dangerWash` pills with words; entering 250 ms; no looping pulse.
- **Waits:** fleet pack definition; **clean-room only** (no Forge UI code — R4).

### S12 — Usage / pricing-table pattern (Uilora) · **Fleet (provisional)** · L5 **shape study only** (R3)
- **TAKE:** operator console plan/usage display; `line` dividers, no gradient headers.
- **LEAVE:** gradient/pattern backgrounds, magnetic cursor, 3D transforms, any Uilora code.
- **All-states (G2):** default · empty · loading · error · many rows/overflow · row hover · row focus/actions · HC (system brushes, table borders visible).
- **Compare (G3):** table on `panel`, `line` dividers, `ink`/`muted` text, no gradient header; numeric alignment; 14/22/12.5 type.
- **Waits:** fleet pack definition; **no code, no purchase** (R3).

## 4. Cross-cutting requirements every admission must satisfy (gate checklist additions)

| ID | Requirement | Source |
|---|---|---|
| **C-REM** | Approvals surfaces: "Remember for \<subject\> for \<action\>" wording (both named); no "always allow" anywhere; never cross-subject/action; revocable; remembered grants visible in the approvals record; Tier B never offers remember/always-allow. Comp 02's affordance only in this scoped form; wider form → Nox. | Nox `20261102T003600Z` §4 |
| **C-W1** | Components live in Ergon's own `main[key]` view / sidebar panels / graph resource contributions; stock chat is **not** replaced in W1; reshape is a W2 delta gated on the C4 contract page. | Nox `20261102T003600Z` §3 (W1 = a) |
| **C-TOK** | Colours/sizes only from `design/tokens.json`; no hand-typed hex; depth only `base → panel → raise`; **no** gradients, glass, shine, reflection, drop-shadow; one accent forward action per view; pills always carry words (colour never the only signal). | DESIGN.md §Colors/§Components/§Don'ts |
| **C-MOTION** | 120/200/250/350 ms, **one ease-out**; no loops/pulse/ping; `transition-all` banned; reduced-motion path required; **decorative motion off under High Contrast**. | `design/motion-spec.md`, DESIGN.md §High Contrast |
| **C-TARGET** | 44 DIP minimum, 56 DIP employee buttons; focus = 2 px `accent` ring + 2 px gap, never animated; radii 8 panel / 6 control / pill; spacing 8·12·16·24·32; type IT 14/22/12.5, employee 16/24/14. | DESIGN.md §Shape and spacing / §Typography |
| **C-A11Y** | Keyboard order; Enter/Space activation; Escape dismiss; focus trap + restore in sheets; roles/`aria-expanded`/`aria-checked`/`aria-describedby` as per row; focus always visible. | Each row's G2 list |
| **C-PROV** | Provenance block: library@version, element id + author + URL, licence line + evidence URL, fetch date, restyle plan, leave-list; licence notice retained exactly where upstream code remains (L1/L2/L3 as per §1). | Annex + R1/R2/R3/R4 |

## 5. What MUST wait — and on what (the asked-for split)

**A. MUST wait for the owner's comps grading (Ergon rows S1–S8).** The design comps
(`board/probes/team-a-design-comps/`) are filed for **OWNER GRADING** and **must not be reworked**. Until graded verdict /
graded deltas arrive, none of S1–S8 may build or be admitted: grading unlocks the component-layer admission proposal with
grading deltas only. Nothing in this work-up substitutes for that verdict.

**B. Fleet-provisional rows (S9–S12) wait on the fleet pack's definition.** The fleet pack's direction/sweep (SWEEP.md) is
Nox-side and not on this seat; these rows stay provisional. **S9 additionally waits on the R1 boundary decision** (in-app
only vs distributed components bundle → clean-room/drop). S11/S12 are clean-room/pattern-only by licence (R4/R3) and can
never ship code from their sources.

**C. Licence-side open item:** the **Uiverse per-element provenance record (R2)** is created at admission time (author +
element URL + fetch date); no bulk dumps; an element whose origin looks derivative is not admitted.

**D. Image-dependent step wall:** **G3 screenshot compare must not run in this DSH seat** (in-process image decode is the
host crash class). Criteria are authored here; the actual comparison is performed by a seat permitted to view images (or
via host-side text/metadata evidence) and attaches named differences — never "looks close".

**E. Nothing here waits on Job 03 / the RUN / the package build.** The parts bin is downstream of owner grading, not of G1.

## 6. Turnover — what the Lead/Nox do next, and what the checker will receive

1. **Lead:** land this report in `origin/team` (the landing rule); no action needed on any repo branch from this work-up.
2. **Owner:** grade the comps (graded deltas only). That single verdict unblocks S1–S8's G2 work.
3. **Nox:** confirm the fleet pack's definition (SWEEP.md) and rule the S9 R1 boundary; both are external to this lane.
4. **Later, when a row's gate actually starts:** the evidence package handed to the fresh checker is
   `<row>/G2-states/**` + `<row>/G3-compare/named-diff.md` + `<row>/G4-verdict.md` + the admission record; the checker must
   have built none of it. The G1 proposals (this file + the shortlist) are the checker's starting point.
5. **Status now:** all 12 rows at **G1 complete, G2/G3 authored, not executed; 0 admitted, 0 vendored, 0 purchased.**

## 7. Fences held

No purchases (Forge UI Pro and Uilora plans unmade); no code vendored into any repo or pack; **no pack admission**; no
screenshots/image reads in DSH; no credentials; no sends/publication; **$0 spend**. Only writes this batch: this report
(Team A board scope). The four Nox RUN-2 rulings above are recorded here and applied where they touch the packs (C-REM,
C-W1); the Job 03 and candidate-branch rulings are recorded for the lane record with no action taken.
