# Ergon checker — JOB 02 GATE VERDICT: **PASS**

**Checker:** `ergon-checker` (independent fresh context; built none of this work). **Reports to:** `lead` (copy to `ergon-lead`).
**Package:** repo `cayleb-james2008/ergon`, branch **`duet/p21-fixes-scaleup`**, cut from merged main `1c71898`, maker-declared final SHA **`4387b519eb64f0fbf2cd8ccf2b060f435cce2474`** (pushed; remote tip = this SHA).
**Gate run:** 2026-10-31 21:36–21:46 ET on `outis-desktop`; my own gate seat `/tmp/ergon-gate-462277c` (fresh clone from the Pack 20 gate), fetched and checked out at `4387b51`; `package-lock.json` md5 unchanged (`094f3d4e…`) so the real `npm ci` dependency tree and hash-verified document runtime stayed valid. Gate environment per my V3 ruling: `OPENCODE_BIN` unset, no `opencode` on `PATH`.
**Writes made:** this report + `board/probes/ergon-checker-gate-job02-runs.txt`. No repo/branch/merge write; zero spend; no credentials; R1 unused.

## Verdict

**PASS on every Job 02 part and every ruled item.** Pack 21 items each have verified before/after evidence (their tests fail on the parent `1c71898` and pass on the item); the scale-up batch driver enforces the two-clock separation and its committed evidence is honest and independently reproducible; both residuals are executed with written rulings; F1–F4 landed; the documented-env suite on `4387b51` is `0 fail / 0 incomplete / exit 0` with **no check excluded**. Three low-severity observations are recorded below; **none blocks the merge**.
**Merge recommendation: YES — the Lead may authorise merging `4387b51` into `main`.** After the merge I run the merged-tree proof (main ancestry, merged-tree suite, 0 branch-attributable). My veto is lifted for this package.

## 1. Package and preconditions

| Check | Result |
|---|---|
| Final SHA is the pushed remote tip | `git ls-remote … duet/p21-fixes-scaleup` → `4387b51…` = my checkout |
| Branch carries the merged main base | `git merge-base --is-ancestor 1c71898 HEAD; echo $?` → **0** |
| Commits ahead of main / clean tree | 12; `git status --porcelain` → **0** |
| Dependency identity | `package-lock.json` md5 `094f3d4e7735bc929c70a3786b029ab0` (same as main) |
| **Handoff SHA note** | Handoff names `90ddcad`; the maker's message and the remote tip name **`4387b51`**. I gated `4387b51`. The `90ddcad..4387b51` delta is **docs/evidence only** (10 files under `docs/acceptance/demo-company`, +251/−225: dry-run evidence regenerated with clock labels). |

## 2. Documented-env suite on the final SHA (my run, 21:36:10–21:37:25 ET)

```
TOTAL 1616 cases: 1607 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 158/158 checks ran, 0 without a complete report;
4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs
Windows); 0 manifest problems
AGGREGATE_OUTCOME {"outcome":"completed","testExecution":"reported-complete","planned":158,"executed":158,
"cases":{"tests":1616,"pass":1607,"fail":0,"cancelled":0,"skipped":9,"todo":0},"uncertainChecks":[],...}
EXIT=0        (grep '^not ok' -> 0 lines)
```
This matches the maker's claim exactly. **Non-passing disposition:** 9 skips, each with a named reason (5 environment: netfx-compile, nsis-compile, outis-dpapi-roundtrip, remote-service-safety, ui-screens; 4 opencode-e2e cases needing the binary — 4 checks skip every case, all manifest-allowed), plus 6 manifest invocations that need .NET/Windows seats and are never counted as passing. No failures, no incomplete checks.

**Count movement is tests, not exclusions — verified:** base main manifest 163 invocations / 157 checks → final 164 / 158. Manifest diff: **added `tests/lock-identity.test.mjs`; removed none**; `test-all --list` → "MANIFEST matches the test files on disk", 0 manifest problems. Case delta 1,609 → 1,616 = +7 net from new cases in existing test files (review, phone-dispatch, anthropic-budget-proxy, db-migration-access, exit-codes) plus the new check.

## 3. Pack 21 — per ruled item (before = parent `1c71898`; after = final `4387b51`)

| Item | Verdict | Before/after evidence (my runs) |
|---|---|---|
| **E-1** review-store retention (`9dadf92`) | **PASS** | Final `tests/review.test.mjs` **28/28 pass**. Parent: exit 1 — `SyntaxError: The requested module '../support/review/core.mjs' does not provide an export named 'enforceRetention'`. Implementation inspected: `LIMITS.retentionDays=90`, `totalRequests=5000`; retention pass on every open + inside submissions; decided-review **BLOB-only** drop keeps rows/body hashes/events + `artifacts_dropped` event; hard backstop evicts oldest stale then oldest decided, **never open**, into a store-level `request_evicted` audit; schema v2 migration; `enforceRetention`/`listAuditEvents`; payload use refuses `artifact_dropped_by_retention` (`core.mjs:327`). |
| **E-6** one lock identity (`247a003`) | **PASS** | Final `tests/lock-identity.test.mjs` **4/4 pass**. Parent: `ERR_MODULE_NOT_FOUND support/lock-identity.mjs`. One module: boot identity (kernel boot UUID or boot-time estimate) + pid + process start time; `holderStatus` self/alive/pid-reused/stale; `isSelfIdentity`. Wired into **onboarding/plans.mjs AND suite/budget-proxy/src/proxy.js** (shared imports, `processIdentity()`, `holderStatus()`; format-2 records; legacy format-1 still probed). The test guards both locks use the shared design, not the old pid-only identity. |
| **E-2** phone-plane isolation + per-request contact checks (`3ad6c38`) | **PASS** | Final `tests/phone-dispatch.test.mjs` **33/33 pass**. Parent: 3 failing E-2 cases — per-request contact checks/digest, store scope check, handler wiring. Implementation: `approvedContacts`+`subAccountId` in the policy digest; every request must name the approved contact, dial exactly its number, match the subaccount (`CONTACT_REQUIRED`/`CONTACT_DENIED`/`SUBACCOUNT_MISMATCH`); `support/server/phone-store.mjs` `assertScope()` (`PHONE_SCOPE_*`) on dispatch + inbound webhook; handler wires both. |
| **S-2 rider** loud named token-map refusal (`fa6573f`) | **PASS** | Final `tests/anthropic-budget-proxy.test.mjs` **39/39 pass**. Parent fails the named rider case: *"S-2 rider: a non-slug token-map key is refused loudly at map load, named by reason/entry with a review event, never echoing key or token"*. Implementation: `suite_map_key_invalid` + reason `employee_id_not_slug` + entry position, never the key/token; durable `<map>.review.jsonl` review event; malformed/duplicate tokens refused by entry. |
| **S-4 rider** origin pinned to deployment domain (`fa6573f`) | **PASS** | Parent fails *"S-4 rider: the suite-map origin is pinned to the deployment's own tailnet domain from its provisioning config"*. Implementation: `deployment.json` provisioning config (`format:1, tailnetDomain`), `init --tailnet-domain`; `enable`/rewrites/`check-suite-map` require origin host == that domain (`suite_map_origin_not_deployment`). |
| **F-2 / F-4 / F-5** dispositions (`1d4d87c`) | **PASS** | Docs-only commit (22 lines). Written reasons in `docs/review-debt/pack21-f-item-dispositions.md`: all three **INCLUDED as verified delivered by PR #6 `f906d56`**, nothing rebuilt. I verified `f906d56` is an ancestor of main (exit 0; "Merge pull request #6 … review/parts-handbook-fixes"), the acceptance tests are on main (F-2 at `kb.test.js:104,115`; F-4 `corrupt_store`; F-5 unknown-source marker), and `suite/parts-handbook` tests **100/100 pass** on the final tree. |
| **F-1** broker-spawn redesign | **PASS (untouched)** | No broker/spawn source file changed on the branch (only the S-2/S-4 test file matches "anthropic-budget-proxy"); no commit claims F-1; brief says out of scope. |

## 4. Template-hardening scale-up X-1..X-4 (`7dd1982`)

- **Built:** `fixtures/batch/instances.mjs` (company-instance generator over sector/size/role/pressure/horizon/seed axes; stack archetype declared constant), `fixtures/batch/run-batch.mjs` (batch driver), `fixtures/capture-batch-evidence.mjs` (reproducible evidence generator).
- **Separation enforcement inspected and exercised:** the driver **refuses to report** unless every act row carries `clock` and a stand-in batch reports **zero** live acts (`run-batch.mjs:74-95`); `--live` refuses today (my run: exit 2, *"a live run would be theatre. Refusing."*).
- **Committed evidence** (`evidence/batch-standin/`): honesty block `{"live":{"acts":0,"seconds":0},"stateAdvanced":{"acts":1768,"simulatedDays":56,"wallSeconds":780.51,"note":"state-advanced time is NOT live operation"},"everyActLabelled":true,"stateAdvancedReportedAsLive":false}`; axes companies 8 / sectors moving,dental,legal,school,fabrication,charity (6) / sizes [6,9,12] / horizons [1,2] weeks / pressure [1,1.25,1.5,2]; **sum 1,768 acts, 56 simulated days**; **72 findings** (all `stand-in-limit`); **1 template candidate** (TC-1, "proposal only — needs the review gate and a regression batch; not applied by this run").
- **Independent reproduction:** I ran a 1-company batch fresh (21:41:13–21:42:42 ET, exit 0) — **152 acts / 314 stand-in calls / 0 live acts / 6 findings**, honesty block correct. This matches the committed Riverton row's 152 acts / 314 calls exactly (wall 88.0 s vs 74.96 s, expected variance).
- **Verdict: PASS.** Honest labelling confirmed: state-advanced is never counted as live.

## 5. Residuals

- **(a) `scripts/check.mjs` RETIRED and executed** (`33ea4f9`): `scripts/check.mjs` and `scripts/Check.ps1` absent; npm `check` alias removed; scaffold messages updated to explain the retirement; `docs/EXIT-CODES.md` exit-code row and usage row removed + a "Retired entry points" note; `tests/exit-codes.test.mjs` updated; ruling doc `docs/review-debt/pack21-check-mjs-ruling.md` rests on evidence (what the script did, OpenClaw superseded by DeepSeek Harness, no DSH-era equivalent, rehearsal independent). **PASS.**
- **(b) Migration access gate landed as declared pre-deployment preflight** (`bb7900c`): `scripts/preflight-db-migrations.mjs`; my run on the final tree with no migrations dir → *"nothing to apply and nothing to check (declared pre-deployment preflight; no product path applies migrations yet)"*, exit 0; given a directory it propagates the gate's exit 1 (test asserts); `docs/DB-MIGRATION-ACCESS-GATE.md` has the "Where the gate is called" section; `tests/db-migration-access.test.mjs` **51/51 pass**. **PASS.**

## 6. F1–F4 docs findings from the Pack 20 gate (`5ac9bd6`) — verified in the diff

F1 `TEMPLATE-HARDENING.md` X-2 now marks `day-close-hashes.json` as **batch-driver output** and points the single-company runner at `summary.json days[].close.canonicalDumpSha256` / `captures.json`; F2 `SELF-IMPROVEMENT.md` step 2 reworded to "No product or runtime code calls …; only the labelled Pack 20 fixture rehearsal does"; F3 the gate-audit block is marked **an annotated rendering of that JSONL**; F4 `README.md` now leads with the P20-8 finding. **PASS.**

## 7. Pack 20 regression + determinism on the Job 02 tree

Fresh dry run on `4387b51` (21:44:38–21:46:10 ET, exit 0): act rows carry `clock:"state-advanced"` + week + simulated date; summary `clocks {"live":{"acts":0,"wallSeconds":0},"stateAdvanced":{"acts":152,"simulatedDays":5,"wallSeconds":91.92}}`; chains 7/7 scored at chain end, 24 hops, C1 `resurfaced.stillMissing: true`. My Pack 20 gate-scope checker with that fresh summary: **GATE-SCOPE PASS 6/6** (no regression to the Pack 20 scope). Fresh day-close hashes equal the committed evidence exactly (mon `bbf6b8fa…`, tue `58e1d54f…`, wed `d76524e3…`, thu `64ec7a78…`, fri `7c43e1c8…`).

## 8. Findings (all low, non-blocking)

- **J2-F1 — `docs/EXIT-CODES.md` line 71 is stale after the retirement**: it still describes `scripts/check.mjs` as an entry point exercised by `tests/exit-codes.test.mjs` from a private copy, but the script and those tests were removed by `33ea4f9`. The retirement note was added; this paragraph was not updated. One-line docs fix in the next Ergon commit.
- **J2-F2 — batch ledger lacks the TC-1 proposal event**: `batch-ledger.jsonl` records the 72 `finding_recorded` events but no candidate/proposal event; the candidate is in `batch-report.{json,md}` with its promotion path. X-3's ledger spec (finding → proposal → sign-off → gate decision → version → regression) would record the proposal when the candidate is promoted; recommend adding it then. Not a blocker for a "proposal only" run.
- **J2-F3 — process note**: the checker handoff names `90ddcad` while the final pushed SHA is `4387b51`; the maker's message and the remote tip agree on `4387b51`, and the delta is docs/evidence-only, so the gate is unaffected — but the handoff should be pinned to the final SHA for the next gate.

## 9. Merge recommendation and next step

**PASS → the Lead may authorise merging `4387b51` into `main`** (no merge by me; my veto is released for this package). After the authorised merge I run the merged-tree proof: remote `main` tip, `git merge-base --is-ancestor 4387b51 <new-main>` = 0, merged-tree suite in the gate environment (0 fail / 0 incomplete / exit 0), and `comm -23` failing-case diff = 0. If any leg fails I name the failing leg and the branch-attributable cause; nothing is reverted or re-merged unilaterally.

## 10. Reproducibility

Raw extracts with timestamps and quoted output: `board/probes/ergon-checker-gate-job02-runs.txt`. Gate scope checker: `board/probes/ergon-checker-gate-checks.mjs`. Throwaway seat/logs under `/tmp/ergon-gate-462277c` and `/tmp/ergon-checker/logs/` (disposable); the durable record is this report + the probe file.

*Written by `ergon-checker`, 2026-10-31 ~21:50 ET. I built none of this work; every number quoted was produced by a command I ran in this gate, on the maker's pushed SHA, in my own seat.*
