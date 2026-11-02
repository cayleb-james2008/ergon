# DESIGN NOTE — OPTION B2+A2 (per-region world overheads + named limitation) — implementation-ready, NOT implemented

Status: awaiting the Nox ruling on **B2+A2** (Lead recommendation), with the optional **plan-ordering** fold. Offline only;
**no Play, no code change yet**. Evidence base: `board/reports/team-d-diagnostic-play10.md`,
`board/probes/team-d/overhead-render-limitation-contingency-20261102.md`.

## 1 · Plan change (B2) — `tools/map_atlas.py`

Replace the single `world-overhead` view with **four bounded per-region world overhead views**, each anchored on ONE
native bounded model (never the whole world), reusing the existing `zonePath` mechanism:

| view id | zone | zonePath (native anchor) | label |
|---|---|---|---|
| `world-region-1-overhead` | world | `City/Junctions/JunctionPad_n_3_2` | world NW quadrant |
| `world-region-2-overhead` | world | `City/Junctions/JunctionPad_n_10_10` | world SE quadrant |
| `world-region-3-overhead` | world | `Highway/Roads/Road_h_top_05__h_top_06` | world north highway |
| `world-region-4-overhead` | world | `Highway/Roads/Road_h_access_062__h_bottom_09` | world south highway |

Add explicit `regionRole='world-region'` (and `zoneRole='highway-zone'` on the existing four zone views) so the validator
does not guess by path prefix; `coverageIds` stay `['world:overview']` (no coverage-test change); planned cameras computed
like other overheads (`at + 2200`). Delete the `world-overhead` add and its scout pick. Scout count becomes
**27 − 1 + 4 = 30**; the assert becomes `24-1+len(highway_contexts)-1+len(world_regions)` (or a named constant).

## 2 · Server change — `tests/atlas/Server.luau`

The zonePath anchor branch already exists; the **fit-bounds block must become zonePath-first** or world-region views would
get the whole support bounds (`view.zone=="world"` branch) and stall again:

```lua
elseif actual.overhead then
 if view.zonePath then actual.fitBounds=actualBounds(anchor)            -- NEW first branch (bounded native model)
 elseif view.zone=="world" then local s=inventory.support; …support bounds…
 elseif anchor then actual.fitBounds=actualBounds(anchor) end
```

Client (`tests/atlas/Client.luau`): **no change** (cap 32 ≥ 30). Other batches: untouched.

## 3 · Validator — `tools/atlas_acceptance.py` (scout only)

Replace the single `zone_views` rule with two role groups (explicit `zoneRole`/`regionRole`), keeping the substitution
spirit:
- exactly **4 highway zone views** (`zone='highway'`, `zonePath` under `Highway/Roads/Road_`, distinct) and exactly
  **4 world-region views** (`zone='world'`, `zonePath` under `City/` or `Highway/Roads/`, distinct within and across
  groups); ids pinned;
- **no `highway-overhead` and no `world-overhead` in the scout plan (nor anywhere in the full planned list);**
- per-view identity: `actual.zonePath == view.zonePath`, `actual.zoneLabel` present; non-empty native bounds;
- **bounded-framing proof for world regions**: each region's `fitBounds` spans must be strictly smaller than
  `report.server.support` spans (and > 1) — proves the whole-world framing is gone, not just renamed.

## 4 · Plan ordering (fold only if Nox adds it) — `tools/map_atlas.py` scout pick order

Current ordinals: …19 garage, 20 `city-overhead`, 21 `world-overhead` (blocker), 22–25 zone views, 26 perimeter, 27 grade.
New order (world regions last, zone views early): 20 `city-overhead`, **21–24 the four highway-zone views**, 25 perimeter,
26 grade, **27–30 the four world-region views**. Then a Play that freezes at a world-region still captures views 1–26
including all four B-zone captures; scout = 30 views, 9 overhead views total (city + 4 zones + 4 regions → 9 × 1.0 s
settles). Ordering is an assertion target: all four highway-zone overheads precede all four world-region overheads.

## 5 · A2 — named limitation (update `board/probes/team-d/overhead-render-limitation-contingency-20261102.md`)

Extend the adopted A note to the **whole-map overhead class**: `highway-overhead` (Play 7: 3× byte-identical, settle 1.0 s,
cameraDelta 0, audit 21/21) **and** `world-overhead` (Play 10: view-020 city-overhead fresh, then view-021 world-overhead
3× byte-identical, settle 1.0 s, cameraDelta 0, audit 21/21; city-overhead fine) — **fail-closed verdict for both one
framings**, named build `Studio 0.745.0.7451137 / Wine 11.0 (Vinegar 1.9.4)`, **one retest per new Studio build** for each
framing closes it if fresh. B2 (per-region views) is the adopted substitution; C/D remain rejected (unchanged from run-7).

## 6 · Files touched

| File | Change |
|---|---|
| `tools/map_atlas.py` | world-overhead → 4 world-region views (+ roles, ordering if ruled, count assert, html cosmetic `scout24`→`len(scout)`) |
| `tests/atlas/Server.luau` | overhead fit block: zonePath branch first |
| `tools/atlas_acceptance.py` | role-group zone/world rules, no whole-map overhead ids, bounded region spans |
| `tests/tools/test_map_atlas.py` | full-list pin: overhead set = city + 4 highway zones + 4 world regions; **no whole-map overhead anywhere**; 30 scout; ordering assertion; count 27→30 |
| `tests/tools/test_atlas_acceptance.py` | 27→30; role-group positives/negatives (missing world regions, lingering `world-overhead`, region span == support, duplicate region path) |
| `tests/tools/test_vinegar_run.py` | atlas end-to-end/retry/audit counts 27→30; overhead settles 6→9; capture calls 30 / 31-with-retry; golden digest recomputed |
| `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` | 27→30 view note + world-region substitution + limitation class |
| `board/probes/team-d/overhead-render-limitation-contingency-20261102.md` | A2 final form (class, both framings, retest rule) |
| `board/reports/team-d-atlas-zones-freeze-20261102.md` (new delta note) | B2+A2 freeze evidence |

`tests/atlas/Client.luau` unchanged (cap 32); `tools/atlas_capture_index.py` unchanged (view list is dynamic).

## 7 · Tests/mutations and Play evidence chain

- Positives: scout = 30; roles/paths/counts as §1/§3; validator accepts; bounded world-region spans < support spans.
- Negatives: drop one world region → fail; re-add `world-overhead` (scout or unused full list) → fail; region span equal to
  support → fail; duplicate region path → fail; world region anchored on `City/Roads/...`? (allowed list only) → fail.
- Mutations: **M23** delete a world-region view (plan/validator/count tests fail); **M24** revert the server fit block to
  world-first (source-shape test + Play would regress); **M25** re-add `world-overhead` only to the full list (R3 full-list
  pin fails).
- Play (after delta, fresh opening per Lead): one `run atlas:scout --seed 2134783819`; expect 30/30 captures if the
  renderer cooperates (Play 10 shows city-overhead + all sub-overhead framings capture; region framings are bounded like
  them), `capture-attempts` 30 == result 30, 9 × 1.0 s settles, fog/viewport records, then runtime/validator/index/look
  evidence and a real cell verdict. If a world-region still freezes, the named limitation covers that framing and the
  ordering rule guarantees the four highway-zone captures already landed in the same run.
- Freeze: clean tree, full suites (runner 91 / atlas N / map / Python / Lune full), sealed unchanged, commit + freeze block;
  then checker delta.

## 8 · Batch record

Written 2026-11-02T06:10Z as the offline implementation-ready note; repo tip stays `8bda598` (tracked tree clean),
no code/plan change, no Play, Studio untouched (pid 881911, opening 2/3, last Play reserved).
