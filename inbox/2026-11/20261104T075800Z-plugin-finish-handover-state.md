# plugin-finish → Nox: handover state (08:00Z), with what changed since 07:12Z

**2026-11-04T07:58Z**, read from `date -u`. `fix/lane-archive-heal` = `0c01771`; `origin/main` untouched; live install 0.3.6.1,
`.lane-heal-applied.json` absent; Host pid 2313700 since 03:46:14Z; one watcher (`p3-watch.service`, MainPID 179677, 60 h → 06 Nov
19:33Z) and one sampler (`fleet-soak-sampler.service`, 36 samples, newest 07:54:28Z).

## Since the 07:12Z note

1. **The installer has a suite (44 cases).** Check/apply/finish/rollback rehearsed on copies of the live 0.3.6.1 state, plus the
   commands an operator can get wrong: `--finish` before `--apply` refuses and leaves the bridge row; a repeated `--apply` is a
   byte-identical no-op; a second `--finish` says "absent already". Two behaviours were *wrong or silent* before this:
   `--rollback` after `--finish` does not re-arm the bridge row (a row without its module would break the profile) while the header
   promised it did — both the header and the output now say which case is which; and `--finish` says out loud when the running Host
   predates `applied_at`, because that order takes the old Host's rebound sockets with it. `f9e0ee2`, `261e2b9`, `0c01771`.
2. **The patched package boots.** `release/test/boot-heal-proof.mjs` (9/9) starts a real scratch Host on the 0.3.6.2 files:
   `status ok`, `integrity {ok:true}`, 11 pinned / 12 disabled, no roster mount error, clean stop. Stated limit: a scratch home has
   no lanes, so it proves the package boots, not the lift (that is `archive-heal.test.mjs`). `80dd423`.
3. **A stray soak unit ran for 47 s and no process check could see it.** `soak-sampler.service` (bare loop) existed 07:19:40–07:20:29Z,
   wrote nothing (its pass landed inside the slot), and `ps` cannot see a 1-second-per-15-min process. `watchdogs-arm.sh` now
   enumerates **units** and exits 4 on a stray; 29/29 with a stub systemd and a mutation.
4. **Workspace organisation is a tested plan, not a sweep.** `workspace-retire.sh`: 43 worktrees → 28 RETIRE / 15 KEEP, each KEEP
   naming its reason; two real git traps found by the suite (`--not --remotes --not refs/heads/X` *toggles* negation when there is no
   remote; porcelain's `locked` has an empty reason). Nothing removed; ~0.3 GB, not 77 GB.
5. **The retired-profile fence is measurable.** `retired-profile-baseline.sh`: 7 982 files / 322 348 281 bytes / digest `05a8ef5c…`,
   `--verify` exits 3 on any change including a same-size edit (18/18). The archive stays gated on the owner's Q8/Q16.
6. **I fed the live watcher on purpose** (marker absent, so it could only refuse): a process with `comm=electron` and the Host's argv
   was detected (`host changed: pid=322874`), sampled, and refused as `no-marker`. The deployed detect path is now verified, not just
   the fake-probe tests. **The same five minutes exposed a hole**: the soak sample would have recorded that pid as the Host. It now
   records the fleet core's own `status.pid`, keeps the probe's answer as `host_pid_probe`, and warns on a mismatch (22/22, verified
   live with the impostor still running).
7. **Three suites' rules are pinned in-suite now** (`soak-check` 25, `soak-sample` 22, `connections-probe` 13) — their mutations had
   only ever been run by hand. `evidence/TEST-NONVACUITY-20261104.md` names the one suite still without them (`p3-watch`, the other
   seat's surface) rather than quietly editing under them.

Battery: **237 cases here + 44 in the repo = 281, 0 failures.**

## Two of my own errors, corrected on the record

* Three filings were timestamped from the order of work rather than the clock (the two named `090000Z`/`095000Z` were written
  07:44/07:45Z). Correction filing `20261104T074600Z` on both boards; bodies now carry the written time.
* The owner sheet's watcher window said "7 Nov ~07:34Z" (72 h). It is **2026-11-06T19:33Z** (07:33:50Z + 60 h), computed and shown.

## Nothing new for the owner

Apply → quit → relaunch → `--finish`; P3 fires by itself; the soak needs the day, a sampled restart and an acked smoke. New
read-only tools for the morning: `watchdogs-arm.sh --status`, `soak-check.sh`, `workspace-retire.sh`, `retired-profile-baseline.sh
--verify`, and `INDEX-plugin-finish.md` as the reading order.
