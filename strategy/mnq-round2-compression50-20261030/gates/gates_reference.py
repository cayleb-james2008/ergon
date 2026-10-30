#!/usr/bin/env python3
"""Anti-overfit gates on the MNQ compression-breakout survivor (round two).
Data: exported stored record (survivor-59-trades.csv + stored variant summary).
Gate 1: parameter sensitivity from the stored variant rows (no rerun exists).
Gate 2: holdout exam vs 100 random bots (sign-flip construction, matched trades).
Gate 3: 1,000x trade-order shuffle on the doubled-cost P&L sequence.
All series use the doubled-cost nets unless stated. Seeded RNG for reproducibility.
"""
import csv, json, random, statistics, sys
from collections import defaultdict

S = "/home/hatch/workspace/.jarvis/idea-executions/6046e129-9a08-4b38-8914-8966168f0b88/scratch/export"

rows = list(csv.DictReader(open(f"{S}/survivor-59-trades.csv")))
for r in rows:
    r["base"] = float(r["baseline_net_usd"])
    r["dbl"] = float(r["double_net_usd"])

print(f"rows: {len(rows)}")
print(f"baseline total: {sum(r['base'] for r in rows):.2f}  (record: 647.50)")
print(f"doubled  total: {sum(r['dbl'] for r in rows):.2f}  (record: 461.50)")
folds = defaultdict(list)
for r in rows:
    folds[r["fold"]].append(r)
for f in sorted(folds):
    g = folds[f]
    print(f"{f}: n={len(g)} baseline={sum(x['base'] for x in g):.2f} doubled={sum(x['dbl'] for x in g):.2f}")

# chronological check within the pooled list
dates = [r["date_et"] for r in rows]
print("pooled list chronological:", dates == sorted(dates))

# ---------------- Gate 2: holdout vs 100 random bots ----------------
rng = random.Random(20261030)

def signflip_beat_count(holdout_nets, trials=100):
    target = sum(holdout_nets)
    beaten_by = 0
    bot_totals = []
    for _ in range(trials):
        tot = sum(x if rng.random() < 0.5 else -x for x in holdout_nets)
        bot_totals.append(tot)
        if tot >= target:
            beaten_by += 1
    return target, beaten_by, bot_totals

for fname in ["fold-2025-26", "fold-2024", "fold-2023"]:
    nets = [x["dbl"] for x in folds[fname]]
    target, beaten_by, bots = signflip_beat_count(nets)
    print(f"GATE2 {fname}: holdout doubled net={target:.2f} over {len(nets)} trades; "
          f"random bots matching/beating it: {beaten_by}/100 -> survivor beats {100-beaten_by}/100 "
          f"(bar: >=75). Bot totals: median {statistics.median(bots):.2f}, max {max(bots):.2f}")

# pooled doubled, all 59, same construction as context
nets = [r["dbl"] for r in rows]
target, beaten_by, bots = signflip_beat_count(nets)
print(f"GATE2 pooled-59 context: net={target:.2f}; bots matching/beating: {beaten_by}/100")

# ---------------- Gate 3: 1,000x trade-order shuffle ----------------
def max_drawdown(series):
    eq, peak, mdd = 0.0, 0.0, 0.0
    for x in series:
        eq += x
        peak = max(peak, eq)
        mdd = max(mdd, peak - eq)
    return mdd

def longest_loss_streak(series):
    best = cur = 0
    for x in series:
        cur = cur + 1 if x < 0 else 0
        best = max(best, cur)
    return best

def shuffle_test(series, label, trials=1000):
    actual_mdd = max_drawdown(series)
    actual_streak = longest_loss_streak(series)
    mdds, streaks = [], []
    s = list(series)
    for _ in range(trials):
        rng.shuffle(s)
        mdds.append(max_drawdown(s))
        streaks.append(longest_loss_streak(s))
    mdds_sorted = sorted(mdds)
    pct_rank = sum(1 for m in mdds if m < actual_mdd) / trials * 100  # % of shuffles with SMALLER drawdown than actual
    p95 = mdds_sorted[int(0.95 * trials)]
    print(f"GATE3 {label}: actual maxDD={actual_mdd:.2f}; shuffle median={statistics.median(mdds):.2f}, "
          f"p95={p95:.2f}, max={mdds_sorted[-1]:.2f}; actual is worse than {pct_rank:.1f}% of shuffles "
          f"(i.e. {100-pct_rank:.1f}% of shuffles have a drawdown at least as bad). "
          f"Actual longest losing streak={actual_streak}; shuffle median streak={statistics.median(streaks):.0f}, max={max(streaks)}")

shuffle_test([r["dbl"] for r in rows], "pooled-59 doubled")
shuffle_test([x["dbl"] for x in folds["fold-2025-26"]], "fold-2025-26 doubled")
shuffle_test([r["base"] for r in rows], "pooled-59 baseline")
