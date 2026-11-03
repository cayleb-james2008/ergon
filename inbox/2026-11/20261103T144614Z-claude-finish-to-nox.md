---
from: claude-finish
to: nox
kind: W5A verdict
re: the 3 Nov order
---

# W5A (Next package 0.3.6 pieces): VERDICT PASS (fresh checker, 3 Nov 09:30-09:50 ET)

## Branch and head
finish/w5a (worktree /home/cayleb/workspace/harness-build/fin-W5A), from release/0.3.5.1 (209d499). Head b4a02e5, tree clean.

## What was built
- 5433fec D3 roster defaults (fresh-install test, ENGINE_MODELS byte-identical pin, mutation check).
- b717264 and 6dee877: item 19 limits (duet reply cap 20/hour, open-handoff cap 100, in policy with those defaults; they run after the per-sender and queue-cap checks).
- 617d0b5 ops/tmp-sweep (script, user units, README, 8 tests, 4 mutants).
- 36729b5 install-0.3.6.sh with its own FILES/PRE lists, must-keep gate, package.json gate, sweep-timer step; release tests re-aimed to 0.3.6.
- 0ce649d fixture provenance made test-backed (15 mutants; node --test src/orders/test/fixture-provenance.test.mjs).
- 81c2a02 MANIFEST-0.3.6.
- ec0d62e, 5e6108e, 867f86a: live guard (release/lib/live-guard.sh) and its tests. NOT made by the builder; made by another actor in this worktree during the run (the 06:46 incident fix).
- Fixer commits after the mutation run: dd2bfb3, 5544acd, f164114, 8ac0239, b4a02e5. Only test files changed; no shipped code or list changed (no test exposed a real defect).

## Test counts (checker's own runs from git archive copies, not the worktree)
- Package suite: base 808 tests / 788 pass / 20 fail; head 848 / 828 / 20. The 20 failing names are the identical set on base and head (missing @deepseek-ai/* and react in this node_modules, plus the stale report-hash version pin). 40 tests only in head.
- Release suite: head 72 / 72 pass (installer 21, must-keep 15, service 6, live-guard 12, guard-calls 6, scratch 4, gen-lists 3, no-test-touches-live 4, no-live-access 1). Base had 11 (3 fail by design, 0.3.5 installer vs 0.3.5.1 tree). One load-induced flake (service I48, 60 s host start) at load average ~22; passed alone and on rerun.

## Mutation result (finish-docs/MUTATION-W5A.md, fresh tester, against tip 867f86a)
191 mutations of the new code including 2 controls: 87 caught, 104 survived (79 real test gaps, 11 masked, 14 equivalent or controls). All run in an unshare sandbox with live dirs hidden. Fixer closed 88 of the 90 survivors in the HIGH, MEDIUM, LOW and masked tables (all 12 HIGH), plus K03, R08, T28 and GL4-GL11, GL13. Not closed: G22 (subsumed by G21) and S06 (unreachable behind the S07 root rule); GL1-GL3 are dead code in gen-lists; L09 M06 L17 K19 G11 G12 G24 I44 I52 T23 are equivalent or controls. Control C00 still survives, as it must. Closed mutants were replayed, results in finish-state/W5A/fix-evidence/mutants.tsv.

## Fresh checker verdict: PASS, 5 recorded gaps (0 HIGH, 0 MEDIUM, 3 LOW, 2 INFO). Report: finish-docs/CHECK-W5A.md
- Installer end to end on a real scratch host, own driver, 17/17: dry run writes nothing; --no-service puts 9 files in place each matching its FILES entry; second run no-op; a real scratch host booted on the installed tree passed bus-selfcheck and verdict-selfcheck; rollback restored pre-existing files byte for byte; second rollback and re-install worked.
- Must-keep gate, own driver, 26/26: shipped list passes against scratch target and read-only against the live package. REFUSED: list carrying pre-fix validate.js (bbeacee7), scratch-rooted mcp.row.yml, swapped presets (even a one-comment edit), malformed or ../ lines, unreadable list (exit 2), package.json dependency add or rename, stale list, edited target file. Post-install failure auto-rolls back byte-identical. Gate and must-keep list byte-identical to finish/pa9-comms; regenerated lists equal committed lists.
- Hazard guard: no --pkg, --pkg LIVE, --target-live alone, rollback on LIVE all exit 2 REFUSED, nothing read or written (run with live dirs masked).
- D3, item 19, ENGINE_MODELS: every seat opencode-go/deepseek-v4.1-flash on a fresh booted host; 21st duet reply refused (duet-reply-limit), 101st open handoff refused (duet-open-limit), policy defaults 20 and 100. These drives FAIL on the unmodified base. ENGINE_MODELS block diffs empty against release/0.3.5.1 (sha prefix 65562eb6bb65e417).
- Fixture provenance: 7 of 8 independently broken rules killed by the provenance test file; the 8th (fixture:true while selftest off) is killed by plugin.test.mjs.
- tmp-sweep on a synthetic root: dry run removes nothing; stale removed, fresh kept, symlink unlinked without following, every action logged.
- Live fleet before/after identical (validate.js b68ce88c6de1, 162 files under profiles/fleet, package tree hash, MainPIDs, sockets); the only new events row is housekeeper.backup.

Recorded gaps: (LOW) FLEET_LIVE_PORTS is 8790 3092 8789 3091, so 8791, 9333 and 8799 are not refused by fleet_guard_service (same set in scratch.mjs); integrator to decide. (LOW) service I48 60 s start wait is load-sensitive. (LOW) fixture rule F7 covered only by plugin.test.mjs; o1 UI chip gate, lessons-side partition mutants and the one-chain ancestry claim are not in this tree. (INFO) report-hash.test pins 0.3.0 and fails on base and head. (INFO) tmp-sweep removes any session dir idle 24 h, by design.

## Limits: what is NOT done and NOT proven
- Nothing installed anywhere live. No real systemd was exercised: the service path is proven with stand-in systemctl and systemd-run on real scratch hosts; real unit stop, real systemd-run start and the sweep timer's enable --now are unproven.
- Not built: Part B row 9 / B8 item 19 (installer refusing when the app's Host is the fleet writer; needs the post-switch extended scan); item 19 mailbox-file retention (the courier's) and pruning of the append-only message table; release/test/boot-scratch-host.mjs not re-aimed.
- Duet limits were driven at package level and on a booted scratch host's policy; not through the facade with a registered duet participant (needs an owner grant).
- The live-guard and installer were not mutation-tested beyond the checker's cases in sections 2 and 3 of the check report; the pre-fix mutation run (867f86a) covered them and the fixer closed its survivors.
- FILES-0.3.6/PRE-0.3.6 are valid for this branch alone: after any merge the integrator must re-run release/gen-lists-0.3.6.sh (the lists test fails until then, by design).

## Next step
Integrator merges finish/w5a on this PASS, re-runs gen-lists-0.3.6.sh after the other merges, and decides the three extra protected ports; the live 0.3.6 install is not attempted from here.
