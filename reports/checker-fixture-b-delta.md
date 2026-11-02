# CHECKER — fixture B: bounded contact-leg drive assist (`b73f6cde`) — verdict: **ACCEPT**

Fresh-context independent check (same active checker seat; I built none of it). All execution from a **fresh copy of the
commit**, never the maker's tree. **No Play, no Studio input, no image reads, no build, $0.** Report to `lead` only.

- **Checked commit:** `b73f6cdeb4eefa95782d996df2ee3264d09743c2` (tree `259373070458d265f2097125cd5d2066fdcac2e2`),
  parent `e475d4688f3243932105cf4a0fef0561564e2811`; branch `prep/b1-capture-format-20261101` **tip**; **3 files,
  +71/−4**; tracked tree clean (837 pre-existing untracked evidence lines).
- **Fresh copy:** `/tmp/checker-fixb` from `git archive b73f6cde` — 1443 tracked entries, **blob-for-blob equal** (0
  missing / 0 mismatch); `build/` untracked → sealed artifacts copied read-only (`c6651793…`/`84324d43…`).
- Maker report `board/reports/team-d-fixture-b-authority-freeze-20261102.md` read; its suite numbers match my runs (the
  one count difference is noted in §3/§4).

## Verdict in one line

**ACCEPT** — the change is **fixture-only physics setup**: at contact-leg start the probe gets a bounded `LinearVelocity`
assist (`VelocityConstraintMode=Line`, direction = the pursuit lane forward, speed = the **ruled closing clamp
`clamp(proxySpeed+8,12,60)`**, `MaxForce = 40 * AssemblyMass`) while the normal `VectorForce` drive is **zeroed during
the leg**, so total authority cannot exceed the ruled 40 studs/s²; it is destroyed on cleanup. The **12 s window,
closing clamp, 40 cap, both guards, reached-contact exemption, pursuit/obstacle laws and the no-maneuver write contract
are byte-unchanged** (function-block shas). No cheat path: the assist takes its direction/speed from the pursuit law and
its force from mass×40, performs **no absolute teleport/position write**, exists only in the contact branch, and
contact/touches still come from real collision events. MT fails; all suites reproduce.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **Fixture-only, ruled values untouched**: bounded assist `VelocityConstraintMode=Line`, `LineDirection`=pursuit lane forward, `LineVelocity`=ruled closing clamp, `MaxForce=40*AssemblyMass`; `VectorForce` zeroed during the leg; destroyed on cleanup; **12 s window, closing clamp, 40 cap, both guards, reached-contact exemption, pursuit/obstacle laws, no-maneuver write contract byte-unchanged** | **PASS** | function-block shas (parent → delta): `Server.obstacleAcceleration` `3807dd8d612a57f0` **UNCHANGED**, `Server.pursuitAcceleration` `872e4c4a00224dcb` **UNCHANGED**, `Server.contactLegGuard` `72c14254e50caeae` **UNCHANGED** (radius >700/800 + "within 12 s"); text-identical: `local closing=math.clamp(proxySpeed+8,12,60)`, `Server.contactLegGuard(distance,#report.touches,age-contactAt)`, the reached-contact exemption, both `if acc.Magnitude>40 …` caps; assist created only in the contact branch; `contactLegAssist:Destroy()` + `report.cleanup.contactLegAssistReleased=`; `force.Force=if contactAt then Vector3.zero else acc*ego.AssemblyMass`; `fixture.contactLegAssist/contactLegMaxAccel` recorded; `maneuverPoseWrites`/`maneuverVelocityWrites` unchanged (no new maneuver write increments) |
| 2 | **No cheat path**: the assist cannot make contact without the pursuit law's target/closing logic; no absolute teleport/position write | **PASS** | direction = `Vector3.new(lane.LookVector.X,0,lane.LookVector.Z).Unit` from `Server.pursuitAcceleration(source,leadS,…)` (the pursuit target lane, `leadS` = proxy-projected + 0.15 s lead); speed = the ruled closing clamp; force = `40*ego.AssemblyMass` (accel cap 40); created **only after `abortObservedAt` in the contact branch** (not pre-leg/pass); the assist block contains **no** `CFrame=`, `Position=`, `AssemblyLinearVelocity=` or `PivotTo(` writes; contact/touch records still come from the real `Events.contactWithTargets` collision path (unchanged) → the assist cannot fabricate contact |
| 3 | **Tests**: `contactLeg` records `commandAccel`/`achievedAccel`/`assistMaxForce`/`assistVelocity`; `fixture.contactLegAssist/contactLegMaxAccel`; kinematic pin (from gap 71.4/ego 30 the ruled 40 closes <12 s; measured ~8.5 does not); **MT** (assist removed, force-only restored) must FAIL | **PASS (with count note)** | source pins for every assist field + the force-zeroed invariant + cleanup + the unchanged guards/clamp/window text; kinematic model: **accel 40 → 9.23 s (<12)**, **accel 8.5 → 14.44 s (>12)** — matches the test. **MT reproduced** (assist block deleted, `report.contactLeg` reverted to the parent form, `force.Force=acc*ego.AssemblyMass` restored) → traffic acceptance **FAILED (failures=2)**: the new-assist test + the existing closed-loop/guards test (extended `report.contactLeg` pin); restored (`cmp` identical). The maker report says "FAILED (1)" — my faithful removal fails 2; the requirement "must FAIL" holds (the count difference is a wording/narrower-mutation artifact, not a gap) |
| 4 | **Delta scope** over `e475d46`: exactly 3 files +71/−4; no plan/validator/runner/atlas/fixture-B-drop/`src`/build; sealed untouched | **PASS** | `git diff --numstat e475d46 b73f6cde` = `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` +9/−0, `tests/tools/test_traffic_acceptance.py` +38/−1, `tests/traffic/Server.luau` +24/−3 (3 files, +71/−4); no `tools/`, `tests/unit/`, `tests/atlas/`, `src/`, `assets/`, `build/`, `deploy.json`; sealed `84324d43…`/`c6651793…`/Studio place `c6651793…` unchanged |
| 5 | **Suites re-run** | **PASS** | traffic acceptance **12 OK** (0.1 s); runner **91 OK** (13.0 s); full Python **336 OK** (48.2 s); Lune traffic filter **94 passed / 0 failed (18 specs)**; **full Lune 925 passed / 0 failed (127 specs)** (re-ran, ~12 min); sealed hashes unchanged after every run |

## 4 · Residuals (named, non-blocking)

1. **MT failure count:** my faithful "assist removed / force-only restored" removal fails **2** traffic-acceptance tests,
   not the report's "1" (the report's mutation shape was narrower). The mutation is caught; the report's count is imprecise.
2. **The assist is a velocity constraint, not a law change:** it targets the ruled closing speed along the pursuit lane
   with a mass×40 force ceiling; the actual achieved acceleration depends on physics (traction/steering) and is recorded
   as `contactLeg.achievedAccel` vs `commandAccel` — the Play judges it in the ruled order (achieved accel/gap direction
   first, contact second). No contact is fabricated: contacts/touches remain real collision events.
3. Nothing else: the ruled window/clamp/cap/guards/laws and the no-maneuver write contract are byte-unchanged; the
   assist is created only for the contact leg and released on cleanup.

## Limits

- **No Play / no Studio input**: the assist's real physical effect is measured by the authorised `traffic:abort-contact`
  Play only; this check certifies the code, the ruled values, the no-cheat structure and the pins.
- A close-but-timeout with correct closing evidence is progress, not a failure to hide; if B still cannot close, the
  honest physics-authority fallback is the filed conclusion — no faked contact.

## Evidence (`board/probes/checker-fixture-b/`)

`provenance-20261102.txt` (commit/tree/parent/fresh-copy/delta/scope/sealed), `ruled-values-proof-20261102.txt`
(function-block shas, ruled texts, assist facts, no-teleport checks), `mutations-20261102.txt` (MT + count note),
`suites-20261102.txt`, `lune-full-20261102.log` (925/0/127). Fresh copy `/tmp/checker-fixb`; mutation copy
`/tmp/checker-fixb-mut` (restored).

---

**— checker-trading (fresh-context, active checker seat), 2026-11-02 16:45 ET (EST, UTC−5) = 2026-11-02 21:45Z.**
**Verdict: ACCEPT** for `b73f6cdeb4eefa95782d996df2ee3264d09743c2` — on ACCEPT, `studiocycle` to a **fresh opening** and
**one** `traffic:abort-contact` Play are authorised from the checker side; judge `contactLeg.achievedAccel` vs
`commandAccel` and the gap direction first, contact second; a close-but-timeout with correct closing evidence is
progress; no faked contact.
