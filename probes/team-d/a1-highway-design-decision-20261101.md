# A1 highway.spec — diagnosis and STOP (spec stale vs the current design), 2026-11-01 12:22–12:24 ET

MCP `execute_luau` (Edit mode, read-only world build, seed 918273, pinned `b67abdd` sources synced into the open
sealed place). Raw outputs: `a1-highway-diagnosis-20261101.txt`, `a1-highway-spacing-20261101.txt`.

## 1 · Diagnosis output (exact, from the running Studio)

```
routes=2  offsets=3  lanesPerDirection=3  laneOffsets=[12,24,36]  geometryVersion=2
baselineMainlineLength=6194.232152887739  baselineTurnMiter=3.178738756743869
per-lane loops (6): clockwise/counterclockwise × offset 12/24/36
  polyLength  10467.6 … 10925.3   baselineLength  5965.4 … 6423.1   count=9   cars=9 each
totalCars=54
min spacing ratio per lane (gap / loop length): 0.0744 … 0.0935
```

The code rule (`src/shared/TrafficSim.luau:312`):
`count = min(10, ceil(HIGHWAY_CARS_PER_LANE * poly.length / baseline))`, with `HIGHWAY_CARS_PER_LANE = 5`;
`poly.length ≈ 1.7 × baseline` on these curved lane loops → **9 cars/lane, 54 total**. The spec expects exactly
**5/lane, 30 total**, and normalises spacing by loop length (`>= 0.1`), so it fails before the density/spacing checks.

## 2 · What the finding is: the SPEC is stale, not a traffic-logic regression

Evidence of deliberate design change after the spec was last touched:

| Evidence | Value |
|---|---|
| Commit that introduced density scaling | **`bdcfcb9`** "Expand the seeded highway with bounded scenery and feasible shared traffic", 2026-10-18 13:31 ET, ancestor of `b67abdd` |
| That commit's `TrafficSim.luau` additions | `baselineMainlineLength`, `count = min(10, ceil(5 * poly.length / baseline))`, two-pass car IDs ("Keep the original first five IDs per highway lane; additional candidates receive IDs above30") |
| That commit touched `tests/studio/highway.spec.luau`? | **No** (`git show bdcfcb9 -- tests/studio/highway.spec.luau` empty); it updated unit specs (`highway-candidates`, `highway-expansion`, traffic-world-seam) instead |
| Studio spec's last change | `c89edb9`, **2026-10-14 19:47 ET** (four days before the design commit; still says "five cars each") |
| Design contract `docs/TRAFFIC_SPEC.md` row 7 | "Initial spacing in every lane … **never closer than 0.6 slot**" — measured minimum is **0.669 slot** (0.0744 × 9), so the current cohort **meets the design contract** |
| Design doc `docs/TRAFFIC_SPEC.md` line 56 | "one per ~1,100 studs, and **26 of 54 cars** …" — the documented current density is **54 cars**, exactly what the code produces |
| Oct-18 gate (`final-candidate-gate.log`) | passed the old 5/lane spec on the pre-`bdcfcb9` build — consistent with the spec predating the design change |

The spacing metric in the spec (gap / loop length ≥ 0.1) is not the design metric (gap / slot ≥ 0.6); longer curved
loops make the old normalisation fail by construction while physical spacing is unchanged (~1,100–1,200 studs,
matching the design doc). **Conclusion: stale Studio spec, not a code defect.**

## 3 · Per the Lead's constraint: STOP — no spec change and no code fix made

A spec change is a design decision, so this stops here for a ruling. Two options, with a recommendation:

1. **(Recommended) Update `tests/studio/highway.spec.luau` to the current design**: derive the expected per-lane
   count from the same graph inputs (or assert all six lanes equal + total equals the sum), and replace the
   gap/loop-length bound with the design contract **≥ 0.6 slot**. No production-code change; re-run A1.
2. If the design actually intends a fixed 5/lane, change `TrafficSim.new` back to a fixed count and update the design
   docs — but that contradicts `docs/TRAFFIC_SPEC.md` (54 cars) and the `bdcfcb9` intent.

**A1 therefore stays RED** until this design decision lands (the failure is a stale acceptance test, not a product
defect); **B1 stays blocked**. Nothing was edited: no spec, no code, no build, no re-pin. Studio remains open in
Edit; sealed candidate artifacts unchanged.
