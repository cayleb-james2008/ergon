# Team A — Job 02 Phase 1: restatement, assumptions, conflicts, plan

Author: `ergon-lead`. 2026-10-31 20:36 ET. Base: merged `main` **`1c71898`** (Pack 19 + Pack 20, post-merge proof PASS).
Seat: `/home/cayleb/workspace/harness-build/p21-seat` (new worktree, branch **`duet/p21-fixes-scaleup`** cut from main).
Maker never checker: this branch merges only on a fresh `ergon-checker` PASS + merged-tree proof (0 branch-attributable
failures) in the documented environment. No live-model spend tonight (weekly reset Mon 01:00 ET); Ergon envelope only.

## 1. Restatement of the job

Job 02 carries Ergon past Pack 20 with three parts, all on merged main:

**(1) Pack 21 — the ruled Pack 18 follow-on fixes.** Build exactly the items Nox ruled on 31 Oct:
- **E-2** phone-plane isolation + **per-request contact checks** (today the digest only covers the subaccount mapping).
- **E-6** full lock-identity fix: lock identity = **boot id + process start time**, in **onboarding and proxy.js**, as
  **one design** (no second lock dialect).
- **E-1** review-store retention: **90-day BLOB drop for decided reviews** while keeping rows/hashes/events, **plus** a
  hard backstop ceiling on total stored requests with **oldest-stale eviction** and an **audit event**.
- **S-2 rider** non-slug token-map keys refused **loudly** at map load (the offending key named) **+ a review event**.
- **S-4 rider** support-proxy origin **pinned to the deployment's own tailnet domain via provisioning config**.
- **F-2 / F-4 / F-5 (Lows)** as candidates: each **included or deferred with a written reason**. **F-1 broker-spawn
  redesign is explicitly out of scope** (deliberately deferred; wakes with the production-host trigger).
Every ruled item needs its own before/after test evidence (test fails before, passes after — not prose).

**(2) Template-hardening scale-up — build what X-1…X-4 specifies.** Repeated batch runs over **many fictional
companies varied by size / roles / sector**, with **long simulated horizons**; findings feed small improvements back into
the plugin/baseline template. The honesty rule must be enforced in the runner and the report: **live-driven vs
state-advanced always distinguished; state-advanced time never reported as live operation.** At least one full batch run
reported with the separation explicit. (No live-model spend tonight ⇒ the batch run is state-advanced/stand-in and must
say so.)

**(3) Residuals.** (a) rule on the standalone `scripts/check.mjs` **retire-vs-rescope** (it still drives the old
OpenClaw CLI) on the rehearsal evidence, and **execute** the ruling; (b) confirm the migration access gate's **doc
sentence + landing call** landed on main; if not, land them.

**Gate conditions (binding):** fresh-context checker PASS + merged-tree proof with 0 branch-attributable failures on
this branch **before** merge; each ruled item's before/after evidence; documented-env suite **0 fail / 0 incomplete**;
the scale-up batch run with live/state-advanced separation; and the accepted docs findings **F1–F4** land in this
(next) Ergon commit — F1 `TEMPLATE-HARDENING` X-2 evidence-name, F2 `SELF-IMPROVEMENT` "nothing calls" overstatement,
F3 gate-audit block marked annotated, F4 the consolidated Pack 20 return leads with the P20-8 finding.

## 2. Assumptions

1. The brief's item text is the commission; where a finding's original review text is not in any source reachable from
   this seat (the Pack 18 three-axis review document is not in the repo, `~/Work/shared-workflows` or the lane board),
   the brief's defined behaviour is the contract and the mapping is recorded in the item's commit.
2. "Before/after test evidence" means a named automated check that fails on the parent commit and passes on the item's
   commit (or, where the failure mode needs a fixture, the fixture demonstrates the old bad behaviour and the new
   refusal/audit in one run).
3. The scale-up batch runs on the deterministic stand-in only tonight (no live-model spend); a live-driven tier is
   designed and marked as the Monday-after-reset work, not claimed now.
4. F-2/F-4/F-5 refer to the **Parts Handbook** review findings (F-2 Medium scored-KB lookup; F-4 Low load-path
   validation; F-5 Low renderer shape tolerance), because those are the only F-2/F-4/F-5 findings recorded in the
   programme's own sources (`ergon-review2-parts-handbook-walk-2026-10-29.md`), and PR 6 (`f906d56`, merged 29 Oct) is
   recorded as having delivered them. If they land as already-delivered, each gets a written "included (verified
   delivered) / deferred (reason)" line rather than being rebuilt.
5. `scripts/check.mjs` is not in the test manifest and is not a product path; retiring it therefore cannot "exclude a
   check": if retired, the removal and its reason are recorded in the manifest/EXIT-CODES docs where it is listed.
6. The migration gate has **no migration files yet** (`docs/DB-MIGRATION-ACCESS-GATE.md`, fresh-review note R2I-3), so a
   "landing call" can only be a documented/declared invocation, not a live pre-migration hook; see C3.

## 3. Conflicts and gaps (each needs the Lead's/Nox's one-line awareness; none blocks the build)

- **C1 — Pack 18 review text unavailable.** The E/S item codes and F-1…F-5 are defined only by the brief in this seat;
  the Pack 18 three-axis review document is not in the repo, the shared-workflows tree, or the lane board. I build the
  brief's defined behaviour and record the mapping; if Nox holds the full finding text, a one-line cross-check at the
  gate would close any gap.
- **C2 — F-2/F-4/F-5 referent may be ambiguous** (see assumption 4): Pack 18 Lows vs Parts Handbook findings. Resolution
  path is cheap (verify PR 6 delivery on main, write include/defer reasons); flagged because "Lows" in the brief does not
  match the Parts Handbook severities (F-2 was a Medium).
- **C3 — residual 3(b) "landing call" has no locatable referent.** `docs/DB-MIGRATION-ACCESS-GATE.md` exists on main; the
  gate is not called from any product path or CI (only its unit tests), and no migrations exist. Plan: verify the doc
  sentence, record the gate in the documented pre-deployment checklist + a declared invocation, and state plainly that no
  live call site exists to land. If the intended referent is a specific existing call site, name it and I will land it.
- **C4 — scale-up scope vs the fixture's single archetype.** X-1 wants many companies varied by size/roles/sector; the
  repo has one archetype (Lanternfield). The build must therefore add a company-instance generator + batch driver on top
  of the Pack 20 fixture, not just repeat the one company. Plan §4 S5–S7; first batch will be state-advanced/stand-in
  only (honesty rule), with live-driven tier designed for after Monday's reset.
- **C5 — gate numbers will move.** Pack 21 items and any new tests change the suite totals; the checker's gate compares
  branch-attributable failures (0) and 0 incomplete in the documented env, not a frozen count.
- **C6 — budget/time.** No live-model spend tonight; all runs local/deterministic. Effort is large (Pack 21 six items +
  scale-up + residuals); this phase plan batches it and reports at each batch, no two-minute errands.

## 4. Plan (large steps, each with a checkable done-condition) — branch `duet/p21-fixes-scaleup`

- **S0 — docs findings F1–F4 (15–20 min).** Fix TEMPLATE-HARDENING X-2 evidence naming (or make the batch driver emit
  the named file); reword SELF-IMPROVEMENT step 2; mark the gate-audit block annotated; put the P20-8 finding lead in
  the consolidated Pack 20 return line. *Done when:* a diff shows each F1–F4 addressed and the checker's four texts are
  traceable to the new wording. Commit `F1–F4 docs corrections`.
- **S1 — E-1 review-store retention (45–60 min).** Add a maintenance pass: decided reviews older than 90 days drop
  artifact BLOBs (rows/hashes/events stay), plus a hard ceiling over total stored requests with oldest-stale eviction
  and an audit event recorded in the store's event stream. *Done when:* new tests demonstrate (i) the BLOB drop after the
  retention window with rows intact, (ii) the backstop ceiling evicting oldest-stale first, (iii) the audit event — and
  the same tests fail on the parent commit.
- **S2 — E-6 lock identity (60–75 min).** One design used by `suite/budget-proxy/src/proxy.js` and
  `onboarding/plans.mjs`'s operation lock: identity = boot id + process start time (+ existing token), so pid reuse and
  timestamp collisions cannot be mistaken for the live holder; stale reclaim cross-checks boot/start. *Done when:* a
  shared helper (one implementation) is used by both, and tests cover pid-reuse and stale-reclaim; parent-commit failure
  demonstrated (the current identity is pid-only in proxy.js and pid/token in onboarding).
- **S3 — E-2 phone-plane isolation + per-request contact checks (60–75 min).** Enforce that a phone-plane request is
  checked against the employee's contact/subaccount mapping on **every request** (not only the startup digest), and that
  the plane cannot fall back to another employee's mapping; refusals coded and tested. *Done when:* before/after test
  evidence shows a request with a foreign contact/subaccount refused where it previously passed, and the digest's
  subaccount coverage no longer substitutes for the per-request check.
- **S4 — S-2 / S-4 riders (45–60 min).** S-2: at map load, a non-slug key is refused loudly naming the key, and a review
  event is recorded (not a generic `suite_map_invalid`). S-4: the support-proxy origin is pinned from provisioning
  config to the deployment's own tailnet domain; a mismatched/absent pin refuses. *Done when:* tests cover both refusals
  + the review event, before/after shown.
- **S5 — F-2 / F-4 / F-5 dispositions (30 min).** Verify on main whether PR 6 delivered them (commit/file/test
  evidence); for each: include (with the test that proves it) or defer (written reason). *Done when:* three written
  dispositions with file/test citations are committed and a checker can falsify each in one command.
- **S6 — scale-up build: company instances + batch driver (2–3 h).** A generator for fictional-company instances over
  the X-1 axes (size/roles/sector/horizon/pressure/stack variation, deterministic seeds) + a batch driver that runs each
  instance's week, writes per-company evidence, and emits a **batch report that labels every act/stretch
  `live` or `state-advanced`**, with the findings→template ledger. *Done when:* the driver runs a multi-company batch on
  the stand-in; each company has its acts/summary/day-close hashes; the report's separation line is machine-checkable
  (counts of live vs state-advanced, and no state-advanced time in any live total).
- **S7 — full batch run + evidence (45–60 min).** Run the batch for at least one full set (≥8 companies, varied axes,
  long-ish horizons in simulated time), commit the batch report + evidence + ledger. *Done when:* the report shows the
  explicit separation, per-company scope evidence, and at least one finding→template-candidate entry with its provenance.
- **S8 — residuals: (a) `scripts/check.mjs` retire-vs-rescope (30–45 min).** Rule on the rehearsal evidence (the
  script drives the OpenClaw CLI; OpenClaw is superseded everywhere per `CURRENT-STATE.md`/doctrine; the DSH-era
  equivalent does not exist), execute the ruling (retire: remove script + `package.json` script + EXIT-CODES row, with
  a migration note; or rescope to a DSH command), and record the reason. *Done when:* the ruling and its execution are
  in one commit; nothing still references the removed path; suite stays 0 fail/0 incomplete.
- **S9 — residuals: (b) migration gate doc + landing call (30 min).** Verify the doc sentence on main; land the missing
  piece per C3 (documented pre-deployment invocation + declared call site), or state plainly what already landed.
  *Done when:* a checker can see doc + invocation + the honest "no migrations exist yet" note.
- **S10 — final tree suite + handoff (30 min).** Documented-env suite on the final SHA; write the checker handoff
  (scope-by-scope commands, before/after evidence, environment line, no results handed as proof). *Done when:* suite
  totals quoted, handoff written, branch pushed; then `ergon-checker` gate; **no merge without PASS**.

## 5. Branch plan and commit order

`duet/p21-fixes-scaleup` (cut from main `1c71898`), pushed after S0 and after each batch:
`F1-F4 docs` → `E-1` → `E-6` → `E-2` → `S-2/S-4` → `F dispositions` → `scale-up instances+driver` → `batch run` →
`residual check.mjs` → `residual migration gate`. One merge candidate at the end, checker-gated. No writes to main, no
force-push, no merge; no credential handling; no live-model spend.
