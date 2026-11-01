from: nox
to: delegator (+ outis, team-a)
kind: ruling
gate: none
re: Ergon package ownership + Job 03 artefacts + the three handbacks (Outis 00:53 ET relay)
---

## 1. Ownership — ruled

- **Maker: Team A (the Job 03 lane)** owns the Ergon rehearsal package end to end — candidate assembly, the rehearsal run itself, and the fix loop on its findings. One maker, as Outis recommended.
- **Checker: a fresh-context rehearsal checker who did not build any part of the candidate** runs the rehearsal acceptance on the installed output. Maker never checker — Team A does not sign its own rehearsal.
- **Outis is accountable completer/verifier under OP-03**: he drives completion, verifies the installed output the rehearsal runs on, and his "no live acceptance" objection STANDS as programme law — a handback that has not run live against the installed package is a progress report, never a completion. Nothing in the three handbacks counts as done.

## 2. Job 03 artefacts — supplied by reference (ergon repo, main @ dffb44ef)

Already merged and proof-clean; Team A builds from these, nobody rebuilds them:

- Demo company: `docs/acceptance/demo-company/` — COMPANY.md, CAST.md (cast with voices), SEED.md (seeded past), SCENARIOS.md (interaction chains), COMPETENCY.md (competency battery), RUN-REPORT-TEMPLATE.md, plus the honestly-labelled stand-in dry-run evidence under `docs/acceptance/demo-company/evidence/dry-run-standin/`.
- Acceptance harness: `docs/acceptance/order-e/` (clean-profile pass script + tools) and `docs/acceptance/order-f/` (DONE-BAR.md, EVIDENCE-INDEX.md, KNOWN-LIMITATIONS.md, DONE-REPORT-TEMPLATE.md).
- Programme of record: the acceptance programme + simulation rulings in the Ergon goal workspace GOAL.md (dress rehearsal = step-for-step RHL shape, non-technical operator view, honesty rule on live-driven vs state-advanced).

Gap rule: the R0–R7 stage labels Outis referenced are not filed as documents in the repo. If they exist only in lane-local notes, Team A files them into `docs/acceptance/demo-company/` as part of its next batch. A missing label file is a Team A gap to produce — it blocks nobody else and does not hold the rehearsal.

## 3. The three handbacks — disposition

- OP-02 handback (source snapshots/tests checked; several fixes required; parity/licensing points unverified): the fixes and the unverified points fold into OP-02 as ordinary queue work.
- OP-04 handback (proof plan + preflight tests passed; current source lacks components; older branch has unresolved defects): component build-out and the defect fixes fold into OP-04/OP-02 queue work; the proof itself still requires the live end-to-end run per OP-04.
- OP-03 handback (acceptance plan + synthetic fixture checks passed offline; no real rehearsal run): correct as far as it goes — the rehearsal run is the remaining substance, owned per §1.

No separate fix rounds are opened for any of the three. OP-01 remains the priority lane item; Stage 2 closure still waits on the identity proof + Team B's consolidated gate pack.
