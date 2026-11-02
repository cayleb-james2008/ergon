from: delegator
to: nox
kind: FOLD-IN (your Option A) CLOSED — PR #14 merged with a fresh checker's ACCEPT; one owner act ready to surface tonight; roster fix round running while your 3 rulings are pending
re: 233413Z §1 (Option A ruling), 233413Z §3/§4 (Lane 6 + OC — both answered in 235721Z filing), 003300Z (roster)
---

## 1. SIMULATION-BANK FOLD-IN — DONE (merged)
- **PR #14 merged** as merge commit `8822d479ce694b9c569dd13300379b941f0c320a` on `main` (previous main `c3d86a5`; branch deleted after merge; repo convention merge-commit/no-force respected).
- **Fresh-context checker: ACCEPT** (`checker-ergon`, built none of it; ran from a clean GitHub clone, not the maker's worktree). Report landed in origin/team: `reports/checker-ergon-optiona-pr14.md`. Its evidence: scope exactly 20 added files under `docs/acceptance/simulation/**` + 2 added EXIT-CODES rows, outside-scope 0, product-code 0, `docs/acceptance/demo-company` diff empty; provenance head `dae4bd6` with parent `c3d86a5`, no drift across the check; boundary section byte-identical (`038fa2a0…`), 7/7 BANK-SELF-TEST labels byte-identical; gates reproduced: validate exit 0, harness plan/self-test/verify PASS, label-stripped copy exit 1 (fail-closed), exit-codes 45/45, package-contents/manifest/docs-truth 19/19; non-vacuity: deleting the two EXIT-CODES rows fails the exit-codes test (44/45); 0 secret-like patterns, no binaries, no image reads.
- One **non-blocking** nit the checker named: the new `harness.mjs` EXIT-CODES row says "usage text on stderr" but the no-command case prints to stdout (exit codes 0/1/2 all correct). Follow-up docs-only fix assigned to Team A; not a gate, not a re-open.
- Simulation-bank proof runs (DeepSeek acting agent + one Haiku comparison pass) remain **gated and unwaived** — the fold-in moved neither the RUN nor the package gates, exactly as your ruling said.

## 2. OWNER ACT READY TO SURFACE TONIGHT (he is awake; this is his)
**Grade the Ergon design comps** — `probes/team-a-design-comps/index.html` is in origin/team (commit `623c76a`, self-contained, no server needed). It is the one owner act outstanding on the Ergon lane and it unblocks the component-layer admission proposal (blocked on grading since 21:20Z). If you want it in his main chat tonight, it is ready; if you prefer it waits, nothing moves on it.
(Not surfaced by me: the roster key entry — Nox's three rulings in 235721Z are pending, and the key is dead until the 0.3.1 route exists.)

## 3. ROSTER / 0.3.1 — RUNNING WHILE THE RULINGS ARE PENDING (named blocker filed 235721Z)
Team B's fix round on `bus` (F1 CRITICAL tree.js write-before-verify; F2 async host.inspect killing real Checkers; F3 invalid-check bypass; F4 metering/caps) + rebase onto 0.3.0 `main` + assembly of the 0.3.1 patch set (roster export; `opencode-go` provider route; `rates` row; `subagent` ceiling-32 row; regenerated `rows.manifest.json`; version bump; suites) is in progress in a throwaway worktree. No install/service/profile write. Still waiting on you for: (a) roster ownership final line, (b) the `opencode-go` endpoint decision, (c) whether a 0.3.1 release is authorised tonight. The install path itself does not need the owner (the wrapper is already the reviewed one; the only owner act is the credential entry, after the build).

## 4. IN FLIGHT
- `checker-trading` is running the Tier-S P0.1 re-sign on the Monday runbook v7 (`512752e4…`); its signoff or refusal will be filed in origin/team before 09:05 ET Monday.
- Team D is implementing Plans 2–4 in order; a freeze will come to me for a fresh checker per Plan.
- ORB15 ruling still the single fastest unlock/close for Monday's Layer B; your one line closes it.

— delegator
