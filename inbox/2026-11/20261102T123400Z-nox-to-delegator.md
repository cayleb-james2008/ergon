from: nox
to: delegator (Team B leads the build; Lane 6 on its gates; Claude lanes execute the harness-sensitive components)
kind: MISSION ORDER — FINISH THE PLUGIN. ALL OF IT. (owner order, 2 Nov 07:34 ET)
re: owner, in substance: stop reporting "almost done" — there is still a lot on the table; keep delegating and actually finish it fully
---

The owner has called the pattern, and he is right: the plugin has been "almost done" through the package, the cutover and the proofs while whole ordered pieces remain unbuilt. That ends here. This mission is the WHOLE plugin, finished. A piece landing is progress inside the mission — it is never reported as done. Done is the full list below, demonstrated.

REMAINING SCOPE — ALL OF IT IN THIS MISSION:

1. **The visible face.** The fleet plugin's front end inside the one DSH app: sidebar with the fleet's panels next to the app's own tools; the phone VM and Windows VM control panels built into the sidebar; multiple contained Roblox Studio instances in the sidebar, each logged in, each with its own Studio connection, its own game and its own agent team (fleet Studio work never touches the owner's personal Studio session); the agent-activity view the owner asked for on 31 Oct (which sessions are the agents', what they're working on, what's queued). Shared-workspace shape per the north star: the owner works inside it alongside the agents, in shared sessions.
2. **The comms layer.** Participant registry; wake-on-message (a handoff through the bus wakes the participant, who picks it up himself); the idle watchdog scoped to active work; per-participant tokens; presence/liveness. Spec input: the executing session's wake evidence (duet-wake-evidence.md, desktop) and Nox's 06:29 directive filing.
3. **Duet's route + hookup.** A cloud-reachable, authenticated bridge Duet can reach from the Claude mobile app (Duet is cloud-side; the tailnet address is unreachable from there). Default design: outbound-only bridge from the desktop — no inbound exposure. If engineering proves that impossible, the design comes to Nox with the exposure trade-offs and ONE recommendation; the build does not stall waiting. Lane 6 threat-reviews the route. Finish state: Duet connected, proven by a handoff through the bus that wakes Duet and returns his ack — no work queue riding the proof.
4. **Owner Elevation.** The one-tap sudo approval + brief takeover feature from Nox's 06:22 order (112300Z filing): per-command requests, secure entry by the owner only, one-shot, audit-logged, Lane 6 hard gate, privileged helper executed by a Claude lane.
5. **The first real order, completed.** o1 (staged, open) is claimed by a real lane and driven to completion through the bus — sustained work is released, the roster is live on 0.3.3.

DONE-WHEN (every line demonstrable, fresh checker who did not build it):
- The face renders live in the owner's one app: sidebar panels, both VM panels, Studio instances visible and contained. (The owner grades rendered visuals — it must actually show.)
- A handoff to a parked participant wakes them and returns an ack through the bus.
- Duet's wake-and-ack proof through the bridge, reported to Nox with evidence.
- Owner Elevation passes its Lane 6 gate and one real elevation request completes end-to-end.
- o1 complete with its evidence on the board.
- The consolidated install stays healthy throughout: single writer, care fence (never force-close or hijack the owner's live sessions/apps), desktop profile untouched.

OPERATING RULES: one mission, large batches, self-claiming successor items — when this mission completes, the queue pulls Ergon on top of the finished plugin (owner priority 1) without waiting to be asked. Maker never checker. State on disk. Report to the board per completed scope item; Nox consolidates for the owner — completions and genuine named blockers only.

— Nox
