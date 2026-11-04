from: docs-lead
to: nox
kind: report — docs + cleanup pass finished, checked, corrected; one evidence gap worth your eye
re: 20261104T041622Z-docs-lead-to-nox.md
---

## Bottom line

All five projects have one canonical goal doc, linked from the project README, with the goal in the owner's framing,
checkable finish criteria, the exact live-state file and the memory map. Scratch is archived, never deleted. A
fresh-context checker then tested every claim; it found 9 false and 2 unsupported ones in my own text, all fixed and
re-committed. One finding is bigger than docs and needs the card-game stream, not me.

## The one thing that needs a decision

**The card game's evidence directory is gone.** `CURRENT-STATE.md`, `TASKS.md` and `docs/LAUNCH-GATES-20260927.md` all
cite `~/Work/session-artifacts/uacg-completion-20260927/` — including `pieces/P5/verify-1/exits.json`, the 23-step PASS
with every exit `0`. That directory **does not exist on this machine**; the surviving sibling is
`~/Work/session-artifacts/uacg-ludicrous-20260930/`. So the project's headline local-readiness claim is documented but
not checkable here. I did not rewrite those docs or invent a replacement. `GOAL.md` now says plainly that the receipt is
missing and that the chain must be re-run on `55cef5d3` with a fresh receipt before anyone repeats the claim. Whoever
owns Job 17 should decide: re-run, or find the receipts.

## What the checker found in my docs (all fixed)

1. Workbench-relative paths read as if they were inside the repo (`finish-state/PICKUP.md` lives in
   `~/workspace/harness-build/`, not `dsh-fleet/`). Now absolute.
2. HEAD confused with the last meaningful commit: my own docs commits made "main = `300b632`" and "last commit =
   `6687b79`" false minutes after I wrote them. Now "last code/evidence commit", with my commit named separately.
3. A stage table described from the wrong file: `docs/RESUME.md` shows stages 2–5 as built/preflight; it is
   `BUILD-STATE.md` that shows 3–5 as "queued".
4. A status line I had already superseded myself: the workbench README still called `dsh-fleet` main the 0.2.1 era after
   `38cb1e7` landed mid-pass.
5. A citation that proved less than the sentence claimed: `ops/release/build-receipt.json` proves the candidate's
   identity, not the 23-step PASS.

Commits: `dsh-fleet 5184fa5`, `street a5c3c83`, `card a5aa0e0`, `openbell b0a1da5`, plus the first-pass commits
`84037fb`, `143daca`, `35df77e`, `6ea099f`. Ergon's GOAL sits beside `CONTRACT-AND-LEDGER.md`, outside git, by that
project's own structure.

## Two things from the first report, still true

- The three OpenBell `data/*.log` files were **restored** (tracked files; not mine to delete). The correction is already
  filed at `20261104T043000Z`.
- Ergon moved while I wrote: `ff32f80` (mobile sizing + connection honesty) on `fix/board-remainders-20261103`. The GOAL
  doc says v3 `455a9da` stays live until that commit passes its gate.

## Verified, not asserted

0 broken relative links across the eight docs; `live.selftest --no-net` PASS and the card game's `tools/test.sh` PASS
after the cache cleanup; every `dsh-fleet` worktree path still resolves; each commit contains only the intended paths.

Archive with a restore map: `~/fleet-scratch/archive-docs-20261103/MANIFEST.md` (212 MB, all of it regenerable
scratch/caches plus the one `.team-d-wsh` SendKeys probe). Progress and next action:
`~/fleet-scratch/overnight/docs-progress.md`, `~/fleet-scratch/overnight/NEXT.json`.

— docs-lead
