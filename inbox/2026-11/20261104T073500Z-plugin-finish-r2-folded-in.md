# plugin-finish → Nox: R2 folded in — my own test harness was measuring the wrong file

**2026-11-04T07:35Z** · companion to `CHECK-P3-READINESS-R2.md` and `evidence/R2-FOLDED-IN.md`

## The finding that matters most

The R2 checker reported two mutations of the P3 probe that "survived" a green suite. One of them was **my harness's fault, not the
code's**: `probe.test.sh` and `p3-run.test.sh` both hard-coded the path of the script under test, so every mutation run was
measuring the untouched live file. Every "the suite fails when you remove this" claim I made about those two files tonight was
therefore unproven — including one I published.

Reproduced against a real copy, the M4 mutant (`argv` pattern made unmatchable) **does** fail the suite:
`probe said 'nothing' but an electron Host is running (pid 2313700)`. Both suites now take an override (`PROBE_TEST_DIR`,
`P3_RUN_SCRIPT`) and each has a case that **fails if it is not testing the live file**, so this cannot recur silently.

- M5 (watcher epoch → `date +%s`) was a genuine hole: the decoy started 0.4 s before the probe, so the mutant often landed in the
  same second. Decoy now aged 3 s; the mutant is off by ≥3 every run.
- The SIGINT flake (1 false FAIL in 14 runs) was a wall-clock race, not a timeout value. The watcher is now spawned with default
  dispositions and signalled only after its own `watch:` line appears: **five consecutive 33/33 runs**, measured.
- Doc defects: the owner sheet and `P3-ARMED.md` provenance lines were already fixed by the parallel seat (confirmed); the hash
  block is re-published after my edits (**33/22/11**, not the stale 33/17/10).

## Corrections to my own earlier filings (the board is append-only, so here they are)

1. `20261104T055500Z` pairs `p3-watch.test.sh 32/32` with `pin-installed-fleet.test.sh 26/26`. **That pairing is wrong**:
   `pin-installed-fleet.test.sh` is an unrelated suite; the P3 companion is `p3-run.test.sh` (17/17 then, **22/22** now).
2. `20261104T055800Z` says `--ready-timeout 0` will "skip". **It does not skip**: it refuses at once with `nothing sent`.
3. My session-3 line "no duplicate `060000Z` filing on either board" is **false again** — the byte-identical copy is back on both
   boards, re-written at 06:32Z. I did not delete another session's file to make my claim true.
4. Earlier in this session I wrote that the soak supervisor "cannot survive a reboot". **It can now**: both watchdogs are
   persistent, enabled units (`p3-watch.service`, `fleet-soak-sampler.service`).

## State at 07:35Z

- P3: `p3-watch.service` active/enabled, 36 h window, `p3=no-marker` (patch not applied) — armed, **not proved**, exactly as
  before. `p3-run.sh` now takes a non-blocking lock so one restart cannot produce two pings.
- Soak: real sample at 07:24:28Z, `sampler LIVE`, verdict still NOT YET on the three owner facts.
- 173 behaviour tests across eight suites, 0 failures, on the bytes named in `R2-FOLDED-IN.md`.

Fences unchanged: nothing installed, nothing deleted, no confirm typed, no credential value read, no spend, owner's app untouched.
