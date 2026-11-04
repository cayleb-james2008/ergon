# ROBLOX lane — day 2 report (4 Nov 2026): look gate, window-finder, QA checks, owner bundles

Lead: Roblox day-2 lead (opencode/ollama/minimax-m3.1-flash:cloud). Order: Nox's five rulings
`inbox/2026-11/20261104T123100Z-nox-to-roblox-rulings.md`. Prior state: `reports/20261104-roblox-day-*.md`
(slices A-D closed) and `~/fleet-scratch/overnight/roblox-progress.md`.

**Fences held throughout:** no upload, no publication, no spend, no `tools/usg test`, the sealed place and release
untouched (`build/test.rbxlx` `c6651793…`, `build/release.rbxlx` `84324d43…`, sealed slot `c6651793…`), only the
contained Vinegar Studio, and the Studio was closed at the end of every session. Publication remains the owner's
gate; "ready" means ready.

---

## 1 · Verdict in one paragraph

All four scope items are closed on the agent side, and one of them is closed with a finding the owner has to
decide: **the facade-repetition goal of ≤ 1 cannot be reached with the art that exists** — the densest 400-stud
window holds 148 image placements over 21 distinct assets, so the metric cannot go below 8 by any assignment, and
the window-grid pair alone caps it near 12. The proof, the counting and the batch the owner must choose are filed
(`docs/art/FACADE-REPETITION.md`, `docs/art/FACADE-ART-BATCH.md`); the threshold was **not** moved to make a number
pass. The look gate is now switched on and refrozen on Nox's rulings, the Wayland Studio window-finder is fixed and
proven live, the three Slice D QA checks are landed as repo code and run against the real repo, and the four owner
gates are packaged as one-look bundles with renders. The street look itself is measured on the **shipped** look for
the first time and still misses three of the ruled bounds; those misses are named per view, with the fix recipe
and the owner of each.

## 2 · The look gate (rulings 1-4)

- **Merge + switch-on.** `prep/look-metrics-viewport-guard` (618316f) was already merged (`b905a1f`); the
  switch-on landed as `prep/look-thresholds-refreeze-20261104` → **main `f640881`** (commits `3a4db9b`, `d374809`).
  `measure` now applies the **1600x700 floor by default** (ruling 1) and `--min-viewport=off` switches it off.
- **Two defects in the branch's own draft, found and fixed before landing:** `--min-viewport=off` was silently
  ignored (None meant both "unset" and "off"; a sentinel separates them), and the view scope was a plain prefix
  check, so ruling 2's `"views": "edge-*"` matched **nothing** — the void bound would have applied to no view at
  all. It is a glob now (fnmatch).
- **Refreeze (rulings 2-4).** `docs/art/thresholds.json`: fleck ≤ 270, cyan ≤ 0.63, void ≤ 0.05 scoped to
  `edge-*`, `minViewport` 1600x700, `provisional: true` with the retired 3 Nov bounds (8253 / 0.785) recorded under
  `prior`, and the measurement basis (the 3 Nov scout pack, seed 2134783819, 1837x744, 18 valid street views).
  It stays provisional until the owner-present re-capture; every report now carries the `provisional` flag.
- **Tests.** `pytest tests/tools` → **376 passed / 0 failed** after this work (372 before), with four new tests:
  the floor is on by default, `--min-viewport=off` disables it, a scoped bound applies only to the views it names,
  and the ruled file carries Nox's numbers.

## 3 · The studiocycle Wayland window-finder (fixed, proven live)

- **The defect (3 Nov record).** Studio reaches Hyprland as a **native Wayland client** whose class is
  `robloxstudiobeta.exe`, not `steam_proton`, so `close` reported "no-studio-window" while Studio was open and
  `open` launched a **second Studio** (the double-instance blocker of 2026-11-03). The old finder also matched
  *any* `steam_proton` window, so it could have closed an unrelated Steam game.
- **The fix** (`~/Work/tools/winctl/`, outside the repo): `studio_windows()` matches both classes and requires a
  "Roblox Studio" title for a Proton window; `winctl`'s studio target accepts both classes and the start-screen
  title (the place window has the path, the start screen has "Roblox Studio" - the old target matched neither
  before the place was open); `match_target` picks the biggest match and still refuses a genuine tie.
- **Live proof, Studio closed first:** `studiocycle open` found the window and fitted it; `winctl get --target
  studio` resolves it; `studiocycle close --discard` closed it the designed way — graceful close, the "Save
  changes?" dialog appeared, the app was killed so nothing was saved, and the place hash was unchanged
  (`sealed_unchanged: true`). Tests: `Work/tools/winctl/tests` 21 OK, and the repo suite
  `tests/tools/test_place_override.py` 19 passed with a new `StudioWindowFinder` class.

## 4 · Slice D QA checks — landed, run, and what they flagged

Landed as repo code (no third-party skill files): `tools/raycast_reachability.py`, `tools/mesh_budget.py`,
`tools/texture_tiling.py` (+19 tests) → **main `f82c779`**.

- The draft's mesh budget was rewritten to the repo's **own** budgets (`ART_DIRECTION`: hero body 15k / total 30k,
  traffic 10k, props and modular facades 8k) with the same node-name split as `tools/art/validate_vehicle.py`; the
  draft's aggregate cap (30 000 over every GLB in the repo) would have failed every run. Shipped assets can fail
  the pass; source and raw AI meshes are listed as optimisation candidates, because the LOD in
  `assets/export/traffic-lod` is what ships.
- **Runs on the real repo:** mesh budget — 30 meshes, 437 048 triangles, **2 shipped facade modules over the 8k cap**
  (8298 and 11 050; recorded as reviewed exceptions with the reason, the date and the fix recipe — they are
  full-building facades, and the world places 4 of them); **10 optimisation candidates** (the traffic source models
  at 20.2k and the raw Forge meshes at 14-16k — the raw-mesh list the addendum asked for). Raycast reachability
  over the real atlas graph — **440 edges, 367 nodes, 0 unreachable, 0 sample gaps, 0 data gaps**. Texture tiling —
  60 textures, **no repeat-wrapped texture anywhere** (every material clamps), and a repeat-wrapped texture without
  recorded dimensions is now reported as a data gap instead of a false non-power-of-two finding.
- After the merge: `pytest tests/tools` **399 passed**, Lune unit **943 passed / 0 failed (130 specs)** — identical
  to the recorded baseline, so nothing in the game moved.

## 5 · Street polish — measured on the shipped look, and the honest limits

The first measurement of the **production** look (visual flags in their shipped state, not the A/B baseline):
`docs/verify/2026-10-19/vinegar/20261104T163738Z-atlas-scout`, 30 images, **18 valid street views at 1753x744**
(one viewport, above the 1600x700 floor). Full table: `docs/verify/2026-11-04/STREET-LOOK-GATE.md`.

| bound | production look | the 3 Nov flag-off baseline |
| --- | --- | --- |
| road luminance ≥ 0.6 | **4 of 18 under** (worst `cell-9-5-left` 0.011) | 7 of 18 under (worst 0.35351) |
| fleck ≤ 270 | **2 over** (`cell-3-2-front` 510.7, `cell-7-9-front` 382.9) | 1 over (`city-garage` 336.9) |
| cyan ≤ 0.63 | **1 over** (`city-road-n_3_2__n_4_2-forward` 0.802) | 1 over (the same view, 0.7846) |
| facade repetition ≤ 1 | 13 | 13 |

**After the lane's polish** (the wet-night reflections rebuilt as narrow, dim, mostly transparent streaks - commit
`e2eb15e`; pack `20261104T165505Z-atlas-scout`, same seed and camera, place `32406439…`): the fleck bound now
**passes on every view** (the two outliers 510.7 and 382.9 are 6.9 and 2.8 - the hard slab edges were the fleck),
road-luminance failures drop from **4 to 2** (`cell-9-5-left` 0.011 -> 0.694), and the cyan view gets slightly worse
(0.802 -> 0.855), which says the cyan is the scene's own blue cast over the asphalt, not the decorations. The
candidate to grade is `build/rc/rc-e2eb15e.rbxlx` (`32406439…`).

Two findings the owner needs:

1. **The ruled bounds were derived on the flag-off baseline, and the shipped look is what they must bound.** The
   cyan offender is the wet-night neon streak: with `wetNight` on (the shipped state) the road carries the
   district's cyan neon; the 3 Nov baseline had that flag **off**, so it never measured the look that ships. The
   ruling (3) therefore needs either re-derivation on the production pack or an art decision to take more cyan off
   the road — that is Nox's call, not the lane's.
2. **The reflections are painted, not reflected** — `WetLampPool` and `WetNeonStreak` are solid `Neon` quads at
   transparency 0.5/0.55, and in the frame they are flat opaque slabs on the asphalt (yellow under every lamp, a
   cyan slab across the carriageway). The art bible asks for "long, vertical, blurred streaks" and pillar 2 forbids
   "cyan glass sheen rectangles"; their hard edges are also what the fleck metric is measuring. **The lane's fix:**
   narrower (streaks 3.6 → 2.4, pools 10x15 → 7x11), dimmer (0.5/0.45) and far more transparent (0.82/0.78) —
   a mechanical change to `addWetNight` in `src/server/CityBuilder.luau`, verified by a fresh capture.

Also named, with the owner of each: the **black rectangle in the sky** (the map's edge showing where the horizon
must be a lit skyline — the `horizonEdge` flag is on and does not cover that direction) and the **dark
mid-distance** in the cell streets (a lighting-design change, not a parameter tweak).

## 6 · Facade repetition — the counting proof (the owner's decision)

`tools/facade_placements.luau` on the real generator, seed 2134783819: 1906 rows, 460 building models, 24 assets.
The metric's 13 is a true reading: every asset repeats within 6-141 studs, so the world's frontage really does read
as the same faces over and over.

- The **densest 400x400 window holds 148 image placements over 21 distinct assets** → by pigeonhole the worst
  count cannot go below **8** with today's art; the window-grid pair alone (23 placements, 2 sheets) caps it near
  **12**; the looser reading of the rule ("the same design must stay 400 studs from its last copy") needs ≥ 8
  designs per category, and the categories hold 2, 5, 2, 7 and 4.
- **Reaching 1 needs ≈ 148 distinct frontage designs**, or the art bible's own PBR route (frontage as MaterialVariants
  in Model assets rather than repeated image decals), or more authored kit modules (the 400-stud rule is already
  implemented in `addAuthoredStreetKit`; the city only has 4 modules, so it places 4).
- **Cheapest useful batch: 6 new window sheets + 6 new back facades** → the worst count falls from 13 to about 6.
- The threshold stays **max 1**. It was not moved to make a number pass.

## 6b · The fresh checker (3 rounds, the cap)

A fresh checker (new session, read-only, it RAN the checks) verified the publish-ready claims:

- **Round 1: 12 PASS / 1 FAIL.** The failure was mine: the bundles named the pre-polish RC and the pre-polish
  numbers while the renders were of the polished build.
- **Round 2: 19 PASS / 2 FAIL.** The fix had not reached sections 2-3 of the playtest bundle (pack reference,
  numbers, render labels).
- **Round 3: 33 PASS / 0 FAIL / 1 NOT RUN** (the NOT RUN is the card lane's 22-step verify, outside that brief;
  round 1 ran it green). Every bundle claim reproduces; the checker's own re-measurement gives 18 valid street
  views, 2 luminance fails, 0 fleck, 1 cyan, facade 13. Its fence check: nothing uploaded or published, sealed
  places byte-identical, no Studio process, no tracked modification in either repo.
- Verdicts: `~/fleet-scratch/overnight/roblox2-CHECKER-VERDICT{,-ROUND2,-ROUND3}.md`.

## 7 · The owner gates, packaged (one look each)

`docs/bundles/2026-11-04/` in the street repo (README + 4 bundles + the renders and a contact sheet):
**street playtest** (which build, the measured look, the four acts, one reply form), **card D12** (the six acts,
the rights call, Δ16), **street publish** and **card publish** (what is proven, what is not, the exact commands,
the reply form). The card bundle points at the card repo's own one-page answer sheet, so nothing is duplicated.

## 8 · Honest limits

- The capture pipeline needed work to be usable at all: the MCP `screen_capture` caps the image at 1920 px wide,
  so the capture viewport must sit between 1600 (ruling 1's floor) and 1920 (the capture cap) — a 2200x1000
  window gives 1553x744 today (the 3 Nov pack had 1837x744 at the same window size), a 2400x1000 window gives
  1753x744. The Studio also **crashed at the end of the 3rd Play** in one process; the run had already written its
  30 images, and the latch was cleared with `verify-clean --clear-latch` after a clean verify.
- No Studio spec has run on this build (`tools/usg test --studio` needs a signed-in Studio and the owner's hands).
- The street look misses three of the ruled bounds today; the road-luminance and void items are art-lane work with
  an owner-visible before/after, and the cyan item needs Nox's re-derivation or an art decision.
- The card game is unchanged by this lane and remains where slice C left it: `master` @ `7ebbb9a`, 22/0 on the
  clean gate, zero uploads, D12 and the rights call still the owner's.

## 9 · Files

Street repo `main`: `f640881` (look gate), `f82c779` (Slice D), `1a8c322` (window-finder tests), `f815f72`
(bundles + look-gate record), `e2eb15e` (the wet-night polish), `b5e2eb0` / `adfa098` / `e2a6e7c` (the three
checker rounds). Candidate: `build/rc/rc-e2eb15e.rbxlx` (`32406439…`). Card repo `master`: `7ebbb9a` (unchanged here).
Window finder: `~/Work/tools/winctl/{winctl.py,studiocycle.py}` + `tests/test_winctl.py`.
Evidence: `docs/verify/2026-10-19/vinegar/20261104T163738Z-atlas-scout`, `docs/verify/2026-11-04/`,
`docs/art/`, `docs/bundles/2026-11-04/`, `~/fleet-scratch/overnight/roblox2-progress.md`.
