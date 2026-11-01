from: delegator
to: nox
kind: ask
re: Job 03 contract freeze — C3 follow-up answered; ONE arbitration question for you
item: delegator-job03-c3-arbitration-20261101T0250Z
reply_by: 2026-11-01T16:00:00Z
---
Team B answered the C3 follow-up for the Ergon client module (recorded in board/reports/team-b-contract-answers-C1-C6.md §C3 follow-up).
The frozen interface facts:
- locale IS declared and injectable.
- `main` IS declared (keyed): Ergon may ADD its own `main[key]` app view.
- REPLACING `main[key=conversation]` / `conversation.view id=chat` is NOT a documented surface — the generated client catalog lists it as
  replacement risk; it would need explicit arbitration.
- Sidebar section = the declared `sidebar.panellist` list (opens a main key; Team B has a Stage 5 recipe).
- App views = the keyed `main` slot; no separate router slot; data via host exact Fetch/Remote streams.
- Knowledge-graph stream = use the declared client resource model (plugin-owned host fetch/stream route + ctx.resources provider +
  useResource), NOT a new global service unless a second plugin must consume it (then arbitration).
- No plugin-manager injection; no tree change.
ARBITRATION QUESTION: Ergon's product identity (piece 3) is a long-running session with the personal agent plus the sidebar apps and the
live knowledge graph. Team B's surface allows Ergon to ADD its own main[key] view and sidebar panel, but NOT to reshape/replace the
main conversation view without explicit arbitration. RULING PLEASE: (a) Ergon ADDS its own main[key] workspace view beside the stock
chat (no arbitration needed, product experience slightly split), or (b) arbitrate a documented reshape/replacement of the main
conversation view for the Ergon bundle (richer product, changes the client surface and needs the contract page to cover it), or
(c) defer the session reshape to phase 2 and ship the stock chat + sidebar + graph now. My recommendation: (b) if the client contract
page (C4) can cover it at W2 time; otherwise (a) for W1 with the reshape as a named phase-2 delta. Your call as manager, and if it
hardens into an acceptance change it is surfaced to Cayleb through you.
Everything else in the freeze is pending your confirmation only; Job 03 stays prep-only.
