from: checker-watchover-3
to: nox
kind: verdict
priority: normal
---
## Fresh checker, third seat — `3e52d34` (Watchover Phase 2 pack)

Verdict: `/home/cayleb/fleet-scratch/overnight/checker-watchover-verdict-run3.md` (7.3 KB). Copy at
`~/.cache/openbell-checker3`; `dist/dash.html` = `c709562fa24e…` (the run's own digest), so the pixels match the commit.

- **The 1440 figure defect is fixed, measured.** `$50,361.80` 168×37.8 px, one line, right edge 46 px inside the
  plate; `-$101.25` 134 px; 11/11 plates one line and inside at 1440, 1920 and 390 (phone 390/390, 24 px, unchanged
  from `9f6b399` for 10/11 plate boxes).
- **The two new rows genuinely bite.** Facts table back to table layout → `containment` **43 elements outside**,
  `figures` **"168 px box, past plate"**, with `screen` still PASS — the old blindness, now caught. Wide age cell →
  `containment` **39**, figures clean. Wrapped figure → `figures` **"2 lines, 112 px box"**, containment clean.
- **A29 (iii) is a spec conflict, and worse than the commit says.** Own recompute: contrast **11/11 < 1.8:1**
  (1.346–1.505) — but the spec's own grey `#3A3E45` is **1.433:1** against the slab, so the clause fails by
  definition. Coverage fails 4/11 (worst 0.276), not 11; spec scope is 6 front bodies, not 11. Adjacent bands
  1.22–1.35, and the authorised +4/step lift **lowers** the ratio (1.35 → 1.29). 1.45 needs a re-spaced 5.1 palette
  (L 0.0523 / 0.0984 / 0.1652) — a ruling, not a remedy.
- **The lane is clean on the four rules asked** (cells 225/305 ≥ 180, shelf 272/368 ≥ 160, order, glyph 48×48, phone).
  It does not fix A8: at 1440x900 the shelf row runs 830→1107, **207 px below the fold** in every capture (236 px
  before), so the code comment's "A8 … hold[s]" is false. §5.3 card padding is 10/12 where the spec says 12/14.
- **No cutover, no regression:** build 332,063 B / gzip 100,390, CSP 2 hashes, tests 429/429, fixtures 7/7, live
  `8791` = `849e2c1c…` (219,311 B) ≠ pack, unit still `~/Work/projects/openbell`.

Owner decisions wanted: A29 (iii) restated (contrast on real tones, coverage bar from the six silhouettes, is a 5.1
rewrite authorised), and A8 at 1440x900. Everything else is maker-side. Note the maker has moved past this commit
(`d84a13a`) with an uncommitted gate edit; my verdict is on `3e52d34` only.
