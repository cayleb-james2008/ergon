# ERGON — the tip failed its independent check; the failure was mine, and it is fixed (interim)

Filed 4 Nov 2026, 01:20 ET, by the ERGON stream lead. Supersedes the "connection honesty: done" reading in
`reports/ERGON-REMAINDERS-AND-QUOTEFORM-20261104.md` and in the morning section until a second check reports.

## What the check found on `d9c782b` (verdict: FAIL)
Full report: `~/fleet-scratch/overnight/BOARD-FINAL-TIP-CHECK.md`.

1. **A product defect, measured, not a test weakness.** At **3.0 s per answer** the connection label cycled `Error → Live`
   every 4 s and a 20 s run ended on **Error** — 36 % of the run claiming an outage about a server that was answering. At
   2.0 s: 17 %. Cause: a fixed 1.5 s probe deadline re-declared the outage on every probe; the late answer then cleared it.
   The suite's 2 s test asserted one instant of that oscillation, which is why every green run missed it.
2. **A second false claim of the same family.** With a first read that hangs, the label said **"Reconnecting" at 1577 ms**
   before Error at 2577 ms, on a page that had never connected and so had nothing to reconnect to. The old test only
   asserted "never Live", so it could not see it.
3. **Two bounds that no longer described the app.** `round3`'s long-task bound was `max(shape, 4 × hostRef)`; since
   `4 × 41 > 100`, the host term won in every run — including runs with zero long tasks — so a 60-job regression landing one
   ~200 ms task passed. `round4` asserted the **best of three** passes, so a consistent +80 ms regression could pass.
4. Smaller: editing is disabled during each false Error window; the 2800 ms bound had 6 % headroom at this host's speed.

## What changed (`2150fa8`, docs `3780e04`)
- **The deadline is measured, not fixed**: 3 × this connection's own average round trip, clamped 1.5–10 s. A fast link still
  names a hang in ~1.5 s; a 2 s or 3 s link stops being called down after one measurement.
- **A timeout is not an outage on its own**: the kept-alive request is learned from; the extra confirm wait is spent only when
  the link is already known slow (≥ 800 ms), so the "Error within about two seconds" promise on a fast link is unchanged.
- **One outstanding request at a time** is preserved: 2 starts in 9 s against a hung path, gaps 3.4 s.
- **"Reconnecting" requires a loaded board**, so a hanging first read shows no word; `data-state` still says `polling`, and
  that is documented.
- **The test now measures the promise**: 12 s of 100 ms samples at 2 s *and* 3 s, requiring Error ≤ 20 % overall, **zero**
  Error in the last five seconds, and a Live label at the end. The hanging-first-read test now also asserts that
  "Reconnecting" never appears.
- Figures: unit **119/119**, connection **10/10**, full e2e **108 tests / 103 pass / 0 fail / 5 skipped, exit 0**
  (`evidence/board-e2e-connfix-20261104.log`).

## Status of the item 7 bounds
The other ERGON session committed its own tightening (`7035ab9`) at the same time: `round3` ceiling 2 × hostRef and
`round4` on the median. That is the same direction and it stands. **Whether those bounds actually fail a regression is being
proved, not assumed**: the second checker is required to inject a deliberate ~200 ms per-job regression and show the test
fails, and to try to construct a case where the new adaptive deadline hides a real outage.

## What this means for the deploy packet
The tip is `3780e04`, not `d9c782b`. Nothing is deployed; live v3 `455a9da` is unchanged; the owner's one yes is still the
only thing between the branch and production.
