from: delegator
to: nox
kind: **ATLAS BREAKTHROUGH — Play 16: 30/30 views captured incl the zone AND world-region overheads; only client camera-type restore (layer 9) remains**; abort-contact (i): C ineffective in live physics (achieved ~5 vs commanded 40 studs/s²; gap grew; guards honest) — **new ruling needed**; both deltas had been checker-ACCEPTED first
re: 072500Z/073317Z run-10; checker deltas; Plays 15/16
---

## 1. ATLAS — PLAY 16: THE SUBSTITUTED PLAN CAPTURES EVERYTHING (one layer from a cell pass)
Both deltas were fresh-checker **ACCEPT** first (`checker-abort-i-delta`, `checker-b2-delta2` — M24 now fails, B-bounded fully reverted,
C applied to the pursuit only with function-block shas unchanged). Then Play 16 (fresh opening, one Play):
- **30/30 planned views captured — 30 distinct sha256, `viewportMatchesPng:true` on all**, all accepted/stationary; **`captureAttempts`
  30 == `capture-attempts.json` 30**; **9 overhead views each settled 1.0 s, one attempt, 0 retries — including highway zones 21–24
  and world regions 27–30**; the previously frozen whole-map framings are now captured through the bounded substituted views (the
  named limitation is retired by evidence for those framings). `worldTrace.fastPathApplied=true`; **runtime ok=true**; collector
  prepared + 30 views + runtime + cleanup; **no crash**; sealed unchanged; plays 1/3 (2 reserved); Studio alive.
- **Only failure:** validator `runtime restoration missing` → client cleanup **`cameraTypeRestored:false`**; every other restoration
  field is true (sameCamera/subject/fov/pose/attrs/UI/menuGrade/effects/noDriver/bindings/monitor/focus). **New layer 9: the
  client camera-type restore path — first live exercise ever** (all earlier atlas Plays failed before capture).
- **What it proves:** B2+A2+ordering is Play-executable and the substituted framings capture fresh; the capture/audit/fast-path layers
  are all clean; **no cell pass yet** (camera-type restore only).
- **Approved and running:** offline instrumentation freeze (record `saved.type` + post-restore `Camera.CameraType` with one
  `RenderStepped` wait + re-check in the client cleanup payload, tests; no behaviour change) → checker delta → **one atlas Play** to
  name the cause before any real fix. If it names a one-line restore defect, the next Play could be the cell's first pass.

## 2. ABORT-CONTACT (i) — C IS INEFFECTIVE IN LIVE PHYSICS (new ruling needed)
Play 15 (recovered opening, one Play): leg start exactly as wired (`abortObservedAt = contactDriveAt = 3.7789 s`, offset 80,
pre-leg 30); **12 s guard fired honestly**, 0 contacts/touches; contactLeg latched at distance 142.36, proxy 52.14, speed 60.
**Fix C evidence:** despite commanding the ruled 40 studs/s² clamp, the ego's **achieved** acceleration was **~5 studs/s²** — it
**decelerated 29.7 → 19.8 at leg start** and the gap grew **71.6 → 141.9**, closing never happened. **C's kinematic ramp does not
translate to vehicle authority; the limiter is traction/steering/physics, not the pursuit command.** Then the **3rd intermittent Wine
shutdown crash** (101.8 MB userland dump, no kernel GPU evidence); sealed BEFORE==AFTER unchanged; recovered via `studiocycle open`
+ latch clear (opening consumed 2/3, recovered 1/3 with 2 reserved).
**Options for your ruling (nothing implemented; Team D preparing an evidence note with the achieved-vs-commanded series):**
- **(A) window 12→20 s** — cannot close a *growing* gap; only helps if the ego can hold closing speed once at speed (it can't yet).
- **(B) fixture/physics change for the contact leg** — give the ego authority consistent with its physics (e.g. adjust the contact-leg
  vehicle setup/drive assist or start geometry so the ruled pursuit command can actually be executed), keeping the ruled 12 s window,
  40 cap, closing clamp and guards untouched.
- **(C) scenario reconsideration** — if a chase-contact is not physically achievable in this fixture with the product traffic
  behaviours, restate the cell (e.g. a contact geometry the ego can reach) — a bigger, honest question.
**Please rule (A/B/C or a combination), and state whether a physics-authority finding is acceptable as the cell's named limit if no
fixture change makes contact achievable.**

## 3. STATE
FL1 unchanged (activation solely the owner's key entry; held for the morning report). FL2 settled (RDP driveable; SSH unavailable with
blocker; no reinstall). Wake-up report refresh #7 will fold in Play 15/16 before the 08:00–09:30 ET window. Monday bookings active
(07:45/09:00/16:15); Monday readiness green. Crash class: 3rd intermittent Wine shutdown dump — recorded, sealed integrity held each
time, no product implication. No spend, no image reads, no publication, desks untouched.

— delegator
