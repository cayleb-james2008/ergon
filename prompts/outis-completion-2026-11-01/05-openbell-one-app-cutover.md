# OP-05 — OpenBell becomes ONE app (Cayleb's direct order, 31 Oct 22:47)

State: Cayleb: "I want it to just be merged into one app." The unified OpenBell repo is canonical (master 6f582f87, all PRs merged); the live meme desk still runs the desk code on the RSI box (momo, hourly self-iteration via the box route since 23:43 ET 31 Oct — the Grok chat route is capped). Cutover prechecks are written: goals/prop-firm-trading-workflow/files/openbell-cutover-prechecks-2026-10-31.md (copy on the team board at reports/openbell-cutover-prechecks-2026-10-31.md), including §3b.

Job (Team C builds; you drive/verify completion and own the desktop legs):
1. Unified app build with the meme desk inside it — same wallet, same ledger, one interface; futures desk and Kalshi arm in the same frame. Checker-passed before anything switches.
2. Switch plan to Nox BEFORE execution: stand-down and cutover as ONE operation (never two live traders), executed at a moment when the desk holds zero open positions; deployed exit rule verified via data/live/exit_rule.json + the startup config line (the log names no variant); ntfy topic rotation; Cloudflare Worker redeploy; ledger continuity proven across the switch.
3. §3b: after cutover, repoint the desktop host layer (memedesk-desk.service BindPaths) and verify, THEN archive the desktop openbell-meme-desk tree. JUP key carries over from the desk's existing config — never hand-carried, never printed.
4. The hourly self-iteration continues inside the one app, uninterrupted in cadence across the switch.

Done when: the unified app is the live trader (one live trader, verified), the ledger is continuous across the switch, the desktop tree is archived per §3b, and the first post-switch hourly version ships from inside the app. Fund boundaries unchanged throughout: existing desk wallet only, no new funds, one live trader.
