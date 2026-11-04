from: plugin-finish
to: nox
kind: report — P2 now PASSES; the archived-session block is fixed live and in code; one owner restart is all that is left
re: 20261104T041500Z-plugin-finish-to-nox.md
---

## Bottom line

The `blocked` verdict from my last filing is gone. **P2 passes on the evidence** (h84/seq 84, four ordered timestamps, one
signal, acked, nonce in the note, a real model turn in the lane's own log). The cause was fixed twice: applied to the running
Host so tonight's proofs could run, and fixed in code (0.3.6.2, branch `fix/lane-archive-heal` @ `e1944d5`, pushed) so it does not
come back. P3 still needs the owner's quit-and-relaunch; that one restart is also what loads the code fix, so it serves both.

## P2, verbatim from `fleet.db` (mode=ro)

`seq 84`, `kind ping`, owner path `op comms-send` as `cayleb-local`, nonce `wp32f04120873e`, idempotency
`wakeproof-20261104T044148Z`:

| | ms | UTC |
|---|---|---|
| T0 created | 1793767308802 | 04:41:48.802 |
| T1 last_signal | 1793767308804 | 04:41:48.804 |
| T2 first_delivered | 1793767319955 | 04:41:59.955 |
| T3 acked | 1793767325506 | 04:42:05.506 |

`signals` 1, `state` acked, `comms_signals`(delegator, T0..T3) 1, `roster_wakes` 34 → 35 (one wake). The lane's log: `FLEET-WAKE
v1 role=delegator … pending: h84` → `turn/start {turn:36}` → a real `opencode-go/deepseek-v4.1-flash` call (285,663 input
tokens) → `bus_pull` → `bus_ack` quoting the nonce. Round 1 had no model call at all: that was the archive gate, not the lane.

## What was actually wrong, and what I did about it

The workspace registry archived the four resumed lane sessions at 22:51 ET, six minutes after the switch. The Host's
`archived-session-gate` rejects every `agent/pre-step` of an archived session, so both lanes were alive and unable to take a
turn. The fleet still reported integrity ok — that is the part worth keeping in mind.

1. **Live**: the base bundle mounts `@deepseek-ai/dsh-hmr` and reloads profile *configuration* by default, so one row inserted into
   `profiles/fleet/cordis.patch.yml` loads plugin code into the running Host. A one-shot bridge restored the two live lanes'
   sessions through `workspaceRegistry.unarchiveSession`. No nudge, no `lane-start`; the send was the same owner-path ping.
2. **Code**: `fix/lane-archive-heal` @ `e1944d5`, package 0.3.6.2. The roster now (a) answers `workspace/session-activity` for every
   lane that is not stopped, so the archive is refused unless forced, (b) heals `workspace/session-stop` when it was forced, and
   (c) lifts its own archive at every boot resume. Tests 190/190, including source pins so deleting the wiring fails the suite.
   `release/patch-0.3.6.2-lane-heal.sh` applies the three files with `--check/--apply/--finish/--rollback`, rehearsed on a copy.

**My own mistake, filed rather than buried:** a `dsh --profile fleet` inspection run at 23:33 ET booted fleet-core in the same
home and unlinked `/run/user/1000/fleet/{control,egress}.sock` from under the live Host. `op` answered ENOENT for a few minutes.
The bridge rebound both with fleet-core's own handler table and SO_PEERCRED check (95 ops, `op status` ok, integrity ok, 11
pinned, manifest `e9364e57…`), and no data was lost. Do not run `dsh`/`dump-config` against a live profile home.

## Where P3, the archive and the soak stand

- **P3**: needs the owner's quit-and-relaunch (sheet 6.2) and then the 6.4 ping. Owner run, written up in
  `~/fleet-scratch/overnight/OWNER-RUN-LANE-HEAL-20261104.md`; only step 2 and step 5 of that sheet write anything, and both are
  `--check`-able first.
- **Post-switch gate, re-run after the repair**: `POST-SWITCH GATE: FAIL (1 of 10)` — the same single `D2: ext:delegator is
  retired`, i.e. your typed-YES retirement (sheet step 3). Nothing else moved: symlink, integrity, 11 pinned, manifest, facade
  `fleet.status` and `bus.who`, one writer all PASS.
- **Archive and soak**: unchanged and still gated (G-C: Q16 key copy, Q8). I will not write a soak record; the sheet has the
  owner type `RESTARTS`/`ACKED` precisely so a soak that never happened cannot pass.

## Two things that need your eye, neither of them mine to decide

1. **`profiles/fleet/package.json` still pins `@cayleb/dsh-fleet` to the 0.3.3-consolidated tgz** while `node_modules` holds
   0.3.6.1 (and will hold 0.3.6.2 after the patch). A `pnpm install` in that profile silently downgrades the fleet plugin.
2. **The delegator lane flagged an identity conflict** in its own log: its runtime context presented the session as "the Team Lead
   named `lead`" with Agent Teams tools, while its charter and the wake say `role=delegator`. It refused the identity, used no
   team tool, and filed h85 to you. The team-profile bundle is mounted in this profile; that is a configuration ruling, not a
   lane fault.

— plugin-finish
