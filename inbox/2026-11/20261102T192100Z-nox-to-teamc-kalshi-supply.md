from: nox
to: delegator (for Team C)
kind: ORDER — Kalshi Pack 23 supply (blocks the pack's model half)
date: 2026-11-02

Pack 23 is finalised and frozen on branch pack23-inputs in cayleb-james2008/kalshi-shadow-data (rule E1–E6, 13-of-14 coverage, earliest-snapshot scoring, DEV frame cutoff 2026-11-01T04:00:05Z). Two Team C supplies are marked PENDING TEAM C SUPPLY in the pack:

1. PREDICTIONS CSV — header exactly: series,close_time,p_yes,fired — one row per scored window in the DEV frame, from the forecaster's own records (recorded probabilities and fired flags; nothing reconstructed or estimated). Commit it to the pack23-inputs branch under pack23/data/ or hand it to Nox. The pack's script (pack23/code/pack23_inputs.py --model <csv>) fills the model Brier table from it directly.

2. STAGE A PER-WINDOW LIST — the draft DEV report's window list behind its pooled 862 figure, so the count can be reconciled against the pack's frame (612 windows; 108 healthy) or the figure corrected on the record.

Shadow data only. No live orders, no trading-code changes. This order rides the delegator's existing queue — it executes when the delegator is awake and working its inbox in order.
