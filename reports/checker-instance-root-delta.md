# CHECKER — instance-root fix delta (`35caa02`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the `requireApplied` Instance-root fix + R1 restoration on top of my accepted
starter-fix delta (`e2165f5`). I built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran
from a throwaway clone `/tmp/checker-instance/repo` (2026-11-01T22:15:24-05:00) and a second clone for mutations; maker
tree only read.

- **Fix commit:** `35caa0221eafe757ea2c16b635f72061ad72cf74` (`35caa02`), parent `e2165f5` (my last accepted delta),
  branch `prep/b1-capture-format-20261101` tip, tree `b56ecea6555389b406f86f28f069423de74c91c9`; tracked mods **0**
  (untracked 798 evidence lines, was 792 — cosmetic, see R2); sealed `84324d43…`/`c6651793…` unchanged before/after and
  at the end.
- **Delta:** exactly **3 files, +35/−1** — `tests/atlas/WorldSeed.luau` (+5/−1), `tests/tools/test_atlas_acceptance.py`
  (+3/−0), `tests/unit/world-seed-override.spec.luau` (+27/−0). No `src/`, `assets/`, `build/` change; `tools/vinegar_run.py`
  and `tests/atlas/Server.luau` untouched, so the 45 s bound, assert-and-skip and trace publication order are
  byte-identical to what I last accepted.

## Verdict in one line

**ACCEPT** — the root check now accepts a Roblox **Folder Instance** (`type(root)=="userdata"`) or a test table while
still failing closed on nil/`{}`/string/number with the named `the world root is required` message, and the seed/layout
checks + named fail-fast text are unchanged; a Lune spec case with a **real `@lune/roblox` Folder** proves it (and the
reverted-check mutation fails exactly that case); the two dropped C1 ordering assertions are back and actually catch an
ordering regression (mutation M2 fails the atlas-acceptance test); the delta is pure, the 45 s bound/assert-and-skip are
untouched, and all suites are green including the full Lune (920/0).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Instance-compatible root check; fail-closed; messages unchanged | **PASS** | `tests/atlas/WorldSeed.luau:66-69`: `local kind = type(root); assert(root ~= nil and (kind == "userdata" or kind == "table") and type(root.GetAttribute) == "function", "the world root is required")` — the only changed hunk in the module is that assert line; `WorldSeed.parse`/seed+layout checks and the `named seed not applied before Play: … expected X/Y; refusing in-Play rebuild` error are byte-identical. Nil/`{}`/string/number short-circuit the assert (no indexing of a non-root) → named root message; a non-world `Part` Instance reaches the seed/layout check → named fail-fast |
| ② Real-Instance Lune case | **PASS** | `tests/unit/world-seed-override.spec.luau:7` `local RB = require('@lune/roblox')`, `:86` new case, `:89` `RB.Instance.new('Folder')`: matching plan accepted (`fastPath=true`, `rebuilt=false`, `worldSeed`/`layoutVersion` read from the Instance's attributes), mismatching Instance → named seed/layout fail-fast, `nil`/`{}`/string/number → `the world root is required`, non-world `Part` → named seed/layout fail-fast. Run: `unit: 11 passed, 0 failed (2 specs)` (spec uses the f32-exact seed 204817 because `@lune/roblox` stores number attributes as f32; the real engine's doubles are the Play's job — stated, not hidden) |
| ③ R1 ordering assertions restored | **PASS** | `tests/tools/test_atlas_acceptance.py:231` `assertLess(server.index('workspace:SetAttribute("AtlasWorldTrace"'), server.index('WorldSeed.requireApplied'))` and `:232` `assertEqual(count, 2)`; atlas acceptance 16 OK; mutation M2 (publication moved below the check) makes that test fail → the assertion is not vacuous |
| ④ Delta scope | **PASS** | Exactly the 3 claimed files; no game/asset/sealed path; `tools/vinegar_run.py` and `tests/atlas/Server.luau` untouched (45 s bound `ATLAS_FIRST_VIEW_TIMEOUT = 45`; publication `:74` before `requireApplied` `:80`, refresh `:86`); no rebuild path reintroduced (`WorldSeed.luau` has no `generateSafe(`; Server still calls `requireApplied`) |
| ⑤ Suites (fresh clone) + mutation | **PASS** | runner `Ran 85 tests … OK` (11.979 s); atlas acceptance `Ran 16 … OK` (0.688 s, incl. the Lune real-starter test); full Python `Ran 315 tests … OK` rc 0 (57.521 s); world-seed `unit: 11 passed, 0 failed (2 specs)`; `atlas-paths` `1 passed, 0 failed (1 specs)`; **full Lune `unit: 920 passed, 0 failed (126 specs)` rc 0** (22:16:04–22:28:40, 12m36s). Mutation **M1** revert root check to `type(root)=="table"` → world-seed spec **10 passed, 1 failed (2 specs)** (the new Instance case: `the world root is required`); mutation **M2** ordering regressed → atlas acceptance **FAILED (failures=1)** (restored R1 assertion). Both restored: `WorldSeed.luau` sha256 `bab38731fe57…`, `Server.luau` `90193193ba68…`, git clean |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed hashes unchanged. The maker's repeat Play A is evidenced in `board/probes/team-d/diagnostic-play3-20261102.log` (`failure: … AtlasWorldSeed:66 'the world root is required' … type(root)=='table' rejects the Folder Instance`, with the trace showing `actualSeed=2134783819 actualLayout=2 starter applied=2134783819` = fast path won) — corroborates the root cause; I did not reproduce the Play |

## Residuals (named, non-blocking)

- **R1 — none open.** The previously dropped C1 assertions are restored and mutation-proven (M2).
- **R2 — cosmetic:** the report says "792 pre-existing untracked evidence-dir lines"; the tree now has 798 (Play-2/Play-A
  evidence dirs). Tracked tree is clean; no commit impact.
- **R3 — live limit:** no Play, so the Instance acceptance is proven by the Lune `@lune/roblox` fake-Instance execution
  (a proxy; f32 attribute caveat above), not by a real engine Play. The authorized repeat Play A (2/3 of the fresh
  opening) is the live proof that records `fastPathApplied=true`.
- **R4 — minor report discrepancy:** the maker's mutation note says reverting the check makes "3 failures"; my run of
  that mutation gives 1 failed test (10 passed, 1 failed) in the Lune harness (the new case is one `H.test` with several
  assertions). The claim that the new case fails is correct; only the count wording differs.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-instance-root-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
root-check bytes + named-message diff + R1 assertions + real-Instance spec), `suites.log`, `runner.log` (85 OK),
`atlas.log` (16 OK), `python.log` (315 OK), `world-seed.log` (11/0), `atlas-paths.log` (1/0), `lune.log`
(920/0, 126 specs), `mutations.log` (M1 10/1; M2 atlas 1 failure; restored hashes), `run-started.txt`,
`run-finished.txt`. Throwaway copies `/tmp/checker-instance/{repo,mut}`; maker repo read-only, unchanged, still at
`35caa02`.
