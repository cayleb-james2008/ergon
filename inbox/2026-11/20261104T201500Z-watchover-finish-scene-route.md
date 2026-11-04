# Watchover — the Phase-3 scene-route question is decided; the graded set moved to 1958Z

From: watchover-finish stream (OpenCode, ollama/minimax-m3.1-flash:cloud)
To: Nox / owner
When: 2026-11-04 ~20:15 UTC

## The decision (mine to take, per the order; inside the spec envelope)

**One self-contained hashed HTML.** The scene module is bundled inside the hashed app block, so:

- `live/dash.py` needs **no new route and no serving-path change** (SPEC §18.1: "one hashed HTML, NO serving-path
  change"). Under §17 a serving-path change would have been Cayleb's ruling to make; this way there is nothing to rule
  on, and the cutover is a one-file swap of `live/dash.html`.
- The pack goes back on the React the spec pins (§11.2 `react@18.3.1`; it had drifted to 19.3.1), which is what makes
  the self-contained page fit the 110 KiB gzip budget: **315,119 B / 99,897 gzip** against 112,640, cold transfer
  (page + vendored three.js) ~271 KB against 300 KB.
- three.js stays external and lazy: the 3D path requests `/`, `/state` and the vendored file and nothing else
  (`npm run smoke`: 30 ok, 0 fail). `tools/serve-dist.py` now adds **no route at all** — its route set is dash.py's.
- The spec already said this: §9.3 "the scene module is a plain object inside the hashed app block", §11.2 "exactly
  two script blocks (importmap, app)". The separate `scene-<hash>.js` contradicted both and created the open question.

## The graded set (this supersedes the 1055Z pointer)

**`renders/20261104T1958Z/`** in `/home/cayleb/workspace/openbell-watchover` (branch `design/openbell-watchover`),
committed at `b6fac18` (the change itself at `c158ba9`).

- 446 rows, **420 pass / 26 fail**, 179 PNGs, 118/118 expected captures, inventory 179/179, `dirty: false`
- Page digest `e80d5066f4fd560b3dad8ebe55894732d64861a1fa26c1ff1627672a8b72ec54` (315,119 B / 99,897 gzip), gate tool
  `f9429f40…`, fixture build READY (7/7 roots)
- **All 179 PNGs are byte-identical to the 1055Z set** of the previous page, and the 446 row verdicts are the same: the
  change is a visual no-op. Only the serving route and the page size moved
  (332,145 B/100,955 gz + a 64,429 B scene asset → one 315,119 B/99,897 gz file).
- The owner's contact sheet is rebuilt for this set: `~/fleet-scratch/overnight/watchover-grade-index.html`
  (179 images; the 26 open rows at the top with their numbers). The older sheet pointed at 0725Z, which no longer
  exists.

## The 26 rows: each written down (no threshold moved, no row hidden)

`design/watchover/OPEN-ROWS.md` (new, in the pack) has every row with its own numbers, the arithmetic, and the
decision it needs. Summary:

- **24 × `a8 plates in viewport`** — A8's viewport clause at 1440×900. The front row's top passes on all 24 rows
  (444–500 px); the roster's bottom is 1103–1253 px in a 900 px window (203–353 px below the fold). It is arithmetically
  unreachable with §5.3's stage formula and §6.1's plate content: 1,107 px of content in the quietest state, and under
  A8's own 212 px alerts budget 48 + 212 + 240 of chrome leaves 400 px for eleven plates that need 599 px. The one
  implementation lever (a 240 px stage) was measured and **rejected**: §9.1 ties the camera's vertical field to the
  stage's aspect, so a 240 px stage crops the Sentinel's and both Brutes' heads off the top of the bay (screenshot in
  the progress file). **Three rulings needed** — restate A8's clause, grade at 1440×1080, or re-derive §5.3's stage
  rule with §9.1's framing.
- **2 × `a29 (iii)`** — grey-tone coverage/contrast (worst coverage 0.28 vs 0.60, worst ring 1.37:1 vs 1.8:1) and
  tonal bands (adjacent 1.22–1.35 vs 1.45:1). Coverage is a shape property and the silhouettes are RULED (ruling 5),
  so the spec's tone-lift remedy cannot move it; the tonal-band clause asks for adjacent pairs, which the RULED §5.1
  table (1.22–1.35 by design) cannot meet, while that table's own band rule (T0 vs T2 ≥ 1.45:1) holds at 1.55–1.75,
  and the spec's remedy makes the adjacent ratios worse (1.227 → 1.212). **Two rulings needed**: the coverage
  threshold (0.60 → the silhouette, or 0.25–0.30) and the band wording (T0 vs T2, or new T1 hexes — a palette ruling).

The spec's A29 remedy (up to three +4 HSL tone steps) was **not baked into the shipped pack**: it makes no row green
and it would change every render the owner is about to grade. The fresh checker executes it in its own copy and
records the step count and the rewritten hexes, as A29/A30 require.

## State of the machine

- **Staging:** `http://127.0.0.1:8792/` serves the new build (digest `e80d5066…`, 315,119 B) through the real
  `live/dash.py`, fixture root `live/fixtures/dash-demo`; `?shot=1&still=1` for the DOM, `&gl=force` for the 3D layer,
  `?preview=…` for the fifteen preview states. The seven authored roots: `python3 -B tools/serve-dist.py --root
  dist/fixture-build/fixtures/<root> --port 8793`.
- **Live:** `:8791` still serves the old dashboard (`849e2c1c…`, 219,311 B). Unchanged, and it stays there until the
  owner grades.
- **A fresh-context checker is running on the new HEAD** with an explicitly pinned model (the requested
  `openai/gpt-6.1-sol` was refused by the provider with HTTP 400 twice; the check runs on `anthropic/claude-opus-5-5`),
  on its own copy, read-only on the tree. Its verdict lands at
  `~/fleet-scratch/overnight/checker-watchover-finish-verdict.md`.

## Recorded, not fixed (found by this stream, outside the 26)

The **48×48 DOM body glyph** (§7) is not wired: `Plate()` takes a `glyph` prop that nothing passes, so every plate
shows an empty panel square. Pre-existing (the prop has been there since the plate lane was built), a DOM-lane task,
left alone here — the owner may want it before the cutover.

Evidence and screenshots: `~/fleet-scratch/overnight/watchover-finish-progress.md` and
`~/fleet-scratch/overnight/watchover-finish-shots/`.
