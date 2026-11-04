# DOCS LEAD → Nox: second pass done — goal docs re-read against tonight's filings, one real finding in the card game

From: docs lead (`opencode-go/deepseek-v4.1-flash`). 4 Nov 2026 05:18Z (00:18 ET).
Mission: one canonical GOAL.md per active project, accurate to the repo it sits in, plus root cleanup.

## What this pass changed

All five GOAL docs existed from the first pass. This pass re-read the repos and the newest inbox filings instead of
trusting the docs' own status lines, and corrected where the night had moved:

1. **Ergon** (`rhl-board-rebuild/GOAL.md` — the root is not a git repo): status rewritten to 05:30Z. Branch
   `fix/board-remainders-20261103` at `7837a89`, verified 6 commits on from `4319779` with `455a9da` an ancestor; unit
   119 / e2e 108 (103 pass / 0 fail / 5 skipped); live v3 untouched; the deploy is the owner's one yes and now has a
   served-behaviour reason (live says `Reconnecting` 117 → 1754 ms on every healthy load). SimBank v3 recorded as a
   separate gate: **NOT PASSED**, one charged-ahead packet (ML-D2) from the re-read, no fourth run, needs a ruling.
2. **OpenBell** (`docs/GOAL.md`): the three sibling worktrees are now named so a cold agent does not file them as
   separate projects — the watchover pack (`design/openbell-watchover`, render gate 169/176 with 11 named failures, the
   checker's FAIL, four owner calls open) and the two shadow-only Kalshi worktrees. Cutover still not run.
3. **Card game** (`GOAL.md` + a dated drift note in `CURRENT-STATE.md`): tonight's fresh evidence bound with its limits.
4. **DSH fleet**: re-checked, not rewritten. `docs/GOAL.md` at `2493502` still matches the host: main 0.3.6.1, heal
   branch 0.3.6.2 unapplied, `verdict-P3.json` still absent.
5. **Cleanup**: card `.pytest_cache` archived (manifest updated); street `.tools/probe/` + `docs/verify/2026-*` (live
   evidence) and the watchover worktree's uncommitted edits left alone and named.

## The finding worth a decision (card game)

The committed build is **one fix behind the committed source**, and the fix's evidence is stranded outside the repo.

- `build/UntitledAnimeCardGame.rbxlx` in the repo still hashes `55cef5d3…` (checked with `sha256sum` tonight).
- The D1 fix `cd91a88` (`operation_id_of` missing from the P0 path) landed 1 Nov, **after** the candidate was preserved
  at `add37b6` on 31 Oct. Nothing in the repo's `build/` includes it.
- Tonight's 22-step Linux-domain PASS (`~/fleet-scratch/overnight/card-evidence/`, two byte-identical builds of
  `99ded6a6…`) was produced in the temp merge worktree `~/Work/tmp/card-merge-wt` (tip `a22c17e`), which also holds the
  matching receipt (built 2026-11-04T04:26:18Z). Its own notes say "local staging build only … Studio Play and touch
  rendering remain unverified", and the log's closing line says "Linux domain tests only, never Studio evidence".

So: no candidate currently matches both the repo's source and its build, and the only receipt for the current source
lives in `/home/cayleb/Work/tmp/`. The card stream's call, not a docs edit: either commit that receipt with the
`99ded6a6…` artefact and name it the candidate, or rebuild in-repo and re-run — before any Studio, upload or publish
work. I did not move or commit anything in that worktree.

## Second look

A fresh pinned explore subagent checked every new claim and falsified two of my own writes — I had said the repo's
`build/` hashed `99ded6a6…` (it does not) and that the fresh run "writes no release receipt" (the worktree has one).
Both corrected, plus a "three rulings" count that is four. Note for the habit file: a scratch log's PASS binds to the
tree that produced it, not to the repo you are standing in.

## Files

- Progress: `~/fleet-scratch/overnight/docs-progress.md` (second-pass section).
- Archive/restore map: `~/fleet-scratch/archive-docs-20261103/MANIFEST.md`.
- Docs commits this pass: `card da27e6b`, `openbell 8f02efd` (plus the earlier first-pass commits).
- Relative-link audit across all five GOAL docs: 0 broken links; all six READMEs still link their GOAL doc.
