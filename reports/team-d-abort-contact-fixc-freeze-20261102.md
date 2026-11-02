# TEAM D — ABORT-CONTACT RE-FREEZE: option-B leg start + fix C (Nox run-10 (i)), 2026-11-02

Nox run-10 (i) ruling: revert to the **option-B leg start** and apply **C** (pursuit forward clamp `10 → 40` inside the
unchanged 40 studs/s² cap). **B-bounded is RETIRED as tested-and-rejected** on its fair sample (0/3 convergence: abort gap
~73.5 stable, monotone growth to ~122–124, ≤25 trigger structurally unreachable; the bounded pre-leg failed closed as
designed). **No ruled value changes**: 12 s window, 40 cap, closing clamp, both guards untouched. (iii) rejected; A stays
rejected (its path back is only a real leg start that still misses on time with the ramp fixed). Fixture-only, no
production code.

**Freeze commit: `1eed7e776d6d2484371b72e6632eb7129cdbee7d`** (`1eed7e7`; parent `746a99e`; tree
`8abc65e560d2f7f2e7f93d9bf2ff40df8b6e2594`; tracked clean; **4 files +71/−104**). Sealed artifacts unchanged
(`84324d43…`/`c6651793…`). Log: `board/probes/team-d/abort-contact-fixc-freeze-20261102.log`.

## 1 · What changed

- `tests/traffic/Server.luau`: the B-bounded gap trigger / `CONTACT_GAP` / `PRELEG_TIMEOUT` / named pre-leg timeout are
  removed; the contact leg starts at `abortObservedAt` again; option-B geometry restored (initial ego offset **80**,
  `preLegSpeed=30`, abort-contact pre-leg 30 / pass 10, no `-18` reverse); **fix C** in
  `Server.pursuitAcceleration`: forward term `math.clamp(…,-38,10)` → `math.clamp(…,-38,40)`; total magnitude cap
  **40 unchanged**; lateral term unchanged; `Server.obstacleAcceleration` keeps `-38,10`; both guards untouched.
- `tests/unit/traffic-contact-pursuit.spec.luau`: extracts and pins **both** helpers — pursuit straight-line forward
  acceleration reaches the full 40, the obstacle law still clamps at 10, the total cap holds with lateral error; the
  earlier direction/cap/closing/guard tests stay.
- `tests/tools/test_traffic_acceptance.py`: pins the option-B leg start and the full **retirement** of the B-bounded
  artifacts (trigger/timeout/`probeGap`/`contactLegTrigger`/`contactGapTrigger`/`preLegTimeout`/named message all absent;
  no 4 s wait; no `-18`; no maneuver writes); fix-C source pins (`-38,40` in pursuit, `-38,10` in obstacle, force-cap
  count 2, identical lateral term, closing/guards untouched); kinematic regression from the Play-11 option-B start
  (gap 71.4, ego 30, proxy 52) — clamp 40 closes inside 12 s, clamp 10 does not.
- `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md`: cell description updated (option-B start + fix C; B-bounded recorded as
  retired).

## 2 · Tests and mutation proofs

- Suites on the freeze commit: traffic acceptance **10 OK**; runner **91 OK**; full Python **331 OK**; Lune traffic
  **94/0 (18 specs)**; full Lune **924 passed / 0 failed (127 specs)**; sealed unchanged.
- **M22** (revert the pursuit clamp to 10, obstacle law untouched): traffic acceptance **FAILED (1)** and the Lune
  pursuit spec **FAILED (1)**. **M29** (restore the B-bounded gap trigger): traffic acceptance **FAILED (1)**. Both
  restored, tracked tree clean.

## 3 · Next

Fresh-checker delta on `1eed7e7` → on ACCEPT, one abort-contact Play in the recovered opening (currently **1/3 used, two
Plays reserved**; a Play is not consumed by this freeze). Evidence chain for the Play: leg starts at `abortObservedAt`,
`contactLeg` series, both guards (12 s timer must not trip if contact lands; radius guard not tripped), contacts/touches,
sealed before/after, plays counter. No image reads; $0.
