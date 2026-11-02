# CHECKER — Team D atlas **B2 + A2 + ordering** delta (`746a99e`) — verdict: **REJECT**

Fresh-context independent check (reassigned seat; I built none of this work — maker-never-checker). All execution from a
**fresh copy of the commit**, never the maker's working tree. **No Play, no Studio input, no image reads, no build, $0.**
Report to `lead` only.

- **Checked commit:** `746a99ed16b46a6ab6f1c45b71394b01a782bdf1` (tree `c70c1d0b5005e0a38e26e47139eca716057acbd0`),
  parent `e6ae1344fd945ff9f9e7ee1d371ad1fc346919ed` (B-bounded traffic freeze), branch `prep/b1-capture-format-20261101`
  tip; tracked tree clean; **7 files, +120/−72**.
- **Fresh copy:** `/tmp/checker-b2/repo` from `git archive 746a99e | tar -x` — **1443 tracked entries, blob-for-blob equal
  to the commit tree** (0 missing / 0 mismatch; symlink `CLAUDE.md -> AGENTS.md` handled). `build/` is untracked in the
  commit, so the sealed artifacts were copied read-only and hash-checked (`c6651793…` test, `84324d43…` release).
- Note: the maker report says "792 pre-existing untracked evidence-dir lines"; the actual count at check time is **823**
  (cosmetic, tracked tree is clean — the report's number is stale).

## Verdict in one line

**REJECT** — B2's plan, validator, ordering, A2 note, scope and every re-run suite check out (30 scout views; 4+4 role
groups; 8 distinct native anchors; no whole-map ids in the 1,898-view full list; bounded-framing proof; ordinals 21–24 /
25–26 / 27–30; runner 91 OK, map 6 OK, atlas acceptance 22 OK, full Python 330 OK, atlas Lune 10/0 across 4 specs, full
Lune 923/0 across 127 specs; M23 and M25 fail as required). **But the design note's M24 mutation — "revert server fit
order" — does NOT fail any offline test**: no test or spec pins the `zonePath`-first `fitBounds` order, so the functional
core of B2 has no regression pin. One-line test addition + re-freeze; nothing else needs rework.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **B2 plan**: single `world-overhead` replaced by four bounded per-region views, explicit `regionRole`, `coverageIds` unchanged | **PASS** | direct plan probe (`plan-probe-20261102.txt`): scout = 30; `world-region-1..4-overhead` with `zone='world'`, `kind='overhead'`, `regionRole='world-region'`, `coverageIds=['world:overview']` (same as the old view), anchors `City/Junctions/JunctionPad_n_{3_2,10_10,6_12,12_6}` — real native pads created at `src/server/CityBuilder.luau:303`; no `world-overhead` / `highway-overhead` id anywhere in the full 1,898-view list; overhead set = `city-overhead` + 4 highway-zone + 4 world-region |
| 2 | **Server fit-bounds**: `zonePath` branch FIRST, else world/support | **PASS (code)** / **FAIL (pin)** | `tests/atlas/Server.luau:232`: `if view.zonePath then actual.fitBounds=actualBounds(anchor) elseif view.zone=="world" then … support … elseif anchor then …` — correct. **But M24 (revert to world-first) passes every suite → no pin** (see §3) |
| 3 | **Validator role groups + bounded-framing proof** | **PASS** | `tools/atlas_acceptance.py`: exactly 4 `zoneRole='highway-zone'` + 4 `regionRole='world-region'` with pinned ids; anchors scoped (`Highway/Roads/Road_` vs `City/` or `Highway/Roads/`); **8 distinct anchors**; ordering fold asserted; `world-region framing must be strictly smaller than the world support`. Direct negative probe: baseline accepted; one region span == support → rejected with that exact message; empty span, missing region, duplicated anchor, wrong highway anchor, whole-map id → all rejected |
| 4 | **No `world-overhead` / `highway-overhead` in the full planned list** | **PASS** | plan probe: only `city-overhead`, `highway-zone-1..4-overhead`, `world-region-1..4-overhead`; `test_map_atlas.py` pins the full list (M25 fails it) |
| 5 | **Scout 27→30; client cap 32 sufficient** | **PASS** | 30 scout views; `tests/atlas/Client.luau` **not in the delta** and still `#plan.views<=32`; 30 ≤ 32 |
| 6 | **Ordering fold**: highway-zone 21–24, perimeter/grade next, world-region 27–30 | **PASS** | scout order probe: 19 garage, 20 `city-overhead`, **21–24** highway-zone, 25 perimeter (`city-junction-n_12_12`, reason `city-perimeter`), 26 grade (`city-road-n_6_7__n_6_8-forward`, reason `city-grade`), **27–30** world-region; world regions last; zone ordinals < region ordinals; asserted in plan + validator + runner tests |
| 7 | **A2 note** (`overhead-render-limitation-contingency-20261102.md`) | **PASS** | records the **whole-map overhead class, both framings**: highway Play-7 (3× byte-identical, settle 1.0 s, `cameraDelta:0`, audit 21/21) + world Play-10 (view-020 fresh, view-021 3× byte-identical); named build **Studio 0.745.0.7451137 / Wine 11.0 (Vinegar 1.9.4)**; **fail-closed verdict, never a pass claim**; **one retest per framing per new Studio build**; **C/D rejected** (C = settings/draw-distance experiments; D = cap/drop) — no settings experiments adopted. Play-7/Play-10 reports exist and match the quoted evidence |
| 8 | **Tests/mutations**: delete a world region; revert server fit order; re-add `world-overhead` to the full list must FAIL | **FAIL** | M23 **fails** (`test_map_atlas` 2F; `test_atlas_acceptance` 1F+4E); M25 **fails** (`test_map_atlas` 1F); **M24 does NOT fail anything** — 22 OK, 91 OK, 6 OK, atlas Lune 10/0, full Python 330 OK (see §3) |
| 9 | **Delta scope** file-by-file; no production/game code; sealed untouched | **PASS** | `git diff --numstat e6ae134 746a99e`: exactly the 7 disclosed files (+120/−72); no `src/`, `assets/`, `deploy.json`, `build/`, `tests/atlas/Client.luau`, `tools/vinegar_run.py`; settle/retry/audit code untouched (only expected counts in tests); sealed `84324d43…` / `c6651793…` / Studio place `c6651793…` unchanged |
| 10 | **Bounded suites re-run** | **PASS** | runner **91 OK** (12.3 s); map_atlas **6 OK**; atlas acceptance **22 OK**; full `tests/tools` **330 OK** (47.1 s); Lune atlas filter **10 passed / 0 failed (4 specs)**; **full Lune 923 passed / 0 failed (127 specs)** (~12 min, re-ran, not relied); sealed hashes re-checked after every run |

## 1. Plan / ordering proofs (quoted from the probe)

```
scout: 30 | full planned views: 1898
world-overhead in full list: False | highway-overhead in full list: False
overhead ids: ['city-overhead','highway-zone-1-overhead',…,'highway-zone-4-overhead','world-region-1-overhead',…,'world-region-4-overhead']
20 city-overhead | 21–24 highway-zone-1..4-overhead | 25 city-junction-n_12_12 (city-perimeter)
26 city-road-n_6_7__n_6_8-forward (city-grade) | 27–30 world-region-1..4-overhead
world-region anchors: City/Junctions/JunctionPad_n_3_2, …n_10_10, …n_6_12, …n_12_6  (8 distinct anchors across the two groups)
world coverageIds: ['world:overview'] for all four (unchanged from the substituted whole-world view)
```
Native anchors are real: `src/server/CityBuilder.luau:303` creates `JunctionPad_<node.id>` under `City/Junctions`.

## 2. Mutation results (the REJECT reason)

| Mutation | Expected | Actual |
|---|---|---|
| **M23** delete a world region | fail | **FAILED** — `test_map_atlas` 2 failures; `test_atlas_acceptance` 1 failure + 4 errors ✔ |
| **M25** re-add `world-overhead` to the full list only | fail | **FAILED** — `test_map_atlas` 1 failure (full-list pin) ✔ |
| **M24** revert the server fit block to world-first (`if view.zone=="world" then …`) | fail | **NOT CAUGHT** — `test_atlas_acceptance` 22 OK, `test_vinegar_run` 91 OK, `test_map_atlas` 6 OK, atlas Lune 10/0, **full Python 330 OK** ✘ |

The design note §7 says M24 is caught by a **source-shape test + Play**; the source-shape test does not exist in the
frozen commit. The only tests that read `tests/atlas/Server.luau` assert unrelated strings (fog block, `WorldSeed`, the
anchor-resolution line `anchor=geometry(view.zonePath);…`) — none asserts the `fitBounds` order. The only Lune spec that
reads the file (`atlas-paths`) asserts `Paths.emit(anchor)`, unrelated to the fit block. **So a silent revert of the B2
server fix would pass the whole offline suite and only surface in a Play — where the named A2 limitation could excuse the
stall.** That is exactly what the mutation proof is meant to prevent.

## 3. Smallest fix (re-freeze after this)

In `tests/tools/test_atlas_acceptance.py::test_zone_overhead_wiring_is_present_in_plan_server_client_and_validator`
(add to the existing `server=(A.ROOT/'tests/atlas/Server.luau').read_text()` block):

```python
self.assertIn('if view.zonePath then actual.fitBounds=actualBounds(anchor)', server)
self.assertLess(server.index('if view.zonePath then actual.fitBounds=actualBounds(anchor)'),
                server.index('elseif view.zone=="world" then'))
```

Optionally add the bounded-framing negative (one world-region `fitBounds` span == support span → `ValueError`) to the
role-group test; the validator already enforces it (direct probe). Then re-freeze (new commit), re-run the same suites and
M24, and re-submit. **No plan/validator/ordering/A2/scope rework is needed.**

## 4. Named residuals (non-blocking; fold into the same re-freeze)

1. **Missing referenced log.** The maker report (line 7) names `board/probes/team-d/atlas-b2-freeze-20261102.log`; that
   file **does not exist** (the run-7 log does; no run-9 freeze log). The §3 table is reproducible (I re-ran it), so this
   is a paperwork gap — drop the reference or write the log.
2. **Stale runbook paragraph.** The runbook now has the new run-9 paragraph (line 85: **30 views**, B+B2+ordering) and,
   later, the old run-7 paragraph (line 106–108) still saying the scout plan "has **27** views (24 − 1 + 4)". The old
   paragraph is not marked historical → contradictory. Remove or annotate it ("superseded by run-9: 30 views").
3. **Design-note anchor deviation (justified, but commit message says "exactly").** Design note §1 anchors regions 3/4 on
   two `Highway/Roads/` roads that are the **same** anchors as highway-zone-1/4 — implementing it literally would violate
   the note's own "8 distinct anchors" rule. The implementation instead uses four distinct city pads (n_3_2, n_10_10,
   n_6_12, n_12_6) and the report states them accurately; only the commit message's "exactly the design-b2 note" is loose.
4. **Cap test removed.** The old `test_server_and_client_agree_on_the_widened_view_bound` (server+client cap 32, 33-view
   rejection) was deleted; the client cap is still asserted in the wiring test, the server cap and the >32 rejection are
   no longer pinned. The cap is unchanged and 30 ≤ 32, so this is coverage, not behaviour.
5. **Bounded-framing negative not unit-tested** (validator works — see direct probe); add it with the M24 fix.
6. Report's "792 untracked" vs actual **823** (cosmetic; tracked tree clean).

## 5. Limits

- **No Play / no Studio input**: native anchor existence/bounds and the four world-region framings are only
  Play-verifiable (the plan is `planned-unseen`; the validator runs on the runtime report). The named A2 limitation is the
  documented cover for the whole-map class; this check does not claim a capture.
- The full Lune run was re-run (923/0/127, ~12 min), but it is **not** what decides M24: no Lune spec reads the fit block.
- Sealed artifacts verified by hash; no build/Play was performed. No production/game code is in the delta.

## 6. Evidence (checker-produced, `board/probes/checker-b2/`)

`provenance-20261102.txt` (commit/tree/parent/branch/fresh-copy/delta numstat), `plan-probe-20261102.txt` (roles,
anchors, ordinals, full-list ids, cap), `validator-negative-probe-20261102.txt` (bounded-framing + role/order/whole-map
negatives), `mutations-20261102.txt` (M23/M24/M25 + full-suite M24), `suites-20261102.txt`,
`lune-full-20261102.log` (923/0/127). Fresh copy `/tmp/checker-b2/repo`; mutation copy `/tmp/checker-b2/mut` (restored).

## 7. Fences held

Read-only against the maker repo; execution only in throwaway copies. **No Play, no Studio launch/input, no build, no
upload/publication, no image read, no credential value, $0.** I made none of the checked work.

---

**— checker-trading (fresh-context, B2+A2 delta seat), 2026-11-02 02:50 ET (EST, UTC−5) = 2026-11-02 07:50Z.**
**Verdict: REJECT** for `746a99ed16b46a6ab6f1c45b71394b01a782bdf1` — M24 (revert server fit order) is uncaught by the
frozen suites; smallest fix §3, then re-freeze. Everything else (B2 plan/validator/ordering, A2 note, scope, suites,
sealed) passes.
