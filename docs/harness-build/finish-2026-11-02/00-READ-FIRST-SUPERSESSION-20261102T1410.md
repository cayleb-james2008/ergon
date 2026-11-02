# CORRECTION (14:25 ET), READ BEFORE ANYTHING BELOW: the message bus is NOT live yet

Section 2 below says the comms core is "done and installed". **That is wrong for the bus.** Verified on the live facade at 14:24 ET: `bus.status` and `bus.who` both answer `bus-unavailable: fleet-comms is not mounted on this host`. The six `bus.*` tools are LISTED (hence "15 tools") but the comms service behind them is not mounted in the installed profile: no participant registry, no mailbox, no tokens, none of the comms owner ops. What IS live and proven: `lane-wake`, `lane-start` and the idle watchdog on roster lanes (the Lead was woken by them; fresh checkers run on real lanes). What is NOT live: handoffs between participants, so nobody, including you, can be connected through the bus yet. A mounting patch (0.3.5) is being built and will be installed only after a fresh checker boots a scratch host and exercises the bus end to end. Treat every statement below about "the bus" as "built, not yet live". See `01-HOOKUP-delegator-fleet-bus-20261102.md` for the connection checklist and what blocks it.

---

# READ FIRST: what changed while you were dormant (supersession note, 2 Nov 14:10 ET)

From: claude-finish (the executing Claude session), on Nox's ruling of 14:09 ET. Your last turn ended at 10:39 ET; your session was re-seeded at 11:22 ET and nothing has woken it since. This note is what you wake up to. It supersedes the parts of Nox's order `154826Z` (10:48 ET) that are already done.

## 0. First action when you wake
Cayleb is home 14:30-15:00 ET. Display the privileged pack to him, open and copy-paste ready: `board/reports/owner-privileged-pack-20261102.md` (this folder). Items 1 and 2 on screen with every command; items 3 and 3b shown as HELD FOR SECURITY REVIEW, with no runnable command for them. A private fallback page exists too (https://claude.ai/artifact/L8kYLcYBQHqpoVTPZs2YPJ, opens only for Cayleb's account).

## 1. The base is 0.3.4, not 0.3.3
`@cayleb/dsh-fleet 0.3.4-consolidated` was installed in the consolidated profile at 10:40 ET (branch `release/0.3.4`, worktree `~/workspace/harness-build/finish-rel`; a fresh checker passed it; all install gates passed: evidence in `~/workspace/harness-build/finish-docs/INSTALL-0.3.4-GATES.log`). It contains the lane real-git fix, both of your interim hotfixes properly fixed with tests (your `brief.js` and `calls.js` edits are superseded; do not re-apply them), and the comms layer. **Do not cut anything from 0.3.3.**

## 2. Built and installed; the bus is NOT yet mounted (see the correction above)
Order `154826Z` asked for "comms core wired into the package" and "T1/T2 wake proof with a real parked lane under a fresh checker". Both are done and live:
- The comms core (participant registry, presence, handoff mailbox, per-participant hashed tokens) ships as the six `bus.*` facade tools. **The facade is nine core tools plus six bus tools = 15, closed.** Any further tool needs a named gap and Nox's ruling.
- `lane-wake`, `lane-start` and the idle watchdog are live. Evidence: the watchdog woke the idle Lead on its own (events 434, 450 and later), `lane-wake` returned `woken: true`, and fresh checkers verify tasks on real lanes.

## 3. The live remainders of `154826Z`
- **Owner Elevation:** the Lane 6 security review (hard gate) and the Claude-lane helper build prep. The design and threat model are filed and reconciled: `finish-docs/leg4-owner-elevation-design.md`, rulings R1-R4 and E1 in `finish-docs/RECONCILE-DECISIONS.md` (on the team branch under `docs/harness-build/finish-2026-11-02/`). No code is built; nothing privileged runs while Cayleb is away.
- **Face P1 data surface** (fleet profile only, never the owner's personal profile). The leg-1 design is in the same folder. Order `o1` is already delivering the sidebar and VM-panel units; see section 4.
- **Duet courier** (Drive mailbox, B2 wording): the design is the leg-3 document; it builds on the comms bus above and waits for Nox's rulings R2 and R3.

## 4. `o1` is NOT yours
`o1` is run by the executing Claude session under Nox's ruling. The earlier duplicate orders `o2` and `o3` were created and cancelled; **create no successor orders.** Order-identity changes (create, cancel, replace, change a cap or a done-when) go through Nox before they are made. DeepSeek V4.1 Flash calls on the OpenCode Go route are not money-gated for testing (owner, 12:51 ET), so do not hold work for per-order spend confirms.

## 5. The privileged pack: items 3 and 3b are HELD
- Item 3 (a sudoers rule for `fleet-dsh`): no Lane 6 review on record; the first question is whether polkit already makes it unnecessary.
- Item 3b: remove the stale V3 identity drop-ins (`/etc/sudoers.d/dsh-identity-v3`, `/etc/polkit-1/rules.d/49-dsh-identity-v3.rules`); filed as Case 3 in Nox's security settlements registry.
- Both go to **Lane 6 (Team E) in one request**: `inbox/2026-11/20261102T184814Z-claude-finish-to-delegator.md` and `…T185442Z…` in your inbox. If you can start Team E, do; if Lane 6 clears neither item before the owner's window, the pack says so.

## 6. Standing rules (unchanged)
No merges to any main branch without a fresh-checker PASS plus Nox's order. No policy changes by any route until Owner Elevation exists; hand-placed confirms happen only under Nox's explicit ruling. No spend outside the existing plan, no public posts, no credential sharing, no live trades or orders.

## 7. Dormancy map (what did not move after 10:39 ET)
Machine evidence: your board folder had 210 file changes between 06:00 and 10:39 and none after. Team A (Ergon repos): no file changes since 31 Oct, so the tiered-plan build has not moved. Team B (fleet source): none since 31 Oct. Team C: the live OpenBell desk kept running on its own, but the executor build shows no changes. Team D: no Roblox build changes. Team E: nothing since 07:31. Resume these in your own order; this list is only the picture.
