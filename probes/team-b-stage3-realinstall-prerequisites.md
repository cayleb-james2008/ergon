# STAGE 3 — real-install run prerequisites (item-7 package v1 is filed)

Filed 2026-11-01T17:12Z by `fleet-lead` (Lane 3). Package v1: `~/workspace/harness-build/evidence/stage-3/redteam-package/INDEX.md`.
Scratch evidence: `board/probes/team-b-final-tree/rehearsals/stage3/raw/` (73 PASS / 1 FAIL / 24 MANUAL).

## What the real-install run needs (in order; none started)
1. **A real install whose identity is accepted** — currently failed for this install (mode mismatch, V3 run
   record `board/probes/team-b-identity-v3-run-record-20261101T1555Z.md`); the run must wait for Nox's repair
   ruling + a fresh approval and a successful identity chain. No Stage 3 claim on the current install.
2. **Exclusive window** allocated by Lane 1 (same window discipline as the identity run; hold declaration +
   start/end receipts).
3. **Journal-clean host state for `3.4c`**: no scratch fleet-core hosts emitting `fleet-chain` heads; the real
   unit's journal otherwise quiet — otherwise `3.4c` fails as a scratch limit again (named, not smoothed).
4. **The unmodified packaged `checks/stage3-check.sh` against the installed output** with `FLEET_CLI` set; one
   run at a time; full private output preserved.
5. **Cayleb's confirmed tty run** (~10 min, `S3_CONFIRM=tty`, up to 9 polkit prompts) arranged through Nox —
   never simulated/substituted.
6. **Nox's 3.1 verdict** on the assembled package (request via Lead).
7. **Fresh checker** for the Stage 3 gate after the run.

## Pre-run preparation Team B can do without approval (scratch)
- Re-derive and hash-pin the execution script set from `cf93107`; verify the 3.4c journal predicate against a
  quiet host; enumerate the exact MANUAL step list for the owner's tty run (one mechanical sequence);
  package the reviewers' finding→fix mapping; record the known-limits appendix v2 (real-install wording).
