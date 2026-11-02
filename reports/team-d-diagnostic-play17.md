# TEAM D — DIAGNOSTIC PLAY 17 REPORT: layer-9 diagnostics Play — camera-type restore write does not take (cause narrowed), 30/30 captured, 4th Wine crash (2026-11-02)

Lead GO 2026-11-02T09:02Z (layer-9 delta ACCEPT). Current recovered opening (pid 973232, 1/3). State read-only + sealed
BEFORE → `preflight` rc 0 (Edit, 1/3, place==build) → **one** `run atlas:scout --seed 2134783819` → Studio crashed
during cleanup; recovered via `studiocycle open` + latch clear. No extra Plays.
Run: `docs/verify/2026-10-19/vinegar/20261102T090304Z-atlas-scout`; log
`board/probes/team-d/diagnostic-play17-20261102.log`.

## 1 · Result

**All 30/30 views were captured again** (30 view sidecars, `captureAttempts` 30, incl. the 4 highway-zone and 4
world-region overheads; 9 × 1.0 s settles), and the **runtime payload with `cameraTypeDiagnostics` was posted**; then
**Studio crashed** during cleanup (`abort:{"skipped":"Studio gone"}`; exit 5; new userland minidump `0f919081-…`
102,975,600 B @09:03:39Z) so the cleanup phase and validator never ran. The diagnostics answer the restore question
precisely enough to rule out the wrong-target and late-read-back cases — but **not** enough to name a one-line defect, so
**nothing was changed** (per the Lead's stop rule).

## 2 · `cameraTypeDiagnostics` (verbatim)

```
savedType: "Custom"        beforeType: "Scriptable"
afterType: "Scriptable"    afterFrameType: "Scriptable"
cameraTypeRestored: false  cameraTypeRestoredAfterFrame: false
sameCamera: true           frameSameCamera: true
```
`cleanup` (all other fields true): `sameCamera, subjectRestored, fovRestored, poseRestored, attributesRestored, uiRestored,
menuGradeRestored, visualEffectsPreserved, noDriver, bindingsDisconnected, monitorDisconnected, focusBindingDisconnected`;
`restored:false`, `viewsAcknowledged:30`, `poseRequired:false`.

## 3 · Cause named (symptom precise, mechanism narrowed)

- **Not category (a) wrong target:** `savedType` is `"Custom"` — a valid, correct restore target.
- **Not category (b) late read-back:** `afterType` is already `Scriptable` immediately after `State.restore` returns, and
  `afterFrameType` is still `Scriptable` one rendered frame later (`cameraTypeRestoredAfterFrame:false`).
- **Not category (c) post-restore overwrite by another script:** the read-back is wrong *within the same call*, before any
  frame boundary; `sameCamera:true` shows the same Camera instance; `frameSameCamera:true` shows it was not replaced.
- **Therefore:** the `Camera.CameraType = saved.type` write performed by `CameraState.restore` **does not take effect at
  all** in this context (immediate read-back is still `Scriptable`). The mechanism is unresolved; the two surviving
  candidates are (i) the write being rejected/ignored because of the camera state at restore time (e.g. `CameraSubject`
  state or default-camera ownership at the moment of the switch), or (ii) something synchronously reasserting
  `Scriptable` in the same tick that the diagnostics cannot see from inside `restore` (the assignment order is subject →
  fov → CameraType → CFrame → Focus, and the check runs after all of them).
- The handling fixture's working restore (`tests/handling/Client.luau` line ~126: `CameraSubject` then `CameraType` then
  FOV) differs from the atlas path only in the surrounding hooks (`focusOwner` BindToRenderStep released just before, two
  RenderStepped waits before the check, CFrame/Focus written after CameraType) — a comparison worth one more diagnostic
  field, not a blind fix.

## 4 · Crash class (recorded, not verdict-affecting)

**4th** intermittent userland Wine Studio shutdown crash (Play 6 106,116,768 B; Play 13 101,859,264 B; Play 15
101,847,584 B; Play 17 102,975,600 B), each **after** the runtime result was posted, no kernel GPU evidence
(`gpuJournal:null`), hostcheck clean; sealed `84324d43…`/`c6651793…` unchanged through every crash; recovery via
`studiocycle open` works and the latch cleared cleanly (`clean:true, latchCleared:true`). **Does not change the verdicts or
sealed integrity.**

## 5 · Next (NOT a one-line fix; stop and report)

No code changed. Proposed minimal next diagnostic (needs GO): extend the (already diagnostics-only) payload with
`subjectName`/`subjectClass` (saved and current), an explicit `pcall(function() camera.CameraType=Custom end)` success flag
and an immediate read-back **inside** the write site, plus one deferred re-assert attempt — that would discriminate
"write rejected" vs "same-tick reassert" without touching behaviour. Current state: Studio recovered (pid **995852**,
Edit, fresh **0/3**), place==build, sealed unchanged; atlas chain paused here per the Lead's rule.
