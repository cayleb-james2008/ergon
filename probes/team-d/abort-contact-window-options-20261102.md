# ABORT-CONTACT WINDOW OPTIONS (offline note for a Nox ruling — NOT implemented) — 2026-11-02

Author: Team D. Evidence: `board/reports/team-d-diagnostic-play9.md`, run
`docs/verify/2026-10-19/vinegar/20261102T055035Z-traffic-abort-contact`. Fixture A+C frozen at `1e8b5b7` (delta
ACCEPTED). **Do not implement any option without the ruling.**

## 1 · The evidence in one line

The pursuit closes (peak **121.5 studs** at t=14.33 s → **79.6** at t=20.98 s, ~**6.7 studs/s** at the window end; ego
**59.06** vs proxy **52.17 studs/s**), but the ruled **12 s no-touch timer** expires first; the gap is still ~80 studs.
The ramp costs ~8 s because the pursuit inherits the shared law's **+10 studs/s² longitudinal clamp** (total force cap 40
unchanged), so the proxy pulls ~120 studs ahead before the chase reaches speed. Guards behave (timer fired, radius guard
correctly not tripped); generation guard stable (no retirement churn).

## 2 · Options (each with exact values; none implemented)

**(A) Extend the no-touch window** (ruled guard-value change → needs Nox)
`contactLegGuard` timer 12 s → **20 s** (or 22–25 s). Bounded: the leg then has 20 s; at the observed ~6.7 studs/s closing
from 79.6 at t≈21 s, contact is expected ~12 s later (≈t 33 s), so **20–25 s** covers it with margin. Radius guard (700
of 800) unchanged. Cost: longer Play (traffic fixture runtime grows to ~35–40 s; still bounded); the documented "fails
fast" property weakens from 12 s to 20–25 s.

**(B) Adjust the fixture start geometry so contact is reachable inside the ruled 12 s** (fixture-only)
Make the contact leg start with the ego already near traffic speed and the proxy near: e.g. begin the abort-contact drive
while the ego is still rolling (the current pre-contact leg commands `wantedSpeed=-18` / then a 4 s pause, so ego enters
the contact leg at ~9 studs/s) and/or start the contact leg from the closest pre-leg point (~12.5 studs at 8.47 s). Exact
fixture-only change: keep the ruled pursuit law/guards, but abolish the 4 s abort wait before `contactDriveAt` (start the
leg at `abortObservedAt` + 0 s) and remove the `-18` reverse so the ego preserves its rolling speed. Estimated: gap ~12–25
studs and ego ~28–36 studs/s at leg start → contact well inside 12 s. Cost: changes the scenario timing the cell
exercises (must be stated in the cell's description; no production change).

**(C) Raise the pursuit's longitudinal acceleration clamp up to the unchanged 40 studs/s² cap** (law detail; may or may
not be a ruled-value change → needs Nox confirmation)
Current: `acc=forward*math.clamp((speed-velocity:Dot(forward))*2,-38,10)+…; if acc.Magnitude>40 then acc=acc.Unit*40 end`
— the **+10** clamp is inherited from the obstacle-avoidance law, so the pursuit can only add 10 studs/s² forward even
though the ruled **40 studs/s² cap** allows more. Change the pursuit's forward upper clamp **10 → 40** (total magnitude
still capped at 40; lateral term unchanged). Estimated: ramp 9 → 60 in ~1.3 s instead of ~8 s; the proxy then leads by
~35–50 studs when the chase is at speed, and at ~8 studs/s closing (60 vs 52) contact is expected within ~5–7 s → **inside
the ruled 12 s window**, no guard values changed. This is the cheapest change and leaves A's ruled closing clamp
`clamp(proxy+8,12,60)`, the 40 cap and both guards untouched.

## 3 · Recommendation

**(C) first** (pursuit forward clamp 10 → 40 inside the already-ruled 40 studs/s² cap; one-line change in
`Server.pursuitAcceleration`, same tests/guards), because it targets the actual limiting factor without changing any ruled
value or the scenario timing. If a Play still misses inside 12 s, **(A) 20 s window** next (bounded guard-value change);
**(B)** only if the ruling prefers the scenario geometry change over the guard value. All three stay offline until the
Nox ruling; the Play budget will be re-sequenced by the Lead after that (the fresh opening has 2 Plays left after Play 9;
the final atlas Play uses one).
