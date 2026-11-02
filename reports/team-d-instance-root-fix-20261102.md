# TEAM D — `requireApplied` INSTANCE-ROOT FIX + R1 (frozen 2026-11-02, for checker delta re-verify)

Lead approval: 2026-11-02T03:10Z. **Offline only — no Play** in this fix round. Machine `outis-desktop`.
**Fix commit: `35caa0221eafe757ea2c16b635f72061ad72cf74`** (`35caa02`; parent `e2165f5` = accepted starter fix; branch
tip; tree `b56ecea6555389b406f86f28f069423de74c91c9`; tracked tree clean; 3 files, **+35/−1**). Sealed artifacts
unchanged (`84324d43…` release, `c6651793…` test/place). Log: `board/probes/team-d/instance-root-fix-20261102.log`.

## 1 · Why (from Play A, 2026-11-02T03:04:04Z)

The fixed starter ran and the fast path won (`starter t0=24.832 s` < `rootFoundAt=24.889 s`, `applied=2134783819`;
world `actualSeed=2134783819 / actualLayout=2 == expected`, `rebuild=false`), the trace was published before the check
and retained on all three paths — then `WorldSeed.requireApplied` rejected the Roblox **Folder Instance** with
`type(root) == "table"` (`AtlasWorldSeed:66: the world root is required`, called from `AtlasServer:80`). An Instance's
`type` is `userdata`. The old `WorldSeed.ensure` carried the same assert, so the bug was masked by the starter crash.
(Play report: `board/reports/team-d-diagnostic-play3.md`.)

## 2 · What changed

① `tests/atlas/WorldSeed.luau::requireApplied` root check is Instance-compatible:
   `local kind=type(root); assert(root ~= nil and (kind == "userdata" or kind == "table") and type(root.GetAttribute) == "function", "the world root is required")`
   — the named fail-fast text and the seed/layout checks are unchanged; a value that cannot be a root is never indexed.

② Lune spec (`tests/unit/world-seed-override.spec.luau`) gains a case with a **REAL `@lune/roblox` Folder Instance
   root**:
   - Instance root matching the plan → accepted: `fastPath=true`, `rebuilt=false`, `worldSeed`/`layoutVersion` read
     from the Instance's attributes;
   - mismatching Instance → still fails fast with `named seed not applied before Play …` (named seed/layout message);
   - `nil`, `{}`, string, number → fail closed with exactly `the world root is required`;
   - a non-world Instance (`Part`) → fails closed on the seed/layout check (still named, never a silent pass).
   *f32 note:* `@lune/roblox` stores number attributes as f32, so the spec uses the f32-exact seed **204817**; the
   engine's doubles are exercised in Play (the trace records `2134783819` exactly).

③ **R1 (test-only, restored):** the two C1 ordering assertions dropped in the previous test rewrite are back in
   `tests/tools/test_atlas_acceptance.py` — the **first** `AtlasWorldTrace` publication must precede
   `WorldSeed.requireApplied`, and there are **exactly two** publications (before the check + refreshed after it).
   `tests/atlas/Server.luau` is untouched (byte-identical to the accepted Plan 4 state).

## 3 · Freeze evidence

| # | Command (fresh checkout of `35caa02`) | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `35caa02…`; tracked tree clean (792 pre-existing untracked evidence-dir lines) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 85 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_atlas_acceptance.py'` | **Ran 16 tests … OK** |
| 4 | `python3 -m unittest discover -s tests/tools` | **Ran 315 tests … OK** rc 0 |
| 5 | `./.tools/bin/lune run tests/unit/run.luau world-seed` | **unit: 11 passed, 0 failed (2 specs)** — includes the real-Instance case |
| 6 | `./.tools/bin/lune run tests/unit/run.luau` | full suite on the fix tree (`board/probes/team-d/lune-unit-play4fix-20261102.log`) |
| 7 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 8 | `git show --stat 35caa02` | 3 files, +35/−1 (`WorldSeed.luau`, world-seed spec, atlas acceptance test) |

**Instance-root proof (verbatim from the Lune spec run):** `unit: 11 passed, 0 failed (2 specs)`; the new case asserts an
Instance root is accepted with `fastPath=true / rebuilt=false / worldSeed=204817` and that nil/`{}`/string/number produce
`the world root is required`, a mismatching Instance produces `named seed not applied before Play`, and a non-world
Instance fails closed.

**Mutation check (maker self-check):** reverting the assert to `type(root)=="table"` makes the Lune world-seed spec fail
(3 failures: Instance accepted case collapses to the named root message); restoring passes 11/0.

## 4 · Next (per Lead's sequencing)

On checker delta ACCEPT of `35caa02`: **repeat Play A once** in the current fresh opening (opening at **1/3**; repeat →
2/3, third Play reserved): `preflight` → one `run atlas:scout --seed 2134783819` → `verify-clean`, recording a clean
**`fastPathApplied=true`** with the identity evidence (actual==expected, `rebuild=false`, starter applied) and sealed
hashes unchanged. Play B (abort-contact) stays gated behind the fixture-geometry ruling; Play 3 reserved.
