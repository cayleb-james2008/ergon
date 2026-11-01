# LANE 3 — SECURITY-AUDIT CALIBRATION QUICK PROFILE (scoped, grant/guard surface first)

Filed 2026-11-01T18:05Z by `fleet-lead` (Team B, Lane 3) per Nox 182200Z item 5, pulled forward. **Calibration
profile + baseline counts only.** Findings handling fences: **never git-tracked**; only counts/states on the board;
no secrets, no credential values, no host/process detail.

## 1. Scope (tiered; grant/guard first)

| Tier | Surface | Why first |
|---|---|---|
| S1 | **Grant/guard surface** — the single-use grants and the guard path: ticket issue/bind/consume, args/agent/order/claim-window legality, 10-min expiry, Tier-B-vs-Tier-A labelling, hold refusal codes, guard ordering (no hold payload executed), no relay/mint path | Highest blast radius: this is the only path from a bot to a consequential act |
| S2 | **Action log + fail-closed** — hash-chained log integrity, append-failure denial, journald delivery | Tamper/fail-open risk |
| S3 | **Policy/authority matrix** — Tier A/B gating, owner-only ops, owner-cookie secret exposure, control-socket principal scoping | Authority leakage |
| S4 | **Containment/egress/fsguard** — bwrap, egress allowlist, masked roots, fsguard denials | Already covered by Stage 2/3 checks; calibration compares, does not re-open |
| Out | anything requiring the installed service's protected tree, credentials, network egress beyond loopback, or a live window; anything git-tracked as findings | fence |

## 2. Method (bounded quick pass)

1. **Evidence-first**: reuse the existing Stage 3 red-team evidence (ten attacks a–j, grant legs, guard-ordering
   3.3, action-log 3.4, ledger 3.5) — the calibration question is "does the existing suite actually cover the
   grant/guard surface, and where are the gaps?", not a re-run of the whole gate.
2. **Suite-as-evidence**: run the guard/grant-focused unit suites + the rehearsal-derived Stage 3 check in the
   scratch host (no real install); record pass/fail counts only.
3. **Static read**: walk the grant/guard code paths (policy ticket state machine, guard ordering, tool-policy
   matrix) against the documented contract; record **counts and states** (covered / gap / unverified).
4. **Calibration output**: a coverage ratio for S1 (tests+static evidence vs the documented grant/guard
   requirements), the list of surfaces with **no** evidence, and a proposed depth for the full audit.

## 3. Baseline counts/states (from existing records — no new findings claimed)

| Source | Findings | Severity counts | State |
|---|---|---|---|
| Contract review (`evidence/review-results/review_contract.txt`) | 16 | 2 high / 6 med / 8 low | fixed in R6 candidate except the named remainders; F11 closed by filed amendment |
| Liveness review (`review_liveness.txt`) | 8 | 3 high / 3 med / 2 low | fixes landed in candidate; Liveness F8 remainder recorded with future home |
| Security review (`review_security.txt`) | 9 | 1 high / 4 med / 4 low | fixes landed in candidate; Security F7/F9 remainders recorded with future homes |
| Stage 3 scratch rehearse | 73 PASS / 1 FAIL / 24 MANUAL | 3.4c = classified scratch limit; 24 MANUAL = owner tty legs | FAIL is environmental (journal-clean needed); not a defect |
| R6 dispositions (`task/mission1/R6-DISPOSITIONS.md`) | 8 carried remainders | — | each has a named reason/future home, none claimed fixed |

**Calibration statement:** the existing evidence is strong for S1's happy path and for the hold-refusal legs, but
the **adversarial depth is calibrated to a rehearsal, not to an external audit** — the quick profile's output is a
coverage map + gaps, not a verdict.

## 4. Fences (bind this profile)

- Findings are **never git-tracked**; board carries counts/states only, no exploitable detail, no paths, no
  credentials, no host/process imagery.
- No real-install run, no service action, no credential values, no new adversarial tooling, no spend.
- Maker never checker: any audit verdict goes through a fresh checker; this profile is maker-side calibration.
- Do not duplicate the Stage 3 gate or the wrapper/V3 critical path.

## 5. Exit criteria for this quick profile

1. S1 coverage ratio computed (documented grant/guard requirements vs evidence: tests + static read).
2. Uncovered surfaces listed with a proposed depth (time-boxed) for the full audit.
3. No findings posted beyond counts/states; any finding body stays local (not git, not board).
