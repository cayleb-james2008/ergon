# Team A — Job 02 checker handoff (Pack 21 + template-hardening scale-up + residuals)

Prepared by `ergon-lead`, 2026-10-31 ~21:45 ET. Per the Lead's conditions: the checker is independent; this file names the
package, the scope-by-scope verification commands and the known open items. It does **not** hand my results over as
evidence — run the actual output in your own fresh context and seat. Gate environment per the checker's ruling:
`OPENCODE_BIN` unset and no `opencode` binary on `PATH`; the documented-env suite must be **0 fail / 0 incomplete** and a
merged-tree proof must show **0 branch-attributable** failures before any merge.

## The package

- Repo `cayleb-james2008/ergon`, branch **`duet/p21-fixes-scaleup`**, cut from merged main **`1c71898`** (Pack 19 + Pack 20,
  post-merge proof PASS), **final SHA `4387b51`** (pushed; the earlier `90ddcad` line was stale — the delta from it to
  the gated tree is docs/evidence-only). One merge candidate; no main write, no force-push.
- Every item commit names the brief text it implements (the Pack 18 three-axis review document was not reachable from
  Team A's seat; Lead-accepted C1: the brief's item text is the contract). This is stated in each commit message.

## What to verify, scope by scope (commands run from the repo root unless noted)

**Pack 21 — each ruled item needs its own before/after test evidence.** The item commits and their test surfaces:

1. **E-1 review-store retention** (`9dadf92`): `support/review/core.mjs` — LIMITS.retentionDays=90 / totalRequests=5000;
   `enforceRetention`, `listAuditEvents`; decide→91 days→`enforceRetention` drops BLOB payloads only (row/body hashes/events
   kept, `artifacts_dropped` event), backstop evicts oldest stale then oldest decided (never open, `request_evicted` audit),
   payload use refuses `artifact_dropped_by_retention`. Tests: `node --test tests/review.test.mjs` (E-1 tests). Before/after
   check in the commit message: parent main exits 1 (no `enforceRetention`), branch exit 0 with the drop/keep summary.
2. **E-6 lock identity** (`247a003`): one module `support/lock-identity.mjs` (boot identity + pid + process start time;
   `holderStatus` self/alive/pid-reused/stale), wired into `onboarding/plans.mjs` and `suite/budget-proxy/src/proxy.js`
   (format-2 records; legacy format-1 still probed; same-process reopen also requires `isSelfIdentity`). Tests:
   `node --test tests/lock-identity.test.mjs`, plus `tests/onboarding.test.mjs` and
   `node --test suite/budget-proxy/test/*.test.js`. Parent main lacks the module (exits 1).
3. **E-2 phone-plane isolation + per-request contact checks** (`3ad6c38`): `scripts/phone-dispatch.mjs` policy carries
   subAccountId + approvedContacts (in the policy digest); every request must name the approved contact, dial exactly that
   contact's number and match the subaccount (CONTACT_REQUIRED/CONTACT_DENIED/SUBACCOUNT_MISMATCH); `support/server/phone-store.mjs`
   `assertScope()` checks dispatch + inbound webhook; `support/server/phone-handler.mjs` wires it end to end. Tests:
   `node --test tests/phone-dispatch.test.mjs`, plus the other phone/agentphone tests. Parent fails (valid request
   INVALID_INPUT, no assertScope, handler unwired).
4. **S-2 rider** (`fa6573f`): map load refuses a non-slug key loudly — `suite_map_key_invalid` + reason
   `employee_id_not_slug` + entry position, never the key/token — and records a durable review event `<map>.review.jsonl`;
   tokens malformed/duplicate refused by entry. **S-4 rider** (same commit): `init --tailnet-domain HOST.ts.net` writes the
   deployment provisioning config (`deployment.json`); suite-map `enable`/rewrites require the origin host to BE that
   domain (`suite_map_origin_not_deployment`); `check-suite-map` audits it. Tests:
   `node --test tests/anthropic-budget-proxy.test.mjs` (S-2/S-4 rider tests + updated expectations).
5. **F-2 / F-4 / F-5 dispositions** (`1d4d87c`): written include/defer reasons in
   `docs/review-debt/pack21-f-item-dispositions.md` — all three **INCLUDED as verified-delivered by PR #6 `f906d56`**
   (Parts Handbook findings; code + acceptance tests on main; `node --test suite/parts-handbook/test/*.test.js`).
   No rebuild (Lead-accepted C2). F-1 unchanged/out of scope.

**Template-hardening scale-up — X-1..X-4** (`7dd1982`, docs `TEMPLATE-HARDENING.md`): `fixtures/batch/instances.mjs`
(company-instance generator over sector/size/role/pressure/horizon/seed axes; stack archetype declared held constant) and
`fixtures/batch/run-batch.mjs` (batch driver that **refuses** to report unless every act row carries `clock` and a stand-in
batch reports zero live acts; `--live` refuses today). The runner gained `--weeks N` state carry with week-suffixed
ids/refs, and the summary carries `clocks.live` vs `clocks.stateAdvanced` separately. Committed evidence:
`docs/acceptance/demo-company/evidence/batch-standin/` (batch-report.{json,md}, batch-evidence.json, batch-ledger.jsonl,
instances.json, manifest.json). **Verify:** the evidence's `honesty` block states live acts 0 and “state-advanced time is
NOT live operation”; the driver's separation check rejects missing labels; reproduce with
`node fixtures/capture-batch-evidence.mjs --companies 8` (or `--from <run dir>` from an existing run). Full run reported:
8 companies / 6 sectors / sizes 6,9,12 / pressure 1–2x / horizons 1–2 weeks / 1,768 acts / 56 simulated days / 0 live acts /
72 findings (stand-in limits) / 1 template candidate (proposal only).

**Residuals.**
- (a) `scripts/check.mjs` retire-vs-rescope: **RETIRED**, executed (`33ea4f9`), with the emailed reasoning in
  `docs/review-debt/pack21-check-mjs-ruling.md`; wrapper `scripts/Check.ps1`, npm alias, EXIT-CODES rows and its tests
  removed; scaffold messages updated.
- (b) migration access gate doc sentence + landing call (`bb7900c`): doc exists on main; no live caller existed (no
  migrations, no runtime applier). Landed as the Lead-ruled **declared pre-deployment preflight**:
  `scripts/preflight-db-migrations.mjs` (+ test in `tests/db-migration-access.test.mjs`, + a "Where the gate is called"
  section in `docs/DB-MIGRATION-ACCESS-GATE.md`) — empty tree prints "nothing to apply and nothing to check" (exit 0,
  explicitly not a gate pass); given a directory it propagates the gate's findings (exit 1).

**Docs findings F1–F4** from the Pack 20 gate: `5ac9bd6` (F1 X-2 batch-driver evidence naming; F2 SELF-IMPROVEMENT wording;
F3 gate-audit block marked annotated; F4 README leads with the P20-8 finding). Verify against the checker's own texts.

## What the checker should independently run

1. `git merge-base --is-ancestor 1c71898 HEAD` exits 0; final tree is `4387b51` (merged to main as `c196c25`, merged-tree proof PASS).
2. The item tests above (or the whole documented-env suite, which includes them).
3. `docs/acceptance/demo-company` fixture checks: `node fixtures/check-voices.mjs`, `node fixtures/check-stack.mjs <tmp>`,
   `node fixtures/chains/chains.mjs --check`/`--calibrate`, and one `node fixtures/dry-run-week.mjs ROOT OUT` run (its act
   rows now carry `clock: 'state-advanced'` and the summary carries `clocks`).
4. Scale-up: inspect `fixtures/batch/run-batch.mjs`'s separation refusal, then reproduce the batch evidence
   (`--companies 8` or `--from` an existing run dir). Do not accept a batch report whose labels are absent or whose live
   total is non-zero without a real model run.
5. Full suite in the documented gate env. Expected shape: 0 fail / 0 incomplete; counts will have moved from main because
   this branch adds tests (see my last message for the exact line; your own run governs). 4 checks skip every case with
   manifest-recorded reasons; 6 manifest invocations need .NET/Windows seats and are never counted as passing.
6. Judge the docs-only parts on content (F1–F4, F-item dispositions, check.mjs ruling, migration preflight doc).

## Known open items the checker should adjudicate (not silently pass)

- Live-model tier of the scale-up was **not** run (no spend tonight; Monday reset); the committed batch is
  state-advanced/stand-in and says so. Do not read the verdicts as model capability.
- The `opencode` gate environment per your ruling; no code change was made for it.
- Nothing is merged: your PASS (with the merged-tree proof) precedes any merge, and the Lead authorises it.

## Deliverable from the checker

A written verdict on `board/reports/`: PASS/FAIL per Job 02 part and per ruled item (with the before/after test evidence
inspected), the documented-env suite totals from the **final pushed SHA**, the batch separation check, every non-passing
check with its disposition, and the merge recommendation. The maker does not certify this work.
