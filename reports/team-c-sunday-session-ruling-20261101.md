# TEAM C — SUNDAY GLOBEX SESSION / ORB15 MAPPING — RULING (1 Nov 2026)

trading-lead, 2026-11-01 13:3x ET. Answering Nox 182800Z item 1 (owner pulled the Monday stack forward): **does the
ORB15 beta design map onto a Sunday-evening Globex session, and may the desk take a first autonomous session tonight?**
Read-only evidence; **no order, no credential, no live action, no spend** — and, as ruled below, **no session tonight**.

## RULING: **IT DOES NOT MAP — ORB15 is a regular-session instrument by construction. MONDAY STANDS.**
No autonomous session tonight. Re-anchoring ORB15 to the 18:00 ET Globex open would be a **new, unvalidated strategy**
(there is no preregistration or forward record for a Globex-open opening range), which the desk's own arm-live rule
forbids. The first live autonomous session remains gated behind G3/G4 + Cayleb's written go; Monday is cards/guard only.

## 1. Why it does not map (code, not opinion)
| Fact | Evidence (file:line) |
|---|---|
| ORB is defined on the **first N minutes of the RTH range** and returns **no signals outside RTH** | `bt/strategies.py:121` class `ORB`: *"Opening-range breakout: close outside the first N-minute RTH range"*; `if not f.rth: return []` |
| RTH is hard-anchored to **09:30–16:00 ET** | `bt/features.py:23-24` `RTH_OPEN = 9*60+30`, `RTH_CLOSE = 16*60`; `:72` `rth = (m >= RTH_OPEN) & (m < RTH_CLOSE)`; `opening_range()` = *"the first `minutes` RTH minutes"* (`:117-121`) |
| Entry window is RTH **09:46–12:00** | `bt/strategies.py` ORB: `if mc <= RTH_OPEN + n or mc > hhmm(last_entry)`; live config `last_entry: "12:00"`, `n: 15` |
| Time exit / flat | ORB signal exit hint `"15:55"`; live config `flat_et: "16:05"`; engine `gate()` blocks entries from 16:05 until 18:00 (`live/engine.py:181-190`) |
| Sunday 18:00 ET is the **Globex week open**, not the RTH open | firm FAQ (quoted in `team-c-firm-automation-verdict.md` §3): *"resumes 5:00 PM CT"* = 18:00 ET; *"all positions closed by 3:10 PM CT = 4:10 PM ET"*, no overnight/weekend holding |
So at 18:00 ET Sunday the strategy sees `f.rth == False` for every bar until Monday 09:30 ET: **zero possible ORB signals**.
The desk's session-day accounting *is* Globex-aligned (engine `_roll_day` rolls at 18:00 ET via `et_info(...)[2]`),
but the strategy is not — accounting maps, the instrument does not.

## 2. Why a Sunday session is impossible tonight even apart from the mapping
1. **No autonomous execution path exists/armed.** The only autonomous build is the contained executor at G2, and G2 is a
   **dry-run-only** package (`mode: dry-run`, no enable artefact, no CDP transport, no order path; G2 signed with
   conditions, G3/G4 unsigned, **no Cayleb written go**). The live engine has **no autonomous mode** (only
   `shadow | cards`; cards = human-placed brackets). So "the desk takes its first autonomous session tonight" cannot
   happen inside the gates — the decision to run it is not open to me.
2. **No feed, no bars, no tab.** At 13:2x ET: heartbeat `feed="retrying (RuntimeError)"`, `feed_age_s≈163,229`
   (~45 h), `market_open=False`, `last_bar_et=16:59` (Friday); `live/bars/2026-11-01.jsonl` and
   `live/ledger/2026-11-01.jsonl` do not exist; the Chromium `/json/list` has **no `trader.tradovate.com` tab**
   (`tabkeeper.json: tradovate="missing"`). `tabkeeper` starts Chromium only on **weekdays 08:30–16:00 ET** and
   **never signs in**; a fresh sign-in is a Cayleb act. Without a tab there are no bars → no signals of any kind.
3. **ORB15 is not record-cleared for live trading.** `live/config.json` `strategies[0].role`: *"shadow plumbing check
   only: over 2016-2024 WF-OOS this config lost $4.31/trade, PF 0.81, 0 of 9 years positive (reports/ORB-RESULTS-2026-10-27.md,
   T4); **do NOT trade its cards**"*. The desk's arm-live rule is "a candidate arms live only when its record says it does
   not lose after costs". This is a **material finding for the Lead/Nox** (see §4) — it does not change tonight's answer
   (no mapping), but it bears on any live/autonomous ORB session.

## 3. Decision (Team C, inside the rails)
- **No session tonight.** Sunday 18:00 ET is observed as a market open only; the desk stays `mode=cards`, flat, no bars,
  no cards. Nothing is manufactured.
- **Monday stands unchanged** (packet v7): 01:00 ET Kalshi resume → 07:45 sweep → 09:05 guard (+ R11 CME-limit record)
  → 09:15 five attestations ((e) `mode=cards — NOT automated`) → 09:35–09:44 binding → 09:45 window → 16:05 flat →
  16:20 outcome. No autonomous fills claim.
- **Re-entry condition for a Globex-open ORB (if ever wanted):** a new preregistration for a Globex-open opening range
  (different anchor, session, sample) + its own frozen forward record + the normal G3/G4 gates. ORB15's RTH record cannot
  be transferred to a Sunday-open anchor.

## 4. Material finding to carry (not tonight's blocker, but needs a ruling before any live ORB)
The ORB15 config's own role note says **do not trade its cards** (WF-OOS −$4.31/trade, PF 0.81, 0/9 positive years), while
the beta booking assumes live ORB15. Before any live/autonomous ORB session this contradiction needs a Lead/Nox/Cayleb
ruling (run the beta as a deliberate exception, or hold it). Monday's cards/guard day is unaffected (it is human-tap and
begins with the guard; no fills are claimed).

## 5. Evidence / reproducibility
Commands run 2026-11-01 13:20–13:30 ET: `live/engine.py` + `bt/strategies.py` + `bt/features.py` reads (quoted above);
`python3` heartbeat dumps; `ss -ltnp | grep :9222`; `curl 127.0.0.1:9222/json/list` (no Tradovate page); `ls live/bars/`,
`ls live/ledger/`; `live/logs/tabkeeper.json`; firm FAQ quote from `team-c-firm-automation-verdict.md` §3; config role
string. Fences: no order, no credential, no live action, no spend; no engine/config change.

## POST-SIGN-IN / WRITTEN-GO ADDENDUM — 2026-11-01T19:21:33Z (14:21:33 ET)

Post-sign-in facts (owner signed in 13:59 ET; detector RESTORED 18:59:11Z) and the owner's written go (Nox 184800Z)
do not change the ruling; they narrow what it permits. Read-only; no order, no credential, no config change.

**(i) Mapping conclusion UNCHANGED.** ORB15 is RTH-anchored by construction (`bt/features.py` RTH 09:30–16:00;
`bt/strategies.py` returns no signals when `not f.rth`; entry window 09:46–12:00; flat 16:05). Sunday 18:00 ET is the
Globex week open; every bar until Monday 09:30 ET has `f.rth == False`, so **zero ORB signals before Monday 09:30 ET**.
Restoring the feed does not create a Sunday opening range — the RTH anchor is still missing.

**(ii) The written go now exists but cannot be exercised.** `live/config.json.mode="cards"` and
`live/heartbeat.json.mode="cards"` (notify-only; a human places any bracket). There is **no autonomous mode** to switch
on; the only autonomous build (G2) is dry-run only and **G3/G4 remain unsigned**. Therefore **neither tonight nor
Monday can be an order-capable autonomous session** until G3/G4 are signed — the go authorises autonomous operation in
principle, not against an engine that has no such mode. Monday stays cards/guard (human-tap), exactly as ruled.

**(iii) Material finding stands and is carried.** `live/config.json` `strategies[0].role`: *"shadow plumbing check only
… lost $4.31/trade, PF 0.81, 0 of 9 years positive … do NOT trade its cards"*. This contradicts using ORB15 cards as a
live beta signal. It needs a **Nox/Cayleb ruling** (deliberate exception, or hold) **before any live ORB session**;
tonight's observation and Monday's cards/guard day are unaffected, and no fills are claimed.

Witnessed 19:21:33Z: broker `attached` / `sync_t=13:59:02 ET`, flat $50,000.00, engine heartbeat 4.0 s, feed `ws`
`feed_age_s=1345.4` (market closed), ledger 8 rows, no signal. Feed tick proof remains pending the 18:00 ET open.
