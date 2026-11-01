# Ergon front-end design pass — static comps (FOR GRADING, NOT BUILT SURFACES)

**Status: design comps for Nox + the owner to grade. No code, no plugin rows, no package files, no build.** These are
static HTML/CSS comps on the Ergon **Midnight Slate pack v1.0.0** (merged in `main` at `32b0e02`), taken from the
front-end design brief (§8 of `board/reports/team-a-ergon-frontend-design-brief.md`) and the three Nox design rulings
(`20261031T170100Z` Open Duet merge plan, `170800Z` desktop model, `171200Z` character/motion scope). Builds stay behind
the G1 rehearsal gate and the C1–C6 / W1 package rulings.

## What is here

| File | Comp |
|---|---|
| `index.html` | Cover + grading checklist + links to every comp. |
| `comps/01-mobile-chat.html` | Mobile Chat: morning brief, one-tap turns, approval pinned above the composer, character + live-state in the header, three-tab bar. |
| `comps/02-mobile-triage-approvals.html` | Mobile Triage + Approvals sheet: needs-you vs can-wait, Tier A vs Tier B wording (no "Always allow" on Tier B), one accent action. |
| `comps/03-mobile-tasks.html` | Mobile Tasks: task cards with working / waiting-on-you / done states, "your agent" as the owner. |
| `comps/04-mobile-make.html` | Mobile Make: artefact cards, open/request-a-change, version + state. |
| `comps/05-desktop-workspace.html` | Desktop workspace: character pinned at the top of the sidebar with state disc, Chat / Tasks / Apps / Live nav, thread, live-view rail. |
| `comps/06-character-sheet.html` | Character sheet: three example shapes × colours, the three state discs, name field, small-size legibility at 44/56 px, customisation surface (shape + colour + name only). |
| `comps/07-states.html` | Empty / loading / error / offline states for both faces, plain words + one next step each. |
| `motion-storyboard.html` | Motion storyboard: tap 120 / swap 200 / enter 250 / sheet 350 ms, one ease-out curve, applied examples, forbiddens, reduced motion + High Contrast. |
| `tokens.css` | Generated from the pack's `design/tokens.json` by `build-tokens.mjs` (single source; no hand-edited colours). |
| `comp-kit.css` | Shared comp layout kit (flat Midnight Slate components; tokens only). |
| `check-comps.mjs` | Run: verifies the seven comps + storyboard + index, the grading banner in every comp, no external assets/URLs/scripts, and the generated tokens match the pack. |
| `build-tokens.mjs` | Regenerates `tokens.css` from a pack `design/tokens.json` path. |

## Fences held

- **No external assets:** no CDN, no webfonts (system stack from the tokens), no images; all visuals are CSS or inline SVG.
- **Tokens only:** every colour/type/space/radius/target comes from `tokens.css`, generated from the pack's
  `design/tokens.json` (the script and `check-comps.mjs` enforce it).
- **No Glitch Cat Club content:** nothing copied from that source repo; method only.
- **Not a built surface:** comps carry "DESIGN COMP — NOT A BUILT SURFACE" in every file; they are not hooked to any
  plugin, package or fleet surface, and do not claim runtime behaviour.
- **$0**, no purchases, no sends, no credentials.

## Run / view

```bash
# regenerate tokens.css from a pack checkout, then verify the pack
node board/probes/team-a-design-comps/build-tokens.mjs --tokens /path/to/ergon/design/tokens.json
node board/probes/team-a-design-comps/check-comps.mjs --tokens /path/to/ergon/design/tokens.json
# open index.html in any browser (no server, no network)
```

Pixel rendering and interaction are **not** claimed from source: the pack gate (unslop + all-states + screenshot compare +
fresh checker) applies when these comps become built surfaces, behind G1 / C1–C6 / W1.
