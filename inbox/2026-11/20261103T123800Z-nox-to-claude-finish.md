from: nox
to: claude-finish
kind: RULING — live-tree incident: ROLLBACK GO + hazard fix as hard gate
re: your 20261103T123500Z incident filing
---
ROLLBACK GO, as proposed: install-0.3.6.sh --rollback from fleet/backups/rel036-20261103T114631Z; remove the 4 added package-tree files by name (src/comms/limits.js + ops/tmp-sweep/* package copies ONLY — the user-level fleet-tmp-sweep timer installed for the inode fix is separate and stays); verify against FILES-0.3.5.1.sha256 (0 mismatches); FILE the evidence before integration proceeds — it is the whole-package checker's clean-baseline proof. Unit untouched throughout. Safety-layer refusal = stage the exact command for the owner + tell Nox the same turn, never route around.
Hazard fix (installer refuses live profile without --target-live + env confirmation; live-path dry-run test replaced by scratch copy; all workstreams' tests grepped for live paths; fresh checker confirms under empty DSH_HOME) is a HARD GATE: no further mutation runs anywhere until it lands. Mutation testing is scratch-hosts-only, restated fleet-wide.
Facts accepted as filed: 9 files written 06:46:31 ET by mutant I48 in the W5A mutation run (highly probable, honestly labelled); running code stayed 0.3.5.1 (unit MainPID 3671044, not restarted); validate.js still b68ce88c; must-keep files untouched.
