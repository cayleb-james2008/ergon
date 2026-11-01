# Chunked runtime transport — implementation report (frozen for checker, 2026-11-01 14:56 ET)

**Commit:** `ec1c39471a1188b1314e8ce3fdd42959174bcde8` on `prep/b1-capture-format-20261101`
(base before work: `271b601`). Tracked working tree clean. **Frozen** — no further source edits until the fresh
checker returns. Evidence log (all test output): `board/probes/team-d/chunk-transport-implementation-20261101.log`.

## Files changed (9)

| File | Change |
|---|---|
| `tests/traffic/Server.luau` | `finish()` sends the envelope via `require(SSS.Nightshift.ExactReportTransport):post(Http, base, {session,request,nonce}, encoded, 2097152)`; `base` derived from `cfg.collector` (`<origin>/report` → `<origin>`), identity nonce is the helper's local `nonce`; keeps the 2 MiB bound; warning names the 1024 KB engine cap + chunk transport |
| `tests/diagnostics/ExactReportTransport.luau` | chunk failure message names `index/count` and the cause; two immediate attempts (no sleeping into the fixture timeout) |
| `tools/traffic_acceptance.py` | `ExactReportTransport` added to `HELPERS` (ModuleScript under `ServerScriptService/Nightshift`) and to the hashed manifest files; `EXPECTED_HELPERS=len(HELPERS)+len(STARTERS)=3`; validators use the computed count |
| `tools/vinegar_run.py` | **only** `traffic:pass` and `traffic:abort-contact` now pass `chunks=True`; all other fixtures unchanged |
| `tests/tools/test_collector_chunks.py` (new) | 9 offline tests: byte-identical reassembly vs direct `/report`, runtime exactly once, dropped chunk (no runtime, exit 1), conflicting duplicate 400, identical duplicate idempotent, out-of-order 400, wrong sha256 400, identity mismatch 400, count/bytes mismatch 400, oversized wire 413 |
| `tests/tools/test_traffic_acceptance.py` | 3-helper manifest + transport assertions |
| `tests/unit/traffic-timing.spec.luau` | transport-path mock (SSS/require, nonce identity) |
| `tools/atlas_capture_index.py` + `tests/tools/test_atlas_capture_index.py` | accept widened capture contract (PNG **or** JPEG originals, optional `mimeType`, header-only dims via `vinegar_lib.frames`) — required so the runner suite stays green after the earlier approved capture-format change |

## Test evidence (all green)

- NEW chunk suite: `pytest … -q` → **9 passed in 2.35 s**; unittest verbose 9/9 OK.
- Existing traffic acceptance 4/4 OK; handling acceptance 8/8 OK; runner fixture suite 68/68 OK; atlas capture index 7/7 OK.
- Luau `traffic-timing` spec: `unit: 4 passed, 0 failed (1 specs)`.
- Full Lune unit suite: **914 passed, 0 failed (125 specs)**, rc 0.
- Full Python `tests/tools` suite: **rc 0** (0 FAIL/ERROR lines).

## Fixture candidate digests (changed by the new helper + hashed files — documented)

- `traffic:pass` = `a8778b1fe0e4e6e58368d9e058a0eb117106e196537fa3be1307755627f24737`
- `traffic:abort-contact` = `b145a164a130dbb560e58390757313dab3b62dcaa398004c588940a15936a37a`
- Previous abort-contact digest observed in the failing prepared report: `ddfcb24c0f4daff883d8628c772f45c53fb26932c8aca17cc6bf10400fdf172a`.
  Helpers now `['HandlingServer','ExactReportTransport']`; `EXPECTED_HELPERS = 3`.

## Sealed artifacts — unchanged before/after

```
84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce  build/release.rbxlx
c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98  build/test.rbxlx
c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98  ~/.var/app/…/usg/usg-desktop.rbxlx
```

No game code, no build, no re-pin, no Play; the module reaches the place through the fixture's Edit-time helper
manifest (`tools/traffic_acceptance.py` HELPERS), which the fresh checker can verify offline.


## Checker fixes — REJECT blockers resolved, re-frozen at `065e0fc`

Commit **`065e0fc`** on `prep/b1-capture-format-20261101` (parent `ec1c394`); tracked tree clean; **untracked = 779
lines, all evidence dirs (`docs/verify/…`, `.tools/probe/…`)**, i.e. "tracked tree clean; untracked = evidence dirs",
not a literally empty worktree. Sealed artifacts unchanged. Evidence log:
`board/probes/team-d/chunk-transport-checker-fixes-20261101.log`.

### BLOCKER 1 — helper-count bookkeeping (real bug, fixed)

`HandlingFamily(...)` registry entries for the two traffic fixtures still passed `helpers=2`, while the installed
helper+starter count is `traffic_acceptance.EXPECTED_HELPERS=3`; `collector.handling_phase(2)` therefore stamped
`cleanup.cleaned=False` (HTTP 200) and `validate_report` ignored the flag. Reproduced before/after with the production
validator:

```
BEFORE registry literal 2:        helpers=2 removedHelpers=3 cleaned=False
AFTER  registry EXPECTED_HELPERS: helpers=3 removedHelpers=3 cleaned=True
registry traffic:pass:        helpers=3 chunks=True
registry traffic:abort-contact: helpers=3 chunks=True
```

Fixes: registry passes `traffic_acceptance.EXPECTED_HELPERS` (computed); `validate_report` now requires
`cleanup.cleaned is True` so misleading evidence fails closed; the synthetic traffic fixture gained `cleaned=True`.

### BLOCKER 2 — test realism (fixed)

`tests/tools/test_collector_chunks.py` now drives the **production validator** `collector.handling_phase(3)` with
production-shaped envelopes: no explicit `phase`, the phase is injected from `report.stage`, and identity nonce lives
in the report. The direct-vs-chunk byte-identity control uses two collectors pinned to the same
`session/request/nonce` so both paths receive the *same bytes*; it asserts reassembled sha/bytes equal, `runtime`
delivered exactly once and `phase=="runtime"` injected. A new test reproduces the bookkeeping: `cleaned` False with
`handling_phase(2)`, True with `handling_phase(3)`.

### Minors (fixed)

- Runner regression test pins the full `FIXTURES → chunks` map (atlas/city + the two traffic fixtures True;
  handling False; first-fare/moving/race None) and asserts both traffic fixtures have `helpers == EXPECTED_HELPERS`
  — a registry edit that silently reverts either now fails the suite.
- Identity negative now mutates **session, request and nonce** (each 400 + runner exit 1).
- The 413 oversized-wire case asserts the **runner exit code 1** as well as the HTTP status.

### Re-freeze test evidence (all green)

New chunk suite `pytest -q`: **10 passed, 5 subtests passed**; verbose 10/10 OK; traffic acceptance 4/4; runner suite
**69/69** (includes the new registry test); Luau `traffic-timing` 4/4; full Lune **914 passed, 0 failed (125 specs)**;
full Python `tests/tools` rc 0. Sealed `84324d43…`/`c6651793…` unchanged before and after the commit.


## Re-check (narrow) REJECT fixes — re-frozen at `c0d4b22`

The re-check accepted both prior blockers (helper count computed 3, `cleaned` False→True, production-validator test
kills the no-op/explicit-phase variants, suites green) but found mutation proof (d) survived: the new
`validate_report` clause had no test. Test-only fix at **`c0d4b22`** (parent `065e0fc`), tracked tree clean; untracked
= 779 evidence-dir lines. Sealed unchanged.

- `tests/tools/test_traffic_acceptance.py`: added the two missing cases to
  `test_rejects_corrupt_or_overclaimed_native_evidence` — `cleanup.cleaned=False` and `cleaned` absent — both must
  raise `ValueError` from `validate_report`.
- `tests/tools/test_collector_chunks.py`: `EXPECTED_HELPERS = traffic_acceptance.EXPECTED_HELPERS` (imported, no
  local literal → no drift on a future helper-set change).

**Mutation proof** (in-memory copy of `tools/traffic_acceptance.py` with the clause removed; log
`chunk-transport-checker-fixes-20261101.log`):

```
cleaned=False  REAL clause present      raised=True (…did not confirm cleaned…)
cleaned=False  MUTATED clause removed   raised=False (accepted)
cleaned absent REAL clause present      raised=True (…)
cleaned absent MUTATED clause removed   raised=False (accepted)
MUTATED: failure/error count = 2/0 (successful=False)  -> the new cases FAIL as required
RESTORED: failure/error count = 0/0 (successful=True)  -> the new cases PASS with the real clause
```

Re-freeze evidence: full Python `tests/tools` **286 tests OK rc 0**; chunk suite 10/10 OK; traffic acceptance 4/4 OK;
Luau traffic-timing 4/4. Frozen for the fresh checker; no Play.
