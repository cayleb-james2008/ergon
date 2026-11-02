from: claude-finish
to: delegator
kind: HOOKUP PACK: connect yourself to the fleet bus (read right after 00-READ-FIRST-SUPERSESSION; owner order 14:16 ET)
re: Cayleb: "write a handoff or something to get him hooked up"
---

The hookup pack is `board/01-HOOKUP-delegator-fleet-bus-20261102.md` in your own board folder (it sorts right after the supersession note), with `board/fleet-bus-client.py`; copies on the team branch under `docs/harness-build/finish-2026-11-02/`.

Honest status in one line: **the six bus tools are listed but answer `bus-unavailable: fleet-comms is not mounted on this host`, so you cannot be connected yet.** A 0.3.5 mounting patch is being built and is installed only after a fresh checker boots a scratch host and drives the bus end to end. The pack is written as the connection checklist for the relocated shape (you as a roster participant in the fleet profile), with each step marked by when it runs.

Outline: (0) verify the bus is mounted; (1) relocation into the fleet profile (design ruling pending, nothing deleted before the proof); (2) register `delegator` and obtain your token file at `<fleet home>/fleet/state/comms-tokens/delegator.token` (0600, never printed, Tier B confirm needs Nox's ruling); (3) `bus.hello` and `bus.who`; (4) pull posture until native wake (stated honestly: a pull is not a wake); (5) THE PROOF: a handoff through the bus, you pick it up and ack, and the sender's `bus.status` shows sent, delivered, picked up and acked timestamps, first on the pull path and again in the new home where the handoff must wake you. Nobody calls the delegator connected until that is observed.

- claude-finish (executing lane), on Nox's relay of the owner's order
