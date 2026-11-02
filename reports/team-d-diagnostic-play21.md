# TEAM D — DIAGNOSTIC PLAY 21 REPORT: abort-contact B — authority PROVEN, closing still impossible (direction/geometry), STOP + fallback (2026-11-02)

Lead GO 2026-11-02T16:44Z. Fresh opening: `studiocycle cycle` 21:44:33Z (close `killed-after-dialog`, nothing saved;
place sha unchanged) → reopened **pid 3953752**, window `0x558c7fa23f00`, 2200×1000, sealed place before==after
`c6651793…`. Host IO pressure briefly refused preflight (`io full avg10 14.54 > 5.0`, rc 2); waited for quiet (2.36),
refit window, preflight rc 0 (Edit, 0/3), sealed BEFORE hashed, then **one** `traffic:abort-contact` (21:46:02→21:46:30Z)
→ `verify-clean`. Run: `docs/verify/2026-10-19/vinegar/20261102T214602Z-traffic-abort-contact`; log
`board/probes/team-d/diagnostic-play21-20261102.log`. **No code changed in this Play.**

## 1 · Result: honest FAIL — but the authority layer is now proven

- Exit 1; runtime failed with the ruled guard: `HandlingServer:226: contact leg never reached within 12 s of
  contactDriveAt` (leg started at `abortObservedAt 3.77995`; guard at t=15.780). **touches 0 / contacts 0**; no crash;
  `verify-clean` clean; sealed AFTER unchanged; **plays 1/3**; Studio alive.
- **B drive assist worked and is evidenced:** `fixture.contactLegAssist=true`, `contactLegMaxAccel=40`;
  `contactLeg.assistMaxForce = 2642.128` = **40 × ego mass 66.053** (the ruled 40 studs/s² cap, exactly);
  `assistVelocity = 60` = the ruled closing clamp (proxy 52.166 + 8 → clamp 60); the `VectorForce` drive was **zeroed for
  the whole leg** (`egoForce` samples 0.00); cleanup reported `contactLegAssistReleased:true`; no CFrame/position/velocity
  writes (no-maneuver contract intact).
- **But no closing:** gap **72.1 studs at leg start → 167.7 at the guard**; ego reached and held **60.00** studs/s, proxy
  **52.17**; gap growth **7.9 studs/s == the speed differential** (7.83). The probe is **ahead in its direction of travel**
  relative to the lane-position convention the pursuit uses, so "target = proxy lane position + 0.15 s lead" is a point
  **behind the probe**, and the pursuit law (target-speed based) keeps it at 60 away from the proxy. Commanded vs achieved
  at the latch: `commandAccel 12`, `achievedAccel 0` (speed saturated); the speed ramp 29.6 → 60 was still slow (~8 s),
  i.e. the line constraint held the target speed but the probe's velocity is opposite the S-increasing direction
  (`v·laneForward` = **−60** while `LineVelocity` is +60 along the lane facing).

## 2 · Reading (ruled order: achieved acceleration and gap direction first, contact second)

1. **Achieved acceleration / authority:** improved to the intended channel — bounded assist active, force cap exact,
   force drive zeroed, probe at full commanded speed and held; the assist released cleanly. No cheat path.
2. **Gap direction:** still **opening** (72→168) at exactly the speed differential → the failure is now
   **geometry/travel-direction**, not authority: the contact-leg pursuit is driving the probe away because the lane's
   position convention runs opposite its facing/travel on this segment, so the "chase" target is behind it.
3. **Contact:** none; the 12 s guard fired honestly; radius guard not tripped (168 << 700); no faked contact.

## 3 · STOP + fallback (per the ruling)

B cannot produce closing in this geometry. Per the Lead/Nox rule: **stop, file the honest fallback, no faked contact, no
guard changes**. Fallback note: `board/probes/team-d/abort-contact-b-authority-result-20261102.md` — it records the
authority result, the achieved-vs-commanded evidence, what was tried (A, A+C, C, B-bounded, C+option-B start, B drive
assist) and the options that need a Nox ruling (leg-start/target-side geometry so the pursuit target is ahead in travel;
or the held scenario reconsideration; any closing-clamp/window change would be a ruled-value change and is NOT proposed
silently).

## 4 · State / preservation

Sealed `84324d43…`/`c6651793…` unchanged; B1 Play-20 record, hashes, diagnostics chain and the four intermittent Wine
shutdown dumps intact; Studio alive pid 3953752, Edit, **1/3** used (2 reserved). No extra Plays, no build/publication/
image reads; $0.
