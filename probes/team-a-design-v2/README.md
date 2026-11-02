# Ergon Lumen v2 — direction pass (design comps for the owner's grade)

**DESIGN COMPS — NOT BUILT SURFACES.** This folder is the v2 design pass: the owner-accepted layout and functional model,
rebuilt in the owner-directed **translucent** identity, presented as live pages with the micro-animation actually running.
Nothing here is product code, a plugin row, a package file, a pack release, or evidence that a built surface behaves this
way. Builds stay behind G1 / C1–C6 / W1; a finished front end is built only after a direction passes the owner's grade.

## Open it (offline, self-contained)

Open **`index.html`** in a browser (double-click is enough — no server, no network, no CDN, no webfonts, no images).
It links the seven comps and the live motion storyboard, and carries the grading checklist.

## Files

| File | What it is |
|---|---|
| `index.html` | **Grader index** — what the v1 grade said, what v2 changes, the checklist, links, Floors preview, Replay motion. |
| `tokens.json` | The proposed v2 token set (the **only** file where colour values are written). |
| `build-tokens.mjs` → `tokens.css` | Generates the CSS custom properties; `--check` proves `tokens.css` is current. |
| `comp-kit.css` | The material/component kit: translucent surface layers, borders, depth, type, controls, character, floors. |
| `comp-motion.js` | All live motion (Web Animations API, JS-driven): reveal, tab swap, sheet, send, approval, character signal, floors preview, replay. |
| `comps/01 … 07` | The seven comps (mobile chat / triage+approvals / tasks / make; desktop workspace; character; states+depth). |
| `motion-storyboard.html` | The four budgets live (tap 120 · swap 200 · enter 250 · sheet 350), one curve, replay + slow-motion grading aid. |
| `check-comps.mjs` | Self-check: labels, self-containment, token discipline, tokens-current, contrast floors, floors present, motion tells. |
| `PACK-CHANGE-PROPOSAL.md` | The pack-change proposal filed **with** the comps (Midnight Slate → Ergon Lumen direction). |

## What v2 is (direction)

**Ergon Lumen** — light through surfaces. Layered translucent planes with real depth, fine bright hairlines, an ambient
light field, and a character that is a little neural network (nodes + connections), not a blob. Depth comes from the
stack (surface-1 → 2 → 3, rising blur + border weight + rim light), not from flat fills and not from spectacle.
Motion is part of the presentation: 120/200/250/350 ms, one ease-out curve, purpose only, no loops.

## Floors (kept, not traded away)

Contrast floors are asserted from `tokens.json` by `check-comps.mjs` (ink ≥ 7:1, muted/status ≥ 4.5:1 on the brightest
text plane, focus ring ≥ 3:1 for non-text, control borders ≥ 3:1); focus is a 2 px accent ring with a 2 px gap everywhere;
reduced-motion and High Contrast paths are real (`prefers-reduced-motion`, `forced-colors`) with a labelled **Floors preview**
on the index for grading. Translucency does not buy its way out of any floor.

## Self-check

```bash
node build-tokens.mjs           # regenerate tokens.css from tokens.json
node build-tokens.mjs --check   # prove tokens.css is current
node check-comps.mjs            # labels, self-containment, tokens, contrast, floors, motion tells
```

The maker's run of `check-comps.mjs` passes; a fresh-context **pack-conformance + unslop checker** is the Lead's to spawn
before anything is presented as done. Maker never checker.

## Provenance / honesty

- Taste reference: **primeintellect.ai**, studied at HTML/CSS level only (fetched pages + stylesheets as text; **no
  screenshot, no image read**, per the DSH crash-class rule). Taste only — no layout, marks, copy or assets copied.
- System fonts only; no webfont files; no network requests from any page.
- The parts-bin curation gate is **FROZEN** (not discarded): its fences are re-validated against this translucent identity
  only after v2 passes the owner's grade.
