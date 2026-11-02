# DESIGN NOTE (proposal, NOT implemented) — `traffic:abort-contact` fixture geometry fix

Author: Team D (`roblox-lead`). Date: 2026-11-02. Offline only — **no Play, no code committed**. For the Lead's ruling.
Evidence: `board/reports/team-d-diagnostic-play.md` (Play 1, 2026-11-02T02:29:40Z, run dir
`docs/verify/2026-10-19/vinegar/20261102T022940Z-traffic-abort-contact`).

## 1 · What the diagnostic Play proved (input to this note)

- The abort-contact cell failed fail-closed with **one** bounded `generationEvent`: step 1386, generation 2→3,
  `bodyReplaced=true`, `newBodyAlive=false`, `bodiesCount=0`, `hasChange=false`, `contacts=0`, `touches=0`, at `t=23.233 s`.
- The probe's own 177-sample distance series explains it exactly: ego↔proxy distance 75.8 studs (t=0.12) → **45.8 at the
  abort (4.983 s)** → **23.9 when the contact drive began (9.003 s)** → then **monotonically rising** → first >800 at
  **t=22.283 s** → retirement **0.95 s later**. Speeds: proxy resumed traffic speed (10.3→**52.2** studs/s) while the ego
  never exceeded **17.1** studs/s.
- `TrafficController.step` retires a car when `distanceToPlayers(...) > RETIRE_RADIUS (800)` for `unobservedSince > 1 s`.
  The guard's event is therefore the controller's **documented** rule, not churn and not a defect. The fixture's
  contact-drive leg drove the sole observer (the ego is the player frame) away from the proxy, so the intended native
  contact never happened (`contacts:0`, `touches:0`, `contactObservedAt:null`) and the cell could not pass.
- Conclusion: the remaining cell defect is **fixture geometry**, not the controller and not the assertion.

## 2 · Goal of the change (fixture only — tests helper, no production code)

Make the abort-contact maneuver satisfy the controller's observation contract while still producing the intended
physics evidence: cut-in signal → abort/hold observed → the probe then **closes to a real contact** (Touched + scored
contact) inside the bounded window, with the ego staying within the observation radius at all times.

## 3 · Options considered

| Option | Change | Pros | Cons |
|---|---|---|---|
| **A. Closed-loop pursuit for the contact leg (recommended)** | After `abortObservedAt`, target the proxy's **current** lane position instead of the ego's own projection; set ego closing speed = clamp(proxy forward speed + closing margin, cap ~60) and keep applying the existing 40-stud/s² force cap | Directly fixes the cause (distance falls instead of rising); no production change; keeps the scenario's physics | Needs a small new acceleration helper; higher ego speed needs a stability check |
| **B. Bound the proxy's escape** | During the contact leg set the fixture-owned `car.commandSpeed`/`car.baseDesiredSpeed` lower (e.g. 12–15 studs/s) so the ego can catch it | Trivial; keeps current ego speed | Changes the scenario cadence (the proxy should behave like traffic); masks speed realism |
| **C. Named fixture guards (recommended, in addition to A)** | Fail fast with named messages if (i) ego↔proxy distance exceeds a safe threshold (e.g. 700 of 800) during the contact leg, or (ii) no touch occurs within N seconds after `contactDriveAt` | Future failures name the fixture contract, never the controller rule; no 60 s blind wait | None material |
| **D. Keep the proxy held/alive** | e.g. pin the probed car or add a fixture-owned observer near it | Guarantees no retire | Out of scope: would change what the cell tests (a held car is not the production rule) |

**Recommended: A + C, with B kept as a fallback** if A cannot close the gap inside the bounded window.

## 4 · Patch sketch (for review; not committed)

In `tests/traffic/Server.luau` (heartbeat, the `cfg.mode=="abort-contact"` leg that currently sets
`wantedLoop=source; wantedSpeed=-18` at `contactAt`):
1. Add a pure helper, e.g. `Server.pursuitAcceleration(loop, targetS, position, velocity, speed)` — same shape as
   `obstacleAcceleration` but the lane frame is `Sim.frame(loop.poly, targetS)` where
   `targetS = (Sim.project(source.poly, proxy.body.Position) + lead)` (`lead ≈ proxy forward speed × 0.15 s`), and the
   speed target is `clamp(proxySpeed + 8, 12, 60)`. Keep the existing 40-stud/s² cap and finite checks.
2. During the contact leg use `pursuitAcceleration` (keep `obstacleAcceleration` for the pre-abort ramp).
3. Guards (C): each heartbeat during the contact leg,
   - `if #report.contacts==0 and (ego.Position - proxy.body.Position).Magnitude > 700 then error("probe left the controller's observation radius (>700 of 800): contact leg unreachable") end`
   - `if contactAt and age-contactAt > 12 and #report.contacts==0 then error("contact leg never reached within 12 s of contactDriveAt") end`
   Both are named, fail-closed, and recorded in the runtime report.
4. Keep the existing evidence fields and ordering: `cutInAt ≤ abortObservedAt < contactDriveAt`, then
   `contactObservedAt` and `touches` populated; the validator's abort-contact branch is unchanged.

## 5 · Offline test list (all runnable without a Play)

1. **Lune spec** (extract the new helper from `tests/traffic/Server.luau` the way `tests/unit/traffic-timing.spec.luau`
   extracts its callback): pursuit acceleration points toward the proxy's lane position, is finite and capped; closing
   speed exceeds the proxy speed; the distance guard raises the exact named message above 700; the no-touch timer names
   its message.
2. **Python source-shape test** in `tests/tools/test_traffic_acceptance.py`: the probe uses the pursuit helper in the
   contact leg, contains both named guard messages, never mutates production car speed fields (or, if option B is ruled
   in, asserts the exact bounded values), and the abort-contact ordering asserts are unchanged.
3. **Synthetic distance-series test**: feed the Play-1 series (closing: 75→45→23; escaping: 23→…→800) through the guard
   logic — escaping must trip the 700 guard, closing must not; keep the Play-1 samples as a regression fixture.
4. Validator tests unchanged (the abort-contact branch must still require `contacts`/`touches` and the ordering).

## 6 · Risks, limits and what this does not do

- Touched contact is native physics; the pursuit must not tunnel through the proxy body (keep the 40-stud/s² cap and the
  existing samples/telemetry to prove approach).
- The scenario cadence changes (contact likely ~10–16 s instead of never); the 60 s bounded mode remains the hard stop.
- This note does **not** implement anything and does **not** claim the cell will pass; it needs the Lead's ruling, a
  freeze (tests included) and **one Play** to validate.
- Play budget: **1 Play left in this Studio opening (2/3 used)**; it competes with the atlas starter-fix Play. Recommend a
  `studiocycle` close/reopen (fresh 3-Play opening) once the Lead sequences both fixes.
