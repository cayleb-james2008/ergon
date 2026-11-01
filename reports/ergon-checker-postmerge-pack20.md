# Ergon checker — Pack 20 post-merge proof: **PASS**

**Checker:** `ergon-checker` (independent; built none of this work). **Reports to:** `lead`.
**Merge under proof:** `main` @ **`1c7189866b41755b92bc57c8b85d91649b529bcd`** — merge commit "Merge duet/p20-demo-company (Pack 20, checker PASS 31 Oct) into main", authorised by the Lead after my PASS on `462277c`.
**Seat:** my own fresh gate clone `/tmp/ergon-gate-462277c`, fetched 2026-10-31T20:08:24 ET and checked out at `main` @ `1c71898`; clean tree; `package-lock.json` md5 unchanged (`094f3d4e…`), so the provisioned dependency tree and verified document runtime remain the gate's. Gate environment unchanged (`OPENCODE_BIN` unset, no `opencode` on `PATH`).
**Writes:** this report only. No repo/branch write, no revert, no re-merge.

## Verdict

**PASS.** The merge is exactly the checker-passed tree, the remote main tip is confirmed, both required ancestors are in `main`, the merged tree is green in the gate environment (0 fail / 0 incomplete / exit 0), and the merged-tree failing-case diff is **0**. There is no failing leg and no branch-attributable cause to name.

## Proof legs (all run by me, 2026-10-31 20:08–20:11 ET)

| # | Leg | Command / result |
|---|---|---|
| 1 | Remote `main` tip | `git ls-remote … refs/heads/main` → `1c7189866b41755b92bc57c8b85d91649b529bcd` (re-checked after the suite run: unchanged) |
| 2 | Merge commit identity | `git cat-file -p origin/main` → tree `f2b08d8ab16ee225fce996928267c0b0c4e23c5e`; parents **`b59b5990c594bfe7dfda859fcac3bff09ef609d3`** and **`462277ca46445273ed346f025fbaf5fddb0ba068`**; message "Merge duet/p20-demo-company (Pack 20, checker PASS 31 Oct) into main" |
| 3 | Required ancestry | `git merge-base --is-ancestor 462277c origin/main` → **0**; `… b59b599 origin/main` → **0** |
| 4 | Tree identity | merge tree = `462277c` tree = `f2b08d8a…`; `git diff --stat 462277c HEAD` → **0 files**; `git diff --stat b59b599 HEAD` = the P20 additions only (73 files, +14,249, 0 deletions) |
| 5 | No force consistency | old main tip `b59b599` is an ancestor of the new tip (history preserved). Server-side reflog is not auditable from the remote; this is the verifiable part of the maker's "no force" claim. |
| 6 | Merged-tree full suite (gate env) | checkout `main` @ `1c71898`, run 20:09:29–20:11:02 ET: `TOTAL 1609 cases: 1600 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 157/157 checks ran, 0 without a complete report; 4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs Windows); 0 manifest problems` — **EXIT=0**; `AGGREGATE_OUTCOME … "uncertainChecks":[]`; 0 `not ok` lines |
| 7 | Merged-tree failing-case diff | `comm -23` merged-tree `not ok` vs checker-passed final-tree `not ok` → **0 lines**; vs old-main same-env baseline → **0 lines** |

Supporting checks on the merged tree: all five Pack 19 merge commits (`d0c81f8`, `761dde5`, `23f3c89`, `4e9a8e0`, `7990341`) are still ancestors of `main`; working tree clean at `1c71898` before and after the suite.

## Job 02 precondition

**Satisfied:** `main` @ `1c71898` contains both Pack 19 (five merge commits + integration, all ancestors) and Pack 20 (merge commit `1c71898`, second parent `462277c`), so Job 02 may build on merged `main` without waiting. **What Job 02 then needs from the checker seat:** the same gate applied to its Pack 21 + scale-up branch — a fresh-context checker PASS **before** merge, with (a) each ruled item's own before/after test evidence (E-2, E-6, E-1, S-2/S-4 riders, F-2/F-4/F-5 include-or-defer reasons), (b) full suite on the branch in the documented environment (0 fail / 0 incomplete), (c) the scale-up batch run with live-driven vs state-advanced separation, (d) merged-tree proof with **0 branch-attributable failures**; plus verification that the accepted docs findings **F1–F4** land in the next Ergon commit. The gate harness (scope checker + documented-env suite + `comm -23` branch-attributable diff) is already on disk and reusable.

## Reproducibility

Raw extracts appended to `board/probes/ergon-checker-gate-pack20-runs.txt` (§10); full suite log was `/tmp/ergon-checker/logs/merged-main-suite.log` (disposable). Seat and commands are quoted above.

*Written by `ergon-checker`, 2026-10-31 ~20:12 ET.*
