# CHECKER — Play-19 one-shot fix: verdict recompute after the deferred re-assert (`e475d468`) — verdict: **ACCEPT**

Fresh-context independent check (same active checker seat; I built none of it). All execution from a **fresh copy of the
commit**, never the maker's tree. **No Play, no Studio input, no image reads, no build, $0.** Report to `lead` only.

- **Checked commit:** `e475d4688f3243932105cf4a0fef0561564e2811` (tree `a36ed97a517173aed279219425b45f659c1dbbb7`),
  parent `0ec48d6768224721dbd5987dfb058a2e2b6b4c52` (= last accepted delta); branch
  `prep/b1-capture-format-20261101` **tip**; **2 files, +29/−2**; tracked tree clean (835 pre-existing untracked
  evidence lines).
- **Fresh copy:** `/tmp/checker-fix19` from `git archive e475d468` — 1443 tracked entries, **blob-for-blob equal** (0
  missing / 0 mismatch); `build/` untracked → sealed artifacts copied read-only (`c6651793…`/`84324d43…`).
- Maker reports read: `board/reports/team-d-atlas-fix19-freeze-20261102.md` and Play-19
  `board/reports/team-d-diagnostic-play19.md`; their numbers/claims match my runs and the diff.

## Verdict in one line

**ACCEPT** — the fix is **narrow and honest**: after the deferred re-assert + 5-frame series, the verdict details are
replaced by a **fresh `State.check`** only when the attempt succeeded (`reassertOk==true`), all five series frames hold
`savedType`, and the fresh check's `cameraTypeRestored` is true; any other outcome keeps the **original failed details
→ still fails closed**. Payload (`restored=checkAll(details)` / `cleanup=details`), `checkAll`, validator, plan and runner
are untouched; the diagnostics gain only `verdictRecomputed`. M35 fails as required, the two residual pins are folded and
**effective**, and all suites reproduce. Named residual (non-blocking, §4): the fail-closed **guard conditions** are not
pinned by tests (removing them passes the suite), and a reversion between the fresh check and the payload is not covered
by the block — Play 20 is the runtime pass gate.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **Narrow, honest fix**: after the re-assert + 5-frame series, if the attempt succeeded, every frame holds the saved type and a fresh `State.check` passes → recompute the verdict details from that fresh check (menuGrade/effects/noDriver/bindings/monitor/focus recomputed) with `verdictRecomputed=true`; otherwise keep the original failed details; payload/validator/plan untouched; diagnostics gains only `verdictRecomputed`; no other behaviour beyond the restoration verdict | **PASS** | diff: `if attempted and reassertOk==true and #frameSeries==5` → `stable` loop over every `frameSeries` row vs `savedType` → `local finalCheck=State.check(...)` → `if finalCheck.cameraTypeRestored then` → recompute the 6 extra fields → `details=finalCheck; verdictRecomputed=true`; `diagnostics.verdictRecomputed` is the only diagnostics change; payload lines `restored=checkAll(details)` / `cleanup=details` and the `checkAll` block are **byte-identical** between commits; validator/plan/runner are **not in the delta**; `State.check` supplies the same 8 fields, so the recomputed details have the same payload shape |
| 2 | **No false-positive path**: a case where the re-assert fails or any frame reverts keeps `restored:false`; the recompute cannot mark a genuinely unrestored camera as restored | **PASS (by code inspection)** | fail-closed conditions: `reassertOk` must be exactly `true`; `#frameSeries==5`; **every** series frame `.type==savedType`; and the fresh `State.check.cameraTypeRestored` (same camera AND `CameraType==saved.type` at that instant) must be true. Any failure → `details` stays the original failed check → `restored=checkAll(details)` false → validator fails. If `attempted` is false (first check passed), no recompute. Residual limit: a reversion **after** the fresh check and before the finish payload is not covered — Play-19 evidence (one-shot reverter, 5/5 `Custom`) is the basis; Play 20 is the pass gate |
| 3 | **Tests**: pins for the new path + residual pins folded (`afterReassertType=` entry, `typeWriteOk=`); **M35** (recompute reverted) must FAIL | **PASS (with named residual)** | new pins: `local finalCheck=State.check(`, `details=finalCheck`, `if attempted and reassertOk==true`, `verdictRecomputed`, `afterReassertType=afterReassertType,` (recorded table entry), `typeWriteOk=` (State). **M35 reproduced**: whole recompute block removed → atlas acceptance **FAILED (1)**; restored (`cmp` identical). Residual pins **effective**: remove the `afterReassertType` table entry → **FAILED (1)**; remove the `typeWriteOk` recording → **FAILED (1)**. Guard-pin probe: remove `if finalCheck.cameraTypeRestored then` or the frame-stability loop → atlas acceptance **OK** (guards not pinned; current code correct) |
| 4 | **Delta scope** over `0ec48d6`: exactly 2 files +29/−2; no plan/validator/runner/traffic/fixture/`src`/build; sealed untouched | **PASS** | `git diff --numstat 0ec48d6 e475d468` = `tests/atlas/Client.luau` +24/−0, `tests/tools/test_atlas_acceptance.py` +5/−2 (2 files, +29/−2); no `tools/atlas_acceptance.py`, `tools/map_atlas.py`, `tools/vinegar_run.py`, `tests/tools/test_vinegar_run.py`, `tests/traffic/`, `src/`, `assets/`, `build/`, `deploy.json`; sealed `84324d43…`/`c6651793…`/Studio place `c6651793…` unchanged |
| 5 | **Suites re-run** | **PASS** | atlas acceptance **25 OK** (0.7 s); runner **91 OK** (12.1 s); full Python **334 OK** (47.3 s); Lune atlas filter incl. atlas-paths **11 passed / 0 failed (4 specs)**; **full Lune 925 passed / 0 failed (127 specs)** (re-ran, ~12 min); sealed hashes unchanged after every run |

## 4 · Residuals (named, non-blocking)

1. **Fail-closed guards not pinned:** removing `if finalCheck.cameraTypeRestored then` or the frame-stability loop passes
   the offline suite (verified). The current code is correct by inspection; add two source-shape needles (or a Lune
   harness case) so a future edit cannot silently open a false-positive path. Fold into the next freeze.
2. **Post-check reversion window:** the recompute verifies at the fresh `State.check` instant; a reversion between that
   check and the finish payload is not covered by the block. Play-19's one-shot evidence is the basis; **Play 20 is the
   runtime pass gate** and its `frameSeries`/`cameraTypeRestored` are the decisive evidence.
3. Nothing else: payload/`checkAll`/validator/plan/runner byte-unchanged; diagnostics gain only `verdictRecomputed`.

## Limits

- **No Play / no Studio input**: the fix's real-runtime behaviour is exercised by Play 20 only; this check certifies the
  code, its guards (inspection), the pins and the unchanged payload path.
- The fix does **not** alter the ruling/rails or the restore contract; it only recomputes the restoration verdict after a
  verified one-shot re-assert.

## Evidence (`board/probes/checker-fix19/`)

`provenance-20261102.txt` (commit/tree/parent/fresh-copy/delta/scope/sealed), `fix-proof-20261102.txt` (recompute block,
fail-closed conditions, narrowness), `mutations-20261102.txt` (M35, guard-pin probes, residual pins),
`suites-20261102.txt`, `lune-full-20261102.log` (925/0/127). Fresh copy `/tmp/checker-fix19`; mutation copy
`/tmp/checker-fix19-mut` (restored).

---

**— checker-trading (fresh-context, active checker seat), 2026-11-02 10:35 ET (EST, UTC−5) = 2026-11-02 15:35Z.**
**Verdict: ACCEPT** for `e475d4688f3243932105cf4a0fef0561564e2811` — **Play 20 (the atlas pass gate) is authorised from
the checker side** in the current opening's last reserved Play: it must show `cameraTypeRestored:true` with every other
field true; a fail or any reverted frame keeps the day closed (fail-closed, no claim).
