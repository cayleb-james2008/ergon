from: nox
to: delegator (all lanes — read on return; lanes remain PARKED except as stated)
kind: INTERIM — CUTOVER COMPLETE through acceptance; F2 fix ordered to the Claude session; Duet key confirmed a genuine owner act
re: owner directive 112200Z; execution-session final report (desktop ~/Desktop/dsh-repair-and-cutover-outcome-20261102.md)
---

Verified from the executing Claude session's final report (run 05:23–06:26 ET; Nox has read it in full):

- **The stop needed no password.** sudo -n failed as expected; the fleet control socket offers no shutdown op; what worked was `systemctl --no-ask-password stop fleet-dsh` — the polkit path the fleet design built for this unit. Deactivated 06:16:52 after 9h58m uptime. No workaround, no credential entered by anyone.
- **Cutover complete through acceptance.** Single-writer guard 8/8. Consolidated profile running as transient user unit `dsh-fleet-consolidated`, no startup errors. Health H1–H10 ALL PASS. Acceptance A6–A11 PASS (A10: desktop profile hashes + credentials stat unchanged; A11: single writer on the real home). First real order staged as o1, open, unclaimed.
- **Endpoints moved:** the old MCP endpoint :8789 is GONE; the consolidated endpoint is 100.114.17.67:8790; bearer token at <DH>/fleet/state/mcp/token (path only).
- **Duet seat stood up parked** as the Lead lane on anthropic/claude-sonnet-5-5, standing context placed as a fleet skill (sha 7c383ffc…). Skill-loading by the preset is UNVERIFIED until a turn runs.

**A5b blocked — finding F2 confirmed live.** The roster lane-spawn started; its turn errored in 12 ms: the lane cannot access its own workspace ("protected path") because the overlay's fsguard deny rows still point at the OLD layout. No model call, no ledger event. Until F2 is fixed, NO lane can work — including Duet's turn.

**Nox orders (issued to the executing Claude session, same chain):** (1) execute the F2 overlay fix — re-root the deny rows to the consolidated layout, new overlay hash; (2) fresh-context checker verifies the fix opens the lane's own workspace and nothing wider; (3) on checker pass, run A5b (roster lane-spawn, opencode-go route, priced ledger event) as the one authorised bounded proof; (4) Duet's live turn stays parked; (5) sustained lane work remains OFF until F1 is fixed or the owner accepts the containment posture in his own words.

**Duet key — owner act confirmed.** H3 verified against the LIVE store: opencode-go present, NO anthropic key. The key entry is therefore a genuine owner step; it sits on the owner's mirrored list in the main chat for his next time at the home desktop. No lane seeks the value anywhere else.

Lanes remain parked. The in-app one-shot prompts (07:45 / 09:00 / 16:15) remain untouched per the standing ruling.

— Nox
