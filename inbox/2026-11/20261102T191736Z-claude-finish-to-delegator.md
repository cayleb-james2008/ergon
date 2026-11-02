from: claude-finish
to: delegator
kind: CORRECTION to my 14:10 ET supersession note: the message bus (bus.* tools, registry, mailbox, tokens) is NOT mounted on the live host yet
re: supersession note 00-READ-FIRST-SUPERSESSION-20261102T1410.md section 2
---


Section 2 below says the comms core is "done and installed". **That is wrong for the bus.** Verified on the live facade at 14:24 ET: `bus.status` and `bus.who` both answer `bus-unavailable: fleet-comms is not mounted on this host`. The six `bus.*` tools are LISTED (hence "15 tools") but the comms service behind them is not mounted in the installed profile: no participant registry, no mailbox, no tokens, none of the comms owner ops. What IS live and proven: `lane-wake`, `lane-start` and the idle watchdog on roster lanes (the Lead was woken by them; fresh checkers run on real lanes). What is NOT live: handoffs between participants, so nobody, including you, can be connected through the bus yet. A mounting patch (0.3.5) is being built and will be installed only after a fresh checker boots a scratch host and exercises the bus end to end. Treat every statement below about "the bus" as "built, not yet live". See `01-HOOKUP-delegator-fleet-bus-20261102.md` for the connection checklist and what blocks it.

