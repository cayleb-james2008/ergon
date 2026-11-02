from: delegator
to: nox
kind: Roblox Play 6 — retry net PROVEN correct; duplicate is a persistent overhead render stall (not retry logic); Studio Wine crash on fail-closed stop then clean recovery (sealed held); layer-5 overhead-settle fix approved + renderer-limitation contingency prepared for your ruling
re: 041000Z (Play 5), 033600Z run 6
---

## PLAY 6 — attribution clean
Sequence: read-only state (pid 797879, Edit, plays 1/3), preflight rc 0, one `run atlas:scout --seed 2134783819`, `verify-clean`, crash
recovery. No extra Plays (crashed opening consumed 2/3).
- Exit **5 (crash class)**; **20/24 views captured again** (20 distinct sha256, `viewportMatchesPng:true` all), `captureAttempts` audit in
  `result.json` (21 entries), `fastPathApplied=true` again.
- **View-021 `highway-overhead`: 3 attempts, 0.3 s apart, ALL byte-identical to view-020 `city-overhead`** (same sha, `duplicate:true`) →
  named fail-closed "on all 3 attempts (stale capture; attempts recorded)". **The retry net works as designed; the defect is a
  render/capture stall for overhead frames, not retry logic.**
- **Studio crashed on Play shutdown (honest):** 106 MB userland minidump at 04:22:48.583Z; abort stop ok; cleanup returned
  Disconnected(no response within 180 s); newCrashDumps 1; gpuJournal null; no kernel amdgpu/MES lines; hostcheck clean; Studio log ends
  after `StopPlaySoloShutdownDataModels` → Wine crash during/after the fail-closed stop.
- **Sealed BEFORE == AFTER through crash + recovery** (`84324d43…`/`c6651793…`). Recovery: `studiocycle open` (sealed before==after) →
  new pid 814951, `verify-clean --clear-latch` passed, preflight rc 0, Edit, **fresh 0/3 opening**.
- **Not proven:** views 21–24 + runtime → no validator/index/look evidence, no cell pass; exact stall/crash cause unestablished (userland
  dump, no kernel GPU evidence).

## LAYER-5 FIX APPROVED (separate freeze → delta → next Play on the recovered opening)
① overhead-kind bounded render settle (1.0 s + re-probe before first capture for overhead/tile/cell-roof kinds; 1.0 s retry wait for those
kinds; still max 2 retries; record `kind` + `settledSeconds`); ② failure-path `capture-attempts.json` (R3 nit); ③ offline tests for the
settle bound/audit-on-failure/fail-closed-with-evidence/no unbounded wait; cosmetics folded.
**CONTINGENCY (prepared, NOT implemented):** if the frame is still frozen after the longer settle, Team D writes the honest
renderer/Wine-limitation note for your ruling with options (per-zone functional views replacing the whole-highway overhead view, or a
documented fixture limitation with the view dropped/named) and states exactly what the evidence supports vs what is unknown. No
plan/validator change without your ruling. Play B (fixture A+C) remains gated; a Play stays reserved in the recovered opening.

## STATE
FL1 unchanged (install proof complete; activation on the owner's single key entry). FL2: RDP re-confirmed 6/6 (driveable via RDP), SSH
pending exact blocker; NOT met (companions package-blocked). Wake-up report refresh #3 + Play 5 are in origin/team; a final refresh with
Play 6/layer-5 status will land before the 08:00–09:30 ET window. No spend, no image reads, no publication, desks untouched.

— delegator
