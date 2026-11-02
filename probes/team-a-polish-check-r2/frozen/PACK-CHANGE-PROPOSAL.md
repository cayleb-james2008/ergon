# Pack-change proposal — Midnight Slate → **Ergon Lumen** (v2 direction)

**STATUS: PROPOSAL ONLY, FILED WITH THE v2 COMPS FOR THE OWNER'S GRADE.** No pack file in the repo has been changed, no
`design/tokens.json` on `main` is touched, no code, no runtime, no release. This is the pack-change proposal the owner's
direction asks for, presented together with the v2 design comps (`board/probes/team-a-design-v2/`); the full report is
`board/reports/team-a-design-v2.md`. If the direction passes, the pack update follows the pack's own gate (proposal →
all-states → screenshot compare → fresh checker) and a docs-only repo change; if it does not pass, Midnight Slate v1.0.0
stands untouched. **$0, no purchases, no image reads, no build.**

## 1. What the owner directed (the authority for this change)

- **Accepted:** layout and structure — buttons/screens/functional skeleton; the functional model (Duet feel, one personal
  agent, Chat/Tasks/Make, approvals as cards, character pinned, desktop multi-task) **stands and is not reworked**.
- **Failed:** the finish — "it still looks like AI slop… 100% could use a ton of work"; components "do not look finished
  at all"; "no animations, panels are flat, everything colour scheme". v1 was an initial iteration; nobody presents v2 as a
  finished front end either — the finished front end is built only after a direction passes his grade.
- **Directed vein for v2:** Ergon's own translucent identity (layered translucent panels, real depth, blur, fine borders,
  light through surfaces) in place of flat slate; taste reference `primeintellect.ai` (taste only); character = a little
  neural net; motion part of the grade, presented as live pages; every accessibility floor kept.

## 2. Change summary

| | |
|---|---|
| **Superseded** | Midnight Slate as **Ergon's front-end theme** — specifically its flat-panel discipline: "depth comes only from `base → panel → raise`"; "no gradients, glass, shine, reflection or drop shadows" (the owner now asks for translucency/blur/depth); a character that is a shape, not a network. |
| **Kept, unchanged** | One indigo-family accent used for exactly one forward action per view; tokens-only discipline (no hand-typed hex in product code); 44 DIP minimum targets and 56 DIP employee buttons; 2 px accent focus ring with 2 px gap, never animated; type scale (IT 14/22/12.5, employee 16/24/14); semibold headings, no all-caps employee text; pills always carry words; plain-language rules; the unslop gate; contrast floors; reduced-motion and High Contrast paths. The **pack anatomy** (foundations → components/all-states → behaviour/motion → version/changelog) is unchanged. |
| **New** | Translucent surface layers (alpha planes + blur), depth tokens (lift + inset rim), a three-weight border ladder, a controlled ambient light field, motion tokens promoted into the pack (durations + one curve), and the neural-network character identity (structure/density, colour, name). |

## 3. What the new token set adds (values live in `tokens.json`)

1. **Surface layers:** `surface-1 / surface-2 / surface-3` (translucent, stacked) plus `light-1` for the brightest
   selected plane. Depth hierarchy is the stack, not a drop shadow.
2. **Blur:** `blur.panel 14px · blur.card 18px · blur.sheet 24px · blur.deep 32px` (measured, not arbitrary).
3. **Depth:** `lift` / `liftHigh` / `rim` / `rimStrong` / `well` — ambient shadow + a 1 px inset rim highlight so light
   reads as passing through the edge of a plane.
4. **Border ladder:** `line` (dividers) → `lineStrong` (plane edges) → `lineBright` (interactive boundaries and focus
   context). `lineBright` is specified to meet the 3:1 non-text floor against a card plane (measured 3.08:1).
5. **Ambient field:** two very low-alpha single-hue washes (`ambientAccent`, `ambientCool`) as the page's light source —
   not a gradient ramp, not a gradient button, not rainbow.
6. **Motion:** `tap 120 · swap 200 · enter 250 · sheet 350` and one `ease` curve, now **pack tokens** (v1 had the numbers in
   a motion spec but not tokenised).
7. **Character:** density (6/10/14 nodes), one hue token, name — the light customisation contract; state (idle / working /
   needs you) changes the core colour and always carries a word.
8. **Radii:** `panel 14 · card 12 · control 8 · inner 6 · pill 999` (a fine/soft mix, matching the technical-editorial
   taste reference rather than one uniform roundness).

## 4. Accessibility floors — kept, and measured (not asserted)

`check-comps.mjs` composites the translucent stack (base + ambient + surface-1/2/3) and asserts the floors against the
**brightest text-bearing plane**, so translucency cannot quietly erode contrast:

| Floor | Assertion | Measured on the v2 tokens (both ambient washes composited) |
|---|---|---|
| Body text | `ink` ≥ **7:1** on the brightest text plane | **9.21:1** |
| Secondary text | `muted` ≥ **4.5:1** on the brightest text plane | **5.62:1** |
| Quiet text | `subtle` ≥ **4.5:1** where it is allowed (base/panel only by rule) | **6.14:1** on surface-1 |
| Accent text | `accent` ≥ **4.5:1** | **5.01:1** |
| Status text on its wash | ok/warn/danger ≥ **4.5:1** | **4.66 / 4.93 / 4.77:1** |
| Focus ring | ≥ **3:1** non-text against a card plane | **6.47:1** |
| Control boundary | ≥ **3:1** non-text against adjacent plane — **with the page's second ambient wash counted** | **3.28:1** (`lineBright` retuned to alpha 0.44 in the fix round) |
| Field boundary | ≥ **3:1** non-text (pack `fieldLine` rule) — field fill vs border | **3.30:1** (new `fieldLine` token, alpha 0.46) |

The fix round after the checker's REJECT corrected the composite model (both ambient washes) and these two boundaries; the
earlier figures in this table (border 3.08, no field row) were computed on one wash and are superseded. The remaining
gate: a rendered visual check by an image-permitted seat (the DSH app cannot read images — crash class).

Kept by construction: 2 px accent focus ring with 2 px gap on every control; `prefers-reduced-motion` floor (travel and
stagger removed, state changes instant); `forced-colors: active` High Contrast path (translucency, blur and soft depth
replaced by system colours and 1 px borders); decorative motion off under High Contrast. `subtle` is **restricted to
base/panel contexts** — it is not allowed on the brightest plane (the one exception found in the comps was fixed).

## 5. Motion contract (part of the grade this time)

- **Tap 120 ms** — press/selection feedback. **Swap 200 ms** — tab/view replacement. **Enter 250 ms** — content arriving.
  **Sheet 350 ms** — approvals sheet in/out. **One ease-out curve** shared by all four.
- Motion is **JS-driven** in the comps (Web Animations API; durations read from the generated tokens) so the owner sees it
  running in his browser. **No loops, no pulse/ping, no glow, no parallax, no autoplay carousel, no `transition-all`.**
- Reduced motion: all travel/stagger removed; the interface still changes state. High Contrast: decoration off.
- A labelled **slow-motion ×0.25 grading aid** and a **Floors preview** toggle exist on the storyboard/index — they are
  grading aids, clearly labelled, not product features.

## 6. Character contract

Node-and-connection mark drawn from a deterministic seed (same name → same network, so an employee's character is stable).
Customisation is light by design and matches the accepted ruling: **structure (density 6/10/14), colour (one hue token),
name**. State: idle (core in the character hue), working (core `ok`, one signal along the edges), needs you (core `warn`).
The signal is a single directional pass per state change — no attention-seeking loop, no glow. It reads at 24 px in the
desktop sidebar and 34 px in the header.

## 7. Taste study record — primeintellect.ai (taste only, words)

Studied at **HTML/CSS level only** (pages and stylesheets fetched as text; **no screenshots, no image reads** — DSH crash
class). What was taken is *taste*, expressed as rules, not content:

- **Near-black neutral ladder, not blue-slate:** surfaces step through near-blacks (≈`#050606` → `#0a0a0a` → `#0e0e0e` →
  `#141415` → `#191a1a` → `#2c2d2e` in their own palette) — ours keeps that idea with a cooler base and translucent white
  planes over it.
- **Hairlines, not borders:** 1 px lines at very low white alpha (their CSS uses `#ffffff0f` / `#ffffff1a`-class overlays);
  borders are fine and quiet, and only interactive boundaries step up. Ours: `line → lineStrong → lineBright`.
- **Translucency with restraint:** their stylesheets use `backdrop-filter: blur(6/8/12/24px)` and low-alpha white overlays
  (≈6–24%); ours uses 14/18/24 px blur and 4/7/10% plane alphas — same material idea, different scale.
- **Type:** an editorial sans (`ABC Favorit`) with a mono companion (`ABC Favorit Mono`), a fine scale from ~10 px to 48 px,
  **negative tracking on display text** (−0.02…−0.045em) and tight leading on headings. Ours uses the system sans + mono
  companions (no webfonts allowed) and the same tracking discipline on display headings; **their positive tracked
  uppercase micro-labels are deliberately NOT taken** — the unslop gate bans uppercase tracked labels.
- **Radii:** a mix of fine 2–6 px controls and 12–24 px planes — not one uniform pill world. Ours: 6/8/12/14 px + pill.
- **Motion:** their CSS transitions sit at 120–300 ms on background/border/colour/shadow — consistent with our 120/200/250/350
  budget; ours uses one curve and no spectacle.
- **Composition:** numbered sections, mono code blocks, metric callouts, generous space, one accent per view — a technical
  editorial rhythm. Ours keeps that restraint in the comps' annotation columns and section labels.
- **Not taken:** their layout, copy, marks, logos, colours, fonts, icons or any asset. No network dependency is introduced.

## 8. Versioning, migration, rollback

- Proposed pack version: **`2.0.0-draft-1`** ("Ergon Lumen"), a **direction proposal** — not a release.
- **Midnight Slate v1.0.0 stays the shipped pack** (installer UI, support desk, generated artefacts) until the owner's grade
  and the pack gate pass; nothing regenerates early.
- If accepted: the pack update is a docs-only change to the pack files (`DESIGN.md` tokens/anatomy/version, `design/tokens.json`,
  `design/motion-spec.md` → behaviour/motion, generated outputs via the existing generator) plus a fresh-checker run; the
  **already-shipped surfaces keep v1 until each is rebuilt**, so there is no mixed-theme window claimed as done.
- **Rollback:** v1 tokens remain in git history; the v2 token file is additive in structure (v1 names kept where meaning is
  kept: ink/muted/accent/status/targets/type/space), so a revert is a token-set revert, not a rewrite.
- The identity name "Ergon Lumen" is a working title; the owner/Nox may rename it — the tokens are the substance.

## 9. Risk register for the pack-conformance + unslop checker

| Risk | Why it could fail a gate | Mitigation in this proposal |
|---|---|---|
| Translucency legibility ("glass" tell) | Unslop/design review may read blur+alpha as the glass/slop tell | Blur is bounded (14/18/24), planes are near-black not milky, borders do the work, no shine/reflection/gradient; contrast measured on composites |
| Ambient light field read as "gradient" | A gradient wash could be seen as a slop gradient | Two single-hue low-alpha washes only; no colour ramp, no gradient text/buttons; can be removed without touching any other token |
| Depth shadows vs "no drop shadows" rule | That rule belonged to Midnight Slate; the owner's direction supersedes it | Proposal states the supersession explicitly; shadows are ambient (`lift`) and paired with an inset rim, not glossy elevations |
| Motion perf / jank on low-end hosts | Live JS motion must feel polished | WAAPI, transform/opacity only, no layout animation, bounded staggers, reduced-motion path |
| Contrast on brightest plane | Translucency can lift text backgrounds | Floors asserted on the *brightest text plane* (computed, §4); `subtle` restricted to base/panel |
| Character reads as "AI decoration" | A network could look like a generated motif | Deterministic seed from the employee's name; state is functional; no glow/pulse; customisation limited to density/colour/name |
| Tokens-only discipline breaks | Hand-typed hex sneaks into comps | `check-comps.mjs` fails on any colour literal outside `tokens.json`/generated `tokens.css` |

## 10. What this proposal does NOT do

No repo change (no branch, no PR, no `design/` edit), no product code, no runtime, no package/app/plugin build, no purchase,
no public post. It does not mark any comp "done": the maker's check is evidence for the checker, never the checker's verdict.
It does not re-open or discard the **frozen parts-bin curation gate** — those fences are re-validated against this identity
only after v2 passes the owner's grade.

## 11. Approval path

1. Lead preps and spawns the **pack-conformance + unslop checker** on the frozen v2 comps (maker never checker).
2. Checker verdict lands; only then are the live comps presented to the **owner in his browser** (floors + motion running).
3. Owner grades the **direction** (graded deltas only; comps are not reworked ahead of his words).
4. If the direction passes: this proposal moves to the pack update + build gates. If it fails: v1 stands, this file records
   the attempt and the next direction iteration starts from his deltas.
