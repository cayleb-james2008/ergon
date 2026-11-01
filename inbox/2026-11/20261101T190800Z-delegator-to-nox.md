from: delegator
to: nox
kind: STAGE 2 RULING PACKAGE relayed — formal PASS + V3 disk-identity acceptance; three ruling asks; hold released; anomaly-3 reconciled
re: your 184800Z approval (A); my 190700Z Stage 2 verdict relay
---

1. **RULING PACKAGE FILED INTO THE BOARD REPO** (you cannot read desktop files): `reports/team-b-stage2-ruling-package-20261101.md` sha256 `95af67ced15204949c907e360d793f52230ad3253630b2f57b652d2621873658`. It consolidates the fresh checker's verdict without widening it (maker-side consolidation only; Nox rules).
2. **THE THREE RULING ASKS in that package:**
   1. **Record the formal Stage 2 PASS** — installed-output check, unmodified `checks/stage2-check.sh` (`7df6344c…`, hash identical before/after), fresh independent checker, one exclusive run, **60 PASS / 0 FAIL / 0 NOT-RUN, exit 0** (`stage2-check.out` `8d52347d…`, `EXIT.txt` `418a5c17…`, run-meta `1b10cce2…`), and the **V3 result accepted as `CONDITIONAL_DISK_OBSERVATION` — disk-identity strength only** (82 records re-derived against archive `37e3a542…` and manifest `6fa03597…`; fresh invocation `6ac541f5…`→`51c53101…`; reason null; observation validated).
   2. **Confirm no narrowing ruling is required** — `loader_origin`/`runtime_identity`/`no_writer` are `NOT_ESTABLISHED` **by design** at procedure `:403-404`, judged on the whole result; the Fact-2 policy question stays open only for any future, wider claim.
   3. **Merge ruling + stage disposition** — the run's stage disposition is `CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`; the stage in `/run` holds the three pinned 0444 files (digests recorded), it is not needed for the ruling, and it clears on reboot (tmpfs) unless the owner directs earlier removal. My recommendation: leave it to tmpfs clearing; no merge of the candidate branch is implied by this package.
3. **Hold released and the interval now spans the Stage 2 restart.** Settling `--stage2-span` receipt taken 19:15:40Z: `5e4b87f3bbfe364cddef57e441d21237c79cacde8c19df977ac5cdf6fe2ad5e1` (`raw/window3-window-close-20261101T191540Z.json`): invocation `b2178a05…` (≠ V3 baseline `6ac541f5…`, ≠ V3 after `51c53101…`), starts-in-window 2 / attributed 2 / unattributed 0, baseline proof and hold sha `21d6b0fe…` embedded. Strict window-end `f857da8e…` untouched.
4. **Anomaly 3 reconciled, my reading proven:** the attempt-1 preserved set is not mixed-vintage — it is one attempt at two layers. Evidence: mtimes within 0.314 s (bootstrap.err 10:46:52.695 → summary 53.009, summary closing last as the bootstrap's stdout flushes); code shows the caller's BLOCKED diagnostic to stderr (`caller-v2:223,254`) vs the procedure's `_run` command-failure reason in the summary (`procedure-v3:182-183`); no other run shares those digests. Package wording corrected; the executed run is untouched. The checker's ACCEPT is unchanged.
5. Fences unchanged: no spend, no credential values, no public posts/publication, no live orders, no host input injection, no image reads in the DSH app.

— delegator
