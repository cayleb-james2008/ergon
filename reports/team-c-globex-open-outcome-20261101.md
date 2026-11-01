# TEAM C — GLOBEX OPEN: TICK PROOF + OBSERVATION OUTCOME — Sun 2026-11-01 18:00 ET open

Owner: `trading-lead` (Team C). Tier S job 12. **Read-only**: no desk process, config, wallet or rail touched; no order,
no card, no credential. Evidence read at **2026-11-01T23:34:36Z–23:43:55Z** (18:34–18:44 ET EST).
This is the post-open outcome filing requested by Nox 20261101T233413Z §5 (chase: "the post-open outcome filing — the feed
tick proof and the observation outcome record"). Companion probe with the raw field-by-field proof:
`board/probes/team-c/tick-proof-20261101T2343Z.md`.

## Verdict at a glance
- **TICK PROOF: PASS.** The 18:00 ET (23:00Z) Globex week open produced a fresh first bar at exactly 23:00:00Z with 815
  tick-prints; 43 further minute bars followed through 23:42Z; feed attached (`ws`, `feed_age_s` 0.1–0.8 s); engine
  heartbeat < 1 s old; `market_open=true`.
- **OBSERVATION OUTCOME: COMPLETE — NOTHING WAS TRADED.** No signal, no card, no order, no fill, no position. That is the
  designed outcome, not a failure: the engine has no order path (`mode=cards` = notify card + human-placed bracket), the
  strategy is RTH-anchored and returns zero signals outside 09:30–16:00 ET, and the ORB15 card contradiction is still
  unruled. Details in §3–§4.

## 1. What the open did (market)
- First bar: **23:00:00Z = 18:00:00−05:00**, o 7626.5 / h 7638.75 / l 7626.5 / c 7636.0, **v 3219, n 815**, `src tv-ws`.
- Through 23:42Z: 43 consecutive one-minute bars, all `tv-ws`, range **7625.5 – 7642.5**, last close **7642.5**
  (23:42:00Z); per-bar tick-prints 146–815. Latest heartbeat `last_price` 7641.5 / 7641.5 (23:43:15Z read).
- Freshness: `heartbeat.json` `t` age **0.16 s**, `et` `2026-11-01T18:43:15-05:00`, `feed "ws"`, `feed_age_s 0.8`
  (`0.1` at 23:38Z), `market_open true`, `last_bar_et "18:42"`, `bars 4947`, `pos null`, `paused false`, `mode cards`.
  File mtimes: heartbeat 18:43:15.73 ET, bars 18:43:00.60 ET, `account_state.json` 18:43:12.89 ET (`frame_t` age 1.2 s),
  `ledger/account-2026-11-02.jsonl` 18:43:06.92 ET. Full table in the probe file.

## 2. What the desk observed (account + day boundary)
- **Broker session live**: `live/account_state.json` `status="attached"`, `source="live"`, `tab_path="/"`. Account
  `71223147` (FN\*\*\*2147): `amount 50000.0`, `amountSOD 50000.0`, `realizedPnL 0.0`, `tradeDate {2026,10,26}` (the
  venue's stale field, recorded not smoothed). `dom`: `open_pnl 0.0`, `equity 50000.0` (age 9.2 s at the 18:40:13 ET read).
  `positions {}`. `attached_t` 13:26:29 ET (tab attachment), `sync_t` 13:59:02 ET (the sign-in sync; static because the
  venue only emits a balance on change/load), `frame_t` 18:40:13 ET (**fresh** — the WS frame clock is current).
- **Desk day rolled at the Globex open**: `live/logs/engine-2026-10-28.log` line
  `2026-11-01 18:01:00,109 openbell.engine INFO session day 2026-11-02: pnl 0.00 trades 0`; the new day's account frame
  file `live/ledger/account-2026-11-02.jsonl` was created (125 frames by 18:33 ET, all `kind:"contract"` WS update
  events; **no** `cashBalance`, `fill` or `position` frames yet — nothing traded).
- **Tab**: `live/logs/tabkeeper.json` at 18:35:19 ET `tradovate: "ok"`, `alerted: false`; CDP `127.0.0.1:9222`
  (`/json/list`) lists the `https://trader.tradovate.com/` page (live desk route).

## 3. What did NOT happen, and why
| Did not happen | Why (evidence) |
|---|---|
| **No signal** | ORB15 is a regular-session instrument: `bt/features.py:23-24` `RTH_OPEN=9*60+30`, `RTH_CLOSE=16*60`; `bt/strategies.py:118` class `ORB` — *"Opening-range breakout: close outside the first N-minute RTH range"*, and its `on_bar` returns `[]` at `if not f.rth` (`bt/strategies.py:133`). Every bar from 18:00 ET Sunday until 09:30 ET Monday has `f.rth == False` → zero possible signals. |
| **No card** | Day-filtered `live/logs/cards.jsonl`: `cards_today(2026-11-02) = 0` (26 cumulative cards in the file, none today). |
| **No decision record** | `live/ledger/2026-11-02.jsonl` (the engine's decision ledger) **does not exist** — it is created lazily on the first signal; absence = 0 signals. |
| **No order / no fill** | The engine has **no order path**: `live/config.json` `mode="cards"`; the engine's own contract is "shadow ledger + ntfy phone cards. It NEVER places, modifies or cancels orders." Cards are information for a human. No autonomous executor is armed: G2 is signed **dry-run only** (`live/exec`, commit `0893497`, digest `32dd2bea…`), G3/G4 unsigned. |
| **No position** | `heartbeat.pos = null`; `account_state.positions = {}`; `dom.open_pnl 0.0`. |
| **No trade taken on a card** | None issued; and the ORB15 config's own role note still says *"…lost $4.31/trade, PF 0.81, 0 of 9 years positive … **do NOT trade its cards**"* — the Nox/Cayleb ruling on that contradiction is still open, so no card may be human-tapped live until it is ruled. |
| **No Sunday session** | Already ruled (`board/reports/team-c-sunday-session-ruling-20261101.md`): re-anchoring ORB15 to the 18:00 ET Globex open would be a new unvalidated strategy. Monday stands; this open was observed, not traded. |

## 4. Observation outcome record (the filing)
- **AT 18:00 ET / 23:00Z the Globex week opened and the desk's feed attached to it cleanly** — first fresh bar exactly at
  the open with 815 tick-prints, 43 minute bars by 23:42Z, no feed error in the window, heartbeat continuous.
- **The desk observed the open in `mode=cards`, flat, `pos=null`, 0 signals, 0 cards, 0 decisions, 0 orders, 0 fills.**
- **Nothing was touched to produce this.** No engine restart, no config edit, no mode change, no wallet op, no order, no
  credential; the only actions were reads (`heartbeat.json`, `bars/2026-11-01.jsonl`, `account_state.json`,
  `ledger/account-2026-11-02.jsonl`, `logs/`, `tabkeeper.json`, `curl 127.0.0.1:9222/json/list`).
- **What it proves for Monday**: the broker session, feed, day-roll and logging surfaces are live and demonstrably current
  under the post-DST clock (EST, UTC−5). It does **not** pre-pass any Monday attestation — Monday's five attestations are
  taken fresh on the day per `board/reports/monday-futures-runbook.md`.
- **Open item carried**: the ORB15 "do not trade its cards" contradiction needs the Nox/Cayleb ruling before any live ORB
  session (including any human-tapped card). No autonomous fills can occur until G2.5 → G3 → G4 are signed.

## 5. Fences / reproducibility
All commands run from `~/Work/projects/openbell` / `~/Work/projects/openbell/live`; every number above is quoted from the
files named, timestamped 23:34:36Z–23:43:55Z. No spend ($0). No image reads, no host input injection, no credential value,
no order, no wallet operation, no rail/threshold change. `mode=cards` is NOT automated — no report here claims an
order-capable session.

— trading-lead, 2026-11-01T23:44Z
