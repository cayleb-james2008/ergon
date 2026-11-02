# TEAM D — DIAGNOSTIC PLAY 20 REPORT: **B1 ATLAS CELL PASS** (the first live cell pass in this chain), 2026-11-02

Lead GO 2026-11-02T10:36Z (fix19 delta ACCEPT; "any fail → the day stays closed"). Current opening (Studio pid 995852,
Edit, 2/3 used, one Play left). State read-only + sealed BEFORE → `preflight` rc 0 → **one**
`run atlas:scout --seed 2134783819` → `verify-clean` → sealed AFTER. No extra Plays, no build/publication/image reads; $0.
Run: `docs/verify/2026-10-19/vinegar/20261102T153706Z-atlas-scout`; log
`board/probes/team-d/diagnostic-play20-20261102.log`.

## Verdict: **B1 ATLAS CELL PASS** — the pass gate is met in full, fail-closed criteria satisfied

- **Runner exit 0**; states `LOCKED → PREFLIGHT → STAGED 2.171 → PREPARED 2.28 → PLAYING 4.169 → RAN 7.002 →
  STOPPED 33.211 → CLEANED 38.261 → VALIDATED 40.134 → FINALIZED 40.345`; **validator passed**; **no crash**
  (`newCrashDumps:null`, `abort:null`); `verify-clean` clean; Studio alive pid 995852.
- **Pass gate — restoration:** client `restored:true`, **`cameraTypeRestored:true`**, and **every other restoration field
  true**: `sameCamera, subjectRestored, fovRestored, poseRestored, attributesRestored, uiRestored, menuGradeRestored,
  visualEffectsPreserved, noDriver, bindingsDisconnected, monitorDisconnected, focusBindingDisconnected`
  (`poseRequired:false` by design). `verdictRecomputed:true`; `reassertAttempted:true`, `reassertOk:true`,
  `afterReassertType:"Custom"`, `reassertFinalType:"Custom"`, `frameSeries` **all five frames `"Custom"`**.
- **Captures / audit (30/30):** 30 view sidecars, **30 distinct sha256**, **`viewportMatchesPng:true` on all**, all
  `accepted`/`stationary`; **`captureAttempts` 30 == `capture-attempts.json` 30**, **zero retries**; the **nine overhead
  views each settled 1.0 s once** (`view-020…024`, `view-027…030` — the highway-zone and world-region substitutions all
  captured); fog record `legacy:true`; technology `Future`.
- **World/identity:** `worldTrace.fastPathApplied:true` (actual==expected seed 2134783819 / layout 2, `rebuild:false`,
  starter applied before `rootFoundAt`); collector phases `prepared + view-001…030 + runtime + cleanup`, rejections `[]`.
- **Validator/index evidence:** `summary = {candidate:1f24da33…, buildSha256:c6651793…, staticCameraImages:30,
  qualifiedAnchorImages:27, qualifiedStaticImages:18, visualAcceptance:false (static camera inventory, by design),
  visualFlags:""}`; coverage index written (`coverage/index.html`) and originals bound to the build hash.
- **Sealed BEFORE = AFTER:** release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place
  `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` — unchanged. **Plays 3/3** in this opening (fully
  consumed; future Plays need a fresh `studiocycle` opening).

## Evidence chain (paths)

- Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T153706Z-atlas-scout`
  (`result.json`, `report.json`, `capture-attempts.json`, `originals.json`, `coverage/`, `views/001…030`).
- Fix that enabled it: `e475d4688f3243932105cf4a0fef0561564e2811` (verdict recompute after the deferred re-assert,
  one-shot reverter; fail-closed on genuine mismatch; validator/plan/runner untouched).
- Diagnostics chain: `team-d-diagnostic-play17/18/19.md`, `team-d-atlas-layer9/10/round3-freeze` reports; crash class (4
  intermittent userland Wine shutdown dumps, sealed unchanged each time) recorded in those reports.

## Notes / boundaries

- This is the **static camera inventory** cell (B1 atlas): `visualAcceptance:false` is the designed meaning of the cell —
  captures are inventory evidence, not visual acceptance. No claim beyond the summary is made.
- Opening is now at 3/3 Plays; remaining programme Plays need a fresh opening. **Abort-contact B is untouched and still
  sequenced after its own Nox ruling** (C evidence/hold recorded separately). No build/publication/credential/image reads.
