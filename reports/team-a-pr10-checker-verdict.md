# PR #10 — independent checker verdict (Team A, Job 03)

**Checker:** fresh-context, independent. I built, wrote and staged nothing under review.
**When:** 2026-11-01 09:05–09:30 ET (America/New_York). All timestamps below are ET.
**Repository:** https://github.com/cayleb-james2008/ergon
**PR:** #10 — **OPEN**, `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`, `mergedAt: null`.
**Refs:** base `main` @ `dffb44efa8c67d2bbb37eafc1af97c525fb5b9db`; head `docs/l5-acceptance-r0-r7` @ `86e874ac5cc01e5ab5fca73d5d19f9c48487af01`.
**Method:** read-only `git`/`gh`; independent re-runs on a `git archive` of `86e874ac` in `/tmp` (no branch writes). I made no commit, no push, no merge, and wrote only this verdict file in the repo.
**Overall verdict: PASS — safe to merge as docs-only.** All four checks pass. Every substantive claim, all six full artefact hashes, all four new-file hashes and all 22 truncated evidence hashes verify. The rehearsal is unrun, C1–C6 are PENDING, and `main` is untouched. One minor, non-blocking documentation defect (F1, below) is a false cross-reference; it misstates where a hash table is repeated, not the hashes or the branch state.

| # | Check | Verdict |
|---|---|---|
| 1 | Branch/PR facts | **PASS** |
| 2 | File contents vs the report | **PASS** (F1 minor, non-blocking) |
| 3 | Maker run evidence | **PASS** (all re-run; no item left VERIFIED-BY-ARTEFACT) |
| 4 | No run claim / no unruled contract applied / no main touch | **PASS** |

---

## 1. Branch and PR facts — PASS

Worktree inspected read-only: `/home/cayleb/workspace/harness-build/a-l5-seat` (clean at `86e874ac`).

| Check | Command (2026-11-01) | Result |
|---|---|---|
| Head commit | `git rev-parse HEAD` (09:26:13) | `86e874ac5cc01e5ab5fca73d5d19f9c48487af01` |
| Base commit | `git rev-parse HEAD^`; `git merge-base dffb44ef 86e874ac` (09:26) | `dffb44efa8c67d2bbb37eafc1af97c525fb5b9db` |
| Base is origin/main | `git ls-remote origin main` (09:29:08) | `dffb44efa8c67d2bbb37eafc1af97c525fb5b9db refs/heads/main` |
| Remote branch tip | `git ls-remote origin refs/heads/docs/l5-acceptance-r0-r7` (09:22) | `86e874ac5cc01e5ab5fca73d5d19f9c48487af01` |
| Commit count above base | `git rev-list --count dffb44ef..86e874ac` (09:26) | `1` (one docs commit; parent = `dffb44ef`) |
| Diff shape | `git diff --stat dffb44ef 86e874ac`; `--numstat`; `--name-status` (09:08) | `5 files changed, 494 insertions(+)`; numstat 51/153/11/180/99, all 0 deleted; A/A/A/M/A |
| PR metadata | `gh pr view 10 --json state,mergeable,mergeStateStatus,baseRefOid,headRefOid,changedFiles,additions,deletions,mergedAt` (09:26:13) | `OPEN`, `MERGEABLE`, `CLEAN`, base `dffb44ef…`, head `86e874ac…`, `changedFiles: 5`, `additions: 494`, `deletions: 0`, `mergedAt: null` |
| PR file list matches local | `gh pr view 10 --json files --jq '.files[].path'` vs `git diff --name-only` | identical, 5 paths |
| Docs only | `git diff --name-only dffb44ef 86e874ac` | only `docs/acceptance/demo-company/{PACKAGE,REHEARSAL-R0-R7,RUN-SHEET,CONTRACT-C1-C6-PENDING}.md` (A) and `README.md` (M); no non-`docs/` path |
| No product/fixture/evidence/test change | `git diff --quiet dffb44ef 86e874ac -- docs/acceptance/demo-company/fixtures docs/acceptance/demo-company/evidence tests scripts support package.json` (09:22) | exit 0 — identical |
| Baseline still OpenClaw | `node -e "…require('./package.json')…"` (09:10) | `openclaw-personalized-agent-baseline 1.5.5` (so the ruled DSH-era package is still absent, as the docs say) |

CI corroboration: `gh pr checks 10` (09:19) showed `CodeRabbit pass` and `test pass` (GitHub Actions run `41313746502`). This is extra, not a substitute for the local runs below.

---

## 2. File contents vs the report — PASS (one minor non-blocking defect, F1)

Read all five files at `86e874ac` (full text, 09:05–09:07), then checked the claims mechanically.

### 2.1 REHEARSAL-R0-R7.md — PASS
- R0–R7 labels are all present as headings (`### R0` … `### R7`), each with who/operator action/expected/evidence/status/grounding.
- Provenance table marks the text source: lane note `board/reports/team-a-job03-rehearsal-spec.md` (workspace, not in repo); the note exists and its R0–R7 headings match.
- Run status is explicit: line 15 `| Run status | **Not run.** Prep only. …` — not a completion claim.
- Per-step status is conservative: R0/R1/R3/R6/R7 `blocked`; R2/R5 `partially runnable` (explicitly "not package shape"); R4 `blocked`/`pending`. No step is called `clean`.
- Repo grounding links resolve (see link check, §2.6).
- N1 (minor prose note): the doc declares the README status vocabulary (`clean/failed/pending/stand-in/not run`) then uses `blocked`/`partially runnable` as pre-run statuses. The meanings are clear and conservative; no state is overstated.

### 2.2 PACKAGE.md — PASS, with F1
- Six named artefacts each carry a full 64-character SHA-256. I recomputed all six from the base tree (`git show dffb44ef:<path> | sha256sum`) at 09:24:51 and compared programmatically: **6/6 MATCH**.
- Evidence: 16 files under `evidence/dry-run-standin/` and 6 under `evidence/batch-standin/` — counts match. All 22 truncated prefixes printed in §3 match the recomputed SHA-256 of the named files (programmatic check: **22/22 PREFIX-MATCH**; full values in Appendix A).
- Honesty labels are present and correct: `"label": "STAND-IN OUTPUT (deterministic local stand-in; not model capability)"`; `clocks.live={"acts":0,"wallSeconds":0}`; `clocks.stateAdvanced={"acts":152,"simulatedDays":5,"wallSeconds":91.45}`; `modelsUsed` both sides say no model was called; `chains.count=7`, `scoredChains=7`, `everyChainScoredAtChainEnd=true`, `hopsRecorded=24` — all read from the committed `summary.json`.
- Batch evidence quotes verified against `evidence/batch-standin/batch-report.{md,json}`: live-driven 0 acts / 0 s; state-advanced 1768 acts / 56 simulated days / 805.47 s; 8 companies, 6 sectors, sizes 6/9/12, horizons 1–2 weeks; 72 `stand-in-limit` findings; TC-1 "proposal only — not applied".
- `order-e/` reference and holds verified in-tree: `README.md:3` GROUNDWORK ONLY; H1 no API key on record, H2 ruled package does not exist on main, H3 clean user not created.
- `order-f/` reference and holds verified in-tree: `DONE-BAR.md:3` HOLD; seven slots L1–L7 all `<HOLD - Nox to supply>`; `EVIDENCE-INDEX.md` rows say "no" / "not run".
- Not-in-package boundary is named (8 rows): absent DSH-era package, C1–C6 pending, Windows companions, phone/voice PACKAGE-PENDING, warehouse board/Supabase on the work-PC window, seven done-bar lines HOLD. `package.json` check confirms the OpenClaw baseline.
- **F1 (minor, non-blocking) — false cross-reference.** `PACKAGE.md:82–83` says: *"The elided tails are in the report's hash run; full 64-character values were measured with `sha256sum` on `dffb44ef` and repeat in `board/reports/team-a-job03-package-assembled.md`."* That is not true: the board report carries only truncated hashes (the dump digest at line 44, the six pinned artefacts and the four new files) and no full evidence values. The hashes themselves are correct — all 22 prefixes match the files — only the pointer to where full values repeat is wrong. Fix: drop the cross-reference, or paste the full 22 values into the report/this verdict (Appendix A has them). This does not affect the package set, the run state, the contract status or merge safety.

### 2.3 RUN-SHEET.md — PASS
- G1–G7 gates present with owners and evidence conditions: G1 installed output (Lane 1), G2 C1–C6 frozen (Nox/Lead), G3 arbitration closed (Nox), G4 host + company data, G5 checker named (Lead), G6 run-config accepted, G7 no secrets.
- Owner map covers operator (Cayleb via Nox), lane owner/recorder (Team A), install owner (Lane 1), fresh-context checker, host purchase/owner acts.
- R0 has exact shell commands (`hostnamectl`/`uname -a`, `sha256sum <package-tarball>`, `node docs/acceptance/demo-company/fixtures/check-run-config.mjs`, `ls -la /srv/ergon* 2>/dev/null`); R1–R7 each have the exact operator action and the evidence pack each must produce.
- Stop rules (four) and successor (G1 trigger → Lead routes to rehearsal checker) present.
- Fixture-side checks are explicitly marked "not the rehearsal" and "do not close any R step".

### 2.4 CONTRACT-C1-C6-PENDING.md — PASS
- Header line 3: `RECORDED, NOT APPLIED. NOTHING IN THIS FILE IS IN FORCE.`
- C1–C6 rows each end `PENDING Nox` — 6/6. The open arbitration (ADD-only `main[key]` W1 view vs documented reshape) has both sides `PENDING Nox — do not build`; line 43: `Nothing in this section is applied. No rows, no client code, no bundle, no install.`
- No contract value is presented as confirmed; C2 explicitly records the re-pin condition as not met.

### 2.5 README.md pointer — PASS
- Diff is +11 lines at the end only: a "Job 03 rehearsal package (Lane 5)" section pointing to PACKAGE.md, REHEARSAL-R0-R7.md, RUN-SHEET.md and CONTRACT-C1-C6-PENDING.md, with "prep only — the rehearsal has not run". Nothing else in the README changes.

### 2.6 Links and new-file hashes — PASS
- `node /tmp/checker-linkcheck.mjs` (my own parser; relative Markdown targets, anchors stripped, existence checked) at **09:29:03**: `checked 87 relative links across 5 files; missing 0` — matches the maker's 87/0.
- New-file SHA-256 from `sha256sum` (09:24:51) vs the report:
  - PACKAGE.md `f83a3b40d620fbda6cb96facd52f828c3d6c0637f1a8f906dcacf54c0142f491` — report prefix `f83a3b40d620fbda…` ✓
  - REHEARSAL-R0-R7.md `987fb156508244d608f1e73b5e0aa149589438751907afc0fdaefd4813d8273c` — `987fb156508244d6…` ✓
  - RUN-SHEET.md `b9bef871472071d37b9bfce8e1c426d264bcbead9a97841236c6d152c209ad63` — `b9bef871472071d3…` ✓
  - CONTRACT-C1-C6-PENDING.md `ef7a308c94405eaa7b9454d487b7312a5c0abd28e60f59df7e8c5bd2beddd143` — `ef7a308c94405eaa…` ✓

---

## 3. Maker run evidence — PASS (all items independently re-run)

I ran the fixtures from a `git archive` of `86e874ac` in `/tmp/pr10-checker-tree`, with the checkout's `node_modules` and `support/server/node_modules` symlinked read-only from the existing `ergon-main-merge` worktree. The extracted tree is byte-identical to the branch tree; no branch file was touched. Node was `v26.7.0` (the committed batch evidence also records `v26.7.0`).

| Claim | My command (timestamp) | My result | Verdict |
|---|---|---|---|
| check-voices 99/99 ALL PASS | `node fixtures/check-voices.mjs` (09:09:09) | `99 written lines checked (85 fixtures + 14 anchors); blind separation 99/99` … `ALL VOICES PASS`, exit 0 | **PASS** |
| check-run-config accepted | `node fixtures/check-run-config.mjs` (09:09:09) | `accepted: employeeModel claude-haiku-5-5, agentModel claude-haiku-5-5, route budget-proxy`, exit 0 | **PASS** |
| chains --check ok | `node fixtures/chains/chains.mjs --check` (09:09:32) | `ok: true`, `problems: []`, 7 chains, 6 kinds (`detail-dropped`, `number-transposed`, `attachment-not-forwarded`, `casual-approval-as-formal`, `urgency-inflated`, `chains-collide`), exit 0 | **PASS** |
| chains --calibrate separatesAll | `node fixtures/chains/chains.mjs --calibrate` (09:09:33) | `separatesAll: true`; 6 rows (C1–C6), every row `caught` / `charged-ahead` / `missed`, exit 0. N2: C7 shares C6's final (`final.shared`), so 6 unique finals cover 7 chains; the report's "every chain" is true of unique finals, though a literal reader may expect 7 rows. | **PASS** |
| setup/dump/teardown exit 0 | `node fixtures/setup.mjs $ROOT`; `dump-state.mjs`; `teardown.mjs` (09:11:42–43) | setup exit 0: `tickets.raised 104`, `replies 133`, `stackArtefacts 9`, `devices 10`; dump exit 0: `{"sha256":"558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1","bytes":315258,...}`; teardown exit 0: `{"empty": true}` | **PASS** |
| dry-run-week exit 0, 1m29.6 s, live 0 / state-advanced 152 acts, 7/7 chains, 24 hops | `node fixtures/dry-run-week.mjs $ROOT $OUT` (09:14:31–09:16:13) | exit 0; external wall **1m42.146 s**; summary: label `STAND-IN OUTPUT (deterministic local stand-in; not model capability)`, `clocks.live={"acts":0,"wallSeconds":0}`, `clocks.stateAdvanced={"acts":152,"simulatedDays":5,"wallSeconds":102.06}`, `chains.count=7`, `scoredChains=7`, `everyChainScoredAtChainEnd=true`, `hopsRecorded=24`, `modelCalls.viaBudgetProxy=314`, `modelsUsed` both sides no model called | **PASS** (deterministic fields exact; N3 timing variance) |
| check-stack 17 artefacts all pass | `node fixtures/check-stack.mjs $OUT` (09:26:08) | `artefacts 17; tools named 8; named but never touched: []; all checks passed`, exit 0 | **PASS** |
| wall-clock estimate exit 0 | `node fixtures/estimate-wallclock.mjs $OUT/summary.json` (09:26:09) | exit 0; basis line present ("assumed Haiku-class figures … to be replaced by measurement") | **PASS** |
| tests/package-contents.test.mjs 5/5 | `node --test tests/package-contents.test.mjs` (09:18:09) and exact `node tests/package-contents.test.mjs` (09:26:42) | `tests 5 / pass 5 / fail 0`, exit 0 (both runs) | **PASS** |
| 87 relative links, 0 missing | my link parser (09:29:03) | `checked 87 relative links across 5 files; missing 0`, exit 0 | **PASS** |
| PR state open/mergeable | `gh pr view 10 …` (09:26:13) | OPEN / MERGEABLE / CLEAN; base `dffb44ef…`, head `86e874ac…`, 5 files, +494/−0, mergedAt null | **PASS** |

No item is marked VERIFIED-BY-ARTEFACT: the full dry-run was run once independently. The maker's timing difference is not a claim failure — the maker's own log `/tmp/l5-package-verify2-20261101T084646.log` starts 08:46:46 and ends 08:48:16 (~90 s), consistent with the reported 1m29.6 s; my host ran the same deterministic work in 1m42.1 s (summary wall 102.06 s). All deterministic outputs (acts, days, chains, hops, model-call count, digest) match exactly.

---

## 4. No run claim, no unruled contract applied, no main touch — PASS

- **No rehearsal-run claim.** PR body: *"The rehearsal has not run; nothing here claims a run."* Commit `86e874ac`: *"Prep only: the rehearsal has not run and nothing here claims a run."* `REHEARSAL-R0-R7.md:15`: `**Not run.** Prep only.` `RUN-SHEET.md:3`: `**Status: PREP ONLY. THE RUN HAS NOT STARTED.**` `PACKAGE.md` §Run state: *"The rehearsal has not run."* No pass-1/pass-2 evidence directory exists in the tree.
- **C1–C6 stay PENDING.** `CONTRACT-C1-C6-PENDING.md` has 6/6 `PENDING Nox` rows and says at line 3 `NOTHING IN THIS FILE IS IN FORCE` and line 43 `Nothing in this section is applied`. No product code, config, row set, bundle or install is added by the PR.
- **Main untouched.** `git ls-remote origin main` = `dffb44ef…` (unchanged); PR `mergedAt: null`; the branch is one commit on `dffb44ef`; I did not merge or push.
- **Docs only.** All 5 changed paths are Markdown under `docs/acceptance/demo-company/`; fixtures, committed evidence, tests, scripts and `package.json` are identical between base and head.
- **Scope note (Batch 2).** The board report was extended at 09:12:50 ET with a "Batch 2" addendum (run-evidence skeleton + R0 provisioning checklist). It explicitly says nothing was committed to this branch and that Batch 2 is workspace-only; the remote branch tip was still `86e874ac` at 09:22 and 09:26. I verified that PR-relevant claim (head unchanged) but did not assess the workspace-only Batch 2 tools, which are outside PR #10.

---

## Findings and notes

| ID | Severity | Finding | Effect |
|---|---|---|---|
| **F1** | Minor, non-blocking | `PACKAGE.md:82–83` falsely says the full evidence hashes "repeat in `board/reports/team-a-job03-package-assembled.md`"; the board report has no such values (only truncated dump/artefact/new-file hashes). All 22 evidence hashes in PACKAGE.md are themselves correct (verified against the files). | Documentation only; fix one sentence or add the full values (Appendix A). Does not affect merge safety. |
| N1 | Note | REHEARSAL declares the README status vocabulary but uses `blocked`/`partially runnable` for pre-run statuses. | Cosmetic; meanings are conservative and clear. |
| N2 | Note | `--calibrate` emits 6 rows for 7 chains because C7 shares C6's final (`final.shared`). The report's "every chain" is true of the 6 unique finals. | No action needed; explanatory. |
| N3 | Note | My dry-run external wall time was 1m42.1 s vs the maker's 1m29.6 s (summary 102.06 s vs 89.51 s). | Environment timing only; all deterministic values match; the maker's log corroborates ~90 s. |
| N4 | Note | The board report gained a Batch 2 addendum at 09:12:50 ET after PR #10 was filed; it is workspace-only and explicitly does not touch the branch. | Out of PR scope; head verified unchanged. |

No FAIL and no UNVERIFIED items remain.

---

## Overall verdict

**PASS — safe to merge as docs-only.** The base/head pins, file count, line counts, docs-only scope, all five file descriptions, all six full artefact hashes, all four new-file hashes, all 22 evidence-hash prefixes, the re-run fixture checks, the 5/5 repo guard, the 87/0 link check, the unrun rehearsal status, the PENDING C1–C6 record and the untouched `main` all verify. Under the task rubric I do not score F1 as FAIL: it is a provenance pointer, not a claim about the branch, the run, the contract status or any hash; all of those verify. F1 is a one-sentence documentation correction — if a zero-defect docs merge is required, fix it before merge, otherwise it is safe to merge and fix as a follow-up. Merge authority remains with the Lead/Nox; I did not merge, push or modify the branch.

---

## Appendix A — full evidence hashes (recomputed 2026-11-01 09:14–09:24 ET)

These are the full values behind the truncated prefixes in PACKAGE.md §3; all 22 match.

| File | SHA-256 |
|---|---|
| `evidence/dry-run-standin/acts-mon.jsonl` | `8b30656564c00cda3740fd6754a5bdaa0528665b7d8ae2531996b1b381c36221` |
| `evidence/dry-run-standin/acts-tue.jsonl` | `f1b8d65d2df5bf65cac25972fc71d5a30a8f96ddf140221caab1b4487a0bd18c` |
| `evidence/dry-run-standin/acts-wed.jsonl` | `94538e3d5e1c1bae7f3b682712912891f14a5e8d63e94bb679738f518859cf9b` |
| `evidence/dry-run-standin/acts-thu.jsonl` | `c65d37da4b6763b1057a271c1e4f428399dc568b1e55523d8d204018a19a7e32` |
| `evidence/dry-run-standin/acts-fri.jsonl` | `686b036cdd3fe095de8a3f7a74e0d3c8f6e6ac03bac1d09ee9a85eeeee6d0547` |
| `evidence/dry-run-standin/summary.json` | `f4708b7e4cba4fccce719293e075c2b39b0972392d9bf4f3241b90f8045af7cb` |
| `evidence/dry-run-standin/captures.json` | `91f50baff19d9f02fe456c4fd9c48232caf4af053eb2cba612a9f977800a571d` |
| `evidence/dry-run-standin/chains-check.json` | `c1e0a1c2859cc775586aaad4a96642ea411b2baf681d18fd2fb1552bbe77f10a` |
| `evidence/dry-run-standin/chains-calibrate.json` | `2ff8abfdcb81b07f93adb91dd0063c538ba7e68934dee1d1d03cbc976bda502b` |
| `evidence/dry-run-standin/check-voices.txt` | `ec08c401f197f90e9bf732c6e60a09c4a467b2ddb95162b5ec863dfe6ab86476` |
| `evidence/dry-run-standin/check-stack.txt` | `ebb53cf588b73009049194a4f3563b2c851a0aeca23b35581897aaf9c1ffef06` |
| `evidence/dry-run-standin/board-friday-close.json` | `d4faf71082ebe932d0c9792fd2d417101c6f794799543bfd0f9f319228a3032e` |
| `evidence/dry-run-standin/friday-answers.json` | `ebe69203edb4fffd46956db86c43ed59b3ca3d64bcba89340863409369a7218c` |
| `evidence/dry-run-standin/monday-answers.json` | `14dc0d25be9817d7fc44b200ef9c5603634516bbadf3ca273f9c063a64ee2d5f` |
| `evidence/dry-run-standin/gate-audit-weekly-estimate.jsonl` | `23de878fbacb732e0f81bf2d6674cfeeeea880ece316070a32cae436d36d328c` |
| `evidence/dry-run-standin/library-weekly-estimate.json` | `14ccc50f550a6140d225c5af04ad04ab30183edca88306e85e8dacd9dae09440` |
| `evidence/batch-standin/batch-evidence.json` | `25506552ec326567b8c6dc41b08a13bc0c9edb0f8c87b283e08967021de581f3` |
| `evidence/batch-standin/batch-ledger.jsonl` | `435a279a4539b58b094e52ad03fefb5a5918b5f44fdf48e70c6cde05450db9cc` |
| `evidence/batch-standin/batch-report.json` | `2997336eb83cada7791d919a66522777b40a6802eccfdafa7e378bb9f292fca8` |
| `evidence/batch-standin/batch-report.md` | `47adaacad731d5b31dca9204a0ebbe4648da2317234a5cb91bfa6b05e91df603` |
| `evidence/batch-standin/instances.json` | `fc570eb3307c72318fb34ccf1c4723007250d559c15d1c031a293c88cb13da41` |
| `evidence/batch-standin/manifest.json` | `9018cf0c7d596ba97db0e3479483e8a99f699780f2e4319be268812159f21773` |

Six pinned artefacts at `dffb44ef` (recomputed by `git show dffb44ef:<path> | sha256sum`): COMPANY.md `6ef0f314f01b3b4eb7187e3fd7b4f573cc8151428964646662c7b520690ddbac`; CAST.md `42baea0e6d59e5e36e60ef267bdf547c5f69d5a121e55c84bafe5ec7525b7b26`; SEED.md `5fbd8e338162051479fa4dac59ecf648d3f2a78b2ee4985c3cc4be9dfb5e23bb`; SCENARIOS.md `44cc9f875814965104ba5b71989ecb09c840b9cfd3ca3c5ab6de4d2878ac46fd`; COMPETENCY.md `8d7701e2d789db480297780b5b1b217faf5d7cd02333fd28486ea31bbc1a1163`; RUN-REPORT-TEMPLATE.md `5b46b876002c9af08d53bcf69b2e6a755db634a1388184b1d2f7c317406ead89`.

## Appendix B — commands and scratch paths

- Worktree: `/home/cayleb/workspace/harness-build/a-l5-seat` (read-only).
- Independent test tree: `/tmp/pr10-checker-tree` = `git archive 86e874ac` + symlinked `node_modules` and `support/server/node_modules` from `/home/cayleb/workspace/harness-build/ergon-main-merge` (read-only).
- Dry-run scratch: `/tmp/pr10-dryroot2-Cqw4Kx` (root) and `/tmp/pr10-dryout2-KW9kg7` (output); log `/tmp/pr10-dryrun2.log`, timing `/tmp/pr10-dryrun2.time`.
- No repository file was modified. This verdict file is the only repo write.
