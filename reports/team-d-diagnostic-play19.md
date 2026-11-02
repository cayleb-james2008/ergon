# TEAM D — DIAGNOSTIC PLAY 19 REPORT: round-3 diagnostics — cadence is a ONE-SHOT reverter; fix applied (2026-11-02)

Lead GO 2026-11-02T10:07Z (round-3 delta ACCEPT). Current opening (Studio pid 995852, Edit, 1/3). State read-only + sealed
BEFORE → `preflight` rc 0 → **one** `run atlas:scout --seed 2134783819` → `verify-clean` → sealed AFTER. No extra Plays.
Run: `docs/verify/2026-10-19/vinegar/20261102T150757Z-atlas-scout`; log
`board/probes/team-d/diagnostic-play19-20261102.log`.

## 1 · Result

Clean run, no crash: **30/30 views captured** (captureAttempts 30, zone + world-region overheads fresh), runtime ok,
validator reached its verdict: **exit 1 `runtime restoration missing`** (expected before the fix). Sealed unchanged;
plays **2/3** in this opening (1 reserved). The full round-3 diagnostics were captured.

## 2 · `cameraTypeDiagnostics` (verbatim)

```
savedType: "Custom"     beforeType: "Scriptable"   afterType: "Scriptable"   afterFrameType: "Scriptable"
typeWriteOk: true       typeWriteError: null       typeAfterWrite: "Custom"  typeAfterFullRestore: "Custom"
sameCamera: true        frameSameCamera: true      subjects: Humanoid/Humanoid (saved==current)
cameraTypeRestored: false   cameraTypeRestoredAfterFrame: false
reassertAttempted: true     reassertOk: true       reassertError: null
afterReassertType: "Custom" reassertFinalType: "Custom"
frameSeries: [{frame:1,Custom},{frame:2,Custom},{frame:3,Custom},{frame:4,Custom},{frame:5,Custom}]
```

## 3 · Cadence named: ONE-SHOT reverter (not per-frame)

The initial `CameraType=Custom` write lands (`typeAfterWrite`/`typeAfterFullRestore:"Custom"`) and is reverted to
`Scriptable` **once** before the delayed check; the deferred re-assert then sets `Custom` and it **holds across all five
frames** (`afterReassertType`/`reassertFinalType:"Custom"`, `frameSeries` all `"Custom"`). So the reverter is a
**one-shot** event during the restore window, not a per-frame actor — the pre-authorised fix applies.

## 4 · Fix (pre-authorised, own freeze)

**Commit `e475d4688f3243932105cf4a0fef0561564e2811`** (`e475d46`; parent `0ec48d6`; tree `a36ed97a…`; 2 files +29/−2):
after the deferred re-assert + 5-frame series, when the attempt succeeded, every frame holds the saved type and a fresh
`State.check` passes, the restoration verdict **details are recomputed from that fresh check** (menuGrade/effects/noDriver/
bindings/monitor/focus recomputed); otherwise the original failed details are kept → **fails closed on a genuine
mismatch**. `cameraTypeDiagnostics` gains `verdictRecomputed`; payload/validator unchanged. Residual pins folded
(`afterReassertType=` table entry, `typeWriteOk=`). **M35** (recompute reverted) → atlas acceptance **FAILED (1)**.
Suites: atlas acceptance 25 OK; runner 91 OK; full Python 334 OK; Lune atlas 11/0; full Lune re-run; sealed unchanged.

## 5 · Next

Fix delta on `e475d46` → on ACCEPT, **Play 20** is the atlas pass gate (`cameraTypeRestored:true` with every other field
true). Note the opening budget: Play 19 was the 2nd of 3 → **one Play left in this opening**; the Lead decides whether Play
20 uses it or a fresh opening is cycled first. Abort-contact B remains sequenced after; sealed unchanged.
