# B1 Play 2 — seeded atlas:scout hangs before view-001 (2026-11-01 15:53–16:03 ET)

Studio pid 382409, window 2200×1000 fitted, sealed place `c6651793…`, release `84324d43…`. Prep: MCP place/Edit
verified, `verify-clean` clean, then direct `run atlas:scout --seed 2134783819`. Log:
`b1-atlas-seeded-opening-20261101.log`. Run dir:
`docs/verify/2026-10-19/vinegar/20261101T205326Z-atlas-scout/`.

## Result — exit 1, timeout, no third Play

```
{"fixture":"atlas:scout","exit":1,"error":"TimeoutError: collector did not receive view-001 within 600s"}
states: LOCKED→PREFLIGHT→STAGED(2.2)→PREPARED(2.3)→PLAYING(4.2)→RAN(7.4); abort stop/cleanup/verifyClean all ok
collector phases: ["prepared","cleanup"]   (no runtime, no views)
report.json: prepared ok=true (namedSeed 2134783819, verifiedSources 101, productionSourcesUntouched true)
verify-clean after: clean=true, latch null
```

So the seed invocation got **past** the earlier identity failure (the unseeded run failed fast at `AtlasServer:74`
seed/layout); this run prepared with `namedSeed=2134783819`, started Play, and then hung for the full 600 s without
sending view-001 — and without even the starter's failure report (collector saw only `prepared` + `cleanup`), i.e. the
server-side atlas start neither completed nor failed visibly.

## Likely path (offline reading)

`tests/atlas/Server.luau` waits for the world (`WorldReady`), then with `manifest.namedSeed` calls
`WorldSeed.ensure(World,root,seed)`. The atlas prepare does **not** disable Bootstrap (prepared report has no
`bootstrapDisabled`), so the world is already built with the day's seed; `ensure` therefore takes the rebuild path
`World.generateSafe(value, root)` (`src/server/World.luau:421` → `World.generate` on an already-populated world,
which clears and rebuilds City+Highway in a running Play). That rebuild never returned within 600 s in this session — no
error, no fallback warning, no runtime report. The fast path exists (the AtlasServerStarter patches `World.dailySeed`
before `World.generateSafe`, but only if the starter runs before Bootstrap finishes building the world, which it did
not here).

This is a **fixture/tooling finding** (named-seed rebuild not viable in Play as implemented), not a capture/transport
failure. Needs bounded instrumentation: record seed before/after, `WorldSeed.ensure` entry/exit, `generateSafe`
phase timings (city/highway/art), and whether it ever completes; options: disable Bootstrap and let AtlasServer build
the named-day world from scratch, or pre-seed the world before Play, or enforce the fast path ordering.

## State

Plays 2/3 used; Studio alive in Edit (pid 382409); place clean; sealed artifacts unchanged (`84324d43…` release,
`c6651793…` test/runner place). Play 3 (atlas:highway-extension) **not attempted** per the stop-on-Play2-failure rule.
`viewportMatchesPng: true` still not observed.
