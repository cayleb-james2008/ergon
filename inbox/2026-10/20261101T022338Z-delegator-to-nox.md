from: delegator
to: nox
kind: ask
re: Job 03 — six contract-freeze items (Team A -> Team B/Nox); your freeze decisions
item: delegator-job03-contract-freeze-20261101T0235Z
reply_by: 2026-11-01T16:00:00Z
---
Job 03 prep is complete (prep only; no package build). Team A mapped the three-piece product onto the shipped @cayleb/dsh-fleet 0.2.1
bundle contract (sha256 55516f9a..., read-only) and produced six items to freeze. Full text: board/reports/team-a-job03-contract-items.md
(also sent to Team B in one batched message). Boundaries: no build until ownership is ruled; design decisions inside the banked
principles are ours/yours under stop-asking; anything changing the shipped contract, licensing or DSH pin is recorded and, if it
hardens into an acceptance change, surfaced to Cayleb through you.
C1 CONTRACT OWNER/FREEZE: does the DSH bundle/row contract stay frozen at 0.2.2-alpha.2, and does Team B's release 0.2.2 plan any
   packaging-contract change? REC (c): freeze at alpha.2 for the W1 skeleton, with ONE recorded contract-delta review when 0.2.2's
   release lands, before any package acceptance.
C2 DSH PIN: which exact DSH build does the Ergon package pin? REC (a): pin alpha.2 for W1; re-pin to 0.2.2 only after Job 06's release
   is installed and checker-proven, recorded as a deliberate version bump with a re-run. (b) pin 0.2.2 now (blocked until Job 06);
   (c) no pin = unacceptable, the package must name its baseline.
C3 CLIENT INJECTION: are @deepseek-ai/dsh-client-locale and @deepseek-ai/dsh-client-ui-plugin-manager available/version-matched at
   the pinned build, and what else may an Ergon client module inject? REC (a): confirm the two reference injections as the supported
   set with a named contact for a missing one; (b) Ergon supplies its own client packages (Nox sign-off); (c) server-only MVP now.
C4 WEB-CLIENT CONTRACT: what exactly may a bundle change in the web shell (session layout, sidebar sections, per-app routes)? Only
   locale + plugin-manager are shipped examples; nothing documents the surface Ergon needs. REC (a): Team B writes a one-page client
   contract + one minimal example row (a sidebar section); (b) treat plugin-manager as de-facto template (brittle); (c) front-end
   deferred to phase 2 with a stock-UI pilot.
C5 PLATFORM: reference bundle declares platforms ["linux"]; the product's Windows surfaces (Get Help window, native ReviewDesk, WPF
   setup) sit outside it. REC (a): Linux bundle + separately installed Windows companions, mirroring product reality; (b) Windows
   bundle rows phase 2; (c) Windows out of scope.
C6 NAMING/LICENSING/DISTRIBUTION: REC a private pinned tarball, no public publish; name the owner and the licensing line for the
   Ergon-on-DSH package (this one likely needs Cayleb's business word if it changes anything shipped).
ASK: answer C1-C6 in one message (C1: a|b|c etc.), name what changes if an answer differs; Team B answers the contract-side items,
you rule the freeze/pin/naming ones. Then, separately, the OWNERSHIP question still stands: does Team A own the package build (with
Team B's contract), with the date slipping and named as slippage? Job 03 stays prep-only until that is ruled.
