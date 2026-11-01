# FLEET-CHECKER — BASELINE VERIFICATION (Stage 2 / Mission 1, Jobs 06–09 gate)

Owner: `fleet-checker` (independent fresh-context checker, Team B). Window **2026-10-31 18:46–20:05 ET** (22:46Z–00:05Z).
Scope: baseline batch V1–V5 of my charter, plus the desktop-residue certification work ordered by the Lead at 19:24 ET.
Status: **BASELINE — NOT A CERTIFICATION.** No gate row is PASS. Certification is issued separately only when every leg is green/evidenced on the named final tree and Nox's independent run is on record.
Fences held: read-only repo inspection; all reproductions in my own throwaway trees under `/home/cayleb/fc-checker-work/` (git-archive extractions); **no mission-branch write, no merge, no deploy, no install, no force-push, no spend, no credential handling, no write to `~/Work/shared-workflows/lanes/board/**`, no write to `~/Documents/Codex/**`, no write over any maker file.** The maker's material was read as CLAIMS, never as evidence.

## 0. Seat and observed identity (2026-10-31)

| Item | Observed value (my run) |
|---|---|
| Host | outis-desktop, Linux, 32 CPUs, load 3–22 during the batch |
| Repo | `~/workspace/harness-build/dsh-fleet` (origin = cayleb-james2008/dsh-fleet) |
| Remote refs @ 19:18:51 ET | `main = 4a8a1d3f0804…`, `next = 0769797e2d32…`, `bus = e8c8a4c2ad37…` |
| Remote refs @ 19:49:29 ET | + `integration/0.2.2 = cf93107bc6e3…` (pushed; Lead's order) |
| Candidate | `0769797e2d32d89d3c22d0bd1cb2d94aef92ca52` (`origin/next`), declares **0.2.1** |
| Pre-fix R6 WIP | `5184978` (parent of 0769797; the recorded gate run's base) |
| Accepted main | `4a8a1d3f080416ccb545bee60655117584c960fd` |
| Final integrated tree (named by Lead) | branch `integration/0.2.2`, head `cf93107bc6e31ed265332b005d75b63d8acb300e`, tree `e15c17a3c93919de6b627e791a469dd9ddeefb28`, package 0.2.2, worktree clean |
| My seat for candidate work | `git archive 0769797` → `/home/cayleb/fc-checker-work/cand` (no `.git`) |
| My seat for final-tree work | `git archive cf93107` → `/home/cayleb/fc-checker-work/final`; tar sha256 `407d5f5969e75baf39e711fb5a68bcd618a6fe423297d90bd1ede4f58bbb3b14` |
| Toolchain | node v26.7.0 (`/usr/bin/node` 26.12.0 in the runtime), python 3.14.8 |

Commands (quoted):
```
$ cd ~/workspace/harness-build/dsh-fleet && git ls-remote origin
4a8a1d3f080416ccb545bee60655117584c960fd	HEAD
e8c8a4c2ad37ea9286c2b97b3d33af762fb19e74	refs/heads/bus
cf93107bc6e31ed265332b005d75b63d8acb300e	refs/heads/integration/0.2.2
4a8a1d3f080416ccb545bee60655117584c960fd	refs/heads/main
0769797e2d32d89d3c22d0bd1cb2d94aef92ca52	refs/heads/next
$ cd ~/workspace/harness-build/dsh-fleet-integration && git rev-parse HEAD HEAD^{tree}
cf93107bc6e31ed265332b005d75b63d8acb300e
e15c17a3c93919de6b627e791a469dd9ddeefb28
$ git archive --format=tar HEAD | tee ~/fc-checker-work/final-cf93107.tar | tar -x -C ~/fc-checker-work/final
407d5f5969e75baf39e711fb5a68bcd618a6fe423297d90bd1ede4f58bbb3b14  final-cf93107.tar
```

## 1. V1 — independent reproduction (candidate 0769797, my throwaway tree)

All commands run with `PATH=/usr/bin:$PATH`, `nice -n 19`, a short TMPDIR **on disk** (`/home/cayleb/fc-checker-work/t/*`) after `/tmp` hit 98% inodes (see D10). Shell: bash. Timestamps ET.

### 1.1 Token-free gate runner and suite totals

The runner itself is `ops/gates/run-gates.sh` at **main 4a8a1d3 only** (added in `913c0c7`, 14:25:35 ET). It is **not in the candidate tree**; the recorded run was the main-side script driving the `dsh-fleet-next` worktree. I ran the same script logic from a copy in my seat with only the TMPDIR path adapted (`/tmp/s6g` → disk path; `run-suites.sh` only re-pointed; sha256 of my runner `7032f66c…`), then the individual legs directly on the candidate tree. **The duplicate run was stopped at the Lead's 19:10 steer; only desktop-vantage suite outcomes from it are used.**

| Leg | My command (abridged) | Result | Time |
|---|---|---|---|
| lint | `signal-scope-lint.sh <tree>/package <tree>/ops <tree>/checks` | `signal-scope-lint: clean` | 19:08:26 |
| sandbox | `cd package/src/sandbox && TMPDIR=<disk> ./run-tests.sh` | **52/52 pass, rc=0** | 19:08:31 |
| fsguard | `cd package/src/fsguard && TMPDIR=<short> node --import ./test/register.mjs --test 'test/*.test.mjs'` | **98/98 pass, rc=0** | 19:08:35 |
| housekeeper | `cd package/src/housekeeper && TMPDIR=<short> node --test test/*.test.mjs` | **21/21 pass, rc=0** | 19:08:38 |
| egress (bare) | `cd package/src/egress && node --test tests/*.test.js` | **2 files, 2 fail**: `ERR_MODULE_NOT_FOUND undici` (env, not code) | 19:08 |
| egress (with read-only deps) | link `node_modules/{undici,@deepseek-ai,@modelcontextprotocol,koffi}` → pinned DSH runtime; rerun | **14/14 pass, rc=0** | 19:33 |
| client (clean env) | `env -u NODE_PATH node --test src/client/test/` | **2 files, 2 fail**: `Cannot find module 'react'` (env) | 19:32 |
| client (pinned deps) | link react/react-dom 18.3.1 + tsdown 0.22.2 + lightningcss 1.33.0 + rolldown 1.1.1 read-only; `env -u NODE_PATH DSH_PIN_CHECKOUT=<pinned checkout> node --test src/client/test/` | **10/10 pass, rc=0** (8 client + 2 conformance, incl. the build-conformance test) | 19:33 |
| core | `cd package/src/core && node --test test/` | **99/99 pass, rc=0** | 19:34 |
| policy / ledger / lessons / orders (full) | `node --import ../core/test/_register.mjs --test 'test/*.test.mjs'` | **65/65, 12/12, 49/49, 196/196 pass, rc=0** | 19:35–19:40 |

Quoted:
```
$ TMPDIR=/home/cayleb/fc-checker-work/t/egr /usr/bin/node --test tests/*.test.js   # src/egress, with read-only deps
ℹ tests 14  ℹ pass 14  ℹ fail 0
$ env -u NODE_PATH DSH_PIN_CHECKOUT=/home/cayleb/Documents/Codex/2026-10-30/task-2/dsh-runtime /usr/bin/node --test src/client/test/
✔ committed client.js is exactly what the pinned DSH preset builds from index.js (326.305186ms)
ℹ tests 10  ℹ pass 10  ℹ fail 0
$ cd package/src/orders && node --import ../core/test/_register.mjs --test test/*.test.mjs
ℹ tests 196  ℹ pass 196  ℹ fail 0      (19:34 and again 19:40)
```

The path into a green run is a **dependency-environment** question, not a product one: bare archive trees cannot resolve `react`/`undici`/`@deepseek-ai/*`; with the repo's pinned versions linked read-only (or `npm ci` in `src/client/build`, as Nox did) every suite is green. **NODE_PATH must be unset for a faithful client run**: the DSH desktop process exports a NODE_PATH containing react 18.3.1, which makes the client suite pass for the wrong reason (observed: 9 pass/1 skip under the exported env vs 10/10 with the pinned links and clean env). See D12.

### 1.2 Walker (candidate tree)

`node walker.mjs replay 1546 400` (the pinned seed and the four-way differential) is reported in §4. The full gate-range sweeps are §9 (final tree). On the candidate tree I additionally ran the package walker suite once: **2/2 tests pass, rc=0** (`walk.test.mjs`, 240 seeds across normal/biased/crash, 11.1 s, 19:07:50).

### 1.3 Mutation runs (candidate tree)

| Set | Command | Result | Time |
|---|---|---|---|
| **policy, 68 mutants** | `cd package/src/policy && TMPDIR=<disk> MUTATE_WORKERS=2 python3 -I test/mutate.py` | **68 KILLED, 0 SURVIVED, 0 NO-MATCH, rc=0** | 19:42:53–19:45:07 |
| orders, 175 mutants (attempt 1) | `cd package/src/orders && TMPDIR=/tmp/fcm …` | **aborted: ENOSPC at ~m40** (`/tmp` inode exhaustion, 98% used); discarded as environmental | 19:21–19:28 |
| orders, 175 mutants (attempt 2) | disk TMPDIR, MUTATE_WORKERS=2 | abandoned at m15 (all KILLED so far) when the Lead ordered no duplication of Team B's in-flight full run; see §9 for the final-tree disposition | 19:50–19:56 |
| orders, 10 documented survivors | `python3 -I test/mutate.py 23 24 41 42 70 89 110 116 152 156` | **10/10 SURVIVED** (matches the maker's re-run claim; equivalence judged in §3) | 19:46–19:52 |

Quoted (policy, tail):
```
62 KILLED (2)     R6-4: the summary omits the argument hash
63 KILLED (2)     R6-3: a replaced task retains its automatic spend ticket
64 KILLED (1)     R6-3: an exhausted order lets another task call the provider
65 KILLED (1)     R6-4: confirm prints nothing before pkexec
66 KILLED (1)     R6-4: confirm prints an agent's control characters raw
67 KILLED (2)     R6-6: a hold is answered tool-error
END policy 2026-10-31T19:45:07-04:00 rc=0
```

## 2. Checker-grade finding — the mutation-count reconciliation (resolved for policy; structural)

Team B's phase-1 figures (orders 164 KILLED / 10 SURVIVED of 174; policy 65 KILLED / 0 of 65) and Nox's independent figures (orders 146/175 KILLED, 29 not killed; policy 52/68 KILLED, 16 not killed, "R6-tagged 29/32 die") are **not the same measurement**. My findings:

1. **Different sets — verified by parsing the harness itself.** `M` in `package/src/orders/test/mutate.py`: **174** entries at `5184978`, **175** at `0769797` (+1 R6 row). `package/src/policy/test/mutate.py`: **65** at `5184978`, **68** at `0769797` (+3 R6 rows). So the candidate sets are exactly Nox's 175/68. Team B's numbers describe the WIP tree; Nox's describe the candidate.
2. **Different vantage — verified by suite-composition arithmetic.** Nox's runnable denominators are the desktop totals minus exactly the composed-host files, which his VM cannot run:

| Component | Desktop, candidate (my runs) | Nox's run | Delta | Composition of delta |
|---|---|---|---|---|
| policy | 65 tests | 43 | 22 | `plugin.test.mjs` 7 + `stage3.e2e.test.mjs` 15 |
| ledger | 12 | 6 | 6 | `plugin.test.mjs` 6 |
| lessons | 49 | 41 | 8 | `plugin.test.mjs` 8 |
| orders | 196 | 162 | 34 | `plugin.test.mjs` 18 + `finisher.test.mjs` 16 |
| sandbox/egress/fsguard | 52 / 14 / 98 | not runnable | — | peer anchors / bwrap / host runtime |
| core | 99 | not reported | — | — |

   The mutation harness runs a **fixed suite list that includes those files** (`schema, claims, board, flow, cycle, plugin, standalone, finisher, freshness, liveness, walk, spend` + `race` for claim/lease mutants; policy: `holds, log, plugin, stage3.e2e` (+`settle` R6)). A mutant whose only killer is in a file the runner could not run reports **SURVIVED** — it is not evidence that the guard is absent.
3. **Decisive test — desktop policy run.** On the candidate tree, with the standard harness (which includes the composed-host plugin/e2e files), my independent run is **68/68 KILLED, 0 SURVIVED**. Nox's 16 policy non-kills (including the 3 R6-tagged he named) therefore **die on the desktop**. The survivor inflation is a vantage artefact, not a candidate defect and not a wrong number on either side.
4. **Orders half — structural, totals pending the final-tree run.** Expected shape: desktop full run ⇒ exactly the 10 documented equivalents survive (165/175 KILLED) while Nox's 29 non-kills = those 10 + ~19 mutants whose killer is in `plugin.test.mjs`/`finisher.test.mjs` (which his runnable set omits). **Exact per-mutant diff against Nox's list is not yet possible: his full report is on his side and not readable from this filesystem; his mail gives aggregates only.** Resolve by either (a) his per-mutant survivor list, or (b) a full desktop run on the final tree (§9). I have run (b) partially and Team B's full run is on the same tree; certification will not accept the disposition until one of these is complete and the 10 survivors are the *only* survivors.

**Verdict wording for the gate report:** the two runs are not comparable as "kill totals"; the candidate-set desktop run is the governing measurement, and Nox's run is independent evidence on the same code with a reduced suite surface. The "41 policy mutants" phrase in the brief remains the denied-op exclusion count in the safe subset (accepted clarification), not a kill total.

## 3. V2 — the 10 surviving orders mutants: my independent judgement (source at `0769797`, file:line verified by me)

Method: extract exact mutated text from `M[i]`, read the current source at the claimed guard and the whole call path, decide whether the rule the mutation breaks is still enforced. I did **not** trust the triage table; the line numbers below are ones I verified.

| # | Mutated rule | My judgement | Evidence I verified (candidate source) |
|---|---|---|---|
| 23 | `roll-up without final task` — removes ` \|\| final === undefined` in `rollUp` | **EQUIVALENT** | The final task is inserted in the same decompose transaction that sets `report_post_id` (`orders/tasks.js:148` under the "no non-failed final" guard; `report_post_id` written only at `orders/orders.js:414`); it is non-replaceable (`tasks.js:104,111` `final-is-harness-owned`), un-deletable (`schema.js:163` delete trigger), and a failed final still has `final=1` and is caught by `tasks.some(t=>t.state!=='done')` first. So at the point the removed clause matters (all non-replaced tasks done, final absent) cannot occur for an in-progress order. |
| 24 | `roll-up with unfinished tasks` — removes the `t.state !== 'done'` term | **EQUIVALENT** | Banking guard `orders/orders.js:432` returns unless every non-replaced task has `banked===1`. Only a done task can be banked: `markBanked` refuses non-done (`tasks.js:806`), the final banks only on its done transition (`tasks.js:470`), and a done task leaving done clears `banked:0` (`releaseFinal`, `tasks.js:~735`). No non-done row can carry `banked=1`. |
| 41 | `maker records verdict` — removes `row.maker === actor.principal` refusal in `recordVerdict` | **EQUIVALENT** | The verdict path first requires the check-window holder (`tasks.js:535` → `orders/leases.js:14` claimant+token); `beginCheck` refuses `checker === row.maker` (`tasks.js:441`) and sets `claimant = checker` (`tasks.js:449-451`), so the maker can never be the holder; external verdicts are `owner`-role. The removed line is a second defence behind a structural one. |
| 42 | `double checker verdict` — removes the `verdictExists` pre-check | **EQUIVALENT** | DB `UNIQUE (task_id, round, source)` (`schema.js:212`); `insertVerdict` maps a UNIQUE violation to the same `verdict-exists` (`tasks.js:513-521`); and the task has already left `checking` (`not-checking`, `tasks.js:~532`). |
| 70 | `R2-1 any checker pass rolls up a command done-when` — removes the `command && !finisherPassed(final)` guard | **EQUIVALENT** | A command-mode final is done only via a checker verdict (`markDone` is reached only from `recordVerdict`/the finisher's own pass), and R5-9 refuses any non-finisher checker on a command final (`tasks.js:443-445`, `FINISHER_CHECKER` `kit.js:277`). So every done command final has a finisher pass and `finisherPassed(final)` (`orders/orders.js:441`) is true whenever the removed guard would be evaluated. **Caveat:** equivalence depends on R5-9; it is not a free-standing defence. |
| 89 | `R2-4 external verdict does not restart the lease` — removes one `settleOrderLease` in `externalVerdict` | **EQUIVALENT** | Only two branches change state and both settle themselves: `applyFail` (`tasks.js:495`) and `markDone` (`tasks.js:477`). The remaining branch (external verdict first, checker pass not yet recorded) changes no lease input: the task stays `checking` with `waiting_on` null, and the wait predicate reads only `waiting_on`/escalations/spend waits (`views.js orderWaitingOn`/`finalWait`). `settleOrderLease` is idempotent. |
| 110 | `R3-5 a fixture skill tries to land` — removes the `fixture → landed=false` short-circuit in `review()` | **EQUIVALENT in security property, with a note** | `land()` itself refuses fixtures (`lessons.js:477` `land-refused`), and the caller's `catch` sets `landed=false` (`lessons.js:419`). The security rule (a fixture never leaves the DB) still holds. **Not behaviourally identical:** the mutated path additionally exposes `land_error` and logs. A test asserting the clean fixture path would kill it; none exists. I judge this **no real gap**, and no test is required for the gate — but the gate report should not call this mutant "identical", only "second defence". |
| 116 | `R3-9 filing with no task says banked` — flips `let banks = false` to `true` | **EQUIVALENT** | `banks` only gates `#notifyBanked`; with `task===''` the banking wire's `bank()` does `orders.getTask('') === null` and no-ops (`banking.js:44-45`); the answer's `banked` is board-derived and false for an empty task (`lessons.js:303-306`). No stored row changes. |
| 152 | `R5-2 a fault leaves the Lead's lease unsettled` — removes `settleOrderLease` after the `harness-fault` post in `dropFinal` | **EQUIVALENT under the shipped composition, with a note** | Every state change in `dropFinal` is a `K.transition`, which emits a `transition` notice naming the order (`kit.js:190-192`); fleet-policy's level-triggered settle runs on every such notice (`store.js watchOrders` 125-141, `policy/port.js:42`), and `systemPost` also names the order. So with the policy wire mounted (production package + the walker) the explicit settle is redundant. **In an orders-only embed it would matter**; no shipped configuration omits fleet-policy. |
| 156 | `R5-4 a run on an order that ended is a fault` — removes the `TERMINAL` order check in `windowClosed` | **EQUIVALENT** | Ending an order closes every live task and check window (`orders/orders.js:315-319 endTasks`, called by fail/cancel; the done path ends the final's windows in `markDone`), so the pass's own windows end with a non-`built` reason and `endedAs(...)` makes `windowClosed` true without the TERMINAL clause (`finisher.js:219-236`); a concurrent deciding verdict also trips the verdict clause. |

**Conclusion (V2): 10/10 are second-defence/equivalence cases; zero real gaps; zero new regression tests required for these ten.** My independent re-run reproduces 10/10 SURVIVED on `0769797` (§1.3). Two judgements carry a caveat the gate report must keep (110: observable `land_error`; 152: conditional on the policy wire); the other eight are structural.

## 4. V3 — seed-1546 differential: verified independently, four ways

Trees: `pre` = `git archive 5184978`, `cand` = `git archive 0769797`; `cfg3` = cand package + pre-fix `walker.mjs`; `cfg4` = pre package + **only** the candidate's benign-raise hunk in `walker.mjs` (applied by me; diff vs pre = exactly that hunk). Command each run:
```
cd <tree>/package/src/orders && TMPDIR=<short> WALK_RESTART=mixed WALK_DBROOT=<dir> \
  nice -n 19 /usr/bin/node --import ../core/test/_register.mjs test/walker.mjs replay 1546 400
```

| Config | Package | Walker | Findings | Runs | Determinism |
|---|---|---|---|---|---|
| 1 | pre `5184978` | pre | **2 × `strand-never-completes`**, step 10149, details `o2 …t14:openFr0/3` and `o3 t1:blocked(budget-exhausted)` | 3/3 | **identical findings + stats hash `c62ca97d4f7c`** |
| 2 | cand `0769797` | cand | **0** | 2/2 | hash `022c9959b489` |
| 3 | cand `0769797` | **pre** | **2** (identical details to config 1) | 2/2 | hash `c62ca97d4f7c` ⇒ the package fixes are **not** what closes 1546 |
| 4 | **pre** `5184978` | pre + **raise hunk only** | **0** | 2/2 | hash `022c9959b489` ⇒ the walker raise alone flips the seed |

All four configs report the same `stats`: `leaseLost 20, spendTickets 14, exhausted 15, raises 8, resumed 7, restarts {plain:0, host:4}, alerts 1, clockSweeps 10` — identical to the committed `evidence/r6-gates/replay-1546.txt` and to the failure line in `walk-n400.txt`. Walker delta `5184978→0769797` is **exactly two hunks** in `walker.mjs` (header claim + benign-raise amount); the committed change isolates it as `ee14591`, and `board/probes/walker-raise-0769797.patch` sha256 `3208e485…` (60 lines, 4 hunks incl. the `walk.test.mjs` steps parameter + pinned test) matches the claimed hash and content. The emitted invariant-kind set is unchanged: **52 kinds before and after** (set diff empty).

**Verdicts:**
- "pre-fix tree reproduces deterministically" — **TRUE** (3/3, byte-identical findings/stats).
- "the walker-raise hunk is the discriminating change" — **TRUE** (config 3 red / config 4 green).
- "fixture-completion-gap, not a package defect" wording — **HONEST as to mechanism**: the failure is the fixture driver's benign phase (tiny repeated raises starving unrelated timers), and the package fixes neither cause nor cure it. The gate report must keep three guards on the claim: (i) the walker header **dropped** its aggregate-envelope assertion — record that as a known limit, not silently; (ii) the pinned `R6-10` test passes on the pre-fix package under the new walker (config 4), so it is a **fixture-path guard**, and the report should label it so; (iii) the walker raise landed as its own reviewed commit (`ee14591`), which Nox's condition requires.
- What **"n400 seed 1546 GREEN on the final tree" must show**: on the named final tree (a) the pinned single-seed R6-10 test passes, **and** (b) the full n400 sweep (normal 1001–2000 @400 steps, rc=0, **zero findings**, raw JSONL/trace retained), with tree hash + command + timestamps; the single seed alone is **not** the sweep and must not be labelled as such (Team B's manifest already draws this distinction — I hold them to it).

## 5. V5 — theatre audit of the DRAFT gate report (current version, updated 19:22 ET / 23:22Z)

What is honest in the draft: every `PENDING/UNVERIFIED/NOT DONE` label is genuine, the five-high leg is correctly named unverified (no soft pass), the client disposition is honest, Nox's verdict is quoted rather than paraphrased into a pass, and §9 says "no residue leg has run" truthfully for its timestamp. Items I name as theatre-adjacent or stale (each must be fixed before the report can be certified):

1. **Row 1 "Token-free gate runner — PASS (recorded run)".** The runner completed, but its own `summary.txt` records `walker n400 rc=1 ℹ pass 0 ℹ fail 1`. The runner has no failure exit and `DONE` merely means it finished. Labelling the leg PASS while one of its legs failed is a soft pass. Correct wording: "runner completed (15 legs; 14 pass, 1 fail — walker n400 seed 1546 on 5184978)".
2. **§2 heading "Our final-tree record on `0769797`".** `0769797` is the candidate, declares 0.2.1, and is not the final tree. Calling candidate evidence "final-tree" is exactly the inference §7 forbids. Rename to "candidate-tree record".
3. **§4a conclusion "no contradiction" written before the desktop evidence existed.** At 19:22 the reconciliation was inference only. It is now true for policy (my 68/68 run, §2.3) but the draft should cite the governing desktop run, not assert. Same for "The remaining 13 non-kills are VM-vantage classes" — now evidenced in kind, not yet per mutant.
4. **Row 3 title "Client suite (2 tests)".** The suite is **10 tests** (8 client + 2 conformance); 2 was the number of files that failed to load in the recorded run. Stale label that makes the residue look smaller than it is.
5. **Row 15 disposition list omits Security F9** (`R6-DISPOSITIONS.md` row "Security F9: fsguard denials without intent rows"; Outis's 23:18Z correction). The parenthetical list must include it; Contract F11's missing disposition is correctly named.
6. **Row 2/§4a omit the core suite** (99 tests; core also has a 24-test composed-host plugin suite). Nox's list does not cover it; it is desktop-only residue and must be a row, not an omission.
7. **§9 "no residue leg has run"** is stale as of the current version: residue attempt 2 ran at 19:30–19:41 (Team B manifest) and attempt 1 is preserved as discarded-environmental. Update the provenance paragraph.
8. **Manifest vs evidence tree mismatch (record, not theatre).** `MANIFEST.md` names final commit `95804ba` / tree `84ba3d8f…`, but every residue `tree.txt` (and my own seat) records `cf93107` / `e15c17a3…` — a docs-only commit (`docs/INTEGRATION-NOTE-2026-10-31.md`, +33 lines) landed after the manifest table was written. Since the manifest itself says any extra commit re-runs every leg, either re-name the final tree as `cf93107`/`e15c17a3` (code identity unchanged, one docs file) or re-run. **Settle this before certification.**
9. **Team B's host-judged full orders mutation runs `MUTATE_WORKERS=4`,** while the standing order says at most **2 mutants in parallel**. Their evidence must disclose the deviation and the load; it does not change kill/survive semantics but it weakens timing and cross-run comparability.
10. **My own correction to the record:** the gate runner's `walk biased 500 @300` and `walk crash 500 @300` commands **do not run the labelled mixes**. `walk.test.mjs` overrides `WALK_MIX`/`WALK_CRASH` per mode from its own `MODES` table, so the runner's `WALK_MIX=biased` env is ignored and both legs run *normal* 1–500 @300 plus biased 1–30 and crash 1–30. The recorded `walk-biased.txt`/`walk-crash.txt` therefore cannot evidence "biased 500 / crash 500". The DRAFT row 5 label overstates what ran. On the final tree I run the **true** ranges (§9) with per-mode envs, not this template.
11. **Provenance gap on the recorded gate run:** the runner `run-gates.sh` exists only on `main` (not in the candidate tree) and calls two absolute paths outside the tree (`evidence/stage-2/fixround5/run-suites.sh`, `ops/test/signal-scope-lint.sh`); the summary does not name the runner version. The evidence should pin the runner's sha256 (mine: `3b283361…` for the tree version, `7032f66c…` for my path-adapted copy) and say which seat ran it.
12. **`abfb842`:** the DRAFT header, §16 and Team B phase1 §6 now carry Nox ruling B correctly ("local-only docs commit in the desktop clone, never present on any remote"). One stale contradiction remains: **`team-b-phase1.md` §8 item 1 still carries the pre-ruling text "Choose: (A) push… *Recommend A"** while the same file's §6/header record ruling B. Fix or strike §8 item 1; otherwise a reader can follow a struck option.

## 6. abfb842 and "nothing was pushed" — verified

```
$ cd ~/workspace/harness-build/dsh-fleet && git ls-remote origin | grep -i abfb842   # -> no output
$ git for-each-ref --contains abfb842 --format='%(refname)'                          # -> refs/heads/main (local only)
$ git merge-base --is-ancestor abfb842 origin/main && echo ancestor || echo "NOT ancestor"   # -> NOT ancestor
$ git diff --name-status 4a8a1d3 abfb842   # M docs/ASSUMPTIONS.md; A docs/STAGE-4B-DESIGN-INPUT-2026-10-31.md; M docs/fleet-plugin-design.md
```
Remote refs at 19:18:51 ET were identical to Team B's first observation (main `4a8a1d3`, next `0769797`, bus `e8c8a4c`); `abfb842` appears on no remote ref and no remote branch. Nox's ruling B (strike, content archived in `task/mission1/owner-steering-docs.patch`, 9,946 B, same three file blocks) is consistent with the record. The integration branch `integration/0.2.2 = cf93107` was pushed after the Lead's explicit order (seen at 19:49:29 ET) — that is the only remote movement in my window. **Nothing else was pushed; `main` and `next` are untouched.**

## 7. V4 — certification checklist (what I will run, what must be evidenced, threshold)

Vantage codes: **D** = desktop-only (Nox cannot run it), **M** = muse/Nox evidence acceptable (his run on record 23:05Z), **B** = both. Nothing is PASS until green on the named final tree.

| # | Leg | Cert command (my seat, final tree) | Evidence it must produce | Threshold | Vantage |
|---|---|---|---|---|---|
| 1 | Token-free gate runner | `bash run-gates.sh <final>/package <out>` (pin runner sha256) | `summary.txt` + `DONE` + runner sha256 + seat | runner completes; every leg result named (no aggregate "PASS") | D |
| 2 | Unit suites sandbox/egress/fsguard/housekeeper (+core) | per-component commands as in §1.1 | raw stdout, exact tests/pass/fail rc | 0 fail each; dependency links recorded read-only | D |
| 3 | Client + conformance | `env -u NODE_PATH DSH_PIN_CHECKOUT=<pin> node --test src/client/test/` after read-only 18.3.1/tsdown-0.22.2 links (or recorded `npm ci`) | raw output + NODE_PATH state + link list + versions | 10/10 pass (8 client + 2 conformance incl. build conformance) | D |
| 4 | Race ×10 | `node --test test/race.test.mjs` ×10 with load recorded | 10 raw files + rc | 10/10 rc=0 | B (Nox 10/10 on record; final-tree re-run preferred) |
| 5 | Walker n200 1–1000 @200 (+bi/cr 30 each) | `WALK_SEEDS=1-1000 WALK_STEPS=200 WALK_WORKERS=4 walk.test.mjs` | raw + diagnostics (seed count, restarts, crashes) | 0 findings, rc=0, both restart modes, crashes>0 | D (final tree) |
| 6 | Walker n400 1001–2000 @400 | `WALK_SEEDS=1001-2000 WALK_STEPS=400` | raw | 0 findings, rc=0 | D (final tree; Nox's candidate run is supporting) |
| 7 | Walker biased 500 @300 | `WALK_BIASED_SEEDS=1-500 WALK_STEPS=300` with small normal/crash ranges | raw + mode-labelled seed counts | 0 findings, rc=0 | D |
| 8 | Walker crash 500 @300 | `WALK_CRASH_SEEDS=1-500 WALK_STEPS=300` | raw + crashes count | 0 findings, rc=0 | D |
| 9 | Seed 1546 pinned | `--test-name-pattern="R6-10: seed 1546"` on walk.test | raw + tree hash | 1/1 pass | D |
| 10 | Orders mutation full | `TMPDIR=<disk> MUTATE_WORKERS=2 python3 -I test/mutate.py` (control first) | control PASS + per-mutant lines + survivor list | survivors ⊆ documented equivalents (10), each judged (§3); 0 NO-MATCH | D (authoritative) |
| 11 | Policy mutation full | same, `src/policy` | per-mutant lines | 0 survivors | D |
| 12 | Composed-host plugin (core/policy/orders/lessons/ledger) | `node --import ../core/test/_register.mjs --test test/plugin.test.mjs` per component (core: own `_register`) | 5 raw files + counts | 24/24, 7/7, 18/18, 8/8, 6/6 | D |
| 13 | Composed-host e2e (Stage 3) | `node --import ../core/test/_register.mjs --test test/stage3.e2e.test.mjs` | raw | 15/15 | D |
| 14 | Service-state vantage | `systemctl show fleet-dsh …`; socket connect probe | quoted output + timestamps | unit state + socket reachability as observed; installed identity is row 18 | D |
| 15 | R6-10 remainders | dispositions incl. Security F9; one regression test per verified high or named unverified | updated `R6-DISPOSITIONS.md` + tests | every entry finished or named | B (record) |
| 16 | Repo reconciliation | one agreed main; no force-push; `ls-remote` before/after | SHA ledger | main/next preserved; final tree named | B |
| 17 | Formal Stage 2 pass by fresh checker | this seat's certification report | veto or FAIL | all rows green/evidenced | D |
| 18 | RELEASE-IDENTITY | verify the accepted **artefact** (name+sha256+byte identity), then release lane installs and reads back running identity | artefact hash + byte-compare; wrapper output; `plugin-add`/loaded-version readback | artefact = final tree; running identity readback matches **before** the formal check | D (install = release lane; I verify) |
| 19 | Residue bundle above on the named tree | all of #1–#14 bound to `tree.txt` | per-leg `tree.txt`/`command.txt`/`raw/`/`result.txt` | tree hash identical in every leg file | D |

Rows that **must not be inferred** (the Lead's list): full n400 sweep; n200/biased/crash ranges; rehearsals (scratch Stage 2 100-entity, Stage 3 default — no run/evidence exists yet); and my veto itself. Rehearsals are the one gate row with **no owner evidence at all**; a certification cannot close the gate while they are NOT DONE unless the Lead rules them out explicitly.

## 8. Certification progress on the final tree `cf93107` (updated 20:12 ET) — VETO HELD, NOT CERTIFIED

Certification document: **`board/reports/fleet-checker-certification-cf93107.md`**. Evidence: `board/probes/team-b-final-tree/checker-verify/` (my seat; maker files untouched).

Green on my seat against `cf93107bc6e3…` / tree `e15c17a3…` / package 0.2.2 (commands in the certification doc):

| Leg | Result |
|---|---|
| sandbox / egress / fsguard / housekeeper | **52/52 · 14/14 · 98/98 · 21/21** pass rc=0 |
| core / policy / ledger / lessons / orders full suites | **99/99 · 65/65 · 12/12 · 49/49 · 196/196** pass rc=0 |
| composed-host plugin (core/policy/orders/lessons/ledger) | **24/24 · 7/7 · 18/18 · 8/8 · 6/6** pass rc=0 |
| composed-host e2e (Stage 3) | **15/15** pass rc=0 |
| client conformance (NODE_PATH unset, pinned checkout, read-only pinned deps) | **10/10** pass rc=0 |
| walker n200 1–1000@200, n400 1001–2000@400, biased 500@300, crash 500@300 | **all PASS, 0 findings** (1060 / 1060 / 550 / 550 seeds; n400 = the full sweep) |
| mutation policy, full 68 | **68/68 KILLED, 0 SURVIVED, 0 NO-MATCH**; controls PASS |
| release artefact | `cayleb-dsh-fleet-0.2.2.tgz` sha256 `37e3a542…` = sidecar; **82/82 packed files byte-identical to this tree**; version 0.2.2 |

Blockers (named in the certification doc, not inferred): **B1** orders mutation full 175 on the final tree — **CLOSED: PASS** (165/175 KILLED, 10 SURVIVED = the documented equivalents, 0 NO-MATCH, control PASS; evidence `checker-verify/mutation-orders-final/`); **B2** RELEASE-IDENTITY row 18 — no install/readback exists (`/var/lib/fleet-drop` still 0.2.0/0.2.1; `fleet-dsh` still the 0.2.1 install); **B3** rehearsals row 12 — no run; **B4** five-high row 11 — unverified (name it, do not waive); **B5** walker-change review record — `ee14591` documents the patch but no reviewer artefact is in the repo; **B6** manifest tree identity mismatch (`95804ba`/`84ba3d8f` named vs `cf93107`/`e15c17a3` evidence — one docs-only commit later; settle or re-run); **B7** R6-10 remainders row 15 incl. Security F9; **B8** reconciliation: integration tree correct (main ancestor, 0 deletions, evidence identical, abfb842 excluded) but no merge to `main`; **B9** Team B's host-judged full-orders run died at 81/175 (its step-3 evidence is incomplete; steps 1–2 were 32/32 and 16/16 KILLED); **B10** Nox's independent run is on record for candidate 0769797 — the final-tree n400 sweep he requires is now green in my evidence and should be acknowledged.

## 9. Provenance, spend and fences

- All runs in this report were executed by me in `/home/cayleb/fc-checker-work/` (extractions, links, TMPDIRs) and logged under `/home/cayleb/fc-checker-work/out/`. No maker tree was written; no board-maker file was edited; the shared repo worktrees were read only.
- Spend: **$0** (no paid calls; local CPU only). Credentials: none touched. Network: none used except local sockets/tests.
- Duplicate-run discipline: I stopped my own full gate run at the Lead's 19:10 steer; I later stopped my full orders mutation when Team B's identical full run on the same tree was in flight, and switched to independent policy + survivor-set + sample verification. The desktop-vantage legs are mine; the final-tree mutation totals will be certified from a completed run (mine or verified maker raw + independent sample re-run).
- Raw logs: `/home/cayleb/fc-checker-work/out/` (`gates-cand/`, `diff/`, `final/`, `walk-*.txt`, `final-mut-*.txt`, `mut-10survivors-cand.txt`).
