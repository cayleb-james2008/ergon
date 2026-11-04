# Slice D — post-generation QA checks (raycast reachability, mesh budget, texture tiling)

Date: 2026-11-04
Source: `fleet-scratch/overnight/roblox-day/SLICE-D-REPORT.md`
Fences: board only. No Studio, no `tools/usg test`, no sealed-place touch, no credentials, no third-party skill files.

## Where the work stands

Worktree `~/Work/tmp/usg-slice-d-wt`, branch `prep/slice-d-qa-20261104`, files **untracked/uncommitted** (`main` @ `a43b7e4` untouched). Lead independently re-ran the tests: **14 passed in 0.10s** (`pytest tests/tools/test_mesh_budget.py tests/tools/test_raycast_reachability.py tests/tools/test_texture_tiling.py`). Tools are ready for review, not yet landed on trunk.

## Files added (new files only; existing tools untouched)

| File | Lines | What |
|---|---|---|
| `tools/raycast_reachability.py` | 127 | Reachability/gap validation over the repo's atlas/map data model (schema read from `tools/map_atlas.py` first). Fail-closed on missing position data — never fakes a PASS. |
| `tools/mesh_budget.py` | 96 | Triangle/vertex budget report: per-asset counts vs `--body-max`/`--total-max`/`--vertex-max`. Smoke run over real exports flagged overruns. |
| `tools/texture_tiling.py` | 117 | Tiling-artefact check over GLB texture metadata (image dimensions, repeat/offset). Smoke run: no repeat-wrapping issues found — all textures use clamp; exported GLBs carry no image width/height, so the check flags repeat+dimensions where present and would fail repeat+non-power-of-two if dimensions appeared. |
| `tests/tools/test_mesh_budget.py` | 71 | Test suite. |
| `tests/tools/test_raycast_reachability.py` | 62 | Test suite. |
| `tests/tools/test_texture_tiling.py` | 73 | Test suite. |

Tests: **14 passed** (lead re-run). No existing test files modified.

## Honest limits

- **Raycast reachability is a static check** over the map-atlas data model, not a live Studio raycast. Missing part positions → fail-closed, not a PASS. Live reachability needs a Studio run — contained Studio only, past all gates until the owner playtest verdict.
- **Texture tiling is metadata-only**: declared dimensions, repeat/offset, wrap mode. It cannot see seams or sampling artefacts on GPU — those still need rendered frames (Stage B3 look-in-motion).
- **Mesh budget covers exported GLB metadata on disk today.** Raw AI meshes not through the repo's export path have no metadata → NOT_RUN by design.

## Next bounded step

Nox/lead review of the three tools, then decide whether to land `prep/slice-d-qa-20261104` into `main` (a docs+tools-only commit, no `src/` change) and wire the three checks into the inspect stage (`tools/look_metrics.py` / `map_atlas.py` pipeline) as follow-up work. Nothing here publishes or uploads.

## Reproduction

```bash
cd ~/Work/tmp/usg-slice-d-wt
python3 -m pytest tests/tools/test_mesh_budget.py tests/tools/test_raycast_reachability.py tests/tools/test_texture_tiling.py -q
# 14 passed
python3 tools/mesh_budget.py --help
python3 tools/texture_tiling.py --help
```
