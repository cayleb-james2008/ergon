# Incident: Team B's lead killed the live OpenBell dashboard (cross-lane fence breach), 2 Nov 2026

**Class:** cross-lane fence breach (a harness-face team acted on a trading-side process). **Impact:** the OpenBell desk dashboard was down about 19 seconds; nothing else changed. **Reported by:** the executing session, from the lane session logs (read-only). **Ruling:** Nox, 16:29 ET: Team B's scope is the harness face; it does not touch OpenBell processes (not the dashboard, not the desk, nothing on the trading side). The fence is explicit and standing.

## Timeline (UTC; ET is UTC minus five hours)
| time | event | source |
|---|---|---|
| 20:30Z | the delegator's hold on Team B (the face and Duet courier prep) is in force | delegator log |
| 21:15:27Z | the delegator's RESUME reaches Team B (`fleet-lead`) after Nox's retraction | delegator log |
| 21:19:21Z | Team B's scratch fleet-host config shows MCP `port: 8791` (its own scratch host) | Team B tool result |
| 21:19:21Z | its scratch host restart is "killed by signal: SIGTERM" | Team B tool result |
| 21:20:31Z | it lists "stray scratch procs" (processes of its own scratch home) | Team B command |
| **21:20:41Z** | **in ONE command it finds the pid listening on :8791 and sends SIGTERM to it**: `ss -ltnp | grep :8791` -> pid -> `ps` printed -> `kill -TERM`. The pid was 1481, `python3 -m live.dash --account --port 8791 --tailnet 100.114.17.67,outis-desktop,outis-desktop.tail1b30fd.ts.net`, the OpenBell desk dashboard, up since 28 Oct | Team B command and its output |
| 21:20:53Z | it searches for the process's unit | Team B command |
| **21:21:00Z** | it starts `openbell-dash.service` ("restore openbell-dash.service (my stray-kill cleanup)"); new pid 3823710, same arguments, listening 127.0.0.1:8791 and 100.114.17.67:8791 | Team B command, my read of `ps` |

## Cause
1. **Port collision by choice.** Team B configured its scratch fleet host to listen on 8791, a port the OpenBell dashboard already owns.
2. **Kill by port, identity read and kill in one command.** It treated "whatever listens on 8791" as its own leftover. The `ps` line was printed in the same command that had already sent SIGTERM, so the identity could not stop the kill.
3. **No standing fence** told the harness-face team that trading-side processes are off limits (now ruled).

## Desk state verified after the kill (read-only, 21:29-21:30Z = 16:29-16:30 ET; nothing on the desk was touched or started by me)
- **Dashboard:** `openbell-dash.service` active since 16:21:00 ET; HTTP 200 on 127.0.0.1:8791 and on the tailnet address 100.114.17.67:8791; running with the same arguments as the killed process.
- **Engine and feed:** `openbell-live.service` active since 28 Oct 15:40 (never restarted by this). `heartbeat.json` rewritten every few seconds (age 3.4 s at 16:29:51 ET): status running, mode cards (observation), feed ws, feed_age 0.1 s, last bar 16:28, `pos None`, `paused False`, trades 1 and day_pnl 72.5 (the 09:46-10:15 shadow card trade Team C reports; no broker counterpart).
- **Account:** `account_state.json` rewritten seconds ago (frames 29,180 and rising): status attached, source dom, equity $50,000.00, open PnL $0.00, no positions. No change from Team C's 16:16:52 reading ($50,000 / SOD $50,000 / realized $0.00).
- **Cards:** the last card is "OpenBell feed OK - engine resumed" at 16:16:54 ET; none was raised by the kill.
- **Other desk units:** arena, kalshi poll, ref and shadow, tabkeeper all active; forward-archive inactive (a timer-driven unit, not checked further).
- **Team C's own reading** (`team-c-monday-day-20261102.md`, filed 16:22 ET, after the kill): day closed, ORB15 HOLD, 0 broker fills, flat verified at 16:16:52 after a 13:30-16:16 broker-session logout it did not cause. Its reads predate nothing the kill could change except the dashboard's account watcher, which is back.
- **Not done:** I did not ask Team C or the delegator for a fresh reading and did not look at the broker page itself; the checks above are from the desk's own state files and units.

## Follow-ups
- Team B: no OpenBell process, port or unit is within its scope. It should pick scratch ports from outside the desk's range and verify a process's identity BEFORE any kill (never kill by port).
- The package rule from Nox (3 of 3): the next package is built on 0.3.5.1; Team B rebases the face work onto 0.3.5.1 first; and the installer self-check must prove `validate.js` survives before any install is proposed (carried into the Option B package plan).
