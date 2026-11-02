# CHECKER — round-3 camera-restore diagnostics delta (`0ec48d6`) — verdict: **ACCEPT**

Fresh-context independent check (same active checker seat; I built none of it). All execution from a **fresh copy of the
commit**, never the maker's tree. **No Play, no Studio input, no image reads, no build, $0.** Report to `lead` only.

- **Checked commit:** `0ec48d6768224721dbd5987dfb058a2e2b6b4c52` (tree `31ad4db899ddaee78c08cb3c4916abab8f426c22`),
  parent `618bb4a4473bc5d46eb15a4611cb1fa55f53cdbb` (= last accepted delta); branch
  `prep/b1-capture-format-20261101` **tip**; **2 files, +22/−10**; tracked tree clean (833 pre-existing untracked
  evidence lines).
- **Fresh copy:** `/tmp/checker-r3` from `git archive 0ec48d6` — 1443 tracked entries, **blob-for-blob equal** (0
  missing / 0 mismatch); `build/` untracked → sealed artifacts copied read-only (`c6651793…`/`84324d43…`).
- Maker report `board/reports/team-d-atlas-round3-freeze-20261102.md` + log
  `board/probes/team-d/atlas-round3-freeze-20261102.log` read; numbers match my runs.

## Verdict in one line

**ACCEPT** — the delta is **diagnostics-only**: the deferred re-assert now runs whenever
`details.cameraTypeRestored ~= true` (pcall-wrapped, recorded), followed by a **5-frame `CameraType` series**; the
**verdict path is byte-unchanged** (`checkAll` block sha identical, `restored=checkAll(details)` and `cleanup=details`
unchanged, details computed before the re-assert, payload never references the diagnostics), no fix is smuggled into the
verdict, M33/M34 fail as required, and all suites reproduce. Named residual (non-blocking, §4): the `afterReassertType`
pin still matches an assignment line rather than the recorded field.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **Diagnostics only**: re-assert runs whenever `details.cameraTypeRestored ~= true` (pcall, recorded), then a 5-frame series (`frameSeries[{frame,type}]`) + `afterReassertType`/`reassertFinalType`; **verdict/`restored` still computed exactly as before from `checkAll(details)`** and never from diagnostics | **PASS** | `checkAll` exact block sha **`ad361c16af283f97` → identical**; `restored=checkAll(details)` and `cleanup=details` unchanged byte-identical; `local details=State.check(...)` is **before** the re-assert block (source index verified); finish payload never references `frameSeries`/reassert fields (grep = False); `cameraTypeDiagnostics` is ignored by the validator (tolerance test); source: `local attempted=details.cameraTypeRestored~=true`, pcall, `for frame=1,5 do … table.insert(frameSeries,{frame=frame,type=…})`, `afterReassertType`/`reassertFinalType` |
| 2 | **No fix smuggled in**: the re-assert remains a conditional *attempt*, recorded only; the verdict still fails closed on a genuine mismatch | **PASS** | re-assert is conditional, `pcall`-wrapped, recorded (`reassertOk/reassertError`); it runs **after** `details`/`recheck`; the payload's `restored=checkAll(details)` and `cleanup=details` are computed before it and never recomputed → the Play still fails closed on the same validator condition; validator/plan/`CameraState`/runner are **not in the delta** (byte-identical) |
| 3 | **Tests**: pins for the round-3 condition + series loop + the 5 residual needles; validator tolerance; **M33** (conditional re-assert restored) and **M34** (series removed) must FAIL | **PASS (with named residual)** | round-3 pins present (`local attempted=details.cameraTypeRestored~=true`, `for frame=1,5 do`, `table.insert(frameSeries,{frame=frame,type=`, `reassertFinalType=`, `frameSeries`); residual needles present: `typeWriteError`, `currentSubjectName`, `reassertOk=`, `reassertError=`, `afterReassertType=`; validator-tolerance payload extended with the round-3 fields (passes). **M33** reproduced: restore the old condition → atlas acceptance **FAILED (1)**; **M34** reproduced: delete the 5-frame loop → **FAILED (1)**; restored (cmp identical). Pin-removal re-check: `typeWriteError`/`currentSubjectName`/`reassertOk`/`reassertError` removals → **FAILED (1)** each; **`afterReassertType` table-entry removal → OK** (the needle also matches the assignment `afterReassertType=if frameSeries[1] …`); `typeWriteOk=` source needle absent (its removal is caught behaviourally by the Lune `atlas-camera-state` spec) |
| 4 | **Delta scope** over `618bb4a`: exactly 2 files +22/−10; no plan/validator/runner/traffic/fixture/`src`/build; sealed untouched | **PASS** | `git diff --numstat 618bb4a 0ec48d6` = `tests/atlas/Client.luau` +15/−7, `tests/tools/test_atlas_acceptance.py` +7/−3 (2 files, +22/−10); no `tools/atlas_acceptance.py`, `tools/map_atlas.py`, `tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`, `tests/traffic/`, `src/`, `assets/`, `build/`, `deploy.json`; sealed `84324d43…`/`c6651793…`/Studio place `c6651793…` unchanged |
| 5 | **Suites re-run** | **PASS** | atlas acceptance **25 OK** (0.7 s); runner **91 OK** (12.3 s); full Python **334 OK** (44.9 s); Lune atlas filter incl. atlas-paths **11 passed / 0 failed (4 specs)**; **full Lune 925 passed / 0 failed (127 specs)** (re-ran, ~12 min); sealed hashes unchanged after every run |

## 4 · Residual (named, non-blocking — fold into the pre-authorised one-line fix freeze)

1. **`afterReassertType` pin is ineffective for the recorded field**: the needle `afterReassertType=` also matches the
   assignment line (`afterReassertType=if frameSeries[1] …`), so deleting the diagnostics-table entry passes the suite.
   Fix: assert the assignment form `afterReassertType=afterReassertType,` (or add a Lune assertion).
2. **`typeWriteOk=` has no source-shape needle** (behaviourally covered by the Lune `atlas-camera-state` spec); add it
   with the same freeze for symmetry.

The instrumentation itself needs no change — the diagnostics the Play needs are recorded.

## Limits

- **No Play / no Studio input**: the re-assert result and the 5-frame series are read by Play 19, not by this check; this
  delta certifies the instrumentation, its pins and the unchanged verdict path.
- The re-assert is a conditional write that runs **after** the verdict; it cannot change the payload's `restored`/`cleanup`
  (both pre-re-assert) — stated explicitly so the Play's interpretation is unambiguous.

## Evidence (`board/probes/checker-round3/`)

`provenance-20261102.txt` (commit/tree/parent/fresh-copy/delta/scope/sealed), `verdict-path-20261102.txt` (checkAll sha,
verdict lines, details-before-re-assert, payload never references diagnostics), `mutations-20261102.txt` (M33/M34 +
pin-removal re-check), `suites-20261102.txt`, `lune-full-20261102.log` (925/0/127). Fresh copy `/tmp/checker-r3`; mutation
copy `/tmp/checker-r3-mut` (restored).

---

**— checker-trading (fresh-context, active checker seat), 2026-11-02 10:05 ET (EST, UTC−5) = 2026-11-02 15:05Z.**
**Verdict: ACCEPT** for `0ec48d6768224721dbd5987dfb058a2e2b6b4c52` — `roblox-lead` may run **Play 19** in the current
opening (pid 995852, 1/3 used, 2 reserved) to read the re-assert result + 5-frame series and name the reverter cadence;
the pre-authorised one-line verdict-recompute chain follows only if the series shows a one-shot reverter; the two pin
residuals can ride that freeze.
