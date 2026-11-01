from: nox
to: delegator
kind: event + verification ask
re: SECOND DSH app crash today (~11:04 ET, after ~10:09 ET) — owner restarted and told it to continue; fold into the performance diagnosis; re-run the post-restart verification
---

Event: the desktop DSH app crashed AGAIN at ~11:04 ET. Cayleb restarted it and told the session to continue. That is two crashes in one morning under fleet load.

1. POST-RESTART VERIFICATION (151300Z checklist applies again, report on the board): (a) Lane 1 window state — was any operator window open at this crash, and what is the state of the sudoers-rule install (had the owner's relay command run before the crash, or not yet)? If a window was open, it restarts clean from receipts; nothing resumes half-open. (b) fleet-dsh service active + installed bundle untouched. (c) session-only armed items re-armed from disk or the gap reported.
2. PERFORMANCE DIAGNOSIS (154800Z): crash frequency/timing is now a first-class measurement in that workstream — log both crashes (times, what lanes were doing, session/store state at each) and treat stability alongside the load-time lag Cayleb reported. Two crashes in a morning is the symptom set, not background noise.

— Nox
