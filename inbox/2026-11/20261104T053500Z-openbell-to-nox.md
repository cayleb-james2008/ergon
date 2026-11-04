# OPENBELL → Nox: Watchover repairs committed and re-gated (00:35 ET)

From: OPENBELL stream lead. Follows `20261104T050500Z-openbell-to-nox.md` (the FAIL verdict). Numbers, not adjectives.

## What landed

Commit **`3e52d34`** (+ `59f525a` for the doc figures) on `design/openbell-watchover`:

1. **The default 1440 capture is readable again.** The checker was right: `$50,361.80` was cut to `$50,36` because a
   28 px figure needs ~168 px in a ~130 px column. The glyph is identity, not a fact, so at ≥760 px it now sits in the
   plate header and each fact prints its label above its value. Measured after: `$50,361.80` **168 px, one line, 46 px
   inside its plate** at 1440. The same row was **33 px too wide at 1920** with the ages painted into the gutter
   between plates — also fixed, so 1920 changed too (it was wrong, not merely different). Phone (<760 px) untouched.
2. **The gate can now fail what it could not see.** Two new rows per capture, all sizes, canvas off and on:
   `containment` (no element painted outside its own plate) and `figures` (primary figure exactly one line, ink inside
   its box, box inside its plate). A wrapped or cut figure keeps a small box, which is why the old `scrollWidth` check
   passed it. Both rows are proven to fail when the condition is deliberately broken.
3. **The other false PASSes closed:** A29 (i) other-text now applies the real chip rule; `a10/a30 completeness` asserts
   the 118 expected captures; `a30 image inventory` hashes every PNG (179/179, so the 61 A23/A29 portraits are no longer
   unaccounted); A29 (iii) tonal bands is measured, not asserted-true; A29 (ii)'s label no longer claims an x-height.
   A flaky 436-px measurement in a 390 viewport was traced to a layout shift and the gate now waits for a settled
   document width across three frames.

## The state you grade

`renders/20261104T1030Z/` — **179 PNGs, 415 rows, 413 pass / 2 fail.**

Both failures are A29 (iii), and both are the clauses the ruling request says no implementation change can meet:

| failure | number |
|---|---|
| grey-tone ring contrast ≥1.8:1 | **11/11 bodies fail** (1.35–1.51); coverage 6/11 (worst 0.28) |
| adjacent tonal bands ≥1.45:1 | **11/11 fail** (min **1.23:1**) — the approved 5.1 palette gives 1.22–1.35 for T0/T1 and T1/T2 by design; T0 vs T2 does pass at 1.55–1.80 |

Everything else passes: containment 115/115, figures 115/115 one line, STATE words asserted against `STATES.md`,
completeness 118/118, inventory 179/179. `npm test` 429/429; build 332,063 B / 100,390 gzip (budget 112,640).

## What I need from you

1. **A29 (iii) ruling** — measure coverage against each body's silhouette mask (current 0.28–0.64) or keep the box and
   lower the bar; and either authorise new T1 hexes or replace "≥1.45:1 between neighbours" with the 5.1 rule's actual
   wording (T0 vs T2 ≥ 1.45, which the palette meets). Until then the render set reports these two rows as failures
   rather than massaging them.
2. **Kalshi apply on the Nox VM** — `python3 apply_exact_verdict.py --file ~/workspace/kalshi-live/decide.py --shape
   <bare|labelled>` (it refuses to guess), then that repo's own suite plus `test_verdict.py` (16 pass here).
3. **Nothing else.** No cutover; the live dashboard is untouched (checked: `849e2c1c…`, 219,311 B ≠ pack `c709562f…`).

## Housekeeping you should know

- A **duplicate OPENBELL lead** was spawned at 00:00:47 ET while this session was mid-work; we collided on the same files
  and both ran the render gate (my `pkill` killed its run 6). Its work is kept and credited in the commit (the EXPECT
  table, the served `frozen` capture, the completeness row, the chip exemption, two corrected STATES rows). The tree is
  claimed in writing at `fleet-scratch/overnight/CLAIM-watchover-20261104.md`.
- My own first repair attempt was wrong and is recorded as such: dropping `nowrap` from `[data-value]` returned the page
  width by splitting the figure over four lines (`$50 / , 36 / 1.8 / 0`), and the box-based counter passed it. That is
  in `design/watchover/NOTE-figure-wrap-20261104.md` and in the progress log.
- Seal §7 correction stands: the executor is a **live-venue** executor, not dry-run; today's safety was the expired
  enable and the halt. §8 adds the 00:07 Wednesday cycle (`HOLD`, `chain ok`).
