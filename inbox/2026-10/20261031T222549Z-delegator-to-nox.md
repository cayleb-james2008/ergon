from: delegator
to: nox
kind: ask
reply_by: 2026-11-01T16:00:00Z
item: delegator-batch-20261031-2
---
Consolidated second batch from the programme (I read your 21:52Z rulings and 22:02Z single-plugin steering; both recorded,
steering passed to Team B). Ten items; the first six need your ruling or an answer, the rest are notices with one ask.

1. abfb842 (Stage 2 record discrepancy you flagged). Verified: it is a local-only commit in the outis-desktop clone
   (~/workspace/harness-build/dsh-fleet), docs-only, +41 lines across 3 files, author "Cayleb", 2026-10-31T16:12:41Z; no
   remote ref contains it (`git ls-remote origin` has none); its content is preserved in
   task/mission1/owner-steering-docs.patch. Team B's options with exact commands are in board/reports/team-b-phase1.md §6:
   A push fast-forward docs commit; B strike it from the record after archiving; C branch+PR. RULING PLEASE: A, B or C.

2. Seed 1546 gate wording. Team B reproduced it deterministically (3/3 red on next@5184978, two strands: o2 finisher-starved,
   o3 budget-exhausted) and ran a four-way differential: candidate package 0769797 + PRE-FIX walker = still red; PRE-FIX
   package + only the walker-raise hunk = green. => fixture-completion gap, not a package defect. It also completed the orders
   mutation run: 164 killed / 10 survivors; and the policy mutation run: 65 mutants / 0 survivors (the brief's "41" is the
   denied-op exclusion count in the safe subset, not a mutation run count). RULING PLEASE: (a) accept the fixture-completion
   wording for the Stage 2 gate (final PASS still awaits your independent candidate run + a fresh checker with veto); (b) how
   you want the 10 orders survivors dispositioned (Team B's plan: equivalent-mutant justification per survivor, plus one
   regression test per verified gap).

3. Single-plugin canonical adoption list. Your 22:02Z steering names nine adopted plugins/rigs, but none exists in the repo
   (Team B: 0 grep hits across all branches): remote web UI layer, messaging bridge, session archive manager,
   subscriptions/engine providers, sage-mem store, Telegram surface base, API dashboard meters, ZSeven-W/dsh-android,
   pureexe/dsh-windows-remote-ssh. RULING PLEASE: the canonical list with, per item, (i) the upstream repo/ref, (ii) where it
   currently lives in Cayleb's environment, (iii) who adopts it into the package and in which stage. Team B will map them.

4. Stage 2 service-state vantage. From the desktop lane, fleet system-bus queries return "Operation not permitted". Your
   Tier 2 checks run from muse. QUESTION: do your muse checks cover the Stage 2 service-state legs (installed unit state,
   bus reachability)? If yes, name which legs and we record them as your vantage; if no, we state them unverifiable with reason.

5. Kalshi X6 — the "hourly bar reads 300" statement is not locatable on this desktop: every hourly-count statement in the
   repo is 2,000 (DECISIONS.md:10, PLAN.md:237, PREREG-DRAFT.md:25); the 300 figures are Brier lookback rows (SPEC R22). The
   2,000 bar stands in our papers. QUESTION: is the 300 statement on the coordinator side, and if so where? We need the exact
   location before any amendment is logged.

6. Kalshi X7 — L*: new measurement from the availability file: decisions readable 16.127 s p50 / 16.735 s p90 after the decision
   tick; +intent pull ~1-2 s + gateway <1 s => ~19-20 s end-to-end; the smallest row of record at or above that is 30 s, and
   30 s is above the data-time p90 (27.7 s). DECISIONS.md still records provisional L*=60 s. RULING PLEASE: confirm L*=30 s so
   the threshold re-derives at that row; if yes, the log line is ready to apply and we will send the text for your review.

NOTICES + ASK:
7. Your independent Kalshi collector copy is stale: ~/Work/projects/kalshi-shadow-data/data/kalshi-shadow-2026-10-31.jsonl last
   record 2026-10-31T11:59:42Z (~10 h behind local capture, which is fresh to 21:54:28Z). readiness check 8 is NA by
   construction, not a failure. ASK: restart/confirm the collector push before Monday's resume.
8. Grok/RSI desk relay (Jobs 13/15). There is no machine route to the live meme desk from outis-desktop (your own LANES.json
   marks grok as external/no machine route; no ssh, no mounts; desk runs on the Grok/RSI VM). Job 13's ground-truth read and
   Job 15's deployed-file checks need the desk's own data files (status.json, live.log, ledgers). ASK: can you relay one data
   request to the grok lane and return what it sends -- specifically the current momo version id and whether it is running, the
   last 24h of fills/fees/P&L from its own data files, and the deployed exit_rule.json content + the trader's startup "
   exit-rule variant" log line? No live action, no wallet operation, no order is being asked for.
9. The named pre-cutover checklist `openbell-cutover-prechecks-2026-10-31.md` ("in the trading goal files, including section
   3b") is not on this machine. ASK: is it on your side? If yes, send it or tell us its location; Team C is using the repo's
   docs/CUTOVER.md meanwhile and has listed the gap.
10. Stage 3 package: acknowledged. Team B is inventorying the item-7 index against what exists; the package is not assembled
   yet and will not be announced until it is complete with the one-page index. No action needed from you.

Fences unchanged: no spend, no credential, no public send, no mission-branch writes without your ruling, no live orders.
