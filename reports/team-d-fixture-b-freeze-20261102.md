# TEAM D — OPTION-B FREEZE (abort-contact fixture start geometry), 2026-11-02

Nox run-8 ruling: **option B adopted** (A and C rejected) — the pursuit and the 12 s guard are correct; the start
geometry made the leg begin at ~20 studs gap with only ~9 studs/s rolling speed. Fixture-only change; ruled law/guards/
values untouched. **Freeze commit: `68c020b71c4dc13288e510da9001b3f5c216c248`** (`68c020b`; parent `8bda598`; tree
`bebf72b0e522431fda4d7129e937dfdf642ae94a`; tracked clean; **3 files +92/−3**); sealed `84324d43…`/`c6651793…` unchanged.
**Checker delta: ACCEPT** (pursuit law/guards byte-identical to parent; only `preLegSpeed=30`, `wantedSpeed` 30 for
abort-contact (10 pass), leg trigger at `abortObservedAt`; `-18` absent; mutations M23/M24/M25 fail; runner 91 /
traffic acceptance 9 / Python 331 / Lune traffic 93-0). Full Lune on the freeze tree: **923 passed / 0 failed (127
specs)**.

**Files:** `tests/traffic/Server.luau` (leg starts at `abortObservedAt`; abort-contact pre-leg rolls 30 studs/s; pass
mode keeps 10; `fixture.preLegSpeed=30`; no maneuver writes, contract intact) · `tests/tools/test_traffic_acceptance.py`
(scenario pins + kinematic expectation using the Play-9 proxy profile; mutations M23–M25) ·
`docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` (cell description: fixture-only timing/start change).

**Play 11 outcome (2026-11-02T06:27Z, run `20261102T062704Z-traffic-abort-contact`): honest FAIL in a new way** — the
option-B wiring ran exactly as ruled (`contactDriveAt=abortObservedAt=3.7826 s`, `preLegSpeed=30`), but this run's proxy
phase differed (proxy ~27 studs/s at abort vs Play 9's ~11), so the **leg-start gap was 71.4 studs, not the modelled
24.3**; the pursuit closed but the 12 s guard fired at t=15.765 s (distance 142.4; radius guard correctly not tripped).
No crash, sealed unchanged, plays 1/3. Named layer 8 = **proxy-phase variability / unbounded leg-start gap**; options A
(20 s window) / B-bounded (gap trigger + named phase fail) filed in `board/reports/team-d-diagnostic-play11.md`; the
kinematic test's fixed Play-9 proxy profile is a one-run model and should be marked as such in a later offline change.
**Nothing else implemented.**
