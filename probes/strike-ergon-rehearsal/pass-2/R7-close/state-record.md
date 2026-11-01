# R7 — state record (pass 2)

Generated: 2026-11-01T22:31:51Z

| | |
|---|---|
| Company | Lanternfield Moving & Storage (fixture company; fresh mktemp root this pass) |
| Fresh root | `/tmp/ergon-rehearsal-20261101T223149Z-GWu5wW` |
| Company host origin (fixture config) | `https://lf-host-01.example-tailnet.ts.net:8443` — made-up name, never resolved; listener 127.0.0.1 only |
| Install identity | release tarball `cayleb-dsh-fleet-0.2.2.tgz` sha256 ``37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177 |
| Seed state digest (R1) | `558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1` |
| Final state digest (before teardown) | `6f35be79869ed76df062f68691b8508be47a7d5d0a87ba63bc820c56f71984c9` |
| Final dump file | `R7-close/state-final-dump.json` (366131 bytes) |

## Cast and assistants

Six cast members: dana-whitcomb, priya-lindqvist, marcus-oyelaran, teo-brandvold, rosa-quintero-hale, sam-akintola.
Fixture profile + onboarding session per member: recorded at R2. Onboarding driven to `active` (product protocol, fixture answers): 6/6. Real fleet agent sessions on the installed plugin: 6.

## Tickets (scenario a and b)

| Ticket | Device | Final state | Receipt summary | Reply |
|---|---|---|---|---|
| `c32bdd43-c3d4-4420-85a0-aeb16e637fc5` | teo-phone | resolved | Told the warehouse lead: line C. | Line C. Dispatch has updated the plan. |
| `8963c19e-2557-4d9a-a186-5d3f25a684a3` | marcus-desktop | resolved | Pointed the assistant at the shared rate card. | The rate card is in the shared estimates folder; I have pointed your assistant at it |

## Warehouse-board task as a real fleet order

Order `o3283`, task `t4156`, checker verdict post `7411`, evidence post `7409`; ops recorded: 12 (see `R5/fleet-order-ops.json`).

The order rolled up to `done` through its own done-when; its board task is done and banked (readback in `R5/fleet-order-readback.json`). The warehouse-board app itself is PACKAGE-PENDING (work-PC source); the board state is the fixture's scripted state.

## Close-out

Teardown ran after this record: root emptied, marker removed (see `R7-close/host-teardown.txt`).
Second pass: this IS the second from-scratch pass; the verdict-table diff is in `R7-close/pass-diff.md`.

