# CHECKER — option-B fixture delta (`68c020b`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the Nox run-8 option-B abort-contact start-geometry fix on top of my accepted layer-6
delta (`8bda598`). I built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a
throwaway clone `/tmp/checker-optionb/repo` (2026-11-02T01:17:34-05:00) and a second clone for mutations; maker tree only
read.

- **Subject tip:** `68c020b71c4dc13288e510da9001b3f5c216c248` (`68c020b`), parent `8bda598`, tree
  `bebf72b0e522431fda4d7129e937dfdf642ae94a`; tracked mods **0**; sealed `84324d43…`/`c6651793…` unchanged
  before/after and at end.
- **Delta over `8bda598`:** exactly **3 files, +92/−3** — `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` (+8),
  `tests/tools/test_traffic_acceptance.py` (+78), `tests/traffic/Server.luau` (+6/−3). No `src/`, `assets/`, `build/`
  change; `tools/vinegar_run.py`, `tests/atlas/*`, `tools/atlas_acceptance.py` untouched → atlas/settle/retry/audit/
  45 s/assert-and-skip intact.
- **Maker report:** `board/reports/team-d-option-b-freeze-20261102.md` is **absent**; verified from the commit.

## Verdict in one line

**ACCEPT** — fixture-only: the ruled pursuit law, both guards and the reachable-contact exemption are **byte-unchanged**
(function-block sha256 identical at parent and tip), the contact leg now starts at `abortObservedAt` (no 4 s wait), the
abort-contact pre-leg rolls at 30 studs/s (pass mode keeps 10), the old `-18` reverse is absent and `fixture.preLegSpeed=30`
is recorded; my independent kinematics give a leg-start gap of **24.31 studs** and **30.0 studs/s** rolling speed (inside
the ruled 12–25 / 28–36 targets), contact in **7.56 s** (<12) vs **17.58 s** from the old 10 studs/s start (>12); the new
scenario/kinematic tests pass, existing A+C tests are unchanged, and all three start-geometry mutations fail the pin.

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Fixture-only; ruled law/guards/exemption byte-unchanged | **PASS** | `git diff 8bda598..68c020b -- tests/traffic/Server.luau` shows only 3 hunks: the `fixture{…}` record (+`preLegSpeed=30`), `local wantedSpeed=if cfg.mode=="abort-contact" then 30 else 10` (was `=10`), and the contact trigger dropping `age-abortAt>=4`. Function-block sha256 of `Server.pursuitAcceleration` (`627e580d95c5…`) and `Server.contactLegGuard` (`d372efb92290…`) are **identical at both commits**; the ruled `closing=math.clamp(proxySpeed+8,12,60)`, the 40 studs/s² cap, the forward `+10` clamp, both named guards and the `touches<=0` exemption are untouched. No `src/` change (delta paths are docs/tests only) |
| ② B implementation + numbers | **PASS (one prose nit)** | `tests/traffic/Server.luau:161` `local wantedSpeed=if cfg.mode=="abort-contact" then 30 else 10` (pass keeps 10); `:206` `if cfg.mode=="abort-contact" and abortAt then contactAt=contactAt or age;…` (no 4 s wait); `-18` absent; `:74` `preLegSpeed=30` in `fixture`. Independent kinematics from the test's Play-9 proxy profile (`start-numbers.txt`): leg-start gap **24.31 studs** (target 12–25), ego rolling **30.0 studs/s** (28–36), min pre-leg gap **24.68** (>17 no-contact threshold), contact from the new start **7.56 s** (<12), from the old 10 studs/s start **17.58 s** (>12). **Prose nit:** the commit/runbook say "~22 studs gap"; the model gives 24.31 (inside the ruled range, test asserts 12–25) |
| ③ Tests + mutation | **PASS** | `tests/tools/test_traffic_acceptance.py` (new): `test_abort_contact_starts_the_leg_at_abort_observed_at_with_rolling_speed` pins the exact `wantedSpeed=if…30…10`, the no-4s-wait trigger, no `-18`, `preLegSpeed=30`, the unchanged pursuit/guard texts and no maneuver writes; `test_option_b_start_geometry_reaches_contact_inside_the_ruled_12_s_window` is the kinematic expectation (leg gap 12–25, ego 28–36, no pre-leg contact, contact <12 s at the rolling speed, >12 s at 10). Existing A+C tests are unmodified (the file diff is additive). Mutations in a throwaway clone, restored: **M23** old start (pre-leg 10 + 4 s wait) → traffic acceptance **FAILED (failures=1)**; **M24** only the 4 s wait → **FAILED (1)**; **M25** only the rolling speed → **FAILED (1)**, each on the scenario-timing test. Restored `tests/traffic/Server.luau` sha256 `9e72bb19aac2…` (frozen), git clean |
| ④ Scope / regressions | **PASS** | Exactly the 3 claimed files; no game/asset/sealed path; atlas/settle/retry/audit/45 s/assert-and-skip untouched (`tools/vinegar_run.py`, `tests/atlas/*`, `tools/atlas_acceptance.py` unchanged) |
| ⑤ Suites (fresh clone) | **PASS** | runner `Ran 91 tests … OK` (12.646 s); traffic acceptance `Ran 9 … OK` (0.082 s); full Python `Ran 331 tests … OK` rc 0 (44.885 s); Lune traffic filter `93 passed, 0 failed (18 specs)` — this filter covers all three Lune specs that read the changed `tests/traffic/Server.luau` (`traffic-contact-pursuit`, `traffic-runtime`, `traffic-timing`). **Full Lune not re-run** (12 min): the delta touches only that file + Python tests/docs; relied on the accepted full-Lune `923/0 (127 specs)` at the parent |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged; maker HEAD `68c020b`, tracked clean at end. Play B (fresh opening) is the live proof |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the start geometry is proven by the source pins + the Play-9-derived kinematic model (the
  proxy is product traffic, unaffected by the probe), not by the engine. Play B is the live proof.
- **R2 — maker report absent:** `team-d-option-b-freeze-20261102.md` is not present; verified from the commit.
- **R3 — prose nit:** "~22 studs gap" vs the model's **24.31** (inside the ruled 12–25 target and the test's range);
  wording only.
- **R4 — model assumption:** the contact-time cross-check uses a conservative steady 52 studs/s proxy (noted in the
  test); it is a bound, not a Play measurement.
- **R5 — cosmetic:** untracked evidence-dir count now 813 vs the "792 pre-existing" wording used in earlier reports.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-option-b-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
atlas invariants + law/guard function-block hashes at both commits + B hunks + `-18`/`preLegSpeed` + start numbers +
maker-report absence), `start-numbers.txt` (gap 24.31 / ego 30.0 / min pre-leg 24.68 / contact 7.56 s vs 17.58 s),
`suites.log`, `runner.log` (91 OK), `traffic.log` (9 OK), `python.log` (331 OK), `lune-traffic.log` (93/0, 18 specs),
`mutations.log` (M23/M24/M25 each 1 failure; restored hash), `run-started.txt`. Throwaway copies
`/tmp/checker-optionb/{repo,mut}`; maker repo read-only, unchanged, still at `68c020b`.
