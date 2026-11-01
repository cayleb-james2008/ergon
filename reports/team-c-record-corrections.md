# TEAM C — RECORD CORRECTIONS (checker-flagged) + the `model.py` incident

trading-lead, 2026-10-31 19:5x ET. Shadow-side only. This file records (1) the `# tamper` incident on
`live/kalshi/model.py`, (2) the four record corrections the trading-checker flagged, and (3) the clean-tree evidence.

## 1. `live/kalshi/model.py` — the `# tamper` line (out-of-band edit; RCA and resolution)

**What happened.** At 19:15 ET I ran a *manual structural dry-run probe* as a one-off shell command. It built a scratch root
under `/tmp` and populated it with **symlinks** to the real pinned files, then appended `# tamper` to
`$TMP/live/kalshi/model.py` to prove `freeze.py verify` catches drift. Because that path was a symlink, the append went
through into the real `live/kalshi/model.py`, leaving one stray line. **It was my probe, not the shipped test and not
freeze.py** — the checker's read is correct: `test_freeze.py` mutates a temp-root **copy**, and `freeze.py` only hashes
PINNED files.

**Why it matters.** `model.py` is a freeze pin. A freeze built on that tree would have anchored the forward count to a
tampered hash (and the running shadow service imports the file). The checker would have failed it first — it did.

**Resolution (done, verified):**
```
$ git -C ~/Work/projects/openbell-kalshi checkout -- live/kalshi/model.py
$ git status --porcelain live/kalshi/model.py          # (empty — clean)
$ sha256sum live/kalshi/model.py
7f45e9330d0278be6499e1acaf6e2abfeabedd8c664f36654c61ebd732d60804  live/kalshi/model.py
$ python3 -c "...chain_head['code_files']['model.py']"
7f45e9330d0278be6499e1acaf6e2abfeabedd8c664f36654c61ebd732d60804   -> MATCH
```
All six `chain_head.code_files` pins now MATCH the real files (asof, decide, fills, ledger, model, shadow).

**Fixes so it cannot recur:**
- The shipped test is hardened (`test_freeze.py`): the tamper target is asserted **not to be a symlink**, the original bytes
  are restored in a `finally`, and it runs only inside a temp root (15 tests still OK).
- The manual probe is replaced by a guarded script: `board/probes/team-c/structural-freeze-dryrun.sh` — it **copies** files
  (never links), snapshots `git status --porcelain` + the pinned hashes before and after, exits 8 if the real tree changed,
  and removes the scratch root on exit. `freeze.py` itself was **not** changed.
- Probe run 2026-10-31 19:55Z (log `board/probes/team-c/structural-freeze-dryrun-20261031T1955Z.log`):
  build dry-run (12 pins) -> promote in scratch -> `freeze record clean: every pin matches and the ledger continues the frozen
  heads` (exit 0) -> tamper a copy -> `PROBLEM: pin drift: live/kalshi/model.py` (exit 5) -> `REAL_TREE_UNCHANGED`.
- Clean-tree status after the probe: `git status --porcelain live/kalshi/model.py` empty; sha256 = the chain_head pin.

**Current full worktree status (named, so nothing is unexplained):**
```
 M data/kalshi15/manifest.json            <- pre-existing (before tonight's session; the print pull's manifest)
 M research/kalshi15/DECISIONS.md         <- tonight: L* = 30 s Amendment 2 (authorised by the Lead on Nox's ruling)
 M research/kalshi15/RESUME.md            <- tonight: collector two-part note + freeze/forward tooling note
 M research/kalshi15/SPEC.md              <- tonight: Appendix T Amendment 4 (L* = 30 s)
?? data/forward/kalshi/                   <- pre-existing (the collector's forward data)
?? research/kalshi15/freeze.py            <- tonight: C16 tool (new)
?? research/kalshi15/forward.py           <- tonight: C17 tool (new)
?? research/kalshi15/test_freeze.py       <- tonight: C16 tests (new)
?? research/kalshi15/test_forward.py      <- tonight: C17 tests (new)
?? research/kalshi15/results/first_scorer.{late,rows,summary}.*  <- pre-existing (13:32 ET, before the session)
```
No `live/` file is modified; no generated result was edited.

## 2. The four record corrections (trading-checker audit)
| # | Correction | Where it was wrong | Applied |
|---|---|---|---|
| 1 | End-to-end latency is **≈18–19 s**, not ≈19–20 s; the L\* = 30 s margin over the data-time p90 is **0.29 s** (27.713 + 2.0 = 29.713 ≤ 30) | `DECISIONS.md` Amendment 2, `SPEC.md` Amendment 4, `board/reports/team-c-kalshi-lstar-text.md` | **Yes** — all three corrected with a dated correction note; the verbatim Nox quote (which says ~19–20 s) is left as the quote it is |
| 2 | Scorer outputs are identical **within tol 1e-9** (integers exact), not "byte-identical" | no doc said otherwise — the JSON itself says "identical (tol 1e-9, integers exact)"; "byte-identical" belongs only to the WP10 dry-run artefact rerun (six files, equal sha256) | **Recorded** in the L\* text file §6 so the two claims are never mixed |
| 3 | The WP10 dry-run `end_to_end_s` p50 **64.5 s is a construction** of the batch rehearsal (row 60), not a measurement | no doc asserted it as measured; risk of being quoted interchangeably with the availability pair | **Recorded** in the L\* text file §6 |
| 4 | `live/kalshi/model.py` dirty tree (this file, §1) | worktree | **Resolved**, verified, guarded |

## 3. Freeze pre-check slot (for the checker; nothing executed)
The real freeze stays unexecuted. When the checker gets its pre-freeze slot it should verify, in this order:
1. `git status --porcelain live/kalshi/` -> empty (no `live/` file modified).
2. The six `chain_head.code_files` hashes equal `sha256sum` of the real files (script: `structural-freeze-dryrun.sh` §1).
3. `freeze.py build --at <Nox's instant> --dry-run` on the real tree -> **must refuse** with the current unfilled prereg slot
   count (7) until the prereg is filled; when the prereg is filled, the dry-run must succeed with 12 pins.
4. `freeze.py verify` on the built record (in a scratch copy) -> clean; a tamper -> drift.
5. `readiness.py` on the day -> 9 PASS / 1 NA / 0 FAIL, quoted.
Nothing here is a substitute for the checker's own run.
