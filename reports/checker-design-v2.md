# CHECKER — Ergon design v2 direction set (frozen `board/probes/team-a-design-v2/`) — verdict: **REJECT**

Fresh-context, independent pack-conformance + unslop review. I built none of this set (maker-never-checker). No image
reads / screenshots (DSH crash class), no host input, no repo edits, no network, **$0**. Frozen set untouched: the
17-file sha256 manifest is identical before and after (`board/probes/checker-design-v2/frozen-sha256.txt` vs
`/tmp/checker-design-v2/frozen-after.txt`; `FROZEN SET UNCHANGED`, 2026-11-01T20:12:06-05:00). All tools ran from a
read-only copy `/tmp/checker-design-v2/set`; mutations on throwaway copies only.

Reviewed against: the current pack rules `~/Documents/Codex/2026-10-23/task/ergon-phase0/repo/DESIGN.md` (Midnight
Slate; the task path's `repo/design/DESIGN.md` does not exist — the pack file is `repo/DESIGN.md`; theme supersession
noted), `board/probes/team-a-pack-conformance-checklist.md`, `PACK-CHANGE-PROPOSAL.md`, the owner direction
`~/Work/shared-workflows/lanes/board/inbox/2026-11/20261102T005200Z-nox-to-delegator.md`, and Team A's
`board/reports/team-a-design-v2.md`. The DSH board copy of the frozen set is byte-identical to the shared-workflows
lanes copy (`diff -rq`, empty).

## Verdict in one line

**REJECT** — the frozen set's central claim does not render: `--surface-1/2/3` are used 35 times but the generator emits
`--surface1/2/3`, so every translucent plane background is invalid/transparent (one-line generator fix, validated
below); and three kept pack floors are not met/not evidenced in the comps (field boundary 1.55:1 vs the pack's 3:1
fieldLine rule, control boundary 2.97:1 once the page's second ambient wash is counted, 36 px controls and no 56 DIP
employee target). The direction itself reads well in code (no unslop tells, real motion, real floors work); after the
named fixes and a re-freeze it should be presentable.

## Per-leg table

| # | Leg | Verdict | Evidence (see §Evidence paths) |
|---|---|---|---|
| 1 | Frozen set / provenance integrity | **PASS** | 17 files, 2,030 lines, sha256 manifest unchanged; DSH↔lanes copies identical. Note: report says "16 files" — actual 17 |
| 2 | Team A tools re-run (read-only copy) | **PASS** | `node build-tokens.mjs --check` → `TOKENS: CURRENT`, rc 0; `node check-comps.mjs` → `V2 COMPS CHECK: PASS`, rc 0; `node --check comp-motion.js` rc 0; storyboard inline script extracted + `node --check` rc 0 (20:10:30-05:00) |
| 3 | Checker assertions non-vacuous | **PASS with gaps** | Mutations that must fail do fail (ext URL, hex literal, floor, label, motion budget — all rc 1). But 4 mutations it must arguably catch still PASS: broken `</div>`, missing local `<script src>`, `color: white`, `animation: float 2s infinite` (§Defects D6) |
| 4 | Self-containment / offline | **PASS** | 0 external refs/assets/webfonts/images on all 9 pages; 35 local `href/src` refs, 0 missing; only URL-shaped string is the SVG XML namespace in `comp-motion.js` (a name, not a fetch); no network use |
| 5 | Tokens-only discipline | **PASS (colours) / PARTIAL (sizes)** | 0 hex/rgb/hsl literals in rendered code (only `transparent`/`currentColor` keywords; `PACK-CHANGE-PROPOSAL.md` quotes the reference palette as docs; `check-comps.mjs` has its own math strings). But 7 controls use inline `min-height:36px`, tabbar `58px`, frame dims/radii are literals — sizes bypass the token set |
| 6 | Contrast floors (independent recompute) | **FAIL** | Their model reproduces exactly (ink 10.03, muted 6.13, subtle 6.64, accent 5.46, status 5.08/5.37/5.11, focus 7.05, border 3.08). But the checker omits the second ambient wash (`ambientCool`, 0.035) that the page actually composites → border **2.968:1 < 3.0**; and the input field border (`--line`) is **1.55:1**, far below the pack's `fieldLine` 3:1 rule (v1 had 3.24:1). Neither is asserted |
| 7 | Rendering / translucent material | **FAIL — blocker** | `var(--surface-1/2/3)` ×35 has no definition: generator emits `--surface1/2/3` (kebab does not split letter→digit), tokens.css has no aliases, no fallbacks → every `background` is invalid at computed-value time → transparent. All planes render as outlines/shadows over the page background (flat/ghost), i.e. the owner's "panels are flat" failure mode. Not caught by the self-check (it never resolves custom properties) |
| 8 | Motion | **PASS** | Tokens 120/200/250/350 + one `cubic-bezier(0.2,0.7,0.2,1)`; all live motion is JS/WAAPI reading the tokens (reveal/send/approval/character/tabs/sheet) plus tokenised CSS transitions; no keyframes, no `infinite`, no `transition-all`, no pulse/ping/glow, no parallax; storyboard runs the four budgets + labelled ×0.25 grading aid |
| 9 | Reduced motion / High Contrast | **PASS with gap** | Real `@media (prefers-reduced-motion: reduce)` + `[data-motion="reduced"]`; JS `reduced()` gates every WAAPI call (travel/stagger removed); real `@media (forced-colors: active)` path; `[data-contrast="high"]` preview + labelled Floors preview. Gap: the HC **preview** covers surfaces/buttons/pills/body only — not inputs, links, `.msg`, character, `.state-dot` |
| 10 | Pack anatomy / states / targets | **FAIL** | Anatomy present (buttons primary/secondary/ghost/icon/disabled/busy, fields, pills-with-words, 2 px accent ring + 2 px gap, one accent per visible surface). But: 7 controls at 36 px (<44 DIP); **no `class="employee"` anywhere** → mobile employee screens use 44 px, while v1 used the 56 DIP employee target in comps 01/06 (kept floor "56 on employee screens" no longer evidenced); **empty state missing** from 07 (v1's 07 was "Empty / loading / error / offline" and had one); comp 02's "Approve once" is a dead control though its note says the action resolves |
| 11 | Labelling / no built-surface claims | **PASS** | 9/9 pages carry the visible "DESIGN COMP — NOT A BUILT SURFACE" banner; index/report state "direction pass, not a finished front end"; no product/runtime claims found |
| 12 | Pack-change proposal coherence | **PASS with nits** | New tokens (surfaces/blur/depth/border ladder/ambient/motion/character), version `2.0.0-draft-1`, v1 stays shipped, rollback = token revert, risk register, floors kept. Nits: its floor table omits the field boundary and the second ambient; `blur.deep 32px` and `light1` are defined but unused (`light1` cannot carry text at floors: muted 4.03 / accent 3.59) |
| 13 | Unslop / taste (code-level) | **PASS** | No gradient text/buttons (only the two single-hue ambient radial washes), no glow/text-shadow/drop-shadow, no emoji/sparkle badges, weights only 600, no uppercase tracked labels (mono labels are lowercase +0.02em), no `transition-all`, no loops, hairlines + radius mix + mono captions give the technical-editorial rhythm. Taste calls for the owner: ambient wash pair (blue+teal) and depth shadows are the deliberate supersessions, not slop tells |

## Defects, smallest fix first (all are new-freeze material)

- **D1 (blocker) — undefined surface custom properties.** `tokens.css:9-12` emits `--surface1/2/3` + `--light1`;
  `comp-kit.css`, `comps/05`, `comps/07`, `motion-storyboard.html` use `--surface-1/2/3` (35 uses). No fallback →
  `background` transparent for `.glass*`, `.card`, `.phone`, `.desktop`, `.sheet`, `.msg*`, `.banner`, `.inset`,
  `button`, `.pill`, composer/tabbar, `.layer` etc. **Smallest fix:** in `build-tokens.mjs` add a digit split to
  `kebab()` (`...replace(/([a-z])([0-9])/g, '$1-$2')...`) and regenerate `tokens.css`. Validated in a throwaway copy:
  tokens.css then has `--surface-1/2/3`, **0 unresolved var()**, `node check-comps.mjs` still PASS (`fix-validation.log`).
  (Renaming the JSON keys to `surface_1…` also works but would require touching `check-comps.mjs`'s math reads.)
- **D2 — kept field-boundary floor lost.** v1 used `fieldLine` #5A6A80 (3.24:1) for text fields; v2 has no fieldLine
  token and inputs use `--line` → **1.55:1** on surface-1/panel. **Fix:** add a ≥3:1 field-boundary token (e.g.
  `fieldLine` ≈ the v1 value, re-measured against the translucent stack) and use it for inputs; assert it in the checker.
- **D3 — control-boundary floor is optimistic.** The checker composites `ambientAccent` only; the page also layers
  `ambientCool` (0.035), which drops `lineBright`-on-card from 3.08 to **2.97** (top-right region of a typical
  viewport receives the cool wash at near-full alpha). **Fix:** include both washes in the checker composite and retune
  `lineBright`/`ambientCool` to keep ≥3.0; or drop the second wash.
- **D4 — target floors.** Seven controls use inline `min-height:36px` (comp 05 ×1, comp 06 ×6) — below the 44 DIP
  minimum; and the 56 DIP employee target is never applied in v2 (v1 used `class="secondary employee"` in comps 01 and
  06). **Fix:** remove the 36 px overrides (use `var(--target)`), put `employee` on the mobile employee-screen buttons,
  and add a target-size assertion to the checker (it only greps for the CSS rule's existence).
- **D5 — state coverage.** Empty state missing from comp 07 (v1 had it); the checklist's empty/reduced-motion/HC rows
  are not rendered per surface (HC is only a preview toggle). **Fix:** restore an empty-state specimen (v1's "No tasks
  yet" + one next step) and label the reduced-motion/HC rows in 07.
- **D6 — checker coverage gaps (evidence quality).** The report claims "div balance and aria-controls/sheet-target ids
  checked on all 9 pages"; the frozen `check-comps.mjs` does not implement it (a dropped `</div>` still PASSes). Also
  missed: local asset refs that do not resolve (missing `<script src>` PASS), named CSS colours (`color: white` PASS),
  animation shorthand loops (`animation: float 2s infinite` PASS), custom-property resolution (D1 slipped through),
  the second ambient wash, and the field boundary. The report's "16 files" is 17. **Fix:** add those assertions (or
  correct the report's claims) before re-freezing; my independent structural pass did confirm 9/9 balanced pages, no
  duplicate ids, no missing `aria-controls` targets, so the set itself is structurally sound.
- **Minor (non-blocking):** comp 02 "Approve once" has no `[data-approval]` ancestor → dead control (its grading note
  says it resolves); comp 06 density-14 renders 16 nodes while labelled 14; comp 06 caption "idle — core in ink" is
  wrong (core uses `--nn-hue`/accent); density "24 px legibility" claim is not demonstrated (`.nn` is fixed 34 px);
  native checkbox (02 remember row, storyboard slow-mo) keeps the UA focus ring, not the 2 px accent ring.

## Contrast recompute (independent implementation; `contrast-recompute.log`)

Their model = base + ambientAccent + surface1→2→3, rounding each composite (matches their code):
ink 10.031, muted 6.127, subtle 6.644, accent 5.456, ok/warn/danger wash 5.081/5.376/5.111, focus 7.052, border 3.080
— all claims reproduce exactly. Actual page model adds `ambientCool` 0.035: ink 9.302, muted 5.681, subtle 6.187,
accent 5.059, status 4.710/4.979/4.756 (still ≥4.5), focus 6.527, **border 2.968 (<3.0)**. Unasserted boundaries:
input border `--line` on surface-1 **1.549**, on card **1.552**; `lineStrong` on card 1.932. Latent (unused) tokens:
`light1` as a text plane gives ink 6.60 / muted 4.03 / accent 3.59 (below floors); `pill.accent` (unused) gives 4.19
with the cool wash. `onAccent` on accent/accentHover/accentPress = 9.70/11.66/8.01 ✓.

## Unslop / taste findings (code-level; no images read)

No "AI slop" tells found in the code: no purple→blue ramp, no gradient text, no glow/glowing pills, no emoji icons, no
sparkle/"New" badges, no 800 weights, no uppercase tracked labels, no off-grid stampede (space tokens used), no
greeting/hype copy, no rainbow charts, no `transition-all`, no pulse/ping loops, no stock icon soup (only a text "↑"
and "…"), no autoplay carousel (the artefact stack is a deliberate, recorded decline of the carousel). The character is
a real deterministic node-and-connection SVG (seeded by name), not a decorative motif loop; motion is one-shot and
purposeful. Taste calls for the owner, not defects: the two single-hue ambient washes (blue + teal) and the ambient
depth shadows are the directed supersessions of the flat/no-shadow v1 rule — judge them rendered.

## Rendering limit — what a visual seat must still confirm

I could not render the pages: image reads/screenshots are the DSH crash class, and a headless-Chromium computed-style
probe I attempted hung in this environment and was killed (no process left; not used for the verdict). The D1 finding
is deterministic from the CSS: an unset `var()` with no fallback makes the declaration invalid at computed-value time,
so the plane `background`s compute to transparent; borders/shadows/blur still paint. **A human/visual seat must
confirm after D1's fix:** that the translucent planes and rim light actually read as "light through surfaces" (not
glass/milky), that blur is bounded and depth is consistent, that the ambient field is a wash and not a gradient tell,
that the neural mark reads at 34 px/24 px, that motion feels polished at 120/200/250/350, and that the Floors preview
(reduced motion + HC) looks right.

## Presentation recommendation (after D1–D5 + re-freeze + re-run of this check)

1. `index.html` — front door: owner-grade mapping, checklist, Floors preview, Replay motion.
2. `motion-storyboard.html` — press **Replay the sequence**, then tick **slow motion ×0.25** (show the 120/200/250/350
   budgets and the single curve; this is the answer to "no animations").
3. `comps/07-states.html` — the finish judgement: states + the three-plane material study (and the restored empty
   state).
4. `comps/02-mobile-triage-approvals.html` — press **Review**: the sheet at 350 ms (depth showpiece); Escape/Later to
   close.
5. `comps/01-mobile-chat.html` — send a message (enter 250) and tap the neural character (one-shot signal).
6. `comps/05-desktop-workspace.html` — tab swap (200) + Change state; then `06` (character), `03`/`04` last.
Motion to show running: storyboard sequence + slow-mo; 02 sheet; 01 send/character tap; 05 tab swap.

## Evidence paths

`board/probes/checker-design-v2/` — `frozen-sha256.txt` (17 files), `frozen-key-sha256.txt`, `run-started.txt`,
`tool-rerun.log` (tools + rc), `mutations.log` (9 mutations: 5 caught / 4 gaps), `contrast-recompute.py` +
`contrast-recompute.log`, `css-var-resolution.log` (D1: 35 uses, 3 missing properties, spec consequence),
`fix-validation.log` (one-line generator fix → all vars resolve, checker PASS), `v1-comparison.log` (employee/fieldLine/
empty-state regressions), `unslop-scan.log`, `audit-summary.log`, `audit.py` + `audit.json` (structure/labels/refs/
colours/targets/states). Throwaway copies: `/tmp/checker-design-v2/{set,mut,fix}` (frozen set read-only; mutations and
the fix only on copies). Frozen set re-hashed after the review: identical.
