# ABORT-CONTACT — B AUTHORITY RESULT + HONEST FALLBACK (for the Nox ruling), 2026-11-02
Evidence: Play 21 (`docs/verify/2026-10-19/vinegar/20261102T214602Z-traffic-abort-contact`), B freeze `b73f6cd`;
Play 15 C evidence note `board/probes/team-d/abort-contact-c-achieved-acceleration-20261102.md`.

## 1 · What B proved (authority layer)

Bounded `LinearVelocity` assist at the contact leg, measured live: `assistMaxForce = 2642.128 = 40 × AssemblyMass 66.053`
(the ruled 40 studs/s² exactly), `assistVelocity = 60` (the ruled closing clamp), `VectorForce` drive **zeroed** during the
leg (egoForce 0.00 in samples), `fixture.contactLegAssist=true/contactLegMaxAccel=40`, released in cleanup
(`contactLegAssistReleased:true`), no CFrame/position/velocity writes, both guards/window/clamp untouched. The probe reached
and held the full commanded speed (60.00) — **the physics-authority deficit that Play 15 measured (command +40 vs achieved
−15…+8.5) is answered**.

## 2 · Why there is still no contact (gap direction is wrong)

Gap **72.1 → 167.7** studs over the leg; growth **7.9 studs/s == ego−proxy speed differential (60.00 vs 52.17)**. So the
probe is *moving away at the relative speed*, not closing: on this segment the probe travels along its facing while the
pursuit's lane-position convention (S increasing) runs **opposite**, so `target = proxyS + 0.15 s lead` is a point **behind**
the probe; the target-speed pursuit law then holds it at 60 away from the proxy. (Measured: `v·laneForward = −60` while the
assist commands +60 along the lane facing; same sign pattern explains Play 15's gap growth.) No contact, no touches; the
ruled 12 s guard fired honestly; radius guard not tripped.

## 3 · What was tried (all honest, all fail-closed)

A (pursuit + closing clamp), A+C (40-cap pursuit), C alone, option-B leg start (abortObservedAt), B-bounded gap trigger
(retired 0/3), option-B start + C (Play 15), B drive assist (Play 21). Every failure was named, guards fired as ruled,
sealed unchanged, no faked contact.

## 4 · Options needing a Nox ruling (NOT implemented; no silent changes)

1. **Leg-start/target-side geometry (fixture):** start the contact leg with the pursuit target ahead in the probe's travel
   direction (e.g. pick the leg-start point/segment where the lane's facing and S convention agree, or reverse the probe's
   start facing/travel so the chase is a real chase). Fixture-only, but changes the scenario start geometry.
2. **Scenario reconsideration (held fallback):** judge a bounded closing attempt / shorter scenario instead of physical
   contact. Changes the cell's acceptance meaning.
3. **Ruled-value changes (window/closing clamp) are not proposed** here because the gap grows monotonically at the relative
   speed — a longer window or higher clamp would not close a diverging gap, and any such change needs Nox explicitly.

**Recommendation:** option 1 first (geometry/direction), since B has already proven the authority layer; option 2 only if
the geometry cannot be made direction-consistent within the fixture contract. B1 cell pass, sealed hashes and the crash
chain stay untouched.
