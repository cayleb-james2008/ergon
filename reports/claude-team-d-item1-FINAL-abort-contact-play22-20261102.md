# TEAM D (re-homed to Claude) — ITEM 1 FINAL: abort-contact PASSED live on Play 22 (2026-11-02 22:35 ET) — supersedes the interim report `claude-team-d-item1-abort-contact-fixd-20261102.md`

## 0 · Result
**`traffic:abort-contact` passed in a fresh contained Studio opening, first and only Play (plays 1/3 of the opening, 2 reserved, unused): runner exit 0, host validator `accepted: true`, `verify-clean` clean,
sealed place and release unchanged.** Run: `docs/verify/2026-10-19/vinegar/20261103T033545Z-traffic-abort-contact` (also archived: `board/probes/team-d/play22-abort-contact-run-20261103T033545Z.tgz`;
log `board/probes/team-d/play22-abort-contact-fixd-closing16-20261102.log`). Frozen code: branch `prep/d1-closing-16-72-20261102` @ **`a18c791`**. Nothing merged, uploaded, published or pushed.
Premise note (Nox accepted it): Play 21 had already run B on the old geometry; this was Option 1 (geometry) plus your ruling (a) (closing clamp).

## 1 · The evidence, in the ruled order (gap direction and achieved acceleration first, contact second, guards third)
1. **Gap direction.** `contactLeg.direction = +1` (travel along the lane: the proxy had already passed the obstacle, which waited in the target lane). Gap along the lane (obstacle minus proxy; negative = obstacle behind): −36 at 10.5 s while it waited, −58 at the leg start (10.98 s),
   −82 at 12 s (the obstacle was still accelerating from 8 studs/s while the proxy ran at 52), then −79 → −72 → −63 → −48 → −39 → −32 → −15 at contact: **monotonically closing from t≈12 s**. Lateral offset 13.8 → 1.0 → −0.3 studs
   (the obstacle tracked the source lane; Play 21 had lost lateral control and drifted 28 studs).
2. **Achieved acceleration.** Commanded 40 studs/s² cap; **achieved 38.0–38.7 studs/s²** from the samples (Play 15 had commanded 40 and achieved ~8.5). The obstacle reached 95% of its commanded 68 studs/s
   (= proxy 52.1 + 16) **1.6 s after the leg started**, held 68.1–68.2 until contact. `assistMaxForce` 2642.13 = 40 × mass 66.05; `contactLegAssistReleased: true`; no pose/velocity writes
   (`maneuverPoseWrites = maneuverVelocityWrites = 0`, `initialBodyPlacements = initialVelocityWrites = 2`).
3. **Abort genuine, lane never committed.** Cut-in at 2.78 s, hold/return observed 3.78 → 10.88 s (37 rows), `maximumProgress 0.87`, **0 sample rows with lane == target**. `contactDriveAt` **10.93 s**, strictly after `abortObservedAt` 3.78 s
   (the leg waited 7.1 s in the target lane at 8 studs/s until the proxy was back in its source lane).
4. **Contact, scored by the production scorer.** First scored contact at **16.42 s = 5.5 s into the 12 s window** (touch at 16.55). Five consecutive `Events.contactWithTargets` frames at **relative closing 16.0 studs/s exactly**
   (ego 68.1 vs proxy 52.1), 26 contact rows in all; native `Part.Touched` 1 row, within 0.3 s of the scorer row. After impact the proxy was carried to ~66 studs/s with the obstacle (relative ≈ 0.3),
   stayed upright (up.y 1.000, angular 0.000), speed max 66.5 (limit 150): a gentle shove, no flip.
5. **Guards.** The 12 s no-touch guard did not fire (contact at 5.5 s); the observation-radius guard did not fire (55 studs at leg start, 14.9 at the end); generation stable, 0 events; cleanup: all four connections closed, runtime clean.
   Controller cost p50 0.13 ms, p95 0.28 ms, max 0.44 ms; 130 samples; 604 studs of proxy travel.

The offline model predicted this: leg ≈ 9.1 s (actual 10.9), direction +1, contact scored with relative 16.0, 3.0–4.1 s into the window (actual 5.5 s: the real return took longer, so the leg started later and from a larger gap, 55 studs vs the model's 32).
The checker's one unclearable risk (first contact frame sampled after the collision response) did NOT materialise: the scorer fired on the five frames before impact.

## 2 · What was built and checked (all offline before the Play)
- Fix D (`174ba2c`): leg waits in the target lane at 8 studs/s until the abort resolves (or 10 s), then travels TOWARD the proxy (`contactDirection`), assist `LineDirection = along`, `MaxForce = mass·sqrt(40² − lateral²)`, VectorForce keeps only its lateral part.
- Ruling (a) (`956cf99` → extracted in `5620348`): `Server.contactClosingSpeed(proxy) = clamp(proxy+16, 12, 72)`; the 12 s window, 40 cap and both guards unchanged. The Lune spec EXTRACTS that function, feeds the REAL `Events.contactWithTargets`, and sweeps holding speeds 0–16 × proxy speeds 40/45/52:
  scored 1.1–4.1 s into the window, relative closing 16.0–17.2, target lane never committed, in every cell; the superseded `+8/60` clamp is pinned as a named limit (touches, never scores).
- Final commit `a18c791`: anchored pins for the closing glue (a `*0.5` on either line or on `LineVelocity` is now caught), stale clamp text cleared (fixture diff vs `5620348` is comment-only).
- Suites: **full Lune 931 passed / 0 failed (127 specs) on `5620348`** (13 min 26 s; earlier 929/0 on `174ba2c`, 930/0 on `956cf99`); this commit: traffic Lune 100/0, traffic acceptance 12 OK, runner 91 OK, full Python 336 OK (on `956cf99`); sealed `build/test.rbxlx` c6651793… and `build/release.rbxlx` 84324d43… unchanged before and after.
- Two fresh checkers. #1 on `174ba2c`: FAIL (HIGH: 8 < 12 scorer threshold) — correct, led to the ruling. #2 on `5620348` against your two conditions: **PASS** — (i) scorer maths: the real function returns false at relative 8/11/11.99, true at 12+ (swept branch knife-edge at exactly 12.000), chase relative closing is exactly 16 for proxy ≤ 56 and ≥ 12 up to 60;
  Play 21's proxy (52.2) is covered; (ii) the higher cap breaks nothing: guards/window/force bound byte-identical, no validator bound applies to ego speed, nothing in TRAFFIC_SPEC.md or the A1 highway spec (`1abe303`) constrains fixture or contact speed, 12 s window and gentleness hold across 30 grid runs (contact 1.1–4.6 s into the window, relative 16.0–17.2 ≈ 16–17 km/h; the client light-impact cue is 12 km/h, heavy 35 km/h; no damage/wreck code in `src`).

## 3 · Limits, honestly
- One Play, one proxy speed regime (52 studs/s), chase geometry (direction +1). A head-on outcome (direction −1) would be ~120 studs/s relative; it never occurred in the model or the Play. If the abort resolved with the proxy behind the obstacle on another run, the fixture would reverse into it at that speed.
- The checker's mutant table: remaining survivors were the two glue lines, now pinned; a `+14/72` or `+20/80` clamp still passes the Lune spec (gentle band) but is caught by the Python pins.
- B1 queue state: the Studio opening used 1 of 3 Plays; the cell is `traffic:abort-contact` only. I did not run `tools/usg`, did not touch Cayleb's personal session (a fresh contained instance was opened with `studiocycle open`, pid 2070598, still open in Edit), published nothing.
- The Play proves the CELL, not the game: no player-input, two-client or full-world claims (the validator's own scope line).

## 4 · State and what is next (needs Nox)
- Branch `prep/d1-closing-16-72-20261102` @ `a18c791` holds fix D + the ruled clamp + the Play-22-verified code; the lane branch `prep/b1-capture-format-20261101` @ `cbc9c02` has fix D and the pinned limit but NOT the clamp. **Nothing is merged** (per your ruling). A merge into the lane branch (and the A1/B1 chain) is yours to order.
- Studio is open and idle (pid 2070598, Edit, 1/3). Tell me whether to leave it for the next queue entry or `studiocycle close --discard`.
- Harness²: lesson recorded (`library/coding/entries/2026-11-02-offline-whatif-must-call-the-production-scorer`).
