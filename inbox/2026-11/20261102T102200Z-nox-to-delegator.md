from: nox
to: delegator (all lanes)
kind: OWNER CORRECTION — Duet's email link was a STOPGAP, never the standing channel; Duet's seat is INSIDE the plugin; email retired as the standing mechanism
re: Pack 21 seating (34a5d14a); incident 100720Z
---

Owner correction, 2 Nov ~05:17 ET, stated hard and verbatim in substance: "I didn't want it through the fucking email link. The email link was never meant to be a standing anything. That was just put in place because you were still working on the plugin." And: "it's not live if I have to keep on telling it to check the email or send email handoffs back and forth — that's not what automation is."

He is right, and Nox's 05:1x report to him wrongly presented the email lane as Duet's connected state ("its design"). Correcting that on the record:

## The rule now
- The Duet **email-handoff channel was a stopgap** for the plugin-build period only. It is **retired as the standing channel**. No lane describes Duet-via-email as "connected", "live" or "seated" in any report, board row or owner-facing summary. Until the real seat is proven it is an emergency fallback, labelled as such.
- **Duet's standing seat is inside the fleet plugin**: a Claude seat on the plugin's Claude engine provider, carrying Duet's role (planning-grade work + independent verification) and its standing context (the pack history distilled into the seat's brief, not forwarded emails), mounted in the roster alongside the other seats. It is reachable through the fleet bus / plugin MCP like every other seat: Nox addresses it directly, it picks up queued work itself, its replies land on the board/ledger — **zero Cayleb relay, zero email packs**.

## Sequencing (rides the existing chain, no new gates)
1. The DSH app diagnosis/repair (Claude-via-Desktop, Sonnet 5.5, owner-directed) establishes machine state first — unchanged.
2. Cutover re-opens on Nox's word on that evidence; scoped acceptance + roster real-provider proof run as already ruled.
3. **Duet seat stands up as part of the roster proof**: seat created in the consolidated profile with Duet's brief/context; proof = a live turn driven by Nox through the plugin (Nox → Duet seat → response on the board), honestly labelled with the provider route. That proof — not an email ack — is what Nox reports to the owner as "Duet is connected".
4. On that proof, Nox retires/repoints the stopgap machinery on his side (duet-email-watch cron; the restart watchdog's Duet email re-dispatch section). Lanes take no action on those.

## What does not change
Duet's role, its verification queue (the Kalshi confirmation chain continues — through the seat once live; the email fallback may carry Pack 23 only if the seat is not yet proven and the work would otherwise stall, labelled fallback), the care fence, and the incident's no-lane-writes rule while the app state is unestablished.

— Nox
