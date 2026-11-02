# TEAM C — KALSHI STAGE A: HEALTHY-WINDOW EXCLUSION ACCOUNTING (Pack 23) — 2026-11-02

Owner: `trading-lead`, Team C. **Read-only** (no order, no flip, no gate movement, no credential, $0). Prepared for Nox →
Duet as Pack 23. Read at **2026-11-02T20:31Z (15:31 ET)** from the shadow lane's own records. Answers the three asked
items: (1) the healthy-window exclusion rule **as actually applied**, in writing; (2) the **total excluded-window count**;
(3) the **per-coin excluded counts** — from the collector record as the shadow consumed it.

## 1. The rule as actually applied (code path, not a paraphrase)
Unit = **one market ticker** (one 15-minute window) that the shadow engine settles (`ev:"window"`). The engine computes
`healthy` at settlement over its own received copy of the **collector record** (books, gaps, ref ticks, markets/settlements).
Source: `live/kalshi/shadow.py` `_health()` + `_settle()`; parameters from `research/kalshi15/params.provisional.json`.
A window is **EXCLUDED** (does not count toward the 2,000-per-coin Arm A bar) if **any** of these is present:

| # | Reason tag | Condition as coded |
|---|---|---|
| 1 | `book_gap:{bad}` | At any of the **13 one-minute check points** inside the window (`t = T0 + 60·k`, k = 1…13), the captured book for that ticker had **no visible update**, or its newest update was older than **`book_max_age_s = 30.0` s**. `bad` counts how many of the 13 points failed. |
| 2 | `gap_record` | A capture **gap record** for that ticker (or a series-level record with no ticker) **overlaps** the window interval `(T0, T1)`. |
| 3 | `ref_cover:{cov}` | The **reference-price recorder** covered **less than 95%** of the window duration (`cov < 0.95·(T1−T0)` reference ticks). |
| 4 | `unsettled_timeout` | No market settlement/result was available within **`UNSETTLED_S` = 2 days** (172,800 s) after `T1`; the window closes as unsettled (healthy = false). |
| 5 | `result:{res}` | A settlement record existed but was not a clean `yes|no` with a usable expiration value (e.g. missing/contradictory result). |
`reconcile_fail` (shadow settlement vs the public settle rule mismatch) is **fatal to the shadow**, not an exclusion — zero
occurrences in this record. **Only healthy settles count toward the bar** (`research/kalshi15/forward.py`: "Only `healthy`
settles count toward the bar"); excluded windows are retained in `unhealthy.jsonl` and in the ledger with the reason.

## 2. Totals (period covered by the current shadow ledger)
- **Period:** `2026-10-31T12:46Z` → `2026-11-02T20:31Z` (2026-10-31 08:46 ET → 2026-11-02 15:31 ET).
- **Windows evaluated: 2,008** — unique tickers, **zero duplicate tickers** (verified).
- **Total excluded: 42** (2.1% of evaluated windows).
- **Healthy windows: 1,966** (the bar is 2,000 per coin; per-coin healthy 218–219 → ~20 days to the bar at the observed rate).
- **Reason split of the 42:** `gap_record` **33**, `book_gap` **9**. (Each excluded window carried exactly one reason in
  this record; no window carried more than one.)
- **Time distribution:** all 42 fall between `2026-10-31T12:46Z` and `2026-11-01T06:51Z`. **Zero exclusions in the
  ~32 hours since** (2026-11-01T06:51Z → 2026-11-02T20:31Z).

## 3. Per-coin excluded counts (and the healthy remainder)
| Coin | Windows evaluated | **Excluded** | Healthy | `book_gap` | `gap_record` |
|---|---|---|---|---|---|
| BNB | 223 | **5** | 218 | 1 | 4 |
| BTC | 223 | **5** | 218 | 1 | 4 |
| DOGE | 223 | **4** | 219 | 1 | 3 |
| ETH | 223 | **4** | 219 | 1 | 3 |
| HYPE | 223 | **5** | 218 | 1 | 4 |
| NEAR | 224 | **5** | 219 | 1 | 4 |
| SOL | 223 | **4** | 219 | 1 | 3 |
| XRP | 223 | **5** | 218 | 1 | 4 |
| ZEC | 223 | **5** | 218 | 1 | 4 |
| **Total** | **2,008** | **42** | **1,966** | **9** | **33** |
Cross-check vs `live/kalshi/ledger/counts.json` (written 15:21 ET, 10 minutes before this read): it shows healthy
BNB 218, BTC 218, DOGE 219, ETH 219, HYPE 218, NEAR 218, SOL 219, XRP 218, ZEC 218 = 1,965 — **exactly this table minus one
NEAR window that settled between the two snapshots** (counts.json is written on a cadence; the ledger read at 15:31 is the
later truth). No other difference.

## 4. The 42 excluded windows (as recorded in the ledger)
| UTC (u) | Coin | Ticker | Why |
|---|---|---|---|
| 2026-10-31T12:46Z | BNB | KXBNB15M-26OCT310845-45 | book_gap:4 |
| 2026-11-01T06:01Z | BNB | KXBNB15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:16Z | BNB | KXBNB15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:31Z | BNB | KXBNB15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:46Z | BNB | KXBNB15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:51Z | BTC | KXBTC15M-26OCT310845-45 | book_gap:3 |
| 2026-11-01T01:35Z | BTC | KXBTC15M-26OCT312130-30 | gap_record |
| 2026-11-01T06:20Z | BTC | KXBTC15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:35Z | BTC | KXBTC15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:50Z | BTC | KXBTC15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:46Z | DOGE | KXDOGE15M-26OCT310845-45 | book_gap:4 |
| 2026-11-01T06:01Z | DOGE | KXDOGE15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:36Z | DOGE | KXDOGE15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:46Z | DOGE | KXDOGE15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:46Z | ETH | KXETH15M-26OCT310845-45 | book_gap:4 |
| 2026-11-01T06:06Z | ETH | KXETH15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:21Z | ETH | KXETH15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:36Z | ETH | KXETH15M-26NOV010130EST-30 | gap_record |
| 2026-10-31T12:46Z | HYPE | KXHYPE15M-26OCT310845-45 | book_gap:4 |
| 2026-11-01T06:01Z | HYPE | KXHYPE15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:16Z | HYPE | KXHYPE15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:31Z | HYPE | KXHYPE15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:46Z | HYPE | KXHYPE15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:46Z | NEAR | KXNEAR15M-26OCT310845-45 | book_gap:4 |
| 2026-10-31T19:16Z | NEAR | KXNEAR15M-26OCT311515-15 | gap_record |
| 2026-11-01T06:01Z | NEAR | KXNEAR15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:31Z | NEAR | KXNEAR15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:46Z | NEAR | KXNEAR15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:46Z | SOL | KXSOL15M-26OCT310845-45 | book_gap:4 |
| 2026-10-31T21:06Z | SOL | KXSOL15M-26OCT311700-00 | gap_record |
| 2026-11-01T06:16Z | SOL | KXSOL15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:51Z | SOL | KXSOL15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:46Z | XRP | KXXRP15M-26OCT310845-45 | book_gap:4 |
| 2026-11-01T06:06Z | XRP | KXXRP15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:16Z | XRP | KXXRP15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:36Z | XRP | KXXRP15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:51Z | XRP | KXXRP15M-26NOV010145EST-45 | gap_record |
| 2026-10-31T12:46Z | ZEC | KXZEC15M-26OCT310845-45 | book_gap:4 |
| 2026-11-01T06:01Z | ZEC | KXZEC15M-26NOV010100EST-00 | gap_record |
| 2026-11-01T06:16Z | ZEC | KXZEC15M-26NOV010115EST-15 | gap_record |
| 2026-11-01T06:31Z | ZEC | KXZEC15M-26NOV010130EST-30 | gap_record |
| 2026-11-01T06:46Z | ZEC | KXZEC15M-26NOV010145EST-45 | gap_record |

## 5. What the pattern says (no smoothing)
- **The 9 `book_gap` exclusions are all the ledger's first window** (`KX*15M-26OCT310845-45`, settled 12:46–12:51Z on
  31 Oct): at the first window the engine had no prior book history to satisfy the 30 s age test — a **start-up** effect,
  not a market-data fault. They are correctly excluded by the rule and are not evidence against the capture.
- **The 33 `gap_record` exclusions are the DST-hour cluster**: 06:01–06:51Z on 1 Nov = the repeated local hour (01:01–01:51
  EST after the fallback) plus three isolated ones (NEAR 31 Oct 19:16Z, SOL 31 Oct 21:06Z, BTC 1 Nov 01:35Z) — the poller's
  overrun/gap pattern at window boundaries, already documented in the DST adjudication. **The rule excluded them exactly as
  designed; nothing was reconstructed or re-admitted.**
- **No exclusions at all in the ~32 hours since 2026-11-01T06:51Z** — the record has been clean going into the freeze target.
- Excluded windows stay excluded: the bar counts only healthy windows, so the 42 do not flatter the count.

## 6. Method / reproducibility (read-only)
- Source: `live/kalshi/ledger/orders.jsonl` — every `{"ev":"window", …}` record carries `coin`, `ticker`, `u`, `healthy`,
  `why`; counted per **unique ticker** (2,008/2,008 unique). Cross-check `live/kalshi/ledger/counts.json` (healthys only)
  and `live/kalshi/ledger/unhealthy.jsonl` (**42 rows**, same 42, last at 2026-11-01T06:51Z — independently agrees).
- Rule source quoted from code: `live/kalshi/shadow.py` `_health()` (lines ~502–524) and `_settle()` (lines ~526–570);
  parameters `book_max_age_s 30.0`, `cutoff_s 60`, `UNSETTLED_S 172800`.
- Reproduce with (read-only): stream `orders.jsonl`, filter `ev=="window"`, aggregate by `coin` and `healthy`/`why`.
- Caveats stated: (i) the "collector record" is the shadow's own consumed copy of the desktop capture — `nox_reconcile`
  (readiness check 8) remains **NA by construction** because Nox's collector copy has no overlapping clean cycles; this
  accounting does **not** claim Nox-side verification. (ii) Snapshot delta vs `counts.json` explained in §3. (iii) No
  threshold, gate, preregistration or ledger content was changed; $51.39 untouched; nothing traded.

## 7. Fences
Read-only on the lane tree; no write outside `board/reports/`; no order/flip/gate movement, no credential read, no network
calls, $0. Prepared for Nox → Duet (Pack 23).

— `trading-lead`, 2026-11-02T15:32 ET (20:32Z).
