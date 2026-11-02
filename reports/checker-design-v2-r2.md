# CHECKER R2 — Ergon design v2 RE-FROZEN set (`board/probes/team-a-design-v2/`, digest `6e5f4cff…`) — verdict: **ACCEPT-WITH-NAMED-CONCERNS**

Fresh-context re-run of the full battery on the fixed freeze. I built none of it (maker-never-checker). No image reads /
screenshots (DSH crash class), no browser, no host input, no repo edits, **$0**. Frozen set verified before and after:
`sha256sum -c FREEZE.sha256` rc 0; manifest digest **`6e5f4cfff3f29204a22886e7e0fd79108d52a0395821df1879c4ec4fbac94f9b`**
(matches the Lead's digest); **17 content files, 2,159 lines + `FREEZE.sha256`**; DSH board copy byte-identical to the
shared-workflows lanes copy. All tools/mutations ran from a read-only copy `/tmp/checker-design-v2-r2/set`.

Reviewed against: `~/Documents/Codex/2026-10-23/task/ergon-phase0/repo/DESIGN.md` (pack rules; theme superseded by the
owner direction), `board/probes/team-a-pack-conformance-checklist.md`, `PACK-CHANGE-PROPOSAL.md`, the owner direction
`~/Work/shared-workflows/lanes/board/inbox/2026-11/20261102T005200Z-nox-to-delegator.md`, and the updated report §6b.

## Verdict in one line

**ACCEPT-WITH-NAMED-CONCERNS** — the previous blockers are genuinely fixed and independently verified (D1: 0 unresolved
custom properties, 73 defined; D2: both-wash floors, field/control boundaries, 44/56 targets, empty state, dead control
all fixed in the frozen bytes; D3: the checker now catches every mutation that previously slipped through; D4: counts
correct). Residual, all small and named below: the message **composer** boundary is still `--line` (1.55:1, pack field
rule 3:1) while `input[type=text]` is fixed; the self-check's freeze integrity is conditional (deleting the manifest or
adding an unlisted file still PASSes); the raw-colour-keyword detector misses a first-declaration inline style; native
checkboxes keep the UA focus ring. None of these blocks the direction grade; C1 is a two-line fix I would apply before
the owner sees it.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | Freeze / provenance integrity | **PASS** | `sha256sum -c` rc 0; digest matches; 17 content files / 2,159 lines + manifest; mirror identical; frozen set unchanged after the review (final sha256 manifest identical) |
| 2 | Team A tools re-run (read-only copy) | **PASS** | `node build-tokens.mjs --check` → `TOKENS: CURRENT` rc 0; `node check-comps.mjs` → `V2 COMPS CHECK: PASS (hardened after checker D1–D4)` rc 0; `node --check comp-motion.js` rc 0 (20:24:20-05:00) |
| 3 | D1 token-kebab fix (blocker) | **PASS** | `kebab()` now splits letter→digit; `tokens.css` emits `--surface-1/2/3` + `--light-1`; my independent resolver: **72 tokens.css defs + 1 kit-local = 73, 0 unresolved, 64 var() uses**; 36 surface-var uses all resolve; `--light-1`/`--blur-deep` now used (decorative light pool, no text) |
| 4 | D2 floors retuned | **PASS** | Independent recompute (both washes, CSS layer order accent-over-cool-over-base) matches every stated floor to 2 dp: ink **9.209** (claim 9.21), muted **5.625** (5.62), subtle **6.138** (6.14), accent **5.009** (5.01), status **4.661/4.931/4.766** (4.66/4.93/4.77), focus **6.467** (6.47), control **3.277** (3.28), field **3.296** (3.30); all above floors (7 / 4.5 / 3). `lineBright` α 0.44, `fieldLine` α 0.46 in the frozen tokens |
| 5 | D2c/d/e/f fixes in bytes | **PASS** | inline `min-height:36px` in pages = **0**; `employee` counts 01=3, 02=4, 03=3, 04=4, 05=0, 06=6, 07=1 (matches claim); 07 has `class="empty"` + "No tasks yet." + one next step + labelled reduced-motion/High Contrast rows; comp 02 sheet carries `data-approval` and `data-approve`, and the JS branch writes "Approved once — recorded" and disables the card's buttons |
| 6 | D3 checker hardening / non-vacuity | **PASS with residual gaps** | My mutation battery (19 cases): every previous slip now FAILs — broken `</div>` (tag balance), missing local `<script src>` (local-ref resolution), `color: white` (keyword detector), `animation: float 2s infinite` (loop detector), `var(--surface-9)` (resolution), stale `tokens.css`, unfrozen edit (FREEZE mismatch), 36 px inline control, `lineBright` 0.40 (→2.97 fail), `fieldLine` 0.10 (→1.30 fail), ink break (→2.41 fail), motion budget; all-employee removal fires 5 findings. Residual gaps C2/C3 below |
| 7 | Self-containment / offline | **PASS** | 0 external refs/assets/webfonts/images; 35 local `href/src` refs, 0 missing; only URL-shaped string is the SVG XML namespace (a name, not a fetch); no network |
| 8 | Tokens-only discipline | **PASS (colours) / PARTIAL (sizes)** | rendered files: 0 hex/rgb/hsl literals, only `transparent`/`currentColor` keywords; sizes: no sub-44 control overrides; a few layout/frame dims and radii remain literals (as in v1; not a floor breach) |
| 9 | Motion | **PASS** | tokens 120/200/250/350 + one `cubic-bezier(0.2,0.7,0.2,1)`; JS/WAAPI reads the tokens; no keyframes, `infinite`, `transition-all`, pulse/ping/glow, parallax; storyboard live + ×0.25 grading aid; reduced-motion honoured in CSS and JS |
| 10 | Reduced motion / High Contrast | **PASS** | real `prefers-reduced-motion` + `forced-colors`; HC preview now covers inputs, messages, links, character, state dots, empty state, light pool (the earlier gap) |
| 11 | Pack anatomy / states / targets | **PASS** | 44 DIP minimum (no sub-44 controls), 56 DIP employee target restored on employee surfaces, 2 px accent ring + 2 px gap, pills carry words, one accent per visible surface (02's sheet is its own modal surface; 07 is a specimen), empty state restored; native checkbox focus ring is the one residual (C4) |
| 12 | Labelling / no built-surface claims | **PASS** | 9/9 pages carry the visible "DESIGN COMP — NOT A BUILT SURFACE" banner; README/report/proposal keep the direction-pass framing |
| 13 | Pack-change proposal coherence | **PASS** | Floor table updated with both-wash model + field boundary; version `2.0.0-draft-1`, v1 stays shipped, rollback = token revert, risk register, floors retained; `light-1`/`blur-deep` now used and the no-text rule on `light-1` is stated on the surface |
| 14 | Unslop / taste (code-level) | **PASS** | no gradient text/buttons (only the two single-hue ambient radial washes), no glow/text-shadow/drop-shadow, no emoji/sparkle, weights only 600, no uppercase tracked labels, no loops/`transition-all`, hairlines + mono captions; the new light pool and empty state introduce no tells. Taste calls for the owner: the ambient wash pair and ambient depth shadows are the directed supersessions |
| 15 | Rendered look | **UNVERIFIED (limit)** | no image reads; a headless computed-style probe hung earlier and was killed; static certainty is now that the material's CSS resolves (D1). A visual seat must still confirm the rendered look (§ below) |

## Residual concerns, smallest fix first

- **C1 (must-fix before the owner sees it) — message composer boundary.** `input[type=text]` now uses `--field-line`
  (3.30:1), but the message composer is a borderless input inside a container bordered with `--line`:
  `comp-kit.css:150` `.phone .composer { border: 1px solid var(--line); … }` and the same inline style on comp 05's
  composer → visible field boundary **1.55:1** (computed), below the pack's `fieldLine` 3:1 rule. The self-check asserts
  the field boundary only on the generic input, not the composer. **Fix:** set the composer border to
  `var(--field-line)` in `comp-kit.css:150` and comp 05's inline style (2 lines), or give `.phone .composer input` the
  `--field-line` border; add a composer assertion.
- **C2 — freeze integrity is conditional.** Deleting `FREEZE.sha256` still PASSes (`freeze: absent`), and adding an
  unlisted file PASSes too. **Fix:** fail when the manifest is absent, and assert the directory contains exactly the 17
  listed content files + the manifest.
- **C3 — raw-colour-keyword detector misses a first-declaration inline style.** `style="color: white"` (colour as the
  first declaration after the opening quote) PASSes, because the regex requires `^`, `;` or `{` before the property;
  `style="margin:0; color: white"` is caught. **Fix:** scan `style="…"` attributes separately or allow the `"` start.
  (The frozen set itself has no such literal — this is checker coverage, not a set defect.)
- **C4 — native checkbox focus ring.** The comp 02 remember checkbox and the storyboard slow-motion checkbox keep the UA
  focus ring rather than the pack's 2 px accent ring. **Fix:** `input[type="checkbox"]:focus-visible { outline: var(--focus-ring) solid var(--accent); outline-offset: var(--focus-gap); }`.
- **C5 — latent token.** `pill.accent` is defined but unused; on a wash it would be 4.20:1 with both washes (<4.5). Fix or
  remove the class before it is used. (`light-1` text would be below floors — now documented and enforced as
  decorative-only with no text, verified in the DOM.)
- **C6 — rendered look.** A visual seat must confirm: planes read as "light through surfaces" (not milky glass), blur
  bounded and depth consistent, ambient reads as a wash not a gradient tell, neural mark legible at 34 px, motion feels
  polished at 120/200/250/350, Floors preview (reduced motion + HC) looks right.

## What the owner should see first (presentation held per Lead; order for the wake-up report)

1. `index.html` — front door, grade mapping, Floors preview, Replay motion.
2. `motion-storyboard.html` — press **Replay the sequence**, then tick **slow motion ×0.25** (120/200/250/350, one curve).
3. `comps/07-states.html` — finish judgement: all states + empty state + three-plane material + decorative light pool.
4. `comps/02-mobile-triage-approvals.html` — press **Review**: sheet at 350 ms; **Approve once** now resolves to
   "Approved once — recorded" and disables the actions.
5. `comps/01-mobile-chat.html` — send a message (enter 250), tap the neural character (one-shot signal).
6. `comps/05-desktop-workspace.html` — tab swap (200) + Change state; then `06`, `03`, `04`.

## Evidence paths

`board/probes/checker-design-v2-r2/` — `FREEZE.sha256.copy`, `freeze-digest.txt`, `freeze-verify.txt`,
`content-counts.txt`, `tool-rerun.log`, `mutations-r2.log` (19 mutations: all previous slips caught; list of expected
findings), `checker-gaps.log` (C2/C3 probes + employee-assertion proof), `fix-verification.log` (D1–D4 in bytes),
`contrast-recompute.log` (independent floors + residual boundaries), `composer-and-lightpool.log` (C1), `audit.py` +
`audit.json` + `audit-print.log` (structure/labels/refs/colours/targets/states/unslop), `frozen-sha256-final.txt`,
`run-started.txt`, `run-finished.txt`. Throwaway copies: `/tmp/checker-design-v2-r2/{set,mut}`; frozen set read-only and
re-hashed unchanged. No image reads, no browser, no host input, $0.
