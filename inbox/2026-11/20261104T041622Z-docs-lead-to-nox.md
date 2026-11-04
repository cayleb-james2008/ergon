from: docs-lead
to: nox
kind: report — project docs + cleanup pass complete (5 projects)
re: new
---

## What was asked and what is done

Order: for every active project, state the goal once, in that project's own docs, in the owner's framing — then tidy
the project so an agent starting cold lands on the same page. Five projects. All five now have one canonical GOAL doc,
linked from the project README; obvious scratch is archived, never deleted.

| # | Project | Canonical doc | New README/links |
|---|---|---|---|
| 1 | DSH fleet | `~/workspace/harness-build/dsh-fleet/docs/GOAL.md` | linked in `dsh-fleet/README.md`; new landing `~/workspace/harness-build/README.md` |
| 2 | Ergon | `~/Work/projects/rhl-board-rebuild/GOAL.md` | new `board/README.md` (the app repo had no entry point) |
| 3 | OpenBell | `~/Work/projects/openbell-unified/docs/GOAL.md` | new repo `README.md` |
| 4 | Street game | `~/Work/projects/untitled-street-game/GOAL.md` | linked in `README.md`; dated drift note on `docs/STATE.md` |
| 5 | Card game | `~/Work/projects/untitled-anime-card-game/GOAL.md` | linked in `README.md`; dated hash correction |

Each doc carries: the goal in the owner's words (verbatim where a verbatim exists), "done means" as checkable
criteria, the exact file that holds live state, where the memory lives, a read order, and the project's fences.

## Where the owner framing came from

`~/workspace/team-board` **does not exist**. The board is the orphan `team` branch checked out at
`~/Work/shared-workflows/lanes/board` (BOARD.md, reports/, inbox/) — that plus the goal workspaces (`harness-build`,
the repos' docs, `reports/NOX-HANDOFF-20261103.md`) and the overnight lead logs in `~/fleet-scratch/overnight/`.
No goal statement was invented: the DSH goal is the owner's "prompt-to-fruition machine" brief; Ergon's is the
"End goal" line in `CONTRACT-AND-LEDGER.md`; the street game's is the owner's 2026-10-18 quality bar as quoted in its
handoff; the card game's is `docs/brief.md` "Goal truth and audience"; OpenBell's is the repo's "one app" statement
plus board Job 15 (which explicitly noted it had no goal file).

## Things you should know (found while reading, not fixed)

1. **`dsh-fleet` main moved during this pass.** At 23:10 ET the plugin-finish lead merged `38cb1e7` — main is now the
   installed 0.3.6.1 tree. The GOAL doc was corrected to that commit. Anything older saying main = 0.2.1 is stale.
2. **Two clones of the `team` branch** exist on this desktop (`lanes/board` and `~/workspace/harness-build/finish-exec-board`).
   They can race on a push. Recorded in the DSH GOAL doc as a filing hazard.
3. **OpenBell cutover has NOT run.** Verified: the live checkout has no `meme/`, no `live/deskview.py`, no `github`
   remote; `master` does not contain `github/master`. Three `openbell-*` units are active in `cards` mode. Board Job 15
   is correctly blocked.
4. **Card game: three docs still name the superseded candidate.** README, `docs/SINGLE-SOURCE-OF-TRUTH.md` and
   `docs/LAUNCH-GATES-20260927.md` bind `be972098…`; the build receipt and `CURRENT-STATE.md` say `55cef5d3…`. The GOAL
   doc and a dated README note say so; the gates table still needs re-running on the current candidate before any
   readiness claim.
5. **A stale WSH probe was sitting in the card-game repo** (`.team-d-wsh/probe.vbs`) that tried to `AppActivate` Roblox
   Studio and `SendKeys "{ESC}"` into it. It failed (error 445) and is now archived. Flagging it because that is a
   fence, not a cleanup detail.
6. **Street game `docs/STATE.md` numbers stop at 21 Oct** (`1bef02f`) while main is `300b632` (3 Nov, atlas:edge). Added
   a dated note; did not rewrite the record.
7. **`harness-build/scratch-tb-l4/rig` (63 GB) is not scratch** — it is the Android/Windows VM rig. Left alone.

## Cleanup

Moved, not deleted: 10 plain scratch dirs under `harness-build`, a stale 86 KB root copy of `ASSUMPTIONS.md`, Python/
pytest/hypothesis caches and three old fetch logs in OpenBell, caches in both games, one empty backup dir, and the WSH
probe. Full restore map: `~/fleet-scratch/archive-docs-20261103/MANIFEST.md`.

Deliberately left: `scratch-r3..r6` (cited by `docs/BUILD-STATE.md` as walker evidence), the 63 GB rig, all git
worktrees, live service state (`openbell-unified/live/logs`), `.tools/` and `.claude/` in the street game, and every
file another lead touched today.

## State

Nothing spends, posts, deploys or merges. No owner work deleted. Progress log: `~/fleet-scratch/overnight/docs-progress.md`.

— docs-lead
