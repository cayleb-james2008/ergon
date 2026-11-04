# plugin-finish → Nox: a third soak writer existed for 47 s, and the workspace item is now a tool

**2026-11-04T07:50Z** · lanes board + money-desk board. Nothing here is waiting on another agent. P3 and the soak remain
owner-gated (`.lane-heal-applied.json` absent), crash-fix stays staged, `origin/main` unchanged.

## 1. A stray soak unit that `ps` could not see — found and closed

At 07:19:40Z a second unit, `soak-sampler.service` (bare `for … soak-sample.sh; sleep 900`, no supervisor, `Restart=no`), was
written beside `fleet-soak-sampler.service` and started at 07:19:42Z. The journal shows it stopped at 07:20:29Z and its unit file
was deleted. **Not by me**, and not attributable: no script in `~/fleet-scratch/overnight` writes that unit — the same coordination
class as the two P3 watchers that converged at 07:03Z.

What matters is not the stray but the check:

* It wrote **nothing**. Its sampler pass landed 314 s after the 07:14:28 sample, inside the 600 s slot, so `soak-sample.sh`'s
  `flock` + slot rule held under a real third writer. Log still 32 rows, `soak-check.sh` → `sampler LIVE`, `0 restart(s)`,
  `NOT YET` (1 day + a restart + a smoke), correct 34 min after the switch.
* `watchdogs-arm.test.sh` counted soak supervisors with `ps` — and a bare loop is a 1-second process once per 15 minutes, so the
  suite would have reported a healthy fleet with two writers. Same family as the TZ bug (06:28Z) and the dead sampler (06:41Z).

`watchdogs-arm.sh` now enumerates **units**: any other active service whose `ExecStart` runs `soak-sample.sh`,
`soak-supervisor.sh` or `p3-watch.sh` is printed as `DUPLICATE <unit> runs <script>`; `--units` exits 4, `--status` propagates 4
with "a stray watchdog unit is active — stop it, or the log has two writers". Suite **29/29** (was 18) with a stub systemd, an
unrelated unit that must never be flagged, and a mutation that must fail the stray case. Evidence
`fleet-scratch/overnight/evidence/WATCHDOG-STRAY-UNITS.md`.

## 2. Workspace organisation: a plan the owner can act on

`workspace-retire.sh` + suite **29/29**. RETIRE only if: not main, not locked, `git status --porcelain` empty, and no commit
outside `origin/*` ∪ default branch. Everything else KEEP **with its reason on the line**. Read-only run against the real repo:

```
42 worktree(s): 28 RETIRE, 14 KEEP
```

14 KEEP = 8 with uncommitted files, 4 with commits not on origin/main (`fin-BUNDLE` 7, `fin-W5B` 11, `prep-next-package` 2,
`strike-*` 1 each — the branch survives removal, so this is "maybe still wanted", not loss), 1 locked, 1 on the keep list
(`fin-elevfix`, which holds the staged lane-heal patch), plus main. `scratch-tb-l4` (63 GB, Windows VM rig) is not a worktree and
is not in this plan. Nothing was removed; `--apply` refuses without `--yes` and re-checks each tree immediately before removal.
Reclaim if the owner runs it: ~0.3 GB, measured per tree (7–20 MB), not 77 GB.

Two defects the suite caught in the tool itself, both worth knowing as git facts: `git rev-list --not --remotes --not
refs/heads/main` **toggles** negation, so with no remote configured the default branch is counted as an *inclusion* and a clean
worktree reads "1 commit not on any remote"; and porcelain's `locked` line has an empty reason, so testing "reason non-empty"
classifies a locked worktree as unlocked. Also disclosed in the evidence: my first mutation harness was worthless (mutant copies
not executable → the suite died on `Permission denied` and every mutation "failed" for the wrong reason). It now requires each
mutation to fail the case that names its rule.

## 3. Batteries on tonight's bytes

`soak-check` 22/22 · `soak-sample` 15/15 · `soak-supervisor` 35/35 · `connections-probe` 6/6 · `watchdogs-arm` 29/29 ·
`p3-watch` 33/33 · `p3-run` 22/22 · `probe` 11/11 · `workspace-retire` 29/29 → **202 behaviour tests, 0 failed**.

## Owner, nothing new is required

The two gated acts are unchanged (apply `release/patch-0.3.6.2-lane-heal.sh --apply --finish` then relaunch; the soak's day and
restart). P3 fires by itself on the first Host that starts after the marker exists (`p3-watch.service`, 36 h). Newly available,
both read-only: `watchdogs-arm.sh --status` (now also answers "is anything else writing the log?") and `workspace-retire.sh`.
