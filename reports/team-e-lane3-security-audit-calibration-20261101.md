# TEAM E / LANE 6 — LANE 3 SECURITY-AUDIT CALIBRATION: QUICK PROFILE, GRANT/GUARD SURFACE FIRST
**Run:** 2026-11-01 13:20–13:50 ET by `upkeep-lead` (Lane 6) · **Pull:** Nox 182200Z item 5 (cross-lane, authorised) · **Profile source:** `team-b-lane3-security-audit-calibration-profile.md` (fleet-lead, 18:05Z — scope/method/exit criteria reused; this run does not duplicate it).
**Scope:** **S1 only — the grant/guard surface** (the only path from a bot to a consequential act). S2 (action log) and S3 (authority matrix) touched only where they are the same code path; S4 containment out of scope.
**Method:** source-first static read of the grant/guard code + reuse of the existing Stage 3 evidence record (ten attacks a–j, grant legs 3.2a–f, guard ordering 3.3, action log 3.4, ledger 3.5, reviewer findings). **No target execution, no install, no live window** — this seat has no OS-enforced sandbox, so every execution-dependent leg is reported `needs_validation`, per the skill's design. **0 new candidate findings claimed; 2 evidence legs unverifiable from this seat; 2 documented requirements unimplemented by decision; 6 carried remainders mapped.** All numbers below are counts/states only — no finding bodies, no exploitable detail, no paths, no host/process detail (fence honoured).

## 1. Coverage result (S1)
**Blocked-by-design matrix:** 16 requirement groups; **11 covered by direct evidence** (tests/e2e/check legs), **2 covered by static-only** (storage invariants + control-op gating breadth), **1 covered but one branch needs_validation**, **2 unimplemented-by-decision** (owner ruling), **2 legs needs_validation** (environmental). **Coverage ratio by group: 14/16 = 88% have some evidence; 11/16 = 69% have executed evidence; 0/16 unexamined.**

| # | Requirement group (documented) | State | Evidence (counts/states only) |
|---|---|---|---|
| R1 | Hold-tool classification; non-hold tools allowed; a bot's `tier` argument is ignored | **covered** | 15 holds unit tests + e2e red-team a–j + check 3.1/3.2 classes |
| R2 | Held call → exactly one new pending ticket; intent row precedes the ticket | **covered** | unit (intent-before-ticket) + e2e + check 3.3b + static |
| R3 | Ticket binding immutable; state forward-only; no delete (storage invariants) | **static-only** | DB triggers read directly; no unit/e2e/mutant asserts them (see G1) |
| R4 | Trade/wallet refused outright — never a ticket, no executor, no relay/mint path | **covered** | unit (trade/wallet, labelled/replayed) + e2e + check 3.2a + static |
| R5 | Lane/role gating: non-lane → `not-a-lane`; role-invisible → pipeline unknown-tool | **covered** | unit + e2e + static |
| R6 | Malformed hold args → body refuses with precise code, opens no ticket | **covered** | unit (malformed→bad-args) + static |
| R7 | Claim binding: no live claim → deny; spend requires the order claim; Lead path uses `args.order` | **covered** | unit + e2e 3.2c/d + check 3.2 |
| R8 | Grant match binds tool+args-hash+agent+session+order; single use; 10-min expiry; mismatch = new ticket, old grant unconsumed | **covered** | 8 grant unit tests + e2e + check 3.2a–f + 5 stage-3 mutants + static `match()` |
| R9 | Tier only from the table; spend tier recomputed at grant and execution against the ceiling (R6-3) | **covered** | unit + e2e R6-3 regressions (17 settle tests) + 4 hold-table mutants |
| R10 | Guard ordering: deny-only guard after pre-execute/approval; prepended allow cannot skip; failed intent append denies and opens no ticket | **covered** | unit + e2e (prepended allow) + check 3.3a–c + 2 guard mutants + static |
| R11 | Approvals: confirm-file validation (regular file, no-follow, owner-uid, ≤4 KiB, names ticket, pkexec uid), windows A 10 min / B 24 h, second approval `not-pending`, expired computed at read | **covered, one branch needs_validation** | 5 unit tests (forged/relayed confirm set incl. wrong uid/symlink/oversize/FIFO) + check 3.2b/3.2e-confirm/3.2-tb-expiry + static poll read; **owner-uid branch cannot be exercised without a second uid** (G4) |
| R12 | Executors run once: deploy/post → prepared package consumed in one txn; credential → no executor; policy → consume+apply one txn; spend raise idempotent on ticket id | **covered** | unit + e2e + check 3.2c–g + static `present()` |
| R13 | Action log: genesis/chain formula; torn tail dropped; broken chain refuses every append (all calls denied); intent+outcome rows every path; independent verify | **covered except one leg** | 6 log unit tests (torn/tampered/partial-write) + independent recompute in check 3.4a (395 rows) + 3.4b append-failure deny + 2 log mutants; **3.4c journald-attribution leg FAIL in scratch = needs_validation** (G3) |
| R14 | Owner ops gated by peer identity (repeated per op); hooks selftest+fixture-only; confirm poll removes every file it sees | **static + e2e** | control-op wrapper read + e2e control ops (C1/C2) + check block I; no dedicated owner-refusal unit (G2) |
| R15 | Control-socket principal scoping: a bot cannot act as the owner on it | **covered** | e2e (j) + check 3.1j (socket unreachable/refused; forged confirm files did not land) |
| R16 | No execution path for trade/wallet for any role (no tool registered) | **covered** | e2e + check 3.2a + static tool table |

**Aggregate evidence reused (not re-run):** policy suite **60 tests** (holds 15, log 6, composition 7, e2e 15, settle 17); Stage 3 rehearsal **73 PASS / 1 FAIL / 24 MANUAL** (FAIL = classified scratch limit 3.4c; MANUAL = the owner's tty/polkit legs); mutation set **68 entries in the suite, 43 touching policy/core/ledger paths** (12-mutant Stage-3 class: 12 killed; one untestable-without-root branch); reviewer findings **16 contract (2H/6M/8L) + 8 liveness (3H/3M/2L) + 9 security (1H/4M/4L)**, five verified highs fixed in R6, **6 carried remainders** with named future homes.

## 2. Gaps and calibration calls (no finding bodies)
| id | state | what it is | proposed handling |
|---|---|---|---|
| G1 | **static-only** | DB storage invariants (immutable bindings, forward-only state, no-delete) have no unit/e2e/mutant assertion; they are enforced by triggers read in source | full audit: add a direct negative test (mutant or SQL attempt) — cheap, 30 min |
| G2 | **static + integration-level only** | owner-op refusal semantics (non-owner peer) are exercised at the integration/control level, not as a unit matrix | full audit: one table-driven negative set per owner op |
| G3 | **needs_validation** | 3.4c journald chain-head attribution FAILed on the shared scratch journald (foreign writers); classified scratch limit, not a defect | real-install / journal-isolated host run (the package's own prerequisite) |
| G4 | **needs_validation** | confirm-file owner-uid branch cannot be exercised without a second uid / root | real-install run or a sandboxed second-uid harness |
| G5 | **unimplemented by decision** | hold-ticket **rate limit** — no contract yet; `security F7` carried, owner to rule a policy row | owner/Nox ruling; then a row + test + mutant |
| G6 | **unimplemented by decision** | **free-text hold patterns** ("best effort" deny of trade-ish shell text) not built; only the argument-pattern class exists | owner decision on patterns; containment remains the backstop |
| G7 | **environmental** | release-identity/installed-output legs belong to Lane 1's window, not this audit | sequence after the install window |

The three "calibration" questions Nox asked are answered: **(a)** the existing suite *does* cover the grant/guard surface's happy path and most negative legs (11/16 groups with executed evidence); **(b)** the **adversarial depth is rehearsal-grade, not external-audit-grade** — the gaps are storage invariants, owner-op refusal matrix, and the two unverifiable branches; **(c)** the honest quick-profile output is a **coverage map + gaps**, not a verdict.

## 3. Calibration result for Nox
- **Signal:** strong; the grant/guard surface is the most test-dense area of the fleet (60 policy tests + dedicated Stage 3 grant/access legs + 43 policy-relevant mutants + 3 independent reviews).
- **Cost of this run:** ~35 minutes, source-first, **0 executes, 0 spend, 0 install, 0 writes outside the board**; nothing duplicated from Team B's profile (it supplied the scope; this run supplied the coverage mapping).
- **Proposed full-audit depth (for Nox's order):** S1 4–6 h with an execution leg inside an OS-enforced sandbox (G1/G2/G4) + the real-install journal-clean run (G3); S2 action-log 2 h (chain/substitution/crash-window), S3 authority matrix 2 h, S4 containment **compare-only** 1 h (already Stage 2/3-covered). Full profile recommended only after the real-install window closes, so it audits the installed output, not the scratch tree.
- **Fences honoured:** findings never git-tracked; board carries counts/states/pointers only; no secret, credential, host/process or exploit detail; no new adversarial tooling; no service action. Any future candidate finding body goes to a hidden local working file and only its count/state to the board — **none was produced by this run**.

## 4. Limits (as written, not smoothed)
1. **Evidence was reused, not re-run** — the suites and Stage 3 rehearsal are Team B's records; my independent contribution is the static read + coverage classification. A full audit re-runs in its own sandbox.
2. **No execution leg** from this seat (no OS-enforced sandbox available), so execution-dependent leads sit at `needs_validation` by design, not as failures.
3. **S1 only**: S2/S3 touched only where the same code path crossed; S4 excluded (covered elsewhere; compare-only).
4. The reviewer-finding counts above are taken from the filed review records and their R6 dispositions; I did not re-adjudicate those findings.
5. Team B's calibration profile was filed while this run was in progress — I reused its scope rather than duplicating it; both files now exist, and the Lead/Nox should read them together (profile = scope+baseline; this = coverage mapping+gap calls).
