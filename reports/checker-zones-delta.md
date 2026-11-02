# CHECKER — B+A zones delta (`d83efa8` over `1e8b5b7`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the Nox run-7 B+A zone-overhead substitution (commits `701f70c` code + `d83efa8` tests)
on top of my accepted fixture A+C delta (`1e8b5b7`). I built none of it. No Play, no Studio input, no image reads, no
host input, **$0**. Ran from a throwaway clone `/tmp/checker-zones/repo` (2026-11-02T00:33:18-05:00) and a second clone
for mutations; maker tree only read.

- **Subject tip:** `d83efa88348ef055ece2602843b5986482677c55` (`d83efa8`), parent `701f70c` (zone substitution), grandparent
  `1e8b5b7`; tree `d93b9970b0760cd084d69c1d148c95cfe369c45a`; tracked mods **0**; sealed
  `84324d43…`/`c6651793…` unchanged before/after and at end.
- **Delta over `1e8b5b7`:** exactly **9 files, +107/−24** — `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` (+8),
  `tests/atlas/Client.luau` (+1/−1), `tests/atlas/Server.luau` (+8/−1), `tests/tools/test_atlas_acceptance.py` (+39/−1),
  `tests/tools/test_atlas_capture_index.py` (+1/−1), `tests/tools/test_map_atlas.py` (+5/−1),
  `tests/tools/test_vinegar_run.py` (+14/−13), `tools/atlas_acceptance.py` (+11), `tools/map_atlas.py` (+20/−6). No
  `src/`, `assets/`, `build/` change; `tools/vinegar_run.py` and `tests/traffic/Server.luau` untouched → settle/retry/
  audit/45 s bound/assert-and-skip/fixture A+C intact.
- **Maker report:** `board/reports/team-d-atlas-zones-freeze-20261102.md` is **absent** (as the Lead anticipated); this
  verification is from the commits + the A note + Play-7 report.

## Verdict in one line

**ACCEPT** — B is exactly scoped: the single whole-highway overhead view is replaced in the scout plan by four bounded
per-zone functional overheads (each anchored on one native `Highway/Roads/Road_<edge>`), the validator enforces the
substitution and per-view zone identity, the server anchors on `view.zonePath` and records it, the client cap is 32, and
the zone views run through the same audited capture path (settle/retry/`capture-attempts`), with no other plan/validator
behaviour change; A is recorded as a named Studio `0.745.0.7451137` / Wine 11.0 (Vinegar 1.9.4) limitation with the
Play-7 evidence (3 attempts byte-identical, `cameraDelta:0`, audit 21/21), a fail-closed verdict and the
one-retest-per-new-build rule; the drop-a-zone mutation fails the pins; all suites are green (91 / 22 / 327 / Lune atlas
10-0 / full Lune 923-0).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① B substitution exactly scoped | **PASS** | `tools/map_atlas.py:219-235`: the old `for zone in ('city','highway')` overhead add is gone; `city-overhead`/`world-overhead` keep their anchors; four deterministic zone views `highway-zone-1..4-overhead` are appended with `zonePath='Highway/Roads/Road_'+edge['id']`, `zoneLabel`, camera/lookAt/fov, `status:'planned-unseen'`; scout picks them (`:283`) and the count assert is `24-1+len(highway_contexts)` = **27** (`:289`). `tools/atlas_acceptance.py:399-406`: scout validation requires ≥2 distinct zone views, all `zonePath` under `Highway/Roads/Road_`, `zone=highway`/`kind=overhead`, and `no v['id']=='highway-overhead'`; `:350-352` per-view `actual.zonePath/zoneLabel` identity + non-empty native bounds. `tests/atlas/Server.luau:171-178`: `if view.zonePath then anchor=geometry(view.zonePath);actual.zonePath=view.zonePath;actual.zoneLabel=view.zoneLabel` (city/world keep original anchors). `tests/atlas/Client.luau:77`: cap `#plan.views<=32`. No other plan/validator change: the remaining diffs are the authorised count/golden-digest/capture-index updates and runbook; `highway_contexts` was only moved earlier for reuse. Sealed place/build untouched; no production/game code (`src/` untouched) |
| ② A limitation note | **PASS** | `board/probes/team-d/overhead-render-limitation-contingency-20261102.md` (updated 2026-11-02T00:27, read-only): names the whole-highway framing as failing closed on **Studio `0.745.0.7451137` / Wine 11.0 (Vinegar 1.9.4, proton prefix studio)**; Play-7 evidence — after the bounded 1.0 s overhead settle, **three captures 1.0 s apart byte-identical** (`sha 74d4f7e9934dd49d…`, view-020 `city-overhead` image) with **`cameraDelta:0`**, client had acknowledged view 021, `capture-attempts` audit **21/21 consistent** (`result.json.captureAttempts` == `capture-attempts.json`); verdict for that one framing **fail closed (named), never a pass claim**; **retirement rule: one retest per new Studio build**, closes if fresh. B adopted substitution documented; C/D rejected. Corroborated by `board/reports/team-d-diagnostic-play7.md` (`sha 74d4f7e9…`, `cameraDelta:0`, 21/21 entries equal) |
| ③ Tests pin the substitution; audited path preserved | **PASS** | `tests/tools/test_map_atlas.py`: scout rows = 27, zone ids exactly `highway-zone-1..4-overhead`, no `highway-overhead` in scout, all `zonePath` start `Highway/Roads/Road_`. `tests/tools/test_atlas_acceptance.py`: `test_scout_plan_uses_per_zone_overheads_and_rejects_the_whole_highway_framing` (zone list/shape, whole-highway absent, validator accepts substituted plan and rejects stripped/bad-anchored/duplicated plans) + source-shape wiring test (plan `zonePath`/`24-1+len(highway_contexts)`, server anchor/identity, client `<=32`, validator "must be substituted"). `tests/tools/test_vinegar_run.py`: 27 views / 27 captures / 28 with one retry / **6 overhead 1.0 s settles** / audit == result (27 entries) — i.e. the zone views use the same settle/retry/audit path. `test_atlas_capture_index.py`: invalid phase updated to `view-099` for the 27-view plan. Runs: runner `Ran 91 … OK`; atlas acceptance `Ran 22 … OK`; full Python `Ran 327 … OK` rc 0; Lune atlas filter `10 passed, 0 failed (4 specs)`; `atlas-paths 1/0`; full Lune `923 passed, 0 failed (127 specs)` rc 0 |
| ④ Mutation | **PASS** | Throwaway clone: **M21** drop the `highway-merge` zone (4→3 zone overheads) → `test_map_atlas` **FAILED (failures=4)** (count/zone-list/coverage pins) and atlas acceptance errors at module/setup (plan shape); restored `tools/map_atlas.py` sha256 `012d0c7738f3…` (frozen), git clean. Informational: re-adding an *unused* whole-highway view without restoring the scout pick stays green (the fixture's `data['views']` is the scout id list) — the scout-plan pins are the operative ones (see R3) |
| ⑤ Scope / earlier layers | **PASS** | Exactly the 9 claimed files; no game/asset/sealed path; `tools/vinegar_run.py` and `tests/traffic/Server.luau` untouched (`ATLAS_FIRST_VIEW_TIMEOUT = 45`, `capture_attempts_max = 3`, `overhead_settle = 1.0` intact); no other plan/validator behaviour change beyond the authorised substitution |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged; maker HEAD `d83efa8`, tracked clean at end. The per-zone views will be exercised by the authorised atlas Play |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the per-zone views are proven structurally/offline (plan, validator, source-shape,
  runner fakes), not rendered. The authorised atlas Play is the live validation.
- **R2 — maker report absent:** `team-d-atlas-zones-freeze-20261102.md` is not present; verified from the two commits,
  the A note and the Play-7 report. If a report is intended, it is missing from the freeze handoff.
- **R3 — coverage nuance:** the tests pin the **scout plan** (`data['views']` = scout ids); re-adding an unused
  whole-highway view to the full planned-views list without putting it in the scout pick stays green (informational
  mutation M20). It cannot affect the fixture (scout uses only the scout ids), so this is a pinning-scope note, not a
  functional gap; a stricter test could assert the full `planned_views` list has no `highway-overhead`.
- **R4 — cosmetic:** the report/evidence untracked count (807 now) vs the "792 pre-existing" wording used elsewhere.
- **R5 — determinism note:** the fourth zone uses `accessRouteIds[-2]`/`joinNodeId` from the frozen graph; deterministic
  for the frozen seed 2134783819 (the golden digest was recomputed on the substituted plan), so the pins are stable for
  this plan.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-zones-delta/` — `delta-verification.log` (HEAD/parent/grandparent/tree/delta numstat + paths +
sealed + earlier-layer invariants + plan/validator/server/client zone wiring + A note header), `suites-and-note.log`
(A note evidence, Play-7 report evidence, maker report absence), `suites.log`, `runner.log` (91 OK), `atlas.log`
(22 OK), `python.log` (327 OK), `lune-atlas.log` (10/0, 4 specs), `atlas-paths.log` (1/0), `lune.log` (923/0, 127
specs), `mutations.log` (M21 failures; restored hash), `run-started.txt`. Throwaway copies
`/tmp/checker-zones/{repo,mut}`; maker repo read-only, unchanged, still at `d83efa8`.
