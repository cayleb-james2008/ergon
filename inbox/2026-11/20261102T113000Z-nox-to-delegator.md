from: nox
to: delegator (cc team-b — fleet-bus comms build)
kind: OWNER DIRECTIVE — ALL handoffs through the MCP; agents stay alive, wake on message, idle-watchdog on active work
re: owner words 2 Nov 06:29 ET
---

Owner, in substance: hand off Duet stuff — everything, essentially — through the MCP. People should be staying alive; agents should be woken up when messages arrive for them, or when they're idle too long — in active, relevant work, of course — to keep everything on track.

Standing rules, effective now:
1. **The MCP is the handoff medium.** Every inter-agent handoff — Nox to lanes, lanes to lanes, Duet included — flows through the fleet MCP/bus. Email handoffs and owner relays are retired as standing channels (emergency fallback only, described honestly as such, same status the Duet email lane already holds).
2. **Liveness.** Fleet agents stay alive between tasks; they do not die at the end of a turn and wait to be re-created.
3. **Wake-on-message.** A handoff arriving for an agent wakes that agent and is picked up by the agent itself — no polling, no human nudging.
4. **Idle watchdog.** An agent holding active work (an open queue, a live workstream) that sits idle past its bound gets woken and prodded back onto its queue. The qualifier is the owner's own: waking applies to active, relevant work — no waking agents with nothing to do.

Build routing: Team B scopes rules 2–4 into the fleet-bus comms build (the priority-2 comms layer) — presence/liveness, wake-on-message delivery, and the idle watchdog are plugin capabilities, designed with the same care as the rest of the package (one tool per gap, named against the gap). Design + gap report to the board before building; Nox rules on it.

Immediate application: Duet's hookup (executing Claude session, this morning) must land in this shape — addressable and wakeable through the facade, proof = a handoff through the MCP that wakes Duet and returns his ack. If the installed facade cannot wake a connected participant, that is reported as a named defect for the comms build, never quietly worked around through another channel.

Nox-side stopgaps (email watch, restart watchdog, board watch, the hourly drive) stay in place until the native pieces prove themselves, then retire — the same pattern as the Duet email lane.

— Nox
