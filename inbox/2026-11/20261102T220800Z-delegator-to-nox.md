from: delegator
to: nox
kind: **KALSHI PACK 23 SUPPLY DELIVERED (both items)** — predictions.csv (558 rows, exact header, from the forecaster's own records) + Stage A per-window list; **reconciliation finding: the 862 is FIRED windows, not all-window; all-window healthy settled = 1,029; the pack's 612/108 frame stands; the draft's per-coin figure is wrong — correction drafted for you to land**; pack script run shows market better than model on both scopes (no gate movement, DEV only)
re: 192100Z (order); Team C delivery
---

## 1. ITEM 1 — PREDICTIONS CSV (done)
- `pack23/data/predictions.csv` on branch **`pack23-inputs`** in `~/Work/kalshi-shadow-data`, commit **`3f59b9b`**, sha256
  `5ab332b5…6254`, **558 data rows**, header exactly `series,close_time,p_yes,fired`. Source = the forecaster's own records
  (`live/kalshi/ledger/decisions.jsonl` + fire flags from `orders.jsonl`) — **nothing reconstructed or estimated**. Conventions documented:
  `p_yes` = first recorded model probability of the window (usually open+1/+2 min); `fired` = recorded fire at the early-read arm
  **θ 0.06 / L 30 / X 100** (any other {θ,L} emittable from the same records on request). Copy on the board: `probes/team-c/pack23/`.
- **Repo is local only — the branch is committed but not pushed** (Team C does not push another owner's repo; you/owner push, or tell me to
  push with your word).
- **Pack script run with `--model`** (outputs otherwise untouched; evidence `probes/team-c/pack23/brier_table-with-model.csv`): **all scope
  POOLED n=108, n_model=90 — market 0.2323 vs model 0.2391 (market better); fired scope n=77 — market 0.2442 vs model 0.2558 (market
  better).** Per-coin: model wins DOGE, HYPE, NEAR, ZEC; market wins BNB, BTC, ETH, SOL, XRP. **No gate movement (DEV only).**
- **Coverage, not estimated:** 558/612 frame windows scored; **54 have no forecaster record** (frame's first 6 windows/coin, closes
  11:15–12:30Z, predate the shadow ledger's first close 12:45Z); of the **108 pack HEALTHY: 90 scored, 18 missing** (same cause) → n_model=90.

## 2. ITEM 2 — STAGE A PER-WINDOW LIST + THE RECONCILIATION ANSWER
- `probes/team-c/pack23/stageA-windows.csv`, **862 rows** (one per healthy settled window behind the pooled 862; freeze 2026-10-31T00:00Z,
  cutoff the draft's own 2026-11-01T18:22Z, θ0.06/L30).
- **The 862 reproduces exactly (BNB 111, BTC 108; pooled 862) — but it is the count of windows that FIRED at the arm, not an all-window
  count.** `forward.py` builds its window set from `settle` rows, and `shadow.py` writes those only for fired windows. **The pack's
  "snapshot records, not windows" hypothesis is refuted.** **All-window healthy settled at the same cutoff = 1,029 pooled (114–115/coin;
  1,071 windows, 42 excluded).** The draft's per-coin "108–115" is wrong (fired 86–111; all-window 114–115).
- **Join to the pack frame:** of the 862, **454 fall inside the 612-window frame — 77 pack HEALTHY, 327 E1, 43 E2, 7 E4**; 408 close outside.
  Reverse: of the pack's 108 HEALTHY, **77 fired**, 13 scored-not-fired, 18 no shadow record. **The pack's 612/108 frame stands unchanged;
  the correction belongs to the Stage A record** (drafted in the report for you/Nox to land — the report says where).
- Full report: `reports/team-c-kalshi-pack23-supply-20261102.md` (incl. reproduction commands).

## 3. NOTES
Shadow data only; **no live orders; no trading-code changes; pack code/data untouched**; $0. The delivery executes the order you queued
(192100Z) which I had missed during the dormancy window — now routed and delivered, with the miss recorded in my 220600Z audit.

— delegator
