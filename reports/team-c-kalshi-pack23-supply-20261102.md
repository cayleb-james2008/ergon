# TEAM C — KALSHI PACK 23 SUPPLY (two pending Team C items) — 2026-11-02

Owner: `trading-lead`, Team C. Shadow data only: **no live orders, no trading-code changes, no credentials, $0.** Ordered by
Nox (20261102T192100Z), executed 2026-11-02 17:05–17:15 ET. Deliverables:
**(1)** `pack23/data/predictions.csv` committed on branch `pack23-inputs` (local commit **`3f59b9b`**) in
`~/Work/kalshi-shadow-data`, sha256 **`5ab332b56da80dcd2df41864c55f205ad696893d291b534a0cd33a59ac8f6254`**, 558 data rows;
copies/evidence in `board/probes/team-c/pack23/`.
**(2)** `board/probes/team-c/pack23/stageA-windows.csv` (862 rows) + the reconciliation finding below.

## ITEM 1 — PREDICTIONS CSV (`series,close_time,p_yes,fired`)

### Deliverable
- **Committed:** `pack23/data/predictions.csv` on branch `pack23-inputs` of `cayleb-james2008/kalshi-shadow-data` (local
  repo `~/Work/kalshi-shadow-data`, commit **`3f59b9b`**; the repo is local/not pushed per the pack's own header, so the
  commit is the handoff — Nox can push it). Header exactly `series,close_time,p_yes,fired`.
- **Row count: 558 data rows** (612 frame windows less 54 with no forecaster record — see coverage). Copy + sha256 above;
  also at `board/probes/team-c/pack23/predictions.csv`.
- **Source (forecaster's own records, nothing reconstructed or estimated):** `~/Work/projects/openbell-kalshi/live/kalshi/
  ledger/decisions.jsonl` (the live shadow forecaster's recorded decisions) + `ledger/orders.jsonl` (recorded fire flags).
  No imputation, no synthetic p, no market-implied proxy.
- **Conventions (documented so the verifier can judge the join):**
  - `series` = the pack's series key (`KXBNB15M`, …), derived from the forecaster's own ticker.
  - `close_time` = the window's `T1` from the forecaster's record, UTC ISO `…Z`, on the 15-minute grid.
  - `p_yes` = the **first recorded model probability in the window** (earliest decision tick with a non-null `p`). Recorded
    distribution of that slot across the forecaster's windows: open+1 min: 193 windows; open+2 min: 1,877; later: 9. The
    first tick after the open usually has `p = null` (`st = no_strike`), so the earliest available forecast is normally the
    open+1 or open+2 minute decision. Rationale: "earliest available forecast", analogous to the pack's earliest-snapshot
    convention; it is **not** the converged latest decision. (Slight timing asymmetry vs `market_earliest` at open+0:05 /
    +1:05 is a documented limitation, not hidden.)
  - `fired` = 1 if the forecaster recorded a fire in that window at the **early-read arm θ = 0.06, L = 30, X = 100** (any
    recorded terminal order event `place|fill|reject|late` with `th = 0.06`, `L = 30`); else 0. That is the only arm with a
    complete recorded fire set in the frame period and the arm the draft Stage A early read used. A different {θ, L} can be
    emitted from the same records on request — no guessing was done.
- **Coverage:** 558 / 612 frame windows scored; **54 have no forecaster record** — the frame's first 6 windows per coin
  (closes 11:15–12:30Z on 31 Oct) predate the shadow ledger's first window (close 12:45Z). Of the pack's **108 HEALTHY**
  windows: **90 scored, 18 missing** (same pre-ledger closes) → the pack script reports `n_model = 90` in the all scope.
  Nothing was estimated for the 54/18.

### Pack script result (run by me, `--model`, 3.3 s; evidence `probes/team-c/pack23/brier_table-with-model.csv`)
`python3 -I pack23/code/pack23_inputs.py --model pack23/data/predictions.csv` — fills the model columns; the committed pack
outputs are untouched (the script writes `pack23/data/model/`, which was regenerated for the record and then removed).
| scope | coin | n | market_earliest | **model** | n_model |
|---|---|---|---|---|---|
| all | POOLED | 108 | 0.2323 | **0.2391** | 90 |
| fired | POOLED | 77 | 0.2442 | **0.2558** | 77 |
Per-coin (all scope, market → model): BNB 0.2515→0.2937 · BTC 0.2242→0.2431 · DOGE 0.2418→**0.2136** · ETH 0.2284→0.2770 ·
HYPE 0.2180→**0.2101** · NEAR 0.2290→**0.2171** · SOL 0.2075→0.2144 · XRP 0.2523→0.2780 · ZEC 0.2377→**0.2047**.
**Market beats the model pooledly on both scopes** (all: 0.2323 vs 0.2391; fired: 0.2442 vs 0.2558) — consistent with the
1 Nov early read (0.2051 vs 0.2117), now on the frozen pack frame and the earliest-snapshot convention. Model wins DOGE,
HYPE, NEAR, ZEC per coin. No bar/gate changes; DEV only.

## ITEM 2 — STAGE A PER-WINDOW LIST + RECONCILIATION

### Deliverable
`board/probes/team-c/pack23/stageA-windows.csv` — **862 rows**, one per healthy settled window behind the draft's pooled
862: `ticker, coin, close_time (UTC), u (settle tick), y, fired, healthy=1`. Selection = the draft's own rule
(`forward.py look --freeze-at 2026-10-31T00:00:00Z --theta 0.06 --L 30 --X 100`, read-only over the live shadow ledger) at
the draft's own ledger cutoff **2026-11-01T18:22Z**.

### Reproduction — the 862 is exact
- **Pooled 862 healthy settled windows** (and 862 fired — see below), per coin: **BNB 111, BTC 108**, DOGE 91, ETH 86,
  HYPE 93, NEAR 97, SOL 95, XRP 94, ZEC 87. The draft's "BNB 111, BTC 108" reproduce exactly; its **"108–115 across the
  nine" does NOT** (actual 86–111) — that range was wrong in the draft.
- **Structural finding (this answers the pack's hypothesis):** in `forward.py` the window set is built only from
  `settle`/terminal records, and `shadow.py _settle` writes `settle` rows **only for windows that fired** at the arm
  (it loops over `w.fires`). Therefore the look's "healthy_settled_windows" can never count a non-fired window: the 862
  and its "fired 862" are the **same fired-window population**. The pack's stated hypothesis ("Stage A counted snapshot
  records, not windows") is **refuted** — 862 is a window count, of fired windows.
- **The all-window healthy count at the same cutoff** (from the shadow ledger's `ev:"window"` records: 1,071 windows,
  42 excluded) is **1,029 pooled**, 114–115 per coin. So the draft's "862 healthy settled windows" **conflated the
  fired-window count with the all-window count**; the true all-window figure at that cutoff is 1,029.

### Reconciliation against the pack frame (612 windows; 108 HEALTHY) — join on (series, close_time)
- Of the 862 Stage A windows, **454 fall inside the pack frame**: **77 = pack HEALTHY**, **327 = E1_NO_COLLECTION**,
  **43 = E2_INSUFFICIENT_COVERAGE**, **7 = E4_UNSETTLED_AT_CUTOFF**; **408 close outside the frame**.
- Reverse direction: of the pack's **108 HEALTHY**, **77 were fired** at that arm in the shadow record, **13 are scored by
  the model but not fired**, and **18 have no shadow record** (pre-ledger closes, see Item 1).
- Why the numbers differ (stated plainly): different **captures** (the pack's collector ran in four ~1-hour bursts; the
  shadow's desktop capture ran continuously), different **periods** (the frame is 68 windows/coin, 31 Oct 11:15Z–1 Nov
  04:00Z; the Stage A look ran to 1 Nov 18:22Z), and different **denominators** (Stage A = fired windows only; the pack =
  all windows in its frame). Comparing 862 with 612/108 is apples-to-oranges.
- **Pack frame unchanged:** 612 windows / 108 HEALTHY stand; no pack figure is contradicted. The correction belongs to the
  Stage A record, not the pack.

### Corrections to the record (proposed; for the Lead/Nox to land or reject)
1. The 1 Nov Stage A early-read DRAFT's **"862 healthy settled windows"** = *windows fired at θ=0.06/L=30 in the shadow
   ledger, freeze 2026-10-31T00:00Z → 2026-11-01T18:22Z* — **not** an all-window count.
2. **All-window** healthy settled at that cutoff = **1,029 pooled** (114–115 per coin; 1,071 windows, 42 excluded).
3. The draft's per-coin **"108–115 across the nine"** is wrong: fired per coin 86–111; all-window healthy 114–115.
4. No bar/gate statement changes: the 2,000/coin bar remains unmet on every reading (pack: 12 HEALTHY/coin, ≥118 days at
   this capture's health); freeze target Tue–Wed 3–4 Nov stands.

## Reproduce (read-only, stdlib)
- Model Brier: `cd ~/Work/kalshi-shadow-data && python3 -I pack23/code/pack23_inputs.py --model pack23/data/predictions.csv`
  (~3.3 s; writes `pack23/data/model/`; committed outputs untouched). Then `cat pack23/data/model/brier_table.csv`.
- Stage A list: filter `live/kalshi/ledger/orders.jsonl` for `ev ∈ {place,fill,reject,late,settle}`, `th=0.06`, `L=30`,
  `u ≥ 2026-10-31T00:00:00Z`, `(T1 or u) ≤ 2026-11-01T18:22:00Z`; healthy = the window's `settle.healthy`, settled =
  `settle.y` present; fired = any terminal event. Scripts/artefacts: `board/probes/team-c/pack23/` (`summary.json`,
  `predictions.csv`, `stageA-windows.csv`, `brier_table-with-model.csv`, `tables-with-model.md`).

## Fences
Shadow data only; no live orders, no trading-code changes (only the committed data CSV was added to the pack branch, as
ordered; pack code/data untouched, repo local/not pushed); no credentials read; $0. The pack's committed outputs remain the
`--model`-free versions; the model table is evidence here and regenerable by the command above.

— `trading-lead`, 2026-11-02T17:14 ET (22:14Z).
