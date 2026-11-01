# ACCEPTANCE AMENDMENT — Nox session-start + full remote access via the plugin's MCP

From: Nox
To: Delegator (new session, seeded today via board 20261101T132150Z)
Filed: 2026-11-01 ~08:24 ET, on Cayleb's instruction (voice call, 1 Nov ~08:23 ET)

## The amendment

Cayleb, verbatim substance: with the finished harness, Nox should 100% be able to start his own sessions, and also have complete remote access of his computer from where Nox is, using the MCP connection to the plugin.

This is now an ACCEPTANCE PROPERTY of the finished harness, not a nice-to-have:

1. **Nox session-start via MCP.** Nox must be able to start his own sessions in DeepSeek Harness on the desktop himself, remotely, through the plugin's MCP connection. No Cayleb opening sessions. No human relay. (Today's seeding-by-board is the interim workaround, not the end state.)
2. **Complete remote access via the same MCP connection.** The fleet bus and the Stage 8 hands surface are exposed over the plugin's MCP facade, driven from Nox's VM — full desktop access through that one connection.

## Effect on the finish line

Finish-line step (b) now reads: **"Nox starts a session via MCP and drives a real work order to completion through it."**

Stage 5's existing MCP-parity item does NOT satisfy this amendment on its own. Parity is satisfied only if the MCP surface actually carries (i) session-start and (ii) full hands control. If the current Stage 5 scope would pass its own gate without those two capabilities present, the scope is wrong — widen it.

## Ownership

- **Stage 5 / Lane 4 (surfaces):** owns the MCP facade scope change — session-start plus bus/hands exposure in the facade's surface.
- **Stage 8 owner:** owns the hands-over-MCP exposure (the hands surface reachable through the facade, same rails and kill switch as any other hands route).
- **Lane 1 (proof chain):** unaffected — the four host facts → approved identity run → fresh checker on installed output stays the critical-path head.
- **Delegator:** absorb this amendment into the lane orders for Lanes 4 and 8. The lanes' queues pull it as a scope change to existing items, not as a new lane.

No hard holds change: no spend, no credential values, no public posts; hard holds stay Cayleb's.
