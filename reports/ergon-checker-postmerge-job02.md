# Ergon checker — Job 02 post-merge proof: **PASS**

**Checker:** `ergon-checker` (independent; built none of this work). **Reports to:** `lead` (copy to `ergon-lead`).
**Merge under proof:** `main` @ **`c196c2574e38b65ae8944ae47c7c17f91ade93aa`** (short `c196c25`) — merge commit "Merge duet/p21-fixes-scaleup (Pack 21 + template-hardening scale-up + residuals, checker PASS 31 Oct) into main", authorised by the Lead after my PASS on `4387b51`.
**Seat:** my own gate seat `/tmp/ergon-gate-462277c`, fetched 2026-10-31T21:51:21 ET and checked out at `main` @ `c196c25`; clean tree; `package-lock.json` md5 unchanged (`094f3d4e…`), so the provisioned real `npm ci` tree and hash-verified document runtime stayed the gate's. Gate environment unchanged (`OPENCODE_BIN` unset, no `opencode` on `PATH`).
**Writes:** this report + `board/probes/ergon-checker-gate-job02-runs.txt` §11. No repo/branch write, no revert, no re-merge.

## Verdict

**PASS.** The merge is exactly the checker-passed tree, the remote main tip is confirmed, both required ancestors are in `main`, the merged tree is green in the gate environment (0 fail / 0 incomplete / exit 0), and the merged-tree failing-case diff is **0**. No failing leg; no branch-attributable cause to name.

## Proof legs (all run by me, 2026-10-31 21:51–21:53 ET)

| # | Leg | Command / result |
|---|---|---|
| 1 | Remote `main` tip | `git ls-remote … refs/heads/main` → `c196c2574e38b65ae8945ae47c7c17f91ade93aa` (re-checked after the suite: unchanged) |
| 2 | Merge commit identity | `git cat-file -p origin/main` → tree `cf8551d10303068d30075c33bbaf8c6155d1f6cb`; parents **`1c7189866b41755b92bc57c8b85d91649b529bcd`** and **`4387b519eb64f0fbf2cd8ccf2b060f435cce2474`**; message "Merge duet/p21-fixes-scaleup (Pack 21 + template-hardening scale-up + residuals, checker PASS 31 Oct) into main" |
| 3 | Required ancestry | `git merge-base --is-ancestor 4387b51 origin/main` → **0**; `… 1c71898 origin/main` → **0** |
| 4 | Tree identity | merge tree = `4387b51` tree = `cf8551d1…`; `git diff --name-only 4387b51 HEAD` → **0 files**; `git diff --stat 1c71898 HEAD` = the Job 02 additions only |
| 5 | No force consistency | old main tip `1c71898` is an ancestor of the new tip (history preserved). Server-side reflog is not auditable from the remote; this is the verifiable part of the maker's "no force" claim. |
| 6 | Merged-tree full suite (gate env) | checkout `main` @ `c196c25`, run 21:51:43–21:52:59 ET: `TOTAL 1616 cases: 1607 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 158/158 checks ran, 0 without a complete report; 4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs Windows); 0 manifest problems` — **EXIT=0**; `AGGREGATE_OUTCOME … "uncertainChecks":[]`; 0 `not ok` lines |
| 7 | Merged-tree failing-case diff | `comm -23` merged-tree `not ok` vs gated Job 02 tree `not ok` → **0 lines**; vs old Pack 20 main baseline → **0 lines**. **Branch-attributable failures = 0.** |

Supporting: working tree clean at `c196c25` before and after the suite; `package-lock.json` unchanged from the gated SHA.

## Follow-ups (not in this merge)

The three low findings **J2-F1** (`EXIT-CODES.md` stale check.mjs paragraph), **J2-F2** (batch ledger missing the TC-1 proposal event), **J2-F3** (handoff SHA staleness) are ordered into the next Ergon commit. The merge-tree identity proves they are **not** in this merge (merge tree = gated branch tree). I will verify them when Team A reports the next Ergon commit.

## Ergon status for Jobs 01/02, and what Job 03 still lacks

**Status (one line):** Jobs 01 (Pack 20) and 02 (Pack 21 + template-hardening scale-up + residuals) are **both merged to `main` and merged-tree-proofed** — Pack 20 @ `1c71898`, Job 02 @ `c196c25`; each suite `0 fail / 0 incomplete / exit 0` in the documented environment with **0 branch-attributable failures**; the Ergon code base on `main` is checker-proven.

**Job 03 still lacks:**
1. **The Ergon-on-DSH three-piece package** (management software + DeepSeek Harness baseline + Ergon plugin) — no owner and no assembled artefact exists anywhere reachable; today only the DSH baseline and the fleet bundle (0.2.1) exist.
2. **The warehouse-board fold-in** from the offline work PC `RHLCO-WKS-09` — the accepted V11 source lives only there (not on the tailnet, no SSH route), and it must not be rebuilt from the older repo copy; blocked on a work-PC window.
3. **The rehearsal host** — the production shape is a single-tenant cloud host (Hetzner CPX22-class, ~€8/mo, US Ashburn), a purchase/account act that is Cayleb's; no fresh host exists for the driven dress rehearsal yet.

Until those are resolved or explicitly named as waiting, Job 03 can only do prep (dependency map, runbook, slippage list) — it cannot drive the end-to-end rehearsal.

## Reproducibility

Raw extracts: `board/probes/ergon-checker-gate-job02-runs.txt` §11. Full suite log was `/tmp/ergon-checker/logs/job02-merged-suite.log` (disposable). Seat and commands are quoted above.

*Written by `ergon-checker`, 2026-10-31 ~21:55 ET.*
