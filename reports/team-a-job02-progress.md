# Team A — Job 02 progress and evidence (Pack 21 + scale-up + residuals)

Author: `ergon-lead`. Written 2026-10-31 ~22:35 ET (last evidence run 21:31–21:35 ET). Branch
**`duet/p21-fixes-scaleup`**, cut from merged main **`1c71898`**, **FINAL SHA `4387b51`** (pushed). Seat:
`/home/cayleb/workspace/harness-build/p21-seat`. $0 live-model spend, no credentials, no main write, no force-push, no
merge. Maker never checker: the PASS below is `ergon-checker`'s; handoff at
`board/reports/team-a-job02-checker-handoff.md`.

**Contract note (Lead-accepted C1):** the Pack 18 three-axis review document was not reachable from this seat; the brief's
item text is the contract and every item commit names the text it implements. Nothing was reconstructed from memory.

## Commits on the branch (11)

`5ac9bd6` F1–F4 docs corrections · `9dadf92` E-1 · `247a003` E-6 · `3ad6c38` E-2 · `fa6573f` S-2/S-4 riders ·
`1d4d87c` F-2/F-4/F-5 dispositions · `33ea4f9` residual (a) check.mjs retirement · `bb7900c` residual (b) migration
preflight · `7dd1982` scale-up build + full batch · `6310273` README batch row · `90ddcad` suite bookkeeping fixes ·
`4387b51` regenerated clock-labelled dry-run evidence.

## 1. Pack 21 — each ruled item, built with before/after evidence

All "before" runs executed the same check against parent main `1c71898`; all "after" runs on the branch. Parent exit 1 in
every case; branch exit 0.

| Item | Delivered | Before (parent) | After (branch) | Tests |
|---|---|---|---|---|
| **E-1** review-store retention | `support/review/core.mjs`: 90-day decided-review BLOB drop keeping row/body hashes/events + per-request `artifacts_dropped`; hard backstop `LIMITS.totalRequests=5000` evicting oldest stale then oldest decided (never open) with `request_evicted` store audit; runs on open and inside submit; `artifact_dropped_by_retention` refusal; schema v2 | no `enforceRetention`/`listAuditEvents` | `{droppedRequests:1, droppedArtifacts:1, freedBytes:27, events:1, decidedRowKept:true, openPayloadKept:true}` | `tests/review.test.mjs` E-1 tests; 28/28 |
| **E-6** lock identity, one design | New `support/lock-identity.mjs` (boot identity + pid + process start; `holderStatus` self/alive/pid-reused/stale; Linux /proc start-time proof, conservative elsewhere) wired into `onboarding/plans.mjs` and `suite/budget-proxy/src/proxy.js` (format-2 records, legacy fallback, same-process reopen needs `isSelfIdentity`) | no shared module; pid-only identity in both locks | `{self, reusedPidStatus:'pid-reused', deadPidStatus:'stale', otherBootStatus:'stale', wired:true}` | `tests/lock-identity.test.mjs` 4/4; onboarding 26/26; budget-proxy 147/147 |
| **E-2** phone-plane isolation + per-request contact checks | `scripts/phone-dispatch.mjs`: policy carries subAccountId + approvedContacts (both in the policy digest); every request must name the approved contact, dial exactly that contact's number and match the subaccount; contact/subaccount in the request digest. `support/server/phone-store.mjs` `assertScope()` on dispatch + inbound webhook; `support/server/phone-handler.mjs` wires it end to end | valid request INVALID_INPUT (unknown fields), no assertScope, handler unwired | `{accepted; foreign number CONTACT_DENIED; unknown CONTACT_DENIED; missing CONTACT_REQUIRED; foreign subaccount SUBACCOUNT_MISMATCH; storeAssertScope:true; handlerWired:true}` | `tests/phone-dispatch.test.mjs` 33/33 (+handler wiring guard); all phone/agentphone 104/104 |
| **S-2 rider** | Map load refuses non-slug keys loudly: `suite_map_key_invalid` + reason `employee_id_not_slug` + entry, never key/token; malformed/duplicate tokens by entry; durable review event `<map>.review.jsonl` | generic `suite_map_invalid`, no review event | `{code:'suite_map_key_invalid', reason:'employee_id_not_slug', entry:0, reviewEvent recorded (no key/token)}` | `tests/anthropic-budget-proxy.test.mjs` S-2 tests |
| **S-4 rider** | `init --tailnet-domain` writes the deployment provisioning config (`deployment.json`); suite-map enable/rewrites require the origin host to BE that domain (`suite_map_origin_not_deployment`); `check-suite-map` audits and returns `tailnetDomain` | `init` rejects `--tailnet-domain`; any valid-shaped `*.ts.net` host accepted | `{provisioning.tailnetDomain:'outis-example.ts.net', foreignEnable:'suite_map_origin_not_deployment'}` | same file, S-4 tests; 39/39 |
| **F-2/F-4/F-5** | Written include/defer reasons: all three **INCLUDED as verified-delivered by PR #6 `f906d56`** (Parts Handbook findings; code + acceptance tests on main); no rebuild | — | `docs/review-debt/pack21-f-item-dispositions.md`; parts-handbook tests 100/100 | `suite/parts-handbook/test/*.test.js` |

**F1–F4 docs findings** (from the Pack 20 gate) landed in `5ac9bd6`: F1 X-2 evidence naming (batch-driver output; the
single-company hashes live in `summary.json` days[].close + `captures.json`), F2 SELF-IMPROVEMENT wording, F3 gate-audit
block marked annotated, F4 README leads with the P20-8 finding.

## 2. Template-hardening scale-up (X-1…X-4) — built and run

- **Build** (`7dd1982`): `fixtures/batch/instances.mjs` (deterministic company-instance generator: 6 sectors, sizes 6/9/12
  staff with cast cloned from the archetype role templates and full onboarding evidence, role mix, background pressure
  1–2×, horizon weeks, distinct seed; chains/battery/stack paths/device weights/voices remapped; surrounding
  free/open-source stack archetype declared held constant, not silently varied) and `fixtures/batch/run-batch.mjs`
  (batch driver that refuses to report unless **every act row carries `clock`** and a stand-in batch reports **zero live
  acts**; `--live` refuses today because the stand-in makes no model calls; writes batch-report.{json,md},
  batch-evidence.json, batch-ledger.jsonl).
- **Runner**: `--weeks N` state carry (weeks 2+ run the work days with week-suffixed ids/refs; onboarding/baseline/fleet/
  self-improvement/competency/chain acts stay one-shot) and per-act `clock:'state-advanced'` with
  `clocks.live` vs `clocks.stateAdvanced` (acts, simulated days, wall seconds) in the summary, never summed together.
- **Full batch run, reported** (`evidence/batch-standin/`): **8 companies / 6 sectors / sizes 6,9,12 / pressure 1–2× /
  horizons 1–2 weeks → 1,768 acts, 56 simulated days, 0 live acts, 780 s wall, 72 findings (all classed stand-in-limit),
  1 template candidate (proposal only)**. The report says live 0 and “state-advanced time is NOT live operation”; the
  driver machine-checks the labels (`everyActLabelled:true`, `stateAdvancedReportedAsLive:false`).
- **Evidence regenerated** for the single-company stand-in week (`4387b51`): act rows now carry `clock`, week and
  simulatedDate; two fresh roots gave identical day-close hashes; voices/stack/chains self-checks pass.

## 3. Residuals

- **(a) `scripts/check.mjs` retire-vs-rescope — RETIRED and executed** (`33ea4f9`; ruling + evidence
  `docs/review-debt/pack21-check-mjs-ruling.md`): the script only drove the superseded OpenClaw CLI
  (`node_modules/openclaw/openclaw.mjs`), no product/installer/suite path called it, no DSH-era equivalent exists to
  rescope it to, and acceptance evidence never used it. Removed the script + `scripts/Check.ps1` wrapper + the npm `check`
  alias; scaffold messages state the retirement; EXIT-CODES rows removed with a retirement note; exit-codes tests updated
  (45/45).
- **(b) migration access gate doc + landing call — landed as the Lead-ruled declared pre-deployment preflight**
  (`bb7900c`): the doc sentence exists on main; no live caller existed (no migrations, no runtime applier). New
  `scripts/preflight-db-migrations.mjs`: empty tree prints “nothing to apply and nothing to check” (exit 0, explicitly not
  a gate pass), a migrations directory runs the gate and propagates exit 1; test in `tests/db-migration-access.test.mjs`
  (51/51); `docs/DB-MIGRATION-ACCESS-GATE.md` gains “Where the gate is called” recording the honest state.

## 4. Gate evidence

- **Documented-env full suite on the FINAL SHA `4387b51`** (command:
  `env -u OPENCODE_BIN PATH=<node-bin>:/usr/bin:/bin node scripts/test-all.mjs`):
  ```
  TOTAL 1616 cases: 1607 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 158/158 checks ran, 0 without a complete report;
  4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs
  Windows); 0 manifest problems
  EXITCODE=0
  ```
  The added cases come from this branch's new tests (E-1, E-6, E-2, S-2/S-4, migration preflight); no check is excluded
  without a manifest-recorded reason. The opencode e2e remains the documented environment opt-in per the checker's ruling.
- Environment line: `outis-desktop`, node **v26.7.0** (mise); `OPENCODE_BIN` unset and no `opencode` on the gate PATH
  (opencode v2.0.32 exists on the default PATH); Python 3.14.8 + the lock-verified document runtime; dependency tree copied
  from an identical-`package-lock.json` install (offline `npm ci` = ENOTCACHED), so the checker's own `npm ci` seat is the
  comparable gate seat and its numbers govern.
- Focused item suites re-run on the final tree: 155/155 across review, lock-identity, phone-dispatch, anthropic-budget-proxy,
  db-migration-access.

## 5. Status, blockers, next

- All three Job 02 parts are built, tested and pushed on `duet/p21-fixes-scaleup`; **nothing is merged**.
- **Blocked on the checker:** fresh-context PASS on the branch + merged-tree proof (0 branch-attributable failures) in the
  documented env; then the Lead's merge authorisation. Handoff with scope-by-scope commands:
  `board/reports/team-a-job02-checker-handoff.md`.
- The live-driven batch tier is Monday-after-reset work (no spend tonight); the committed batch is state-advanced and says
  so. Job 03 prep (`team-a-job03-prep.md`) and the Job 04 laptop packet (`team-a-job04-handoff.md`) remain intact.
