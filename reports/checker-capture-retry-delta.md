# CHECKER — layer-4 capture-retry delta (`74cb299`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the bounded fresh-capture retry on top of my accepted fog-guard delta (`a231e3f`). I
built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway clone
`/tmp/checker-retry/repo` (2026-11-01T23:15:23-05:00) and a second clone for mutations; maker tree only read.

- **Fix commit:** `74cb299d6a6fbbb1020629a73af1bb53173af970` (`74cb299`), parent `a231e3f` (my last accepted delta),
  branch `prep/b1-capture-format-20261101` tip, tree `5caad7ada5cee68b6bc423848de46b383bfdf691`; tracked mods **0**
  (untracked 802 evidence lines — cosmetic, R2); sealed `84324d43…`/`c6651793…` unchanged before/after and at end.
- **Delta:** exactly **2 files, +85/−7** — `tools/vinegar_run.py` (+35/−6), `tests/tools/test_vinegar_run.py` (+50/−1).
  No `src/`, `assets/`, `build/` change; `tests/atlas/{Client,Server,WorldSeed}.luau` and `tools/atlas_acceptance.py`
  untouched → 45 s bound, assert-and-skip, guarded fog and no-rebuild path intact.

## Verdict in one line

**ACCEPT** — a duplicate-bytes capture is retaken after a 0.3 s wait with the camera re-probed, at most **3 attempts
total (1 + 2 retries)**, the duplicate/stale rule remains the final fail-closed outcome (named message after the last
attempt), a camera that leaves the ack pose inside the retry window fails closed with no capture accepted, and every
attempt is recorded (`result.json.captureAttempts`, `capture-attempts.json` once a retry happens, per-view sidecar
`captureAttempts`). Three offline tests prove duplicate→fresh, always-duplicate (exactly 3 attempts, no loop) and
camera-moved; mutations raising the cap, disabling the retry, and removing the camera-held check fail exactly those
tests (1 / 2 / 1). The delta is pure and suites are green (88 / 20 / 322 / atlas-paths 1-0).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Bounded retry in `Atlas.run` | **PASS** | `tools/vinegar_run.py:617-618`: `capture_attempts_max = 3` (one capture + at most 2 retries), `capture_retry_wait = 0.3`; loop `for attempt in range(1, capture_attempts_max+1)`: re-probe `before`, capture, re-probe `after`, sha, `duplicate = any(c["sha256"]==sha for c in self.captures)`, camera delta vs ack; `attempts.append({attempt, sha256, duplicate, cameraDelta})`; `if not duplicate: break`; duplicate + camera moved → `capture_attempts` recorded and named `camera moved during the capture retry …; no capture accepted`; duplicate on the last attempt → recorded + named `screen_capture returned the same bytes as an earlier view on all N attempts (stale capture; attempts recorded)`; else `ctx.sleep(0.3)`. After the loop: attempt history appended to `self.capture_attempts`, `capture-attempts.json` written once any retry happened, `row["captureAttempts"] = attempts` when >1 attempt, and `result_trace()` merges `captureAttempts` into `result.json`. No unbounded loop; a duplicate is never accepted |
| ② Offline tests (read + run) | **PASS** | `test_atlas_duplicate_capture_retries_once_and_keeps_the_attempts`: capture script {1:1, 2:1, 3:2} → view-002 duplicate then fresh → exit 0, 24 sidecars, sidecar attempts `[duplicate:true, duplicate:false]`, `result.captureAttempts[1].phase=="view-002"`, `capture-attempts.json` present, **25 `screen_capture` calls** (24 + exactly one retry). `test_atlas_always_duplicate_capture_fails_closed_after_max_retries`: `same_png=True` → exit 1, error contains `on all 3 attempts (stale capture; attempts recorded)`, `captureAttempts[1].attempts` all duplicate, 4 calls (view-001 once + view-002 exactly 3), only view-001 stored. `test_atlas_camera_moved_during_capture_retry_fails_closed`: duplicate view-002 first attempt moves the camera → exit 1, error `camera moved during the capture retry`, attempts `[duplicate:true]`, only view-001 stored. Existing `test_atlas_stale_identical_capture_is_rejected` still fails closed and still contains "stale capture". Runner suite `Ran 88 tests … OK` |
| ③ Delta scope / invariants | **PASS** | Exactly the 2 claimed files; no game/asset/sealed path; 45 s bound (`ATLAS_FIRST_VIEW_TIMEOUT = 45`) untouched; assert-and-skip (`WorldSeed.requireApplied`, no `generateSafe(`), guarded fog and no-rebuild path untouched (files unchanged) |
| ④ Suites (fresh clone) | **PASS** | runner `Ran 88 tests … OK` (11.827 s); atlas acceptance `Ran 20 … OK` (0.738 s); full Python `Ran 322 tests … OK` rc 0 (48.726 s); `atlas-paths` `1 passed, 0 failed`. **Full Lune not re-run**: the delta touches only Python (`tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`), no Lune-read file (`tests/unit/`, `src/`, `tests/atlas/*.luau`) changed — relied on the accepted `920 passed, 0 failed (126 specs)` at `a231e3f` |
| ⑤ Mutations (throwaway clone, restored) | **PASS** | **M8** `capture_attempts_max = 5` → runner `FAILED (failures=1)` (`always_duplicate` expects 3 attempts / 4 captures / "on all 3 attempts"); **M7** `= 1` (retry disabled) → `FAILED (failures=2)` (`always_duplicate` + `duplicate_retries_once`); **M9** camera-held check removed → `FAILED (failures=1)` (`camera_moved_during_capture_retry`). Restored `tools/vinegar_run.py` sha256 `b1874c5b2758…`, git clean |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged. Play 5 evidence (20/24 distinct captures, `viewportMatchesPng:true`, client holding pose at view 21) corroborates the duplicate-capture cause; the retry path will be visible in `captureAttempts` if it recurs in the authorized Play |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play, so the retry path is proven offline (fake capture script + fake studio) only; the next
  atlas Play is the live exercise (and will show `captureAttempts` if a duplicate recurs).
- **R2 — cosmetic:** code comment typo `capture_attempts_max = 3  # launch March: …` (meaning "at most 2 retries");
  report says "792 pre-existing untracked evidence-dir lines", tree now has 802. No functional impact.
- **R3 — minor evidence nuance:** `capture-attempts.json` is written only when a retry happened (success path); on the
  fail-closed paths (always-duplicate, camera-moved) the attempts are kept in `result.json.captureAttempts` only. The
  claim says "sidecar/report", so this is met (report), but if file-level failure evidence is wanted, write the file
  before the raises too (one-line move).
- **R4 — observation:** `result.json.captureAttempts` carries one entry per view (single-attempt views included), not
  only retried views; tests index the retried view. Not a defect.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-capture-retry-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
45 s/assert-and-skip + retry implementation + tests + Lune-read scope), `suites.log`, `runner.log` (88 OK), `atlas.log`
(20 OK), `python.log` (322 OK), `atlas-paths.log` (1/0), `mutations.log` (M8 1, M7 2, M9 1; restored hash),
`run-started.txt`. Throwaway copies `/tmp/checker-retry/{repo,mut}`; maker repo read-only, unchanged, still at `74cb299`.
