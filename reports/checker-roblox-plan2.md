# CHECKER — Plan 2 (bounded world-ready gate) — verdict: **ACCEPT**

Fresh-context independent check of Team D Batch 7's Plan 2 freeze. I built none of it (maker-never-checker).
All work below ran from a throwaway clone, never the maker's working tree. No Play, no Studio input, no build,
no credential value, $0. Report to `lead` only.

- **Checked commit:** `29a626b2b8a13556ac5ee07db6fdb60401ecc6e8` (tree `9dba457b070564c042df82dae41092e75cec610e`),
  branch `prep/b1-capture-format-20261101` in `~/Work/projects/untitled-street-game`, parent `a6ac63a`
  (doc-only, parent `d2a94db`).
- **Fresh copy:** `/tmp/checker-roblox-plan2/repo` (`git clone --no-hardlinks --branch prep/b1-capture-format-20261101`,
  2026-11-01T19:30:05-05:00, rc 0). HEAD == the checked commit, tree matches, `git status --porcelain` = 0 lines.
  `build/` is gitignored and **not in the commit**, so the two sealed artifacts were copied read-only into the clone
  and hash-checked before/after every run (unchanged).
- **No remotes** are configured in the repo (`git remote -v` empty): provenance is local-object based (commit object,
  tree, parent chain, branch containment, working-tree hash equality).

## Verdict in one line

**ACCEPT** — `29a626b` is exactly the disclosed 3-file fixture-timing gate (+203/−2: `tools/vinegar_run.py`,
`tests/tools/test_vinegar_run.py`, one runbook paragraph), all three claimed suites reproduce green from a fresh copy
(81/81, 298 OK, 914/0 across 125 specs), all four mutation proofs reproduce (6 / 1 / 3 non-passing; hang caught), and the
gate is 30 s bounded, 0.25 s re-polling, fail-closed with the named message, writes no capture/sidecar on expiry, and
leaves the frame-sidecar contract and all game code untouched.

## 1. Provenance — PASS

Commands (read-only in the maker repo; fresh clone for execution):

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD` / `git rev-parse HEAD^{tree}` (maker repo) | `29a626b2b8…` / `9dba457b0705…` |
| 2 | `git rev-parse 29a626b^`; `git rev-parse a6ac63a^` | `a6ac63a07c05…`; `d2a94db54cc3…` (so base `d2a94db` is the parent of the doc-only commit, as claimed) |
| 3 | `git branch --contains 29a626b` | only `prep/b1-capture-format-20261101` (branch tip) |
| 4 | `git status --porcelain` | 0 tracked modifications; 792 pre-existing `??` evidence lines (as the maker reported) |
| 5 | clone + `git rev-parse HEAD HEAD^{tree}` + `git status --porcelain \| wc -l` | `29a626b2b8…` / `9dba457b0705…` / `0` |
| 6 | sha256 of the 3 changed paths: fresh copy vs maker working tree | identical for all three: `vinegar_run.py 171cf689…`, `test_vinegar_run.py 77a3464b…`, `VINEGAR_RUNBOOK.md 9c4cf1d8…` |
| 7 | sha256 sealed artifacts (clone and maker repo, before/after every suite) | `84324d434266…` (1,990,548 B) `build/release.rbxlx`; `c6651793ace0…` (2,403,691 B) `build/test.rbxlx`; Studio place `usg-desktop.rbxlx` = `c6651793ace0…` — all unchanged |
| 8 | maker repo re-checked at end (19:42:42-05:00) | HEAD `29a626b`, tracked mods 0, sealed hashes unchanged |

Evaluated tree == commit tree: the suites ran in a clone whose HEAD is the commit and whose tracked tree is clean;
the two changed source files hash-match the maker working tree, which is tracked-clean at the same commit.
A parent-baseline clone at `a6ac63a` (tree `a316f71e…`, clean) was used for the test-count delta (§3).

## 2. Scope — PASS, no out-of-scope change

- `git diff --numstat 29a626b^ 29a626b` → **3 files, +203/−2**: `tools/vinegar_run.py` +81/−0,
  `tests/tools/test_vinegar_run.py` +120/−2, `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` +2/−0.
- `git diff --numstat d2a94db 29a626b` → same 3 files (+209/−8 including the doc fix); nothing else in the range.
- `a6ac63a` = **1 file, +6/−6**, doc-only (`git show --stat a6ac63a`); no code/test change.
- Untouched by both commits: `src/` (no game code), `build/` (no sealed artifact is even tracked),
  `tools/vinegar_lib/frames.py`, `docs/art/SIDECAR.md`, `tools/look_metrics.py`, `tests/unit/` — so the frame-sidecar
  contract and the Lune sources are unchanged. The only capture-related addition is a `result.json` trace.
- The +2 runbook lines in `29a626b` document the new gate; the +6/−6 queue-rules rewrite is the separate doc fix
  `a6ac63a` and is consistent with the code (§6). No out-of-scope change found.

## 3. Suites re-run from the fresh copy — PASS (all three match the claim)

| # | Command (cwd = `/tmp/checker-roblox-plan2/repo`) | Actual result |
|---|---|---|
| 1 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 81 tests in 10.458s — OK**, rc 0 (19:31:23–19:31:33-05:00) |
| 2 | `python3 -m unittest discover -s tests/tools` | **Ran 298 tests in 41.816s — OK**, rc 0 (19:31:37–19:32:20-05:00) |
| 3 | `./.tools/bin/lune run tests/unit/run.luau` (bounded `timeout -k 10 1800`) | **unit: 914 passed, 0 failed (125 specs)**, rc 0, 11m12s (19:31:17–19:42:29-05:00); `ls tests/unit/*.spec.luau \| wc -l` = 125 |
| 4 | `python3 -m py_compile tools/vinegar_run.py tests/tools/test_vinegar_run.py` | rc 0 |
| 5 | parent baseline clone at `a6ac63a`: same runner-suite command | **Ran 75 tests — OK**, and the gate tests are absent (`grep -c` = 0) → the freeze adds exactly 6 tests (5 gate tests + 1 Lune mock test) |
| 6 | the six new tests run verbosely | all **ok**, including `test_world_ready_luau_reports_the_gate_contract_against_a_mock_server` (real `WORLD_READY` Luau executed under Lune; not skipped) |

Sealed hashes re-checked after every run: unchanged (row 7 above). Host state untouched: no latch, the Plays counter
(`~/.cache/usg/vinegar-plays.json`, mtime 15:53, pid 382409, 2 Plays) is unchanged, Studio pid 388820 still in Edit.

## 4. Mutation proofs independently reproduced (throwaway clone `/tmp/checker-roblox-plan2/mut`) — PASS

`tools/vinegar_run.py` was restored to `171cf689…` after each mutation (sha256 checked each time); the restored tree
re-ran green (81 OK). Mutations applied by exact-string patch, one at a time:

| # | Mutation | Claim | Actual |
|---|---|---|---|
| a | delete the `await_world_ready` block from `FirstFare.run` | 6 failures | **FAILED (failures=3, errors=3) = 6 non-passing** (3 `KeyError: worldReady` errors + 3 assertion failures, incl. both lighting subtests) — matches the maker's own detail ("3 failures + 3 errors"); the task's shorthand "6 failures" is 6 non-passing tests |
| b | drop `skyPresent`/`atmospherePresent` from `_world_ready` | 1 failure | **FAILED (failures=1)**: `test_world_ready_gate_keeps_requiring_the_readable_lighting_contract (label='no sky/atmosphere')` |
| c | timeout returns the trace as-if-ready instead of raising | 3 failures | **FAILED (failures=3)**: fails-closed test + both lighting subtests |
| d | delete the timeout `raise` (loop has no exit) | suite hangs / caught | **bounded `timeout -k 5 30` killed it: rc=124**, 28 test dots, no `Ran`/`OK` summary, no leftover processes — the deadline is the only exit |

## 5. Behaviour / contract — PASS

Verified from code + an independent behaviour probe (real `run_fixture` state machine with the repo's fake MCP; script
and JSON output under `board/probes/checker-roblox/plan2-behaviour-probe.*`):

- **30 s bound**: `WORLD_READY_TIMEOUT = 30`, default used by the call; fake clock shows expiry at 30.0 s, 121 polls,
  120 sleeps, named message "within 30s". Deadline is additionally capped by `ctx.remaining()` so it never runs past
  the run budget (`RUN_BUDGET=900`, `CLEANUP_RESERVE=240` → ~660 s left at run start, so 30 s in practice).
- **0.25 s re-poll**: `WORLD_READY_POLL = 0.25`; fake clock recorded only `0.25` sleeps, 3 not-ready reads → 4 polls,
  `waitedSeconds=0.75`; a real-race scenario (`ready_polls=3`) passes with `polls=4`.
- **Reads the right state**: `WORLD_READY` Luau (read-only; no `SetAttribute`/`Instance.new`/`Destroy`) reads
  `workspace.Nightshift` `WorldReady`/`WorldSeed`/`WorldLayoutVersion` plus `Lighting:FindFirstChildOfClass("Sky")` and
  `("Atmosphere")`; executed as a **Server** exec (`exec_json(WORLD_READY, "Server", 30)`). `src/server/World.luau`
  confirms the race shape (`WorldReady=false` first at `:350`; `WorldSeed`+`WorldLayoutVersion`+`WorldReady=true` only
  at the end, `:393-395`) and `src/server/CityBuilder.luau:1330/1530` creates `NightshiftSky`/`NightshiftAtmosphere` inside
  `CityBuilder.build`, i.e. before `WorldReady=true`.
- **Before the FIRST capture, both fixtures**: `FirstFare.run` calls `await_world_ready` before the `while` loop's first
  `probe()`/`capture()`; `FIXTURES["moving"]` and `FIXTURES["first-fare"]` both construct `FirstFare`; a `first-fare`
  test proves the gate runs even though that fixture captures no frames. Probe: readiness read happened with
  `captures=0`, then 3 captures.
- **Fail-closed, named, no capture/sidecar on expiry**: probe (patched 0.05 s, never ready) → exit 1, error
  `WorldNotReady: world not ready within 0.05s (Nightshift/WorldReady/WorldSeed/WorldLayoutVersion/Sky/Atmosphere): {…}`,
  `screen_capture` calls = 0, `frames/*.json` = none, abort `{stop: ok, cleanup: ok, verifyClean: ok}`, `report.json`
  written as the failing evidence. `WorldNotReady` subclasses `FixtureFailed` and takes the normal ABORT path.
- **`worldReady` trace in `result.json`**: pass path (probe, real code): `{"waitedSeconds":0.0,"polls":1,
  "seed":2134783819,"layoutVersion":2,"lighting":{"technology":null,"readable":false,"skyPresent":true,
  "atmospherePresent":true}}`; failure path carries the timeout trace (`timeout:true`, `seconds`, `last`). Merged in
  `run_fixture` with `result.setdefault` (never overwrites a result field) via the new `Fixture.result_trace()` hook.
- **Sidecar contract untouched**: no change to `tools/vinegar_lib/frames.py`, `docs/art/SIDECAR.md` or
  `tools/look_metrics.py`; the sidecar's `lighting.technology`/`sky.present`/`atmosphere.present` requirements are
  unchanged, so a capture that lacks them still fails closed. The gate only removes the timing race.
  (`Lighting.Technology` remains unreadable from `execute_luau`, so the gate records it observed and never requires it —
  the disclosed deviation the Lead has already ruled on as an interim; it does not relax any contract.)

## 6. Doc fix `a6ac63a` — PASS (doc matches the code)

`docs/verify/2026-10-19/VINEGAR_RUNBOOK.md:62-74` re-read against `tools/vinegar_run.py`: queue lines are
`shlex.split` and validated with the run options (`QUEUE_OPTIONS = --seed/--timeout/--every/--visual-flags`, same
fixture gates/ranges; `smoke` refuses options) ✔; `done` gets only exit-0 entries, `attempts` keeps every attempt,
checkpoint refuses when `queueSha256`/entries differ ✔; non-zero → `failed` with `class` (`EXIT_CLASS`: 1=cell,
2/3/4/5=harness; smoke non-pass = `harness`/`smoke-failure`) and the queue stops ✔; `--resume --rerun` retries exactly
one entry per invocation, plain `--resume` refuses while `failed` is set, no auto-rerun loop ✔; an interrupted entry
stays undone and a plain `--resume` retries it ✔; a stopped queue returns the failed entry's code, paused/complete
returns 0 ✔. Example seed `2134783819` matches the frozen named seed / `layoutVersion 2` in
`board/plans/team-d/queue-seed-checkpoint-plan-20261101.md` and `atlas-named-seed-plan-20261101.md`
(`Graph.LAYOUT_VERSION = 2`).

## 7. Limits / non-findings

- **No Play was run** (not required by Plan 2, and out of scope for this check): the gate's live effect on the real
  `World.generate` race is argued from code + offline tests only. A future Play still has to prove the capture.
- **`Lighting.Technology` blocker persists**: the sidecar contract still requires a string technology, so after this
  freeze `moving`/atlas captures will still fail closed on that field until the Lead's interim/contract amendment is
  implemented (Plan 2 deliberately does not touch it). This is disclosed by the maker and covered by the Lead's ruling;
  it is not a Plan 2 defect and not a reason to REJECT Plan 2.
- Minor reporting nit (not blocking): on expiry the message/trace name the 30 s **constant**, so if the run budget ever
  shortened the effective bound the message would still say "30s" (fake-clock budget=5 s → waited 5.0 s,
  `seconds:30`). In realistic runs ~660 s remain at run start, so the effective bound is 30 s.
- Minor bound nuance (not blocking): the deadline is checked after each read, so the worst case is 30 s + one 0.25 s
  poll + one read latency (per-call MCP timeout 30 s) before raising; the loop itself has no unbounded path (proved by
  mutation d).
- The gate requires `skyPresent`/`atmospherePresent` to be **true**, which is stricter than the sidecar's
  `present`-is-boolean check (fail-closed direction; no relaxation).
- `build/` is gitignored and absent from the commit: I copied it read-only from the maker repo into the throwaway
  copies (hash-verified unchanged) because the preflight path references `build/test.rbxlx`; this does not affect the
  frozen tree.
- Lune is a regression run only (no Lune/src file changed by the commit). The maker's referenced evidence logs were
  not relied on; every number above was produced by me. Suite timings differ from the maker's by <1 s (Python) and
  <1 min (Lune).
- No out-of-scope change was found; no repo modification, no merge/push, no Studio/Play, no credentials, $0.

## 8. Evidence paths

- Report: `board/reports/checker-roblox-plan2.md` (this file).
- Probe + logs: `board/probes/checker-roblox/` —
  `plan2-behaviour-probe.py`, `plan2-behaviour-probe.json` (constants, pass/expiry traces, fake-clock cadence/bound),
  `runner-suite.log` (81 OK), `python-suite.log` (298 OK), `lune-unit.log` (914/0, 125 specs), `parent-runner.log`
  (75 OK at `a6ac63a`), `new-tests-verbose.log` (the six new tests), `mut-a.log` (3F+3E), `mut-b.log` (1F),
  `mut-c.log` (3F), `mut-d.log` (hang, rc 124), `mut-restored.log` (81 OK), `sealed-before.txt`, `provenance.txt`,
  `run-started.txt`, `lune-start.txt`, `lune-end.txt`.
- Throwaway working copies (not evidence of record; re-creatable from the repo):
  `/tmp/checker-roblox-plan2/{repo,parent,mut}`.
