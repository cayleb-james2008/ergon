# TEAM D — CAPTURE-RETRY FIX (layer 4), frozen 2026-11-02

Lead approval: 2026-11-02T04:10Z (Play 5 result accepted; layer-4 fix "exactly as proposed"). **Offline only — no Play.**
Machine `outis-desktop`, spend $0.
**Fix commit: `74cb299d6a6fbbb1020629a73af1bb53173af970`** (`74cb299`; parent `a231e3f` = accepted fog fix; branch tip;
tree `5caad7ada5cee68b6bc423848de46b383bfdf691`; tracked tree clean; **2 files +85/−7**). Sealed artifacts unchanged
(`84324d43…` release, `c6651793…` test/place). Log: `board/probes/team-d/capture-retry-fix-20261102.log`.

## 1 · Why (Play 5 layer 4)

On the fog-fixed code Play 5 captured **20/24 views** (20 distinct sha256, `viewportMatchesPng:true` on every view, fog
layer clean, `fastPathApplied=true`) and then failed fail-closed at **view-021 `highway-overhead`**:
`screen_capture returned the same bytes as an earlier view (stale capture)`. The client had acknowledged view 21 and was
holding the pose (console `ATLAS VIEW READY 21/24`); the 20 earlier captures are all distinct — the duplicate came from
the capture path, not from the pose plan. The stale guard was correct: no dishonest evidence was stored.

## 2 · What changed (runner/tooling only; assertion unchanged)

`tools/vinegar_run.py` `Atlas.run` per-view capture loop:
- On a capture whose bytes duplicate an earlier view, the runner records the attempt, waits **0.3 s**
  (`capture_retry_wait`), **re-probes** the camera and re-captures — **at most 3 attempts total (1 + 2 retries)**
  (`capture_attempts_max`).
- The **duplicate/stale rule remains the FINAL fail-closed outcome** (`… on all 3 attempts (stale capture; attempts
  recorded)`); the camera must still match the ack inside the retry window, otherwise a named fail-closed
  `camera moved during the capture retry (… > 1e-3); no capture accepted`.
- Every attempt is auditable: `result.json.captureAttempts`, `capture-attempts.json` (once any retry happened) and the
  affected view's sidecar `captureAttempts = [{attempt, sha256, duplicate, cameraDelta}, …]`.
- R2/R4 cosmetics folded (report/evidence wording; untracked evidence-dir count is pre-existing cosmetic).

## 3 · Tests (offline, runner suite +3 → 88)

1. **duplicate→fresh accepted**: view-002's first capture returns view-001's bytes → runner retries once, accepts the
   fresh capture; `result.json.captureAttempts[1]` = attempts `[{duplicate:true},{duplicate:false}]`,
   `capture-attempts.json` written, that sidecar carries both attempts, total `screen_capture` calls = 25 for 24 views.
2. **always-duplicate fails closed**: 3 attempts (1 + exactly 2 retries, **no loop beyond**), exit 1, message names
   `on all 3 attempts (stale capture; attempts recorded)`, only the first (unique) view was stored.
3. **camera moved during the retry**: while a duplicate is being retried the scenario camera leaves the ack pose →
   named `camera moved during the capture retry` fail-closed, no capture accepted.
Existing stale-capture test still passes (the final message still contains `stale capture`).
**Mutation proofs (maker self-check, each caught, restored):** M7 retry disabled (max 1) → FAILED (2); M8 retry bound
raised to 5 → FAILED (1); M9 camera-held check removed → FAILED (1).

## 4 · Freeze evidence (fresh checkout of `74cb299`)

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `74cb299…`; tracked tree clean (792 pre-existing untracked evidence-dir lines, cosmetic R2) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 88 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools` | **Ran 322 tests … OK** rc 0 |
| 4 | `./.tools/bin/lune run tests/unit/run.luau` | full suite on the fix tree (`board/probes/team-d/lune-unit-retryfix-20261102.log`) |
| 5 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 6 | `git show --stat 74cb299` | 2 files, +85/−7 (`tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`) |

**Exact checker reproduction:** `git checkout 74cb299d6a6fbbb1020629a73af1bb53173af970` fresh worktree → `git status`
empty; run rows 2–4 (no Play/Studio needed); sealed row 5. Contract spot-checks: `Atlas.capture_attempts_max == 3`,
`capture_retry_wait == 0.3`, duplicate path raises the named final message only after `attempt == max`, camera delta is
checked before every retry, and `result_trace()`/sidecars carry the attempt history. Mutations M7/M8/M9 fail as above.

## 5 · Next (per Lead/Nox sequencing)

On checker delta ACCEPT: next atlas Play in the current fresh opening (**1/3 used**; one Play → 2/3, third reserved) —
`preflight` → one `run atlas:scout --seed 2134783819` → `verify-clean`, sealed before/after, capture result/trace; the
retry path will be visible in `captureAttempts` if a duplicate occurs again. Then fixture **A+C** (closed-loop pursuit
`clamp(proxy+8,12,60)` + 40 studs/s² cap; >700/800 radius guard; 12 s no-touch timer; B only if A cannot close, with
exact bounded values; D rejected) as its own freeze → checker delta → Play B; third Play reserved.
