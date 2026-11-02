# DELEGATOR CONTINUITY NOTE — what must survive the relocation (from the delegator seat, 2 Nov 16:52 ET)
Written on my own initiative as **input for the handover pack** the leg-3 Part B design calls for (the design says the executing session
writes it from board files + rulings; this is the seat's own account of what is not on the board). Read with
`docs/harness-build/finish-2026-11-02/leg3-duet-bridge-design.md` Part B.

## 1. What this seat is right now
- DSH session in the **owner's personal `desktop` profile**; on the fleet bus as **`ext:delegator`** (kind external, pull-only, token file
  `/home/cayleb/.config/fleet-bus/delegator.token`, never read/printed; `bus.hello` is **not** in scope — `who/pull/ack/send` are).
- **Pull posture:** nothing wakes me; honest latency = my pull interval. I pull at turn start and ≥ every 15 min while work is open, and
  declare `poll_s: 30`. Handoffs h1/h2 pulled+acked; **h3 (to `nox`) is queued — `nox` has no pull route** (known gap).
- **Channels:** working board `/home/cayleb/Documents/deepseek-harness/default-workspace/board` (files, probes, reports, plans); shared board
  `~/Work/shared-workflows/lanes/board` (branch `team`, remote = ergon repo) is the **record**; bus is delivery only when the recipient has a
  pull route. The board is not delivery.

## 2. The team (this session's children — all durable teammates, cap 8)
`ergon-lead` (A: Ergon/design), `fleet-lead` (B: fleet plugin/face/Duet courier/elevation spec), `trading-lead` (C: desk), `roblox-lead`
(D: Roblox atlas/abort-contact), `upkeep-lead` (E: estate/Lane 6/N5), `checker-ergon`, `checker-trading`, `checker-roblox` (fresh-checker
seats, rotated). **Maker-never-checker** is absolute. Current states: A idle (Lumen closed, reel intake filed), B active (face rebased onto
0.3.5.1, port-safety encoded, no install), C idle (desk closed), D idle (Play 21 stopped, fallback filed, Studio idle 1/3), E active (Lane 6
review of the leg-3 bridge + leg-4 elevation in progress), checkers idle. **A relocated delegator lane will not have these teammates** unless
the new home provisions equivalents (the design states the capability reduction explicitly — Q1).

## 3. Things only this seat carries (not on the board) — must be handed over or lost
1. **The schedule bookings**: the Monday bookings (07:45 sweep, 09:00 desk day, 16:15 outcome+N5) live in **this app's schedule store**
   (`~/.desktop-build/development/home/storages/schedule.json`), restored by me after the earlier store loss. **Relocation deletes/replaces
   that home** → the bookings will not survive unless the new home re-creates them or the robot moves to cron/systemd timers. This is a real
   continuity item for Part B (the design's deletion plan and archive should include or explicitly drop them; I recommend re-creating them in
   the new home or as systemd timers with the same times).
2. **The browser sidecar / presentation staging** (right-column browser, Lumen presentation path) is profile-local state; the owner's grade
   was presented from it. The relocated seat will need a presentation route in the fleet profile (the face itself renders there per G-D1).
3. **The wake-up report and morning runbook** (`reports/WAKE-UP-REPORT-20261102.md`, `plans/MORNING-20261102.md`) exist on the board, so
   those survive; but the **owner-facing presentation habits** (present in-app, rendered, no static stills; key acts in-app; no deadline
   talk) are standing rules to carry in the handover brief.
4. **The session's own context** is ~17 MB — the design is right that it is distilled, not copied; the handover pack must cover: the mission
   plan + round-4 state table, pending rulings, the incident list (dashboard kill; brief.js/calls.js hotfixes now superseded by 0.3.4+),
   the bus posture, the team roster and states, and the owner's open acts (Lumen grade, privileged sitting items, Tier-B confirms).
5. **My two hotfixes** (`brief.js` commit rule, `calls.js` decompose coercion) are **superseded** by 0.3.4's proper fixes — the new home must
   not re-apply them; they exist only in the running install today (`0.3.5-consolidated` profile) and are under the executing session's
   review. Recorded here so nobody carries them forward by mistake.

## 4. Pending rulings at this filing (the handover pack should carry their state)
1. **Abort-contact fallback**: option 1 (fixture leg-start/target-side geometry so the chase target is ahead in the probe's travel direction —
   recommended) vs option 2 (accept the honest physics/direction finding as the cell's named limit). No code change made; 2 Plays reserved.
2. **Leg-3 Duet bridge + relocation (Part A/B)**: with Nox; Lane 6 review in progress (Team E). Duet's connection is the owner's end-state
   trigger (once proven, he shuts the Claude desktop app and DSH is the only harness).
3. **Face live install**: artifact `0.3.5.1-face-prep` ready and scratch-proven; needs the installer self-check + must-keep hash gate + Nox's
   go (the executing session owns that gate).
4. **Owner acts open**: Lumen final grade; privileged sitting items (Tailscale SSH/OpenSSH verified-ready; sudoers + stale V3 drop-ins HELD
   for Lane 6); Tier-B confirms for the delegator token were done (15:18/15:20) — the relocation's new-home proof (Part B) is separate.
5. **o1**: administratively closed (Option C); stored `failed` = closure; the Option B route fix is the next package.

## 5. Honest risks of the relocation as designed
- The seat in the fleet profile loses shell/network/browser/teammates/scheduler (design Q1) — every capability this note used (board edits,
   bus client, systemctl checks, browser presentation) must be re-provided or explicitly dropped. The handover pack should map each to its new
   owner.
- The app switch (symlink `profiles/desktop` → `fleet`) changes where the owner's app runs; **the schedule store, sessions, storages and the
   desktop-side browser state are profile-local** and the deletion plan must state which of those the owner keeps. I recommend: keep and
   migrate the **schedule/automation**, drop the rest per his order, archive first, delete last (as the design orders).
- Nothing in this note is a build or a move; it is the seat's continuity account for whoever writes the handover pack.

— delegator (this session), 2 Nov 2026 16:52 ET
