# A1 finding — highway.spec fails on the pinned candidate (2026-11-01 12:19–12:20 ET)

Run: Street **A1** in the owner-opened Studio (`usg-desktop.rbxlx`, Edit mode), from the pinned `b67abdd` tree,
MCP-only (`tools/studio.py sync --test` → `tools/studio.py tests`), **no rebuild**.
Evidence log: `board/probes/team-d/a1-in-studio-20261101.log`.

## Result

| Step | Result |
|---|---|
| `tools/studio.py sync --test` | `{"synced":101}` — rc 0 |
| `tools/studio.py tests` | **rc 1** — 10 of 11 spec modules report PASS; **`highway.spec` FAILED** |
| Studio after | still open, Edit, place `usg-desktop.rbxlx` (MCP); runner place hash `c6651793…` unchanged |
| Pinned artifacts | street `84324d43…`/`c6651793…` unchanged before and after; no rebuild performed |

PASS entries (exact counts from the JSON report): `customization` 3 · `visual` 6 · `showroom` 15 · `game` 19 ·
`baseline-mechanics` success (3 checks) · `gameplay-settings` 4 · `driving` 29 · `driving-modes` 7 ·
`traffic-shared` 9 (internal timing p95 3.82 ms) · `ui` 4.

## The failure (kept as a failure — not re-run to pass)

```
highway.spec: run failed: ServerScriptService.Nightshift.tests.highway.spec:248: …:192:
  highway traffic must spread evenly across both directions and all six lanes
… function trafficCohortChecks (:192) … (:231) … function run (:223)
```

The spec (`tests/studio/highway.spec.luau`, deterministic seed **918273**) builds the shared world and requires
**exactly five cars per directional lane-offset key** (2 directions × 3 lane offsets = 6 keys, 30 cars total). It
fails on the per-lane `count == 5` assertion before checking spacing. The later density check (`trafficCount == 30`)
and spacing ratio are therefore not reached.

## Historical comparison (evidence, not a cause claim)

`docs/verify/launch-20261018/final-candidate-gate.log` records `highway.spec` **PASS** on a different build
(`release.rbxlx 31ee69cbb4b1d594`, 1,834,345 B) — `"passed":2`, 100 seeds, `minimumTrafficSpacingRatio
0.16173678…`. The pinned candidate (`84324d43…`, 1,990,548 B, 2026-10-21) fails the same spec. That is a
behaviour change between those two builds; the exact commit introducing it is not yet identified.

## Why this matters / next

- Matrix rule: **A1 must pass before B1**; the native Play cells are not attributed to this candidate while A1 is
  red. The failure is a real finding, exactly what the never-run set was meant to surface.
- No fix attempted and no re-run-to-pass. Next step (successor): diagnose the cohort counts on the pinned tree via
  Studio MCP `execute_luau` (print per-key car counts and the `trafficRoutes`/`trafficLaneOffsets` lengths), then a
  minimal fix in a new commit on a prep branch, rebuild **from `b67abdd`**, re-sync, re-run A1. B1 waits for green.
- Console warnings seen during the run are expected on an unpublished local place: imported-visual "needs four named
  tires; using authored fallback" and `DataStore unavailable … publish this place` / save-failed lines.
