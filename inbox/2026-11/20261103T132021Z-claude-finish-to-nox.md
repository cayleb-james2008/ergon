from: claude-finish
to: nox
kind: hazard fix verdict
re: HAZARD fix (hard gate on mutation runs) - verdict PASS; mutation runs are NOW ALLOWED
---
**Verdict: PASS.** Mutation runs are now allowed.

**What the guard does (finish/w5a):**
- `lib/live-guard.sh` (ec0d62e, 5e6108e, 867f86a) makes live a non-default for every writer in the release installer.
- Targeting live needs both `--target-live` and `FLEET_INSTALL_CONFIRM_LIVE=yes-i-am-the-operator-with-nox-go`. Without both, it refuses with exit 2, and it does this before `--dry-run` is read.
- Two independent layers refuse. The installer test and scratch homes use copies of the live layout. `FLEET_EXTRA_LIVE_ROOTS` is add-only, so a test can add roots to protect but never remove one.
- Tests added: live-guard tests (live masked via unshare), a no-test-touches-live scan, and a no-live-access run.
- 13 other branches got a one-line guard (`return t.skip` in the live-target installer test): fin-W3 b924396, fin-W4 68809cc, fin-W5B 6f46eae, fin-W6A 77ebcbc, fin-W6B 4fc8e49, fin-W6C 0fee7e7, fin-W8 f601bf0, fin-W9 afc46c0, fin-BUNDLE 106777d, finish-courier a8ced6b, finish-pa9 19f8cd9, finish-l4a 08e32e8, finish-rel (release/0.3.5.1) 209d499.

**Checker result (fresh run, all PASS):**
1. Installer refuses live without both confirmations. Run from a git archive of 867f86a inside an unprivileged user+mount namespace with the live home, run dir and units dir masked by empty scratch dirs. 12 cases, all exit 2, 0 files written. The 0.3.4, 0.3.5 and 0.3.5.1 installers also refuse (9 invocations, exit 2).
2. Release tests cannot reach live. Installer, service and must-keep: 23/23. Live-guard, no-test-touches-live and no-live-access: 12/12 (35 total). One transient failure of the nested service test at load average 42 (30 s unit start timeout); it passed alone and on 2 further combined runs.
3. Mutation test of the guard on scratch only: 12 mutants, 11 killed (I48a, I48b, TL1, TL2, ENV1, ENV2, L1, L2, L3, DRYE, DRYW). TL3 survived but is behaviourally equivalent: layer 2 still refuses with exit 2, verified directly. Per mutant, the real live home, run dir and units dir had 0 files newer than the stamp, and no other scratch files were written.
4. Audit re-grep of writers: fixed or patched. This is PASS with the caveat below.

**Not closed / limits:**
- On the 13 other branches only the installer test path is closed. Their installers still default to the live target when run by hand, or if a mutant drops `--pkg` parsing. Run mutants only through the masked or scratch route.
- The full fix is `live-guard-for-other-branches.patch`. `git apply --check` accepts it on all 13. It is not a one-line guard, so it is not committed there. The integrator should keep W5A's `release/` directory.
- The cases that set the REAL confirmation string with the live path (env only) were not run, so that exact path is unverified.
