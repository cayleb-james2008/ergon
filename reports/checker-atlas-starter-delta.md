# CHECKER — atlas-starter fix delta (`e2165f5`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the named-seed starter fix on top of my accepted Plan 4 fix round (`659779c`). I built
none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway clone
`/tmp/checker-starter/repo` (2026-11-01T21:56:41-05:00) and a second clone for mutations; maker tree only read.

- **Fix commit:** `e2165f530be1350c0a402faa62c90fde3f126c91` (`e2165f5`), parent `659779c` (my last accepted delta),
  branch `prep/b1-capture-format-20261101` tip, tree `0e4c907ab6bc225fb111bc527f2e772d4917eaae`; tracked mods **0**;
  sealed `84324d43…`/`c6651793…` unchanged before/after every run and at the end.
- **Delta:** exactly **4 files, +143/−29** — `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` (+4/−1),
  `tests/tools/test_atlas_acceptance.py` (+91/−12), `tests/tools/test_vinegar_run.py` (+18/−4),
  `tools/atlas_acceptance.py` (+30/−12). `git diff --name-only 659779c..e2165f5` lists exactly these; **no `src/`,
  `assets/`, `build/` change**; `tools/vinegar_run.py` and `tests/atlas/{Server,WorldSeed}.luau` are untouched, so the
  45 s bound, assert-and-skip, trace publication order and no-rebuild path are byte-identical to what I last accepted.

## Verdict in one line

**ACCEPT** — ① the generated seeded starter now `require`s both modules and calls `apply`/`applied` on the module
tables (the `apply is not a valid member of ModuleScript` class is structurally removed; source-shape assertion exists
and a mutation reverting it fails 2 tests); ② the whole seed patch is inside the starter's error-reporting `xpcall` and
the failure path posts a `runtime` error carrying `AtlasWorldTrace` + `AtlasStarterTrace` (proven by a Lune execution of
the real starter with a deliberate crash → exactly one trace-bearing runtime POST, server never starts; mutation
unwrapping the xpcall fails 2 tests); ③ both offline tests exist and pass. Suites on the fix tip reproduce (85 / 16 /
315 / 10-0 / 1-0). One residual: this delta removed the C1 ordering regression assertions from the rewritten
atlas-acceptance test (behaviour is untouched and was verified at `659779c`; smallest fix is to re-add the two lines).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① requires both modules, calls on required tables | **PASS** | Generated seeded starter (`A.starters(seed=2134783819)`): `local World=require(assert(script.Parent:WaitForChild("World",20)));`, `local WorldSeed=require(assert(script.Parent:WaitForChild("AtlasWorldSeed",20)));`, `WorldSeed.apply(World,seed)`, `applied=WorldSeed.applied(World)`; `S.apply(W,seed)`/`S.applied(W)` absent. Source-shape test `test_seeded_starter_requires_the_modules_and_reports_every_crash` asserts all of these. The Play-2 console evidence file (`board/probes/team-d/console-after-atlas-play-20261102.json`) shows the old failure verbatim: `apply is not a valid member of ModuleScript "ServerScriptService.Nightshift.AtlasWorldSeed"` at `AtlasServerStarter, Line 1` — that call no longer exists |
| ② whole seed patch inside the error-reporting xpcall; crash posts trace | **PASS** | `tools/atlas_acceptance.py:starter_source(seed)`: `local n=…; local ok,err=xpcall(function() <SEED_PATCH> local h=…; require(h).start(n) end,debug.traceback); if not ok then <STARTER_ERROR_POST>; error(err) end`. Generated source index check: `xpcall(function()` at 67 < `USGWorldSeed` at 120 (seed patch inside); server start also inside. `STARTER_ERROR_POST` posts `phase="runtime", ok=false, report={error=…, worldTrace=workspace:GetAttribute("AtlasWorldTrace"), starter=workspace:GetAttribute("AtlasStarterTrace")}`; the unseeded starter gets the same trace-bearing error post. Lune test `test_real_seeded_starter_runs_under_lune_and_its_crash_class_posts_the_trace` executes the real generated starter against a fake Instance tree: normal path (requires World then AtlasWorldSeed, `apply` receives the World **table**, `applied` reads back 2134783819, server start gets `nonce-owned`, `AtlasStarterTrace` written, no POST); crash path (injected crash in the seed module require) → exactly **one** runtime POST, `report.error` contains the crash, `report.worldTrace` carries the trace, `start` is nil |
| ③ offline tests (Lune real starter + Python source-shape) | **PASS** | Both tests exist in `tests/tools/test_atlas_acceptance.py`; atlas acceptance `Ran 16 tests … OK`; the Lune test run verbosely `Ran 1 test … OK` (0.154 s); full Python `Ran 315 tests … OK` rc 0 |
| Delta purity / invariants | **PASS** | 4 claimed files only; no game/asset/sealed path; `tools/vinegar_run.py` untouched (`ATLAS_FIRST_VIEW_TIMEOUT = 45`, `except (TimeoutError, FixtureFailed, ValueError)`); `tests/atlas/Server.luau` untouched (publication before `requireApplied` at `:74`/`:80`, refresh `:86`); `tests/atlas/WorldSeed.luau` untouched (`requireApplied` present, no `generateSafe(`); runbook records the require + xpcall rule and the superseded "rebuild hang" reading |
| Suites on the fix tip (fresh clone) | **PASS** | runner `Ran 85 tests … OK` (11.707 s); atlas acceptance `Ran 16 … OK` (0.614 s); full Python `Ran 315 tests … OK` rc 0 (51.835 s); world-seed `unit: 10 passed, 0 failed (2 specs)`; `atlas-paths` (Lune spec reading `tests/atlas/Server.luau`) `unit: 1 passed, 0 failed (1 specs)`; the new Lune SEED_PATCH test verbose OK. **Full Lune not re-run** (12 min) — the delta touches no Lune-read file (no `.luau`/`src` path changed; Lune specs read `src/**` and `tests/atlas/{Server,WorldSeed}.luau`, all unchanged), and I re-ran the two Lune specs that read atlas sources |
| Mutation M1 (revert to WaitForChild Instance, no require) | **PASS (2 failures)** | Throwaway clone: atlas acceptance `FAILED (failures=2)` — `test_real_seeded_starter_runs_under_lune_and_its_crash_class_posts_the_trace` + `test_seeded_starter_requires_the_modules_and_reports_every_crash` |
| Mutation M2 (move the seed patch outside the xpcall) | **PASS (2 failures)** | Same suite `FAILED (failures=2)` — the same two tests; restored file sha256 `0f33184d…` (frozen), `git status` clean |
| No Play / no Studio input | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed hashes unchanged. (The maker's Play 2 is evidenced by the console JSON and the Plays counter mtime 21:33; I did not reproduce it) |

## Residual (named, non-blocking)

- **R1 — C1 regression assertions dropped.** The delta rewrote `test_seeded_starter_traces_the_fast_path_and_the_server_asserts_instead_of_rebuilding` into the new source-shape test and, in doing so, removed the two assertions that pinned the C1 ordering (`assertLess(server.index('workspace:SetAttribute("AtlasWorldTrace"')…, server.index('WorldSeed.requireApplied'))` and `count(publication)==2`). The `Server.luau` behaviour is untouched and was verified at `659779c`, so this is a test-coverage regression only. **Smallest fix:** re-add those two lines to the new test.
- **R2 — cosmetic:** report says "792 pre-existing untracked evidence-dir lines"; the tree now has 796 (Play-2 evidence dirs). Tracked tree is clean; no commit impact.
- **R3 — live limit:** no Play, so the crash-class fix is proven by the Lune fake-tree execution + source-shape/mutations, not by a live starter crash; the fresh 3-Play opening + Play A is the live proof. The Play-2 root-cause claim is corroborated by the recorded console JSON, not independently re-run.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-atlas-starter-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
generated-starter structure checks + invariants + C1-assertion check), `suites.log`, `runner.log` (85 OK),
`atlas.log` (16 OK), `python.log` (315 OK), `world-seed.log` (10/0), `atlas-paths.log` (1/0), `mutations.log`
(M1/M2 both `FAILED (failures=2)`, restored sha256), `run-started.txt`. Throwaway copies
`/tmp/checker-starter/{repo,mut}`; maker repo read-only, unchanged, still at `e2165f5`.
