# RULING REQUEST — Watchover plate density: A8 and A29 cannot both hold at 6 front cells

From: OPENBELL stream lead (model `opencode-go/deepseek-v4.1-flash`), 4 Nov 2026 ~05:00Z, for Nox / the owner.
Evidence: `workspace/openbell-watchover/design/watchover/renders/20261104T0620Z/` (run notes + `results.json`),
commits `b10291c` (code) and `61c3ebd` (renders). Read-only measurements; nothing is patched around this.

## The conflict, in numbers

The spec asks for four things at once:

| requirement | spec | measured consequence |
|---|---|---|
| 6 drawn front bodies on desktop | §4, F18 (RULED) | at 1440 the front cell is **224.7 px**; at 1920 it is **304.7 px** |
| a front cell ≥ 180 px | §5.3 | holds (224.7 ≥ 180), so the grid itself is compliant |
| a 48 px glyph on every plate | §6.1, §7 | with 20+14 px padding and a 12 px gap it leaves **130.7 px** of content at 1440, **210.7 px** at 1920 |
| front primary figure ≥ 28 px, state word ≥ 13 px | A29 (i) | A 28 px mono figure is **~16.8 px per glyph**: `-$101.25` measures 134.4 px (8 glyphs), so `$50,361.80` (10 glyphs) is **~168 px** |

Then add the two other columns every fact row carries: the label column (**54 px** measured, `equity`) and the age
column (**23 px** measured, `0 s old`), plus the tag run (`SYNTHETIC` 74 px, `MANUAL` 54 px, which now wraps).

- At **1440**: 54 + 168 + 23 = **245 px needed** against **130.7 px available** → 114 px short. The Account 1 and
  meme plates therefore widen the page: 1,485-1,540 px in a 1,440 viewport in four meme previews.
- At **1920**: 245 needed against 210.7 available → **34 px short**, which is exactly what `preview-memegone`
  measures (1,940/1,920, Account 1's facts table 33 px past its cell).

So the failure is not a CSS bug to patch: with the glyph beside the facts and a label column beside a 28 px figure,
the plate needs ~245 px of content, which the 6-cell grid does not provide at 1440 (and only barely at 1920).

## What the measurements rule out

| candidate fix | arithmetic | verdict |
|---|---|---|
| smaller figure at narrow widths | 24 px figure ≈ 144 px, still 54+144+23 = 221 > 130.7 at 1440 | violates A29, still fails |
| fewer front cells at 1440 (5) | 269 px cell → 175 px content, still < 245 | fails |
| hide the glyph below 1600 | frees 60 px → 190.7 at 1440, 270.7 at 1920 | 1440 still 54 px short |
| drop the age column (age inline) | saves ~35 px → 155.7 at 1440 | still short |
| fixed table layout + wrapping | already applied; the figure is atomic by design | no further gain |

## The three viable rulings (pick one; I will build the winner)

1. **Plate-lane redesign (recommended):** glyph goes in the plate's header row beside the label (it is identity, not a
   fact), and the fact rows lose the label column at ≤ some width (label above value, as the phone lane already does).
   Content becomes 190.7 px at 1440 and 270.7 px at 1920, which holds a 28 px `$50,361.80` plus its age. Cost: a
   visible layout change on every plate; ~1 day of work including the tests that assert row order.
2. **Relax A8 at 1440 to "plates may scroll horizontally inside the bay region, the page may not".** Cost: a
   scroll affordance on the plate row, and the owner grades a 1440 render with one plate partly off-screen.
3. **Relax A29's 28 px front figure to 24 px at widths under 1600.** Cost: the distance stand-in (A29 ii) then needs
   re-measuring; 24 px is 6 px at the 4x stand-in, still above the 5 px rule.

## Separate question — A29 (iii) coverage threshold

A29 (iii) asks that body pixels inside each front body's projected box cover **≥ 60 %** of the box. Measured on the
grey-tone render (every body `#3A3E45`), the *best* of the six bodies reaches **0.28**, and the render looks correct
(the bodies are upright, lit flatly, pads visible; see `a29/grey-1920.png` and the x4 downscale). An articulated body
inside its own bounding box is mostly empty space by construction — a humanoid's box includes the gaps between limbs
and head. The ring-contrast half of the same row passes; the tone-lift failure rule in A29 changes brightness, not
coverage, so it cannot rescue this row.

**Ask:** either the threshold is meant to be measured against the body's *silhouette* (which is 100 % by definition
and therefore not a test), or the intended number is lower (0.25-0.30 would pass every body and still catch a body
that failed to draw). I have not changed the threshold; the row reports FAIL with its measurement.

## Revision, 4 Nov 00:10 ET — after the fresh checker read the same renders

The independent checker (`~/fleet-scratch/overnight/checker-watchover-verdict-run1.md`) reproduced every number above
and then corrected two of my own statements. Both corrections make the ask sharper, not softer.

1. **It is not only the meme previews.** The **default** state at 1440 already has four primary figures cut mid-number
   (`$50,36`, `-$101.`, `Unknowr`), with five cards overflowing their own box by 44–112 px. My request framed 1440 as
   four meme previews widening the page; the truth is that the arithmetic in the table above applies to every state,
   and the meme previews are only where the damage also pushes the page width. The gate marked those rows PASS because
   its A7 row compares `scrollWidth` to `clientWidth` and cannot see occlusion.
2. **The 1920 shortfall is 20 px, not 34, and the offender is the meme plate's facts table**, not Account 1's figure
   (Account 1 overflows its own card by 32 px at 1920, but does not widen the page). The arithmetic conclusion is
   unchanged: ~245 px wanted, ~210.7 px available.

### The A29 (iii) question is bigger than coverage

On the grey-tone render, **11 of 11** bodies fail the 1.8:1 **mean contrast** clause (measured 1.35–1.51); only 6 of 11
fail the 60 % coverage clause (worst 0.28). Two consequences the earlier text did not state:

- The grey render paints **every body with one token** `#3A3E45` (1.43:1 against the slab). A29 (iii)'s failure rule
  lifts each body's T0/T1/T2 hexes — hexes that **do not appear in that image at all** — so up to three lifts change
  neither number.
- The third clause (three tonal bands as separate modes, ≥1.45:1 **between neighbours**) is unreachable with the
  approved 5.1 palette: adjacent T0/T1 and T1/T2 ratios measure 1.22–1.35 on every one of the seven body rows; only
  the non-adjacent T0/T2 reaches 1.55–1.80. Meeting it needs new hexes, i.e. a palette ruling, not a render change.

### Ask, restated as three separate rulings

1. **A29 (iii) measurement:** measure coverage against each body's own silhouette mask (current range 0.28–0.64), or
   keep the bounding box and lower the bar; and state that the tone-lift remedy applies to the *real-tone* histogram
   row only, not to the single-tone grey render.
2. **A29 (iii) tonal bands:** either replace "≥1.45:1 between neighbours" with the 5.1 rule's actual wording (T0 vs T2
   ≥ 1.45, which the palette does satisfy at 1.55–1.80), or authorise new T1 hexes.
3. **The 1440 plate lane:** unchanged from the three options above. Option 1 (plate-lane redesign) is the only one that
   keeps A8, A29 and 6 cells simultaneously; option 3 is the cheapest and trades the 28 px floor for a re-measured
   distance stand-in.

## Second addendum, 00:25 ET — the tonal-band clause is now measured, and it fails on the palette

The gate used to record "tonal bands" without asserting anything. It now evaluates the clause as written, two ways:
the T0/T1 and T1/T2 ratios come from the 5.1 identity hexes (the table is the source) and each band must actually be
painted inside the body's own box in the real-tone render (so a band that is never drawn cannot count as a mode).

Result on the current build, 11 front bodies:

| measurement | result |
|---|---|
| adjacent band ratio T0/T1 or T1/T2 ≥ 1.45:1 | **0 of 11 pass**; minimum **1.23:1** |
| every band painted inside its own box | 11 of 11 carry all three bands |
| T0 vs T2 (non-adjacent) | 1.55–1.80:1 — passes 1.45 comfortably |

So the arithmetic is unambiguous: the palette satisfies the clause for the *outer* pair and cannot satisfy it for the
*adjacent* pairs, on every body, by design (each step is about +4–5 % lightness). No render change can fix it; it needs
either new T1 hexes (a 5.1 ruling) or the wording to be the 5.1 rule's actual requirement (T0 vs T2 ≥ 1.45).

Both A29 (iii) failures now in the render set are this clause and the coverage/ring-contrast row — i.e. **the two
A29 (iii) sub-clauses that cannot be met by an implementation change.** Everything else in the gate passes.

## Third addendum, 00:55 ET — three sub-clauses, three different answers, and one of my own claims corrected

Re-measured independently against run `20261104T1115Z` (commit `d84a13a`, 413/415, the same two A29 iii rows).
Scripts, JSON and crops: `fleet-scratch/overnight/a29-arithmetic/`. Read-only; nothing in the worktree changed.

My 0-step column reproduces the gate's own numbers exactly (1.349–1.508 against the row's "worst 1.35"), which is why
the predictions below are arithmetic rather than a guess.

| sub-clause | floor | at 0 lift | reachable by an implementation change? |
|---|---|---|---|
| ring contrast vs the 8 px ring | ≥ 1.8:1 | 1.349–1.508, **0/11** | **YES — two steps of the spec's own remedy** take it to 11/11 (min **1.836**); three steps give min **2.138** |
| adjacent bands T0/T1, T1/T2 | ≥ 1.45:1 | min **1.227**, 0/11 | **NO — the remedy makes it worse** (min **1.212** after three steps); needs new hexes |
| coverage of the projected box | ≥ 60 % | 0.276–0.641, **6/11** fail | **NO as written**; 4/11 still fail on a tight ink box |

### Correction 1 — the ring-contrast half is *not* unsatisfiable

`#3A3E45` on `#222529` is **1.4325:1** by arithmetic, so no render change can reach 1.8 with those two colours — that
part of my earlier note stands. What does not stand is the conclusion I drew from it. The spec's own failure rule lifts
body tones, and the grey-tone token **is** a body tone. Applied to it: one step `#434850` 1.672 (still short), two steps
`#4D525B` → **11/11 at 1.836**, three steps `#565C66` → **min 2.138**. Both stay inside A20 (saturation 4–5 %, far under
the `#D8DCE2` cap). The 8 px ring is bay pixels *outside* the box, so a change of body colour moves only the numerator:
this is exact arithmetic, and it is the only sub-clause the existing remedy can actually fix.

**Ask:** does the failure rule name the grey-tone token as well as the 5.1 hexes? If yes, three steps on the grey token
closes this row and A29 (iii) is down to the coverage clause alone.

### Correction 2 — "silhouette is 100 % by definition" was wrong

A silhouette *mask* is 100 %, which is why that reading is vacuous. A **tight bounding box** around the same body is not:
measured on the same frame it is **0.276–0.641**, and **4 of 11 bodies still fail at 60 %** (grok 0.276, meme 0.342,
orb15 0.378, account-1 0.393). So the useful question is not "mask or box"; it is which number separates a drawn body
from a missing one. On this set **0.25 passes every body** and a plate with no body scores 0.00. `grok-box.png` shows
why: a thin disc on a narrow stem over a wide base fills about a quarter of its own projected bounds at a 3/4 camera,
and the spec forbids the one change that would fatten it (outline thickening).

### Correction 3 — the "band absent" count is sampling noise, not a defect

The current run records **5 of 11** bodies with a band absent from their box, not the 11/11 I reported earlier. It is
measured by exact-token match at 6/255 on a small shaded 3D body, so it flaps between runs (the gate says so and
deliberately does not threshold it). The **ratios** come from the 5.1 identity table and do not depend on pixels, so the
band clause fails on arithmetic alone.

### The three rulings, restated with the numbers

1. **Ring contrast:** authorise the existing remedy to lift the grey token too (three steps, `#565C66`, min 2.14) — or
   state that the clause is deliberately unreachable. This is the only one of the three that a render change can close.
2. **Coverage:** lower 0.60 to **0.25** (passes 11/11, min 0.276) or change the box definition in the spec, because
   0.60 cannot be met by a thin articulated body inside its own projected bounds at any camera in use.
3. **Adjacent bands:** keep the wording and authorise new T1 hexes, or take the 5.1 rule's actual requirement
   (T0 vs T2 ≥ 1.45, met at 1.55–1.80). The failure rule cannot help — it compresses the ratios toward 1.

## Third addendum, 00:46 ET — A8's viewport clause was never measured, and it fails

The gate compared `scrollWidth` to `clientWidth` and never asked whether the *plates* fit the viewport. It now does, at
1440×900 (A8's stated size), on every state:

| state | front row top (<=500) | plates bottom in a 900 px viewport |
|---|---|---|
| `default` | 496 (pass) | **1107 — 207 px below the fold** |
| `preview-warn` / `preview-crit` / `preview-breach` | 500 (pass) | **1159 — 259 px below the fold** |

A8 (§5.3, §412) says "all plates inside the viewport without scroll". They are not, and were not before this stream
touched the pack: the same measurement on `9f6b399`'s CSS put the shelf 236 px below the fold.

The arithmetic that makes it unavoidable with the spec as written: at 900 px the stage formula gives
`max(240, min(44vh = 396, 520, 500 − 48 − alerts))` = **396 px**. Header 48 + stage 396 + front row (~322) + 12 gap +
shelf (~277) = **1,055 px** before a single alert row. At 1920×1080 it fits (shelf bottom 1056 ≤ 1080); at 900 px high it
cannot. So A8 and §5.3's stage rule are in conflict for a 900 px-tall window, and no plate-lane change reaches it.

**Ask:** either the stage height yields to the plate rows at short windows (`min(44vh, 900 − 48 − alerts − front − 12 −
shelf)`), or A8's clause is restated as "the front row is fully visible without scroll; the shelf may sit below the fold
on a window under ~1,050 px tall". Until one is chosen the render set reports 23 failing A8 rows rather than hiding
them. Note this is exactly the kind of blind spot the earlier verdict was about: a spec row that the gate passed while
never measuring it.

### A fourth point, which changes the framing of ruling 2

The coverage clause's population is **self-declared by the renderer**: the row iterates `window.__bay.bodies` and
divides by `bodies.length`, so a body that fails to mount is simply absent from the array — it drops out of the
denominator and the row can only *improve*. Nothing in the gate compares that count with the number of plates drawn
(11 plates, 11 bodies this run, coincidentally equal; no row asserts the equality). Read literally, the clause therefore
cannot detect a missing body at **any** threshold, which is the one thing it exists to do.

So ruling 2 is really two asks: (a) the threshold, where 0.25 is the measured number and 0.60 is unreachable for a thin
body; and (b) the population, where the count must come from the DOM (drawn plates with `[data-body]`) and be compared
with `bodies.length`, or the clause is decorative. (b) is a small gate change; I have not made it, because the tree is
claimed by another live seat and I am not editing it.

## Fifth addendum, 01:00 ET — I was wrong about the ring contrast: I rendered it, and it still fails

The third addendum above says the ring-contrast clause is reachable in two or three steps of the spec's own remedy.
That was arithmetic on a model I had not tested, and the test says it is wrong. **It supersedes the ring-contrast row of
that table and Correction 1.**

How it was tested, with no shipped byte changed: a `git archive HEAD` copy at `/tmp/opencode/wo-proof`, one edit —
`tokens.json` `render-greytone` `#3A3E45` → `#565C66` (three +4 steps) — then `node tools/render-gate.mjs --only a29`.
The built page stays byte-identical (`dist/dash.html` sha256 `c709562f…`, 332,063 B) because the grey token is a
checker-only asset fetched at run time, which is also why this cannot affect the app or the graded screenshot set.

Measured on the same 480×270 stand-in, same 11 bodies:

| | 0 steps (run `1115Z`) | 3 steps (`PROOF-3STEP`) |
|---|---|---|
| ring contrast | 1.349–1.508, **11/11 below 1.8** | **1.583–1.983, 6/11 still below 1.8** |
| coverage | 0.276–0.641, 6/11 below 0.60 | 0.328–0.771, **4/11 still below 0.60** |
| the row as written | 11/11 bodies fail | **9/11 bodies fail** |

Why the model was wrong: the ring is not independent of the body colour. At the 4× stand-in a 24 px body downscales into
6 px and the 8 px ring takes the bleed from the body's own edge — measured `ringL` on `manual-6` rose **0.0212 → 0.0313**
when the body got brighter. My prediction held `ringL` fixed, so it overstated the gain by about a third. The 0-step
column reproducing the gate exactly did not validate the model; it only validated the inputs.

**So A29 (iii) is unreachable inside the spec's own three-step remedy, and that now comes from a render rather than an
argument.** The live asks are ruling 2 (coverage 0.60 → 0.25, with the population fix) and ruling 3 (adjacent-band
wording, or new T1 hexes). There is no implementation change to authorise for the ring: lifting the grey token improves
it from 11/11 failing to 6/11 failing and cannot reach 1.8 in the three steps allowed.

Evidence: `fleet-scratch/overnight/a29-arithmetic/PROOF-3STEP/` (notes, `results.json`, the six PNGs).

## Second-seat confirmation of the A8 numbers, 01:05 ET

I did not write the A8 addendum above; I reproduced it, on the app bytes that are HEAD (`dist/dash.html` sha256
`c709562f…`, 332,063 B — unchanged by anything in this memo), with an independent script and the fixture server:

```
1440x900 : 11 plates, scrollWidth 1440/1440, first plate top 496, last plate bottom 1107  -> 207 px below the fold
1920x1080: 11 plates, scrollWidth 1920/1920, first plate top 500, last plate bottom 1056  ->  24 px inside the fold
```

So the row's numbers hold exactly, the failure is real at A8's stated 1440x900, and it does not occur at 1920x1080. That
paragraph is therefore evidence, not a claim: A8 as written and §5.3's stage height cannot both be satisfied at a 900 px
tall window.

## Fourth addendum, 01:15 ET — I was wrong that the ring-contrast clause is unfixable

My addendum at 00:25 said "no implementation change can meet" both A29 (iii) sub-clauses. That is true for the
adjacent-band clause and **false for the ring-contrast clause**, and the error matters because it made a fixable row
look like a dead end. Recomputed here (WCAG 2 relative luminance, the 5.1 method) against the bay slab `#222529`:

| grey token | vs slab |
|---|---|
| `#3A3E45` — what the render paints today | **1.433** |
| `#434850` — one HSL lightness step (+4) | 1.672 |
| `#4D525B` — two steps | **1.959** |
| `#565C66` — three steps | 2.285 |

The gate's own measured worst ring contrast is 1.35–1.51 against the *8 px ring* rather than the slab, so the ring is
slightly lighter than the slab; two steps land the ring at ~1.84–1.96, i.e. **11/11 bodies pass at two steps**, which is
inside the spec's own remedy allowance ("up to three steps") and inside A20's limits (saturation stays ~5 %, far below
44 %).

What A29 (iii)'s failure rule actually names is "each T0, T1 and T2 hex of every body" — **not** the single grey render
token. So the question is one line of spec, not an implementation change:

> **Does the failure rule name the grey-tone render token as well as the 5.1 body hexes?**

If yes, A29 (iii) drops to two clauses: coverage (needs its own ruling) and adjacent bands (needs new T1 hexes or the
5.1 wording). If no, the grey render was never meant to be graded on contrast and the row should not exist.

Adjacent bands, for completeness, cannot be lifted into compliance: T0/T1 and T1/T2 measure 1.29–1.35 on every body and a
uniform lift leaves the ratio where it was; only T0/T2 (1.68–1.80) clears 1.45. That one still needs the palette ruling.
