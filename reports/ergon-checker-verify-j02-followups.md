# Ergon checker — J2-F1/F2/F3 follow-up verification: **PASS**

**Checker:** `ergon-checker` (independent; built none of this work). **Reports to:** `lead` (copy to `ergon-lead`).
**Package:** repo `cayleb-james2008/ergon`, branch **`duet/j02-followups`**, single commit on merged main `c196c25`, **final SHA `a61fdd59601b372675cda927373e407d400c7886`** (pushed; remote tip = this SHA).
**Gate run:** 2026-10-31 22:11–22:14 ET; my own gate seat `/tmp/ergon-gate-462277c`, checked out at `a61fdd5`; `package-lock.json` md5 unchanged (`094f3d4e…`); clean tree; gate environment per my ruling (`OPENCODE_BIN` unset, no `opencode` on `PATH`).
**Writes:** this report + `board/probes/ergon-checker-gate-job02-runs.txt` §12. No repo/branch write; no merge.

## Verdict

**PASS on all three findings.** J2-F1/F2/F3 are real, landed, and nothing else changed: the commit is exactly the ordered scope (8 files: `docs/EXIT-CODES.md`, `fixtures/batch/run-batch.mjs`, six regenerated batch-evidence files; J2-F3 is the board handoff outside the repo). The documented-env suite on `a61fdd5` is green. **Merge recommendation: YES** — the Lead may authorise merging `a61fdd5` into `main`; I run the merged-tree proof on the new main SHA afterwards.

## 1. Scope and preconditions

- Remote tip `a61fdd59601b372675cda927373e407d400c7886` = maker final SHA; parent = merged main `c196c25`; `git merge-base --is-ancestor c196c25 HEAD` → **0**; 1 commit; clean tree; lock md5 unchanged.
- Diff `c196c25..a61fdd5` = **8 files, exactly the ordered scope**: `docs/EXIT-CODES.md` (2 lines), `docs/acceptance/demo-company/fixtures/batch/run-batch.mjs` (+6), six `evidence/batch-standin/` files regenerated. J2-F3 is `board/reports/team-a-job02-checker-handoff.md` (board report, outside the repo).

## 2. J2-F1 — stale check.mjs paragraph in EXIT-CODES.md: FIXED

`docs/EXIT-CODES.md` "Usage errors, run for real" now reads: *"`scripts/check.mjs` was retired in Pack 21 (see "Retired entry points" below), so it is no longer run here: its private-copy usage rows and behavioural tests were removed with it. The remaining private-copy runs are the ones named above."* The stale "it is also run from a private copy…" description is gone. The remaining mentions are the correct retirement note and the historical Pack 17 note.
**Test:** `node --test tests/exit-codes.test.mjs` → `# tests 45 # pass 45 # fail 0` (maker's 45/45 claim confirmed).

## 3. J2-F2 — batch ledger candidate event: FIXED and behaviourally verified

- `fixtures/batch/run-batch.mjs` now pushes one `template_candidate_proposed` ledger event per candidate (id, class, title, reason, evidence, promotion) alongside the `finding_recorded` events.
- Committed `evidence/batch-standin/batch-ledger.jsonl`: **`{finding_recorded: 72, template_candidate_proposed: 1}`**; the candidate event is **TC-1** with promotion *"proposal only — needs the review gate and a regression batch; not applied by this run"*.
- **Regenerated evidence matches the current driver:** `generatedAt 2026-11-01T02:09:01.684Z` (commit at 22:09:22 ET); numbers unchanged from the gated evidence — 8 companies / 1,768 state-advanced acts / 56 simulated days / 0 live acts / 72 findings (all stand-in-limit) / 1 candidate; honesty block unchanged: live `{acts:0,seconds:0}`, state-advanced note **"state-advanced time is NOT live operation"**, `everyActLabelled:true`, `stateAdvancedReportedAsLive:false`.
- **Independent reproduction with the new driver** (1-company batch, 22:12:24–22:13:43 ET, exit 0): ledger `{finding_recorded: 6, template_candidate_proposed: 1}`, candidate TC-1 present, honesty block correct — the event is emitted by the driver, not just present in the committed file.

## 4. J2-F3 — handoff SHA: FIXED

`board/reports/team-a-job02-checker-handoff.md` line 12 now pins **`4387b51`** (with a note that the earlier `90ddcad` line was stale and the delta was docs/evidence-only), and line 75 records it merged as `c196c25` with the merged-tree proof PASS. No stale `90ddcad` reference remains.

## 5. Documented-env suite on `a61fdd5` (my run, 22:11:51–22:13:09 ET)

```
TOTAL 1616 cases: 1607 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 158/158 checks ran, 0 without a complete report;
4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs
Windows); 0 manifest problems
AGGREGATE_OUTCOME {"outcome":"completed","testExecution":"reported-complete","planned":158,"executed":158,
"cases":{"tests":1616,"pass":1607,"fail":0,"cancelled":0,"skipped":9,"todo":0},"uncertainChecks":[],...}
EXIT=0        (grep '^not ok' -> 0 lines)
```
Matches the maker/Lead claim exactly. Non-passing: 9 skips with named reasons (5 environment + 4 opencode-e2e binary cases; 4 all-skipped manifest-allowed) + 6 .NET/Windows seat invocations, never counted. No failures, no incomplete checks.

## 6. Next step

**Merge recommendation: YES** (single commit `a61fdd5`, merge commit, no force). After the Lead-authorised merge I run the merged-tree proof on the new main SHA: remote tip, `a61fdd5` ancestor of main, merged-tree suite `0 fail / 0 incomplete / exit 0`, and `comm -23` failing-case diff = 0.

*Written by `ergon-checker`, 2026-10-31 ~22:16 ET.*
