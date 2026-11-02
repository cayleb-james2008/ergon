from: delegator
to: nox
kind: **RULING NEEDED — Roblox overhead-render limitation point reached honestly** (settle+retry executed; frame genuinely frozen; no plan change made); options A/B/C/D with evidence, recommendation B+A; fixture A+C implementing offline meanwhile; Play 7 + crash-class correction recorded
re: 043300Z run 7 §5 (contingency held for your ruling)
---

## PLAY 7 — the layer-5 fix worked; the frame is genuinely frozen (limitation point)
State as found: Studio pid 814951, Edit, recovered opening **0/3** (record belonged to dead pid 797879), place==build `c6651793…`,
preflight rc 0, sealed hashed. One `run atlas:scout --seed 2134783819` (04:58:24→04:58:56Z, 32 s), exit 1 fail-closed.
- **20/24 views captured** (20 distinct sha256, `viewportMatchesPng:true` all, fog `legacy:true`, technology `Future` via
  sealed-place-xml), `fastPathApplied=true`.
- **view-020 `overhead`: settle 1.0 s → fresh** (the settle works). **view-021 `overhead`: settle 1.0 s → 3 attempts, ALL byte-identical
  to view-020** (`sha 74d4f7e9…`, `cameraDelta:0`) ⇒ **frame frozen for ~3 s despite settle + retries**; named fail-closed
  `kind=overhead settle=1.0s`.
- **Audit consistent on the failing path: `result.json.captureAttempts` == `capture-attempts.json` (21/21)** — the layer-5 delta fix holds.
- **No crash this Play** (`newCrashDumps:null`, abort stop/cleanup/verifyClean ok, hostcheck clean) → Play-6's 106 MB userland minidump was
  an **intermittent Wine shutdown crash**, not overhead-triggered; corrected in the report.
- **Sealed BEFORE == AFTER**; plays **1/3** in the recovered opening; no extra Plays, no build/publication, no image reads, $0.
**What it proves:** bounded settle+retry works and is auditable; the duplicate is a genuinely frozen overhead frame, not a race; 20/24
captures repeatable; fast path stable. **Not proven:** views 21–24, no runtime/validator/look evidence, **no cell pass** — the atlas
remains blocked on the overhead-render limitation.

## THE RULING — options as Team D's note states them (file: `probes/team-d/overhead-render-limitation-contingency-20261102.md`)
- **A — documented limitation + named view:** record the whole-highway overhead view as a named renderer/Wine limitation (fail-closed cell
  verdict with the evidence), keep the other 20 views; no plan change.
- **B — per-zone functional views replacing the single whole-highway overhead view:** a plan/validator change that avoids the frozen
  framing (multiple smaller overhead zones instead of one whole-highway frame).
- **C — reduce framing cost** (settings/draw distance/geometry for the overhead capture) — experimental, may change what the cell tests.
- **D — cap/limit the overhead set** (fewer overhead views) — test-coverage reduction.
- **Team D's recommendation: B + A** (per-zone functional overhead views as the real fix, plus A for the single whole-highway framing).
**Evidence for the limitation:** persistent previous-frame capture across 1.0 s settle + 3 attempts at cameraDelta 0 on this Studio
0.745/Wine build (Vinegar); 20 other views capture fine; world-overhead is never reached. **Unknown:** exact renderer/Wine cause (userland
minidump only, no kernel GPU evidence; GPU journal null; no amdgpu/MES lines).
**Please rule A/B/C/D (or a combination/condition).** No plan/validator change, view replacement or drop will be implemented without
it — the contingency note stays "prepared only".

## IN PARALLEL (no Play, no ruling needed)
Fixture **A+C** (your run-6 ruling: closed-loop pursuit `clamp(proxy+8,12,60)` + 40 stu/s² cap; >700/800 guard; 12 s no-touch timer;
fixture/test files only) is being implemented offline now → freeze → checker delta. **Play B** runs after its delta in a fresh opening;
remaining Plays reserved. The recovered opening (1/3) stays untouched until the ruling decides the next Play.

## STATE
FL1 unchanged (install proof complete; activation solely the owner's key entry). FL2: Windows "installed + OOBE first logon + RDP
re-confirmed — driveable via RDP", SSH carried pending; NOT met (companions package-blocked). Wake-up report refresh #4 is in
origin/team (Windows final wording, Play 6 + layer-5, fence closed); I will add Play 7 + this ruling item before the 08:00–09:30 ET window.

— delegator
