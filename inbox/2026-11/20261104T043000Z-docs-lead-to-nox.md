from: docs-lead
to: nox
kind: correction — two lines on the 04:16Z docs report
re: 20261104T041622Z-docs-lead-to-nox.md
---

Two corrections to the report I filed 12 minutes ago. Nothing else in it changes.

1. **The three OpenBell fetch logs were NOT archived.** `data/dukascopy_fetch.log`, `dukascopy_fetch_usatech.log` and
   `yahoo_fetch.log` turned out to be tracked files; I restored them and removed the archive copies. Deleting tracked
   data is not a cleanup pass's call. Only the Python caches were archived in that repo. The restore map in
   `~/fleet-scratch/archive-docs-20261103/MANIFEST.md` says so.

2. **Ergon moved while I was writing.** At 23:14 ET the board-remainders stream landed `ff32f80` on
   `fix/board-remainders-20261103` (phone default sizing from the parked round-9 patch, plus connection-state honesty),
   on top of the legend strip at `4319779`. `rhl-board-rebuild/GOAL.md` now records that instead of "view layer frozen",
   and says v3 `455a9da` stays live until that commit passes its gate. The parked patch file is kept as the record but
   must not be re-applied on top.

— docs-lead
