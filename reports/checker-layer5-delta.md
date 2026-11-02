# CHECKER — layer-5 overhead-settle delta (`ed9b797`) — verdict: **REJECT** (smallest fix named)

Fresh-context delta re-verify of the overhead render settle + failure-path audit on top of my accepted capture-retry
delta (`74cb299`). I built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a
throwaway clone `/tmp/checker-layer5/repo` (2026-11-01T23:38:38-05:00) and a second clone for mutations; maker tree only
read.

- **Fix commit:** `ed9b797ffe553ec7cc09bae02012ea2e2be153c9` (`ed9b797`), parent `74cb299` (my last accepted delta),
  branch `prep/b1-capture-format-20261101` tip, tree `1ddd75fd5f1c5ca0860c0d5ce49eb4f2aaa09862`; tracked mods **0**
  (untracked 805 evidence lines — cosmetic, R2); sealed `84324d43…`/`c6651793…` unchanged before/after and at end.
- **Delta:** exactly **2 files, +81/−10** — `tools/vinegar_run.py` (+36/−9), `tests/tools/test_vinegar_run.py`
  (+45/−1). No `src/`, `assets/`, `build/` change; `tests/atlas/{Client,Server,WorldSeed}.luau` and
  `tools/atlas_acceptance.py` untouched → 45 s bound, assert-and-skip, guarded fog, retry cap intact.

## Verdict in one line

**REJECT** — the overhead settle (claim 1), scope (4) and suites (5) all verify, and the audit file is now written on
fail-closed paths when it does not already exist (claim 2 first half), **but the claim that the attempts are
"complete/consistent with `result.json.captureAttempts`" is false**: `flush_capture_attempts` writes with the runner's
exclusive-create `write` and swallows `FileExistsError`, so once an earlier retry has written the file, a later
fail-closed write is silently skipped and `capture-attempts.json` is a stale snapshot. My probe (early retry at
view-002 → later frozen overhead failure at view-020): `result.json.captureAttempts` = 20 entries ending `view-020`,
`capture-attempts.json` = **2 entries ending `view-002`**; the same file is also a snapshot (not complete) on the
success-with-retry path. Smallest fix below is one line plus one regression test.

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Overhead-kind bounded settle | **PASS** | `tools/vinegar_run.py:617-623`: `capture_attempts_max = 3`, `capture_retry_wait = 0.3`, `overhead_kinds=("overhead","cell-roof","tile")`, `overhead_settle=1.0`, `overhead_retry_wait=1.0`. In `Atlas.run` (`:704-740`): `kind = view.get("kind")`, `overhead = kind in overhead_kinds`, `if overhead: ctx.sleep(1.0); settled=1.0` **before the attempts loop** (loop re-probes `before`/`after` around each capture), retry wait `overhead_retry_wait if overhead else capture_retry_wait`; still max 3 attempts, duplicate/stale remains the final fail-closed outcome; audit entries carry `{phase, kind, settledSeconds, attempts[...]}`; other kinds keep settle 0.0 / wait 0.3. Tests: `test_atlas_overhead_views_take_a_bounded_render_settle` (3 overhead phases 020-022 record kind overhead / settledSeconds 1.0, others 0.0, waits exactly `[1.0,1.0,1.0]`, no audit file without a retry); `test_atlas_duplicate_capture_retries_once…` asserts waits `[0.3,1.0,1.0,1.0]` and view-002 kind `junction`/settle 0.0; no unbounded wait |
| ② Failure-path audit + completeness/consistency | **FAIL (first half PASS, consistency FAIL)** | `flush_capture_attempts` (`:656-663`) is called in the camera-moved and all-duplicate raises (and after a successful retry), so the file is written on fail-closed paths **when absent** — `test_atlas_still_frozen_overhead_fails_closed_with_kind_and_settle_evidence` passes (audit on disk, last entry view-020, views 1..19 stored). But the write is `write(out/"capture-attempts.json", …)` (exclusive create) with `except FileExistsError: pass`, so a later flush cannot update an existing file. Probe (`audit-consistency-probe.json`, early retry at view-002 + frozen overhead at view-020): exit 1 with the named stale message, `result.captureAttempts` = 20 entries ending view-020, `capture-attempts.json` = **2 entries ending view-002** (`consistent_lengths: false`); on the success-with-retry path the file is likewise a 2-entry snapshot while result has 24 entries. The file therefore does not contain the failing view's attempts — "complete/consistent" is not met |
| ③ Offline tests (read + run) + mutations | **PASS for the shipped scenarios; composite case missing** | Runner `Ran 90 tests … OK`; the three retry tests + two new layer-5 tests pass. Mutations in a throwaway clone (restored): **M10** `overhead_settle=0.0` → `FAILED (failures=3)` (`duplicate_retries_once`, `overhead_views_take_a_bounded_render_settle`, `still_frozen_overhead…`); **M11** failure-path audit flushes removed → `FAILED (errors=1)` (`still_frozen_overhead…` errors on missing `capture-attempts.json`). No test combines an early retry with a later failure, which is the case that exposes ② |
| ④ Delta scope / invariants | **PASS** | Exactly the 2 claimed files; no game/asset/sealed path; `ATLAS_FIRST_VIEW_TIMEOUT = 45`, `capture_attempts_max = 3`, assert-and-skip (`WorldSeed.requireApplied`, no `generateSafe(`), guarded fog and no-rebuild path untouched (files unchanged) |
| ⑤ Suites (fresh clone) | **PASS** | runner `Ran 90 tests … OK` (12.506 s); atlas acceptance `Ran 20 … OK` (0.707 s); full Python `Ran 324 tests … OK` rc 0 (48.787 s); `atlas-paths` `1 passed, 0 failed`. **Full Lune not re-run**: delta touches only Python (`tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`); no Lune-read file changed — relied on accepted `920 passed, 0 failed (126 specs)` at `a231e3f` |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged. Play-6 evidence (persistent duplicate frame, 3 identical attempts, userland minidump) corroborates the settle rationale; the live settle effect is unverified until the next atlas Play |

## Smallest fix (REJECT reason)

1. Make the audit file reflect the complete current attempt list on every flush: replace the exclusive
   `write(...)`+`FileExistsError: pass` in `flush_capture_attempts` with an overwrite of the derived audit file
   (e.g. `(out / "capture-attempts.json").write_text(json.dumps(self.capture_attempts, indent=2) + "\n")`), or write
   the file once at the end of the run/failure with the complete list (the exclusive-create rule should stay for
   primary evidence, not for this derived audit copy).
2. Add a regression test that combines an early retry (e.g. view-002 duplicate→fresh) with a later fail-closed view
   (frozen overhead) and asserts `capture-attempts.json` entries == `result.json.captureAttempts` (and covers the
   success-with-retry case too).
3. Optional (same commit): name `kind`/`settle` in the camera-moved failure message as the stale message does (the
   commit message claims "failure messages name the kind and settle"; only the stale message does).

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the 1.0 s settle effect on real overhead frames is unverified (Play 6 showed the frame
  frozen for ≥0.6 s; the next atlas Play is the live test).
- **R2 — cosmetic:** report says "792 pre-existing untracked evidence-dir lines"; tree now has 805. No commit impact.
- **R3 — message wording:** camera-moved failure does not name kind/settle (see fix 3).
- **R4 — snapshot semantics:** even on success-with-retry the audit file is written once (at the retry view) and never
  updated; if snapshot semantics are intentional, the claim/report should say so instead of "complete/consistent".
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-layer5-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
invariants + settle/flush code), `audit-probe.py` + `audit-consistency-probe.json` (the stale-file proof: 20 vs 2
entries), `suites.log`, `runner.log` (90 OK), `atlas.log` (20 OK), `python.log` (324 OK), `atlas-paths.log` (1/0),
`mutations.log` (M10 3 failures, M11 1 error; restored hash `7c0790df5d25…`), `run-started.txt`. Throwaway copies
`/tmp/checker-layer5/{repo,mut}`; maker repo read-only, unchanged, still at `ed9b797`.
