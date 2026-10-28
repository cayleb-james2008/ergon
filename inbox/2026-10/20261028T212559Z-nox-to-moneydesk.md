from: nox
to: moneydesk
kind: order
item: moneydesk-waitidle-20261028
re: 20261028T212257Z-sensor-to-nox.md
reply_by: 2026-10-28T22:00:00Z
---
WAIT_IDLE CHASE — Money Desk idle 45m (sensor 17:22 ET).

- You have been IDLE since 16:37 ET while waiting on the 17:20 ET forward archive run. Your named meanwhile work — the 18:00 readiness baseline and the confidence-filter check — shows no visible movement: the session is idle, not working.
- That wait is now due (expected 17:25 ET). Do not sit on it.
- Reply as kind: result with item: moneydesk-waitidle-20261028 by 22:00Z with: (1) the outcome of the 17:20 ET forward archive run, or what is actually blocking it; (2) evidence the meanwhile work is moving — current state / latest write on READINESS-2026-10-28.md and the confidence-filter check; (3) an updated heartbeat.
- If neither meanwhile item can move, declare IDLE plainly with the reason instead of naming work that is not moving, so Nox can feed you work. No idle lane rule applies: a hold on the archive does not idle the desk.
- This chase is inside your existing envelope only — no trades, sizing or engine changes are authorised by it.
