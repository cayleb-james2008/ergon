# plugin-finish → Nox: soak record was silently dead; now supervised and pinned

**From:** plugin-finish (o15 Stream 1) · **2026-11-04T06:52Z** · model `opencode-go/deepseek-v4.1-flash`

## The finding, in one line

The soak sampler armed at 01:03 ET **died at 06:19:44Z** (harness restart) and the toolchain did not notice: `soak-check.sh` still
printed a calm `days_aside 0 / restarts_seen 1 / NOT YET`. A sampler dead for a day would still have produced `days_aside 1`, so
three soak facts could have read true from a two-hour window of samples — a soak that never happened would have passed.

## Fixed, each with a test that fails when the rule is removed

1. **A dead sampler is a verdict.** `soak-check.sh` prints `sampler LIVE/STALE` (default limit 1800 s = two cycles) and `STALE`
   is now a NOT YET reason that blocks READY. Suite **16/16**; mutation (drop `and not stale`) → 1 fail.
2. **A corrected timestamp was counting as the required restart.** Restart detection keyed on `(pid, start)`; the 06:02Z TZ fix
   rewrote the start of 21 rows for the *same* pid 2313700, so the live run read `1 restart(s)` for a Host that has never
   restarted. Now keyed on pid: live reads `0 restart(s); 1 distinct Host pid(s) across 30 samples`. Mutation → 2 fails.
3. **`soak-supervisor.sh`** (new, armed pid 3776780 at 06:47:15Z, 104 × 15 min = 26 h, detached): flock single instance — because
   **two samplers were writing the same log** (mine, and a resume-spawned seat's unguarded `for` loop pid 3668935, started
   06:41Z), which is why the sampler now has a `--min-gap-s` slot rule; read-only `--status`; TERM/INT/HUP 143/130/129; sampler
   timeout; failures counted, not fatal. Suite **29/29**.

Three defects were in my own new code and were caught before shipping (a failing sampler looped for 24 h; the flock outlived the
process via an inherited fd and refused the next copy; `--status` named the helper's pid). All three are now tests.

## What is still owner-gated (unchanged by any of this)

- `.lane-heal-applied.json` is **still absent** → the lane-heal patch is staged, not installed; P3 and the soak's real restart
  both need the one quit + relaunch (owner sheet steps 1–2, 4–5).
- The day: not before **2026-11-05T03:45Z**. `RESTARTS=` / `ACKED=` stay typed by the owner (8.3); nothing here writes `soak.json`.
- After a reboot the supervisor does not return; run the one line in its header (`soak-supervisor.sh --status` says `alive=no`).

## Fences

Nothing installed, nothing deleted, no confirm typed, no credential value read, no spend this session. The owner's app and its
processes untouched; the other seat's sampler loop left alone (another agent's process, and harmless now).

Evidence: `~/fleet-scratch/overnight/evidence/SOAK-SUPERVISOR.md`; owner steps: `OWNER-RUN-LANE-HEAL-20261104.md` §6.
Independent checker running → `CHECK-SOAK-CHAIN.md`.
