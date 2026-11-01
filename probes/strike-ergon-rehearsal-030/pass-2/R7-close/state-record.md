# R7 — state record (pass 2)

Generated: 2026-11-01T23:08:14Z

| | |
|---|---|
| Company | Lanternfield Moving & Storage (fixture company; fresh mktemp root this pass) |
| Fresh root | `/tmp/ergon-rehearsal-20261101T230811Z-IvO8Gi` |
| Company host origin (fixture config) | `https://lf-host-01.example-tailnet.ts.net:8443` — made-up name, never resolved; listener 127.0.0.1 only |
| Install identity | release `cayleb-dsh-fleet-0.3.0.tgz`, version `0.3.0`, sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070`; digest verified — source: journald install audit line (dsh-fleet-admin plugin-add) — what the install reports |
| Seed state digest (R1) | `558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1` |
| Final state digest (before teardown) | `ff769d1c6f11880ade39249d3b5c7a530542a64be120e3d966aa9e5538b64ce1` |
| Final dump file | `R7-close/state-final-dump.json` (366131 bytes) |

## Cast and assistants

Six cast members: dana-whitcomb, priya-lindqvist, marcus-oyelaran, teo-brandvold, rosa-quintero-hale, sam-akintola.
Fixture profile + onboarding session per member: recorded at R2. Onboarding driven to `active` (product protocol, fixture answers): 6/6. Real fleet agent sessions on the installed plugin: 6.

## Tickets (scenario a and b)

| Ticket | Device | Final state | Receipt summary | Reply |
|---|---|---|---|---|
| `ed8511de-1318-4dfd-897f-4a2db0022720` | teo-phone | resolved | Told the warehouse lead: line C. | Line C. Dispatch has updated the plan. |
| `e20845c4-9c68-4a15-94ab-c539022368d1` | marcus-desktop | resolved | Pointed the assistant at the shared rate card. | The rate card is in the shared estimates folder; I have pointed your assistant at it |

## Warehouse-board task as a real fleet order

Order `o3287`, task `t4164`, checker verdict post `7464`, evidence post `7462`; ops recorded: 12 (see `R5/fleet-order-ops.json`).

The order rolled up to `done` through its own done-when; its board task is done and banked (readback in `R5/fleet-order-readback.json`). The warehouse-board app itself is PACKAGE-PENDING (work-PC source); the board state is the fixture's scripted state.

## Close-out

Teardown ran after this record: root emptied, marker removed (see `R7-close/host-teardown.txt`).
Second pass: this IS the second from-scratch pass; the verdict-table diff is in `R7-close/pass-diff.md`.

