# CHECKER — extended camera-restore diagnostics delta (`618bb4a`) — verdict: **ACCEPT**

Fresh-context independent check (same active checker seat; I built none of it). All execution from a **fresh copy of the
commit**, never the maker's tree. **No Play, no Studio input, no image reads, no build, $0.** Report to `lead` only.

- **Checked commit:** `618bb4a4473bc5d46eb15a4611cb1fa55f53cdbb` (tree `59f953257ee82b753f4bc93fd2a86e8ee54b337d`),
  parent `7c7c1e777507001d88e91b8342cc6cc57128704e` (= last accepted delta); branch
  `prep/b1-capture-format-20261101` **tip**; **4 files, +49/−6**; tracked tree clean (831 pre-existing untracked
  evidence lines).
- **Fresh copy:** `/tmp/checker-diag2` from `git archive 618bb4a` — 1443 tracked entries, **blob-for-blob equal** (0
  missing / 0 mismatch); `build/` untracked → sealed artifacts copied read-only (`c6651793…`/`84324d43…`).
- Maker report: the brief's expected `board/reports/team-d-atlas-diag2-freeze-20261102.md` does **not** exist; the actual
  report is `board/reports/team-d-atlas-layer10-freeze-20261102.md` (read; consistent), log
  `board/probes/team-d/atlas-layer10-freeze-20261102.log`.

## Verdict in one line

**ACCEPT** — the delta is **instrumentation-only**: the write site records the saved/current camera subjects, a pcall
success flag + immediate read-back, and one deferred re-assert; the **verdict path is byte-unchanged** (`checkAll` block
sha identical, `restored=checkAll(details)` and `cleanup=details` unchanged, `State.check`/`capture`/`present` unchanged,
validator/plan/runner not in the delta), the re-assert is recorded and never used to recompute the verdict, M31/M32 and
the pinned-field mutation fail as required, and all suites reproduce. Residual (non-blocking, §3): **four secondary new
fields have no source-shape pin / Lune assertion** — fold the needles into the pre-authorised next freeze.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **Instrumentation-only, no behaviour change** — write site records `subjectName`/`subjectClass` (saved vs current), pcall flag + immediate read-back, one deferred re-assert; verdict/payload/validator/plan/State-check/runner byte-unchanged; new fields must not feed the verdict | **PASS (with one literal nuance)** | `checkAll` exact block sha **`ad361c16af283f97` → identical**; `restored=checkAll(details)`, `cleanup=details` unchanged; `State.check` `78bbf70c…` unchanged, `State.capture/present/sameValue` unchanged; validator/plan/runner **not in the delta** (byte-identical); `details=State.check(...)` is computed **before** diagnostics/re-assert, payload `restored`/`cleanup` are pre-re-assert; `cameraTypeDiagnostics` ignored by the validator (tolerance test). Literal nuance: the write site lives in `tests/atlas/CameraState.luau` (`State.restore`), so that file **does** change (pcall + read-back + `return State.check(...), diagnostics`; the first return value is the same `State.check(...)`); `State.check`/verdict path is unchanged |
| 2 | **No fix smuggled in** — deferred re-assert diagnostics-only (recorded, not used to alter the restoration verdict); no behaviour/repair logic change to the verdict | **PASS** | re-assert is conditional (`typeAfterWrite ~= savedType`), wrapped in `pcall`, recorded (`reassertAttempted/Ok/Error/afterReassertType`) and runs **after** `details`/`recheck`; the payload's `restored=checkAll(details)` and `cleanup=details` are computed before it and are never recomputed → the Play still fails closed on the same validator condition; no acceptance path was changed. Only error-path delta: the CameraType write is now pcall-wrapped (a write error is recorded instead of propagating) — this is the Lead-authorised `pcall` instrumentation; the re-assert can change the post-verdict camera state but cannot reach the verdict |
| 3 | **Tests** — source-shape pins for the new fields (incl. the previous delta's 3 secondary needles), payload keys; validator tolerance; a mutation removing one new field/payload key must FAIL | **PASS (with named residual)** | previous 3 needles now pinned (`cameraTypeRestored=`, `sameCamera=`, `frameSameCamera=`) ✔; new pins: `reassertAttempted=`, `afterReassertType=`, `writeDiagnostics`, and in `CameraState`: `local writeOk,writeErr=pcall`, `typeAfterFullRestore`, `savedSubjectName`, `savedSubjectClass`, `currentSubjectClass`, `typeAfterWrite`; Lune layer-10 case asserts `savedType`/`typeWriteOk`/`typeAfterWrite`/`savedSubjectName`/`savedSubjectClass`/`details.cameraTypeRestored`; validator-tolerance payload extended (passes). Mutation results: remove pinned `reassertAttempted` → **FAILED (1)**; **M31** (write-site pcall diagnostics removed) → **FAILED (1)**; **M32** (whole deferred re-assert feature removed) → **FAILED (1)**; all restored. **Residual:** 5 new recorded fields are not actually pinned — `typeWriteError`, `currentSubjectName`, `reassertOk`, `reassertError`, and the `afterReassertType` **table entry** (the needle `afterReassertType=` also matches the local declaration `local reassertOk,reassertErr,afterReassertType=nil,…`, so removing the recorded key leaves the suite green); removing any of these (valid code) passes atlas acceptance (verified; `typeWriteOk` removal IS caught by the Lune spec, the other four are not) |
| 4 | **Delta scope** over `7c7c1e7`: every changed file named; no plan/validator/runner/traffic/fixture/`src`/build change; sealed untouched | **PASS** | `git diff --numstat 7c7c1e7 618bb4a` = exactly `tests/atlas/CameraState.luau` +13/−2, `tests/atlas/Client.luau` +14/−2, `tests/tools/test_atlas_acceptance.py` +10/−2, `tests/unit/atlas-camera-state.spec.luau` +12/−0 (4 files, +49/−6); no `tools/atlas_acceptance.py`, `tools/map_atlas.py`, `tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`, `tests/traffic/`, `src/`, `assets/`, `build/`, `deploy.json`; sealed `84324d43…`/`c6651793…`/Studio place `c6651793…` unchanged |
| 5 | **Suites re-run** | **PASS** | atlas acceptance **25 OK** (0.7 s); runner **91 OK** (12.1 s); full Python **334 OK** (45.1 s); Lune atlas filter incl. atlas-paths **11 passed / 0 failed (4 specs)**; **full Lune 925 passed / 0 failed (127 specs)** (re-ran, ~12 min); sealed hashes unchanged after every run |

## Residuals (named, non-blocking)

1. **Pin coverage (fold into the pre-authorised next freeze):** add source-shape needles `typeWriteOk=`, `typeWriteError=`,
   `currentSubjectName=`, `reassertOk=`, `reassertError=` and make the `afterReassertType=` pin unambiguous (it currently
   also matches the local declaration), and extend the Lune case (`typeWriteError==nil`, `currentSubjectName`) so that
   removing any new field fails a suite. The instrumentation itself needs no change.
2. **`State` wording:** the brief says "State byte-unchanged"; `CameraState.luau` necessarily changes because the write
   site is `State.restore`. The verdict function `State.check` and all other State blocks are byte-unchanged, and the
   first return value of `State.restore` is the same `State.check(...)` — recorded so the literal wording is not read as
   a violation.
3. **Maker report filename:** actual `team-d-atlas-layer10-freeze-20261102.md`, not the brief's `…diag2…` — content
   present and consistent; non-material.

## Limits

- **No Play / no Studio input**: the diagnostics' *content* (the camera-type restore cause) is read by the now-authorised
  atlas Play, not by this check; this delta certifies the instrumentation, its pins and the unchanged verdict path.
- The re-assert is a conditional write that runs after the verdict; it cannot change the payload's `restored`/`cleanup`,
  but it can change the post-verdict camera state — stated explicitly so the Play's interpretation is unambiguous.

## Evidence (`board/probes/checker-diag2/`)

`provenance-20261102.txt` (commit/tree/parent/fresh-copy/delta/scope/sealed), `no-behaviour-change-20261102.txt`
(checkAll/State block shas, verdict lines, new-field/verdict separation), `mutations-20261102.txt` (pinned-field mutation,
M31/M32, the four unpinned-field gaps), `suites-20261102.txt`, `lune-full-20261102.log` (925/0/127). Fresh copy
`/tmp/checker-diag2`; mutation copy `/tmp/checker-diag2-mut` (restored).

---

**— checker-trading (fresh-context, active checker seat), 2026-11-02 04:40 ET (EST, UTC−5) = 2026-11-02 09:40Z.**
**Verdict: ACCEPT** for `618bb4a4473bc5d46eb15a4611cb1fa55f53cdbb` — one atlas Play in a fresh opening (Studio pid
995852, 0/3) is authorised from the checker side to read the extended diagnostics and name the cause; the pin residual
can ride the pre-authorised one-line-fix freeze.
