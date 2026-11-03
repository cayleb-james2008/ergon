# TEAM D (Claude) — Item 1 merge done, merged-tree verification, successor queue entry run (2026-11-02 evening, after Nox's four rulings)

## 1 · Merge (ruling 1)
- `prep/d1-closing-16-72-20261102` (freeze `a18c791`, Play 22 live pass) merged into **`main` as merge commit `0c325f8`** (no-ff, same pattern as the A1 merge `de361ba`; A1 `1abe303` was already in main, so the chain order is A1 -> B1 chain). **Local only, not pushed.** The chain touches tests / tools / docs only (32 files, +3308/-350), no production `src/`; `git merge-tree` showed no conflicts.
- Verified ON the merged tree: **full Lune 943 passed / 0 failed (130 specs)**; full Python **336 OK** (includes traffic acceptance and runner 91); sealed `build/test.rbxlx` **c6651793…** and `build/release.rbxlx` **84324d43…** unchanged (and the place file in the Vinegar data dir c6651793…). Nothing published.

## 2 · A finding for you about the merged tree (native runs)
The runner refused a Play from merged `main` (exit 2, **no Play taken**, plays stayed 1/3, no latch, fail-closed as designed): the sealed test place embeds older scripts, and `main` has newer sources from other lanes
(`src/shared/Catalog.luau`, `src/client/Actors.client.luau`, `src/client/WorldRender.luau`, plus `tests/studio/highway.spec.luau` from A1). **Native runs on current `main` need a rebuilt test place, which changes the sealed hash and needs the `tools/usg build` that this lane must not run.**
Until you order that (it is the C1 "native runs on the RC commit's test place" step), native Plays run from the d1 tree (`a18c791`, identical to Play 22): I did that. The working tree is left on `prep/d1-closing-16-72-20261102` for that reason.

## 3 · Successor queue entry (ruling 3), named and run
B1 queue (`docs/verify/2026-10-19/vinegar/v1.queue`) status: `smoke` done, `atlas:scout` done (Play 20), `traffic:abort-contact` done (Play 22). **Next in design order = the abort-contact cadence repeat**; I ran it as **Play 23** in the kept-open Studio (pid 2070598, never closed):
**exit 0, validator accepted, verify-clean clean, sealed hashes unchanged.** Same story as Play 22: cut-in 2.77 s, abort 3.78 s (0 target-lane rows), leg 10.87 s (gap 59.9 studs, direction +1), achieved acceleration 38.4 studs/s², first scored contact at **relative closing 16.0** **5.68 s into the 12 s window** (Play 22: 5.48 s), 1 touch / 26 contact rows, proxy upright. Two consecutive live passes on identical code = repeatable.
Opening is now **2/3**; the runner's 3-Play budget leaves one more Play, which the queue design wants AFTER a reopen (cadence "Plays 2-3, pause, Play 1 after the reopen"). I did not take it and did not close Studio (your ruling 2).
**Next entries, in order:** abort-contact cadence 3/3 (after a reopen), `atlas:highway-extension`, `moving` (first fare with moving frames), then the flag-off baseline packs (`atlas:scout`, `atlas:edge`, `moving`), the written cadence verdict (`docs/verify/<date>/cadence.md`) and frozen `look_metrics` thresholds. The checkpoint file `v1.queue.checkpoint.json` is stale (Nov 1) and will refuse to resume; I used single `run` calls like Plays 19-22.
Evidence: `docs/verify/2026-10-19/vinegar/20261103T040510Z-traffic-abort-contact`; log `board/probes/team-d/play23-abort-contact-cadence2-merged-20261102.log` (also holds the refused attempt).

## 4 · Item 2 (ruling 4)
The guided-continue finding is logged in the card-game spec: branch `prep/card-game-design-spec`, new commit **`6b9c985`** on top of the v0.1 freeze `84c4018` (worktree `~/Work/tmp/card-spec-wt`): Decisions item 6, variance row **V22** (proposed **Δ16**, flagged as NOT in the 15 and an owner decision), and **Appendix D** (the four owner findings re-verified, the guided-continue numbers, other playtest findings, the checker's corrections to the spec's own text, playtester ratings). **Documents only: no code change, no upload, none of Δ1-Δ15 implemented**; the spec-first ruling stands.
