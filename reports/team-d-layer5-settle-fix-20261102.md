# TEAM D — LAYER-5 FIX: overhead render settle + failure-path capture-audit, frozen 2026-11-02

Lead approval: 2026-11-02T04:30Z (Play 6 accepted; layer-5 fix "exactly as proposed"). **Offline only — no Play.**
Machine `outis-desktop`, spend $0.
**Frozen fix commit (after the checker delta fix round): `fc546e582bc4e9729875b612e74b8302c8ad9704`** (`fc546e5`;
parent `ed9b797` = first layer-5 freeze; branch tip; tree `858225ecfa0563a21bc9e02bab1a691bec419b96`; tracked tree
clean; delta round **2 files +38/−10**). The first layer-5 commit `ed9b797` (+81/−10) was **REJECTED** by the checker on
one real defect (C1 below) and is superseded by `fc546e5`. Sealed artifacts
unchanged (`84324d43…` release, `c6651793…` test/place). Log: `board/probes/team-d/layer5-settle-fix-20261102.log`.

## 1 · Why (Play 6 — the crash class recorded honestly)

Play 6's bounded retry proved the view-021 duplicate is **persistent, not transient**: three attempts 0.3 s apart were
all byte-identical to **view-020 (`city-overhead`)** — the capture kept returning the previous overhead frame — while the
client had acknowledged view-021 (`view-021` in the collector phases). After the fail-closed stop (`abort.stop:"ok"`),
Studio **crashed**: new **userland minidump 106,116,768 B** (`2742f914-…dmp`, 2026-11-02T04:22:48.583Z), Studio log ends
right after `StopPlaySoloShutdownDataModels` begins, `cleanup`/`verifyClean` = `Disconnected('no response to tools/call
within 180s')`, `newCrashDumps:1`, exit **5**; **`gpuJournal:null`**, no kernel `amdgpu|MES|ring|GPU reset` lines,
hostcheck `{stalls 0, label "clean"}`; **sealed hashes unchanged through the crash and the `studiocycle open` recovery**
(`c6651793…`/`84324d43…`), which produced a fresh 0/3 opening (pid 814951). The defect is attributed to an overhead-frame
render/capture stall, not to retry logic.

## 2 · What changed

① **Overhead bounded render settle** — `tools/vinegar_run.py Atlas.run`: views whose plan `kind` is in
`("overhead","cell-roof","tile")` now wait a bounded **1.0 s settle** before the first capture (the camera is re-probed
after the settle) and use a **1.0 s bounded retry wait** (non-overhead keeps 0.3 s); still **max 3 attempts total**, and
the duplicate/stale rule remains the **final fail-closed outcome**.
② **Failure-path audit file** — `capture-attempts.json` is now written on the fail-closed paths too (camera-moved and
all-attempts-duplicate raises), not only after a successful retry (R3 nit folded), so the attempts are always on disk;
failure messages name the kind and settle (`… on all 3 attempts (stale capture; kind=overhead settle=1.0s; attempts
recorded)`).
③ **Audit shape** — entries are `{phase, kind, settledSeconds, attempts[{attempt, sha256, duplicate, cameraDelta}]}` in
`result.json.captureAttempts`; the per-view sidecar keeps `captureAttempts` when a retry happened.
R2/cosmetic nits folded into the report/evidence wording.
**Contingency prepared (NOT implemented):** `board/probes/team-d/overhead-render-limitation-contingency-20261102.md` —
if the settle does not unfreeze the frame: options A (documented fixture limitation + named view), B (per-zone
functional views replacing the single whole-highway overhead; plan/validator change), C (reduce framing cost), D (cap the
overhead set); recommended B+A; states plainly what the evidence supports (previous-frame persistence ≥0.6 s, userland
minidump, no kernel GPU fault) vs what is unknown (exact stall/crash mechanism, world-overhead behaviour).

## 3 · Tests (offline, runner suite +2 → 90)

1. **overhead settle recorded and bounded**: scout plan's three overhead views (phases 20–22) each record
   `kind:"overhead"`, `settledSeconds:1.0`; non-overhead entries record `0.0`; only the three 1.0 s settles are
   scheduled; **no audit file when no retry happened** (no noise).
2. **still-frozen overhead fails closed with kind+settle evidence**: a script that freezes view-020 to view-019's bytes
   → exit 1, message `view-020 … on all 3 attempts (stale capture; kind=overhead settle=1.0s; attempts recorded)`,
   audit entry `(kind:"overhead", settledSeconds:1.0, 3 duplicates)`, waits exactly settle+2×1.0 s (**bounded, never
   more**), `capture-attempts.json` on disk, views 1–19 stored.
3. **non-overhead retry keeps the short wait** (0.3 s), and **always-duplicate still fails after exactly 3 attempts**
   with the kind/settle message and the failure-path audit file.
**Mutation proofs (maker self-check, each caught, restored, tree clean):** M10 overhead settle disabled → runner suite
FAILED (3); M11 failure-path audit flush removed → FAILED (1 error); M12 kind/settled evidence dropped → FAILED (2 errors).

## 3b · Checker delta fix round (LAYER-5 REJECT → fixed) — `fc546e5`

**Defect (checker-layer5-delta, real):** `flush_capture_attempts` used an exclusive-create write and swallowed
`FileExistsError`, so once an earlier retry had written `capture-attempts.json`, a **later failure's flush was silently
skipped** — probe: `result.json.captureAttempts` had 20 entries ending view-020 while the file had 2 entries ending
view-002; on success-with-retry it was likewise a stale snapshot (2 vs 24).

**Fix (exactly as named):** the flush now **overwrites** the derived audit file with the **current complete list**
(it mirrors `result.json.captureAttempts` by design), and a real write error is raised, never swallowed. The flush runs
after every view's attempts are recorded, so the file can never be a stale snapshot. The camera-moved failure message
now also names `kind`/`settle` (optional item ③); R2 cosmetic folded into this report.

**Regression tests:** a new case combines an **early retry** (view-002 duplicate→fresh) with a **later failure** (frozen
overhead view-020) and asserts `capture-attempts.json == result.json.captureAttempts` by **length and content** (20
entries, last phase view-020). The success-with-retry (24/24), failure-only (2/2) and no-retry (24/24) cases now all
assert the same equality. **Mutation M13** (restore exclusive-create + swallow) → runner suite FAILED (5).

**Delta evidence on `fc546e5`:** runner **91 OK**; full Python **325 OK**; full Lune re-run; sealed unchanged; tracked
clean. Repro delta: `git checkout fc546e582bc4e9729875b612e74b8302c8ad9704`, rows 2–5 of §4, M13 mutation.

## 4 · Freeze evidence (fresh checkout of `fc546e5`)

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `ed9b797…`; tracked tree clean (792 pre-existing untracked evidence-dir lines, R2 cosmetic) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 91 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools` | **Ran 325 tests … OK** rc 0 |
| 4 | `./.tools/bin/lune run tests/unit/run.luau` | full suite on the fix tree (`board/probes/team-d/lune-unit-layer5-20261102.log`) |
| 5 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 6 | `git show --stat ed9b797; git show --stat fc546e5` | first freeze 2 files +81/−10 (rejected on the audit defect); delta 2 files +38/−10 |

**Exact checker reproduction (delta):** `git checkout fc546e582bc4e9729875b612e74b8302c8ad9704` fresh worktree → `git status`
empty; run rows 2–4 (no Play/Studio needed); sealed row 5. Contract spot-checks: `Atlas.overhead_kinds ==
("overhead","cell-roof","tile")`, `overhead_settle == 1.0`, `overhead_retry_wait == 1.0`, non-overhead
`capture_retry_wait == 0.3`; the failure raises call `flush_capture_attempts()` before raising; audit entries carry
`kind`/`settledSeconds`. Mutations M10/M11/M12 fail as above; M13 (stale-snapshot restore) fails (5).

## 5 · Next

On checker delta ACCEPT of `fc546e5`: **next atlas Play** on the recovered fresh opening (currently **0/3**; this Play would be 1/3,
one more stays reserved): `preflight` → one `run atlas:scout --seed 2134783819` → `verify-clean`, sealed before/after.
`captureAttempts` will show `kind:"overhead"`, `settledSeconds:1.0` and whether the post-settle frame was fresh. If it is
still frozen, apply the contingency ruling (note prepared, not implemented) — no plan change without the Lead/Nox ruling.
Fallback (a) for the named seed remains closed; fixture **A+C** (abort-contact) remains the next separate freeze; Play B
stays gated behind it; the reserved Play stays reserved.
