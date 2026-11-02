# TEAM D — DIAGNOSTIC PLAY 15 REPORT: abort-contact option-B start + fix C — 12 s window expired, C's ramp not achieved, Wine crash (2026-11-02)

Lead GO 2026-11-02T08:24Z. Current recovered opening (pid 933721, 1/3). State read-only + sealed BEFORE → `preflight`
(rc 0, Edit, 1/3, place==build) → **one** `run traffic:abort-contact` → `verify-clean` → sealed AFTER. No extra Plays.
Run: `docs/verify/2026-10-19/vinegar/20261102T082424Z-traffic-abort-contact`; log
`board/probes/team-d/diagnostic-play15-20261102.log`.

## 1 · Verdict

The option-B leg start ran exactly as wired (`abortObservedAt=contactDriveAt=3.7789 s`), but **the 12 s no-touch guard
fired** (`HandlingServer:221`, t=15.778) with **0 contacts/touches**, then **Studio crashed** on shutdown (userland
minidump `023c0d5b-…` 101,847,584 B @08:24:48Z, exit 5; abort stop/cleanup/verifyClean disconnected). **Fix C did not
close**: although the pursuit commanded the 40 studs/s² clamp, the ego's **achieved** acceleration was only ~5 studs/s²
(and it even decelerated from 29.7 to 19.8 at leg start); the gap went **71.6 → 141.9** and the window expired. This is
live evidence that the kinematic model's 40-cap ramp does not translate into vehicle acceleration — the limiting factor
is traction/steering/physics authority, not the pursuit command.

## 2 · Evidence

| Fact | Value |
|---|---|
| Sealed BEFORE/AFTER | `84324d43…` release, `c6651793…` test/place — unchanged |
| Run | 08:24:24Z → 08:25:47Z; exit **5**; states up to `RAN 6.77` |
| Leg start | `cutInAt 2.7622`, `abortObservedAt 3.7789`, **`contactDriveAt 3.7789`** (abort+0), fixture `preLegSpeed=30`, offset 80 |
| Failure | `HandlingServer:221: contact leg never reached within 12 s of contactDriveAt` (12 s guard at t=15.778) |
| Cell | **FAIL** — touches 0, contacts 0; `completedAt 6.833`; `maximumProgress 0.9971`; contactLeg latched `{t:15.778,distance:142.36,proxySpeed:52.14,speed:60,lead:0.15}` |
| Series (samples 123) | leg start gap **71.6**, ego **29.7**, proxy 27.4 → ego **drops to 19.8** (t=4.73) → 25.2 (6.8) → 33.2 (7.8) → 48.2 (9.8) → **58.6 (11.8)**; gap **71.6 → 141.9**; achieved accel ≈ **5 studs/s²**, not the commanded 40 |
| Guards | **12 s timer tripped** as designed; radius guard not tripped (max 142 << 700); reached-contact exemption untested |
| Crash | `newCrashDumps:1` (101,847,584 B userland minidump @08:24:48Z); abort `stop/cleanup/verifyClean` Disconnected(60 s); Studio died; sealed unchanged; plays 2/3 on the dead opening |
| Post | Studio dead → recovered with `studiocycle open` + latch clear for Play 16; `verify-clean` clean on the recovered place |

## 3 · Judgement / next

- **What it proves:** the option-B start + fix-C wiring is correct; the ruled 12 s guard and radius guard still behave;
  **C's commanded 40 studs/s² is not achieved by the ego in live physics** (~5 studs/s² observed, including a
  deceleration at leg start) — so C does not close from this start. Crash class (3rd intermittent userland Wine shutdown
  dump) recorded honestly; sealed integrity holds.
- **What it does not prove:** no contact/cell pass; the cause of the traction/steering authority limit is not diagnosed
  (candidate: force applied via `assemblyMass` but the vehicle's wheels/steering/ground friction cap the longitudinal
  acceleration, and the pursuit's lateral term fights the heading).
- **For the ruling:** the (i) evidence is now "leg starts, C commanded, no close because achieved acceleration ≈ 5
  studs/s²"; the next step is a Lead/Nox decision (further fixture/physics work or the A window path), not a silent
  change.
