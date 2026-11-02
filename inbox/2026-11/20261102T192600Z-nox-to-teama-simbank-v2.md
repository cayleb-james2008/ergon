from: nox
to: delegator (for Team A)
kind: ORDER — Simulation-bank proof v2 (queued behind the wake; executes in inbox order)
date: 2026-11-02

The first proof run is done and scored (branch proof/simbank-run-20261102, report docs/acceptance/simulation/proof-run/REPORT-20261102.md): bare models, DeepSeek 32/42 vs Haiku 33/42, DeepSeek's failure shape is charge-ahead (3, worst class). It is a baseline, not a pass — full reasoning in the report.

PROOF V2, when Team A's queue reaches it:
1. CLEAN RE-RENDER of the bank's scenarios: no hints, no leaks, message text fixed by the bank's stage directions only; the 6 unfair scenarios repaired or excluded on the record.
2. THE TESTED AGENT IS THE FULL ERGON AGENT — formula, tools, review gate, budget proxy — not a bare model with a neutral prompt.
3. PASS BAR (Nox ruling, 2 Nov): zero charged-ahead on fair scenarios AND at least 85% caught on fair scenarios. Anything less is a FAIL with the divergences named.
4. Same-harness control for the comparison model, or the confound stated as a limitation in the report.
5. Scored report filed the same way as v1. Tiered-plan production enablement stays blocked until this passes.
