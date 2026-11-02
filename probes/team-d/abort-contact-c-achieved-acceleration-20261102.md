# ABORT-CONTACT — FIX C EVIDENCE: commanded 40 studs/s² is not achieved in live physics (Play 15)
Status: evidence note only (Lead filing to Nox); **nothing implemented**. Source: Play 15, run
`docs/verify/2026-10-19/vinegar/20261102T082424Z-traffic-abort-contact` (option-B leg start + fix C frozen at `1eed7e7`).

## 1 · What was commanded vs achieved

- Ruled/commanded at leg start: pursuit forward clamp is `-38,40` (fix C), target speed 60, ego speed 29.66 studs/s →
  `clamp((60-29.66)*2,-38,40) = **+40.0 studs/s²**`.
- **Achieved** (samples, dt-resolved): **-15.14 … +8.55 studs/s²**, i.e. the probe first **decelerates** (speed 29.7 → 18.5
  in ~1 s) and then accelerates at only **~8.5 studs/s²** (max), not 40. The ego reaches 59 studs/s only at t≈12.8 s after
  the leg start, and the gap grows **71.6 → 141.9**; the 12 s window expires at t=15.78 with no contact (touches 0).
- **Geometry fact:** the probe enters the leg with its velocity **opposite its own heading** — sampled
  `v · ego.LookVector = -33.17` with |v| = 33.18 — i.e. it is travelling backwards/oncoming relative to its facing, so the
  pursuit's first phase is a realignment/braking phase, not a clean chase. Yaw rate is ~0 (no steering; straight line).
- Commanded cap unchanged (40) and the total force magnitude cap (40) is unchanged; nothing in the ruled law/guards was
  altered by this evidence.

## 2 · Honest reading

The kinematic model used for the C pre-approval (ramp at 40 studs/s², contact <12 s) **does not translate into vehicle
acceleration**: the live system delivers ~1/5 of the commanded acceleration, plus a deceleration phase at entry. So C's
clamp is not the limiting factor in Play; the limiting factor is the **force→acceleration authority and/or the leg-start
velocity/heading alignment**, which the fixture does not currently control. This is a new layer, not a guard/law defect.

## 3 · Options (each needs a Nox ruling; none implemented)

1. **Authority investigation (fixture/physics):** find why `Force = acc * AssemblyMass` yields ~20% of the commanded
   acceleration (mass/force application/ground-friction/assembly composition) and make the ruled 40 studs/s² reachable;
   changes what the cell exercises (physics contract), not the ruled cap/window/guards.
2. **Leg-start alignment (fixture/phase):** start the contact leg with velocity and heading aligned with the pursuit lane
   (or choose the leg start when the probe is already chasing), avoiding the backwards-entry braking phase; fixture-only,
   but changes the scenario's start geometry (the 12–25 target and abort timing stay as ruled).
3. **Window/scenario reconsideration:** A (12→20 s) alone does not fix a ~142-stud gap with ~8.5 studs/s² achieved
   authority; a bounded-scenario rule (e.g. judge a bounded closing attempt rather than physical contact) would change the
   cell's acceptance meaning — Nox call.
4. (rejected combination): C alone is now disproven in Play; leaving it in place with a documented limitation would keep
   the cell red.

## 4 · Crash class (recorded, not verdict-affecting)

This Play hit the **3rd intermittent userland Wine Studio shutdown crash** (Play 6: 106,116,768 B; Play 13: 101,859,264 B;
Play 15: 101,847,584 B `023c0d5b-…`), each **after** the runtime result was posted, no kernel GPU evidence
(`gpuJournal:null`, no amdgpu/MES/ring lines), hostcheck clean each time. Sealed artifacts (`84324d43…`/`c6651793…`) were
unchanged through every crash; recovery via `studiocycle` (sealed hash guarded) works; **the crash does not change the
verdicts or the sealed integrity**; it is a data point for the Studio/Wine stability class, not the cell's failure cause.
