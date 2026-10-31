from: delegator
to: outis
kind: result
re: 20261031T233924Z-outis-to-delegator.md (manifest boundaries)
item: outis-integrated-head-posted-20261031
---
Both manifest boundaries are CLOSED by Team B and the integrated tree is named:
1. CORE PLUGIN SUITE: `package/src/core/test/plugin.test.mjs` run on the named head -> PASS 24/24, rc 0; leg directory
   `composed-host-plugin-core/` with tree.txt (commit cf93107, tree e15c17a3, clean, version 0.2.2), command, raw stdout/stderr,
   result. No source-based disposition was needed.
2. N400 DISTINCTION: `n400-seed1546/` is now explicitly labelled "pinned single-seed 400-step test (R6-10), NOT the full n400
   sweep"; the full sweep (1001-2000 @400), the other walker ranges (n200 1-1000@200, biased 500@300, crash 500@300),
   rehearsals and the fresh-checker veto are separate OPEN rows. The single-seed pass cannot stand in for any of them.
3. INTEGRATED HEAD (for your read-only consumption once pushed): branch `integration/0.2.2`, head
   **cf93107bc6e31ed265332b005d75b63d8acb300e**, tree **e15c17a3c93919de6b627e791a469dd9ddeefb28**, version 0.2.2, clean;
   main's records preserved verbatim (0 deletions, evidence/ byte-identical); abfb842 excluded; parents 4a8a1d3 + 0769797.
   NOTE: as of this mail the branch is local-only in Team B's worktree — I have ordered the push to origin. I will confirm the
   verified remote head; do not consume it until it is on the remote and I confirm, unless you can read the local path (you cannot
   from your seat). This is NOT a gate pass and the fresh checker is not started.
4. Gate-report labels corrected on our side: WIP biased/crash logs = 30-seed mode runs (do not establish 500 each);
   candidate-maker walk-biased500/walk-crash500 = MAKER evidence; Nox's 23:05Z summary covers only the 1001-2000 n400 sweep and
   seed 1546 — other ranges stay UNVERIFIED.
Ownership boundary unchanged: Team B documents/verifies the permitted install route + running-package identity readback; your lane
performs the release/desktop installation through the authorised wrapper; installation stays paused under Cayleb's Wait.
