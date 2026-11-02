# TEAM D — DIAGNOSTIC PLAY 18 REPORT: extended camera-restore diagnostics — write lands, value reverted within ~2 frames; re-assert diagnostic skipped (2026-11-02)

Lead GO 2026-11-02T09:45Z (extended-diagnostics delta ACCEPT). Fresh recovered opening (pid 995852, 0/3). State read-only
+ sealed BEFORE → `preflight` rc 0 (Edit, 0/3, place==build) → **one** `run atlas:scout --seed 2134783819` →
`verify-clean` → sealed AFTER. Clean run (no crash). Run:
`docs/verify/2026-10-19/vinegar/20261102T094547Z-atlas-scout`; log `board/probes/team-d/diagnostic-play18-20261102.log`.

## 1 · Result

**30/30 views captured again** (captureAttempts 30, all zone + world-region overheads, 9 × 1.0 s settles), runtime ok,
**cleanup phase present**, and the validator reached its verdict: **exit 1, `runtime restoration missing`** — client
`restored:false`, `cameraTypeRestored:false`, all other restoration fields true. **No crash this Play** (`abort:null`,
`newCrashDumps:null`; states `RAN → STOPPED → CLEANED → VALIDATED`); sealed unchanged; plays **1/3**; Studio alive pid
995852. The extended diagnostics were captured in full.

## 2 · Extended `cameraTypeDiagnostics` (verbatim)

```
savedType: "Custom"            beforeType: "Scriptable"
typeWriteOk: true              typeAfterWrite: "Custom"          typeAfterFullRestore: "Custom"
afterType: "Scriptable"        afterFrameType: "Scriptable"
cameraTypeRestored: false      cameraTypeRestoredAfterFrame: false
sameCamera: true               frameSameCamera: true
savedSubjectName/Class: "Humanoid"/"Humanoid"    currentSubjectName/Class: "Humanoid"/"Humanoid"
reassertAttempted: false       reassertOk: null   reassertError: null   afterReassertType: "Scriptable"
```

## 3 · Mechanism named (and the diagnostic gap stated honestly)

- **The `CameraType = Custom` write LANDS**: `typeWriteOk:true`, the **immediate read-back inside the write site is
  `"Custom"`**, and it is **still `"Custom"` at the end of `State.restore`** (`typeAfterFullRestore:"Custom"`). The Camera
  instance is the same before/after (`sameCamera:true`, `frameSameCamera:true`) and the subjects are a real Humanoid on
  both sides.
- **But by the delayed check (~2 `RenderStepped` frames later) the property is `"Scriptable"` again**
  (`afterType`/`afterFrameType`/`cameraTypeRestoredAfterFrame`). So this is **category (b): the write lands, then a
  frame-based actor reasserts `Scriptable` before the restoration check.**
- **The deferred re-assert diagnostic did NOT run** (`reassertAttempted:false`): its condition was
  `typeAfterWrite ~= savedType` (both are `"Custom"`, so it was skipped); `afterReassertType:"Scriptable"` is merely the
  initialised `afterFrameType`, not an attempted write. This is an instrumentation limitation, not evidence about the
  re-assert.
- **Not** category (a) (no rejection — the write succeeded and held through the function) and **not** category (c) (the
  check does not disagree with a stale object; the property genuinely reverts between frames).

## 4 · Judgement / next required diagnostic (nothing changed yet)

The cell state is now precise but **not one-line-fixable**: something frame-based resets `CameraType` to `Scriptable`
within ~1–2 frames of a successful `Custom` write. Required next step (offline, diagnostics-only, then delta → Play):
1. make the **deferred re-assert unconditional when `details.cameraTypeRestored == false`** (the accepted attempt, now
   actually exercised) and record `reassertAttempted/Ok/afterReassertType`;
2. record a **5-frame `CameraType` series** after the re-assert (and its timestamps vs `RenderStepped` count) to see
   whether the reverter runs once or every frame;
3. fold the checker's **5 pin additions** residual into the same freeze.
If the delayed re-assert then holds `Custom`, the likely one-line fix is to perform the `CameraType` restore **after a
frame** (deferred restore) and recompute the verdict honestly; if the value flips every frame, the next diagnostic must
identify the actor (frame-series + per-frame ownership) before any fix. Plan/validator untouched; no Plays beyond this
one; Studio alive (pid 995852, Edit, 1/3 used, 2 reserved); sealed unchanged.
