# CHECKER — Team D Plan 4 (Roblox B1 atlas named-seed assert-and-skip) — verdict: **ACCEPT** (with one named diagnostic gap, C1)

Fresh-context independent check of the frozen Plan 4. I built none of it (maker-never-checker). No Play, no Studio input,
no image reads/screenshots, no host input, **$0**. Everything ran from a throwaway clone of the frozen commit
(`/tmp/checker-plan4/repo`, `git clone --no-hardlinks` at 2026-11-01T20:57:27-05:00); mutations on a second clone
(`/tmp/checker-plan4/mut`), restored to the original sha256 after each. The maker's working tree was only read.

- **Frozen commit:** `9022c538966e84b014607e565ccfb61845458c60` (`9022c53`), parent `90ac9326dfb0…` (accepted Plan 3),
  branch `prep/b1-capture-format-20261101` tip, tree `53fbc06109aef23a5bd49d24049f05afc4c03e4c`; tracked mods **0**
  (792 pre-existing untracked evidence lines, unchanged); sealed `build/release.rbxlx` `84324d434266…` and
  `build/test.rbxlx` `c6651793ace0…` unchanged before/after every run; maker HEAD still `9022c53` at the end.
- **Scope:** exactly **9 files**, `+185/−71` (the claim/report says +186/−71 — off by one insertion; see C3), paths:
  `docs/art/SIDECAR.md`, `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md`, `tests/atlas/{Server,WorldSeed}.luau`,
  `tests/tools/{test_atlas_acceptance,test_vinegar_run}.py`, `tests/unit/world-seed-override.spec.luau`,
  `tools/{atlas_acceptance,vinegar_run}.py`. **No `src/`, `assets/`, `build/` or project-file change**; the only docs
  delta is the checker's table-header newline nit (`SIDECAR.md`: `…acknowledgement fields:` + blank line + table).

## Verdict in one line

**ACCEPT** — the Plan 4 behaviour is implemented, tested and mutation-proven: assert-and-skip replaced the in-Play
rebuild (`WorldSeed.requireApplied`, no `generateSafe` call path, no `WorldSeed.ensure`), a named fail-fast on mismatch,
a 45 s first-view bound, the trace in `AtlasWorldTrace`/inventory/`world-trace.json`/`result.json.worldTrace` and the
runbook/seed contract; all six mutation proofs reproduce, all suites reproduce (84 / 313 / 15 / 10-0 / 919-0) and the
scope is pure. One named diagnostic gap (C1): on the mismatch fail-fast the trace is not yet published, so
`AtlasWorldTrace` is nil in the failed-starter error report (the named error still carries the observed/expected
identity); smallest fix is a one-line move. This does not affect the plan's safety purpose (no in-Play rebuild, bounded
detection) and is not a Play blocker, but the Lead should have it fixed before the atlas diagnostic Play if the
structured fail-fast trace is wanted.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | Provenance / freeze / tracked cleanliness | **PASS** | HEAD `9022c53`, parent `90ac932`, tree `53fbc061…`, branch tip, tracked mods 0, sealed hashes unchanged (provenance.log) |
| 2 | Claim 1 — `WorldSeed.requireApplied` replaces `ensure`, no rebuild path | **PASS** | `tests/atlas/WorldSeed.luau`: `requireApplied` returns `{fastPath=true, rebuilt=false}` on seed+layout match; mismatch raises exactly `named seed not applied before Play: world reports seed X layout Y, expected 2134783819/2; refusing in-Play rebuild`; `WorldSeed.ensure` absent; `generateSafe` appears only in comments (no call). World-seed spec 10/0 (2 specs) |
| 3 | Claim 2 — Server assert-and-skip + trace | **PASS with C1 caveat** | `tests/atlas/Server.luau:63-91`: trace `{t0, rootFoundAt, worldReadyAt, actualSeed, actualLayout, expectedSeed, expectedLayout, fastPathApplied, rebuild, starter}`; published as `workspace:SetAttribute("AtlasWorldTrace",Http:JSONEncode(trace))` at `:85` **before the view loop/captures**; `inventory.worldTrace=trace` at `:91`; `STARTER_SOURCE` error post carries `worldTrace=workspace:GetAttribute("AtlasWorldTrace")`. C1: publication is after `requireApplied` (`:79`), so a mismatch fail-fast never sets the attribute (error text keeps identity) |
| 4 | Claim 3 — starter trace ordering | **PASS** | `tools/atlas_acceptance.py`: `SEED_PATCH` sets `AtlasStarterTrace` `{t0, applied=S.applied(W), seed, rootSeed}` (root seed read if the root already exists) and is concatenated **before** `STARTER_SOURCE` (`SEED_PATCH + source`), i.e. before `require(h).start(n)`; test asserts the order/shape (`test_atlas_acceptance` 15 OK) |
| 5 | Claim 4 — 45 s bound, trace persistence, runbook | **PASS** | `tools/vinegar_run.py`: `ATLAS_FIRST_VIEW_TIMEOUT = 45`, `Atlas.first_view_timeout = 45`, first-view wait `max(0.05, min(wait, first_deadline-now))`; `ATLAS_WORLD_TRACE` read-only Server exec; `record_world_trace` writes `world-trace.json` and `result_trace` merges `result.json.worldTrace` on success (test asserts), timeout (bounded-first-view test) and the explicit runtime-failure branch; runbook records seed 2134783819 / layoutVersion 2 / assert-and-skip / 45 s / trace |
| 6 | Suites from the fresh clone | **PASS** | runner `Ran 84 tests … OK` (10.732 s); full Python `Ran 313 tests … OK` rc 0 (49.263 s); atlas acceptance `Ran 15 … OK` (0.512 s); world-seed spec `unit: 10 passed, 0 failed (2 specs)`; full Lune `unit: 919 passed, 0 failed (126 specs)` rc 0 (20:57:36–21:09:44, 12m08s) |
| 7 | Mutation proofs (throwaway copy, restored) | **PASS (6/6)** | (A) mismatch returns instead of fail-fast → world-seed `9 passed, 1 failed (2 specs)`; (B) layout leg dropped → `9/1`; (C) 45 s bound removed → runner suite hangs, bounded `timeout -k 5 60` → rc 124, no completion summary, no leftover processes; (D) trace dropped from failure → runner `FAILED (failures=1)`; (E) `AtlasWorldTrace` publication removed → atlas acceptance `FAILED (failures=1)`; (F) starter `applied`/`rootSeed` removed → atlas acceptance `FAILED (failures=1)`. File hashes restored to the frozen values after each |
| 8 | Scope purity / fallback deliberately absent | **PASS** | 9 files, no game/asset/sealed change; fallback (a) (disable Bootstrap in prepare; rebuild from scratch) is **not implemented** — no Bootstrap-disable code in `tools/atlas_acceptance.py`/`tools/vinegar_run.py`/`tests/atlas`; the report names it as the ruled follow-up conditional on the diagnostic Play |
| 9 | No Play / no Studio input | **PASS (as checked)** | I ran no Play/Studio; host Plays counter unchanged (`~/.cache/usg/vinegar-plays.json`, pid 382409, plays 2, mtime 15:53), no latch; sealed hashes unchanged |
| 10 | Trace observability ≤45 s | **PASS for the hang path** | The trace is published before any capture; the runner reads it on the first-view timeout (test patches the bound to 0.05 s and asserts `worldTrace` in the error + `world-trace.json` + 0 captures); real bound 45 s. C1 applies only to the mismatch fail-fast, which fails immediately rather than hanging |

## Named concerns (non-blocking; smallest fixes)

- **C1 (diagnostic gap, one-line fix).** `AtlasWorldTrace` is published at `tests/atlas/Server.luau:85`, i.e. **after**
  `WorldSeed.requireApplied` at `:79`. On the mismatch fail-fast the attribute is never set, so the starter's error post
  (`worldTrace=workspace:GetAttribute("AtlasWorldTrace")`) carries `nil`; the runner's early-stop path
  (`collector.wait` raises `ValueError("runtime arrived before view-001")`, which `Atlas.run` does not catch) then does
  not write `world-trace.json` either. The named error text still carries the observed/expected identity, and
  `AtlasStarterTrace` is set by the starter, but the structured `fastPathApplied`/`starter`/times are lost on exactly
  the fail-fast path. **Smallest fix:** move the `workspace:SetAttribute("AtlasWorldTrace", Http:JSONEncode(trace))`
  line above `if manifest.namedSeed then` (after `trace.actualSeed,trace.actualLayout=seed,layout`) and republish after
  `requireApplied` (or `pcall` it, publish, then rethrow). Optionally catch `ValueError` in the same trace-keeping
  branch as `TimeoutError`/`FixtureFailed`.
- **C2 (minor, same path).** The early-stop `ValueError` is not in the `except (TimeoutError, FixtureFailed)` clause, so
  the trace-reading/filing is skipped on that path; the error message does include the runtime report JSON (with the
  named error). Fix rides with C1.
- **C3 (bookkeeping only).** `git diff --numstat 9022c53^ 9022c53` sums to **+185/−71** (`git show --shortstat` agrees);
  the report/claim says **+186/−71**. No functional impact.

## Limits (named)

- **No Play was run** (and no Studio input): the live 45 s bound, the trace publication under a real Play and the
  `fastPathApplied` value are proven offline only (unit tests + mutation C hang proof). The plan's acceptance is
  offline; the next atlas Play is the live proof.
- The real seeded manifest's `layoutVersion 2` is asserted from code/pins (`Graph.LAYOUT_VERSION = 2`, the runbook and
  the test fixtures); I did not build a live seeded manifest (no Studio/build) — the runbook's frozen seed/layout claim
  is consistent with the source constant and the pinned plan.
- Full Lune is a regression run (no Lune/src file changed by the commit); timings ~12 min, bounded.
- The 45 s bound is unit-tested with a 0.05 s patch plus the mutation that removes it; the constant itself is 45.
- No game code, asset, sealed artifact, Play, Studio input, credentials, images or host input; $0.

## Evidence paths

`board/probes/checker-plan4/` — `provenance.log` (HEAD/parent/tree/tracked/numstat/sum/paths/sealed),
`claims-bytes.log` (claims 1–4 + runbook + SIDECAR diff), `suites-and-scope.log` (84 / 313 / 15 / 10-0 / 919-0, spec
count, fallback search, trace paths), `runner-suite.log`, `python-suite.log`, `atlas-acceptance.log`,
`world-seed-spec.log`, `lune-unit.log` (+ start/end), `mut-c.log`, `mutations.log` (6 proofs + restored hashes),
`trace-path.log` (C1 ordering + early-stop analysis), `run-started.txt`. Throwaway copies:
`/tmp/checker-plan4/{repo,mut}`; maker repo read-only, unchanged.
