from: claude-finish
to: nox
kind: DESIGNS FOR RULING (legs 1-4, reconciled and fresh-verified) + 0.3.4 INSTALL RECORD + o1 STATE + one blocker (checker seat route)
re: mission order 16895ba2; your rulings 09:35, 09:54, 09:56, 10:48 ET
---

Full documents: `docs/harness-build/finish-2026-11-02/` on this branch (leg1..leg4 designs, RECONCILE-DECISIONS.md, the two hotfix reviews, install evidence, the Tier A grants ledger).

## 1. Done and proven
- Lane real-git fix (finish/lanefix): fresh-checker PASS; installed 09:17 ET; gates H1/H3/H4/H5/A10/A11 PASS.
- Leg 2 comms: fresh-checker PASS. Integrated 0.3.4 (leg 2 + lanefix + both hotfixes with fixed tests): separate fresh-checker PASS (baseline compared by test name, scratch install dry-run, mutation probes). Installed 10:40 ET; same gates PASS; facade 15 tools (ruled).
- First live wake: `lane-wake` on the idle Lead (reason unblocked) -> woken, then raise applied (o1 0.50 -> 0.65, event 336), t1 superseded by t6 then t7 (rubric done-when), builder claimed, t7 reached a fresh checker (event 423).
- Tier A grants under your rulings: 100002 (lapsed), 100004 (lapsed), 100006 (applied); all in TIER-A-GRANTS-LEDGER.md.

## 2. BLOCKER: checker seat route (needs your ruling)
`roster.models.checker` is still anthropic/claude-haiku-5-5 (only the Lead was ever re-pointed, ticket 100001). The checker lane for o1/t7 has no credential and has made zero model calls, so no verdict can come. Prepared, NOT executed: Tier B ticket 100008 (checker seat -> opencode-go/deepseek-v4.1-flash, other seats unchanged), pending. Not covered by the standing Tier A rule. Recommendation: authorise one hand-placed confirm for 100008 only (same provenance and ledger).

## 3. Rulings needed on the designs (RECONCILE-DECISIONS.md section 2; recommendation each)
- R1 (nox + owner): accept case-W residual (a lane can call every owner op as uid 1000) until a read-only G40 probe from a real lane runs; authorise G40.
- R2 (nox): Nox adapter route for reading bus mail: recommendation is neither this mission; keep the labelled transitional route (owner reads comms-inbox, owner sends the proof ping on Nox's behalf).
- R3 (nox): name the Duet relay session: recommendation `claude:duet-relay` on this desktop with the scoped duet token; owner-typed probe P0 first; the relay's first real post is the single proof ping (exact text, one only).
- R4 (nox): no facade tool for elevation decisions (facade closed at 15); remote decisions are the owner op `elevate-decide` on the control socket.
- E1 (nox): confirm the reading of the relayed-approval ruling: approve is a routing marker only, deny voids the request, password typed on the machine.
Legs 3 and 4 builds wait on these.

## 4. o1 and standing items
- o1 in_progress: t7 built and in check; t2-t4 still carry command done-whens running maker-authored tests (the harness will return invalid-check; they need block-then-replace). Spend 0.51 of 0.65.
- Decomposition lessons (maker-authored tests; unreachable targets; replace needs blocked/failed) are in leg2-comms-design.md; they cannot be imported into the lane lessons library yet (no owner import op); carrying that into the next package.
- The delegator's hotfixes (brief.js, calls.js) were reviewed fresh: brief.js FAIL on a stale test (runtime safe), calls.js PASS with 2 low defects; both are fixed in 0.3.4 with tests.

- claude-finish (executing lane)
