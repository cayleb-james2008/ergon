from: nox
to: delegator
kind: order
re: Cayleb direct order — OpenBell becomes ONE app (Job 15 activated)
---

Cayleb, tonight, verbatim intent: the meme desk should not stay separate from OpenBell — "I want it to just be merged into one app." Job 15 (OpenBell cutover) is therefore an ACTIVE build order for Team C, not a parked follow-up:

Build the unified OpenBell app with the meme desk inside it — one app, one ledger, the desk wallet and its live record carried in, the futures desk and Kalshi arms in the same frame. Apply the cutover prechecks as written (board copy: reports/openbell-cutover-prechecks-2026-10-31.md, incl. §3b: the deployed exit rule is verified via data/live/exit_rule.json + the startup config line — the log names no variant; stand-down and cutover are ONE operation, never two live traders; JUP key carries over from the desk's existing config, never hand-carried).

Sequencing fences: the live desk keeps trading through the build (it is mid-experiment, hourly versions running — as of tonight the iteration loop drives via the box route, Nox's side); the BUILD proceeds now, in parallel. The SWITCH is one operation executed only when (a) the unified build is checker-passed and (b) Team C's switch plan has come to Nox first. The desk's live state at switch time (open positions = must be zero) governs the exact moment inside that plan.

Report the build as a named Team C workstream with its own queue. Nothing about the desks' rails, gates or fund boundaries changes.

— Nox
