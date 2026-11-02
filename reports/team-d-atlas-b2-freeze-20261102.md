# TEAM D — ATLAS B2 + A2 + ORDERING FREEZE (per-region world overheads), 2026-11-02

Nox 20261102T063400Z approved **B2 + A2 + ordering fold** exactly as the design-b2 note. Offline only — no Play.
**Freeze commit: `746a99ed16b46a6ab6f1c45b71394b01a782bdf1`** (`746a99e`; parent `e6ae134` = B-bounded traffic freeze;
branch tip; tree `c70c1d0b5005e0a38e26e47139eca716057acbd0`; tracked tree clean; **7 files +120/−72**). Sealed artifacts
unchanged (`84324d43…` release, `c6651793…` test/place).
Design note: `board/probes/team-d/design-b2-world-regions-20261102.md`. Log: `board/probes/team-d/atlas-b2-freeze-20261102.log`.

## 1 · What changed (substitution + limitation + ordering)

- **B2 — plan** (`tools/map_atlas.py`): `world-overhead` is removed; four bounded **`world-region-1..4-overhead`** views
  replace it, each anchored on ONE native bounded model — `City/Junctions/JunctionPad_n_3_2`, `…n_10_10`, `…n_6_12`,
  `…n_12_6` (four spread city-junction pads; never the world support). **Justified deviation from the design note:** the
  note's region-3/4 anchors reused two highway-zone road anchors; the commit uses four **distinct** city pads instead so
  the validator's eight-distinct-anchor rule holds and the 8 anchors are unique across both role groups. — `zone:'world'`, `regionRole:'world-region'`,
  `coverageIds:['world:overview']`, planned cameras like other overheads. The four existing highway-zone views carry
  `zoneRole:'highway-zone'`. Scout count goes **27 → 30** (`24 − 2 + 8`), assert updated, html scout count derived.
- **Ordering fold**: scout order is `city-overhead`, four `highway-zone` overheads (ordinals **21–24**), perimeter,
  grade, then the four `world-region` overheads **last (ordinals 27–30)** — a world-region freeze still lands all four
  zone captures in the same run.
- **Server** (`tests/atlas/Server.luau`): overhead fit-bounds are now **zonePath-first** (`actual.fitBounds =
  actualBounds(anchor)` before the zoneless-world support fallback), so world-region views use their native bounded model
  bounds; the previously substituted highway-zone views keep working through the same branch.
- **Validator** (`tools/atlas_acceptance.py`, scout only): explicit role groups — exactly **4 highway-zone** +
  **4 world-region** views with pinned ids; highway anchors under `Highway/Roads/Road_`, world anchors under `City/` or
  `Highway/Roads/`; **8 distinct native anchors**; **no `highway-overhead` and no `world-overhead` anywhere**; the
  **ordering fold** asserted (world regions last; zone views before them); and the **bounded-framing proof**: each
  world-region `fitBounds` span is strictly smaller than the native support spans (and non-empty).
- **A2** (`board/probes/team-d/overhead-render-limitation-contingency-20261102.md`): final form — the named limitation
  covers the **whole-map overhead class, both framings** (highway Play 7; world Play 10 evidence), build
  `Studio 0.745.0.7451137 / Wine 11.0 (Vinegar 1.9.4)`, fail-closed verdict, **one retest per framing per new Studio
  build** to retire it. C/D remain rejected (run-7); C is the separate abort-contact pre-approval, not this limitation.
- **Docs**: `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` atlas note rewritten (30 views, substitution, ordering,
  limitation class). No other plan/validator behaviour changed.

## 2 · Tests

- `tests/tools/test_map_atlas.py`: full planned list pinned — no whole-map overhead ids anywhere, exactly the four
  highway-zone + four world-region bounded views, 8 distinct anchors, overhead set = city + 8; scout = 30 with the four
  world regions last and zone views before them.
- `tests/tools/test_atlas_acceptance.py`: 30 views; role-group positives; negatives — missing world-region group,
  missing highway-zone group, wrong highway anchor, duplicated world-region anchor; full scout accepted with the
  substituted plan; fixture carries the native support bounds for the bounded-framing proof.
- `tests/tools/test_vinegar_run.py`: counts 27 → 30 (captures, staticCameraImages, screen_capture calls, rows, audit
  lengths), overhead settles 6 → 9, golden digest recomputed on the authorised plan; fake support bounds widened for the
  bounded proof.
- **Mutations (design note M23/M24/M25)**: an added/restored whole-map overhead or a broken role/anchor/bound fails the
  plan/validator pins; the B2 negatives above cover drop/duplicate/wrong-anchor/whole-map-id regressions.

## 2b · Checker-trading delta (REJECT → fixed, second freeze)

Checker delta (`board/reports/checker-b2-delta.md`) rejected only on **M24**: "revert the server fit order" did not fail
any test because nothing pinned the zonePath-first `fitBounds` order. Smallest fix + folded residuals:
1. `test_atlas_acceptance.py` wiring test now pins the exact order
   (`if view.zonePath then actual.fitBounds=actualBounds(anchor)\n   elseif view.zone=="world" then`) **and** asserts the old
   world-first form is absent. **M24 re-run: FAILED (1)** — mutation now caught.
2. The old server/client **cap test restored** (`#manifest.views<=32` / `#plan.views<=32`, no `<=24`, 30-view scout fits,
   33 rejected).
3. The **bounded-span negative is unit-tested**: shrinking `server.support` below a region's fit now fails with
   `world-region framing must be strictly smaller than the world support`; **M25b (rule removed) → FAILED (1)**.
4. Missing log filed: `board/probes/team-d/atlas-b2-freeze-20261102.log`.
5. Stale runbook run-7 "27 views" paragraph removed (only the 30-view paragraph remains).
6. Report untracked count corrected 792 → **823**; anchor deviation recorded (above).
**Delta tip:** see the commit after `746a99e` (same checker seat re-verifies). Suites re-run on the delta tip: atlas
acceptance **24 OK**, full Python, Lune atlas + full Lune.

## 3 · Freeze evidence (fresh checkout of `746a99e`)

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `746a99e…`; tracked tree clean (823 pre-existing untracked evidence-dir lines, R2 cosmetic) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 91 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_map_atlas.py'` | **Ran 6 tests … OK** |
| 4 | `python3 -m unittest discover -s tests/tools -p 'test_atlas_acceptance.py'` | **Ran 22 tests … OK** |
| 5 | `python3 -m unittest discover -s tests/tools` | **Ran 330 tests … OK** rc 0 |
| 6 | `./.tools/bin/lune run tests/unit/run.luau atlas` | **10 passed, 0 failed (4 specs)** |
| 7 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 923 passed, 0 failed (127 specs)** rc 0 (~12 min) |
| 8 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 9 | `git show --stat 746a99e` | 7 files, +120/−72 |

**Exact checker reproduction:** `git checkout 746a99ed16b46a6ab6f1c45b71394b01a782bdf1` fresh worktree → `git status`
empty; run rows 2–7 (no Play/Studio needed); sealed row 8. Contract spot-checks: `V.atlas_acceptance.manifest('scout')[0]`
has 30 views with exactly the eight role-tagged bounded overheads, no whole-map overhead ids, world regions last;
validator rejects a plan missing a role group / with a wrong highway anchor / duplicated world-region anchor; server fit
block is zonePath-first; `Atlas.overhead_kinds` includes the region views through `kind:'overhead'`.
**Next:** checker delta on `746a99e` → on ACCEPT, one atlas Play (seed 2134783819) in a fresh opening with the full
evidence chain (30 captures, `capture-attempts` consistency, 9 × 1.0 s settles, fog/viewport records, runtime/validator/
index/look, cell verdict).
