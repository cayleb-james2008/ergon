# Team A — Job 02 merge record (Pack 21 + scale-up + residuals)

Recorded by `ergon-lead` 2026-10-31 21:50 ET, on the Lead's explicit authorisation ("JOB 02 PASS — MERGE AUTHORISED").
Maker never checker: the PASS is `ergon-checker`'s (`board/reports/ergon-checker-gate-job02.md`); the merged-tree proof is
the checker's to run.

## The merge

- Branch merged: `duet/p21-fixes-scaleup` @ **`4387b519eb64f0fbf2cd8ccf2b060f435cce2474`** (checker PASS on that SHA).
- Target: `main` @ **`1c7189866b41755b92bc57c8b85d91649b529bcd`** (Pack 20 merge) — server truth checked immediately before
  the merge (`git ls-remote --heads origin main`).
- Style: merge commit, `--no-ff` (repo convention is descriptive merge commits; no squash, no rebase of main, no
  force-push).
- **Merge commit: `c196c2574e38b65ae8945ae47c7c17f91ade93aa`** (`c196c25`), message
  “Merge duet/p21-fixes-scaleup (Pack 21 + template-hardening scale-up + residuals, checker PASS 31 Oct) into main”.
  - parents: **`1c71898`** and **`4387b51`** (`git cat-file -p HEAD`; tree `cf8551d10303068d30075c33bbaf8c6155d1f6cb`).
- **Tree identity:** merge tree = branch tree (`git diff --stat 4387b51 HEAD` empty); the merge introduces exactly the
  checker-passed tree.
- **Ancestry:** `git merge-base --is-ancestor 4387b51 HEAD` = **0**; `... 1c71898 HEAD` = 0.
- Working tree clean; **nothing else in the push**.

## Push result

```
$ git push origin main
To https://github.com/cayleb-james2008/ergon.git
   1c71898..c196c25  main -> main
$ git ls-remote --heads origin main
c196c2574e38b65ae8945ae47c7c17f91ade93aa	refs/heads/main
```

**Remote main tip after push: `c196c2574e38b65ae8945ae47c7c17f91ade93aa`.**

## Merged-tree proof (checker, 2026-10-31)

`ergon-checker` ran the merged-tree proof on main `c196c25`: remote tip confirmed; merge tree = `4387b51` tree; both
parents ancestors of main (exit 0); documented-env merged-tree suite `TOTAL 1616 cases: 1607 pass, 0 fail, 9 skipped;
158/158 checks ran, 0 without a complete report; 0 manifest problems`, **EXIT=0**, `uncertainChecks:[]`; merged-tree
failing-case diff vs the gated tree = **0**, branch-attributable = **0**. J2-F1/F2/F3 confirmed **not** in the merge
(tree identity). Report: `board/reports/ergon-checker-postmerge-job02.md`; raw runs
`board/probes/ergon-checker-gate-job02-runs.txt` §11. **Job 02 is CLOSED.**

## Follow-ups (next Ergon commit, NOT in this merge — ordered by the Lead)

- **J2-F1** `docs/EXIT-CODES.md` ~line 71 still describes `scripts/check.mjs` as run by the tests (stale after retirement).
- **J2-F2** `batch-ledger.jsonl` carries the 72 finding events but not a TC-1 template-candidate event; the driver should
  emit the candidate proposal event too (and the committed batch evidence regenerated).
- **J2-F3** the checker handoff must pin the final SHA (`4387b51`; the file currently names `90ddcad` in one line — the
  delta is docs/evidence-only).
These three are low/non-blocking and are named for the next Ergon commit after this merge. Job 03 prep and the Job 04
packet remain intact; successors per the Lead: Job 03 (Ergon-on-DSH package is the named 11–14-day slippage risk; board
fold-in blocked on the work-PC window), Job 04 Mon 01:00 ET from the handoff packet, Job 05 standing (no Kimi handback).
