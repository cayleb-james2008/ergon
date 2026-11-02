# Team A — Ergon design v2: owner-directed translucent pass (comps built, frozen for the checker)

**STATUS: v2 DIRECTION PASS BUILT AND FROZEN FOR THE PACK-CONFORMANCE + UNSLOP CHECKER. NOT A FINISHED FRONT END, NOT A
BUILT SURFACE, NOT A PACK RELEASE.** The comps are live, self-contained pages under
`board/probes/team-a-design-v2/` (grader index: `index.html`), with the pack-change proposal filed alongside them as
`PACK-CHANGE-PROPOSAL.md` and summarised here. Nothing was presented to the owner by this lane — the standing rule is
checker first, then the owner sees it rendered in his browser. **Spend $0**; no purchases; no image reads/screenshots; no
host input; no repo writes (all artefacts are board probes + this report).

Written 2026-11-02 ~00:58Z by `ergon-lead` (Team A), on the owner grade + Nox direction (`20261102T005200Z`).

## 1. The owner's grade, and what this pass does about it

| Grade | Verbatim / substance | v2 response |
|---|---|---|
| **ACCEPTED** | Layout and structure — buttons/screens/functional skeleton; the functional model (Duet feel, one personal agent, Chat/Tasks/Make, approvals as cards, character pinned, desktop multi-task) | **Kept unchanged.** Every comp keeps the v1 information architecture and controls; no screen was re-scoped. |
| **FAILED** | "it still looks like AI slop… 100% could use a ton of work"; components "do not look finished at all"; "no animations, panels are flat, everything colour scheme" | **Finish rebuilt in the directed vein:** translucent layered surfaces with real depth, blur and fine borders; an ambient light identity; a neural-network character; and motion that actually runs in the browser instead of being described. |

v1 was an initial iteration; **no claim here that v2 is a finished front end**. If the direction passes, the finished front
end is built after that grade, inside the package work and behind its gates.

## 2. Direction interpretation

**Ergon Lumen** (working title): *light through surfaces.* The product's depth is not painted — it is built from a stack of
translucent planes (`surface-1 → surface-2 → surface-3`), each with rising blur (14/18/24 px), rising border weight
(`line → lineStrong → lineBright`) and a 1 px inset rim light, over a near-black base lit by two very low-alpha single-hue
ambient washes. Text and controls are the same disciplined Midnight Slate grammar (one accent forward action, tokens only,
44/56 targets, 2 px focus ring), so the accepted model does not move — only its material does. The character becomes a
little **neural network** (a node-and-connection mark), light customisation on structure/density, colour and name. Motion
is part of the presentation: **t120 / s200 / e250 / sh350 ms**, one ease-out curve, purpose only.

## 3. What I changed vs v1 — comp by comp, and why

| Comp | v1 | v2 (changed) | Why |
|---|---|---|---|
| **01 mobile chat** | flat cards, glyph avatar | translucent phone shell + cards; neural character with live state + tap signal; content reveals; send inserts a real message (enter 250); tab swap (200) | The first screen the owner reads; finish and motion both visible immediately. |
| **02 triage + approvals** | flat sheet, static | sheet is the depth showpiece (brightest plane, 24 px blur, scrim, 350 ms); Tier A vs Tier B explicit; **scoped remember wording** "Remember for Oakridge move for sending estimates to Marcus" (one subject, one action, revocable, recorded); Tier B states it never offers remembering | Nox RUN-2 ruling §4 applied; the sheet is where "finished" is judged hardest. |
| **03 mobile tasks** | flat list | layered task planes (working brighter than done), word+colour states, staggered reveal | Task surface should read as stacked cards with state, not a list of boxes. |
| **04 mobile make** | artefact cards | artefact **stack** (no carousel), depth hierarchy, staggered reveal, request-a-change in place | **Deliberate judgement:** the parts-bin carousel mechanics would import an autoplay/gallery tell and are frozen with the parts-bin gate; the accepted model only needs artefact cards. Noted as a declined pattern. |
| **05 desktop workspace** | flat 3-column | three distinct translucent planes (sidebar / workspace / live view), pinned neural character, live-state readout, tab swap + send + approval + state-change motion | The "extra room goes to tasks, apps, live view" model kept; the finish carries the depth. |
| **06 character sheet** | blob-ish avatar system | neural net: deterministic seed from the employee's name; density 6/10/14; one hue token; name; states idle/working/needs-you with a **single** directional signal (no loop) | Owner direction #3 verbatim; also replaces the "shape + colour + name" language with a mark that is Ergon's own. |
| **07 states + depth** | states page | **repurposed** into states **and material study**: default/hover/focus/pressed/disabled/loading/error controls, fields, pills, character states, and the three-plane material with token names + floors | v1 failed on finish, so the finish is made inspectable: every state, and the material itself, on one page. |
| **motion storyboard** | static figures | live: tap/swap/enter/sheet play buttons, replay sequence, slow-motion ×0.25 grading aid, floors preview | "No animations" was a named failure; the storyboard now runs. |
| **index (new)** | — | **grader index**: owner-grade mapping, checklist, links, Floors preview, Replay motion, honest notes | The direction pass needs a front door for grading. |

**The set:** seven comps + live storyboard + index (I judge this the right set for a direction pass — same accepted screens,
plus the material/state page and the grader index; nothing new in product scope).

## 4. Taste cues taken from primeintellect.ai — in words (studied at HTML/CSS level only, no image reads)

- **Near-black neutral ladder, not blue-grey slate:** their surfaces step through near-blacks; ours keeps a cooler base with translucent white planes over it instead of flat fills.
- **Hairlines, not heavy borders:** 1 px lines at very low white alpha (their CSS: `#ffffff0f`-class overlays; blur 6/12/24 px). Ours: a three-weight line ladder where only interactive boundaries step up.
- **Translucency with restraint:** low-alpha white overlays + `backdrop-filter` blur are the material; ours uses 4/7/10% plane alphas and 14/18/24 px blur — same idea, calmer scale.
- **Editorial type discipline:** an editorial sans + a mono companion, a fine size range, **negative tracking on display** (−0.02…−0.045em), tight heading leading. Ours: system sans + mono (no webfonts allowed), same display tracking; **their positive tracked uppercase micro-labels are deliberately NOT taken** — the unslop gate bans uppercase tracked labels.
- **Radius mix:** fine 2–6 px controls against 12–24 px planes — not one uniform pill world. Ours: 6/8/12/14 px + pills.
- **Measured motion:** their CSS transitions cluster at 120–300 ms on background/border/colour — our 120/200/250/350 budget with one curve fits the same restraint.
- **Technical-editorial composition:** numbered sections, mono code blocks, metric callouts, one accent per view, generous space — kept as annotation/section rhythm.
- **Not taken:** layout, copy, marks, logos, colours, fonts, icons, or any asset. No network dependency introduced.

## 5. Pack-change proposal (filed with the comps)

`board/probes/team-a-design-v2/PACK-CHANGE-PROPOSAL.md` — **Midnight Slate as Ergon's front-end theme is superseded by the
owner's direction**; the proposal is the pack-level change request, not a pack change:

- **Kept:** one accent forward action per view; tokens-only; 44/56 targets; 2 px focus ring + 2 px gap; type scale; pills carry words; plain-language rules; unslop gate; reduced-motion + High Contrast; contrast floors; pack anatomy.
- **New:** translucent surface planes + `blur` tokens (14/18/24/32), `depth` tokens (lift/rim/well), a three-weight border ladder, an ambient light field, **motion tokens promoted into the pack** (120/200/250/350 + one curve), neural-character tokens (density, hue).
- **Floors are measured, not asserted** (composited translucent stack, worst case = brightest text plane): ink **10.03:1** (floor 7), muted **6.13:1** (4.5), subtle **6.64:1** on panel (4.5; base/panel only by rule), accent **5.46:1**, status on washes **5.08 / 5.37 / 5.11:1**, focus ring **7.05:1**, control boundary **3.08:1** (non-text floor 3).
- **Versioning:** `2.0.0-draft-1` direction proposal; **Midnight Slate v1.0.0 stays the shipped pack** until the owner's grade and the pack gate; shipped surfaces keep v1 until each is rebuilt (no mixed-theme claim); rollback is a token-set revert.
- **Risk register** for the checker: translucency legibility/glass tell, ambient field read as gradient, depth shadows vs the old "no drop shadows" rule (superseded explicitly), motion perf, contrast on the brightest plane, character as decoration, token-literal drift — each with its mitigation.

## 6. Verification (maker's evidence — a fresh checker re-runs it)

```bash
$ node build-tokens.mjs --check
TOKENS: CURRENT — tokens.css matches tokens.json

$ node check-comps.mjs            # exit 0
V2 COMPS CHECK: PASS
  pages: 9 labelled comp pages (7 comps + index + motion storyboard)
  self-containment: 0 external refs, 0 remote assets, 0 webfonts, 0 images
  tokens: tokens.css generated-current from tokens.json; 0 colour literals outside the token files
  contrast: ink 10.03:1 · muted 6.13:1 · subtle(panel) 6.64:1 · accent 5.46:1 · ok/warn/danger wash 5.08/5.37/5.11:1 · focus 7.05:1 · border 3.08:1
  floors: reduced-motion + forced-colors + High Contrast preview present; 2px accent focus ring with 2px gap
  motion: 120/200/250/350 ms, one ease-out curve; no transition-all, no loops/pulse, no uppercase tracking, no gradient text
```

- **Structure:** `div` balance and `aria-controls`/sheet-target ids checked on all 9 pages (no mismatches, no missing ids); every page carries the banner **"DESIGN COMP — NOT A BUILT SURFACE"** and the "DESIGN COMPS FOR GRADING, NOT BUILT SURFACES" kit header.
- **JS syntax:** `node --check comp-motion.js` OK; the storyboard's inline script extracted and `node --check`ed OK.
- **Approvals wording:** no "always allow" wording anywhere in the comps; comp 02 asserts "Remember for \<subject\> for \<action\>", "one subject, one action", and "Tier B approvals never offer remembering" on the surface (self-check asserts all three).
- **One accent forward action (accounting):** 01 / 03 / 04 / 05 each carry exactly **one** accent (`primary`) action; 06 carries none;
  02 carries one on the triage page and one inside the approvals sheet — the sheet is its own modal surface (the page is dimmed behind the scrim),
  so this is one accent per visible surface, not two on one screen; **07 is a states specimen** where several primary buttons are the point
  (it exists to expose every state, not to be a product screen). The kit keeps Back/Close/Cancel/ghost actions non-accent throughout.
- **Honest limit:** I did not view rendered pages (image reads/screenshots are the DSH crash class). The rendering grade is the
  checker's and the owner's; my evidence is structural, numeric and syntax-level. Motion is JS-driven and was validated for
  syntax/consumption of tokens, not visually.

## 7. Checker handoff (Lead preps/spawns; maker never checker)

- **Frozen set:** `board/probes/team-a-design-v2/` (16 files, 2,030 lines) — do not edit after this report; any change is a new freeze.
- **Reproduce:** `cd board/probes/team-a-design-v2 && node build-tokens.mjs --check && node check-comps.mjs`; open
  `index.html` and the seven comps from the folder (file:// is enough; no network, no server).
- **Judge:** pack-conformance (tokens discipline, floors, anatomy, versioning per PACK-CHANGE-PROPOSAL §3/§4/§8) and
  **unslop** (no gradient text, glows, uppercase tracked labels, 800 weights, emoji icons, sparkle badges, `transition-all`,
  loops; the ambient field is a single-hue low-alpha wash, not a ramp; blur is bounded) — plus whether the material reads as
  "light through surfaces" rather than glass/slop.
- **After ACCEPT only:** present to the owner **rendered in his browser** (live motion + Floors preview per standing rule) and
  take graded deltas only. No comp changes before his words.

## 8. Frozen parts-bin gate

The parts-bin curation gate (`board/reports/team-a-parts-bin-curation-gate.md`) is **FROZEN, not discarded**. Its fences
were written against the flat Midnight Slate identity; they are re-validated against this translucent identity **only after
v2 passes**. One declined pattern is already recorded (`04` carousel mechanics: autoplay/gallery tell; the artefact stack
serves the accepted model). The S1–S12 admissions remain unstarted.

## 9. Milestones (ETA-free — sequence, not dates)

1. **Freeze (done):** comps + pack-change proposal + grader index filed; self-check PASS quoted.
2. **Checker:** Lead spawns the pack-conformance + unslop checker on this frozen set; verdict recorded.
3. **Owner presentation:** on ACCEPT, the live pages are opened in the owner's browser (motion running; floors toggles
   available) — rendered, not stills.
4. **Owner grade:** direction pass verdict, graded deltas only.
5. **If the direction passes:** the pack-change proposal enters the pack gate (docs-only pack update + fresh checker), and the
   finished front end is planned as a build behind G1 / C1–C6 / W1. **If it fails:** v1 stands; next iteration starts from
   the owner's deltas, not from our preferences.
6. **Then:** parts-bin fences re-validated against the accepted identity; component-layer admission resumes.

## 10. Fences held

$0 spend; no purchases; no vendoring; no repo writes (probes + this report only); no image reads/screenshots; no host input;
no credentials; no sends/publication; no built surface, no plugin rows, no package files; maker never checker. The comps are
labelled **DESIGN COMP — NOT A BUILT SURFACE** on every page and in the kit/generator headers.

## 11. Artefacts

| Path | What |
|---|---|
| `board/reports/team-a-design-v2.md` | this report |
| `board/probes/team-a-design-v2/index.html` | **grader index** (owner-grade mapping, checklist, links, floors/replay) |
| `board/probes/team-a-design-v2/comps/01…07` | the seven reworked comps |
| `board/probes/team-a-design-v2/motion-storyboard.html` | live motion storyboard |
| `board/probes/team-a-design-v2/tokens.json` + `tokens.css` + `build-tokens.mjs` | the proposed v2 token set and generator |
| `board/probes/team-a-design-v2/comp-kit.css` + `comp-motion.js` | material kit + all live motion |
| `board/probes/team-a-design-v2/check-comps.mjs` | self-check (labels, self-containment, tokens, contrast, floors, motion tells, approvals wording) |
| `board/probes/team-a-design-v2/PACK-CHANGE-PROPOSAL.md` | the pack-change proposal (Midnight Slate supersession, new tokens, floors, risks, approval path) |
