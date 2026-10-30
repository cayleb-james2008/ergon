# Anti-overfit gate methods — the desk's standing configuration

Set by Nox, Fri 30 Oct 2026, from Cayleb's accepted gates proposal (Algory/QuantStack intake). These three gates run on every strategy-search survivor from round 3 onward, and on Kronos when its trade data lands. They advise ahead of R3; R3 remains the live gate and is unchanged by them. All profit-and-loss series are at **doubled costs** unless a gate says otherwise. Any deviation from these methods in a batch must be named in that batch's manifest **before** its runs start.

Reference implementation: `gates_reference.py` in this folder — the exact script that produced the round-two verdicts on the exported record (seeded RNG, seed 20261030, reproducible). Report format: the gate report issued 30 Oct 2026 (`mnq-anti-overfit-gate-report-2026-10-30.md`, prop-firm goal files) — one section per gate with method, numbers and verdict, an overall verdict, and a closing statement that R3 is unchanged and nothing was armed.

## Gate 1 — Parameter sensitivity

- For the candidate's family parameter, evaluate the stored or run variants at nudges of ±10–20% relative to the selected value, across the same walk-forward test folds.
- Verdict **fragile** if a majority of the nudged variants finish at or below zero out-of-sample at doubled costs.
- Where the record holds only stored summary rows (no rerun), judge from them and state plainly how many neighbours exist — a single neighbour that reverses the result is itself a fragile finding, reported as such, not silently upgraded to a grid result.
- Round-two reference outcome: the survivor at a 50% compression fraction (+$461.50 doubled, pooled) against its one stored neighbour at 75% (−$1,061.00 doubled) — fragile.

## Gate 2 — Holdout exam vs 100 random strategies

- The exam slice is a walk-forward test fold the candidate was **not** selected on; the most recent fold is the deciding slice. Report every fold, not only the deciding one, plus the pooled record as context.
- 100 random bots per slice, **sign-flip construction**: each bot takes the same trades in the slice and flips a fair coin for direction on each trade — matched opportunities, matched trade count. Bot total = the slice's doubled-cost net under its coin flips.
- **Bar (set before the run): the survivor must beat at least 75 of the 100 bots.** Beaten-by counts include bots that match the survivor's total.
- Round-two reference outcome: 2025–26 fold beats 62/100 — fail; 2024 beats 77/100 — narrow pass; 2023 beats 40/100 — fail (the slice itself lost money).

## Gate 3 — Trade-order shuffle ×1,000

- Permute the doubled-cost profit-and-loss sequence 1,000 times (seeded); recompute the equity path's maximum drawdown for each permutation.
- **Pass** when the real path's maximum drawdown is not in the luckiest quartile of orderings — operationally, at least ~25% of shuffles show a smaller drawdown than the real path. **Fragile** when fewer than ~25% do (the smooth recorded path depended on lucky sequencing).
- Report the real drawdown, the shuffle median and 95th percentile, and the longest losing streak (real vs shuffle median/maximum) as supporting evidence.
- Path drawdowns are multi-period figures; never compare them against the desk's daily stop.
- Round-two reference outcome: pooled real drawdown $792 vs shuffle median $757 (real path worse than 56% of shuffles); 2025–26 fold $688 vs median $589.50 (worse than 72%) — pass.

## Standing notes

- Verdicts rest on the actual trade list, never on summary claims. If the trade list cannot be produced, the gates are recorded as **not run**, with the missing piece named — never reconstructed, never estimated.
- A candidate that fails or rates fragile on any gate is not rejected outright: the verdict is recorded, R3 still governs arming, and the preregistered prospective test carries the deciding evidence.
