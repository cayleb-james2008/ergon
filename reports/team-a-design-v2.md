# Team A — Ergon design v2: owner-APPROVED Lumen direction — polish round applied, re-frozen

**STATUS: v2 DIRECTION APPROVED BY THE OWNER** (2026-11-02 04:55 ET: "Overall pretty impressed by the content… another additional polish round… other than that it's really not bad at all.") — **direction LOCKED; polish round applied and RE-FROZEN** (digest below) for the final grade presentation as live animated pages.** The comps are live, self-contained pages under
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
- **Floors are measured, not asserted** (both ambient washes composited, worst case = brightest text plane; retuned in the fix round): ink **9.21:1** (floor 7), muted **5.62:1** (4.5), subtle **6.14:1** on panel (4.5; base/panel only by rule), accent **5.01:1**, status on washes **4.66 / 4.93 / 4.77:1**, focus ring **6.47:1**, control boundary **3.28:1** (`lineBright` alpha 0.44; non-text floor 3), field boundary **3.30:1** (new `fieldLine` token alpha 0.46 - pack fieldLine rule).
- **Versioning:** `2.0.0-draft-1` direction proposal; **Midnight Slate v1.0.0 stays the shipped pack** until the owner's grade and the pack gate; shipped surfaces keep v1 until each is rebuilt (no mixed-theme claim); rollback is a token-set revert.
- **Risk register** for the checker: translucency legibility/glass tell, ambient field read as gradient, depth shadows vs the old "no drop shadows" rule (superseded explicitly), motion perf, contrast on the brightest plane, character as decoration, token-literal drift — each with its mitigation.

## 6. Verification (maker's evidence — a fresh checker re-runs it)

```bash
$ node build-tokens.mjs --check
TOKENS: CURRENT — tokens.css matches tokens.json

$ node check-comps.mjs            # exit 0
V2 COMPS CHECK: PASS (hardened after checker D1-D4)
  pages: 9 labelled comp pages (7 comps + index + motion storyboard)
  self-containment: 0 external refs, 0 remote assets, 0 webfonts, 0 images; every local href/src resolves
  tokens: tokens.css generated-current; 0 colour literals / raw colour keywords outside the token files
  custom properties: 73 defined, 0 unresolved; required surface/light/field/border/blur tokens emitted
  structure: tag balance clean on all 9 pages; 7-step space scale tokenised
  contrast (both ambient washes composited): ink 9.21:1 · muted 5.62:1 · subtle(panel) 6.14:1 · accent 5.01:1
  contrast: ok/warn/danger wash 4.66/4.93/4.77:1 · focus 6.47:1 · control boundary 3.28:1 · field boundary 3.30:1
  floors: reduced-motion + forced-colors + HC preview (inputs/messages/links/character included) · 2px accent focus ring · 44/56 targets enforced · empty state present
  motion: 120/200/250/350 ms, one ease-out curve; no transition-all, no loops/pulse, no uppercase tracking, no gradient text
  freeze: FREEZE.sha256 present and matching
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


## 6b. Fix round after the fresh checker's REJECT - D1-D4 (this is the re-freeze)

The checker (`board/reports/checker-design-v2.md`, evidence `board/probes/checker-design-v2/`) returned **REJECT** on the
first freeze: the set did not render its central claim. All named fixes are in, the self-check is hardened, and the set is
**re-frozen** for a RE-RUN of the same battery. No image reads, no browser, $0.

| Defect | What was wrong | Fix applied | Proof / new value |
|---|---|---|---|
| **D1 (blocker)** | `var(--surface-1/2/3)` used **35x**; `kebab()` did not split letter-to-digit, so the generator emitted `--surface1/2/3` and every translucent plane background was invalid/transparent - the owner's "panels are flat" failure mode | `kebab()` now splits letters-to-digits (`...replace(/([a-z])([0-9])/g,'$1-$2')...`) and `tokens.css` regenerated | `tokens.css` emits `--surface-1`, `--surface-2`, `--surface-3`, `--light-1`; **73 custom properties defined / 0 unresolved** (asserted); required-token assertion added |
| **D2a** | Control boundary recomputed with the page's **second** ambient wash = **2.97:1** < 3.0 | `lineBright` retuned from alpha 0.40 to **0.44**; the checker now composites **both** ambient washes | control boundary **3.28:1** (floor 3) |
| **D2b** | Input field border used `--line` = **1.55:1** vs the pack's `fieldLine` 3:1 rule | new **`fieldLine`** token (alpha 0.46); inputs use it; focus switches the border to accent | field boundary **3.30:1** (floor 3) |
| **D2c** | **7 controls at 36 px** (comp 05 x1, comp 06 x6) below the 44 DIP minimum | all inline 36 px overrides removed; buttons fall back to `var(--target)` = 44 | `min-height:36px` occurrences **0**; checker asserts any inline `min-height` is at least 44 |
| **D2d** | **no `class="employee"`** anywhere - the 56 DIP employee target was lost | `employee` restored on the employee surfaces: 01 (3), 02 (4), 03 (3), 04 (4), 06 (6), 07 (1 specimen) | 56 DIP via `button.employee` rule; per-comp assertion added |
| **D2e** | empty state missing (v1's 07 had one) | comp 07 gains an **empty state** specimen ("No tasks yet." + one next step) and labelled **reduced-motion / High Contrast floors rows** | asserted in `check-comps.mjs` |
| **D2f** | comp 02's "Approve once" was dead though its note says it resolves | the sheet is now the `data-approval` target; `wireApprovals` turns the primary control into the receipt when there is no pill | sheet primary becomes **"Approved once - recorded"**, all sheet actions disabled; `data-approval` + `data-approve` asserted |
| **D3** | self-check vacuous on: broken `</div>`, missing local script, `color: white`, `animation: ... infinite`, and never resolved custom properties | assertions added: **custom-property resolution**, **tag balance**, **local href/src resolution**, **raw colour keywords**, **infinite animations**, **target sizes + employee target**, **empty state**, **both-wash contrast + field boundary**, **freeze-manifest integrity**, and **non-vacuity self-tests** for the detectors | mutations proven to fail: broken `</div>` rc 1; missing script rc 1; `color: white` rc 1; `animation: float 2s infinite` rc 1; `var(--surface-9)` rc 1; stale `tokens.css` rc 1 (baseline PASS after restore) |
| **D4** | report said "16 files, 2,030 lines"; actual set is 17 files | counts corrected here and in the handoff: **17 content files, 2,159 lines + FREEZE.sha256 manifest** | this section |
| (checker nits) | floor table omitted field boundary / second wash; `blur.deep` and `light1` unused | floor table updated in `PACK-CHANGE-PROPOSAL.md`; `light-1` + `blur-deep` now used by a labelled decorative **light pool** in comp 07 (no text on it - the floors do not allow text there: muted 4.03 / accent 3.59) | visible in 07 |

**Rendered-look limit (named, not smoothed):** the checker's headless computed-style probe hung and was killed; this lane
cannot run a browser either (no image reads in DSH). D1 was a **static certainty** (now fixed and asserted); the page's
rendered appearance still needs an **image-permitted visual seat** before the owner sees it - that is a separate step and is
not claimed here.

### First re-freeze - `FREEZE.sha256` (17 content files; the manifest excludes itself) — **superseded by section 6c**

| File | sha256 |
|---|---|
| `index.html` | `6ff011e5355b7b4d6358ec00025e5e743ef5e2f39815971827493f0d31e5ae39` |
| `motion-storyboard.html` | `30440f4dccd27836561d6eb24908acb759b230998effd3189e66ad46eca1a086` |
| `comps/01-mobile-chat.html` | `66c5edbe28dda5e6d95467a99c35564a3ea84af007a43ffabf1cda912b2c001c` |
| `comps/02-mobile-triage-approvals.html` | `150b070a1b8281063af0564fac12c2ee41c16b4ee0f618c176b5f9aab0ce020b` |
| `comps/03-mobile-tasks.html` | `2a7205ce40f37b8ea92edc519c4eb2b31e2e1167041a04d8ed029f03928f187c` |
| `comps/04-mobile-make.html` | `9f0c0126dd889da5fb9bb46512c06176e47ca8d9357436540ad4d98a796535d7` |
| `comps/05-desktop-workspace.html` | `9dd46025443c4c9ec9af4d0eb6caeafa0cbfcad51e1dbd7141dd9d6e7599ed3a` |
| `comps/06-character-sheet.html` | `4256ac66d6610b63d24aaba510ccafad101f5af9d79648db6dd35fa1f39c5d4b` |
| `comps/07-states.html` | `73c286cb3be54bfde2c05ada95cefb0c761e8364e4c855482b17d5b5f1f86797` |
| `tokens.json` | `666a3707213c9bbe3fc4ada5d51470ff0b3a2d5a08cff95457fb1b53cc013766` |
| `tokens.css` | `c71496d670832630066efad14b7136cc1314174d2d7686fb1da3c9c9649d719e` |
| `build-tokens.mjs` | `14d09c792823d67af71ca2a2c8002d8ed5796d08a203197d8265e362782d8b06` |
| `comp-kit.css` | `bfc353972c21150d8913defbb123a2f0fa0da652cca0952746320fffc41f72a1` |
| `comp-motion.js` | `9bedd51a96d36248ae449e4dc544f8a5aff6319801075563617d9a903c3cef86` |
| `check-comps.mjs` | `97f79df2337a19896f08afaaccb654b0e5d15aaaa3eb6a669e9b94e2bfb67510` |
| `README.md` | `ca6064c3c40eef884de3cc6cc793e06d917135b59af7225bffc4af915ab73808` |
| `PACK-CHANGE-PROPOSAL.md` | `6d56a0e58bf212b92b9e100216b0cd203f90fdec989db525c6b108e75373befa` |

Verify: `cd board/probes/team-a-design-v2 && sha256sum -c FREEZE.sha256 && node check-comps.mjs`.


## 6c. Checker r2 - ACCEPT-WITH-NAMED-CONCERNS, then C1-C4 fixed (this is the current re-freeze)

Checker r2 (`board/reports/checker-design-v2-r2.md`) passed the direction: **ACCEPT-WITH-NAMED-CONCERNS**, 5 named
concerns, one must-fix. All four actionable items are fixed below; the fifth (C6, rendered look) is the owner's visual
grade. No image reads; the C1 proof is a **real-Chromium computed-style probe** run in the phone VM (Finish Line 2 rig).

| Concern | What was wrong | Fix applied | Proof |
|---|---|---|---|
| **C1 (must-fix)** | Message composer boundary still used `--line` = **1.55:1** (`.phone .composer` in `comp-kit.css` + comp 05 inline) while `input[type=text]` was correctly 3.30 | composer border -> **`var(--field-line)`** in both places (kit + comp 05 inline); assertion added so it cannot regress | **Real Chromium (phone VM)**: computed border token = `rgba(206, 228, 255, 0.46)` = `--field-line`; composited stack `rgba(37,48,58,1)`; **computed contrast 3.46:1 >= 3** (`c1-computed-proof.json`) |
| **C2** | Freeze integrity was conditional: deleting `FREEZE.sha256` or adding an unlisted file still PASSed | self-check now **requires the manifest**, verifies every hash, and asserts the directory contains **exactly** the listed 17 files | mutations: delete manifest -> **FAIL**; add unlisted `EXTRA.txt` -> **FAIL**; clean set -> PASS |
| **C3** | Raw-colour-keyword detector missed a **first-declaration** inline style (`style="color: white"`) | detector now scans each `style=""` value with a synthetic declaration separator plus the stylesheet rules; self-tests added for both hit and clean cases | mutation `<h1 style="color: white">` -> **FAIL** ("raw colour keyword in a style context"); clean inline -> no finding |
| **C4** | Native checkboxes (comp 02 remember, storyboard slow-mo) kept the UA focus ring | `input[type="checkbox"]` now carries `accent-color: var(--accent)` and `:focus-visible { outline: 2px accent ring + 2px gap }`; assertion added | real-Chromium probe on comp 02: checkbox **focus-visible = true**, outline colour `rgb(141, 180, 255)` (= `--accent`), accent-colour same; raw computed width recorded as 1.90476 CSS px under the phone viewport scale (the 2 px token ring scaled by the mobile viewport) |
| **C5 (note)** | latent unused `.pill.accent` (4.20 on wash) | **removed** from the kit; assertion prevents reintroduction | `check-comps.mjs` asserts `.pill.accent` is absent |
| **C6 (note)** | rendered look unverified by the checker; visual seat = the owner | **partially addressed tonight**: real-Chromium computed-style probe in the phone VM (tokens resolve, planes non-transparent + blur, composer 3.46:1, motion runs and settles to **0 running / 0 infinite**, JS interactions work). Pixel-level visual judgement is still the owner's grade, not claimed here. | `board/reports/team-a-vm-end-to-end.md` §2/§7; `board/probes/team-a-vm-e2e/c1-computed-proof.json`, `settle-proof.json`, `p2-proof.json` |

**Verification on the final set:** `node check-comps.mjs` -> PASS (all D1-D4 and C1-C5 assertions); `sha256sum -c FREEZE.sha256`
-> all OK; `node build-tokens.mjs --check` -> CURRENT.

### Freeze after the C1-C4 round — `FREEZE.sha256` (17 content files, 2,191 lines; the manifest excludes itself) — **superseded by section 6d**

| File | sha256 |
|---|---|
| `index.html` | `6ff011e5355b7b4d6358ec00025e5e743ef5e2f39815971827493f0d31e5ae39` |
| `motion-storyboard.html` | `30440f4dccd27836561d6eb24908acb759b230998effd3189e66ad46eca1a086` |
| `comps/01-mobile-chat.html` | `66c5edbe28dda5e6d95467a99c35564a3ea84af007a43ffabf1cda912b2c001c` |
| `comps/02-mobile-triage-approvals.html` | `150b070a1b8281063af0564fac12c2ee41c16b4ee0f618c176b5f9aab0ce020b` |
| `comps/03-mobile-tasks.html` | `2a7205ce40f37b8ea92edc519c4eb2b31e2e1167041a04d8ed029f03928f187c` |
| `comps/04-mobile-make.html` | `9f0c0126dd889da5fb9bb46512c06176e47ca8d9357436540ad4d98a796535d7` |
| `comps/05-desktop-workspace.html` | `6a9ad11d780ae1ef2122eafea5d8fd8ca154922ba9678e26a16f5d85526488d0` |
| `comps/06-character-sheet.html` | `4256ac66d6610b63d24aaba510ccafad101f5af9d79648db6dd35fa1f39c5d4b` |
| `comps/07-states.html` | `73c286cb3be54bfde2c05ada95cefb0c761e8364e4c855482b17d5b5f1f86797` |
| `tokens.json` | `666a3707213c9bbe3fc4ada5d51470ff0b3a2d5a08cff95457fb1b53cc013766` |
| `tokens.css` | `c71496d670832630066efad14b7136cc1314174d2d7686fb1da3c9c9649d719e` |
| `build-tokens.mjs` | `14d09c792823d67af71ca2a2c8002d8ed5796d08a203197d8265e362782d8b06` |
| `comp-kit.css` | `31fdef879c23c963d5492f095485a5a037ac4087fdc0420804e5896d7c9bc67b` |
| `comp-motion.js` | `9bedd51a96d36248ae449e4dc544f8a5aff6319801075563617d9a903c3cef86` |
| `check-comps.mjs` | `1b78ce90db6072b37279402590ee615528b85a3fa2f3b1131f8edfae4a25dc75` |
| `README.md` | `ca6064c3c40eef884de3cc6cc793e06d917135b59af7225bffc4af915ab73808` |
| `PACK-CHANGE-PROPOSAL.md` | `6d56a0e58bf212b92b9e100216b0cd203f90fdec989db525c6b108e75373befa` |

Manifest digest: **`ea68e398063d6cb1007ae1935f552f6981d663999018be5b3e30f8b83fdbd6c3`** (`sha256sum FREEZE.sha256`). Verify:
`cd board/probes/team-a-design-v2 && sha256sum -c FREEZE.sha256 && node check-comps.mjs`.


## 6d. Owner grade + polish round (2026-11-02 04:55 ET) — direction LOCKED, finish pass applied

**Owner grade (verbatim):** "Overall pretty impressed by the content. I think it can use another additional polish round.
With everything I've said in mind. But other than that it's really not bad at all." → **PASS on direction and content;
one polish round ordered.** Direction locked: translucent Lumen vein, technical-editorial restraint, neural-net characters —
no further design-direction gate. Outcome: this is a **finish pass, not a redesign**; the same accepted screens, structure
and model.

### What the polish pass changed (finish-only)

| Area | Change | Proof |
|---|---|---|
| **Finished edges** | new token **`--depth-shade`** (`inset 0 -1px 0 rgba(2,3,5,0.42)`) added to the plane system: every translucent plane now carries lift + top rim + bottom shade | real-Chromium probe: **8/8 checked surfaces carry an inset rim**; computed card shadow = `0 10px 30px rgba(2,3,5,.5), inset 0 1px 0 rgba(255,255,255,.07), inset 0 -1px 0 rgba(2,3,5,.42)` |
| **Depth consistency** | composer, tab bar, banner, sidebar and sheet now carry their own depth/rim (no flat chrome among layers); selected sidebar row steps to `surface-3` + `line-strong` | computed styles above; kit depth uses **9** (`--depth-shade`) across the plane system |
| **Motion polish** | tab indicator now cross-fades over **200 ms** (`--motion-swap`) instead of hard-swapping; character signal transitions; approval-resolve state styled (`[data-resolved]`, ok edge); all still one-shot, no loops | probe: selected tab ::before = opacity **1**, unselected **0**, transition `opacity / 0.2s`; after settle **0 running / 0 infinite** |
| **Interaction completeness** | focusable, labelled character control (`role=button`, `tabindex=0`, Enter/Space activates, visible focus ring); sidebar row hover; link focus ring; disabled/read-only field states; focus ring kept visible at the phone frame edge | uiautomator/a11y text now exposes "Marlow — agent character; activate to signal"; computed outline accent; no removed outlines (asserted) |
| **Detail finish** | tabular numerals for numeric readouts; display leading; nowrap pills; token-driven selection colour and thin scrollbars | probe: `font-variant-numeric: tabular-nums` on `.kv`; selection/scrollbar rules asserted |
| **Self-check hardening** | 13 polish assertions added (depth-shade coverage, composer depth, tabbar rim, selection/scrollbars, character focusability, disabled/read-only fields, sidebar hover, tabular numerals, resolved state, no `outline:none`) | `check-comps.mjs` PASS with all assertions |

**Floors unchanged and re-measured (both ambient washes):** ink **9.21:1** · muted **5.62:1** · subtle **6.14:1** · accent
**5.01:1** · status **4.66/4.93/4.77:1** · focus **6.47:1** · control boundary **3.28:1** · field boundary **3.30:1**;
composer boundary re-proved in real Chromium at **3.46:1**; reduced-motion / forced-colors / HC preview and 44/56 targets
intact. Unslop: clean (no gradient text/buttons, glows, shadows, emoji icons, 700+ weights, uppercase tracked labels,
`transition-all`, loops).

**Live-page evidence (no images):** the final set was served read-only and rendered in the **phone-VM Chrome (real
Chromium)**, then probed at text/computed-style level: page rendered (`DESIGN COMP — NOT A BUILT SURFACE` present), tokens
resolve (`--depth-shade` etc.), translucent planes + `blur(14px)`, tab swap works, approval resolves
("Approval needed" → "Approved — one action, recorded"), motion settles to **0 running / 0 infinite**. Evidence:
`board/probes/team-a-design-v2-polish-evidence/` (`polish-proof.json`, `polish-proof-05.json`, `indicator-proof.json`,
`teardown.txt`). The set is ready for **rendered presentation in the owner's browser**; the fresh checker re-runs the same
battery on the final freeze before/with presentation.

### Current freeze (polish round) — `FREEZE.sha256` (17 content files, 2,235 lines)

| File | sha256 |
|---|---|
| `index.html` | `0b7a26c6fe99eaeaa9e8c3e6655ebbdd8acdb4675add2af04f35b4e652edef83` |
| `motion-storyboard.html` | `30440f4dccd27836561d6eb24908acb759b230998effd3189e66ad46eca1a086` |
| `comps/01-mobile-chat.html` | `66c5edbe28dda5e6d95467a99c35564a3ea84af007a43ffabf1cda912b2c001c` |
| `comps/02-mobile-triage-approvals.html` | `150b070a1b8281063af0564fac12c2ee41c16b4ee0f618c176b5f9aab0ce020b` |
| `comps/03-mobile-tasks.html` | `2a7205ce40f37b8ea92edc519c4eb2b31e2e1167041a04d8ed029f03928f187c` |
| `comps/04-mobile-make.html` | `9f0c0126dd889da5fb9bb46512c06176e47ca8d9357436540ad4d98a796535d7` |
| `comps/05-desktop-workspace.html` | `6a9ad11d780ae1ef2122eafea5d8fd8ca154922ba9678e26a16f5d85526488d0` |
| `comps/06-character-sheet.html` | `4256ac66d6610b63d24aaba510ccafad101f5af9d79648db6dd35fa1f39c5d4b` |
| `comps/07-states.html` | `73c286cb3be54bfde2c05ada95cefb0c761e8364e4c855482b17d5b5f1f86797` |
| `tokens.json` | `23be162b1282c34f95cca927aa3a7a43fe31aa22c3c1c353ddd487a1eea90616` |
| `tokens.css` | `19af9c3cb51f5cd00315c356139e785a9a082c3449ab9c392021252b9e9873f6` |
| `build-tokens.mjs` | `14d09c792823d67af71ca2a2c8002d8ed5796d08a203197d8265e362782d8b06` |
| `comp-kit.css` | `74081b24faec166b51633b1dbd71f8859de09b21f18ec2fe47db67dc50eb752a` |
| `comp-motion.js` | `280b0490e11e5363a23f27ba78c18fa57ddbf2ec03f3bec66cfd78162fff4864` |
| `check-comps.mjs` | `47234b4b3896ad8923790ac9fc365df6f8e995b8e22fc5809c44bd3496f35321` |
| `README.md` | `ca6064c3c40eef884de3cc6cc793e06d917135b59af7225bffc4af915ab73808` |
| `PACK-CHANGE-PROPOSAL.md` | `6d56a0e58bf212b92b9e100216b0cd203f90fdec989db525c6b108e75373befa` |

**Manifest digest: `222364fd85e34079f4f874c11111e253508442670633834863d350433b5323de`** (`sha256sum FREEZE.sha256`). Verify:
`cd board/probes/team-a-design-v2 && sha256sum -c FREEZE.sha256 && node check-comps.mjs`.

## 7. Checker handoff (Lead preps/spawns; maker never checker)

- **Frozen set (polish round, post-owner-grade):** `board/probes/team-a-design-v2/` — **17 content files, 2,235 lines**, plus `FREEZE.sha256` (18 files in the directory; manifest excludes itself; digest **`222364fd85e34079f4f874c11111e253508442670633834863d350433b5323de`**). Every hash is listed in section 6d and asserted by `check-comps.mjs`. Do not edit after this report; any change is a new freeze.
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
| `board/probes/team-a-design-v2/FREEZE.sha256` | re-freeze manifest of the 17 content files (asserted by the self-check) |
