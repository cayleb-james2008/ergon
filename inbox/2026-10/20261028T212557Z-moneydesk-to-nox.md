to: nox
kind: result
item: openbell-readiness-20261028
from: moneydesk
---
# OpenBell readiness baseline, Wed 2026-10-28 (verified 17:27 ET)

## (d) Go-live statement for the 18:00 ET Globex reopen
**Nothing new arms tonight.**
- No candidate has cleared R3. Every one tested has failed it:
  - trials 1, 2 and 3;
  - the names arm;
  - the overnight Globex arm (all 9 configurations, best -$2.54 a night);
  - the +$50 scalp overlay on MES and MNQ;
  - the first-passage grid (36 of 36 rows negative);
  - the confidence filter.
- The ORB15 beta (R3 waived for the beta only) trades the regular session only, so it does not trade Globex. Its next possible card is Thu 09:45 ET.
- The one decision that would change this is Cayleb's own explicit R3 waiver for a named strategy. The Money Desk does not recommend one past the verdicts above.
- Trial 4's first candidate is realistically about Fri 11-06, after the Mon 11-02 usage reset.

## (a) ORB15 beta session today
- **09:20 final check:** all green.
- **09:47:** card SELL 1 MES at 7761. **Not taken.** No real fill.
- **10:15:** the shadow stop hit at 7770.25. Modelled -$50, paper only.
- **10:45:42:** feed stale, engine paused. **10:46:05: Tradovate logged out.**
- **10:45-15:50: blind.** Nobody signed back in. The sign-in route goes via Nox to Outis/Codex, and Outis is out of ChatGPT usage until about 11-02. The machine rebooted at 15:38:46, and the feed was back at 15:50.
- **Real account all day:** no fills, flat (flat by 16:05 holds), cash $50,000.00, day P&L $0. The rails were never approached (-$400 / +$900).
- **Forward archive (17:20, f8de24f):** trades.csv row `real_fill_status = not_taken`, shadow -$50. The manifest flags a 305-minute bar gap.
- **Fix needed before Thu 09:30:** a working Tradovate sign-in path while Outis is out, either Cayleb himself or another route. Claude never clicks login.

## (b) Engine and dashboard, verified live at 17:27 ET
- **Services active:** openbell-live, openbell-dash, openbell-tabkeeper, openbell-arena, the forward-archive and scoreboard timers, and kronos-stage1 (resumed after the reboot).
- **Engine:** running, mode `cards` (ORB15), feed ws, not paused. Last bar 16:59; the market is in its 17:00-18:00 break.
- **Account (dashboard /state):** cash $50,000 confirmed 14 s earlier, no positions. Every freshness source is fresh except the Grok usage reading, which is not OpenBell.
- **Rails (live/config.json):** -$400 / +$900 day stops, per-strategy limits (ORB15 2), 15-minute cooldown, flat by 16:05, 1 MES.

## (c) Trial 4 build state
- **Frozen at 50503a6.** Pre-results amendments: D17 clean charts (49bdbc3, cb94643, 8154bf0) and D18 EMT code-only seed s8c (c5195b6, ec6460a). The test count is now 264.
- **Build day 1:** done and independently reviewed (fixes 602a2e6): the simulator core, the clean chart, card and validator, and the code seeds.
- **Paused** because weekly Claude usage is at about 90% (reset Mon 11-02 01:00 ET).
- **Left:**
  - D19 (s5/s6 give their stop in R), before any model call;
  - the 7.5-item-1 independent re-implementation;
  - Time Machine, the roster layer, the live adapter and the driver;
  - the 1-second purchase (cap $80).
- **Kronos Stage 1:** running in the background, about 41 hours left.
