---
from: claude-finish
to: nox
kind: W5B verdict
re: the 3 Nov order
---
# W5B (Option B: rubric final-task route, dispatch-a-banker) verdict: FAIL (liveness gate NOT MET)

Branch/head: finish/w5b, worktree /home/cayleb/workspace/harness-build/fin-W5B, from release/0.3.5.1. Head f7962b3, tree clean.

## Built
- d09263e: finisher hands a ready rubric final task to `built`; harness bank with an audited `how`; check start renews the order lease.
- 3b99a3d: fresh Checker for a rubric final task (one verified tree per done task); `bank-due` wake to a live idle maker; `roster/banker.js` harness bank only when the maker lane is gone (`maker-gone`) or still unbanked past the bound (`maker-idle-past-bound`); never over a running lane; `roster.harness-bank` kill switch; `orders.banked` records `by`, `how`, `maker`; `task-banked` post on the order board; lessons store unchanged (only the maker's own filing banks through it).
- Design note finish-docs/OPTION-B-DESIGN.md.

## Tests
Builder re-run: 499/499 (was 476/476; 23 new pass).
Fresh checker (finish-docs/CHECK-W5B-r3.md, on f7962b3), by name vs release/0.3.5.1: base 968 tests / 963 pass / 5 fail; w5b 1033 / 1028 / 5 fail. The 5 failing names are identical on both (first-run-status, client.test.mjs, conformance.test.mjs, in-host status payload shape, shipped-file manifest vs files[]). 0 names pass on base and fail on w5b; 65 new passing names.

## Mutation (finish-docs/MUTATION-W5B.md)
154 mutants: 100 killed, 54 survived. 29 survivors proven real gaps by probe; 13 equivalent/double-guarded; 12 real gaps unprobed. Fix commits since: 7d498b5, acd06b9, 526ef6e (also fixes N34: a final check over the brief size cap blocks with brief-too-large instead of throwing), 6a33d70 (composed-host tests for I01, I02), 31416d6 (load-flaky tests now poll), 0690b68 and f7962b3 (researcher-only rule, SA01, files[] coverage), 1a424ee (liveness drive tooling). 40 survivors closed at the 526ef6e/6a33d70 point; later commits close more. I have not re-run the full 154-mutant set against f7962b3, so the final survivor count is unmeasured. Checker spot mutants of banker.js (bank over running lane, no grace, fixture orders, final task banked): all 4 killed.

## Fresh checker verdict: FAIL
- Liveness A (rubric final on a booted host): NOT MET. Driver exited 3, NO MODEL ROUTE: scratch home has no OpenCode Go credential; no lane spawned, no model call.
- Liveness B (command order, done and unbanked, bank and close): NOT MET, not driven, same reason.
- Same drives failing on base: NOT DEMONSTRATED.
- Maker-only banking audit trail on a real host: NOT DEMONSTRATED (unit/composed tests and a code read of banker.js and markBanked only; no defect found).
- Suites by name, spot mutants: PASS, unit evidence only.
- Live fleet untouched: PASS (validate.js sha b68ce88c6de1, events max seq 2189, credential file stat only, no live socket, no push).

## Defect/blocker
BLOCKER (gate, not a code defect): staging the credential (release/liveness/stage-credential.mjs, kernel copy, mode 600, never a symlink) is owner-run or a session the owner has cleared to hold the key. Nothing in the relayed request clears the checker; round 1's copy was refused by the safety classifier as credential materialization. No code defect was found, and none was ruled out by a live drive.

## Not done / not proven
Both liveness drives; the base comparison; the real-host audit trail; a real Checker with git trees for a rubric final; the `bank-due` wake on real lanes. Open items: Lead's lease is not suspended during ordinary `checking` (needs a ruling); standalone ./store has no roster, so a rubric final waits there for a Checker (pinned by SA01); release lists and the installer's pinned file set do not yet list banker.js and the changed files (integrator's job); researcher-only tasks are left out of the final Checker's parts (pinned by test, intent unruled).

## Next step
Owner (or a session the owner clears) stages the credential per drive and runs: prep.mjs from a finish/w5b worktree, stage-credential.mjs S, drive.mjs S A; then a second staged home with drive.mjs S2 B --stop-maker; repeat on a base-prepped home. W5B stays unmerged until both drives pass.
