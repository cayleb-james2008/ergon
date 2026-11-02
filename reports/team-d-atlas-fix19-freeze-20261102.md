# TEAM D — PLAY-19 FIX FREEZE: verdict recompute after the deferred re-assert, 2026-11-02

Pre-authorised fix (Nox run-11 chain) after Play 19 named the cadence as a **one-shot reverter**. Client cleanup only;
the ruled guards/plan/validator/runner and sealed place are untouched. **Commit
`e475d4688f3243932105cf4a0fef0561564e2811`** (`e475d46`; parent `0ec48d6`; tree `a36ed97a517173aed279219425b45f659c1dbbb7`;
tracked clean; **2 files +29/−2**); sealed `84324d43…`/`c6651793…` unchanged. Log:
`board/probes/team-d/atlas-fix19-freeze-20261102.log`; Play-19 report: `board/reports/team-d-diagnostic-play19.md`.

**Change:** in `tests/atlas/Client.luau restore()`, after the deferred re-assert and its 5-frame `frameSeries`, if the
attempt succeeded, **every series frame holds the saved CameraType**, and a **fresh `State.check`** passes, the verdict
details are replaced by that fresh check (menuGrade/effects/noDriver/bindings/monitor/focus recomputed at that point) and
`verdictRecomputed=true`; otherwise the original (failed) details are kept — **fail closed on genuine mismatch**. Payload
`restored=checkAll(details)`/`cleanup=details` still reference only the verdict details; `cameraTypeDiagnostics` gains one
field.

**Tests:** fix pins (recompute block, fresh check, fail-closed condition + `verdictRecomputed`), residual pins folded
(`afterReassertType=` recorded entry, `typeWriteOk=` in `CameraState`), validator tolerance unchanged. **M35** (recompute
reverted) → atlas acceptance **FAILED (1)**. Suites: atlas acceptance **25 OK**; runner **91 OK**; full Python **334 OK**;
Lune atlas **11/0 (4 specs)**; full Lune re-run on the fix tree; sealed unchanged. **Next:** fresh-checker delta on
`e475d46` → Play 20 (pass gate: `cameraTypeRestored:true` with every other field true); the current opening has one Play
left after Play 19, so the Lead decides Play-20 budget vs a fresh opening.
