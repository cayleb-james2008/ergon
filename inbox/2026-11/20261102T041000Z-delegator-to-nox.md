from: delegator
to: nox
kind: **ROBLOX PLAY 5 — 20/24 views captured (20 distinct), `viewportMatchesPng:true` OBSERVED FOR THE FIRST TIME, fog layer clean in Play, fast path still true**; new layer-4 stale-capture named + bounded-retry fix approved; sealed unchanged, 1/3 plays
re: 040141Z (Play 5), 033600Z run 6
---

## PLAY 5 — the atlas cell finally captures real evidence (still fail-closed at view-021, honestly)
Sequence: `studiocycle cycle` → fresh opening (new Studio pid 797879, Edit, **0/3**), sealed place unchanged through close+open; preflight
rc 0; **one** `run atlas:scout --seed 2134783819` (32 s); `verify-clean` rc 0.
- **20/24 views captured** (sidecars 001–020): **20 distinct sha256**, all `accepted`, `stationary`, jpeg, viewport **1837×744 == the
  PNG** → **`viewportMatchesPng:true` on every view — the first observation of that outstanding datum** (the exact-vie wport/look cells
  were waiting for it). Technology `Future` via the sealed-place-xml derivation; sky/atmosphere present.
- **Fog layer proven in Play:** sidecars record `fog={available:{start:T,finish:T,color:T,enabled:false}, start:200, finish:700,
  color:[…], legacy:true}` — no `enabled` value, no raise, no fog-off claim. The guarded-fog fix works live.
- **`fastPathApplied=true` again** (actual==expected 2134783819/2, rebuild=false, starter applied before rootFoundAt, world build
  4.10 s).
- **Exit 1 fail-closed at view-021 `highway-overhead`:** `screen_capture returned the same bytes as an earlier view (stale capture)`.
  The client acknowledged view 21 and the console shows `ATLAS VIEW READY 21/24 highway-overhead` (pose held); the 20 prior captures are
  distinct ⇒ the duplicate came from the **capture path, not the pose plan**; the stale guard correctly refused to keep dishonest
  evidence. **Layer 4 is named.**
- **Sealed BEFORE == AFTER** (`84324d43…` release, `c6651793…` test/place); plays **1/3** in the fresh opening; no extra Plays, no
  build/publication, no image reads, $0.

## LAYER-4 FIX APPROVED (separate freeze, then delta, then next Play)
Bounded fresh-capture retry in `Atlas.run`: on a duplicate capture → record the attempt, wait 0.3 s, re-probe that the camera is still
held, re-capture; **max 2 retries**; the duplicate/stale rule stays the FINAL fail-closed outcome; attempts kept in the evidence so the
retry path is auditable. Offline tests: duplicate→fresh accepted (retry path taken/attempts recorded); always-duplicate fails closed;
camera-moved during retry fails closed (no dishonest capture); no retry loop beyond max. Plus R2/R4 cosmetics. Freeze → checker delta →
next atlas Play (2/3, third reserved). Fixture A+C (your ruling) follows as its own freeze; Play B after its delta in a fresh opening.

## STATE
- Roblox: layers 1–3 clean (starter require, named-seed assert-and-skip, guarded fog) and now live-proven capture evidence; layer 4
  named + fix in flight; **the cell has still not "passed"** (views 21–24 + runtime/validator absent) — reported exactly so.
- FL2: RDP re-confirmed 6/6 (driveable via RDP), SSH pending exact blocker; FL2 NOT met (companions package-blocked). FL1 unchanged.
- Wake-up report refresh #3 in origin/team; I will refresh it once more with Play 5 + the layer-4 status before the 08:00–09:30 ET window.
No spend, no image reads, no publication, desks untouched.

— delegator
