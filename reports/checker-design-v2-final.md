# CHECKER FINAL — Ergon design v2 freeze `ea68e398…` — verdict: **FINAL ACCEPT**

Fresh-context final spot re-verify of Team A's C1–C5 fix round. I built none of it (maker-never-checker). No image
reads/screenshots, no browser, no host input, no repo edits, **$0**. Everything verified from the frozen bytes and a
read-only copy `/tmp/checker-design-v2-final/set`; mutations on throwaway copies only. Presentation held (not presented).

## Freeze

- `sha256sum FREEZE.sha256` → **`ea68e398063d6cb1007ae1935f552f6981d663999018be5b3e30f8b83fdbd6c3`** (matches the Lead's
  digest); `sha256sum -c FREEZE.sha256` → **17/17 OK, rc 0**; **17 content files, 2,191 lines** (+ manifest); DSH copy
  byte-identical to the shared-workflows lanes copy. Frozen set re-hashed after the check: unchanged.
- **Delta vs my r2-verified freeze is exactly the claimed fix round:** 3 files changed, 14 unchanged, none added/removed —
  `check-comps.mjs` (`97f79df2…`→`1b78ce90…`, 244 lines), `comp-kit.css` (`bfc35397…`→`31fdef87…`, 238),
  `comps/05-desktop-workspace.html` (`9dd46025…`→`6a9ad11d…`, 104). The full diffs contain only the C1–C5 changes below;
  tokens, pages 01–04/06/07, index, storyboard, README, proposal are byte-unchanged.

## Spot-verification of the claimed fixes (bytes + tools)

| Claim | Result |
|---|---|
| **C1** composer boundary | **PASS.** `.phone .composer` (comp-kit.css:151) and comp 05's inline composer border now `1px solid var(--field-line)`; the checker has hard assertions for both. My independent recompute (both ambient washes, CSS layer order accent-over-cool-over-base): composer fill = **rgba(37,48,58)**; composited border (field-line α 0.46) = **rgba(115,131,149)**; boundary contrast = **3.4620:1 ≥ 3** (claimed 3.46 — matches). Generic `input[type=text]` field boundary stays 3.2965:1. |
| **C2** freeze required + exact set | **PASS.** Mutation re-proofs in a copy: deleting `FREEZE.sha256` → **FAIL** ("the set is not frozen"); adding an unlisted file → **FAIL** ("directory contents are not exactly the frozen set — extra: extra-unfrozen.txt"). The checker now walks the directory and compares it to the exact 17-file list. |
| **C3** inline first-declaration colour | **PASS.** `style="color: white"` as the first declaration → **FAIL** ("raw colour keyword in a style context"); the detector now scans each `style="…"` value with a synthetic `;` separator. |
| **C4** checkbox focus/accents | **PASS (static computed-style derivation).** `input[type="checkbox"] { accent-color: var(--accent) }` and `input[type="checkbox"]:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px }` in the kit, no later override; token `accent: #8DB4FF` → computed `rgb(141, 180, 255)`; both page checkboxes are `input type="checkbox"`. A live-Chromium computed check was not feasible here (the headless probe hangs in this environment and was killed earlier; no image reads), but custom-property substitution is deterministic. Removing the rule → checker **FAIL** ("native checkboxes must carry the 2px accent focus ring"). |
| **C5** `.pill.accent` removed | **PASS.** No `.pill.accent` rule remains; the checker guards against reintroduction (re-adding it → **FAIL**). |

Tools re-run from the read-only copy (20:37:45-05:00): `node build-tokens.mjs --check` → `TOKENS: CURRENT`, rc 0;
`node check-comps.mjs` → `V2 COMPS CHECK: PASS (hardened after checker D1–D4)` rc 0 (73 defined / 0 unresolved; floors
ink 9.21 · muted 5.62 · subtle 6.14 · accent 5.01 · status 4.66/4.93/4.77 · focus 6.47 · control 3.28 · field 3.30);
`node --check comp-motion.js` rc 0; `sha256sum -c` rc 0.

## No-regression sweep (independent audit)

9/9 pages labelled; tag balance clean; no duplicate ids; 35/35 local refs resolve; 0 unresolved custom properties;
rendered files carry only `transparent`/`currentColor` keyword colours (no literals); unslop clean (only the two
single-hue ambient radial washes, no uppercase/emoji/glow/text-shadow, weights 600, one cubic-bezier, no keyframes);
07 still covers default/hover/focus/pressed/disabled/loading/error/**empty**; employee counts and 44/56 targets unchanged
from the r2-verified state; freeze manifest integrity verified before and after.

## Residual (named, not blocking)

- **Rendered look** is still unverified: no image reads/browser in this seat. An image-permitted visual seat must confirm
  the material reads as "light through surfaces" (not milky glass), blur/depth consistency, ambient as a wash, mark
  legibility, motion polish, and the Floors preview. That is the only remaining gate before the owner's grade, and it is
  unchanged from r2.
- C4's computed-style claim is verified by deterministic token substitution, not by a live browser (environment limit).
  No other residual: C1–C3 and C5 are verified in bytes and by mutation; nothing outside the claimed delta changed.

## Evidence paths

`board/probes/checker-design-v2-final/` — `FREEZE.sha256.copy`, `freeze-digest.txt`, `freeze-verify.txt`,
`content-file-count.txt`, `content-line-count.txt`, `per-file-delta-vs-r2.log`, `tool-rerun.log`,
`c1-c5-verification.log` (C1 recompute + C4/C5 static derivation + assertions), `mutations-final.log` (7 re-proofs:
no-manifest, extra-file, inline colour, composer kit, composer inline, pill.accent, checkbox rule — all rc 1),
`audit.py` + `audit.json` + `audit-print.log`, `frozen-sha256-final.txt`, `run-started.txt`, `run-finished.txt`.
Throwaway copies: `/tmp/checker-design-v2-final/{set,mut}`. Frozen set read-only and re-hashed unchanged.
