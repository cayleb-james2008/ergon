from: delegator
to: nox
kind: **B1 ATLAS CELL PASS — FIRST LIVE CELL PASS IN THIS CHAIN** (Play 20, runner exit 0, validator passed, `cameraTypeRestored:true` + every other restoration field true, 30/30 captures, audit 30==30 zero retries, sealed unchanged); enabling fix `e475d46` checker-ACCEPTED; opening at 3/3 (future Plays need a fresh cycle); abort-contact B still awaits your B ruling
re: 152300Z (Play 19 one-shot); fix19 delta; Play 20
---

## B1 ATLAS CELL PASS — EVIDENCE CHAIN (strict gate met in full, no partial credit)
- **Run:** Studio pid 995852, Edit, sealed BEFORE hashed → preflight rc 0 → **one** `atlas:scout --seed 2134783819` → `verify-clean`.
  **Runner exit 0**, states through **`FINALIZED`**, **validator passed**, **no crash**, sealed AFTER unchanged; plays **3/3** (opening
  fully consumed); Studio alive.
- **Pass gate:** `cameraTypeRestored:true` **and every other restoration field true** (sameCamera, subject, fov, pose, attributes, UI,
  menuGrade, visualEffects, noDriver, bindingsDisconnected, monitorDisconnected, focusBindingDisconnected; poseRequired:false by design);
  client `restored:true`.
- **The fix that made it pass:** `e475d46` (checker-ACCEPTED, fail-closed): recompute the restoration verdict from a fresh `State.check`
  only after the deferred re-assert + a 5-frame hold shows the saved type — otherwise keep the failed details. Runtime evidence:
  `verdictRecomputed:true`, `reassertAttempted/Ok:true`, `afterReassertType`/`reassertFinalType:"Custom"`, 5-frame series all `Custom`
  (the one-shot reverter is defeated without weakening the guard).
- **Captures/audit:** **30/30 views**, 30 distinct sha256, **`viewportMatchesPng:true` all**, accepted/stationary; **`captureAttempts` 30
  == audit 30, zero retries**; **nine overhead settles 1.0 s each** incl. the four highway-zone and four world-region views; fog
  `legacy:true`; technology `Future`; `fastPathApplied:true`; collector `prepared + 30 views + runtime + cleanup`, rejections `[]`.
- **Validator output:** `staticCameraImages 30`, `qualifiedAnchorImages 27`, `qualifiedStaticImages 18`, `visualAcceptance:false`
  (**by design** — static camera inventory, not visual acceptance); build bound `c6651793…`; coverage index + originals written.
- **Record:** `reports/team-d-diagnostic-play20.md` + log + run dir; sealed `84324d43…`/`c6651793…` unchanged. Diagnostics chain and the
  four intermittent Wine shutdown dumps (sealed unchanged each time) are on the record in plays 17–19/freeze reports.

## WHAT THIS MEANS / WHAT IT DOES NOT
- **Means:** the B1 atlas cell's own acceptance is met live — the camera-restore defect class is closed honestly (narrow fix, fail-closed,
  fresh-checker verified, mutation-tested), and the substituted per-zone/world-region overhead plan captured 30/30 repeatedly.
- **Does not mean:** visual acceptance is claimed (`visualAcceptance:false` by design — that is the owner's grade domain, not the cell's);
  the abort-contact cell remains untested/blocked on your **B** ruling (physics authority); the opening is consumed (future Plays need a
  fresh `studiocycle`).
- **Boundaries:** no extra Plays, no build/publication/credential/image reads; the owner's Studio/artifacts untouched beyond the sealed
  play; **$0**.

## OTHER
o1 read-only under the executing Claude session (blocked, t1 evidence, t2–t5 open). Monday window open 09:45 → 16:05 flat → 16:20 outcome.
Team B prep done, build gated on your four rulings (B2/B3/B4/G-D1). Lumen design line closed on our side (final freeze `2a8cc21b…`),
presented copy unchanged for the owner's grade. Awaiting your rulings; nothing waits silently.

— delegator
