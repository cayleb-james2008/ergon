# FLEET-CHECKER — CERTIFICATION, FINAL TREE `cf93107` (Stage 2 desktop residue)

Owner: `fleet-checker` (independent fresh-context checker, Team B). Issued **2026-10-31 20:10 ET** (00:10Z); **re-issued 21:02 ET (01:02Z) after B1 completed**.
**TREE OF RECORD: `cf93107` / tree `e15c17a3…`** (Lead ruling 2026-11-01T00:15Z; `95804ba`/`84ba3d8f` was the pre-docs state, docs-only difference).
**VERDICT: DESKTOP-RESIDUE SUBSET CERTIFIED GREEN on the named final tree — the STAGE 2 GATE IS NOT CLOSED.** Every residue leg the Lead ordered (a)–(e), plus the walker ranges, race and the artefact, is green and evidenced on `cf93107`; the orders mutation is now complete on signature (165/175 killed, 10 survivors = the documented equivalents). The gate cannot be certified closed because rows 11 (five-high, Nox), 12 (rehearsals, deferred by Lead ruling), 13/18 (release-identity, **blocked on owner Wait**) and 15/16 (remainders; merge to main) are not discharged. **No residue leg failed**; the blockers are not-done/owner rows, and this seat will not sign the gate PASS while they are open. Evidence: `board/probes/team-b-final-tree/checker-verify/` (index inside), raw logs `/home/cayleb/fc-checker-work/out/`.

## 1. Tree and seat certified against

| Field | Value |
|---|---|
| Branch | `integration/0.2.2` (remote `refs/heads/integration/0.2.2`) |
| Head | `cf93107bc6e31ed265332b005d75b63d8acb300e` |
| Tree SHA | `e15c17a3c93919de6b627e791a469dd9ddeefb28` |
| Package | `@cayleb/dsh-fleet` **0.2.2** (`package.json` sha256 `ef93187a…`) |
| Base | accepted main `4a8a1d3` is an ancestor; candidate `next 0769797` integrated; **`abfb842` is not an ancestor** (verified) |
| Main records | **0 deletions vs main; `evidence/` byte-identical to main** (verified: `git diff --diff-filter=D --name-only 4a8a1d3 HEAD` empty; `git diff --stat 4a8a1d3 HEAD -- evidence/` empty) |
| Walker change | own commit `ee14591` (2 files, +12/−5; patch sha256 `3208e485…` matches); commit message records the change; **no independent reviewer artefact exists in the repo** (see blocker B5) |
| My seat | `git archive cf93107` → `/home/cayleb/fc-checker-work/final`; tar sha256 `407d5f59…`; source worktree clean (0 dirty). Remote ref verified 19:49:29 ET. |
| Lint | `signal-scope-lint: clean` on the named tree (run at 19:08 on the candidate; the packaged tree is byte-identical for `package/ops/checks` — re-run recorded in the gate-runner leg) |

## 2. Legs green on `cf93107` (my commands + raw evidence; nothing inferred)

| Leg (gate row) | Command | Result | Evidence |
|---|---|---|---|
| gate-sandbox (row 2) | `cd package/src/sandbox && TMPDIR=<disk> ./run-tests.sh` | **52/52 pass rc=0** | `checker-verify/gate-sandbox/` |
| gate-egress (row 2) | `cd package/src/egress && TMPDIR=<short> node --test tests/*.test.js` (undici 8.11.2 + host runtime linked read-only) | **14/14 pass rc=0** | `checker-verify/gate-egress/` |
| gate-fsguard (row 2) | `cd package/src/fsguard && TMPDIR=<short> node --import ./test/register.mjs --test 'test/*.test.mjs'` | **98/98 pass rc=0** | `checker-verify/gate-fsguard/` |
| gate-housekeeper (row 2) | `cd package/src/housekeeper && TMPDIR=<short> node --test test/*.test.mjs` | **21/21 pass rc=0** | `checker-verify/gate-housekeeper/` |
| suite-core-full | `cd package/src/core && TMPDIR=<short> node --test test/` | **99/99 pass rc=0** | `checker-verify/suite-core-full/` |
| composed-host plugin, core (row 19) | `cd package/src/core && node --import ./test/_register.mjs --test test/plugin.test.mjs` | **24/24 pass rc=0** (maker's 24/24 re-run by me) | `checker-verify/composed-host-plugin-core/` |
| composed-host plugin, policy/orders/lessons/ledger | `cd package/src/<c> && node --import ../core/test/_register.mjs --test test/plugin.test.mjs` | **7/7, 18/18, 8/8, 6/6 pass rc=0** | `checker-verify/composed-host-plugin-*/` |
| composed-host e2e (Stage 3) | `cd package/src/policy && node --import ../core/test/_register.mjs --test test/stage3.e2e.test.mjs` | **15/15 pass rc=0** | `checker-verify/composed-host-e2e/` |
| full component suites | `node --import ../core/test/_register.mjs --test 'test/*.test.mjs'` | **policy 65/65, ledger 12/12, lessons 49/49, orders 196/196 pass rc=0** | `checker-verify/suite-*/` |
| client conformance (row 3 residue) | `cd package/src/client && env -u NODE_PATH DSH_PIN_CHECKOUT=/home/cayleb/Documents/Codex/2026-10-30/task-2/dsh-runtime node --test test/` with read-only react/react-dom 18.3.1 + tsdown 0.22.2 + lightningcss 1.33.0 + rolldown 1.1.1 links | **10/10 pass rc=0** (8 client + 2 conformance incl. build conformance) | `checker-verify/client-conformance/` |
| walker n200 1–1000 @200 (+biased/crash 30) | `WALK_SEEDS=1-1000 WALK_STEPS=200 WALK_WORKERS=4 node --import ../core/test/_register.mjs --test test/walk.test.mjs` | **PASS**: 1060 seeds, 1020 orders done, 0 findings, rc=0 | `checker-verify/walker-n200/` |
| **walker n400 1001–2000 @400** (open row) | `WALK_SEEDS=1001-2000 WALK_STEPS=400 …` | **PASS**: 1060 seeds, 2056 orders done, **0 findings**, rc=0 (339 s); pinned seed-1546 test PASS inside | `checker-verify/walker-n400/` |
| walker biased 500 @300 (true range) | `WALK_BIASED_SEEDS=1-500 WALK_STEPS=300 …` | **PASS**: 550 seeds, 510 orders done, 0 findings, rc=0 | `checker-verify/walker-biased500/` |
| walker crash 500 @300 (true range) | `WALK_CRASH_SEEDS=1-500 WALK_STEPS=300 …` | **PASS**: 550 seeds, 489 orders done, 0 findings, 2172 crash commits, rc=0 | `checker-verify/walker-crash500/` |
| mutation policy, full set 68 | `TMPDIR=<disk> MUTATE_WORKERS=2 python3 -I test/mutate.py` | **68/68 KILLED, 0 SURVIVED, 0 NO-MATCH** (part 0–50 + rerun 51–67 after an environmental copytree race) | `checker-verify/mutation-policy-final/` |
| mutation controls | `python3 -I test/mutate.py control` (orders, policy) | **PASS / PASS** | `checker-verify/mutation-controls/` |
| release artefact (row 18, artefact half) | `sha256sum cayleb-dsh-fleet-0.2.2.tgz` vs sidecar; extract; byte-compare to my seat | **sha256 `37e3a542…` = sidecar; 82/82 packed files byte-identical to `cf93107` seat; version 0.2.2** | `checker-verify/release-artefact/` |
| **Orders mutation, full 175** (row 9) | `cd package/src/orders && TMPDIR=<disk> MUTATE_WORKERS=2 nice -n 19 python3 -I test/mutate.py` (control first) | **165/175 KILLED, 10 SURVIVED, 0 NO-MATCH, rc=0**; survivors **exactly** {23,24,41,42,70,89,110,116,152,156} = the documented equivalents I judged 10/10 equivalent (baseline §3); control PASS | `checker-verify/mutation-orders-final/`, `mutation-controls/` |
| race ×10 (row 4) | `cd package/src/orders && TMPDIR=<short> nice -n 19 node --import ../core/test/_register.mjs --test test/race.test.mjs` ×10, load recorded | **10/10 rc=0** (durations 9–21 s, load 9.1–10.9) | `checker-verify/race-x10/` |
| lint (row 1 component) | `signal-scope-lint.sh <final>/package <final>/ops <final>/checks` | **signal-scope-lint: clean** | `checker-verify/lint/` |
| owner-kit hash (identity) | `cd ops && for f in <nine kit FILES>; do sha256sum "$f"; done \| sha256sum \| cut -c1-64` (same recipe as `ops/owner-setup.sh:24`) | **`2e0832bc20508e6d…` — first 12 = `2e0832bc2050`, matches the recorded Stage-1 kit hash** (owner setup kit unchanged) | this report / `checker-verify/release-artefact/` |

**Note on walker mode labels (finding):** the recorded gate runner's `walk biased 500 @300` / `walk crash 500 @300` commands do **not** run those mixes — `walk.test.mjs` overrides `WALK_MIX`/`WALK_CRASH` from its own per-mode table, so both recorded legs ran *normal* 1–500 @300 plus biased/crash 1–30. My runs above use the true per-mode ranges; the gate report must not cite the recorded labels as the claimed sweeps.

**Note on my own environment errors (honesty):** (i) my first sandbox run failed `ENOENT lstat` because I pointed TMPDIR at a nonexistent dir; (ii) one housekeeper run failed 17/21 for the same reason — both are checker-environment errors, re-run clean and green, **not product failures**; (iii) the policy mutation part 1 stopped at mutant 50 because a concurrent fsguard suite of mine left a transient `.tmp-test` dir that raced the copy — environmental; mutants 51–67 re-ran green. All recorded in the leg evidence.

## 3. BLOCKERS — why the gate is not certified (named, not inferred)

| # | Blocker | State (Lead rulings 2026-11-01T00:15Z applied) | Owner |
|---|---|---|---|
| B1 | **Orders mutation, full 175, on the final tree** | **CLOSED — PASS**: 165 KILLED / 10 SURVIVED / 0 NO-MATCH, rc=0 (20:00:04–20:59:42); survivors are exactly the ten documented equivalents; control PASS | CLOSED |
| B2 | **RELEASE-IDENTITY row 18** | **BLOCKED ON OWNER WAIT**: installation is paused under Cayleb's explicit Wait, so no install/readback exists to certify. Recorded state: `fleet-dsh` still runs 0.2.1 (MainPID 2602074, active since 12:27:26); `/var/lib/fleet-drop` holds 0.2.0 and 0.2.1 only. Release lane owns the install; Lead surfacing the owner decision to Nox. | owner / release lane; checker verifies artefacts |
| B3 | **Rehearsals row 12** | **NOT RUN — deferred by Lead ruling 2026-11-01T00:15Z; runs after B1.** No PASS language anywhere. | Team B |
| B4 | **Five-high repro row 11** | **UNVERIFIED — keep named so until the reviewers' existing repro scripts are run as five recorded tests**; no substitute, no rephrasing | Nox |
| B5 | **Walker-change review record** | **CLOSED** — checker review issued: `board/reports/fleet-checker-walker-review-ee14591.md` (SAFE and CORRECTLY SCOPED, test-only, 3 known-limit statements to carry). Team B carries the record into the handoff/gate pack. | CLOSED |
| B6 | **Manifest tree identity** | **RULED — RESOLVED BY ORDER**: final tree of record = `cf93107` / `e15c17a3…`; Team B ordered to rename MANIFEST and every leg directory to it, with a one-line provenance note that `95804ba`/`84ba3d8f` was the pre-docs state. No re-run needed for the naming fix; my seat already records `cf93107`. | Team B (rename) |
| B7 | **R6-10 remainders row 15** | open; Team B ordered to add **Security F9** to the disposition list (Outis 23:18Z) and finish each entry | Team B |
| B8 | **Repo reconciliation row 16** | integration tree correct (main ancestor; 0 deletions vs main; `evidence/` byte-identical; `abfb842` not an ancestor) but **no merge to `main`**; release merge is the release lane's step after certification | release lane |
| B9 | **Team B's host-judged mutants leg** | their raw run **died after 81/175** of the full orders set (no END line; no process); steps 1–2 completed (policy R6-tagged 32/32 KILLED, orders R6-tagged 16/16 KILLED). **My B1 independent full run supersedes it** — record the maker leg as incomplete/superseded, not as evidence. | Team B (record) / me (supersede) |
| B10 | **Nox's independent run** | on record for candidate `0769797` (mail 23:05Z). His final-tree condition ("n400 seed-1546 green on the final tree") is **now satisfied in my evidence** (n400 PASS, 0 findings, pinned test PASS). Nox's ack is being requested by the Lead. | Lead / Nox |

## 3b. Authoritative row status for the final gate report (DRAFT numbering; B1 pending)

| Row | Leg | Final-tree verdict (checker) | Evidence |
|---|---|---|---|
| 1 | Token-free gate runner | **PASS — components independently re-run on cf93107** (lint clean; suites; race 10/10; four walker ranges; both mutations). The runner's own `summary.txt` was not regenerated: the script exists only on `main` (sha256 `3b283361…`) with two absolute out-of-tree paths and a `/tmp` TMPDIR that was 98% inode-full — the per-leg evidence is the stronger form and is bound to `cf93107`. | `checker-verify/{lint,gate-*,race-x10,walker-*,mutation-*}` |
| 2 | Unit suites sandbox/egress/fsguard/housekeeper (+core) | **PASS** 52/52, 14/14, 98/98, 21/21 (+99/99) | `checker-verify/gate-*`, `suite-core-full` |
| 3 | Client suite + conformance | **PASS 10/10** (8 client + 2 conformance incl. build conformance), `NODE_PATH` unset, pinned checkout, read-only pinned deps | `checker-verify/client-conformance` |
| 4 | Race ×10 | **PASS 10/10** rc=0, load recorded | `checker-verify/race-x10` |
| 5 | Walker n200 1–1000@200 + biased 500@300 + crash 500@300 (true ranges) | **PASS**: 1060 / 550 / 550 seeds, 0 findings, both restart modes, crashes exercised | `checker-verify/walker-{n200,biased500,crash500}` |
| 6 | Walker n400 1001–2000@400 | **PASS**: 1060 seeds, 2056 orders done, **0 findings** rc=0 | `checker-verify/walker-n400` |
| 7 | Seed 1546 (inside n400 + pinned test) | **PASS**: pinned single-seed test 1/1; full sweep green | `checker-verify/walker-seed1546-pinned`, `walker-n400` |
| 8 | Candidate walker evidence at `0769797` | **superseded by the final-tree sweeps above** (historical candidate record stands as history, not as final-tree evidence) | — |
| 9 | Orders mutation, full 175 | **PASS**: 165/175 KILLED, 10 SURVIVED, 0 NO-MATCH; survivors exactly the ten documented equivalents, each judged equivalent by this checker (baseline §3); control PASS | `checker-verify/mutation-orders-final`, `mutation-controls` |
| 10 | Policy mutation, full 68 | **PASS 68/68 KILLED, 0 SURVIVED, 0 NO-MATCH** + control PASS | `checker-verify/mutation-policy-final`, `mutation-controls` |
| 11 | Five-high repro | **UNVERIFIED — no valid verdict**; must stay named so until the reviewers' existing repro scripts are run as five recorded tests (Nox) | — |
| 12 | Rehearsals | **NOT RUN — deferred by Lead ruling 2026-11-01T00:15Z; runs after B1.** No PASS language. | — |
| 13 | Real-install check | **BLOCKED ON OWNER WAIT** (installation paused; no install/readback exists; `fleet-dsh` still 0.2.1; `/var/lib/fleet-drop` 0.2.0/0.2.1) | — |
| 14 | Service-state vantage | **PASS as desktop evidence** (checker verified independently: `fleet-dsh` active/running, MainPID 2602074, InvocationID `df02e9346ebd43a680d21ae9eccc9f07`, `User=fleet`, `is-enabled=disabled`, ExecStart as recorded; `/run/fleet/control.sock` CONNECT OK, `egress.sock` permission-denied by design; installed-tree byte identity still unreadable from this scope) | Team B probe + my re-verification (20:10) |
| 15 | R6-10 remainders | **OPEN** — dispositions recorded incl. **Security F9**; Contract F11 has no disposition. Not a re-run; Team B to finish each with one regression test per verified high or name unverified. | `task/mission1/R6-DISPOSITIONS.md` |
| 16 | Repo reconciliation to one agreed main | **PASS for the integration tree** (main `4a8a1d3` ancestor; 0 deletions vs main; `evidence/` byte-identical; `abfb842` not an ancestor; candidate `0769797` integrated). Merge to `main` remains the release lane's step. | verified by me 20:16 |
| 17 | Formal Stage 2 pass | **residue subset CERTIFIED by this checker; the GATE cannot be certified closed** while rows 9 (B1 completion), 11, 12, 13/18, 15, 16(merge) are not discharged. | this report |
| 18 | RELEASE-IDENTITY | **BLOCKED ON OWNER WAIT** — release lane owns install + running-identity readback; artefact itself verified (sha256 `37e3a542…`, 82/82 files = cf93107). | `checker-verify/release-artefact` |
| 19 | Desktop residue bundle | **GREEN** except row 9 (B1 in flight); every leg bound to `cf93107`/`e15c17a3` (maker legs renamed per Lead ruling; host-judged carries a `PROVENANCE.md` normalising its pre-docs run) | `checker-verify/INDEX.md`, `team-b-final-tree/` |

## 3c. Rehearsals (row 12) — verified, with a corrected 3.4c attribution

Verified on `cf93107` (evidence: `board/probes/team-b-final-tree/rehearsals/`; my review `checker-verify/rehearsals-review/verification.md`): both legs bound to the tree/package/artefact identity; Stage 2 scratch **PASS 60/60** with `RACE_N=100`, derived check path-only substitutions, 410-row invariant clean, scratch socket never the real one; Stage 3 default mode **73 PASS / 1 FAIL / 24 MANUAL**, the only FAIL = `3.4c` journald fresh-head delivery, with `3.4a` chain verification PASS (395 rows recomputed). **3.4c classification: environment-caused, not a package defect — correct in kind but the maker's cause is wrong:** the "bad" lines are the **Stage-2 scratch host's own chain head** (seq 2861 / `4d874599…` = final row of `scratch-tb-r9/s2home/fleet/log/actions.jsonl`), left running during the Stage-3 check; the real 0.2.1 install emits no `fleet-chain` journal lines at all. The journald leg does **not** strictly require the real install — it requires a journal-clean window (stop other fleet-core hosts); it is a Stage 3 (Job 07) leg, so it does not block the Stage 2 gate, but **3.4c is not claimed PASS**.

## 4. Sign-off statement

**Residue subset: CERTIFIED GREEN on `cf93107`/`e15c17a3`.** The Lead-ordered legs (a)–(e) are all green with command+raw+timestamp bound to the named tree, the mutation reconciliation is fully resolved (policy 68/68 desktop vs Nox 52/68 = vantage artefact; orders 165/175 with only the documented equivalents), the walker ranges including the full n400 sweep are green, the client conformance is 10/10 in a pinned environment, and the 0.2.2 artefact and owner-kit identity are verified.

**Gate: formal Stage 2 PASS SIGNED on the Lead's order** (sign-off: `board/reports/fleet-checker-stage2-signoff.md`) with rows 11/12/13/18/15/16 declared as ruled. The gate report cannot be published as final until five text-only record corrections are applied (sign-off §5); the Mission-1 close-out is not complete (install/row 18 outstanding). The remaining rows are not residue evidence failures and no residue leg failed; they are:
- row 11 five-high: **UNVERIFIED** — must stay named so (Nox to run the reviewers' existing scripts as five recorded tests);
- row 12 rehearsals: **NOT RUN — deferred by Lead ruling 2026-11-01T00:15Z; runs after B1** (B1 is now complete, so this is schedulable);
- row 13/18 release-identity: **BLOCKED ON OWNER WAIT** — no install exists; `fleet-dsh` still 0.2.1; `/var/lib/fleet-drop` 0.2.0/0.2.1; release lane owns the install and the running-identity readback;
- row 15 R6-10 remainders: open dispositions (Security F9 now recorded; Contract F11 has no disposition);
- row 16: integration tree correct but the merge to `main` remains the release lane's step.

When rows 11/12/13-18/15/16 are discharged (or the Lead records a compliant disposition), the row table in §3b is the evidence map for signing the formal Stage 2 PASS.


— `fleet-checker`, 2026-10-31 21:02 ET
