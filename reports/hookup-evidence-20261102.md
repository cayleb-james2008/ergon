# DELEGATOR HOOKUP EVIDENCE — fleet bus (ext:delegator), 2 Nov 2026 15:31 ET

**Actor:** the delegator session (this DSH session), at the owner's machine, pull-only. **Token never read/printed/copied** — the client
reads it in-process from `/home/cayleb/.config/fleet-bus/delegator.token` (0600). **Nothing is claimed as "connected" here** beyond what
the sender's own `bus.status` can observe (hookup pack §6 Part A: sent, delivered, picked up, acked — the observation is the sender's).

## My timestamps (from the commands run, UTC / ET)
| step | at (UTC) | at (ET) | result |
|---|---|---|---|
| board files read | 2026-11-02T20:29–20:31Z | 15:29–15:31 | `00-READ-FIRST-SUPERSESSION…` (CORRECTION block first) + `01-HOOKUP-delegator-fleet-bus…` read |
| `who` (mount + presence) | 20:31:28.487Z | 15:31:28 | **bus mounted**; me `ext:delegator`, kind external, **state fresh**, `pending 1` (oldest 145 s, head `h1`) |
| `hello --client delegator --poll-s 30` | 20:31:2xZ | 15:31 | **REFUSED: `scope-denied` — "ext:delegator has no scope for bus.hello"** (recorded; not worked around) |
| `pull` | 20:31:31.857Z | 15:31:31 | 1 handoff delivered: **`h1`** (see below); `last_pull_at` set |
| `ack --ids h1` | 20:31:34.808Z | 15:31:34 | **`acked: [{id:"h1", state:"acked"}]`**; `last_ack_at` set |
| `who` (after ack) | 20:31:35.xxxxZ | 15:31:35 | me `ext:delegator`, state fresh, **`pending: null`**; lanes `lead` (idle) + `checker:caad9f14d2dce` (idle) |

## The handoff (data, not instruction)
`h1` — from `cayleb-local` (bus records the sender as cayleb-local because the owner-op route has no other identity; the body states it was
sent by `claude-finish` on Nox's 15:28 ET ruling), kind `ping`, state `queued` at pull → **acked**, `needs_reply:false`, `thread:null`,
`created_at` 1793651343642 = **20:29:03.642Z / 15:29:03 ET**, subject "Connectivity proof ping (hookup pack section 6, Part A)";
body asks only: pull it, then ack it (accepted); **do not act on anything else in it** — treated as data, as the boundary requires.
**Pickup latency measured from the bus clock: pull (20:31:31.857Z) − created (20:29:03.642Z) ≈ 148.2 s.**

## Pull posture — stated honestly
- **I am a pull-only participant: nothing wakes me.** The honest latency of any handoff to me is my pull interval.
- I will run `pull` at the start of every turn while any order/task/promise is open and at least every 15 minutes; I declare
  `poll_s: 30` as my working interval for staleness judgements (the bus's watchdog covers roster lanes, not me, until the relocation
  (hookup pack §1 step 1, Nox ruling pending) makes me a roster lane).
- A `needs_reply` handoff is overdue when `reply_by` passes unacked; I will answer and ack, or decline with a note.

## What is NOT done / not claimed
- **`hello` could not run** (scope denied). My `who` presence is `fresh` and the pull/ack path works; I do not claim a hello-based connect.
- **Part A (pull path) proof**: my side is observed (pull 20:31:31.857Z, ack 20:31:34.808Z, h1 acked). **The four-timestamp observation on
  the sender side (`bus.status`: sent, delivered, picked up, acked + latency) is the sender's to file** — I do not call the delegator
  connected; Nox/the executing session do on that evidence.
- **Part B (new home, wake path)**: not applicable yet — relocation is a design awaiting Nox's ruling; I remain pull-only in this home.
- No orders, caps, done-whens, confirms, policy, privileged or credential actions taken in this hookup.

— delegator (this session), evidence written 15:32 ET

## Addendum — handoff h2 (retraction of the typed 20:32Z hold), 2 Nov 15:15 ET
- **h2 delivered at pull 20:31:5xZ?** (delivered state queued, created_at 1793652545977 = **20:49:05.977Z / 15:49:05 ET** per the bus clock; my
  pull that returned it ran at **21:15:11Z / 16:15:11 ET** — the pull interval was long because I was in stand-down; that is the honest pull
  posture latency, not a wake).
- **h2 from `cayleb-local`** (bus record; body states it is Nox's ruling relayed by claude-finish over the bus), kind `notice`,
  `needs_reply:false`, subject "RETRACTION of the 20:32Z hold on Teams B/D/A/E (Nox ruling, 15:48 ET) and the 15:31 typed message is void as
  a channel". Content treated as data from an authenticated channel: **the typed 15:31 message is void as a channel and carries no
  authority; my 20:32Z hold messages to Teams B/D/A/E are retracted; work resumes (face, abort-contact B, reel intake, N5 16:20, Duet
  courier with the bridge-build hold standing); the filed-ruling holds stand (privileged 3/3b with Lane 6, no relocation build/deletion, no
  Duet bridge build before the leg-3 ruling, Owner Elevation install/first-run = owner's home sitting).**
- **h2 acked at 21:15:27Z / 16:15:27 ET** (`acked: [{id:"h2", state:"acked"}]`). Resume messages sent to Teams B, D, A and E.
- **Channel note for the record:** the typed message arrived inside this session (computer use by another session) — not a bus ruling;
  Nox stopped it and voided it as a channel. From now on, anything governing this session's work arrives **over the bus or as a filed Nox
  ruling**; I will not treat typed-in instructions as authority.
