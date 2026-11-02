# Install record: release 0.3.5.1 (verdict criteria-as-text fix), 2 Nov 2026

**Who ran it.** The install was run by the owner in his own terminal at 16:04:45 ET (21:04:45Z) after the executing session's attempt was refused by its permission layer (classifier: production deploy). The only evidence of the run is machine state, listed below; the installer's own final line ("INSTALL PASS" or "INSTALL INCOMPLETE") went to his terminal and the executing session has NOT seen it.

**Authority.** Nox's Ruling 2 (15:48 ET): fresh checker first; install at a quiet moment with the same gates as 0.3.5; post-install live drive of a real verdict call. Fresh checker: PASS, 7 of 7 (CHECK-0.3.5.1.md). Quiet moment at 21:00-21:01Z: Lead idle, nothing claimed or in flight, no pending ticket.

## Machine evidence after the run (read at 21:05Z)
| what | evidence |
|---|---|
| file replaced | live `src/orders/validate.js` sha256 b68ce88c6de137834b6faa2a3ded82900f5acd9b267b261a67bbe543e5bedc22 = the release file; was bbeacee7... |
| backup | `fleet/backups/rel0351-20261102T210445Z`, MANIFEST.tsv: one row, pre bbeacee7..., post b68ce88c..., existed yes, mode 644 |
| unit | dsh-fleet-consolidated active, ActiveEnterTimestamp Mon 2026-11-02 16:04:45 EST; start log `consolidated-20261102T210445Z.out` one line, no error/degraded/EACCES/EADDRINUSE |
| integrity (control `status`) | integrity ok, rows ok, pinned 9, manifest 8b1f87d6aebf (unchanged from 0.3.5) |
| live verdict drive (the installer's verdict-selfcheck, fixture orders 6 and 7) | order 6 (shape A, JSON string) task 17 verdict pass (event 1769), task done (1771); order 7 (shape B, backslash-escaped) task 19 verdict pass (event 1797), task done (1799); 4 `verdict` calls reached the tool in total = 2 accepted string-criteria verdicts plus the 2 non-JSON controls (refused, no event) |
| before the fix (same self-check on the unfixed live host, 20:51Z) | 4 of 11 checks FAILED with `criteria-required` on both string shapes |

## Not verified
- The installer's own pass/fail line and its bus-selfcheck output (in the owner's terminal).
- Fixture orders 4, 5, 6, 7 remain on the live board (Nox: leave as is; the tasks are done and banked or done).

## Rollback
`bash release/install-0.3.5.1.sh --rollback` (restores validate.js from the backup above, byte for byte and mode); proven on a booted scratch host by the fresh checker (C4c).

## Side effect seen
At 21:04:5xZ, immediately after the restart, o1's order lease expired for the third time (`lease-lost-x3`) and the order went `blocked`. It was unblocked by the executing session (owner op `unblock`, Nox's standing word for this case) at about 21:07Z.
