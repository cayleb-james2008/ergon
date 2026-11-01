# A1 highway.spec — design-contract alignment (prep branch, awaiting independent checker)

Status: **change made and maker-self-checked; A1 is NOT declared green.** Independent review pending (Lead spawns
the fresh checker). No production code changed, no build, no candidate re-pin, no spec change to `main`.

| Item | Value |
|---|---|
| Repo | `~/Work/projects/untitled-street-game` |
| Branch | **`prep/a1-highway-spec-20261101`** (built from the sealed candidate commit, **never `main`**) |
| Base | `b67abdd` (sealed candidate source) |
| Commit | **`1abe30391caf88db9d76bd424d87461899b5cdf9`** — "tests(a1): align highway.spec with the highway traffic design contract" |
| Files changed | `tests/studio/highway.spec.luau` only — `67 insertions(+), 20 deletions(-)` |
| Diff for review | `board/probes/team-d/a1-highway-spec-change-20261101.diff` (148 lines) |
| Sealed artifacts | street `84324d43…`/1,990,548 B and `c6651793…` unchanged before/after; not re-pinned |

## Why (design authority, as ruled)

`docs/TRAFFIC_SPEC.md` **row 7**: "Initial spacing in every lane … never closer than 0.6 slot"; **line 56**: "one per
~1,100 studs, and **26 of 54 cars**". Commit **`bdcfcb9`** (2026-10-18) introduced the density rule
`count = min(10, ceil(5 * poly.length / baselineMainlineLength))` and did **not** update this Studio spec (last changed
`c89edb9`, 2026-10-14, still asserting five cars/lane and a loop-length-normalised ≥ 0.1 spacing). The spec asserted a
stale count; the code meets the documented design.

## What changed in the spec (and what did not)

Changed:
- header now cites the design contract (`TRAFFIC_SPEC.md` row 7 + line 56) and commit `bdcfcb9`;
- new `expectedLaneCounts(graph)` derives each directional lane-offset count from the graph rule (same inputs
  `TrafficSim` uses: `baselineMainlineLength`, lane offset, `baselineTurnMiter`, `TrafficSim.laneLoop` length);
- asserts **one shared per-lane count** across all six lanes and each lane equals the derived count;
- total car count is asserted against the **sum of the derived per-lane counts** (54 at seed 918273), not a constant;
- spacing is measured in **slots** (`slot = lane length / cars in lane`) and asserted **≥ 0.6 slot** per row 7;
- result fields: `minimumTrafficSpacingSlots` (was `minimumTrafficSpacingRatio`) and `expectedTrafficCount` added;
  suites line updated to name the design contract.

Unchanged (kept): all graph/circuit checks (100 seeds), access-ramp clearance, surface/road/ground ray checks,
junction pad, arrival-bay checks, shared-world contract, `passed = 2`, `seedsPassed = 100`, sample counts.

## Maker self-check (agent-run, NOT certification)

`board/probes/team-d/a1-highway-spec-selfcheck-20261101.log` — updated spec synced into the sealed open place
(`{"synced":101}`), then `tools/studio.py tests`:

```
tests_rc=0   (all 11 Studio spec modules pass)
highway.spec: {"passed":2,"seedsPassed":100,"expectedTrafficCount":54,"minimumTrafficSpacingSlots":0.6694870414212347,
               "roadSamples":3142,"groundSamples":5,"graphNodesChecked":20686}
sealed artifacts after: 84324d43… / c6651793…  (unchanged; no rebuild)
```

## Re-check procedure for the independent checker

1. Read the diff (`a1-highway-spec-change-20261101.diff`) against `docs/TRAFFIC_SPEC.md` row 7 + line 56 and
   `git show bdcfcb9` (density rule) — confirm the assertions match the design and nothing else was weakened.
2. Confirm the diff touches only `tests/studio/highway.spec.luau`, branch base is `b67abdd`, and `main` is untouched.
3. Confirm the rule derivation is not self-fulfilling: it must fail if a lane count is perturbed or if spacing drops
   below 0.6 slot (negative control can be done offline/Studio as the checker sees fit).
4. Re-run A1 in the open Studio: `tools/studio.py sync --test` then `tools/studio.py tests` from the prep branch;
   expected `highway.spec` PASS with `expectedTrafficCount=54`, `minimumTrafficSpacingSlots≈0.669`, exit 0.
5. Confirm sealed candidate hashes unchanged before/after.

Only after the checker PASS may A1 be declared green; then B1 unblocks. Studio remains open in Edit; everything is
MCP-only, text/JSON.
