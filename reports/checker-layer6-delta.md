# CHECKER — layer-6 allowlist fix delta (`8bda598`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the server-allowlist fix (Play 8: stale server `<=24` vs the 27-view B+A plan) on top
of my accepted zones delta (`d83efa8`). I built none of it. No Play, no Studio input, no image reads, no host input,
**$0**. Ran from a throwaway clone `/tmp/checker-layer6/repo` (2026-11-02T00:56:39-05:00) and a second clone for
mutations; maker tree only read.

- **Subject tip:** `8bda598f7cf023a9eb58aab1a76c93dfce3d5ba3` (`8bda598`), parent `d83efa8`, tree
  `7face17b2215a45153eb1d59b9d94fddd98b15ac`; tracked mods **0**; sealed `84324d43…`/`c6651793…` unchanged
  before/after and at end.
- **Delta over `d83efa8`:** exactly **3 files, +28/−1** — `tests/atlas/Server.luau` (+1/−1),
  `tests/tools/test_atlas_acceptance.py` (+14), `tests/tools/test_map_atlas.py` (+13). No `src/`, `assets/`, `build/`
  change; `tools/map_atlas.py`, `tools/atlas_acceptance.py`, `tools/vinegar_run.py`, `tests/traffic/Server.luau`
  untouched → plan/validator substitution, settle/retry/audit/45 s bound/assert-and-skip/fixture A+C intact.
- **Maker report:** `board/reports/team-d-layer6-allowlist-fix-20261102.md` is **absent**; verified from the commit.

## Verdict in one line

**ACCEPT** — the server allowlist bound is now `<=32` (mirrors the client; the stale `<=24` is gone) and the bound is
still enforced (the >32 case is pinned as rejected); the server/client agreement test and the full-planned-list
assertion are in place and both mutations fail them (M20 bound→24 → 1 failure; M21 unused whole-highway view in the full
list → 1 failure); the delta is fixture/test-only, sealed untouched, and all suites are green (91 / 23 / 6 / 329 / Lune
atlas 10-0). One brief-wording discrepancy is noted (R3: the full planned list has 1,895 views, not 27 — 27 is the scout
plan; the new test correctly pins the full list's overhead composition, which is what the R3 fold needed).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Server bound 32, stale 24 gone, bound still enforced | **PASS** | `tests/atlas/Server.luau:56`: `assert(manifest.candidate==cfg.candidate and #manifest.views<=32,"candidate/allowlist mismatch")`; `tests/atlas/Client.luau:77`: `#plan.views<=32,"native allowlist invalid"`; `grep views<=24` across `tests/atlas/` + `tools/atlas_acceptance.py` + `tools/map_atlas.py` → **no matches** (only the new tests' `assertNotIn` strings). The bound is an upper bound, not removed; the >32 rejection is pinned (see ②) |
| ② Offline tests: agreement + >32 rejection | **PASS (offline pin)** | `test_server_and_client_agree_on_the_widened_view_bound`: asserts `#manifest.views<=32` (server), `#plan.views<=32` (client), `views<=24` absent in both, scout `len(data['views'])==27`, `bound(views)` true for 27 and false for 33 (`data['views']+[{}]*6`). Note: the >32 rejection is a **source-shape pin + a test-side mirror** of the Luau assert (`bound=lambda views:0<len(views)<=32`), not an execution of the Luau code — offline limit, consistent with the rest of the fixture tests |
| ③ R3 folded: full-planned-list assertion | **PASS (with R3 wording note)** | `test_full_planned_list_contains_no_whole_highway_overhead_and_exactly_the_zone_views` reads `atlas.planned_views(self.graph)` (the **full** list, not the scout subset) and asserts: no `highway-overhead` anywhere, exactly `highway-zone-1..4-overhead` with distinct `Highway/Roads/Road_` paths, all `kind=overhead`/`zone=highway`, and the full overhead set is exactly `{city-overhead, world-overhead} ∪ zones`. My independent computation: full planned list = **1,895 views**, scout = **27**, zone views exactly 4, overhead views exactly the six, no `highway-overhead` anywhere (`full-list-facts.json`). **Brief wording note:** the task says "the full list has exactly 27 views" — that is not true (the full list is 1,895; **27 is the scout plan**). The new test pins composition, not a full-list length (which would be wrong); the scout length 27 is pinned elsewhere (test_map_atlas scout rows, test_vinegar_run manifest). The R3 intent — an unused whole-highway view in the full list cannot linger green — is met (M21) |
| ④ Scope / regressions | **PASS** | Exactly the 3 claimed files; no game/asset/sealed path; `tools/map_atlas.py`/`tools/atlas_acceptance.py` untouched (B substitution unchanged beyond the pin); `tools/vinegar_run.py` untouched (`ATLAS_FIRST_VIEW_TIMEOUT = 45`, `capture_attempts_max = 3`, `overhead_settle = 1.0`); `tests/traffic/Server.luau` untouched (fixture A+C intact) |
| ⑤ Suites + mutations | **PASS** | Fresh clone: runner `Ran 91 tests … OK` (12.493 s); atlas acceptance `Ran 23 … OK` (0.733 s); map atlas `Ran 6 … OK` (0.360 s); full Python `Ran 329 tests … OK` rc 0 (47.527 s); Lune atlas filter `10 passed, 0 failed (4 specs)`; `atlas-paths` `1 passed, 0 failed`. Mutations in a throwaway clone, restored: **M20** server bound reverted to 24 → atlas acceptance **FAILED (failures=1)** (`test_server_and_client_agree_on_the_widened_view_bound`); **M21** unused whole-highway view added to the **full** planned list → map atlas **FAILED (failures=1)** (`test_full_planned_list_contains_no_whole_highway_overhead_and_exactly_the_zone_views`). Restored `tests/atlas/Server.luau` sha256 `62a9a56e6515…`, `tools/map_atlas.py` `012d0c7738f3…` (frozen), git clean |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged; maker HEAD `8bda598`, tracked clean at end. The final atlas Play (1 Play left) is the live proof of the widened allowlist |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the bound fix is offline-proven (source pin + mutation); the final atlas Play is the live
  proof that the 27-view plan passes `AtlasServer:56`.
- **R2 — maker report absent:** `team-d-layer6-allowlist-fix-20261102.md` is not present; verified from the commit.
- **R3 — brief-wording discrepancy:** "the full list has exactly 27 views" is inaccurate — full planned list = 1,895
  views, scout = 27. The new test pins the full list's overhead composition (no `highway-overhead` anywhere, exactly the
  four zone views) and the scout length 27 is pinned elsewhere; no defect, but the handoff wording should say "scout
  list = 27; full planned list contains no whole-highway overhead".
- **R4 — offline nuance:** the >32 rejection is pinned as a source-shape string + test-side mirror predicate, not by
  executing the Luau assert (no Play/Studio in this check).
- **R5 — cosmetic:** untracked evidence-dir count now 811 vs the "792 pre-existing" wording used in earlier reports.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-layer6-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
earlier-layer invariants + bounds + agreement test + full-list test), `full-list-facts.json` (full list 1,895 / scout 27 /
zone views / overhead set / no highway-overhead), `suites.log`, `runner.log` (91 OK), `atlas.log` (23 OK), `map.log`
(6 OK), `python.log` (329 OK), `lune-atlas.log` (10/0, 4 specs), `atlas-paths.log` (1/0), `mutations.log` (M20 1, M21 1;
restored hashes), `run-started.txt`. Throwaway copies `/tmp/checker-layer6/{repo,mut}`; maker repo read-only, unchanged,
still at `8bda598`.
