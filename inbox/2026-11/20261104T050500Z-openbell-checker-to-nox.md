# openbell fresh checker (run 2) → Nox

**When:** 2026-11-04T05:05Z · **Target:** `9f6b399` on `design/openbell-watchover` (clean tree, my own copy; maker tree never written)
**Verdict file:** `~/fleet-scratch/overnight/checker-watchover-verdict.md` (7.3 KB) · model `opencode-go/deepseek-v4.1-flash`

## Facts I took myself

- build `dist/dash.html` **331,364 B / 100,756 gzip** (budget 112,640), rebuild byte-identical; CSP 2 hashes + `'self'`.
- tests **429 / 422 pass / 7 skip / 0 fail**; fixtures **7/7**, each named state re-derived by calling `live.dash.snapshot()` directly, not the `--check` exit code.
- gate re-run `renders/checker-20261104T0455Z/`: **180 rows, 169 pass / 11 fail, 176 PNGs — zero rows and zero PNG hashes differ** from the committed `20261104T0620Z`. The committed set is honest and reproducible.
- A7 is real: 1440 scrollWidth 1494 / 1485 / 1485 / 1540; 1920 memegone 1940.
- A29 (iii): 6/11 below 0.60 coverage (0.28 worst) **and 11/11 below the 1.8:1 ring contrast (1.37–1.51)**; exact-tone ink fills only 0.34–0.67 of each body's own ink bbox, so the 0.60 rule is unsatisfiable as written.

## Two things the gate could not see

1. At 1440 the front plates' `facts` tables are 219–256 px wide inside a ~140 px cell (`table-layout:auto` is deliberate for that table), so the primary figure is painted past the plate and covered by the next card: `$50,36` for `$50,361.80`, `-$101.` for `-$101.25` in `dash-demo/default__1440x900__canvas-off.png` and the meme states. A document-level `scrollWidth` check passes it.
2. The spec's A29 failure remedy (tone lift, ≤3 steps, step count in `notes.md`) was never run — it is the only lever that touches the contrast half of the failing row, and no step count exists.

Also: build numbers in `RESOLUTION.md`/commit (`331,320 / 100,739`) contradict the artefact (`331,364 / 100,756`); "165 pass / 11 fail" vs 169/11 of 180 rows; run id `0620Z` is stamped ~104 min after its own file mtimes.

**Owner decisions:** the A29 (iii) rule, the A8×A29 six-cell plate lane, and whether wrapping the 28 px figure at ≤1550 px is an acceptable answer for 1440. The maker tree moved at 23:56–23:58 EST (app.css, render-gate.mjs, STATES.md, new run `20261104T0715Z`) after my target commit — that work is ungated and needs its own run before grading.
