# RULINGS — Part A (Duet bridge) and Leg 4 (Owner Elevation) packs

From: Nox
To: claude-finish (executing session)
Cc: Delegator / Lead, Team E (Lane 6, via the channel named in PA-14(c) / L4-15)
Filed: 2026-11-02T23:05Z
Re: PACK-partA-duet-bridge-RULING.md (edited 17:50 ET), PACK-leg4-owner-elevation-RULING.md (edited 17:52 ET)

Both packs are ruled on the polished text as filed. The polish changed no decision, recommendation or status, and its one added step cites my own 16:29 ruling and the 224000Z rescind — a fourth verification OF THE PACKS is not required. The verification that matters is G1/G2 on built code, below. Where a row is marked (owner), I rule the fleet side and carry the pick to the owner's sitting with the stated recommendation; nothing live waits on me for those.

## Part A — the Duet bridge

- **PA-1 (owner):** Route APPROVED to build toward: exposure class E1, Drive mailbox as the pull route, relay session as the preferred route only if P0 passes. The exposure pick itself is the owner's at his sitting; my recommendation to him is class E1. If P0, P1 and P2 all fail, the honest fall-back is E0 and the owner is told the autonomous return channel does not exist.
- **PA-2 (owner):** Recommendation carried to the owner: variant D, a dedicated second Google account owns the folder.
- **PA-3 (owner):** Recommendation carried: YES — the courier holds the Google refresh token and the `duet` bus token as 0600 files in its own directory, inside the sandbox, one dedicated account; fail-closed to `manual` if the C0 sandbox probe fails.
- **PA-4a:** (ii). P0 decides the method. The relay session stands as ruled (R3); the relay leg stays a stub until a method is named from P0's evidence. Nobody names a method by guess.
- **PA-4b:** The Drive courier is the end-state leg. The relay is a leg that exists only while a Claude session hosts it; no plan may depend on the relay after the desktop-app shutdown.
- **PA-5a:** Confirmed: yes, B2 covers the Part A proof. The acceptance wording is delivery → pickup on Duet's own check → ack through the bus, three timestamps, never a push or instant-wake claim.
- **PA-5b (owner):** Recommendation carried: (e) — the owner decides the shutdown trigger with the measured latency in front of him.
- **PA-6:** ACCEPT planning free text in the Drive file, bounded exactly as designed: senders `nox` and the owner only, 4 KB, secret-shape scan both ways, no order bodies, money, credentials or customer records.
- **PA-7:** (a) G40 runs FIRST, and its result is read before C5 is finalised. (b) The executing session runs it from a REAL LANE's own shell, Lane 6 observing; the output must show that lane's own cgroup (the executing session's own shell measures the wrong cgroup and is not evidence). (c) FIX THE PATH FIRST: the probe tests the live socket `/run/user/1000/fleet/control.sock`, not the retired unit's directory. This is the same answer as L4-4; one ruling closes both.
- **PA-8:** RULED (R2), stands. No new question.
- **PA-9:** APPROVED, all four parts: (a) build C1, C2, C3, C5; (b) C4 is INCLUDED (the Part A dependency list's omission is an error the design's own checklist contradicts); (c) the Lane 6 additions A2, A3, A4, A6 as specified (report-pickup binding to `duet`-addressed handoffs; server-side envelope check at send with ping-nonce idempotency defined; C5 extended to `comms-release` — see PA-14(b); the overdue notice's reader is the owner through `comms-inbox` per R2); (d) the base is `release/0.3.5.1`, the package goes through a fresh checker, and a must-keep hash gate refuses any release that would overwrite the 0.3.5.1 `validate.js` fix. Build in a worktree; a live install is a separate proposal needing my go under the 16:29 rule.
- **PA-10:** (a). Ruling 4 STANDS: `comms-register duet` and `comms-token-issue duet` wait on L4-A, proved on the real machine. Stated explicitly, per the 202249Z warning: registering `duet` widens the participant set, and it does so on THIS ruling, not on inference. This is the same answer as L4-6(a); one ruling closes both rows.
- **PA-11:** Team B's courier code is allowed offline PREP, not accepted as the courier. It is reconciled to the design at build step 5 (participant id `duet`, scope per P2, the closed envelope — not an `ext:` id, not a fixed scope, not a free-form ack). No live construction ahead of the build steps under this ruling.
- **PA-12:** Probes BEFORE the build. P0, P1, P2 and P6 move to the owner's next sitting, ahead of build step 1. P4 runs overnight after the standing instruction is pasted (PA-19). P3 stays late; it needs the courier.
- **PA-13:** (a). The proof text is the em-dash text as recorded, byte-identical, registered as the proof text (the C5 byte-equality route): "Duet — Nox. Connectivity proof ping via the fleet bridge relay. Please acknowledge with any short reply." The hyphen form reported in drafts is unsourced and void.
- **PA-14:** (a) ADOPTED: G1 on the built code before any external write, plus G2 (items 2, 22, 26) at the live proof with the owner present. A filed design review is not the gate's evidence. (b) ADOPTED as build gates: G40 first; the C0 sandbox as a hard stop (no hold = route `manual`); C5 extended to `comms-release` (I do not take the accept-the-residual alternative — extend it); C1 and C2 built and mutation-tested before any live `duet` token; the PA-9(c) additions; and the review's C4 (i)–(iii) and C7 as they bear on the `duet` grants and the courier. (c) CHANNEL NAMED: Team E (`upkeep-lead`) runs both G1 and G2; results filed as a report under the delegator board's `reports/` with mail to Nox; the runner is never the lane that built the code. Same answer as L4-15.
- **PA-15:** ACCEPT the relay session's exposure with the design's mitigations, as specified.
- **PA-16 (owner):** Recommendation carried: REQUIRE lowering `mcp.legacy-token` to read-only before the `duet` token goes live, after listing who currently holds the shared bearer. The "when" is the owner's.
- **PA-17 (owner):** Recommendation carried: relay if P0 passes, then Drive, then manual; email OFF for the finish.
- **PA-18:** CONFIRM both: courier health stays off the status pill; the Duet probes P0–P6 are owner-typed and the proof ping is the only bridge ping.
- **PA-19:** The standing-instruction text is APPROVED as drafted (design 6.4); Lane 6 reads it at G1. The owner pastes it at his sitting with the fleet account address filled in.
- **PA-20:** DO NOT AUTHORISE the model-backed red team (checklist item 18) now. It needs the owner's explicit spend + credential authorisation with a cost cap; ask him after G1 if it is still wanted.

## Leg 4 — Owner Elevation

- **L4-1:** (a). The reconciled leg-4 A+ governs the build. D1 — the owner-run root install of the entry program's static files — goes on the single privileged list for a home sitting, presented inside the DSH app, AFTER Lane 6 reviews the install kit. The finish-A-3 A+ is superseded as the build spec and kept as history.
- **L4-2 / L4-3:** RULED at 11:55 ET and confirmed again for the record: no `elevate.list` / `elevate.decide` facade tools (the facade stays closed at 15; remote decisions are the owner op `elevate-decide` only); ruling E1's reading R stands (approve = routing marker, deny voids, password only at the machine). If the question is ever reopened it follows the pack's stated conditions — list before decide, decide deny-only, only after N1 exists and G40 shows tokens confined, with a named gap and my ruling.
- **L4-4:** (a). G40 runs first, from a real lane's own shell, Lane 6 observing, with the socket path corrected to the live socket before it runs. Same ruling as PA-7.
- **L4-4b:** (a) conditional, with the rider: if G40 shows lane shells sitting in the fleet unit's cgroup, adding that unit to `denyCgroupUnits` and making the CLI `enter` the launch path (app card read-only) come BACK to me with the G40 output before anything is applied to the live host's config.
- **L4-5:** (b). Lane 6's condition is ADOPTED: before `enter`, a machine-side ticket display shows the ticket's real detail (kind, scope, expiry, requested action) from the daemon's ticket record plus the stored grant summary, and `enter` is refused until it has been shown. Standing bar: the owner is never shown LESS than the interim scripts show him today.
- **L4-6:** (a). Ruling 4 stands — the two `duet` tickets are confirmed through A2 in the sitting that proves L4-A. Same answer as PA-10(a). Interim rule (ii) CONFIRMED as standing until A2 is proved on the real machine: no agent-placed Tier B confirms; Tier A only under my per-ticket or bounded rulings; the owner's script is the only Tier B route, one ticket per call, recorded `confirm-file`, `authenticated 0`.
- **L4-8:** (a). The reconciled leg-4 design is the only build spec for the Claude-lane helper. B4's "the skeleton stands" is superseded by this ruling: the batch-2 skeleton stays on its branch as reference, nothing of it installed or running.
- **L4-9:** (a). Accept the audit-marker v1 for "take control briefly", and the owner is told plainly: v1 records when he took control; it is not a new way to reach the desktop.
- **L4-10:** (a), with the condition stated plainly: before his first run the owner is told the T3 residual in plain words — a lane can substitute a look-alike prompt; an unexpected prompt is treated as hostile. That telling is part of the sitting, and Lane 6's condition on it stands.
- **L4-11:** DEFERRED stands: phone-side password entry is NOT built in v1.
- **L4-12:** (a). Accept the control ops (`deny`, `lock`, `unlock`, clearing a suspension, `hand-back`) as advisory, recorded `authenticated 0`, residuals listed under T13 and T20. No polkit second factor in this leg.
- **L4-13:** (a), as designed: A2 covers Tier A AND Tier B confirms; owner-filed requests are exempt from the agent caps (the 15-minute expiry, the pending cap and the entry limits still apply); the CLI split and the `confirm-file` channel stand as stated. The consequence is accepted knowingly: after the A3 flip, every Tier A spend confirm needs the owner's password at the machine — that is the price of closing the forge path, and it is paid deliberately.
- **L4-14:** DEFERRED stands: the panel stays read-only in v1.
- **L4-15:** (a). Lane 6's intake channel is NAMED: delegator mail plus a filed report under the delegator board's `reports/`; Team E (`upkeep-lead`) runs both the gate on the built helper and the review of the install kit. Same answer as PA-14(c).
- **L4-16:** (a). CONFIRM both reconciled items: C6 (a separate accessor for unstaffed work) and `session-join` / `session-leave` as the one owner-present op pair.

## The unblocked sequence

1. G40 first (path fixed; real lane's shell; Lane 6 observing) — its output also settles L4-4b's return to me.
2. Comms amendments (PA-9) on the 0.3.5.1 base, worktree, fresh checker, must-keep hash gate.
3. Courier + fake-Duet harness build (offline); the leg-4 helper + L4-A build in a worktree on fake sudo, in parallel (builds proceed per the 11:55 rulings).
4. Lane 6 G1 on the built code, via the named channel.
5. The owner's sitting: probes P0/P1/P2/P6, the carried owner picks (PA-1/2/3/5b/16/17), the standing-instruction paste, the T3 telling, D1 install after the kit review, the L4-A proof, then the two `duet` confirms through A2.
6. G2 with the owner present, then the single proof ping, byte-identical, once.

Nothing live installs without a separate proposal and my go (the 16:29 rule; the leg-4 pack's done-proof step 3 says the same). Relocation (Part B) is untouched by this filing and still starts only on its fourth checker's PASS.

— Nox
