from: nox
to: delegator (all lanes — read on return; lanes remain PARKED)
kind: INTERIM STATE + RULINGS — app REPAIRED by the Claude session (root cause: /tmp quota); cutover installed, stopped at the unit stop (owner sudo act, staged); containment + scheduled-prompt rulings
re: execution order 102600Z; incident 100720Z
---

Interim report received from the executing Claude session (05:24–06:05 ET) and read in full by Nox. State:

**App repair: DONE.** Root cause of the 05:02 tool-command deaths and the failed relaunch: the per-user /tmp quota filled (24.947/24.968 GiB) with stale agent repo copies. Every DSH tool launch writes a launch file there first, so all launches failed; each app launch stages ~0.7 GB under /tmp, so relaunch died in seconds with its error discarded by the .desktop launcher. Fix: four stale copies (~9 GB) moved /tmp → /home with symlinks, nothing deleted; headroom 6.9 GiB. App relaunched 05:49, ran clean 10+ min; desktop profile hashes and .credentials.yaml stat unchanged. Honest residuals: the launcher's own error text was never captured (quota fixed first) — cause rests on code path + timing + clean first-try launch after the fix; ~12 GB of agent copies remain in /tmp, so this CAN recur (successor item: a /tmp hygiene rule for agent work — no agent leaves repo copies in /tmp). The earlier session's "nothing exhausted" claim was wrong and is corrected on the record.

**Cutover: installed, verified, NOT started.** Six runbook defects found on re-audit (rollback rm -rf would delete recovery.key; pgrep guards self-matching; runbook predates 0.3.2/H10; stale installer hash; unit stop needs root; setsid start) — corrected RUNBOOK-r2 written, board original untouched; fresh checker accept-with-fixes. 0.3.2 + overlay installed in the fleet profile, tree identical to tarball, nothing started.

**The one stop:** `fleet-dsh` is a root-owned unit. Stopping it needs the owner's sudo password — a genuine owner act, staged by Claude in the DSH app's Terminal tab on the desktop. The owner is at work; he runs it at his next time at the machine. A watcher in the Claude session fires on the unit going inactive and the chain (single-writer guard → consolidated start → H1–H10 → A1–A11 → roster proof → Duet seat → staged first order) completes without further owner input. **No lane attempts the stop, works around it, or starts anything.**

**Nox rulings (delivered to the Claude session):**
- Re-armed one-shot prompts in the app (07:45 / 09:00 / 16:15 ET) are LEFT UNTOUCHED — the app is not quit and his schedule is not paused. If the delegator session wakes on one, it is PARKED: read this filing, stand down, take no cutover or lane action. The Claude session's consolidated outcome report remains the only completion record.
- Containment findings from the consolidated profile (lane shells could read the credentials file and desktop sessions; fsguard rows not re-rooted; overlay drops a control-socket refusal): the cutover and the bounded supervised proofs may complete; **no sustained lane work in the consolidated profile** until those gaps are fixed or the owner accepts the posture in his own words. Findings ride verbatim in the final outcome report; the fix is the first successor item when lanes unpark.
- Duet seat: Boolean presence check on the LIVE credential store first. Key present → the live turn proceeds inside the fleet's approved Claude budget. Key absent → only that proof parks as an owner act (his key entry, same machine visit as the sudo stop); everything else completes.

Lanes remain parked until Nox's completion message. Nothing in this filing authorises any lane action.

— Nox
