# R7 — state record (pass 2)

Generated: 2026-11-01T22:07:38Z

| | |
|---|---|
| Company | Lanternfield Moving & Storage (fixture company; fresh mktemp root this pass) |
| Fresh root | `/tmp/ergon-rehearsal-20261101T220736Z-ZUb78e` |
| Company host origin (fixture config) | `https://lf-host-01.example-tailnet.ts.net:8443` — made-up name, never resolved; listener 127.0.0.1 only |
| Install identity | release tarball `cayleb-dsh-fleet-0.2.2.tgz` sha256 ``37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177 |
| Seed state digest (R1) | `558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1` |
| Final state digest (before teardown) | `1572becb1b8baa5458c65611edb7e5aae5e0de3dd3941c8873beb809905931fa` |
| Final dump file | `R7-close/state-final-dump.json` (366131 bytes) |

## Cast and assistants

Six cast members: dana-whitcomb, priya-lindqvist, marcus-oyelaran, teo-brandvold, rosa-quintero-hale, sam-akintola.
Fixture profile + onboarding session per member: recorded at R2. Onboarding driven to `active` (product protocol, fixture answers): 6/6. Real fleet agent sessions on the installed plugin: 6.

## Tickets (scenario a and b)

| Ticket | Device | Final state | Receipt summary | Reply |
|---|---|---|---|---|
| `3094f609-8220-42ea-aa39-cc8a18d0c1e1` | teo-phone | resolved | Told the warehouse lead: line C. | Line C. Dispatch has updated the plan. |
| `7fb33cc5-91ca-45e2-8423-08dffd1fb44e` | marcus-desktop | resolved | Pointed the assistant at the shared rate card. | The rate card is in the shared estimates folder; I have pointed your assistant at it |

## Warehouse-board task as a real fleet order

Order `o3276`, task `t4142`, checker verdict post `7320`, evidence post `7318`; ops recorded: 12 (see `R5/fleet-order-ops.json`).

The order rolled up to `done` through its own done-when; its board task is done and banked (readback in `R5/fleet-order-readback.json`). The warehouse-board app itself is PACKAGE-PENDING (work-PC source); the board state is the fixture's scripted state.

## Close-out

Teardown ran after this record: root emptied, marker removed (see `R7-close/host-teardown.txt`).
Second pass: this IS the second from-scratch pass; the verdict-table diff is in `R7-close/pass-diff.md`.

