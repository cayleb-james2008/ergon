# TEAM D — DIAGNOSTIC PLAY 16 REPORT: atlas B2+A2 — 30/30 views captured (zones + world regions fresh), validator blocked on client camera-type restore (2026-11-02)

Lead GO 2026-11-02T08:24Z (after Play 15's crash killed its opening). Studio recovered with `studiocycle open`
(sealed place hash before==after `c6651793…`), latch cleared (`clean:true, latchCleared:true`), preflight rc 0, Edit, 0/3.
**One** `run atlas:scout --seed 2134783819` → `verify-clean`; no extra Plays. Run:
`docs/verify/2026-10-19/vinegar/20261102T082650Z-atlas-scout`; log `board/probes/team-d/diagnostic-play16-20261102.log`.

## 1 · Verdict

**The B2+A2 substitution works in Play — the cell captured everything for the first time:** **30/30 planned views**
captured, **30 distinct sha256**, **`viewportMatchesPng:true` on every view**, all `accepted`/`stationary`;
**`captureAttempts` 30 entries == `capture-attempts.json` 30**; the **nine overhead views settled 1.0 s each with a
single attempt and zero retries** — including the four **highway-zone** views (ordinals 21–24) and the four
**world-region** views (ordinals 27–30), i.e. the previously frozen whole-map framings are now captured through bounded
per-zone/per-region views; `worldTrace.fastPathApplied=true`; **runtime ok=true**; collector phases `prepared + 30 views
+ runtime + cleanup`; **no crash**; sealed unchanged; plays 1/3. The **only** failure is the validator's
`runtime restoration missing`: the client's final cleanup reports **`cameraTypeRestored:false`** (so `restored:false`)
while every other restoration check is true. This is a **new layer** — the client camera-type restore path was never
reached before (all earlier atlas Plays failed at capture), so this is its first live exercise.

## 2 · Evidence

| Fact | Value |
|---|---|
| Sealed BEFORE/AFTER | `84324d43…`/`c6651793…` — unchanged |
| Run | 08:26:50Z → 08:27:34Z (~44 s); exit **1** (validator); states `LOCKED → … → RAN 6.496 → STOPPED 32.241 → CLEANED 37.082 → VALIDATED 38.965`; `abort:null`, `newCrashDumps:null` |
| Views | **30/30**, 30 distinct sha256, `viewportMatchesPng` true all, `accepted` all, `stationary`; `viewsAcknowledged:30` |
| Audit | `captureAttempts` 30 == file 30 (**consistent**); 9 overhead entries `(view-020/021/022/023/024/027/028/029/030, kind overhead, settle 1.0, 1 attempt)`; **0 retries** |
| Zone/world captures | views 21–24 (`highway-zone-1..4-overhead`) and 27–30 (`world-region-1..4-overhead`) all fresh — the B2 substitution is exercised |
| Runtime | `runtime.ok=true`; `worldTrace.fastPathApplied=true` (actual==expected 2134783819/2, rebuild=false); chunks `null` |
| Validator failure | `runtime restoration missing` → client cleanup `restored:false`, **`cameraTypeRestored:false`**; all other fields true (`sameCamera, subjectRestored, fovRestored, poseRestored, attributesRestored, uiRestored, menuGradeRestored, visualEffectsPreserved, noDriver, bindingsDisconnected, monitorDisconnected, focusBindingDisconnected`) |
| Studio/post | alive pid **973232**; `verify-clean` clean; plays **1/3** in the recovered opening (2 reserved) |

## 3 · Judgement / next named layer

- **What it proves:** the B/E2+A2+ordering plan is PLAY-EXECUTABLE: 30/30 captures, all zone + world-region overheads
  fresh, audit consistent, fast path, fog/viewport records, sealed integrity; the renderer limitation is retired for the
  substituted framings by evidence. **Not proven:** no cell pass — the validator blocks on the client camera-type restore
  flag.
- **New named layer (layer 9): client restoration check `cameraTypeRestored:false`.** `CameraState.restore` sets
  `Camera.CameraType=saved.type` and immediately compares; every other restore field passes, so the likely cause is the
  assignment not taking effect / being reasserted in the same tick (candidate: the default PlayerModule camera script
  reasserting the custom camera state, or an enum/no-op assignment). The mechanism is **not diagnosed** yet.
- **Proposed offline next step (needs GO):** instrument the client cleanup payload with the concrete values
  (`saved.type`/`current.CameraType` before/after restore, plus one `RenderStepped` wait + re-check) — a small
  fixture-side diagnostic change, offline freeze, then one atlas Play to name the cause; only then a real fix (either
  restore path fix or an evidence-honest check adjustment if the camera script legitimately reasserts).
