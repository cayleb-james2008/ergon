# Team A — ERGON — Phase 1 report + first-batch evidence

Author: `ergon-lead` (Team A lead, jobs 01–05). Written to disk 2026-10-31, ~18:20–18:35 ET.
Host: `outis-desktop` (Linux 7.2.8-9-omarchy x86_64, user `cayleb`). Working dir: `/home/cayleb/Documents/deepseek-harness/default-workspace`.
Batch: A1–A5 of `board/TEAM-ASSIGNMENTS.md` — read-only/local only. **No API spend, no credential handling, no branch writes, no merge, no push.** Everything below was verified by running the command quoted; each command carries its result and the time it was run.

**Status in one line:** batch 1 complete; A1, A2, A4, A5 are settled with evidence; A3's suite runs are done — **0 failing cases on both Pack 20 and main**, with one baseline old-path check that cannot complete on this host (so exit 1 on both, not branch-attributable); the batch surfaces three real programme questions (Ergon-on-DSH package has no owner; rehearsal host; case-count/budget conflicts) for the Lead to batch.

**Spend:** $0.00 of the ~$30 Console credit + ≤20 % weekly Claude envelope. No credential was read, printed, copied or committed. DeepSeek R1 was not used anywhere; no Ergon work was routed to it.

---

## 1. Phase 1 — restatement, assumptions, conflicts

### 1.1 The jobs in my own words (Team A owns all of Ergon)

- **Job 01 — finish Pack 20 (demo company + simulated week).** Take branch `duet/p20-demo-company` (target commit `709bf0e`) from its WIP state to a checker-clean Pack 20: (1) rewrite the simulated-week scripts so they run **in the cast voices** with the company's open-source stack artefacts and the seven interaction chains carried through the week (a Monday distortion must still be in Thursday's record); (2) add **chain support to the dry-run tooling** so a chain executes and is scored at its end, not hop by hop; (3) write design section **X-1…X-4**, the template-hardening loop over many fictional companies, with the honesty rule that live-driven and state-advanced stretches are always distinguished; (4) run the full suite plus an **independent fresh-context check** and fix what it finds. Done-bar: all four items, suite green on the final tree, checker PASS, and at least one genuine recursive self-improvement instance through the promotion gate *or* a written finding at the top of the return explaining why none occurred.
- **Job 02 — Pack 21 fixes + template-hardening scale-up.** Blocked until Pack 19 and Pack 20 are merged. Then build the ruled Pack 21 (E-2 phone-plane isolation, E-6 lock identity, E-1 review-store retention, S-2/S-4 riders, F-2/F-4/F-5 include-or-defer with written reasons; F-1 deliberately out of scope), the scale-up pack to Job 01's X-1…X-4 design, and two residuals (`scripts/check.mjs` retire-vs-rescope; migration-gate doc sentence + landing call). Merge only on a fresh-context checker PASS plus a merged-tree proof with 0 branch-attributable failures.
- **Job 03 — driven dress rehearsal + RHL deployment prep.** Drive Cayleb's actual RHL setup end-to-end from the non-technical operator's view (fresh host → company setup → first personalised agents → shareable non-localhost URL from a second device → phone enrolment → desk task → voice/message flow), fix findings and re-run until clean; fold the warehouse-board source from the work PC into the package and wire it to Supabase (`ergon_warehouse`); write the plain-language first-employee enrolment runbook and a named slippage list for w/c 2 Nov. The board fold-in waits on a work-PC window; never rebuild the board from the older repo copy.
- **Job 04 — laptop lane Monday resume pack.** From `laptop/acceptance-battery` (`62a277e`) and its CHECKPOINT.md: close the checkpoint remainder (independent review, docs-truth re-run, stub-heading tidy, consolidated report, shared-tools refresh), add the mobile-emulator/cross-device extension, and build/check the .NET cross-items (SC-5 C# hunk; FP-5 SetupWindow.cs / ProviderCore.cs / ProviderConfigBridge.cs). Lane dark until the Claude usage reset, Mon 2 Nov 01:00 ET. Checkpoint file wins on facts.
- **Job 05 — Kimi redo handback audit.** Standing instruction until the handback exists: check for the branch, and if absent report exactly "not arrived" and stop. On arrival: verify remote branch/commit before reading the self-report, then the doctrine alignment audit (three-piece package; Haiku 5.5; R1 banned; hosted model; agent home on the cloud host), force the deferred set to be named, and give adopt / adopt-with-fixes / reject with the merge route.

### 1.2 Assumptions I am working under (flag if any is wrong)

1. **`duet/p20-demo-company` remains Job 01's target and the brief's `709bf0e` is the correct tip.** Verified on the remote at batch time; no newer P20 commit exists.
2. **"Suite green" is the repo's own `node scripts/test-all.mjs` exit 0** with no *branch-attributable* failure; environment-only failures must be named and dispositioned by the fresh checker, never silently counted as passes.
3. **The Ergon build seat named "Nox side" is not reachable from this session** (proved in A2). Team A works in fresh worktrees of the reachable clone; final acceptance/merge still runs under the checker's own environment.
4. **Pack 19 is merged to `main`** (proved in A2); Pack 20's branch predates that merge, so Job 01's final state must absorb main before its own merge.
5. **No live-model spend is needed for the suite itself** (all model paths in the suite are fakes/stand-ins; proved by the 0-key run). Live-model runs are a Monday-after-reset question (Q4).
6. **The work PC (`RHLCO-WKS-09`) window is a machine/scheduling dependency, not a Cayleb act**; until the window there is no board fold-in.
7. **Job 05's outcome is "not arrived"** until a branch/PR exists on the remote; nothing else about the handback counts as delivery.

### 1.3 Conflicts, gaps and things that cannot be verified (each goes to the Lead; C1–C3 are questions in §4)

- **C1 — "1,577-case suite" in the brief is not evidenced anywhere I could run.** Measured today: **1,558 cases** on `709bf0e` and **1,603 cases** on `main` (`b59b599`); the newest *recorded* proof in the repo is **1,451 cases** on main `6185fc9` (`docs/acceptance/GROUNDWORK-REPORT-2026-10-31.md:109`). No tree I can reach produces 1,577. Either the figure is stale from another tree or a different counting basis; use the measured numbers until ruled.
- **C2 — Pack 20's branch is 27 commits behind `main` and does not contain the Pack 19 merge.** `gh api compare main...duet/p20-demo-company` → `{"status":"diverged","ahead_by":15,"behind_by":27}`; merge-base `482fbc8`. The branch is therefore **not mergeable as-is under Job 02's "19 + 20 merged" precondition** — main must be merged into the branch first. Dry run says that merge is **conflict-free** (§2.1).
- **C3 — "the fleet build's output" (Job 03's ruled product package) is not an Ergon package today.** What exists is `@cayleb/dsh-fleet` 0.2.1 (a DSH bundle) plus the DSH 0.2.2-alpha.2 install; the Ergon-on-DSH three-piece package (management software + DSH baseline + Ergon plugin) is **not assembled anywhere I can reach**. The `ergon` repo's `plugins/*` are OpenClaw-era packages (`"openclaw": {...}`), and no `ergon-dsh-plugin` / `@ergon/dsh-*` package exists on this desktop. No job in the 19-job programme builds the Ergon-on-DSH package. Job 03's rehearsal will hit this at the S3/S4 step the brief already names — needs a ruling (§4 Q1).
- **C4 — budget wording conflict for Job 03.** Job 03 brief: "running-cost budget at most $100/month per company"; doctrine v2 (`docs/nox-governing/ergon-current-state-brief-v2-2026-10-30.md:43`): "hosting ≤ $100/mo per company; API credits … planning anchor ~$2,000/mo API credits per company". As written, a full Ergon company cannot run under $100/mo all-in. Ask which envelope governs (§4 Q3).
- **C5 — `.NET` toolchain claim is only half true on this desktop.** `/usr/bin/dotnet` exists and the host reports `.NET Host 10.0.13`, but **no SDKs are installed** (`dotnet --list-sdks` → empty) and there is no Mono/csc/NSIS (`makensis`) either. Job 04's .NET cross-items still need the laptop/toolchain; the GROUND-STATE line "this desktop HAS /usr/bin/dotnet" is true only as a host, not as a build seat. Recorded, not a blocker tonight.
- **C6 — work PC has no route from this desktop.** `tailscale status` currently lists 6 nodes (`outis-desktop`, `cayleb-desk` offline 253d, `muse`, `nothing-phone-3a`, `outis` offline 151d, `srv1360751` offline 253d) — **no `RHLCO-WKS-09`**. Combined with the doctrine's SOURCE WARNING (accepted board V11 source lives only on the work PC), board fold-in is BLOCKED-ON-MACHINE, as expected; nothing to work around.
- **C7 — model policy vs workstream.** Ergon is Haiku 5.5 / R1 banned; this session's model is not Anthropic. Ergon *build* model policy applies to the agents doing Ergon work; nothing in this batch routed Ergon work to R1. For live-model test runs, the Console route needs a key that this session does not have (§2.2, Q4).
- **C8 — 93 remote branches.** The count and the "no open PRs" state from GROUND-STATE were re-confirmed; several `duet/*` branches are pre-P19 fix branches still open on the remote. They are not P20's problem; Job 18/Lead may care later.
- **C9 — the suite does not exit 0 on this desktop, and the single cause is a baseline check, not Pack 20.** With a faithful dependency tree the definitive runs give **0 failing cases on both trees** (§2.3b): Pack 20 `709bf0e` = 1,558 cases / 1,553 pass / 0 fail / 5 skipped; `main` `b59b599` = 1,603 / 1,598 / 0 / 5. Both exit 1 for the **same single reason**: `tests/opencode-e2e.test.mjs` fails to complete (60 s timeouts on the Outis-era path; `opencode v2.0.32` is installed). It appears identically on main ⇒ not branch-attributable; it is the same old-path debt Job 02 item 3(a) rules on. Job 01's done-bar requires the suite to be green, so the fresh checker must adjudicate this check (retire/rescope/fix) on a real install — the batch cannot declare the tree green by itself.

---

## 2. First-batch evidence (A1–A5)

All commands below were run from this host on 2026-10-31 between 17:48 and 18:35 ET. `$` lines are the commands; the following line(s) are the actual output.

### 2.1 A1 — remote branch, target commit, and tree vs Job-01 scope items 1–4

**Branch and commit exist on the remote (GitHub API, ground truth):**

```
$ gh api repos/cayleb-james2008/ergon/branches/duet/p20-demo-company --jq '.name+" "+.commit.sha'
duet/p20-demo-company 709bf0e62dc441d3defb1b110968706ad5937116

$ gh api repos/cayleb-james2008/ergon/commits/709bf0e --jq '.sha+" | "+.commit.author.date+" | "+(.commit.message|split("\n")[0])'
709bf0e62dc441d3defb1b110968706ad5937116 | 2026-10-31T20:01:55Z | P20-WIP: chain items in the competency battery and scorer (Addendum B in progress)
```

**The commit's own message names exactly what is still missing** (this is a *claim by the branch*, and the tree was checked against it — not accepted on trust):

```
$ git log -1 --format='%B' 709bf0e
P20-WIP: chain items in the competency battery and scorer (Addendum B in progress)

Finished in this commit: two chain items in competency/battery.json (56 items) and chain scoring in
competency/score.mjs; --calibrate gives null chain 0/2, oracle 2/2.
Partial, not in the repo: the week scripts are not yet rewritten in the cast voices and do not yet carry
the chain hops or stack artefacts (generator edits in progress in the session scratchpad); dry-run-week.mjs
has no chain.hop/chain.final handlers yet; SEED.md, prove-rerunnable.json, the dry-run evidence and the
docs predate the stack and voices; X-1..X-4 design doc not started; suite not run.
```

**Tree contents vs the four scope items — checked file by file, not from the message:**

| Scope item | Verdict | Evidence |
|---|---|---|
| 1. Week scripts in cast voices + stack artefacts + chains through the week | **OPEN** | The five day fixtures exist and are real: `fixtures/week/{mon,tue,wed,thu,fri}.json` (mon has 16 acts; 65 files / 11,210 insertions on the branch). But every act's key set is `id, at, actor, device, kind, inputs, expect, standIn, packagePending, binding, modelCalls`; `grep '"[a-zA-Z]*[Vv]oice[a-zA-Z]*"' fixtures/week/*.json` → **0 matches**; `grep '"chain[A-Za-z]*"' fixtures/week/*.json` → **0 matches**. No voice, no stack-artefact, no chain-hop fields. |
| 2. Chain support in the dry-run tooling | **PARTIAL** | Battery/scorer half is in: `fixtures/competency/battery.json` → 56 items, including `marcus-oyelaran:c1` and `priya-lindqvist:c1`; scorer calibration per the commit message `null chain 0/2, oracle 2/2`. Tooling half missing: `grep -c chain docs/acceptance/demo-company/fixtures/dry-run-week.mjs` → **0**. |
| 3. X-1…X-4 design (template-hardening loop) | **OPEN** | No design file in `docs/acceptance/demo-company/` (11 docs: CAST, COMPANY, COMPETENCY, README, RUN-REPORT-TEMPLATE, SCENARIOS, SEED, SELF-IMPROVEMENT, WEEK + evidence/fixtures). A whole-tree `grep -rIl 'X-1' docs/` gives **3 files, all `docs/review-debt/*`** (unrelated review-debt notes), not the design. |
| 4. Full suite + independent check | **OPEN / NOT RUN** | Commit message: "suite not run". No fresh-context checker report for Pack 20 exists on the branch or on disk. My own run (§2.3) is *evidence for* a checker, not the checker. |

**Branch relation to main (matters for Job 01's completion and Job 02's precondition):**

```
$ gh api 'repos/cayleb-james2008/ergon/compare/main...duet%2Fp20-demo-company' --jq '{status,ahead_by,behind_by}'
{"ahead_by":15,"behind_by":27,"status":"diverged","total_commits":15}

$ git merge-base origin/duet/p20-demo-company origin/main
482fbc84d69ee9f1f91005c4f9d04b7cf2637d28      # the README's "main at the time of writing"

$ git merge-tree --write-tree origin/main origin/duet/p20-demo-company
3117a2081f2ddb59a59a7819aef937c8e3fa4f90      # exit 0: a real merge is conflict-free
```

So Pack 20's 15 commits sit on a base 27 commits older than main; merging main into the branch is mechanical (clean dry run), and must happen before the Job 01 checker gate.

**What the branch does contain (so the checker knows the starting point):** P20-1 company (`COMPANY.md`, `fixtures/data/company.json`), P20-2 cast of six with devices/profiles/onboarding (`CAST.md`, `cast.json`), P20-3 scenarios (a)–(f) + seven interaction chains (`SCENARIOS.md`, `fixtures/chains/chains.json`), P20-4 seeded six-week backlog and re-runnable setup/teardown, P20-5 finding probes with captured output, P20-6 five day scripts + wall-clock estimate + stand-in dry run evidence, P20-7 zero-to-competent baseline/battery/scorer, P20-8 self-improvement verdict (**"current code cannot produce a genuine instance"**, with the missing steps named), P20-9 `packagePending` per act, and Addendum-2 run config. The stand-in dry-run summary is labelled `"STAND-IN OUTPUT (deterministic local stand-in; not model capability)"` with `modelsUsed.* = "stand-in (no model was called)"` — i.e. the honesty rule is already being applied on the branch.

### 2.2 A2 — the Ergon build seat, Pack 19 on main, and the $30 Console route

**Pack 19 IS merged to `main` — confirmed from the remote, with the merge commits quoted:**

```
$ gh api 'repos/cayleb-james2008/ergon/commits?sha=main&per_page=10' --jq '.[] | .sha[0:10]+" | "+.commit.author.date+" | "+(.commit.message|split("\n")[0])'
b59b5990c5 | 2026-10-31T20:17:39Z | Integration (Pack 19 on Pack 18): name the entry position in the suite token-map key-shape refusal
7990341f07 | 2026-10-31T20:11:23Z | Merge duet/p19-4-bridge-gate (Pack 19, checker PASS 31 Oct)
4e9a8e0451 | 2026-10-31T20:11:23Z | Merge duet/p19-5-provisioning (Pack 19, checker PASS 31 Oct)
23f3c891ce | 2026-10-31T20:11:23Z | Merge duet/p19-3-refusal-settle (Pack 19, checker PASS 31 Oct)
761dde5946 | 2026-10-31T20:11:23Z | Merge duet/p19-2-owner-mismatch (Pack 19, checker PASS 31 Oct)
d0c81f89ac | 2026-10-31T20:11:23Z | Merge duet/p19-1-token-reload (Pack 19, checker PASS 31 Oct)
750ae0777a | 2026-10-31T19:51:12Z | P19-5: provision the suite token map from the admin script and pin the support proxy origin (S-4)
```

Remote `main` tip = `b59b5990c594bfe7dfda859fcac3bff09ef609d3` (`git ls-remote`), i.e. Pack 19's five legs + integration are on main. **The old brief wording ("waiting on Pack 19") is satisfied on `main`; it is *not* satisfied on the P20 branch, which predates the merge (C2).**

**Which clone/machine is the working seat — established by probing, not assumed:**

- Reachable clone: `/home/cayleb/workspace/harness-build/ref/ergon-team`, `origin = https://github.com/cayleb-james2008/ergon.git`, branch `team` @ `91d1ea9` — **a board-sync clone, not a build seat**: it has *no* `node_modules`, no `support/server/node_modules`, no `.document-runtime`, and does not contain `709bf0e` until fetched. Two more ergon clones exist (`~/Work/shared-workflows/lanes/board`, `~/Work/shared-workflows/money-desk/board`) but both are **Nox lane board clones on `team`** — read-only for us, not build seats.
- The "Nox-side" seat: **UNREACHABLE/UNVERIFIED from this session** — `/home/nox` is permission-denied, `sudo -n true` → "a password is required", no SSH route to any other machine (GROUND-STATE: no `~/.ssh` keys/config), and the tailnet has no other online host.
- Working seat **proved possible here**: fetch + fresh worktree + matching dependency scaffolding, then the full suite runs (§2.3). Offline dependency install is **not** possible: `npm ci --offline` fails `ENOTCACHED` for npm 12.1.0 itself; the only usable trees on disk are existing installs from an identical `package-lock.json` (md5 `094f3d4e7735bc929c70a3786b029ab0`) with an identical document runtime (11/11 package versions equal). This matters for the checker: **the canonical seat must have a real `npm ci` tree; the numbers below were produced on a reconstructed one.**
- Spend route: **NO usable Console API route from this session without credential handling** (and none is attempted). `env` shows **0** `ANTHROPIC*` credential variables (names inspected, values never read); no `~/.config/deepseek-harness` in this account; `claude` CLI exists but is the subscription route whose meter is ~90 % used until Mon 01:00 ET. The suite itself needs no key (it passed 1,549+ cases with no key and its model paths are fakes/stand-ins; the stand-in summary explicitly says "no model was called").

**Answer to the A2 question:** build seat = *this desktop in a fresh worktree, for local/mock work only*; Nox-side seat UNVERIFIED; Pack 19 merged to main **YES**; `$30` Console route usable tonight **NO** (no key present; using it would need Cayleb's write-only paste — his act) and it would anyway be outside tonight's "local/mock/offline only" envelope. See Q4.

### 2.3 A3 — what the 1,577/1,558/1,603-case suite needs, and what actually ran

**Inventory (run, not read):**

```
$ node -e "const m=require('./scripts/test-manifest.json'); ..."    # manifest
invocations: 163   kinds: { node: 152, python: 5, dotnet: 5, windows: 1 }

$ node scripts/test-all.mjs --list
...
MANIFEST matches the test files on disk        # exit 0
```

Requirements: **node ≥ 24.16** (this host: v26.7.0; a `/opt/node24` the demo-company README mentions does **not** exist here), Python 3.14 + the repo's `.document-runtime` for 5 python invocations (system `python3` 3.14.8 present; an existing runtime with the **exact** lock versions — python-docx 1.2.0, reportlab 5.0.1, pypdfium2 5.14.0, etc. — was found and linked), the repo's `node_modules` + `support/server/node_modules` (absent in every reachable clone; reconstructed from an identical-lock install), and separate .NET/Windows seats for 6 manifest invocations (**not run here, never counted as passing**). **No API spend and no network beyond localhost are needed**: every model path is a fake/local stand-in; the run made **no model calls and cost $0**.

**What ran — run #1, P20 commit `709bf0e`, fresh worktree `/tmp/ergon-p20-709bf0e`, dependency tree symlinked (`node_modules` as symlinked children):**

```
$ node scripts/test-all.mjs
TOTAL 1558 cases: 1552 pass, 1 fail, 0 cancelled, 5 skipped, 0 todo; 157/157 checks ran, 1 without a
complete report; 4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here
(5 need .NET, 1 needs Windows); 0 manifest problems
EXITCODE=1
```

The one failure was `installer/test-payload.py` — and it was **my scaffolding**, not the branch: `stager.staging_outside_inputs()` resolves the path, and a symlinked `node_modules` made `/tmp/ergon-p20-709bf0e/node_modules/staging` resolve outside the checkout, so the expected refusal did not fire. Re-running that test with a real directory fixed it: `Ran 46 tests ... OK`. This is why the numbers needed a second, faithful run (§2.3b) — a tree that lies about its install layout can manufacture failures.

**Run #2 — baseline comparison, same scaffolding, main `b59b599`:**

```
TOTAL 1603 cases: 1594 pass, 4 fail, 0 cancelled, 5 skipped, 0 todo; 157/157 checks ran, ...
```

P20 and main under this same scaffolding share the **same four problem checks** (so none is branch-attributable); three of them vanish with a real dependency tree (§2.3b), confirming they were install-tree artifacts:

- `tests/hindsight-memory-config.test.mjs` — `not ok 3 - OpenClaw 2026.10.1 plugin-aware validator accepts fresh and migrated enrolled Hindsight configs`
- `tests/overlay.test.mjs` — `not ok 2 - Verified installed patches pass repeat integrity check`; `not ok 5 - Same-version changed installed contents fail closed`
- `tests/routing.test.mjs` — `not ok 6 - Actual pinned OpenClaw schema accepts Haiku-first fields and plugins`
- `tests/opencode-e2e.test.mjs` — `FAILED ... Error: spawnSync .../node ETIMEDOUT`; `not ok 3 - a queued ticket spins up an opencode session ...`; `not ok 4 - the single model verifies on screen and resolves ...` (Outis-era path; `opencode v2.0.32` is installed, the 60 s tests still time out).

**Run #3 (definitive) — P20 and main re-run with a real (copied) dependency tree, same host:** see §2.3b. The disposition rule is fixed: a failure identical on main and P20 in the same environment is **baseline/environment**, not Pack 20's branch; only a failure that appears on P20 and not on main under the same environment can block Job 01.

**Answer to A3:** the suite needs node (≥24.16; v26.7.0 here), a real repo dependency install, the document runtime, and (for 6 invocations) .NET/Windows seats; it runs with **$0 API spend** (no credential present, no model called). Exact totals: **P20 `709bf0e` 1,558 cases — 1,553 pass, 0 fail, 5 skipped, 1 incomplete check**; **main `b59b599` 1,603 cases — 1,598 pass, 0 fail, 5 skipped, 1 incomplete check**; the incomplete check is the same baseline old-path one (`opencode-e2e`), so the tree has **no branch-attributable failure** but the suite does **not exit 0** until that check is dispositioned. The "1,577" figure is not produced by any tree tested here (C1).

### 2.3b A3 definitive runs — real dependency tree (P20 vs main, same host, same lock)

Run #1's three install-tree failures and `test-payload.py` failure were scaffolding artifacts (§2.3). To remove the scaffolding variable, both worktrees were re-pointed at a **real (copied, not symlinked)** `node_modules` + `support/server/node_modules` matching `package-lock.json` md5 `094f3d4e7735bc929c70a3786b029ab0`, plus a `.document-runtime` whose 11 package versions match `documents/dependency-lock.json` exactly (python-docx 1.2.0, reportlab 5.0.1, pypdfium2 5.14.0, lxml 6.1.3, openpyxl 3.1.5, pillow 12.3.0, …). Then:

```
$ cd /tmp/ergon-p20-709bf0e && node scripts/test-all.mjs        # commit 709bf0e (Job 01 target)
TOTAL 1558 cases: 1553 pass, 0 fail, 0 cancelled, 5 skipped, 0 todo; 157/157 checks ran, 1 without a
complete report; 4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here
(5 need .NET, 1 needs Windows); 0 manifest problems
P20_EXITCODE=1                     # start 18:18:32 ET, end 18:22:55 ET

$ cd /tmp/ergon-main-b59b599 && node scripts/test-all.mjs       # origin/main b59b599 (Pack 19 merged)
TOTAL 1603 cases: 1598 pass, 0 fail, 0 cancelled, 5 skipped, 0 todo; 157/157 checks ran, 1 without a
complete report; 4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here
(5 need .NET, 1 needs Windows); 0 manifest problems
MAIN_EXITCODE=1                    # start 18:22:55 ET, end 18:27:05 ET
```

Both aggregate outcomes name the **same one** uncertain check and **zero case failures**:
`uncertainChecks: ["tests/opencode-e2e.test.mjs"]`, `cases: {"tests":1558,"pass":1553,"fail":0,"skipped":5}` (P20) and `{"tests":1603,"pass":1598,"fail":0,"skipped":5}` (main). The incomplete check's own lines:

```
$ grep 'not ok\|ETIMEDOUT' /tmp/ergon-suites-real-deps.log
not ok 3 - a queued ticket spins up an opencode session that the primary model resolves through Outis tools
not ok 4 - the single model verifies on screen and resolves (no promotion to a second model)
FAILED /tmp/ergon-p20-709bf0e/support Error: spawnSync /home/cayleb/.local/share/mise/installs/node/26.7.0/bin/node ETIMEDOUT
```

`opencode v2.0.32` is installed on this host but these two 60-second Outis-era tests do not complete; identical on main. **Interpretation (important for the checker):** with a faithful dependency tree the Pack 20 branch has **0 failing cases**; both trees exit 1 only because that one old-path check cannot complete. The Job 01 done-condition ("full suite green on the final tree") cannot be claimed as exit 0 until that check is dispositioned (fixed, retired, or rescoped — exactly Job 02 item 3(a)'s ruling) or the checker records a principled, evidenced exception. The four checks that failed earlier were the scaffolding, not the code: with the real tree they pass. The suite consumed **$0** (no credential present, no model called: the opencode/worker paths use a scripted local model and the fake Anthropic upstream).

<!-- DEFINITIVE-TOTALS -->

### 2.4 A4 — Kimi handback re-check (Job 05)

```
$ gh api 'repos/cayleb-james2008/ergon/branches?per_page=100' --paginate --jq '.[].name' | wc -l
93
$ ... | grep -i 'kimi\|redo'          # (case-insensitive, 93 branches)
(no output; 0 matches)
$ git ls-remote --heads origin | grep -i 'kimi\|redo'
(no output; 0 matches)                # server-side truth, not local refs
$ gh pr list -R cayleb-james2008/ergon --state all --search kimi
(no output)
```

**Answer:** no Kimi handback branch exists on the remote (93 branches, 0 matching `kimi`/`redo`; no PR). Job 05 remains **STANDING — "not arrived"**. Per its stop rule, that one-line report *is* the correct outcome of the run; nothing else was done and nothing was assumed.

### 2.5 A5 — Job 03's dependency map: "the fleet build's output", the RHL target, and the work-PC fold-in

**What "the fleet build's output" is today (probed on disk, with hashes):**

- `~/workspace/harness-build/dsh-fleet/` is the fleet harness repo. Its build product is the DSH **bundle** `@cayleb/dsh-fleet` in `package/`, packaged to `dist/cayleb-dsh-fleet-0.2.1.tgz` — 164,714 bytes, built 31 Oct 10:46 ET, sha256 `55516f9ae9535dc585da96587af45f55d83edac67c88e48d3bdc32ad652d10d4` (matches its own `.sha256` file). **There is no 0.2.2 tarball** — `dist/` contains only the 0.2.1 artefacts; BUILD-STATE's Stage 2 row says "its real-install run waits for S1-P1 on the real install and fix round R5", and release 0.2.2 is Job 06's unclosed work.
- The *real install* on this desktop is the fleet user's: DSH **0.2.2-alpha.2** + the `fleet` profile went in through the owner-setup wrapper (Stage 0 CLOSED 07:49 ET 31 Oct, per `dsh-fleet/docs/CLOSEOUT-0-2.md`); the installed plugin bundle is 0.2.1. The `fleet` user's `/var/lib/fleet` is 0750/fleet-owned → **not readable from `cayleb`** (permission denied); the install claim above is quoted from the repo's evidence/close-out, and labelled as such.
- **Ergon-on-DSH does not exist as a build artefact I can reach.** The `ergon` repo's `plugins/` are six OpenClaw-era packages (`employee-routing`, `rhl-connectors`, `rhl-documents`, `rhl-onboarding`, `rhl-phone`, `rhl-review`; each `package.json` declares `"openclaw": {...}`, none declares `dsh.bundle`). `grep -rn 'dsh.bundle'` across the repo → 0. The fleet design doc refers to "Cayleb's Ergon DSH work (`ergon-dsh-plugin`, `@ergon/dsh-voice`, `@ergon/dsh-shell`, review gate, library lifecycle)" as **concepts only, code does not merge** (`dsh-fleet/docs/fleet-plugin-design.md:168`); no such package directory was found on this desktop in a bounded search. The three-piece doctrine (`ergon-current-state-brief-v2:14`) says Ergon *is* management software + DSH baseline + Ergon plugin; only piece (2), DSH, is assembled today.
- Consequence for Job 03: the rehearsal's step "company setup → first personalised agents → shareable URL → phone enrolment → desk task → voice" needs the Ergon plugin/management package and the hosted-URL shape. With only the fleet bundle + DSH available, the driven pass will stop at the S3/S4 boundary the brief already names. **This is a programme gap, not a Team A execution failure** — the gap is C3 and Q1.

**The RHL deployment target for w/c 2 Nov (from the governing doctrine, quoted):**

- `docs/nox-governing/ergon-current-state-brief-v2-2026-10-30.md:41`: "**Dedicated cloud host per company:** the production shape is a single-tenant cloud host per company (standard: **Hetzner CPX22 class, ~€8/mo, US Ashburn**), never a shared multi-company service… **Fleet end-state:** once a company approves, the Ergon harness is installed on every computer there…".
- same file `:43`: "**Data:** everything on **Supabase — one Pro org of ours, one project per company** … Budgets: hosting ≤ $100/mo per company; API credits … planning anchor ~$2,000/mo API credits per company".
- same file `:75–76`: "**SOURCE WARNING:** the accepted V11 source lives on the work PC (RHLCO-WKS-09), which is OFFLINE. The repo's board copy is OLDER. Do not 'fix' or rebuild the board from the repo copy; board source edits wait for the work PC's next window." Board data: "hosted copy in the 'Ergon' Supabase project, schema `ergon_warehouse`, tables `lines`/`entries`"; the weekend orders name the project ref `yhpjpjeedpuvcgnvfpka` and the verified counts **19 lines / 12 entries**.
- Current demo hosting (doctrine §19, `ergon-weekend-mission-orders:178`): apps hosted on one machine, today the coworker's Windows PC `RHLCO-WKS-09`, used through a shareable URL — the production cloud host replaces this per the locked note above.

**What the work-PC fold-in waits on, precisely:** a scheduled window on `RHLCO-WKS-09` (offline, not on the tailnet, no SSH route — C6), during which the V11 board source is copied into the package and wired to the `Ergon` Supabase project (`ergon_warehouse`); it must **not** be rebuilt from the repo's older copy. Until then Job 03's fold-in leg is BLOCKED-ON-MACHINE; everything else in Job 03 (runbook, slippage list, dependency map, package audit) can proceed.

**Answer to A5:** fleet build's output today = `@cayleb/dsh-fleet` 0.2.1 tarball + the DSH 0.2.2-alpha.2 install; **no Ergon package exists**; RHL target = single-tenant Hetzner CPX22-class host (~€8/mo) + Supabase project-per-company (`Ergon`/`ergon_warehouse` today), hosted-URL model, budget conflict in C4; work-PC fold-in waits only on the `RHLCO-WKS-09` window and cannot be worked around.

---

## 3. Phase 2 plan — large steps, each with a checkable done-condition

Ordering follows Cayleb's standing priority ("Ergon completely done first"). Each step is tens of minutes to 1–2 h. No step runs without the Lead's go; no step crosses a hard hold. Job 01's steps assume the Lead authorises branch writes on `duet/p20-demo-company` only (job branch, no main merge, no push to main).

### Job 01 — Pack 20 (the main event; 4 items + checker)

- **S1. Seat + branch refresh (45 min).** In a durable worktree of the reachable clone, merge `origin/main` (`b59b599`) into `duet/p20-demo-company` (dry run clean, §2.1), resolve nothing (none expected), run the suite on the merged tree.
  *Done when:* `git merge-base --is-ancestor origin/main HEAD` exits 0 on the branch; the merged-tree suite run's totals are quoted; the only failures are the four baseline checks reproduced on main in the same environment (§2.3b). Branch pushed (job branch only) with the merge commit.
- **S2. Scope item 1 — week scripts in the cast voices (1.5–2 h).** Rewrite `fixtures/week/*.json` acts so every employee act carries its cast voice (register/jargon/verbosity/rushed-careful), the day's open-source stack artefact reference, and the chain hop/final reference; carry the Monday distortion so it is still in Thursday's record.
  *Done when:* a checker script lists, per day, non-zero counts for `voice`, `stack`, `chain.hop`/`chain.final` fields and at least one item created Monday with a matching Thursday reference (script + output committed as evidence). The stand-in/replay dry-run still completes and its output names models honestly.
- **S3. Scope item 2 — chain support in the dry-run tooling (1–1.5 h).** Add `chain.hop` / `chain.final` handlers and chain scoring at the END of the chain in `fixtures/dry-run-week.mjs` (+ lib), keeping the battery's 56 items and the `--calibrate` null 0/2 / oracle 2/2 behaviour.
  *Done when:* `grep -c 'chain.hop\|chain.final' fixtures/dry-run-week.mjs` > 0; one dry run reports each of the seven chains executed once with an end-of-chain verdict (not per hop); `--calibrate` still gives null 0/2, oracle 2/2.
- **S4. Scope item 3 — X-1…X-4 design (1–2 h).** Write the template-hardening design section (new doc under `docs/acceptance/demo-company/`, e.g. `TEMPLATE-HARDENING.md`): X-1 many-companies batching, X-2 long-horizon simulated runs, X-3 findings→template improvements, X-4 the honesty rule and reporting format; sparser variants only as calibrated extra tiers.
  *Done when:* the doc exists with the four sections; the run-report template in it distinguishes live-driven vs state-advanced stretches explicitly; a fresh reader can see how a batch run feeds a template change without a single live claim.
- **S5. Scope item 4 — full suite + independent checker (2 h + checker time).** Run the full suite on the final merged tree; request a fresh-context checker from the Lead (Lead owns checker spawn; I never certify my own work); fix branch-attributable findings; hand the checker the §2.3/§2.3b evidence as *input*, not as proof.
  *Done when:* final-tree suite totals quoted (with the same 4 baseline checks dispositioned in writing); a fresh checker's PASS report on Pack 20 scope exists at `board/reports/`; no branch-attributable failure remains.
- **S6. P20 self-improvement honesty leg (30 min).** The brief allows a written finding instead of a genuine recursive instance; `SELF-IMPROVEMENT.md` already argues "current code cannot produce a genuine instance" and names the missing steps (runtime caller; agent birth of a tool; friction record after Monday). Put that finding at the **top** of the Pack 20 return and have the checker test it (no fixture theatre).
  *Done when:* the return's first paragraph carries the verdict and the checker records whether the claim is honest.
- **Successor:** Job 02 (after the Lead records the checker PASS and authorises the merge), running Pack 21 + scale-up.

### Job 02 — Pack 21 + scale-up (blocked until 19+20 merged)

- **S1. Precondition proof (15 min).** Prove `main` contains Pack 19's merge commits and Job 01's Pack 20 merge; quote `git log`/`gh api`. *Done when:* both are ancestors of `origin/main` and the proof is on the board.
- **S2. Pack 21 build (1.5–2 h).** E-2 phone-plane isolation + per-request contact checks; E-6 lock identity = boot id + process start time in onboarding AND proxy.js (one design); E-1 review-store retention (90-day BLOB drop keeping rows/hashes/events + hard backstop ceiling with oldest-stale eviction + audit event); S-2 non-slug token-map key refusal + review event; S-4 support-proxy origin pinned to deployment tailnet domain via provisioning config; F-2/F-4/F-5 each included or deferred with a written reason. F-1 explicitly not touched.
  *Done when:* each ruled item has a commit and a named test that fails before / passes after (or a written deferral reason); fresh checker PASS; merged-tree proof 0 branch-attributable failures.
- **S3. Scale-up pack (2 h).** Implement what X-1…X-4 specifies: batch runs over many fictional companies (size/roles/sector) with long horizons; findings feed small template improvements; live vs state-advanced always separated.
  *Done when:* at least one full batch run is reported with the separation visible and the batch evidence committed.
- **S4. Residuals (1 h).** `scripts/check.mjs` retire-vs-rescope ruling made on rehearsal evidence and executed; migration-gate doc sentence + landing call confirmed on main (or landed).
  *Done when:* the ruling is on the board with its evidence and the executed outcome; the migration-gate items are proven present by `git grep` on main.
- **Successor:** the Pack 21 checker gate, then Job 03's deployment leg can use the merged base.

### Job 03 — dress rehearsal + RHL deployment prep (prep now; blocked legs named)

- **S1. Dependency map + rehearsal spec (1 h).** Written from §2.5: what must exist for each rehearsal step, which pieces exist today, which are missing (C3), the fresh-host requirement, and the per-step evidence layout (secrets-free).
  *Done when:* the rehearsal spec names every artefact by version/hash and every missing piece with its owner gap; a second reader can run the checklist without asking a question.
- **S2. Rehearsal passes on a fresh host (2 h/pass, 2+ passes).** Preferred: a throwaway VM/container on this desktop for passes 1–2 (no spend); final pass on the provisioned single-tenant cloud host when Cayleb authorises the €8/mo. Fresh host, no pre-seeded state, non-technical operator's view, non-localhost shareable URL, second device, phone enrolment, desk task, voice/message flow; every finding fixed and re-run.
  *Done when:* one complete clean pass is recorded step-by-step, with screenshots/URLs/commands per step and no developer shortcuts; any genuine blocker is named with evidence (not smoothed).
- **S3. Warehouse board fold-in (on the work-PC window).** Copy V11 source from `RHLCO-WKS-09` into the package, wire to Supabase `Ergon`/`ergon_warehouse`; never rebuild from repo copy.
  *Done when:* the board runs from the packaged source against the company database, with the 19-line/12-entry data verified read/write and the phone/second-device proof repeated; or the leg is explicitly named as waiting on the window.
- **S4. Deployment runbook + slippage list (1 h).** Plain-language first-employee enrolment runbook (what Cayleb clicks/enters, in order, no secrets), hosting per the ruled model, and a dated slippage list.
  *Done when:* the runbook is reviewed against the doctrine and has no secret/credential step for a teammate; every unresolved gap is dated on the slippage list.
- **Successor:** w/c 2 Nov deployment: Cayleb's enrolment steps, then the first employees' agents.

### Job 04 — laptop lane (starts Mon 2 Nov 01:00 ET)

- **S0 (now, read-only, 30 min):** read `CHECKPOINT.md` from the remote `laptop/acceptance-battery` and inventory the seven items + the .NET files needed; record the .NET-toolchain fact (C5). *Done when:* the checkpoint list is transcribed with file paths and the Monday order is on disk.
- **S1 (Mon): checkpoint remainder (2 h)** — independent review, docs-truth re-run on merged tree, stub-heading tidy, consolidated report, shared-tools refresh. *Done when:* each checkpoint item is closed with its quoted command/output and the reviewer's verdict exists.
- **S2 (Mon): mobile emulator + cross-device extension (2 h)** — emulator as the phone connection (enrolment + phone flows), real desktop↔laptop communication so Cayleb's phone is only a final spot-check. *Done when:* an emulator enrolment + phone flow passes in the battery, and a real desktop↔laptop round-trip is demonstrated.
- **S3 (Mon): .NET cross-items (1.5 h)** — SC-5 C# hunk and FP-5 `SetupWindow.cs`, `ProviderCore.cs`, `ProviderConfigBridge.cs` built and checked on the seat that has the toolchain. *Done when:* the build/check output is quoted and a fresh checker reviews the diff.
- **Successor:** Job 03's deployment prep consumes the acceptance evidence; no idle lane on Monday.

### Job 05 — Kimi audit: STANDING, next check at the next cycle or on the Lead's word

- **S1.** Re-run the one-line branch check each cycle (already verified absent, §2.4). *Done when:* branch check re-run and "not arrived" re-recorded; stop.
- **S2 (on arrival, 1.5 h):** verify remote branch/commit before reading the self-report; then doctrine alignment audit (three-piece, Haiku 5.5, R1 banned, hosted model, agent home on cloud host), force the deferred set to be named, verdict + merge route. *Done when:* the audit quotes remote state and actual test runs, every silently-missing commission item is a finding, and the verdict is on the board.

---

## 4. Questions the Lead must batch to Cayleb (with my recommended answer)

Cayleb gets these batched by the Lead, one short question each, never directly from me. Recommendations are mine; the Lead decides what to send.

- **Q1 — Who builds the Ergon-on-DSH package that Job 03 must rehearse?** Today only the fleet DSH bundle exists (0.2.1) plus the DSH baseline; the three-piece Ergon package (management software + DSH baseline + Ergon plugin) has no owner in the 19-job programme and the Kimi handback is still absent. *Options:* (a) proceed with the rehearsal on the pieces that exist, name the missing package as a genuine blocker and stop at the S3/S4 boundary — **recommended**; (b) assign a new packaging build (needs an owner and an envelope); (c) wait for the Kimi handback. *If Cayleb picks nothing:* Job 03 proceeds to the S3/S4 boundary and reports it as blocked, nothing is invented.
- **Q2 — Rehearsal host.** The production shape is a single-tenant cloud host (~€8/mo Hetzner CPX22), which is a purchase/account act of Cayleb's and not tonight's spend. May the rehearsal's first passes use a throwaway VM/container on this desktop (fresh host, no spend), with the final pass on the cloud host when he authorises it? *Recommended:* yes. *If nothing:* Job 03 cannot start the driven pass at all; it writes the spec and waits.
- **Q3 — Job 03 budget conflict.** The job says running cost ≤ $100/month per company; the governing doctrine says hosting ≤ $100/mo **plus** API credits with a ~$2,000/mo planning anchor per company. Which envelope governs the w/c 2 Nov plan — $100 all-in (then API usage must be capped hard) or $100 hosting + a separately set API cap? *Recommended:* $100 hosting + API metered separately with his cap (the doctrine's reading). *If nothing:* the stricter reading applies (hosting $100 + no API spend until a cap is given).
- **Q4 — Console route for Ergon live-model runs.** Tonight is local/mock only by the standing rule. From Mon 01:00 ET (usage reset), either (a) Cayleb pastes the Anthropic Console key at the Ergon seat into the write-only field (his act, ~1 min) so Haiku 5.5 runs can be metered against the ~$30 credit, or (b) Ergon stays local/stand-in-only and no live-model result is claimed. *Recommended:* (a) Monday, not tonight. *If nothing:* (b) stands; no live-model evidence will be produced.
- **Q5 — P20 branch refresh order (process, low risk).** Merge `origin/main` into `duet/p20-demo-company` (clean dry run) before the checker gate, rather than rebasing? *Recommended:* merge (preserves history, no force-push, one writer). *If nothing:* I do not touch the branch; Pack 20 pauses at S1.
- **Q6 — Case-count conflict.** The brief says "1,577-case suite"; measured today: 1,558 on P20, 1,603 on main, newest recorded proof 1,451. Is there a newer proof elsewhere that should be the baseline, or do we use the measured numbers? *Recommended:* measured numbers, with the tally named in every report. *If nothing:* measured numbers stand.
- **Q7 — .NET for Job 04.** No SDK/csc/Mono is installed on this desktop (C5); the laptop remains the .NET seat per the brief. Confirm the laptop window Monday is expected and that Job 04's .NET items are not to be attempted here. *Recommended:* yes, Monday on the laptop. *If nothing:* the .NET items stay open, not failed.

---

## 5. Status, spend, blockers, next step, successor

| Job | Status after batch 1 | Blocked on | Next step |
|---|---|---|---|
| 01 Pack 20 | **READY TO EXECUTE** (scope verified open; suite evidence §2.1/§2.3b: 0 case failures on P20 and main) | Lead's go + branch-write authorisation; checker spawn (Lead) | S1 branch refresh → S2–S4 build → S5 checker |
| 02 Pack 21 + scale-up | **BLOCKED-ON-DEP** (19 merged; 20 not merged; X-1…X-4 not written) | Job 01 completion + Lead-authorised merge | starts after Job 01 gate |
| 03 Rehearsal + deploy prep | **PREP POSSIBLE / BLOCKED-ON-MACHINE** for fold-in; package gap C3 | Q1, Q2, work-PC window | S1 dependency map + spec now |
| 04 Laptop pack | **BLOCKED-ON-DEP** (lane dark) | Mon 2 Nov 01:00 ET reset + laptop | S0 checkpoint read now, S1–S3 Monday |
| 05 Kimi audit | **STANDING — not arrived** | handback arrival | one-line re-check per cycle |

**Spend:** $0 used / ~$30 Console + ≤20 % weekly Claude. **Credentials:** none read, printed, copied, logged or committed; no secret value appears in this report. **DeepSeek R1:** not used anywhere. **Writes made by this batch:** only this report (`board/reports/team-a-phase1.md`) — no repo writes, no branch writes, no push, no merge; the worktrees and dependency scaffolding live under `/tmp` and are disposable.

**What is blocked on the Lead:** (a) the go for Job 01 with branch-write authorisation and a fresh-context checker seat; (b) Q1/Q2 answered (package ownership, rehearsal host) — they gate Job 03's execution; (c) the §4 questions batched to Cayleb; (d) a decision on the canonical build seat for final acceptance (Nox-side unreachable; this desktop's reconstructed dependency tree is not a substitute for a real `npm ci` seat in the checker's environment).

**Next step + successor:** on "go", run Job 01 S1 (branch refresh + merged-tree suite) and proceed S2→S5; successor is Job 02, gated on the checkers' PASS and the Lead-authorised merge. If the Lead does not answer, nothing else in Team A can advance past documentation tonight, and that is the honest quiet result of this batch.

*Report written 2026-10-31 18:18–18:31 ET; all commands in §2 were run 17:48–18:27 ET on `outis-desktop`. Evidence logs kept at `/tmp/ergon-p20-suite.log`, `/tmp/ergon-suites-compare.log`, `/tmp/ergon-suites-real-deps.log` (disposable scratch; quotes above are the durable record).*
