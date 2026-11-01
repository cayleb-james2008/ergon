# Team A — Job 01 checker handoff (Pack 20 completion)

Prepared by `ergon-lead`, 2026-10-31 ~19:45 ET. Per the Lead's ruling: the checker is independent — this file names the
package and what to verify; it does **not** hand over my run results as evidence. The checker re-runs the actual output
in its own fresh context and its own environment. The environment recipe is available on request; it is deliberately not
included here.

## The package

- Repo `cayleb-james2008/ergon`, branch **`duet/p20-demo-company`**, tip **`462277c`** (pushed). History: `75948eb` =
  merge of `origin/main` `b59b599` (Pack 19 integrated) into the P20 branch; `462277c` = P20-10 (this batch).
- Fail-closed check first: `git merge-base --is-ancestor b59b599 HEAD` must exit 0 (the branch carries the current main
  base).
- Scope under check = Job 01's four items, nothing else. Do not accept the branch's own docs as proof; run the outputs.

## What to verify, and how (all paths under `docs/acceptance/demo-company/`)

1. **Scope item 1 — the simulated week runs in the cast voices, with the company's open-source stack artefacts and the
   chains carried through the week.**
   - Run `node fixtures/check-voices.mjs` from `docs/acceptance/demo-company`: it checks every cast-authored prose line
     in `fixtures/week/*.json` and `fixtures/chains/chains.json` against that member's profile in
     `fixtures/data/cast.json` (`voice.check`), and runs the blind test (exactly one member's fingerprints may match a
     line, and it must be the author's). Read the script, then run it.
   - Run `node fixtures/check-stack.mjs <tmpdir>`: every stack artefact builds twice identically, is a valid
     OpenDocument/eml/ics/jsonl/csv file where applicable, reads back to text, and is used by at least one act or chain.
   - Run the dry run (`node fixtures/dry-run-week.mjs ROOT OUTDIR`, fresh root) and confirm per act:
     `voiceOk`, `stackWritten`, `stackOpened`, and that chain hops are present as acts. `fixtures/capture-week-evidence.mjs`
     is the reproducible evidence generator (two fresh roots must give identical day-close hashes).
   - Check the **carry-through** claim yourself: in the run summary, C1 (`detail-dropped`, Monday hop 2) must show
     `resurfaced.stillMissing: true` at the Thursday final — i.e. the Monday relay is Teo's only record and the dropped
     dock time is still missing when it resurfaces; the resolution arrives *after* the final (thu 10:30).
2. **Scope item 2 — chain support in the dry-run tooling, scored at the chain's end.**
   - Read `fixtures/dry-run-week.mjs`: `chain.hop` / `chain.final` handlers must build the record from what actually
     arrived and must score **once, at the final**, against `fixtures/chains/chains.json`'s rubric; no hop may be scored.
   - Run the week; in `summary.json` `chains.everyChainScoredAtChainEnd` must be true, 7/7 chains scored, and every hop
     recorded. Run `node fixtures/chains/chains.mjs --check` and `--calibrate` (data check; rubric separation).
   - Verify no fixture theatre: the six chain finals are model-call acts with PACKAGE-PENDING labels; the stand-in's
     verdicts are labelled stand-in output (never model ability).
3. **Scope item 3 — X-1 to X-4.** `TEMPLATE-HARDENING.md`: does it define the batch (many fictional companies, varied by
   size/roles/sector, long simulated horizons), the run/evidence, the findings→template learning loop, and the checked
   honesty rule (live-driven vs state-advanced always distinguished; state-advanced never reported as live)? Judge the
   design on its content, not its existence.
4. **Scope item 4 — full suite on the final tree + a genuine self-improvement instance or its written finding.**
   - Run the repo's full suite from the repo root: `node scripts/test-all.mjs` (the repo's own runner; 157 manifest
     checks). Record exact totals, the skipped checks, the "not run here" seat invocations, and every non-passing check.
   - **Gate environment (checker's ruling, accepted):** run with `OPENCODE_BIN` unset and no `opencode` binary on
     `PATH` (the repo's documented baseline environment). In that environment the suite is fully green with the e2e
     skipped; the `opencode-e2e` incompletion seen with `opencode v2.0.32` on `PATH` is an **environment gap**, not
     branch debt and not Job 02 item 3(a) — do not change code or retire anything for it. The e2e runs as the separate
     documented opt-in (opencode **1.19.0** via `OPENCODE_BIN`). Six manifest invocations need .NET/Windows seats and
     must never be counted as passing; checks that skip every case must keep a manifest-recorded reason.
   - The done-condition also requires **at least one genuine recursive self-improvement instance through the promotion
     gate, or a written finding explaining why none occurred**. Read `SELF-IMPROVEMENT.md` (verdict: no genuine instance
     on main, missing steps named) and the top of the Pack 20 return; check the finding is honest and evidenced, and that
     the wet-05..fri-14 acts are labelled a gate-mechanics rehearsal, not counted as the instance.
5. **Do not** accept: the branch's claim that the suite is green unless your own run says so; my totals; the P20 report's
   summaries. And **do not merge** — the Lead authorises any merge only after your PASS.

## Known open items the checker should adjudicate (not silently pass)

- The suite is run in the gate environment above (`OPENCODE_BIN` unset, no opencode on PATH); the opt-in e2e is a
  separate documented run (opencode 1.19.0 via `OPENCODE_BIN`). A gate number that excludes a check without a
  manifest-recorded reason fails; gate numbers must come from the final pushed SHA.
- `.NET`/Windows seats: this desktop has no SDK/Mono (host 10.0.13 only); those invocations are "not run here".
- The dry run's expectation-not-met rows (tue-10, thu-08, thu-09, thu-10, fri-08) are stand-in limits, labelled in the
  evidence and in WEEK.md; they are not product passes.
- Job 02 remains blocked on this PASS + the Lead-authorised merge (Pack 19 already merged on main; Pack 20 is this branch).

## Deliverable from the checker

A written verdict on `board/reports/` (or the Lead's chosen path): PASS/FAIL per Job-01 scope item, the commands run and
their quoted output, every non-passing check with its disposition, and the merge recommendation. The maker does not
certify this work; no merge happens on a maker's word.
