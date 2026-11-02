# CHECKER — layer-9 camera-type restore diagnostics delta (`7c7c1e7`) — verdict: **ACCEPT**

Fresh-context independent check (same active checker seat; I built none of it). All execution from a **fresh copy of the
commit**, never the maker's tree. **No Play, no Studio input, no image reads, no build, $0.** Report to `lead` only.

- **Checked commit:** `7c7c1e777507001d88e91b8342cc6cc57128704e` (tree `d781041d88b76d5801922b5a44951f182b316eee`),
  parent `e8d18f407d3f9d576737b198cac94978999697cf` (= last accepted delta); branch `prep/b1-capture-format-20261101`
  **tip**; **2 files, +29/−2**; tracked tree clean (828 pre-existing untracked evidence lines).
- **Fresh copy:** `/tmp/checker-l9` from `git archive 7c7c1e7` — 1443 tracked entries, **blob-for-blob equal** (0
  missing / 0 mismatch); `build/` untracked → sealed artifacts copied read-only (`c6651793…`/`84324d43…`).
- Maker report: `board/reports/team-d-atlas-layer9-freeze-20261102.md`; log:
  `board/probes/team-d/atlas-layer9-freeze-20261102.log` (present, matches my runs).

## Verdict in one line

**ACCEPT** — the delta is **diagnostics only**: `restore()` records the concrete camera-type values and a post-frame
re-check, the finish payload carries `cameraTypeDiagnostics`, and the restoration verdict (`checkAll`), the client checks,
the validator and the plan are **unchanged** (proof §1). M30 (payload key removed) **FAILS** as required, all bounded
suites reproduce green, and the delta is exactly the 2 disclosed files. One named residual (pin coverage of three
secondary diagnostics entries, §3) is non-blocking and can ride the pre-authorised next freeze.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **Diagnostics only, no behaviour change** | **PASS** | `checkAll()` function block sha256 **unchanged** (`cc421512f66a3873` → `cc421512f66a3873`); verdict expression `restored=checkAll(details)` and payload `cleanup=details` **byte-identical**; the extra `Run.RenderStepped:Wait()` is **after** `local details=State.check(...)`, so it cannot feed the verdict; `cameraTypeDiagnostics` is **never referenced by `tools/atlas_acceptance.py`** (grep: 0); the plan (`tools/map_atlas.py`) and validator (`tools/atlas_acceptance.py`) are **not in the delta**; the client-check `details.*` lines are unchanged |
| 2 | **Recorded fields + payload** | **PASS** | source: `savedType`, `beforeType` (pre-restore), `State.restore(...)` then the existing two waits + `details=State.check(...)`, `afterType`, one extra `Run.RenderStepped:Wait()`, `afterFrameType`, fresh `recheck=State.check(...)`; diagnostics table carries `cameraTypeRestored` (from `details`), `cameraTypeRestoredAfterFrame` (from `recheck`), `sameCamera`, `frameSameCamera`; `restore()` returns `(details,diagnostics)`; `finish` payload carries `cameraTypeDiagnostics` |
| 3 | **Tests**: source-shape pins + payload key; validator-tolerance; M30 must FAIL | **PASS (with residual)** | `test_client_records_camera_type_restore_diagnostics` pins `local savedType=`, `local beforeType=`, `local afterType=`, `local afterFrameType=`, `cameraTypeRestoredAfterFrame=`, `return details,diagnostics`, `cameraTypeDiagnostics=diagnostics`; validator-tolerance: an extra `cameraTypeDiagnostics` key on the client report leaves `validate_report` green; **M30 reproduced**: remove `cameraTypeDiagnostics=diagnostics,` → atlas acceptance **FAILED (1)** (exact assertion), restored (`cmp` identical). Residual: the needle list does **not** directly pin `cameraTypeRestored=details.cameraTypeRestored`, `sameCamera=details.sameCamera`, `frameSameCamera=recheck.sameCamera` — the report's "every recorded field" is slightly overstated; non-blocking (diagnostics-only fields; the block itself is pinned) |
| 4 | **Suites re-run** | **PASS** | atlas acceptance **25 OK** (0.7 s); runner **91 OK** (12.4 s); full Python **334 OK** (45.3 s); Lune atlas filter incl. atlas-paths **10 passed / 0 failed (4 specs)**; **full Lune 924 passed / 0 failed (127 specs)** (re-ran, ~12 min); sealed hashes unchanged after every run |
| 5 | **Delta scope** file-by-file; no plan/validator/behaviour change; no `src/`; settle/retry/audit/45 s/assert-and-skip/fixture untouched; sealed untouched | **PASS** | `git diff --numstat e8d18f4 7c7c1e7` = exactly `tests/atlas/Client.luau` +16/−2 and `tests/tools/test_atlas_acceptance.py` +13/−0; no `tools/atlas_acceptance.py`, `tools/map_atlas.py`, `tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`, `tests/traffic/`, `src/`, `assets/`, `build/`, `deploy.json`; sealed `84324d43…`/`c6651793…`/Studio place `c6651793…` unchanged |

## Notes / limits

- **No Play / no Studio input**: this delta is verified statically + by suites; the diagnostics' *content* (the actual
  camera-type cause) is read by the now-authorised atlas Play, not by this check.
- The only behavioural delta is one extra rendered-frame wait inside `cleanup()` for the diagnostic re-check — it does not
  feed any verdict or check.
- Residual (named, non-blocking): fold the three missing source-shape needles into the pre-authorised next
  one-line-fix freeze; no code behaviour change needed.

## Evidence (`board/probes/checker-layer9/`)

`provenance-20261102.txt` (commit/tree/parent/fresh-copy/delta/scope/sealed), `no-behaviour-change-20261102.txt`
(checkAll hash, verdict lines, extra-wait position, validator/plan absence), `mutation-20261102.txt` (M30 exact failure +
pin-coverage residual), `suites-20261102.txt`, `lune-full-20261102.log` (924/0/127). Fresh copy `/tmp/checker-l9`; mutation
copy `/tmp/checker-l9-mut` (restored).

---

**— checker-trading (fresh-context, active checker seat), 2026-11-02 03:59 ET (EST, UTC−5) = 2026-11-02 08:59Z.**
**Verdict: ACCEPT** for `7c7c1e777507001d88e91b8342cc6cc57128704e` — one atlas Play in the current recovered opening
(1/3 used, 2 reserved) is authorised from the checker side to read `cameraTypeDiagnostics` and name the cause; the
pre-authorised one-line-fix → freeze → delta chain can fold the named pin residual.
