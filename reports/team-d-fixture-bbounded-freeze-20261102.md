# TEAM D — B-BOUNDED FREEZE (abort-contact gap trigger + bounded pre-leg), 2026-11-02

Nox run-9 ruling: **B-bounded adopted** (A and A+B-bounded rejected; C held pre-approved as the next step if a B-bounded
Play starts inside the gap and still misses contact within 12 s). Fixture-only: the contact leg starts only when the proxy
is within 25 studs, bounded by a 4 s named pre-leg after `abortObservedAt`; initial ego offset 70; the ruled 12 s window,
closing clamp, 40 studs/s² cap, both guards and the reached-contact exemption are untouched.
**Freeze commit `e6ae1344fd945ff9f9e7ee1d371ad1fc346919ed`** (`e6ae134`; parent `68c020b`; tree `1c0bb6b8…`; 3 files
**+85/−61**; tracked clean; sealed `84324d43…`/`c6651793…` unchanged). **Checker delta: ACCEPT** (ruled blocks
byte-identical; trigger/timeout verified; Play-11 profile: gap at abort 66.62 → trigger 2.60 s after abort, gap 24.87,
contact 7.63 s vs old 17.65 s; mutations M26/M27/M28 fail; runner 91 / traffic 9 / Python 331 / Lune traffic 93-0).
Full Lune on the freeze tree: **923 passed / 0 failed (127 specs)**.

**Files:** `tests/traffic/Server.luau` (`CONTACT_GAP=25`, `PRELEG_TIMEOUT=4`, offset 70, gap-trigger + named phase
message, `report.preLeg`/`report.contactLegTrigger`, fixture audit fields) · `tests/tools/test_traffic_acceptance.py`
(scenario pins + Play-11-profile kinematic trigger test, Play-9 profile marked one-run) ·
`docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` (cell description). **Play 12 outcome:** honest named phase fail (gap at
abort 73.55, never ≤25, named timeout at 4 s) → per ruling, option A goes back to Nox with that evidence; C not
triggered. See `board/reports/team-d-diagnostic-play12.md`.
