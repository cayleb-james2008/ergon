# TEAM D — ATLAS ZONES FREEZE: per-zone overhead substitution + named limitation (Nox run-7 B+A), 2026-11-02

Nox ruling 20261102T051200Z: **B + A adopted**, **C rejected for now** (settings/draw-distance experiments change what
the cell tests; revisit only if per-zone views hit the same freeze), **D rejected**. Offline only — no Play.
Machine `outis-desktop`, spend $0.

**Freeze tip: `d83efa88348ef055ece2602843b5986482677c55`** (`d83efa8`, test-pin follow-up; tree
`d93b9970b0760cd084d69c1d148c95cfe369c45a`) over **`701f70c331fc0f38423f4f2284b380bba78bc99b`** (`701f70c`, the B+A
plan/validator/server/client/runbook commit; parent `1e8b5b7`; 9 files +71/−24). Tracked tree clean; sealed artifacts
unchanged (`84324d43…` release, `c6651793…` test/place). Log: `board/probes/team-d/atlas-zones-freeze-20261102.log`.

## 1 · B — authorised substitution (implemented, substitution only)

The single **whole-highway overhead** framing (`highway-overhead`) is replaced in the scout plan by **four bounded
per-zone functional overhead views**, each anchored on **ONE native highway road model**:

| Zone view | zonePath (native anchor) | label |
|---|---|---|
| `highway-zone-1-overhead` | `Highway/Roads/Road_h_top_05__h_top_06` | highway-mainline |
| `highway-zone-2-overhead` | `Highway/Roads/Road_h_east_03__h_east_04` | highway-turn |
| `highway-zone-3-overhead` | `Highway/Roads/Road_h_access_030__h_access_031` | highway-access |
| `highway-zone-4-overhead` | `Highway/Roads/Road_h_access_062__h_bottom_09` | highway-merge |

- `tools/map_atlas.py`: the four views are generated deterministically from the graph's highway contexts (bounded
  `ground`/planned cameras), tagged `zonePath`/`zoneLabel`; the whole-highway overhead view is removed; the scout count
  assert is now `24 − 1 + len(highway_contexts)` → **scout = 27 views**.
- `tests/atlas/Server.luau`: an overhead view with `zonePath` anchors on **that exact native road model** and records
  `actual.zonePath`/`actual.zoneLabel`; `city-overhead`/`world-overhead` keep their original anchors.
- `tests/atlas/Client.luau`: view cap raised `<=24` → `<=32` (27-view plan).
- `tools/atlas_acceptance.py` (scout plan only; other batches untouched): requires ≥2 distinct zone overhead views, each
  anchored under `Highway/Roads/Road_`, `zone=='highway'`/`kind=='overhead'`, and **no `highway-overhead` framing**;
  per-view zone identity + non-empty native-bound checks. The same audited capture path applies (settle/retry,
  `capture-attempts` consistency, fog-capability and viewport records).
- Tests: plan/validator counts updated (scout 27), the no-seed **golden digest recomputed on the authorised substituted
  plan**, capture-index mutation phase updated; new tests pin the zone list/paths/distinctness, that the validator
  rejects a stripped plan / non-highway anchor / duplicate zone path, and the plan/server/client/validator wiring.

## 2 · A — the limitation, named and adopted

`board/probes/team-d/overhead-render-limitation-contingency-20261102.md` is now the adopted **named renderer/Wine
limitation**: the **whole-highway overhead framing fails closed on Studio `0.745.0.7451137` / Wine 11.0 (Vinegar
1.9.4)** — Play 7 evidence: after the 1.0 s overhead settle, three captures 1.0 s apart were **byte-identical**
(`sha 74d4f7e9934dd49d…`, the view-020 frame) with **`cameraDelta:0`**, client had acked view 021, `capture-attempts`
audit **21/21 consistent**; verdict for that one framing = **fail closed, never a pass claim**. Play 6's userland
minidump did not repeat (intermittent Wine shutdown crash); no kernel GPU evidence. **One retest of the whole-highway
framing per new Studio build is authorised** and closes the limitation if it captures fresh. C/D rejected; no other
plan/validator change.

## 3 · Test / mutation evidence

- **Mutation M18** (zonePath anchored on the city instead of a highway road) → `test_map_atlas` FAILED (1) and
  `test_atlas_acceptance` FAILED (2 failures, 4 errors).
- **Mutation M19** (validator substitution requirement removed) → `test_atlas_acceptance` FAILED (2).
- Both restored; tracked tree clean.

## 4 · Freeze evidence (fresh checkout of the tip)

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `d83efa8…`; tracked tree clean (792 pre-existing untracked evidence-dir lines, R2 cosmetic) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 91 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_atlas_acceptance.py'` / `'test_map_atlas.py'` / `'test_atlas_capture_index.py'` | 22 / 5 / 7 … OK |
| 4 | `python3 -m unittest discover -s tests/tools` | **Ran 327 tests … OK** rc 0 |
| 5 | `./.tools/bin/lune run tests/unit/run.luau atlas` | **10 passed, 0 failed (4 specs)** |
| 6 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 923 passed, 0 failed (127 specs)** rc 0 (~12 min; green at the B+A code commit `701f70c`; the tip adds Python tests only) |
| 7 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 8 | `git show --stat 701f70c; git show --stat d83efa8` | 9 files +71/−24 (B+A) + 1 file +36 (test pin) |

**Exact checker reproduction:** `git checkout d83efa88348ef055ece2602843b5986482677c55` fresh worktree → `git status`
empty; run rows 2–6 (no Play/Studio needed); sealed row 7. Spot checks: `V.atlas_acceptance.manifest('scout')[0]` has 27
views including exactly the four `highway-zone-N-overhead` with the zonePaths above and **no `highway-overhead`**;
validator rejects a stripped plan with `zone overhead views missing or duplicated` and a non-highway anchor with `zone
overhead must anchor on one native highway road`; client cap `<=32`; server anchors `geometry(view.zonePath)` and records
the identity. Mutations M18/M19 fail as above.

## 5 · Next (per the Lead/Nox sequence)

On checker delta ACCEPT: **one atlas Play** in the current recovered opening (**1/3 used, 2 left**) validating the
per-zone views — audited capture path/`capture-attempts` consistency, `fastPathApplied`, fog record,
`viewportMatchesPng`, retention; then `studiocycle` to a **fresh opening for Play B** (abort-contact, fixture A+C frozen
at `1e8b5b7`, its checker delta running). Reserved Plays stay reserved. Studio currently idle; no Play performed here.
