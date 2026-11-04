# OPENBELL WATCHOVER — fresh check 4: A8 honest, word figures only half-fixed (4 Nov 06:17Z)

Verdict file: `fleet-scratch/overnight/checker-watchover-verdict-run4.md`. Read-only on
`~/workspace/openbell-watchover`; everything below measured in `~/.cache/openbell-checker4` against a dist I rebuilt
(`c709562f…`, 332,063 B). Commits `64b8d49`, `d84a13a`, `a1c7e4d`+`bd8f0451`, `2938541`.

## Result: 3 pass, 1 fail, 1 pass-with-caveat

1. **A8 PASS.** The new `a8 plates in viewport` row measures the spec's clause correctly. At 1440x900 on my build:
   front row top **496**, plates bottom **1107** (207 below the fold); with one alert row 500/**1159**; `preview-missing`
   496/**1198**. Alert bands 0–**128.64 px**, inside A8's 212 px budget — so the **23 A8 failures are honest**, not an
   artefact.
2. **Figures FAIL.** The meme word is caught (`not running`, **184.81 px at 28 px**), but the shelf primaries are not:
   the selector still lands on the 15 px `payouts` span, and **111 of 176** figure entries at 1440 are not the primary
   (grok `Unknown`, meme `ok`, all 5 shelf). Proof by break: capping **only** the shelf primary leaves `figures`
   **46/46 PASS** with the ink overflowing; capping every primary gives 46/46 FAIL with numbers. So the row can fail —
   it is simply not looking at the shelf.
3. **Provenance PASS.** `dirty:false`, `dirty_paths:[]`; `tool_sha256` = sha256 of `render-gate.mjs`@`bd8f0451`;
   `build` = sha256 of my own build. My re-run: 368 rows, **0 verdict differences** vs the committed set.
4. **No cutover PASS.** 8791 serves `849e2c1c…` (219,311 B), not the pack; unit still
   `~/Work/projects/openbell`, up since 2 Nov 16:21; **429/429** tests, 2 CSP hashes, 7/7 fixtures.
5. **Reporting PASS.** 23 A8 + 2 A29 = 25; grouped multiplicities sum 23; my numpy recompute reproduces grok
   **0.276 / 1.369** and orb15 **0.329 / 1.509**.

## The three things that still need a ruling

- **A8 vs stage formula.** All 23 states end 203–298 px below 900. The gate now reports it instead of hiding it; the
  arithmetic or the clause must change.
- **A29 (iii).** Still 11 bodies counted where the spec says 6 front bodies (front scope: contrast 6/6, coverage 4/6),
  and the spec's own "lift every tone one step without waiting" remedy is still replaced by a ruling request.
- **Coverage before A30.** The `frozen` state ships three captures but gets no `a8`/`containment`/`figures` row, and
  the shelf word primary is unchecked. Decide whether those must be measured before the owner grades.

Nothing was armed, spent, posted or written to `live/`. The maker tree is clean; my run dirs live only in the copy.
