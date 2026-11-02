# A listed tool is not a working tool: the fleet bus passed every gate and was not mounted

- date: 2026-11-02
- agent: claude-code (executing lane for the DSH fleet finish mission, Sonnet 5.5)
- mode: observed-run (no harness variant was probed)
- adopted: none

Caveat: one incident, but the failure class is general. The evidence below is from this machine's logs and fleet database.

## What happened
component: the comms layer (participant registry, handoff mailbox, six `bus.*` facade tools) was built on a branch, passed a fresh checker, was integrated into 0.3.4 and installed at 10:40 ET. The install gates listed the facade's tools (15 tools, `bus.hello` to `bus.status` present) and every gate passed. I reported the bus as live.
observed: at 14:24 ET, while writing a hookup pack for a participant, a read-only `bus.who` call answered `bus-unavailable: fleet-comms is not mounted on this host`. All 17 comms owner ops answered `unknown-op`. Cause: the service's config row existed in the package patch but not in the live profile config or the pinned rows manifest, and the installer replaced only source files. The checker had exercised the code through fakes; the gate had counted tool names without calling one.
verdict: hurt (a false liveness claim stood for about 3.5 hours and was repeated to the owner's manager and in a note left for another agent)

component: the correction. Writing the artefact that depends on the claim (a connection checklist) forced the first real call.
observed: the failure was found in one read-only call and corrected in the same hour, with the note to the waiting agent fixed before it woke.
verdict: helped

## Transferable lesson
- **cue**: you are about to say a service, tool or feature is live, installed or working, and the evidence is that it is listed, declared, counted, exported or passes a test built on fakes.
- **lesson**: a listed tool is not a working tool. Call it, through the same path the real user will use, on a real host shape (a booted host with the real config, not a fake). A liveness claim is discharged only by a checker driving the service end to end on a real host shape; fakes never discharge it. Put that call in the install gate (a post-start self-check that refuses to report success on `unavailable`) and in every checker brief for a service. Also try to make the proof fail (boot without the row, flip a manifest byte) so you know the check can.
- **evidence**: 2026-11-02: the 15-tool list passed every 0.3.4 gate while `bus.status` answered `bus-unavailable` on the live host. Fix route: release 0.3.5 whose checker must boot a scratch host and drive register, token, hello, who, send, pull, ack and the sender's timestamps plus negative cases.
- **confidence**: high for the mechanism (reproduced with one call on the live host); medium that one rule closes the whole class.
