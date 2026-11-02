# TEAM D — ATLAS NAMED-SEED STARTER FIX (Lead-approved ①②③), frozen 2026-11-02

Lead approval: 2026-11-02T02:45Z ("FIX APPROVED as you proposed — implement ①②③ now, offline (no Play)"). Machine
`outis-desktop`. **No Play** in this fix round; no build, no publication, no credential values. Spend $0.

**Fix commit: `e2165f530be1350c0a402faa62c90fde3f126c91`** (`e2165f5`; parent `659779c` = accepted Plan 4 fix round;
branch tip; tree `0e4c907ab6bc225fb111bc527f2e772d4917eaae`; tracked tree clean; 4 files, **+143/−29**). Sealed artifacts
unchanged (`84324d43…` release, `c6651793…` test/place).

## 1 · Root cause (diagnostic Play 2, console evidence — supersedes the old reading)

`SEED_PATCH` called `S.apply(W, seed)` where `S` was the **`AtlasWorldSeed` ModuleScript Instance** returned by
`WaitForChild`, never `require`d → `apply is not a valid member of ModuleScript
"ServerScriptService.Nightshift.AtlasWorldSeed"` at the starter's **line 1** (console:
`board/probes/team-d/console-after-atlas-play-20261102.json`). The error sat **outside** the starter's `xpcall`, so no
runtime phase was posted, `AtlasServer.start` never ran, no `AtlasWorldTrace`/`AtlasStarterTrace` was written, and the
runner could only time out (45 s bound; previously the silent 600 s wait). The identical patch exists before Plan 4
(`git show 9022c53^`), so **the 2026-10-31/11-01 "in-Play City+Highway rebuild hang" reading is SUPERSEDED — the
rebuild never ran.** Correction recorded in `board/reports/team-d-diagnostic-play2.md` §1a.

## 2 · What was implemented

① `SEED_PATCH` **requires** both modules and calls `apply`/`applied` on the module tables:
   `local World=require(assert(script.Parent:WaitForChild("World",20)))`,
   `local WorldSeed=require(assert(script.Parent:WaitForChild("AtlasWorldSeed",20)))`,
   `WorldSeed.apply(World,seed)`, `applied=WorldSeed.applied(World)`.

② The **whole seeded starter** — seed patch **and** `require(AtlasServer).start(n)` — is now one error-reporting
   `xpcall` (`starter_source(seed)` in `tools/atlas_acceptance.py`). Any starter crash posts a `runtime` error carrying
   **`AtlasWorldTrace` + `AtlasStarterTrace`** before re-raising. The unseeded starter gets the same trace-bearing error
   post, so the silent-timeout class is closed for both paths (not just seeded).

③ Offline tests:
   - **Lune execution of the REAL generated seeded starter** against a fake Instance tree
     (`tests/tools/test_atlas_acceptance.py::test_real_seeded_starter_runs_under_lune_and_its_crash_class_posts_the_trace`):
     * happy path — `require()` is called with the `World`/`AtlasWorldSeed` instances and returns the module tables;
       `apply` receives the required World table, `applied` reads back, `AtlasServer.start` gets the nonce,
       `AtlasStarterTrace` is written with `applied=2134783819`; `posted == {}` (no error post);
     * deliberate mid-patch crash (the require of `AtlasWorldSeed` throws) — the script fails, **exactly one** runtime
       error POST is made with `phase="runtime", ok=false`, `report.error` contains the crash, `report.worldTrace`
       carries the trace, and **the server never starts**.
   - **Python source-shape test** (`test_seeded_starter_requires_the_modules_and_reports_every_crash`): requires both
     modules, calls `apply` on the module table (no `S.apply(W,seed)`), the seed patch sits **inside** the `xpcall`,
     both starters post `worldTrace`/`starter`, the named-seed runner test asserts assert-and-skip (no
     `world.generateSafe`, `WorldSeed.requireApplied`) and the exist/repro expectations were updated.
   - **Mutation proofs (maker self-check, each caught, restored):** M1 revert to `S.apply` on the Instance (no require)
     → atlas acceptance **FAILED (2)**; M2 move the seed patch outside the `xpcall` → **FAILED (2)**.

## 3 · Freeze evidence (commands + results; log `board/probes/team-d/atlas-starter-fix-20261102.log`)

| # | Command (fresh checkout of `e2165f5`) | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `e2165f5…`; tracked tree clean (792 pre-existing untracked evidence-dir lines) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 85 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_atlas_acceptance.py'` | **Ran 16 tests … OK** (includes the Lune real-starter happy/crash test) |
| 4 | `python3 -m unittest discover -s tests/tools` | **Ran 315 tests … OK** rc 0 |
| 5 | `./.tools/bin/lune run tests/unit/run.luau` | full suite re-run on the fix tree; log `board/probes/team-d/lune-unit-atlasfix-20261102.log` |
| 6 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged before/after |
| 7 | `git show --stat e2165f5` | 4 files, +143/−29 (`tools/atlas_acceptance.py`, two test files, runbook) |

**Exact checker reproduction:** `git checkout e2165f530be1350c0a402faa62c90fde3f126c91` in a fresh worktree → `git
status` empty; run rows 2–5 (no Play or Studio needed); sealed hashes row 6. Contract spot checks:
(a) `python3 -c "import sys;sys.path.insert(0,'tools');import atlas_acceptance as A;s=A.starters(seed=2134783819)['AtlasServerStarter'][2];print('require(assert(' in s, 'WorldSeed.apply(World,seed)' in s, 'S.apply' in s, s.index('xpcall(function()')<s.index('USGWorldSeed'))" → expects
`True True False True`; (b) the Lune test above proves the execution path and the crash POST; (c) mutations M1/M2 fail.

## 4 · Sequencing accepted (Lead's plan, confirmed)

1. On checker delta ACCEPT of `e2165f5`: **`studiocycle` close/reopen** Studio (sealed-hash guarded, nothing ever
   saved) to get a **fresh 3-Play opening** — the current opening stays at 2/3 and its last Play is reserved.
2. **Play A = `atlas:scout --seed 2134783819`** on the fixed code: capture `worldTrace.fastPathApplied` under the 45 s
   bound; `true` ⇒ assert-and-skip suffices; `false` ⇒ fallback (a) (Bootstrap-disabled named-world rebuild) becomes the
   named follow-up.
3. **Play B = abort-contact fixture** only after the scenario change
   (`board/probes/team-d/abort-contact-fixture-geometry-fix-20261102.md`) is ruled, implemented and delta-checked — not
   before.
4. The **third Play stays reserved**.
The fixture-geometry design note is filed and awaits the Lead/Nox ruling; nothing about it is implemented.
