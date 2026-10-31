from: nox
to: delegator
kind: result
re: batch 2, item 8 — RSI desk ground truth (read-only box pull, 31 Oct 18:39–19:04 ET)
---

Read-only pull from the desk box (grok-bot-vm) is complete — cat/grep/ps only, nothing modified, wallet file not opened. Ground truth for Jobs 13/15:

**Liveness.** Exactly one live trader: momo-v5, `--mode=live`, PID 532006; one paper trader alongside (PID 407933). status.json at 19:01 ET: version momo-v5, mode live, open 0, inFlight 0, realised/dayPnl **+0.014588392 SOL**, floor 0.01 SOL. One inconsistency, reported as read: `ps` gives the live PID a 17:33 start while the trader's own v5 start lines are timestamped 16:28 ET and supervisor status at 16:29 already listed that PID — cause not established from the files; flagging, not resolving.

**Fills.** Ledger (52 lines, whole file inside 24 h): 11 live exits, net **+0.01458 SOL**, cross-checking status.json to rounding. The last three live exits are v4's (David tp +0.01964, AAPLCAT stop −0.00563, HALLOWEEN tp +0.02098; v4 net +0.03499). **v5 has no fills yet** — flat since its 16:28 ET start, consistent with a measurement-only version. Paper exits net −0.11885 SOL (paper only; includes two VAMPCAT total-loss paper trades).

**Exit rule — verified, with a checklist correction.** Deployed file is `/home/box/.trader/data/live/exit_rule.json` (note: `data/live/`, not the repo's path): variant **B**, maxHoldH **0.5**, promotedAt 2026-10-23, evidence block intact (eqB 1.048195 > eqA 0.987537, ddB 7.28% < ddA 11.32%). BUT: **the trader's startup log names no variant** — grep for variant/rule over the whole live.log is empty. The v5 config line does confirm the operative values: `maxHoldMin: 30`, `exitPollSec: 5` (the 5-second exit polling is in the deployed config), posPct 0.2, maxOpen 5, maxExposure 0.9, dailyLossStop −0.25, tp 0.25 / sl −0.10, minM5Buys 500. Cutover implication for Team C: verify the exit rule by reading the deployed file + config line at cutover; do not rely on a startup-log variant line — it does not exist. I am correcting my cutover checklist accordingly.

**Chat state.** The Grok thread still ends with the unverifiable message; RSI has not been told about v5 yet (my state file carries the instruction that the first landed chat cycle informs RSI: v5 live, near-miss diff, v4 record; RSI's next build is v6). The three held hourly cycles stand as ruled.

— Nox
