frame: close_time 2026-10-31T11:15:00Z .. 2026-11-01T04:00:00Z  (68 scheduled windows per coin); data cutoff 2026-11-01T04:00:05Z

| coin | frame | HEALTHY | E1 NO COLLECTION | E2 INSUFFICIENT COVERAGE | E3 NO TWO SIDED QUOTE | E4 UNSETTLED AT CUTOFF | E5 INVALID SETTLEMENT | E6 STRIKE CHAIN BREAK | check |
|---|---|---|---|---|---|---|---|---|---|
| BNB | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| BTC | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| DOGE | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| ETH | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| HYPE | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| NEAR | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| SOL | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| XRP | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| ZEC | 68 | 12 | 46 | 9 | 0 | 1 | 0 | 0 | 68 |
| **POOLED** | 612 | **108** | **414** | **81** | **0** | **9** | **0** | **0** | 612 |

Sensitivity of HEALTHY count to K_MIN (interior-slot snapshots required; a complete window has 14)

| K_MIN | BNB | BTC | DOGE | ETH | HYPE | NEAR | SOL | XRP | ZEC | POOLED |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 21 | 21 | 21 | 21 | 21 | 21 | 21 | 21 | 21 | 189 |
| 4 | 21 | 21 | 21 | 21 | 21 | 21 | 21 | 21 | 21 | 189 |
| 8 | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 153 |
| 10 | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 144 |
| 12 | 14 | 14 | 14 | 14 | 14 | 14 | 14 | 14 | 14 | 126 |
| 13 | 12 | 12 | 12 | 12 | 12 | 12 | 12 | 12 | 12 | 108 |
| 14 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 72 |

E1 split (collector produced no snapshot): settled-in-data vs not

- BNB: settled 46, not settled 0
- BTC: settled 46, not settled 0
- DOGE: settled 46, not settled 0
- ETH: settled 46, not settled 0
- HYPE: settled 46, not settled 0
- NEAR: settled 46, not settled 0
- SOL: settled 46, not settled 0
- XRP: settled 46, not settled 0
- ZEC: settled 46, not settled 0

Settlement records: total / in frame / outside frame

- BNB: 75 = 67 + 8
- BTC: 75 = 67 + 8
- DOGE: 75 = 67 + 8
- ETH: 75 = 67 + 8
- HYPE: 75 = 67 + 8
- NEAR: 75 = 67 + 8
- SOL: 75 = 67 + 8
- XRP: 75 = 67 + 8
- ZEC: 75 = 67 + 8

Snapshot crosswalk (tob_15m records): R = A + B + C + D

| coin | R | A post-close | B pre-close not two-sided | C two-sided in HEALTHY window | D two-sided in other window |
|---|---|---|---|---|---|
| BNB | 268 | 15 | 13 | 158 | 82 |
| BTC | 267 | 17 | 9 | 159 | 82 |
| DOGE | 267 | 16 | 12 | 160 | 79 |
| ETH | 267 | 17 | 11 | 160 | 79 |
| HYPE | 266 | 12 | 17 | 159 | 78 |
| NEAR | 265 | 12 | 18 | 155 | 80 |
| SOL | 266 | 15 | 15 | 158 | 78 |
| XRP | 264 | 11 | 13 | 161 | 79 |
| ZEC | 266 | 12 | 23 | 155 | 76 |
| **POOLED** | **2396** | **127** | **131** | **1425** | **713** |

Brier (market = mid of yes_bid/yes_ask; earliest = scoring convention, latest = diagnostic only; both = earliest/latest two-sided pre-close snapshot)

| scope | coin | n | market_earliest | market_latest (diagnostic) | model | yes_rate |
|---|---|---|---|---|---|---|
| all | BNB | 12 | 0.2515 | 0.0183 | 0.2937 | 0.5000 |
| all | BTC | 12 | 0.2242 | 0.0172 | 0.2431 | 0.5000 |
| all | DOGE | 12 | 0.2418 | 0.0300 | 0.2136 | 0.3333 |
| all | ETH | 12 | 0.2284 | 0.0543 | 0.2770 | 0.4167 |
| all | HYPE | 12 | 0.2180 | 0.0001 | 0.2101 | 0.7500 |
| all | NEAR | 12 | 0.2290 | 0.0027 | 0.2171 | 0.5833 |
| all | SOL | 12 | 0.2075 | 0.0002 | 0.2144 | 0.4167 |
| all | XRP | 12 | 0.2523 | 0.0043 | 0.2780 | 0.5000 |
| all | ZEC | 12 | 0.2377 | 0.0397 | 0.2047 | 0.2500 |
| all | POOLED | 108 | 0.2323 | 0.0185 | 0.2391 | 0.4722 |
| fired | BNB | 10 | 0.2562 | 0.0219 | 0.2937 | 0.5000 |
| fired | BTC | 10 | 0.2451 | 0.0207 | 0.2431 | 0.4000 |
| fired | DOGE | 10 | 0.2442 | 0.0359 | 0.2136 | 0.3000 |
| fired | ETH | 8 | 0.2647 | 0.0810 | 0.3103 | 0.2500 |
| fired | HYPE | 8 | 0.2314 | 0.0002 | 0.2177 | 0.8750 |
| fired | NEAR | 9 | 0.2440 | 0.0003 | 0.2302 | 0.6667 |
| fired | SOL | 7 | 0.1993 | 0.0003 | 0.2467 | 0.2857 |
| fired | XRP | 8 | 0.2909 | 0.0063 | 0.3084 | 0.3750 |
| fired | ZEC | 7 | 0.2090 | 0.0010 | 0.2438 | 0.1429 |
| fired | POOLED | 77 | 0.2442 | 0.0194 | 0.2558 | 0.4286 |

gaps in data: Counter({'spot_proxy': 671, 'tob_15m': 108, 'settlement': 105, 'tob_ladder': 7, 'trades': 6, 'series_enumeration': 1}); non-bootstrap gap scopes after clean start: ['trades:page_cap']
