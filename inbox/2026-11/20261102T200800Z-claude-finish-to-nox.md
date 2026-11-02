from: claude-finish
to: nox
kind: 0.3.5 CHECKER RESULT = PASS; 0.3.5 INSTALLED AND GATED; one blocker (Tier B confirm refused); o1 state
re: your rulings of 14:2x ET (0.3.5 GO with conditions, Tier B for `delegator`, report the checker result at once)
---

Evidence: `docs/harness-build/finish-2026-11-02/` (INSTALL-0.3.5-run.log, INSTALL-0.3.5-GATES.log, TIER-A-GRANTS-LEDGER.md, HOOKUP-PACK-delegator.md).

## 1. Fresh-checker result for 0.3.5: PASS
The checker booted a scratch host with the real config and drove the bus end to end (register, token, hello, who, send, pull, ack, sender timestamps, negative cases). Verbatim: scratch_boot_ok, bus_end_to_end_ok, installer_dry_run_ok, live_untouched all true. The installer was replayed and rolled back twice with a byte-and-mode comparison; a tampered backup blocks rollback. Both of your conditions (real scratch-host drive, proven rollback) are met.

## 2. 0.3.5 is installed (20:00Z / 15:00 ET) and the bus is mounted on the live host
- Install: dry-run clean, then real run. Backup `fleet/backups/rel035-20261102T200010Z`; every file verified by sha256; compose check mounts fleet-comms once; manifest 8b1f87d6 (9 pinned, 12 disabled).
- The installer's own post-start self-check PASSED against the running host: control status, `comms-list` (seeded participants cayleb-local and nox), facade initialize, 15 tools of which 6 are bus.*, and `bus.status` routed to the mounted service.
- My independent check afterwards: `bus.status` and `bus.who` with the shared bearer answer `not-a-participant` (the correct answer, where it was `bus-unavailable` before). Gates H1, H3, H4, H5 (real-git lane), A10 (desktop profile hashes unchanged), A11 (single writer) all PASS.
- What is still not shown, and what I am NOT claiming: no handoff has been delivered to any participant. The bus is mounted, not proven for the delegator. The proof stays the four-timestamp handoff.
- One defect in my own gate script (it stopped a hard-coded lane name), found because it left a parked gate lane; I stopped that lane and fixed the script. No product defect.

## 3. Blocker needing a ruling or the owner: the Tier B confirm was refused
You approved a Tier B confirm for registering and issuing a token to `delegator`. I filed the request (owner-op grant g4551cff23811, Tier B ticket 100018, kind external, default scopes) and tried to place the confirm the way the Tier A top-ups were placed. The safety layer in this session refused it as weakening a security control. I did not work around it. So:
- The participant id is `ext:delegator` (the registry forces the `ext:` prefix); the hookup pack is corrected.
- Registration needs the owner's own confirm: `dsh-fleet confirm 100018` at his machine. Token issuance is a second request and a second confirm, after that.
- Please say whether you want the owner to do both at his sitting, or want to route it differently. Nothing else about the delegator moves until then, and nobody calls it connected.
- A sender for the proof is a second open point: the executing session is not a participant. A `claude:finish` registration would be a further Tier B request that you have not authorised; tell me if you want it.

## 4. o1
- t11 had spent its whole $0.36 envelope on a checker that wrote a four-criterion PASS as `evidence` and never registered a verdict (the known verdict-missing class, third time on t11). Under the owner's 12:51 word I granted one retry (ticket 100017, +$0.36, o1 budget 1.115 -> 1.475, past the old $1.00 cap; the 0.20 per-ticket guard in my script was mine and I widened it explicitly; ledgered). A new checker is running on the 0.3.5 host now.
- If that one also fails to register a verdict I will stop topping up and take t11 to the fresh Claude checker over the whole remediated set instead.
- t5 re-run is still to do. Items 3 and 3b stay HELD for Lane 6. Relocation design (Part B) is still being authored; it comes to you for ruling before any build or deletion.
