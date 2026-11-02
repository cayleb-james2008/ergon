# TEAM D — ABORT-CONTACT B FREEZE: bounded contact-leg drive assist (physics-consistent authority), 2026-11-02

Nox run-12 (B, relayed after the voided hold): give the ego **authority consistent with its physics** for the contact leg
(contact-leg vehicle setup / drive assist and/or start geometry), keeping the ruled **12 s window, 40 studs/s² cap,
closing clamp `clamp(proxy+8,12,60)` and both guards untouched**; fixture/test files only; success measured on
**achieved acceleration and gap direction first, contact second**. Offline only in this freeze (no Play).
**Commit `b73f6cdeb4eefa95782d996df2ee3264d09743c2`** (`b73f6cd`; parent `e475d46`; tree
`259373070458d265f2097125cd5d2066fdcac2e2`; tracked clean; **3 files +71/−4**); sealed
`84324d43…`/`c6651793…` unchanged. Log: `board/probes/team-d/fixture-b-authority-freeze-20261102.log`.

## 1 · Why (evidence this addresses)

Play 15 (option-B start + fix C) commanded **+40 studs/s²** at the leg start but the ego **achieved only ~−15…+8.5
studs/s²** (it decelerated 29.7→18.5, then crept to 59 only at ≈12.8 s); the gap grew **71.6 → 141.9** and the 12 s guard
fired. Root cause named in the evidence note (`board/probes/team-d/abort-contact-c-achieved-acceleration-20261102.md`):
the ruled command does not translate into vehicle authority (traction/steering/force application), and the probe enters
the leg with velocity **opposite its heading** (`v·LookVector = −33.17`), so a realignment/braking phase precedes any
chase. B fixes the authority channel, not the ruling.

## 2 · What changed (fixture only)

- `tests/traffic/Server.luau`: at contact-leg start the probe gets a **bounded `LinearVelocity` assist** (new `Attachment`
  + constraint): `VelocityConstraintMode=Line`, `LineDirection` = pursuit lane forward, `LineVelocity` = the **ruled
  closing speed** `clamp(proxySpeed+8,12,60)`, **`MaxForce = 40 * ego.AssemblyMass`** (the same ruled 40 studs/s²). The
  existing `VectorForce` drive is **zeroed while the leg is active** so total authority stays at the ruled cap. The assist
  is destroyed in cleanup (`report.cleanup.contactLegAssistReleased`), `report.contactLeg` now records
  `commandAccel`/`achievedAccel`/`assistMaxForce`/`assistVelocity` (achieved-vs-commanded witness), and
  `fixture.contactLegAssist=true; fixture.contactLegMaxAccel=40` are recorded (physics setup, **not** maneuver pose/velocity
  writes — the fixture write contract is unchanged).
- **Untouched:** 12 s no-touch window, radius guard (>700/800), closing clamp, 40 cap, reached-contact exemption,
  pursuit/obstacle laws, no-maneuver write contract, validator/plan/runner.
- `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md`: cell description of the bounded assist.

## 3 · Tests / evidence

- Source pins for every assist field + the force-zeroed invariant + cleanup + guards/clamp/window unchanged text; kinematic
  expectation: from the observed option-B leg start (gap 71.4, ego 30) the ruled 40 studs/s² closes **<12 s** while the
  measured ~8.5 does not.
- **Mutation MT** (assist removed, force-only drive restored) → traffic acceptance **FAILED (1)**; restored, tracked clean.
- Suites: traffic acceptance **12 OK**; runner **91 OK**; full Python **336 OK**; Lune traffic filter **94/0 (18 specs)**;
  full Lune **925/0 (127 specs)**; sealed unchanged. **Checker-ready.**

## 4 · Next (per the ruled sequence)

Fresh-checker delta on `b73f6cd` → on ACCEPT, **`studiocycle` to a fresh opening** (current opening is at 3/3), then **one**
`traffic:abort-contact` Play. Judge in this order: (1) `contactLeg.achievedAccel` vs `commandAccel` and the gap direction
from the sample series; (2) contact inside the ruled 12 s; (3) guards (`12 s` must not trip if contact lands; radius guard
not tripped). A close-but-timeout with correct closing evidence is progress, not a failure to hide; if B still cannot
close, the held fallback is the scenario reconsideration with the honest physics-authority finding. B1 cell-pass record
(Play 20), sealed hashes, diagnostics chain and the four intermittent Wine shutdown dumps stay intact.
