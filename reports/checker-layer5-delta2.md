# CHECKER — layer-5 delta fix (`fc546e5`) — verdict: **ACCEPT**

Fresh-context re-verify of the audit-file fix on top of the rejected layer-5 tree (`ed9b797`). I built none of it. No
Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway clone `/tmp/checker-layer5fix/repo`
(2026-11-01T23:49:48-05:00) and a second clone for the mutation; maker tree only read.

- **Fix commit:** `fc546e582bc4e9729875b612e74b8302c8ad9704` (`fc546e5`), parent `ed9b797` (the rejected layer-5 tree),
  branch `prep/b1-capture-format-20261101` tip, tree `858225ecfa0563a21bc9e02bab1a691bec419b96`; tracked mods **0**
  (untracked 805 evidence lines — cosmetic, R2); sealed `84324d43…`/`c6651793…` unchanged before/after and at end.
- **Delta:** exactly **2 files, +38/−10** — `tools/vinegar_run.py` (+9/−8), `tests/tools/test_vinegar_run.py` (+29/−2).
  No `src/`, `assets/`, `build/` change; `tests/atlas/{Client,Server,WorldSeed}.luau` and `tools/atlas_acceptance.py`
  untouched → 45 s bound, assert-and-skip, guarded fog, retry cap and overhead settle intact.

## Verdict in one line

**ACCEPT** — `flush_capture_attempts` now **overwrites** the derived audit file with the current complete list (no
exclusive-create, no swallowed `FileExistsError`, real write errors raise), it is called after every view's attempts are
recorded and on both fail-closed raises, and my independent probe reproduces the previously failing scenario exactly:
early retry (view-002) + later frozen overhead failure (view-020) → `capture-attempts.json` == `result.json.captureAttempts`
(20/20, length and content), and the success-with-retry (24/24) and no-retry (24/24) cases also match. The new
regression test pins this; mutation M13 (restore exclusive-create + swallow) fails exactly 5 tests. Delta is pure and
suites are green (91 / 20 / 325 / atlas-paths 1-0).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① `flush_capture_attempts` overwrites current complete list, no swallow | **PASS** | `tools/vinegar_run.py:656-665`: docstring states the derived-mirror contract; body is now `if not self.capture_attempts: return; (out / "capture-attempts.json").write_text(json.dumps(self.capture_attempts, indent=2) + "\n")` — no `write()` exclusive create, no `except FileExistsError`; a real `write_text` error propagates. Call sites: `:728` (camera-moved raise), `:735` (all-duplicate raise), `:741` **after every view's attempts are appended** — so the file is always the current complete list, never a stale snapshot. Camera-moved message now also names `kind=`/`settle=` (`:729-731`) |
| ② The exact failing scenario (early retry + later failure) and success-with-retry | **PASS** | Independent probe (`audit-consistency-probe.json`): (a) `early_retry_then_later_failure` (view-002 duplicate→fresh, view-020 frozen overhead) → exit 1, result 20 entries ending `view-020`, audit 20 entries ending `view-020`, **`equal:true` (length AND content)**; (b) `success_with_retry` → exit 0, 24/24, `equal:true`; (c) `no_retry_success` → 24/24, `equal:true`. New regression `test_atlas_audit_file_equals_result_after_an_early_retry_then_a_later_failure` asserts exactly this; the success-with-retry, failure-only and no-retry tests now all assert audit == result as well |
| ③ Delta scope / invariants | **PASS** | Exactly the 2 claimed files; no game/asset/sealed path; `ATLAS_FIRST_VIEW_TIMEOUT = 45`, `capture_attempts_max = 3`, `overhead_settle = 1.0`, `overhead_retry_wait = 1.0` intact; Client/Server/WorldSeed/atlas_acceptance untouched (assert-and-skip, guarded fog, no-rebuild path intact) |
| ④ Suites (fresh clone) + mutation | **PASS** | runner `Ran 91 tests … OK` (12.444 s); atlas acceptance `Ran 20 … OK` (0.743 s); full Python `Ran 325 tests … OK` rc 0 (48.166 s); `atlas-paths` `1 passed, 0 failed`. Mutation **M13** (restore exclusive-create write + swallow `FileExistsError`) → runner **FAILED (failures=5)**: `always_duplicate`, `audit_file_equals_result_after_an_early_retry_then_a_later_failure`, `duplicate_capture_retries_once`, `overhead_views_take_a_bounded_render_settle`, `still_frozen_overhead…`; restored `tools/vinegar_run.py` sha256 `cae90e62d7dc…` (frozen), git clean. **Full Lune not re-run** (delta Python-only; no Lune-read file changed) — relied on accepted `920/0 (126 specs)` at `a231e3f` |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged; maker HEAD `fc546e5`, tracked clean. The next atlas Play (Play 7) is the live test of the settle and the audit trail |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the 1.0 s overhead settle effect and the live audit file are proven offline only (the
  probe + tests). Play 7 on the recovered opening is the live proof.
- **R2 — cosmetic:** report says "792 pre-existing untracked evidence-dir lines"; tree now has 805. No commit impact.
- **R3 — behaviour note:** the audit file is now written after every view (even when no retry happens), which is what
  makes it complete; the old "no audit file noise without a retry" behaviour is intentionally gone and the tests assert
  the new contract. 24 small writes per run — negligible.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-layer5-delta2/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
invariants + overwrite flush + call sites + camera-moved message + regression test), `audit-probe.py` +
`audit-consistency-probe.json` (three cases, all `equal:true`), `suites.log`, `runner.log` (91 OK), `atlas.log` (20 OK),
`python.log` (325 OK), `atlas-paths.log` (1/0), `mutations.log` (M13 → 5 failures; restored hash `cae90e62d7dc…`),
`run-started.txt`. Throwaway copies `/tmp/checker-layer5fix/{repo,mut}`; maker repo read-only, unchanged, still at
`fc546e5`.
