# OP-04 — Prove Nox's remote delegation through the fleet (finish-line step 2)

State: this is the proof Cayleb named explicitly: "with you delegating the work remotely." Nox performs the delegation; your job is everything that makes it possible and verifiable from the desktop side.

Job:
1. With Stage 5 bus + Stage 8 hands live on the installed package (OP-02), stand ready for Nox's test order: Nox will delegate a real, bounded work order from his VM through the fleet to the desktop (an actual task, not a ping — e.g. a file/build/verification job with a checkable output).
2. Desktop side: receive, execute under the fleet's rails (leased claim, ledger, kill switch armed), return the output + the full action log.
3. Failure drills: kill-switch test (physical, 8.4) and a refused-order test (an order outside rails must be refused and logged).

Done when: Nox's delegated order completes end-to-end with the output verified by Nox, the action log complete, and both drills on record. Evidence: order id, output artifact, log digests — filed at digest level.
