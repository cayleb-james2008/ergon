# Team D — A1 `highway.spec` spec-alignment: independent checker verdict

**Checker:** fresh-context independent checker for Cayleb's Roblox lane (Team D). Did not write the change; reports to the Lead only.
**When:** 2026-11-01 (ET). **Mode:** read-only; text/commands only — no Studio, no GUI, no process driving, no image reads.
**Subject:** `prep/a1-highway-spec-20261101` @ `1abe30391caf88db9d76bd424d87461899b5cdf9`, change to `tests/studio/highway.spec.luau` only, +67/−20.
**Design authority:** `docs/TRAFFIC_SPEC.md` row 7 (line 29), row 8 (line 30), line 56; count/spacing code in `src/shared/TrafficSim.luau`.

---

## VERDICT: **PASS**

The spec change correctly aligns the acceptance test to the documented design contract. It touches only the Studio spec; no production code, no design doc, no other test, no build, no re-pin. The derived count reproduces the documented density (9 cars/lane, 54 total at seed 918273), and the spacing bound is the design's own ≥ 0.6 slot metric, replacing the non-design, length-normalised 0.1 bound.

**A1 may be re-run** from the prep branch after this PASS. This verdict does **not** declare A1 green: the run itself (Studio, from the prep branch, with sealed hashes recorded before/after) is still required and must be observed, not inferred.

| # | Item | Verdict |
|---|---|---|
| 1 | Branch / base / commit as claimed; `main` untouched; diff touches only the spec; nothing else weakened | **PASS** |
| 2 | Assertions implement the actual design contract (graph-derived count, shared count, derived total, slot spacing, all other assertions preserved) | **PASS** |
| 3 | Nothing hides a real defect; diagnosis numbers and 54 cars are correct per the design | **PASS** (one minor numeric imprecision in the brief, not in the change) |
| 4 | Sealed artifacts unchanged (`84324d43…`, `c6651793…`); no build/re-pin | **PASS** |
| 5 | Honest limits stated; A1 must be re-run | **PASS** (re-run still outstanding — see §5) |

---

## 1 · Branch, base, scope, main — PASS

```
$ git branch --show-current
prep/a1-highway-spec-20261101
$ git rev-parse HEAD
1abe30391caf88db9d76bd424d87461899b5cdf9
$ git rev-parse HEAD^
b67abdd253029378a960a0899d06c672470e9072
$ git merge-base prep/a1-highway-spec-20261101 main
b67abdd253029378a960a0899d06c672470e9072
$ git rev-parse main
8dfb2040f26187272fb28be3be8ca1775ce27310
$ git log --oneline main..prep/a1-highway-spec-20261101
1abe303 tests(a1): align highway.spec with the highway traffic design contract
$ git diff --name-status main...prep/a1-highway-spec-20261101
M	tests/studio/highway.spec.luau
$ git diff --stat HEAD^ HEAD
 tests/studio/highway.spec.luau | 87 ++++++++++++++++++++++++++++++++----------
 1 file changed, 67 insertions(+), 20 deletions(-)
```

* Base is exactly `b67abdd` (also the full parent hash, not a re-created lookalike); the branch contains one commit on top of it.
* `main` untouched by this work: its newest reflog entry is `main@{2026-10-21 06:41:44 -0400}: commit (amend): Docs: correct the stale numbers …`; the 2026-11-01 candidate-identity probe (`board/probes/team-d/d2-candidate-identity-20261101.json`, `head_at_probe`) also records `8dfb204…`, i.e. `main` has not moved since before this change.
* `main` is 16 commits ahead of the base — that is the documented, intentional split, not drift from this change: the probe pins the street sealed candidate at `b67abdd` and says the live repo HEAD is not used ("the street game's sealed candidate is **commit `b67abdd`** … the test place must stay `c6651793…`"). Of those 16 commits, none touches `src/shared/Graph.luau`, `src/shared/TrafficSim.luau`, `src/server/HighwayBuilder.luau` or `src/server/World.luau` (`git diff --name-only b67abdd..main` lists only `src/client/WorldRender.luau` among traffic/world-named files, plus client/actors, Catalog, docs, unit/playtest tests). The branch is therefore merge-clean against `highway.spec` (`git diff b67abdd..main -- tests/studio/highway.spec.luau` is empty).
* Working tree has no modified tracked files (`git status --porcelain` shows only pre-existing untracked `docs/verify/…` material).
* Diff artifact identity: `board/probes/team-d/a1-highway-spec-change-20261101.diff` is byte-identical to `git diff HEAD^ HEAD -- tests/studio/highway.spec.luau` (checked with `diff -`); its blob header `index ccb69e4..a44aacc` matches `git rev-parse HEAD^:… HEAD:…` = `ccb69e4f67…` / `a44aaccf6f…`.

**No other test weakened:** the commit changes one file only, in 6 hunks (header comment; `trafficCohortChecks`, two hunks; `Spec.run` call/report/return, three hunks). Every other assertion in the 308-line spec is byte-identical to the base. The only removed assertions are the two stale ones (`count == 5`, `minimumSpacingRatio >= 0.1`), both replaced by stricter, contract-derived checks (§2).

---

## 2 · Do the new assertions implement the actual design contract? — PASS

| Required contract | Where asserted (new spec) | Where the contract/code lives | Check |
|---|---|---|---|
| Per-lane count `min(10, ceil(5 · poly / baseline))` | `highway.spec.luau:183-195` | `TrafficSim.luau:312` | exact same formula |
| Same graph inputs TrafficSim uses | `:184-195`: `baselineMainlineLength`, `(if route.id == "clockwise" then 1 else -1) * 2 * offset * baselineTurnMiter`, `TrafficSim.laneLoop(graph, route.nodeIds, offset)` | `TrafficSim.luau:303-306` (identical sign/`2·lane·miter` baseline, identical `laneLoop` call) | identical |
| One shared per-lane count across all six lanes | `:208-210` (`assert(expectedCount == sharedExpectedCount, …)`) | design row 8 (one shared seed/density); the six-lane integer equality is this review's brief, stricter than row 8 | present (see observation 1) |
| Each lane's observed cars == derived count | `:233` | — | exact equality, so a perturbed lane count fails |
| Total == sum of the derivation (54 at seed 918273) | `:212` (sum), `:276` (`trafficCount == expectedTrafficCount`) | design line 56 "26 of 54 cars" | derived, no `54` literal in code (only in the header comments, lines 7 and 177) |
| Spacing in slots, ≥ 0.6 slot | `:228-247`; `slot = length / count` at `:238`; assert at `:246` | `TRAFFIC_SPEC.md:29` row 7 "never closer than 0.6 slot"; `TrafficSim.luau:24` `SLOT_JITTER = 0.4` → floor `(1−0.4)` slot | present (exact design bound) |
| All other assertions preserved | 100-seed graph/circuit loop (`:254-261`), access clearance, road/ground rays, junction pad (`roadSurfaceCount > #graph.edges`), arrival bays, shared-world contract, `passed = 2`, `seedsPassed = 100` | same file, untouched hunks | preserved byte-for-byte |

**Measured against the design, not the stale metric.** The old check normalised the gap by loop length at ≥ 0.1: with five cars that was 0.5 slot (below the design's 0.6), and with nine cars the same formula would have implied 0.9 slot — a number the design never states, produced by an accidental length normalisation that gets stricter purely because the extended loop is longer. The new check uses the design's own unit (`slot = length / cars`) and the design's own floor (0.6, row 7); the observed 0.669 floor clears it. The per-lane count is now exact rather than "any 5". This is alignment to the ruled contract, not a loosening of a design requirement: the 0.9-slot figure was never the contract (row 7 is written in slots, not loop fractions), and the design doc itself documents the physical spacing as ~1,100 studs/car, which the 54-car cohort matches.

**Not self-fulfilling.** The expectation is recomputed in the spec from graph geometry and the documented rule; it is never read back from `sim.cars`/`loop.count`. Negative controls all bite:
* one lane at 8 cars (or an extra car) → `:233` fails;
* clustering below 0.6 slot → `:246` fails;
* lanes differing from each other → `:210` fails;
* cars outside the six lane keys → `:223` fails.

---

## 3 · Is 54 correct per the design (nothing hidden)? — PASS

**Independent recomputation (no Studio).** From `Graph.luau:371-372,377,379` for seed 918273 (`halfLength = 1380`, `radius = 108`, `turnSections = 8`, `extensionLength = 2250`) and `Graph.luau:511-512`:

```
baselineMainlineLength = 4·1380 + 4·8·108·sin(π/16) = 6194.232152887739176   (diagnosis: 6194.232152887739)
baselineTurnMiter      = 14·tan(11.25°) + 4·tan(5.625°) = 3.178738756743869   (diagnosis: 3.178738756743869)
```

Both match the diagnosis to the last printed digit, and every per-lane baseline in it reproduces exactly from
`baseline ± 2·offset·miter` (e.g. `clockwise:12 = 6194.232152887739 + 2·12·3.178738756743869 = 6270.521883…`).

Using the measured lane-loop lengths from `a1-highway-diagnosis-20261101.txt`:

| Lane | poly length | lane baseline | poly/baseline | 5×ratio | count | studs/car |
|---|---:|---:|---:|---:|---:|---:|
| clockwise:12 | 10772.736 | 6270.522 | 1.7180 | 8.590 | 9 | 1197.0 |
| clockwise:24 | 10849.026 | 6346.812 | 1.7094 | 8.547 | 9 | 1205.4 |
| clockwise:36 | 10925.315 | 6423.101 | 1.7009 | 8.505 | 9 | 1213.9 |
| counterclockwise:12 | 10620.156 | 6117.942 | 1.7359 | 8.680 | 9 | 1180.0 |
| counterclockwise:24 | 10543.867 | 6041.653 | 1.7452 | 8.726 | 9 | 1171.5 |
| counterclockwise:36 | 10467.577 | 5965.363 | 1.7547 | 8.774 | 9 | 1163.1 |

`min(10, ceil(5·ratio)) = 9` in all six lanes → **54 total**. This is exactly what the design documents: row 8 (line 30) "**5-10 cars per lane; today 9 per lane on the default seeds**, about one car per 1,100 studs (~310 m) of lane", and line 56 "one per ~1,100 studs, and **26 of 54 cars**". The derived ~1,163–1,214 studs/car is consistent with "about one per ~1,100", and 54 sits inside the sim's own cap (`TrafficSim.HIGHWAY_CANDIDATE_LIMIT = 60` = 10·6). **54 is correct per the design.**

Cross-artifact consistency of the spacing number: the pre-change spacing probe gives `clockwise:12` gap/length `0.07438744904680386`; × 9 cars = `0.6694870414212347` — exactly the `minimumTrafficSpacingSlots` the maker self-check reports. The measurement path is unchanged apart from the normaliser.

**Diagnosis-claim audit:**

| Claim (brief/probe) | Verdict | Evidence |
|---|---|---|
| routes = 2 | true | `Graph.luau:519-522` (`clockwise`, `counterclockwise`); also spec's preserved 100-seed check |
| offsets = 3, `laneOffsets = [12,24,36]` | true | `Graph.luau:518`; spec asserts the same values (untouched hunk) |
| poly ≈ 1.7× baseline | true | measured 1.7009–1.7547; my model `baseline + 2·extensionLength ± 2π·offset` reproduces the measured polys within 0.05% |
| 9 cars/lane = 54 | true | table above; design row 8 + line 56 |
| min spacing 0.669–0.769 slot | **partly imprecise** | per-lane minima are 0.669, 0.769, 0.807, 0.809, 0.818, 0.842 slot (old ratios × 9). True range 0.669–0.842; all ≥ 0.6. The brief's upper bound understates one lane; the change and its assertion are unaffected. |
| spec stale vs design, not a code regression | true | `bdcfcb9` (2026-10-18 13:31) added `baseline`/`count` (`git show bdcfcb9 -- src/shared/TrafficSim.luau` lines 64/72-73) and did not touch `tests/studio/highway.spec.luau`; that spec's last change is `c89edb9` (2026-10-14 19:47), still asserting five per lane. |

Nothing here hides a defect: a real under- or over-population, a mis-derived lane, or clustering would all fail §2's assertions; the only constant removed is the stale 30/5-per-lane one.

---

## 4 · Sealed artifacts and pin — PASS

```
$ sha256sum build/release.rbxlx build/test.rbxlx
84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce  build/release.rbxlx
c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98  build/test.rbxlx
build/release.rbxlx 1990548
build/test.rbxlx    2403691
$ sha256sum ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx
c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98  …/usg-desktop.rbxlx
```

* Both pinned digests and sizes match the candidate-identity receipt (`d2-candidate-identity-20261101.json`: release 1,990,548 B, test 2,403,691 B), and the maker log records the same pair before/after its self-check.
* The commit contains only `tests/studio/highway.spec.luau` (`git show --stat HEAD`), so no build output or pin document is in it; `git status` shows no modified tracked file.
* `tools/studio.py sync --test` rebuilds the script trees in the **live Studio DataModel over MCP** (it re-creates `ReplicatedStorage/Nightshift/Shared`, `ServerScriptService/Nightshift`, StarterPlayer scripts from an HTTP manifest); it does not rebuild or rewrite the `.rbxlx`, consistent with the on-disk open place still hashing `c6651793…`. The sealed runner place is unchanged; only the in-memory DataModel carries the new spec, as intended by the "no rebuild" ruling.

---

## 5 · Limits of this verdict (honest)

* **No execution by me.** I did not run Studio, `tools/studio.py tests`, or `tools/usg test --studio` (explicitly out of scope: no Studio/GUI/process driving). The maker self-check log (`a1-highway-spec-selfcheck-20261101.log`) is maker-run; I corroborated its numbers against the independent arithmetic and the pre-change probes (exact agreement) but did not observe the run.
* **A1 is not green.** This PASS says the spec change is correct; A1 must be re-run from `prep/a1-highway-spec-20261101 @ 1abe303` in the owner-opened Studio. **Do not use `tools/usg test --studio` for this re-run:** that wrapper calls `build()` first (`tools/usg:90-91`), which rewrites `build/release.rbxlx` and `build/test.rbxlx`; the test place embeds the spec source, so its digest would stop being `c6651793…` and the sealed-artifact condition would be broken. Use the no-rebuild pair the maker used: `python3 tools/studio.py sync --test` then `python3 tools/studio.py tests` (`tools/studio.py` contains no `build()` call; `sync` only rebuilds the script trees in the live DataModel). If a full-matrix `usg` run is ever required, do it on a scratch copy of the tree, never against the sealed artifacts. Expected `highway.spec` result: `passed=2`, `seedsPassed=100`, `expectedTrafficCount=54`, `minimumTrafficSpacingSlots≈0.6694870414212347`, `roadSamples=3142`, `groundSamples=5`, `tests_rc=0`.
* Record `sha256sum build/release.rbxlx build/test.rbxlx` (and the open place) before and after the re-run; both must stay `84324d43…` / `c6651793…`.
* After the run, the branch needs a rebase/merge onto `main` (`8dfb204`); the merge is clean for this file, but the spec should be re-run on the merged tree before A1 is declared green and B1 unblocked.
* No images were read (host image-decode crash class avoided); all evidence above is git/text/markdown.

### Non-blocking observations

1. **Shared-count equality is stricter than the design row 8 range (5–10/lane).** `:210` requires all six lanes to hold the same integer count. At seed 918273 all are 9, so it holds; but a future geometry change could legitimately yield 8 and 9 across lanes and fail this spec without violating the design. The spec only builds seed 918273, so this is a future-seed false-failure risk, not a false pass — worth knowing before anyone parameterises the seed.
2. **Row 7's first clause** ("varies by more than 5 % of a slot") is not asserted here; it is already asserted at unit level: `tests/unit/traffic-nht.spec.luau:367` (`minimum >= 0.6·slot`) and `:368` (`maximum - minimum > 0.05·slot`) — so the design predicate stays covered; this Studio spec correctly scopes itself to the built-world cohort and the 0.6 floor.
3. **Report-field rename** (`minimumTrafficSpacingRatio` → `minimumTrafficSpacingSlots`, plus `expectedTrafficCount`): no repo script reads the old name (only historical `docs/verify/…` logs and the sealed `build/test.rbxlx` embed it). Any external A1 gate that parses the old field name must be updated, or it will read `nil`.

---

## Evidence index

* `board/probes/team-d/a1-highway-spec-change-20261101.diff` (148 lines; byte-identical to the commit diff)
* `board/probes/team-d/a1-highway-spec-change-20261101.md` (note + re-check procedure)
* `board/probes/team-d/a1-highway-spec-selfcheck-20261101.log` (maker self-check; not certification)
* `board/probes/team-d/a1-highway-diagnosis-20261101.txt`, `a1-highway-spacing-20261101.txt` (Studio probes, pre-change)
* `board/probes/team-d/a1-highway-finding-20261101.md`, `a1-highway-design-decision-20261101.md`, `a1-in-studio-20261101.log`
* `board/probes/team-d/d2-candidate-identity-20261101.json` / `.md` (sealed candidate pin)
* Repo: `tests/studio/highway.spec.luau` (new), `src/shared/TrafficSim.luau`, `src/shared/Graph.luau`, `docs/TRAFFIC_SPEC.md`, `tests/unit/traffic-nht.spec.luau`

**Overall: PASS** — the spec change correctly aligns the test to the design contract; A1 may be re-run from the prep branch after this PASS.
