# CHECKER — Plan 3 (bounded generation instrumentation + offline lighting technology) — verdict: **ACCEPT**

Fresh-context independent check of Team D Batch 8's Plan 3 freeze. **I built none of it** (maker-never-checker).
All execution ran from a **fresh copy of the commit**, never the maker's working tree. **No Play, no Studio input, no build,
no image reads/screenshots, no host input injection, $0.** Report to `lead` only.

- **Checked commit:** `90ac9326dfb0037d5af7505d64bd230d1213fb88` (tree `8a488c490ab8a82622b68ec6323d754714f7a472`),
  parent `29a626b2b8a13556ac5ee07db6fdb60401ecc6e8` (= accepted Plan 2, `board/reports/checker-roblox-plan2.md`
  verdict ACCEPT), branch `prep/b1-capture-format-20261101` tip; tracked tree clean; 792 pre-existing untracked
  evidence lines; **17 files, +779/−90**.
- **Fresh copy:** `/tmp/checker-plan3/repo` from `git archive 90ac932 | tar -x` (2026-11-02T01:26Z). **1442 tracked
  entries, blob-for-blob equal to the commit tree** (0 missing / 0 mismatch; the one symlink `CLAUDE.md -> AGENTS.md`
  handled). `build/` is gitignored and untracked in both parent and commit, so the two sealed artifacts were copied
  read-only into the copy and hash-checked (unchanged before/after; §2).

## Verdict in one line

**ACCEPT** — `90ac932` is exactly the disclosed Plan 3: (a) a bounded ≤20-event physical-generation recorder that keeps
the traffic probe **fail-closed** (`error(violation)`), and (b) an **offline sealed-place-XML** `Lighting.Technology`
derivation whose sidecar field stays **required** and is **hash-bound** to the build. All five claimed suite results
reproduce from the fresh copy (83 OK, 311 OK, 919/0 across 126 specs, 7 OK, 6 OK), **all six mutation proofs reproduce**
(3/1/3/1/2/1), the sealed artifacts are unchanged, and no game code or out-of-scope contract change is in the commit.
One docs-formatting defect found — named, non-blocking (§6).

## Per-leg table

| # | Leg (claim) | Verdict | Evidence |
|---|---|---|---|
| 1 | **3a guard**: `tests/diagnostics/GenerationGuard.luau` bounded ≤20 events, compares body identity / `body.Parent` / `car.generation` each step, fixed event shape, returns the fail-closed message | **PASS** | source read (`MAX_EVENTS = 20`; `changed` on generation drift / `bodyAlive` / `sameBody`; event keys incl. `source="step"`; message `physical generation retired/replaced at step N: old->new`); Lune spec **5 passed**; mutation F reproduces the bound |
| 2 | **3a probe**: `tests/traffic/Server.luau` installs the guard as the 4th helper, replaces the bare assert but still `error(violation)`, `report.diagnostic` populated on pass and failure | **PASS** | diff + source probe: `require(SSS.Nightshift.GenerationGuard)`, `guard=GenerationGuard.new(car.generation)`, `if violation then report.diagnostic=guard:report(car.generation);error(violation) end`; finish path attaches `guard:report(...)`; bare assert absent; `EXPECTED_HELPERS=4`, manifest helpers = HandlingServer, ExactReportTransport, GenerationGuard |
| 3 | **3a validator**: `validate_diagnostic` shape/bound (≤20, finite clocks/generations/counters, boolean legs, `source="step"`) and a **passing** runtime must show `generationStable=true`, no events, `finalGeneration==fixture.generation` | **PASS** | direct probe: baseline accepted; pass-with-instability rejected (`generation instability recorded on a passing run`); 21 events rejected (`exceeds the bounded 20`); failed runtime still rejected (`did not finish`) with its events kept; mutations A/B/C reproduce 3/1/3 |
| 4 | **3b record**: `tools/vinegar_lib/place_lighting.py` derives raw int (4=Future) + file + sha256 + enum mapping + project cross-check | **PASS** | direct derive on the sealed XML: `raw=4, value=Future, enum=Enum.Technology, source=sealed-place-xml`, sha256 `c6651793…` = the sealed test build, mapping `0=Legacy;1=Voxel;2=Compatibility;3=ShadowMap;4=Future`, project `Future`; fail-closed cases covered by `test_place_lighting` **6 OK** |
| 5 | **3b runner**: derives ONCE before any Studio change; derivation failure = clean refusal, no latch, no Play | **PASS** | code order in `run_fixture`: derive inside PREFLIGHT, **before** `connect`/`play_budget`/latch; `test_place_technology_derivation_failure_is_a_clean_refusal` asserts `EXIT_REFUSED`, state `PREFLIGHT`, `read_latch is None`, `start_stop_play` calls = 0 |
| 6 | **3b sidecar**: `technologySource:"sealed-place-xml"` + `technologyEvidence`, claims NO engine read | **PASS** | direct `frames.sidecar` probe: `lighting = {technology:"Future", technologySource:"sealed-place-xml", technologyEvidence:{file,sha256,raw,enum,mapping,project}}`; probe's own `technologyReadable:false` is **never required** (sidecar still complete); missing record → `ValueError` fail-closed |
| 7 | **3b judge**: `look_metrics` hash-binds to `--build`; stale/wrong XML invalidates | **PASS** | direct probe: correct build → `[]` reasons; stale sha256 / wrong raw+value → `does not match the measured build … (stale or wrong XML)`; `measure` derives the build once and passes it to `capture_validity`; mutation E reproduces 2 failures |
| 8 | **3b required field not softened** | **PASS** | missing record refused in `frames`; `technology_problems` emits reasons for missing value/source/evidence/build; passing-run stability is enforced in the traffic validator (leg 3) |
| 9 | **Scope purity**: no game code, no sealed-artifact change, no capture/sidecar contract change beyond the ruled technology-source field; captures depending on the derived field = moving frames + all `atlas:*` + B3 look packs (not `first-fare`, race, handling/city/traffic) | **PASS** | `git diff --name-only` has **no `src/`, `assets/`, `deploy.json`, `default.project.json`, `build/`**; `frames.sidecar` is called only from `Atlas` (all 5 views) and `FirstFare` when `moving=True` — `first-fare` constructs `FirstFare(moving=False)` so it captures no frames; `look_metrics` now requires the build-bound evidence for every pack it measures (B3 look packs); race/handling/city/traffic write no sidecars |
| 10 | **Suites**: runner 83 OK; full Python `tests/tools` 311 OK; full Lune 919 passed/0 failed/126 specs; traffic-acceptance 7 OK; place_lighting 6 OK | **PASS** | re-run from the fresh copy — see §3 |
| 11 | **Mutation proofs** (six) | **PASS** | independently reproduced — see §4 (3 / 1 / 3 / 1 / 2 / 1 exactly as claimed) |
| 12 | **Sealed** `84324d43…` / `c6651793…` unchanged | **PASS** | measured `84324d43…` (1,990,548 B) release, `c6651793…` (2,403,691 B) test, Studio place = `c6651793…`; mtimes 2026-10-21 (predate the batch); identical to the Plan 2 accepted baseline; unchanged after every run |

## 1. Provenance — PASS

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD`, `HEAD^{tree}`, `90ac932^`, branch tip (maker repo, read-only) | `90ac9326dfb…`, `8a488c490ab8…`, `29a626b2b8a1…`, tip = `90ac9326dfb…` |
| 2 | `git branch --contains 90ac932` | only `prep/b1-capture-format-20261101` |
| 3 | `git status --porcelain=v1` | 0 tracked modifications; 792 pre-existing `??` evidence lines |
| 4 | `git diff --stat 29a626b 90ac932` | 17 files, +779/−90 (matches the report exactly) |
| 5 | fresh copy: `git archive 90ac932 \| tar -x` + blob-for-blob compare of all 1442 entries | **0 missing / 0 mismatch** → the evaluated tree **is** `8a488c4…` |
| 6 | `build/` in either commit | 0 entries (untracked); sealed artifacts copied read-only and hashed |

No remotes are configured in the repo; provenance is local-object based (commit object, tree, parent, branch containment,
fresh-copy blob equality) — same basis the Plan 2 checker used.

## 2. Scope — PASS, no out-of-scope change

- `git diff --name-only 29a626b 90ac932`: 17 files — 2 docs, 3 tools, 2 new modules/tests (`GenerationGuard.luau`,
  `place_lighting.py`), 5 test files, 3 unit specs, 1 test fixture (`tests/traffic/Server.luau`).
- **Untouched:** `src/` (no game code), `assets/`, `deploy.json`, `default.project.json`, `build/` (no sealed artifact is
  even tracked). `git diff` scope-purity grep for `^src/|^assets/|^deploy.json|^default.project.json|^build/` = empty.
- The only sidecar-contract change is the ruled one: `report.lighting.technology` read path moves offline, plus the
  explicit `technologySource` / `technologyEvidence` fields (`docs/art/SIDECAR.md` + `frames.py` + `look_metrics.py`).
  No other required field was relaxed or removed.

## 3. Suites re-run from the fresh copy — PASS (all five match the claim)

| # | Command (cwd = `/tmp/checker-plan3/repo`) | Actual result |
|---|---|---|
| 1 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 83 tests in 11.076s — OK**, rc 0 |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_traffic_acceptance.py'` | **Ran 7 tests in 0.079s — OK**, rc 0 |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_place_lighting.py'` | **Ran 6 tests in 0.016s — OK**, rc 0 |
| 4 | `python3 -m unittest discover -s tests/tools` | **Ran 311 tests in 49.528s — OK**, rc 0 |
| 5 | `./.tools/bin/lune run tests/unit/run.luau` (lune 0.10.5) | **unit: 919 passed, 0 failed (126 specs)**, 0 `FAIL` lines, ≈11 min; `ls tests/unit/*.spec.luau \| wc -l` = **126**. My log (16,677 B) differs from the maker's log only in 8 nondeterministic `synthetic id …` lines — everything else, including the final counts, is identical |
| 6 | sealed hashes after every run | unchanged (`84324d43…`, `c6651793…`, Studio place `c6651793…`) |

## 4. Mutation proofs independently reproduced — PASS (throwaway `/tmp/checker-plan3/mut`; each file restored and `cmp`-identical after)

| # | Mutation | Claim | Actual |
|---|---|---|---|
| A | remove `validate_diagnostic(report)` from `validate_report` | 3 fails | **FAILED (failures=3)** ✔ |
| B | remove the `≤20` bound check | 1 | **FAILED (failures=1)** ✔ |
| C | remove the pass-with-instability check | 3 | **FAILED (failures=3)** ✔ |
| D | remove `error(violation)` from `tests/traffic/Server.luau` | 1 | **FAILED (failures=1)** ✔ |
| E | remove the `look_metrics` build-hash binding | 2 | **FAILED (failures=2)** ✔ |
| F | remove the recorder bound in `GenerationGuard.luau` | Lune 1 fail | **unit: 4 passed, 1 failed (1 specs)** — `FAIL the event list is bounded at 20 and truncation is recorded: value expected 20, got 21` ✔ |
| — | restore | — | filtered guard spec back to **5 passed / 0 failed**; mut-tree diff vs fresh copy = only `__pycache__` `.pyc` files (no source residue) |

## 5. Behaviour / contract spot-checks (no Play needed) — PASS

Direct probes from the fresh copy (scripts + output under `board/probes/checker-plan3/`):

- **(a) generation guard**: `GENERATION_MAX_EVENTS=20`, `EXPECTED_HELPERS=4`, manifest helpers `HandlingServer,
  ExactReportTransport, GenerationGuard`; baseline report accepted; a passing report carrying any event / `stable=false` /
  `finalGeneration!=fixture.generation` is rejected; >20 events rejected; a failed runtime is still rejected while its
  events are kept; probe source has `guard:observe` + `error(violation)` and the bare assert is gone.
- **(b) offline technology**: sealed XML derives `raw 4 = Future` with sha256 `c6651793…` and project cross-check
  `Future`; a sidecar built from that record carries `technologySource:"sealed-place-xml"` + evidence
  `{file, sha256, raw, enum, mapping, project}` and is complete **even when the probe reports
  `technologyReadable:false`**; `look_metrics.technology_problems` returns `[]` for the correct build and the
  stale/wrong-XML reason for a mismatched build; a sidecar without the record raises
  `place technology record missing` (field stays required).

## 6. Named non-material findings (do not block Plan 4)

1. **`docs/art/SIDECAR.md` line 51 — malformed table header (docs-only).** The new sentence ends
   `…look_metrics needs these acknowledgement fields:| Field | Accepts |` on one line, so the atlas-acknowledgement table
   loses its header row and renders as text. Content is correct and the code enforces it; no functional effect.
   **Smallest fix** (next commit, not a re-freeze trigger): insert a newline before `| Field | Accepts |`.
2. **Report §2 wording:** "The probe's own `technologyReadable:false` observation is kept as evidence only". The probe
   still records it in its reply and it is never required (verified), but the sidecar no longer carries it
   (`look_report` builds `lighting` from the offline record only). The substantive claim (never required, no engine-read
   claim) is verified; "kept as evidence" is not in the capture artefact.
3. **Plan-doc vs frozen code:** the plan note said "validation stays as-is"; the frozen commit adds the validator
   shape/bound + passing-run stability checks. That is the Lead's ruled strengthening (the task's claim list), not a
   softening — recorded so the delta is not mistaken for scope creep.

## 7. Limits (named)

- **No Play, no Studio input**: the instrumentation's behaviour in a real `traffic:abort-contact` Play and a real capture
  sidecar through Studio are **not** exercised here; they are verified by the Lune unit suite, synthetic Python fixtures,
  and source contracts only. The gated diagnostic Play remains the first real-runtime evidence.
- **No build**: the derivation is verified against the existing sealed `build/test.rbxlx` (raw 4/Future); it does not
  prove what a future rebuilt place will contain — that is exactly why the field is hash-bound.
- The maker's "no Play / no Studio input this batch" is a fence claim, not provable from the commit; what is provable is
  that the commit adds no capture artefacts and the sealed artifacts are byte-unchanged.
- `first-fare` is correctly *not* affected because `FIXTURES["first-fare"]` builds `FirstFare(moving=False)` and only the
  `moving` branch writes frames/sidecars (verified in code, not by a capture).
- Lune output contains nondeterministic `synthetic id` lines; the pass/fail counts and all assertions are deterministic
  and match the maker's log.

## 8. Evidence (checker-produced, under `board/probes/checker-plan3/`)

`provenance-20261102.txt`, `sealed-hashes-20261102.txt`, `suites-20261102.txt`, `mutations-20261102.txt`,
`contract-probe-20261102.txt` (3b sidecar + look_metrics binding), `generation-contract-probe-20261102.txt` (3a guard +
validator), `lune-unit-fresh.log` (full 919/0/126 run). Fresh copy `/tmp/checker-plan3/repo`; mutation copy
`/tmp/checker-plan3/mut` (restored).

## 9. Fences held

Read-only against the maker repo; execution only in throwaway copies. **No Play, no Studio launch/input, no build, no
upload/publication, no image read/screenshot, no host input injection, no credential value, $0.** I made none of the
checked work.

---

**— checker-trading (fresh-context, Plan 3 seat), 2026-11-01 20:34 ET (EST, UTC−5) = 2026-11-02 01:34Z.**
**Verdict: ACCEPT** for `90ac9326dfb0037d5af7505d64bd230d1213fb88`; one non-blocking docs nit (§6.1) for the next commit.
