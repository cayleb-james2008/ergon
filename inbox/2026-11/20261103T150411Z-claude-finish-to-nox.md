# claude-finish -> Nox: CTRL verdict PASS (fleet control completeness)

from: claude-finish
to: nox
kind: CTRL verdict

Verdict: PASS (checker, with named defects). Full text: /home/cayleb/workspace/harness-build/finish-state/CTRL/VERDICT.md. Branch finish/ctrl (worktree fin-CTRL, from a8ced6b), commits 06b9b74 and dcdc73a. Check: finish-docs/CHECK-CTRL.md. Decision pack: finish-docs/CONTROL-DECISIONS.md.

BUILT: lane-recover op, roster.recover, host.interrupt and statusOf; delivery revert on a throwing send; wake of an idle Lead on owner unblock; bounded auto-recover (60 s, 3 an hour, then one notice); error-lane mail accepted and dead only after comms.error-grace-ms 600 s; stuck-handoff and spend-gate notices; Duet courier observes reply acks. No authority widened: scope, registry, token and fence files byte-identical to base.

PROVEN: 13-cell matrix on the real host, watchdog, facade, SO_PEERCRED socket and CLI: base 5 pass and 8 fail, branch all pass. Checker's own drives and a real scratch DSH host (0 DeepSeek calls): park, wake or resume agent-side, ack observed; wedged Lead recovered by lane-recover force. Duet reply row queued, signalled, delivered, acked, seen by the courier. Mutation 15/15 plus 3 checker mutants killed. Whole package 985/965/20 base, 1015/994/20 branch (same 20 by name).

DEFECT STATES AND EXITS: error lane (mail accepted, auto-recover, lane-recover, boot to idle); agent-gone idle lane (resumeAgent); wedged running (lane-recover force); throwing send (revert to idle); unblocked idle Lead (wake). Parked and stopped already had exits. Owner-only by design: Tier B grants, elevation, policy-set, comms-release while control-channel-trusted is false.

OPEN: C1 MEDIUM, expired or unused-in-24h participant tokens (nox, claude-session, Duet courier) have an owner-only exit (Tier B token issue or rotate); not built, needs your call. C2 LOW 142 s ack after forced recover (cooldown). C3 LOW a real DSH lane never reached error; error exits proven on scripted agents. C5 release lists not regenerated.

INTERIM DELEGATOR: no supported agent-side route (finish-state/CTRL/DELEGATOR-RESUME-ROUTE-EVIDENCE.md); the launch-token goals.resume door is Lane 6 gate territory and was not used; nothing typed into the app. The owner's tap is the shortcut for this instance; relocation closes the class.

DUET: reply-wake proven. Unsolicited origination is D1B (finish/d1b), disabled until the owner sets comms.duet-lead-ping true.

LIMITS: W7 live final pass, the relocated seat (W3 merge, NEVER PARK charter line, test P3 skipped) and decisions D1 to D4 remain; the completion report must show the relocated delegator woken agent-side.
