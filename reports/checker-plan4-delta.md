# CHECKER — Plan 4 fix-round delta (`659779c`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the C1/C2/C3 fix round on top of the Plan 4 commit I accepted (`9022c53`). I built none
of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway clone
`/tmp/checker-plan4-delta/repo` (2026-11-01T21:21:02-05:00) and a second clone for mutations (`/tmp/checker-plan4-delta/mut`),
restored afterwards; maker tree only read.

- **Fix-round commit:** `659779ce89fa18c8ab2dc13a29906f7ab05e1499` (`659779c`), parent `9022c53` (the accepted Plan 4),
  branch `prep/b1-capture-format-20261101` tip, tree `844523c3fcbd97dcca0caf2db8fbb25d9217a34c`; tracked mods **0**
  (792 pre-existing untracked evidence lines); sealed `84324d43…`/`c6651793…` unchanged before/after every run and at
  the end.
- **Delta:** exactly **4 files, +33/−2** — `tests/atlas/Server.luau` (+2/−1), `tests/tools/test_atlas_acceptance.py`
  (+3/−0), `tests/tools/test_vinegar_run.py` (+25/−0), `tools/vinegar_run.py` (+3/−1). `git diff --name-only
  9022c53..659779c` lists exactly these 4; **no `src/`, `assets/`, `build/` or project-file change**.

## Verdict in one line

**ACCEPT** — C1 is fixed in the frozen bytes (trace now published at `Server.luau:74` **before** `WorldSeed.requireApplied`
at `:80`, refreshed at `:86`; the atlas-acceptance test asserts the ordering and the two publications), C2 is fixed
(`ValueError` early-stop is caught with `TimeoutError`/`FixtureFailed`, and a new runner test proves the fail-fast trace
is kept), C3 is corrected in the report (+185/−71 original, +33/−2 fix round), the delta is pure, the 45 s bound and
starter ordering are intact, no rebuild path is reintroduced, and the delta mutations fail exactly as claimed (G → atlas
acceptance 1 failure; H → runner 1 failure). Suites on the fix tip: runner 85 OK, atlas acceptance 15 OK, world-seed
10/0 (2 specs), full Python 314 OK; Lune spec reading the changed file (`atlas-paths`) 1/0 and world-seed 10/0; full
Lune relied on the 919/0 at `9022c53` (stated below).

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | Provenance / freeze / cleanliness | **PASS** | HEAD `659779c`, parent `9022c53`, tree `844523c3…`, branch tip, tracked mods 0; sealed hashes unchanged |
| 2 | C1 — publication ordering | **PASS** | `tests/atlas/Server.luau:73-87`: `trace.actualSeed/trace.actualLayout` set at `:73`; **`workspace:SetAttribute("AtlasWorldTrace",…)` at `:74` BEFORE `if manifest.namedSeed then` (`:75`) and `WorldSeed.requireApplied` (`:80`)**; refresh publication at `:86` before any view/capture; assert at `:87`. Regression test: `test_atlas_acceptance.py:222` `assertLess(server.index('workspace:SetAttribute("AtlasWorldTrace"'), server.index('WorldSeed.requireApplied'))` + `:223` `assertEqual(server.count(…), 2)` |
| 3 | C2 — early-stop trace | **PASS** | `tools/vinegar_run.py:672` `except (TimeoutError, FixtureFailed, ValueError) as exc:` then reads the trace and raises the named failure with `worldTrace` attached; new `test_atlas_early_stop_runtime_error_keeps_the_world_trace` (fail-fast runtime before view-001 → exit 1, `worldTrace` in error, `world-trace.json` on disk, 0 captures, clean abort) |
| 4 | C3 — bookkeeping | **PASS** | Report §1b/§2 now says `9022c53` = **+185/−71** (was +186/−71) and fix round = 4 files **+33/−2**; `git show --shortstat 9022c53` = 185/71 and `git diff --numstat 9022c53..659779c` sums to 33/2 |
| 5 | Delta purity / invariants | **PASS** | 4 claimed files only; no game/asset/sealed path; **no rebuild path reintroduced** (`WorldSeed.ensure`/`generateSafe(` absent from Server/WorldSeed); 45 s bound intact (`ATLAS_FIRST_VIEW_TIMEOUT = 45`, first-view clamp); `AtlasStarterTrace` ordering intact (`tools/atlas_acceptance.py` untouched: `SEED_PATCH + source`, trace set before `require(h).start`) |
| 6 | Suites on the fix tip | **PASS** | runner `Ran 85 tests … OK` (11.073 s); atlas acceptance `Ran 15 … OK`; world-seed `unit: 10 passed, 0 failed (2 specs)`; full Python `Ran 314 tests … OK` rc 0 (47.088 s); Lune spec that reads the changed `Server.luau` (`atlas-paths`) `unit: 1 passed, 0 failed (1 specs)`; world-seed spec re-run green |
| 7 | Delta mutation G (publication moved back) | **PASS** | Throwaway clone, delete the pre-check publication line → atlas acceptance **FAILED (failures=1)** (`test_seeded_starter_traces_the_fast_path_and_the_server_asserts_instead_of_rebuilding`); restore → `git diff` clean, file sha256 `90193193…`/commit blob `ac94b2b1…` |
| 8 | Delta mutation H (C2 regression) | **PASS** | Remove `ValueError` from the except tuple → runner **FAILED (failures=1)** (`test_atlas_early_stop_runtime_error_keeps_the_world_trace`); restore → file sha256 `3e102893…`/commit blob `6a8cb3f3…` |
| 9 | No Play / no Studio input | **PASS (as checked)** | I ran no Play/Studio; host Plays counter unchanged (pid 382409, plays 2, mtime 15:53), no latch; sealed hashes unchanged |

## Residual (named, non-blocking)

- **Report accuracy nit:** §2 row 4 says "the fix round touches no Lune-read file". That is not literally true —
  `tests/unit/atlas-paths.spec.luau:18` reads `tests/atlas/Server.luau` (source-shape check for `Paths.emit(anchor)`).
  The delta does not change that assertion, and I re-ran the exact spec on the fix tip (`1 passed, 0 failed`), plus the
  world-seed spec (`10/0`); I did **not** re-run the full 12-minute Lune suite on the fix tip and rely on the 919/0 at
  `9022c53`. No functional risk, wording only.
- C1/C2 are now covered by regression tests and mutation proofs; no other residual from the previous delta findings.
- Limits: no Play, so the live mismatch fail-fast/trace publication is still only offline-proven (unit tests + mutation
  G); the diagnostic Play remains the live proof. No game code, asset, sealed artifact, Studio input or image read; $0.

## Evidence paths

`board/probes/checker-plan4-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + changed paths + sealed +
ordering + bound + starter order + report bookkeeping), `runner.log` (85 OK), `atlas.log` (15 OK), `world-seed.log`
(10/0), `python.log` (314 OK), `atlas-paths.log` (1/0, the Lune spec reading the changed file), `mutations.log` (G/H
failures + restored hashes), `run-started.txt`. Throwaway copies `/tmp/checker-plan4-delta/{repo,mut}`; maker repo
read-only, unchanged, still at `659779c`.
