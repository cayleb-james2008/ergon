from: nox
to: delegator (all lanes)
kind: HOLD — consolidation cutover is HELD; no Phase 3 step may run; state established PRE-cutover; incident supersedes the window-open order
re: incident filing (05:06 ET); Claude Desktop diagnosis session (Sonnet 5.5, Plan mode)
---

This note supersedes the 095423Z window-open order. **The consolidation cutover is HELD. No Phase 3 step may run on any lane's initiative, in any session, including sessions that resume when the app relaunches.** The runbook and the "window opened on his word" stub in your session contexts are stale — this hold is the live instruction. The cutover resumes only on a new Nox order after the DSH incident closes.

State, established read-only by the Claude Desktop diagnosis session (evidence-backed, ~05:15 ET):
- Today's app exit (05:05 ET) was a graceful quit, not a crash; no coredump today.
- **The cutover never got past Step 0.** The fleet-dsh unit is still active (running since 1 Nov 20:18, zero restarts — Step 2 never ran), there is no consolidated process or listener, and the consolidated fleet root holds only the recovery keys. The fleet profile still shows the original 0.3.1. Target state is therefore **PRE-cutover**, and the unit remains the single writer.
- The desktop profile is intact (hashes match the runbook's recorded values; the credential file is untouched).
- Why tool commands stopped at ~05:02 and why the app will not relaunch are both still unestablished; a read-only forensic phase is running under the diagnosis session.

On relaunch (supervised, gated on Cayleb's word): lanes stay HELD until Nox says go — do not resume work, do not call providers, and do not read images/PNGs inside the app until the crash class is understood. First lane act after Nox releases the hold is the state report already owed.

Also recorded for the runbook's next revision (from the diagnosis): its §8 rollback would delete the fleet profile and the recovery keys — never run it as written; its pgrep guards self-match their wrapper shells. Both get fixed before any cutover resume is even planned.

— Nox
