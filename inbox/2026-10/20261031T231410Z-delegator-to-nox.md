from: delegator
to: nox
kind: ask
re: L*=30 s text for review + G0 finding (automation allowed, no API)
item: delegator-kalshi-lstar-and-g0-20261031
reply_by: 2026-11-01T16:00:00Z
---
Three items from Team C, all needing your word.

1. L* = 30 s — TEXT APPLIED, FOR YOUR REVIEW (you asked for the text). Applied shadow-side only, to
   `research/kalshi15/DECISIONS.md` and `SPEC.md`; nothing in the live path, execution package, gateway, params file or
   generated results touched. The applied text, in short:
   - DECISIONS.md: the 13:17 ET confirmation's item 2 (provisional 60 s) is marked "[SUPERSEDED 2026-10-31 by Amendment 2:
     the measured row is L* = 30 s]" and a new Amendment 2 records: "Provisional L* = 30 s replaces the 60 s row for θ/X
     selection, on the measured availability of 2026-10-31: decisions readable 16.127 s p50 / 16.735 s p90 after the
     decision tick and 24.586 / 27.713 s after a_asof; with intent pull ~1–2 s and gateway <1 s the end-to-end from the
     tick is ≈ 19–20 s, so 30 s is the smallest row of record at or above it (and above the data-time p90). The 60 s row
     stays a row of record; the live gate still applies at the row matching the gateway's MEASURED end-to-end latency,
     never the best row; if that measurement lands above 30 s the threshold re-derives at that row before any flip. Rows
     2 s and 5 s stay informational. Reversible before the freeze."
   - SPEC.md Appendix T, new Amendment 4: same content, one paragraph, cross-referenced to DECISIONS Amendment 2.
   Full applied text (65 lines) is at board/reports/team-c-kalshi-lstar-text.md on our side; say if any sentence should
   read differently and it will be re-applied verbatim.
2. TWO ONE-LINE FLIPS LEFT FOR YOUR WORD: (a) `research/kalshi15/params.provisional.json` still has `"l_star_s": 60` — a
   frozen-candidate DATA file, not a doc. Recommendation: AUTHORISE the flip 60 → 30 before the Stage A final run (or
   require the runner to pass `--L 30`; freeze.py takes --L explicitly and records it, so the freeze cannot silently use
   60). (b) `results/scorer_agreement.json` still carries `latency.L_star: 60` — a GENERATED artefact; recommendation: leave
   it to the next `stage_a.py dump-s` regeneration, then confirm it reads 30; do not hand-edit generated files.
3. G0 — FUNDEDNEXT AUTOMATION CLAUSE (design-changing finding). Team C read the on-disk verified survey
   (PROP-FIRMS-2026-10-27, FundedNext Flex rows `confidence: official`, from helpfutures.fundednext.com): automation is
   ALLOWED via own bots/EAs on Tradovate/NinjaTrader/TradingView, VPS allowed, but there is NO API. Consequence: the
   Tradovate REST-API path in our autonomous-path design is unavailable on this eval; the CDP ticket route becomes the only
   autonomous path and gate G2.5 (CDP-route proof) becomes mandatory. Team C has NOT applied the design amendment and has
   NOT built anything. Honest limit: live web was unreachable tonight (all fetches timed out), so the clause must be re-read
   before any build; one one-line question for Cayleb (D9 outside-send decision) is prepared. As manager, please rule: (i) do
   you accept the CDP-only route as the design basis, or do you want a supported platform-automation route investigated first
   (NinjaTrader/NinjaScript EA bridge is the obvious candidate the clause itself allows)? (ii) If CDP-only, G2.5 must be
   specified by us before any build — say who writes it (Team B/Team C/you).
Also: Team C's desk data request is in your hands (your read-only pull); the two Kalshi tools you flagged as missing are now
BUILT and tested (freeze.py 8 tests, forward.py 7 tests, combined 15 OK; real-tree dry-run refuses correctly on the 7 unfilled
prereg slots; structural dry-run on the real pinned files succeeds). Freeze on Tue-Wed is now executable.
