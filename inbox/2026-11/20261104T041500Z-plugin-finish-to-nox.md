from: plugin-finish lead
to: nox
kind: BLOCKER + evidence — P2 wake proof FAILS on a state gate; both fleet lanes cannot execute a turn (post-switch)
re: o15 stream 1 (plugin finish); RD1 sheet steps 5/6
---

## The one thing that matters tonight
The fleet is **up** (status ok, integrity ok, 11 pinned, manifest `e9364e57…8843`, one writer = the app's Host,
control socket answering) but **both lanes are archived in the fleet overlay's workspace registry**, so every wake is
rejected as `blocked` before a model call. Nothing the fleet was told to run overnight can run until this is lifted.

- `$DH/fleet/state/storages/workspace.json`, written 22:51 ET (six minutes after the 22:45 switch), archives
  `fleet-delegator-b88fba3e…`, `fleet-lead-575c86c8…`, `fleet-checker-e9524977…`, `fleet-builder-dfe0f0d5…`.
- `packages/api/session-controller/src/archived-session-gate.ts`: an archived session's `agent/pre-step` is rejected,
  "which the loop ends as `blocked` without a request".
- Evidence, verbatim from the delegator session log: FLEET-WAKE (pending h81) → `turn/start turn:32` →
  `turn/end {reason:{kind:"blocked"}}` in 11 ms. Turns 30 and 31 (the two wakes before my ping) are identical.
  Turn 29, the last real work turn at 21:12 ET before the 21:59 crash, ended `completed`.

## P2 result: FAIL, fail mode `blocked`
P1 verdict re-derived and written. Baseline taken. Ping sent through the owner path (`op comms-send`, `cayleb-local`,
no bearer): `h81` seq 81, `route: followup`, `signalled: true`, one signal only, nonce `wpeeae52eb8de9`.
Delivery: `signalled` → never delivered, never acked, no `bus_pull`. Not a re-signal, not a recipient-parked, not a
bus failure — the lane woke and the gate refused the step. Per the sheet: no nudge, no `lane-start` called a pass.

## What closes it (code, then re-run)
1. Unarchive the four sessions. Either the owner's one click (sidebar → Restore), or the code fix: `fleet-roster`'s
   boot/resume unarchives its own lane session ids via the existing workspace-registry `unarchiveSession` before the
   first wake. Installing that needs a Host restart = an owner window, so tonight it is staged, not faked.
2. Then re-run P2 (new nonce) and P3 per RD1 5/6.

## Also recorded, not worked around
A1 post-switch gate is **9/10**: `D2 ext:delegator is retired` FAILS (`retired_at: null`). RD1 step 3 is owner-run by
construction (typed YES in `PACK/scripts/retire-ext-delegator.sh`); staged, listed, not bypassed.

Full evidence: `~/fleet-scratch/overnight/p2-blocked-finding.md`.

— plugin-finish lead
