# B1 opening (post-ACCEPT) — traffic retry did not run; atlas/moving surfaced two new findings (2026-11-01 15:46–15:47 ET)

Studio pid 327254, window 2200×1000 fitted, sealed place `c6651793…`, release `84324d43…`, 0/3 Plays.
Prep: MCP verify → re-sync B1 tree (`{"synced":101}`) → verify-clean clean (no latch). Log:
`b1-queue-retry-chunk-20261101.log`.

## Deviation: the approved traffic retry was skipped by the queue checkpoint

`queue --resume` did **not** run `traffic:abort-contact`: the checkpoint's `done` list already contained the three
`traffic:abort-contact` entries (exit 1 from the pre-fix attempts — queue appends any usable result to `done` and
advances). It therefore ran the remaining entries and finished the queue:

```
{"fixture":"atlas:scout","exit":1,...}
{"fixture":"atlas:highway-extension","exit":1,...}
{"fixture":"moving","exit":1,...}
{"queue":"complete","done":[… traffic:abort-contact ×3 (exit 1) …, atlas:scout, atlas:highway-extension, moving]}
```

So **the accepted chunk transport has still not been exercised in Play**; `--resume` cannot rerun it (queue is
complete). The next attempt must be a single direct `python3 tools/vinegar_run.py run traffic:abort-contact`
(possibly after a fresh queue file if a queue is wanted). 3/3 Plays consumed; `verify-clean` clean afterwards; Studio
alive in Edit; sealed hashes unchanged before/after.

## Finding A — atlas plan/manifest vs the actual world (seed/layout mismatch)

Both atlas cells fail at Play start:

`ServerScriptService.Nightshift.AtlasServer:74: actual seed/layout differ; regenerate plan, never infer`

Run dirs: `docs/verify/2026-10-19/vinegar/20261101T204612Z-atlas-scout/`,
`…20261101T204628Z-atlas-highway-extension/` (report.json runtime error; abort path clean).

Offline cause: the atlas manifest is the frozen plan `seed=2134783819, layoutVersion=2, highwayGeometryVersion=2`
(`tools/atlas_acceptance.py:109-117` from `map_atlas.BASELINE_REF`), and `AtlasServer` asserts the world reports
exactly that. The world only reports it when the **named-seed path** is installed (`--seed`, which patches
`World.dailySeed` and rebuilds); the queue has no per-entry `--seed` option, so `atlas:scout` etc. were invoked with
`seed=None` → no named-seed patch → actual daily seed differs → hard fail. This is a **queue/runbook wiring gap**:
v1.queue's atlas entries cannot pass as written; the intended invocation is `run atlas:scout --seed 2134783819`
(and likewise for the other atlas batches), or the queue needs a seed option. Design decision for the Lead.

## Finding B — moving/first-fare probe contract before world-ready

`moving` fails with:

`FixtureFailed: frame-001.jpg: probe reply lacks sidecar field(s) seed, lighting.technology (docs/art/SIDECAR.md)`

Run dir: `docs/verify/2026-10-19/vinegar/20261101T204642Z-moving/`. The JPEG capture **was written**
(`frames/frame-001.jpg`, 309,009 B — capture path works in Play, header JPEG), but the client probe reply lacked
`seed`/`lighting.technology`, so no sidecar was written (the contract keeps the frame on disk and fails closed). The
fixture appears to capture before the world/lighting contract fields are available — a real first-fare/moving fixture
timing finding to diagnose, independent of the chunk transport.

## Not observed

`viewportMatchesPng: true` at the fitted window was **not** observed: atlas failed before any view was captured and
moving failed before its sidecar. The Edit-side fitted precondition (1837×744 = viewport) still holds; the Play-mode
confirmation is still owed.
