# Ergon checker — independent baseline (V1–V4)

**Checker:** `ergon-checker` (fresh context; I built none of the work I checked). **Reports to:** `lead`. I hold **veto** on any merge of Ergon work to `main`.
**Batch:** V1–V4 of my charter (read-only/local; no branch, merge or push writes; zero spend; no credential handling; DeepSeek R1 not used).
**Host:** `outis-desktop` (Linux, user `cayleb`). **Written:** 2026-10-31 ~19:40 ET.
**Writes made:** this report + `board/probes/ergon-checker-baseline-runs.txt` (raw run extracts) + `board/probes/ergon-checker-gate-checks.mjs` (my gate scope checker). No repo, branch or worktree outside my own throwaway `/tmp/ergon-checker/` was modified.

**One-line status:** V1 **confirmed** (with one recorded state change: the remote P20 branch moved from `709bf0e` to the S1 merge `75948eb` at 18:34:49 ET, *after* Team A's report was written); V2 **confirmed exactly** on both trees in the environment Team A used; V3 ruled an **environment gap** (opencode v2.0.32 on `PATH` vs the repo-documented v1.19.0; the check has no version guard) — **not branch-attributable, not a Pack 20 defect, not Job 02 item 3(a)**; in the repo's **documented baseline environment** the full suite is **exit 0, 0 fail, 0 incomplete on P20, on `main` and on the current S1 tip**; V4 gives the exact Pack 20 PASS requirements and the commands I will run at the gate.

---

## 1. Verification table (what I checked, and the verdict)

| # | Fact / claim (source) | My independent result | Verdict |
|---|---|---|---|
| V1a-i | remote `duet/p20-demo-company` = `709bf0e` (Team A A1) | **True when their report was written.** `709bf0e` is authored 2026-10-31T20:01:55Z and their report says it was written 18:18–18:31 ET (22:18–22:31Z). At my probe 18:37:42 ET the remote tip is **`75948eb5b195f26ae01ab3113ef058a710c0ec3b`**, a merge commit with parents `[709bf0e, b59b599]`, authored **18:34:49 ET**, message "S1: merge main (Pack 19 on Pack 18, b59b599) into duet/p20-demo-company before Pack 20 completion". `709bf0e` is still an ancestor of the branch and is the unchanged P20-WIP tree. | **CONFIRMED at report time; branch has since advanced** (S1 merge pushed 3 min after their report) |
| V1a-ii | the `709bf0e` tree really LACKS Job-01 scope items 1–3 (no voice/stack/chain fields in `fixtures/week/*.json`, 0 chain handlers in `dry-run-week.mjs`, no X-1..X-4 design) | Confirmed by my own checker on **both** `709bf0e` and the current tip `75948eb`: 88 acts across 5 days (mon 16, tue 13, wed 17, thu 20, fri 22); **0** schema fields whose name contains `voice`, `chain`, or `stack`; **0** occurrences of `chain.hop`/`chain.final` in the dry-run tooling; **no** doc containing X-1..X-4 (the only `X-1` hits in `docs/` are 3 unrelated `docs/review-debt/*` notes). The branch DOES contain 7 chains (`fixtures/chains/chains.json`), 56 battery items, 2 chain items, and calibration null chain 0/2 / oracle 2/2. | **CONFIRMED** |
| V1b | `main` contains the Pack 19 merge commits, tip `b59b599` (Team A A2) | Confirmed from the remote and my own clone: `origin/main` = `b59b5990c594bfe7dfda859fcac3bff09ef609d3`; all five P19 merge commits (`d0c81f8`, `761dde5`, `23f3c89`, `4e9a8e0`, `7990341`) and the integration commit are ancestors of `main` (`git merge-base --is-ancestor` exit 0 for each). | **CONFIRMED** |
| V2a | P20 `709bf0e` = **1,558 cases, 0 fail**, exit 1 only because `opencode-e2e` does not complete | Reproduced **exactly** in the same environment Team A used (opencode v2.0.32 on `PATH`, `OPENCODE_BIN` unset): `TOTAL 1558 cases: 1553 pass, 0 fail, 0 cancelled, 5 skipped, 0 todo; 157/157 checks ran, 1 without a complete report`; **EXIT=1**; the incomplete check is `support/tests/opencode-e2e.test.mjs` (runner prints it as `tests/opencode-e2e.test.mjs`, cwd `support`), with failing cases `not ok 3` and `not ok 4` named below. | **CONFIRMED** |
| V2b | `main` `b59b599` = **1,603 cases, 0 fail**, exit 1 for the same single cause | Reproduced **exactly**: `TOTAL 1603 cases: 1598 pass, 0 fail, 0 cancelled, 5 skipped, 0 todo; 157/157 checks ran, 1 without a complete report`; **EXIT=1**; same incomplete check, same two failing case names, ~60.15 s each (60157/60148 ms; P20: 60160/60148 ms). | **CONFIRMED** |
| V2c | implied by Team A: "the suite cannot exit 0 until that check is dispositioned" (C9) | **REFUTED.** In the repo's **documented baseline environment** (no `opencode` binary on `PATH`, `OPENCODE_BIN` unset) both trees are **fully green**: P20 `TOTAL 1564 cases: 1555 pass, 0 fail, 9 skipped, 0 without a complete report`, **EXIT=0**; `main` `TOTAL 1609 cases: 1600 pass, 0 fail, 9 skipped, 0 without a complete report`, **EXIT=0**. The current S1 tip `75948eb` is also **1,609 cases / 1,600 pass / 0 fail / 9 skipped / 0 incomplete, EXIT=0**. No code change and no disposition is needed to reach a green suite; the environment is. | **REFUTED (with evidence)** |
| V3 | the incomplete check is "the same old-path debt Job 02 item 3(a) rules on" (Team A C9) | **Partly right, mis-attributed.** It is old-path debt, but Job 02 item 3(a) is the standalone `scripts/check.mjs` / old OpenClaw CLI ruling — a different file and path. The reproduced cause is: opencode **v2.0.32** on `PATH` prints `server listening on http://127.0.0.1:PORT`, while the dispatcher waits for `/opencode server listening on (http:\/\/127\.0\.0\.1:\d+)/` → 60 s start timeout; the repo documents opencode **1.19.0** via `OPENCODE_BIN`; with that binary the e2e passes **6/6 on both trees**. Not branch-attributable (identical on both; passes identically with the documented binary). | **PARTLY CONFIRMED / corrected in §4** |

---

## 2. V1 — the two load-bearing facts (quoted)

All commands below were run by me from a **fresh clone** (`/tmp/ergon-checker/ergon`, cloned 2026-10-31T18:40:09 ET from `https://github.com/cayleb-james2008/ergon.git`). Nothing was read from Team A's worktrees or logs.

### 2.1 V1(a) — branch, commit, and the scope gaps

```
$ date -Is
2026-10-31T18:37:42-04:00
$ git ls-remote https://github.com/cayleb-james2008/ergon.git refs/heads/duet/p20-demo-company refs/heads/main
75948eb5b195f26ae01ab3113ef058a710c0ec3b	refs/heads/duet/p20-demo-company
b59b5990c594bfe7dfda859fcac3bff09ef609d3	refs/heads/main

$ gh api repos/cayleb-james2008/ergon/branches/duet/p20-demo-company --jq '.name+" "+.commit.sha'
duet/p20-demo-company 75948eb5b195f26ae01ab3113ef058a710c0ec3b

$ git cat-file -p 75948eb          # parents + message
tree 3117a2081f2ddb59a59a7819aef937c8e3fa4f90
parent 709bf0e62dc441d3defb1b110968706ad5937116
parent b59b5990c594bfe7dfda859fcac3bff09ef609d3
author Cayleb <cayleb@example.com> 1793486089 -0400     # 2026-10-31T18:34:49 ET
S1: merge main (Pack 19 on Pack 18, b59b599) into duet/p20-demo-company before Pack 20 completion

$ git rev-list --left-right --count origin/main...origin/duet/p20-demo-company
0	16                         # main fully absorbed; branch 16 ahead (15 P20 + the S1 merge)
$ gh api 'repos/cayleb-james2008/ergon/compare/main...duet%2Fp20-demo-company' --jq '{status,ahead_by,behind_by,merge_base:.merge_base_commit.sha}'
{"ahead_by":16,"behind_by":0,"merge_base":"b59b5990c594bfe7dfda859fcac3bff09ef609d3","status":"ahead"}
```

**Pre-merge state Team A described (C2) is also exactly right**, checked against the immutable commits:
`git merge-base 709bf0e b59b599` = `482fbc84d69ee9f1f91005c4f9d04b7cf2637d28`; `git rev-list --left-right --count b59b599...709bf0e` = `27	15` (27 main-only, 15 P20-only). Their predicted conflict-free merge tree `3117a208…` is exactly the tree of the real S1 merge commit.

**Tree vs Job-01 scope items** (my own checker, run on both `709bf0e` and the current tip `75948eb`; same result on both):

```
$ node /tmp/ergon-checker/checks/scope-check.mjs /tmp/ergon-checker/wt/p20-709bf0e
mon: acts=16 / tue: 13 / wed: 17 / thu: 20 / fri: 22 / TOTAL acts: 88
act key set: ["actor","at","binding","device","expect","id","inputs","kind","modelCalls","packagePending","standIn","teaches","with"]
field-name hits across week fixtures: voice=0 fields stack="stack*"=0 "chain*"=0
dry-run-week.mjs chain.hop/chain.final handler hits: 0
demo-company docs: CAST COMPANY COMPETENCY README RUN-REPORT-TEMPLATE SCENARIOS SEED SELF-IMPROVEMENT WEEK (.md)
TEMPLATE-HARDENING.md exists: false
chains count: 7 / battery items: 56
```

Team A's exact quoted greps reproduce byte-for-byte:
```
$ grep -c chain fixtures/dry-run-week.mjs                 -> 0
$ grep -c '"[a-zA-Z]*[Vv]oice[a-zA-Z]*"' fixtures/week/*.json
  fri.json:0  mon.json:0  thu.json:0  tue.json:0  wed.json:0
$ grep -o '"chain[A-Za-z.]*"' fixtures/week/*.json | wc -l  -> 0
$ grep -rIl 'X-1' docs/ -> docs/review-debt/{ergon-medium-triage,fresh-review-main-4fb9619,ORDER-B-FIX-REPORT}.md (3 files, all unrelated review-debt notes)
```
(The 8 `/voice/` substring hits my broader scan found are prose, not schema: `invoices-week-45.md`, "Voice note not understood".)

**State change the Lead must register:** Team A's report says "no repo writes, no branch writes, no push" for their batch, and that was true of the batch they describe. The remote branch nevertheless moved at **18:34:49 ET** — 3–16 minutes after their report text was written — to the S1 merge. The commit message matches Team A's planned S1 exactly, and the merge tree matches their dry run. I cannot verify from the record who authorised the push; the Lead should confirm it was the authorised S1 go. **Consequence for the gate: Team A's baseline numbers are on `709bf0e`, not on the live tip; the gate runs on the final tree, whatever its SHA.**

### 2.2 V1(b) — `main` contains Pack 19

```
$ git log --oneline -8 origin/main
b59b599 Integration (Pack 19 on Pack 18): name the entry position in the suite token-map key-shape refusal
7990341 Merge duet/p19-4-bridge-gate (Pack 19, checker PASS 31 Oct)
4e9a8e0 Merge duet/p19-5-provisioning (Pack 19, checker PASS 31 Oct)
23f3c89 Merge duet/p19-3-refusal-settle (Pack 19, checker PASS 31 Oct)
761dde5 Merge duet/p19-2-owner-mismatch (Pack 19, checker PASS 31 Oct)
d0c81f8 Merge duet/p19-1-token-reload (Pack 19, checker PASS 31 Oct)
750ae07 P19-5: provision the suite token map from the admin script and pin the support proxy origin (S-4)
9d38a9d Merge duet/p18-s-security (Pack 18, checker PASS 31 Oct)

$ for c in b59b599 7990341 4e9a8e0 23f3c89 761dde5 d0c81f8; do git merge-base --is-ancestor $c origin/main; echo "$c -> $?"; done
all -> 0 (ancestors of main)
```
**Verdict: confirmed.** Pack 19 is on `main`; it is also now on the P20 branch via S1.

---

## 3. V2 — the suite, run by me, on my own dependency tree

### 3.1 My environment (independent of Team A)

- Fresh clone → three throwaway worktrees: `wt/p20-709bf0e` (`709bf0e`), `wt/main-b59b599` (`b59b599`), `wt/s1-75948eb` (`75948eb`).
- **Real `npm ci`** in each tree (registry reachable; 347 packages, exit 0) and `npm ci` in `support/server` (`ws` 8.21.1). `package-lock.json` md5 is identical on both trees: `094f3d4e7735bc929c70a3786b029ab0`. **No symlinked scaffolding** (Team A's symlinked first run was the source of their four install-tree artifacts; I avoided it).
- **Hash-verified Python document runtime**: copied a pre-existing runtime artifact (2026-10-13 session artifact; its `lockSha256` equals `sha256(documents/dependency-lock.json)` = `2d14623f…`), excluded stale `__pycache__` so the repo's own integrity check passes, then `./.document-runtime/bin/python -I -S documents/verify-dependencies.py` → **"Completed pinned private document runtime verified."**, exit 0, in every tree.
- `node v26.7.0`; `opencode v2.0.32` on `PATH` (this is the environment variable that matters — §4).
- No Team A worktree, log, dependency tree or number was reused. All totals below are from my logs, with timestamps.

### 3.2 Exact totals (all runs on 2026-10-31 ET)

| Run | Tree | Environment | Cases | Pass | Fail | Skipped | Checks | Incomplete checks | Exit |
|---|---|---|---|---|---|---|---|---|---|
| A (as found) | P20 `709bf0e` | opencode v2.0.32 on PATH, `OPENCODE_BIN` unset | **1,558** | 1,553 | **0** | 5 | 157/157 | 1 (`support/tests/opencode-e2e.test.mjs`) | **1** |
| B (as found) | `main` `b59b599` | same | **1,603** | 1,598 | **0** | 5 | 157/157 | 1 (same) | **1** |
| C (documented baseline) | P20 `709bf0e` | **no opencode binary on PATH**, `OPENCODE_BIN` unset | **1,564** | 1,555 | **0** | 9 | 157/157 | **0** | **0** |
| D (documented baseline) | `main` `b59b599` | same | **1,609** | 1,600 | **0** | 9 | 157/157 | **0** | **0** |
| E (documented baseline) | S1 tip `75948eb` | same | **1,609** | 1,600 | **0** | 9 | 157/157 | **0** | **0** |
| F (opt-in e2e, documented) | P20 + main | `OPENCODE_BIN=<opencode 1.19.0>` | 6/6 e2e cases | 6 | 0 | 0 | 1/1 file | 0 | **0** |

Timings: A end 18:58:58, wall 4m24.8s; B end 19:05:55, wall 4m12.5s; C/D ~1m30s each (no 180 s timeout burn); E 19:31:32–19:33:00. Raw quotes in `board/probes/ergon-checker-baseline-runs.txt`.

**Failing / incomplete test names (identical on A and B):**
```
not ok 3 - a queued ticket spins up an opencode session that the primary model resolves through Outis tools
not ok 4 - the single model verifies on screen and resolves (no promotion to a second model)
TEST_OUTCOME {"check":"tests/opencode-e2e.test.mjs","cwd":".../support","testExecution":"failed",
 "error":"ETIMEDOUT","signal":"SIGKILL"}
```
The 9 skips in runs C–E are the same five environment skips Team A reported (netfx-compile, nsis-compile, outis-dpapi-roundtrip, remote-service-safety, ui-screens) **plus the 4 opencode-e2e cases that need the binary**, each with its own named reason ("opencode binary not found (install opencode-ai and set OPENCODE_BIN)"). The other 2 cases in that file run and pass. 6 manifest invocations are not runnable on this host (5 need .NET, 1 needs Windows) and are never counted as passing.

**What this confirms and refutes:** Team A's 1,558/1,603 and "0 failing cases on both" are **exactly reproducible** — their numbers are honest. Their statement that both trees "exit 1 solely because the opencode check cannot complete" is also correct **for that environment**. What they did not find is that the same suite is **exit 0 on both trees in the repo's documented baseline environment** — the incomplete check is an environment artifact, not a tree property (§4).

**On the "1,577-case" figure (Team A C1):** no environment I can construct produces 1,577. Measured: 1,564 (P20, documented) / 1,609 (`main` and S1, documented); 1,558 / 1,603 as-found. The repo's newest recorded proof is 1,451 cases **on a Windows laptop with 87 failures** (`docs/acceptance/order-e/evidence/suite-main-6185fc9-windows-laptop.txt`) — not a green Linux baseline and not comparable. Use the measured numbers, with the environment named.

---

## 4. V3 — ruling on the single incomplete check

### 4.1 What actually happens (reproduced, not inferred)

`support/tests/opencode-e2e.test.mjs` runs `opencode serve` through `spawnOpencode` (`support/worker/opencode-dispatcher.mjs`) and waits for the server to announce itself. On this host the binary on `PATH` is **opencode v2.0.32** (`~/.local/bin/opencode` → `~/.opencode/bin/opencode`). Started by hand with the same scrubbed environment:

```
$ env -i PATH=... HOME=$T/home XDG_CONFIG_HOME=... XDG_DATA_HOME=... XDG_STATE_HOME=... XDG_CACHE_HOME=... \
    OPENCODE_CONFIG_CONTENT='{"$schema":"https://opencode.ai/config.json","autoupdate":false,"share":"disabled"}' \
    OPENCODE_SERVER_PASSWORD=testpw ... timeout 25 opencode serve --hostname=127.0.0.1 --port=0
server listening on http://127.0.0.1:45103        # v2.0.32 banner
```
The dispatcher waits for `/opencode server listening on (http:\/\/127\.0\.0\.1:\d+)/` (`support/worker/opencode-dispatcher.mjs:191` on `709bf0e`, `:248` on `main`) — the **v1** banner. The match never fires → `spawnOpencode` hits its **60 s** `opencode_start_timeout` → the dispatcher run fails → the test's `assert.ok(offered.size >= 20, 'all bridge tools are offered')` fails (both failing cases, ~60.16 s each). The node test process then does not exit (an opencode child survives), so `scripts/test-all.mjs` kills the check at its **180 s** `spawnSync` timeout → no TAP footer → "1 without a complete report" → suite exit 1. This is why the durations are 60 s per case and the whole suite run takes ~4.5 min instead of ~1.5 min.

### 4.2 It is not a Pack 20 defect — the check passes with the documented binary

The repo itself documents the required binary and the opt-in command (`docs/DEVELOPMENT-AND-TESTING.md:28`):
> "The opencode end-to-end suite needs the opencode binary (`npm install opencode-ai@1.19.0`) and npm registry access for the `@ai-sdk/openai-compatible` provider package on first run: `OPENCODE_BIN=/abs/path/node_modules/opencode-linux-x64/bin/opencode node --test support/tests/opencode-e2e.test.mjs`"

With the repo-pinned **opencode 1.19.0** (already installed on this host under mise), the documented opt-in run passes **6/6 on both trees** (my runs: P20 19:13:48 ET, `main` 19:16:04–19:17:49 ET, 0 fail, exit 0). The manifest also records the intended baseline: `support/tests/opencode-e2e.test.mjs` — "needs the opencode binary (`OPENCODE_BIN`) and npm registry access; CI provides neither, so the opencode end-to-end suite does not run on CI". In the documented baseline environment the 4 binary-dependent cases skip with a named reason and the check completes.

Counter-case (why the **full suite must not** be run with `OPENCODE_BIN` exported): `OPENCODE_BIN=1.19.0 node scripts/test-all.mjs` gives `TOTAL 1564 cases: 1558 pass, 1 fail`, exit 1 — `support/tests/smoke-cli.test.mjs` subtest 2 spawns the smoke script with `PATH=/usr/bin:/bin` to prove "no opencode binary", but inherits `OPENCODE_BIN`, so the script finds the binary and fails on the missing API key instead. That is a **test-hermeticity bug** (the test does not unset `OPENCODE_BIN`), not a Pack 20 defect; it only bites the undocumented full-suite-with-pin invocation.

### 4.3 Classification

| Question | Ruling | Evidence |
|---|---|---|
| Branch-attributable? | **NO** | Same two case names, same error, same ~60 s durations, same incomplete check on `709bf0e` and `b59b599`; passes 6/6 with the documented binary on both; P20 adds no test files (0 test files in its 65-file diff). |
| Environment gap? | **YES — primary cause** | Host `PATH` resolves opencode **2.0.32**; repo documents **1.19.0** via `OPENCODE_BIN`; `findOpencode()` accepts any `opencode` on `PATH` with no version check, so an unsupported major turns a documented skip into a hard failure. |
| Stale debt? | **YES — tail, not cause** | The Outis opencode v1 support-session path is documented as superseded: `docs/consolidation/PR5-AUDIT.md` Outcome — "Superseded, nothing ported, patch preserved. … Ergon's support sessions move to DeepSeek Harness, so this OpenCode V1→V2 port is no longer on the path." The V1→V2 port was "written from the V2 reference without a real V2 binary" (U1–U7 unverified); `origin/draft/opencode-2-port` (a0087f4) is 182 commits behind `main` and unmerged. **Do not apply it.** |
| Real defect? | **NO** | The check is green in its documented environment and green in the documented full-suite baseline; no Pack 20 code is implicated. |
| Job 02 item 3(a)? | **NO** | 3(a) is the standalone `scripts/check.mjs` retire-vs-rescope ruling (old OpenClaw CLI) — a different file and path. |

### 4.4 Recommendation: **rescope the environment, not the code; retire nothing yet**

- **For the Pack 20 gate:** no Pack 20 code change is required or accepted for this check. Run the full suite in the **documented baseline environment** (no opencode on `PATH`, `OPENCODE_BIN` unset) → exit 0, 0 fail, 0 incomplete; run the **documented opt-in e2e** with `OPENCODE_BIN=<opencode 1.19.0>` → 6/6. Record both in the gate evidence. This is a principled exception with evidence, not a redefinition of the done-bar: the suite's own documentation defines this environment.
- **Do not retire the check now:** it passes 6/6 in its documented environment and still guards the Outis dispatcher; retiring it would remove coverage before the DSH support-session migration lands.
- **Job 02 residual candidates (low, not blockers):** (i) version-guard `findOpencode()` so an unsupported major (v2+) skips with a named reason instead of failing; (ii) unset `OPENCODE_BIN` in `smoke-cli.test.mjs`'s child environment; (iii) when the DSH support-session migration lands, retire the opencode e2e together with the path (PR5-AUDIT outcome). These are for the Lead to schedule, not Pack 20 work.
- **Environment hazard to fix before the gate:** the v2 failure path leaks `opencode serve` processes. I observed **8 orphans** (~115 MB RSS each; start times 18:03:10, 18:12:01, 18:16:31, 18:21:18, 18:25:46, 18:38:42, 18:57:20, 19:04:36 ET). Two were mine (18:57:20, 19:04:36) and I killed them; the six earlier ones were left in place and are reported here. Also `/tmp` is at **98% inode use** (1,019,045/1,048,576) — my third `npm ci` failed ENOSPC on `mkdir node_modules/.bin`. The host needs `/tmp` cleanup (old worktrees/dependency trees) before the gate provisions a fresh seat.

---

## 5. V4 — what a Pack 20 PASS will require, and the exact gate commands

### 5.1 What the maker must deliver (I will verify each by running it)

On branch `duet/p20-demo-company`, final SHA **pushed to the remote** (a local-only commit does not exist for the gate):

1. **Scope item 1 — week scripts in the cast voices.** Every day's fixture carries the cast voice, the open-source stack artefact reference, and the chain hop/final reference; at least one Monday-created item/distortion is still present in Thursday's record. Evidence: per-day counts + the Monday→Thursday reference, committed as a script + output (not prose).
2. **Scope item 2 — chain support in the dry-run tooling.** `chain.hop`/`chain.final` handlers exist; one dry run executes the seven chains once each and scores **at the chain's end**, not hop by hop; `--calibrate` still gives null chain 0/2, oracle 2/2. Evidence: handler count > 0, dry-run output with end-of-chain verdicts, calibration output.
3. **Scope item 3 — X-1..X-4 design.** A doc under `docs/acceptance/demo-company/` containing X-1..X-4 (template-hardening loop over many fictional companies) with the honesty rule: live-driven vs state-advanced (scripted time-skip) stretches always distinguished; state-advanced time never reported as live. Evidence: the doc itself; a fresh reader can see the batch→template feedback loop without a single live claim.
4. **Scope item 4 — full suite + independent check.** Suite totals on the **final tree**, quoted with the environment named; my fresh-context PASS on the Pack 20 scope (this report defines the gate; the maker's own run is input, never proof).
5. **S1/main absorbed.** The final tree has `origin/main` as an ancestor; if `main` moves during S2–S5, re-merge before the gate (the S1 merge `75948eb` already absorbs `b59b599`; the gate re-proves it on the final SHA).
6. **Honesty leg.** The return's first paragraph carries either a genuine recursive self-improvement instance through the promotion gate, or the written finding explaining why none occurred — no fixture theatre. I will test the claim, not read it.
7. **No branch-attributable failures, no uncommitted files, no stale evidence.** `git status --porcelain` empty; dry-run evidence regenerated on the final tree (the current `evidence/` predates the stack/voices per the P20-WIP commit message).

### 5.2 The exact gate commands I will run

```bash
# 0) Fresh seat of the maker's final pushed SHA — never the maker's worktree.
git clone https://github.com/cayleb-james2008/ergon.git /tmp/ergon-gate && cd /tmp/ergon-gate
git checkout <FINAL_SHA>
git status --porcelain                       # expect empty
git rev-parse HEAD origin/main

# 1) Merged-tree proof (main absorbed before the gate).
git fetch origin main
git merge-base --is-ancestor origin/main HEAD; echo "merged_tree_proof=$?"     # expect 0

# 2) Real dependency install (no symlinks) + document runtime.
npm ci --ignore-scripts && node scripts/patch-bundled-npm.mjs
(cd support/server && npm ci --ignore-scripts)
./.document-runtime/bin/python -I -S documents/verify-dependencies.py          # expect exit 0
#   (the gate seat needs a hash-verified .document-runtime matching documents/dependency-lock.json;
#    a tree without the real dependency install is not a valid seat — my negative control scored 17
#    incomplete checks, not a tree verdict.)

# 3) Scope checks (my own script, on disk).
node /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/ergon-checker-gate-checks.mjs "$PWD"
#   expect: GATE-SCOPE PASS (week-fields, mon-carry-thu, chain-handlers, x1-x4-design, scorer-calibration)
#   then run the maker's dry run on the final tree and check 7 end-of-chain verdicts:
ROOT=$(mktemp -d); OUT=$(mktemp -d)
node docs/acceptance/demo-company/fixtures/dry-run-week.mjs "$ROOT" "$OUT"; echo "dry_run_exit=$?"   # expect 0

# 4) Full suite in the repo-documented baseline environment (no opencode binary, OPENCODE_BIN unset).
CLEANPATH=$(echo "$PATH" | tr ':' '\n' | grep -v -E '/(\.local/bin|\.opencode/bin|mise/shims|mise/installs/opencode/)' | paste -sd:)
env -u OPENCODE_BIN PATH="$CLEANPATH" node scripts/test-all.mjs | tee /tmp/gate-final.log; echo "suite_exit=${PIPESTATUS[0]}"
#   expect: exit 0; TOTAL line with 0 fail and 0 without a complete report (>=1,609 cases expected;
#   exact number depends on tests the maker adds)

# 5) The documented opt-in opencode e2e (separate from the full suite).
(cd support && OPENCODE_BIN=/home/cayleb/.local/share/mise/installs/opencode/1.19.0/opencode \
   node --test --test-reporter=tap tests/opencode-e2e.test.mjs)               # expect 6/6 pass

# 6) Branch-attributable failure count = 0: same suite on main, same environment, diff failing case names.
git worktree add /tmp/ergon-gate-main origin/main
#   provision /tmp/ergon-gate-main identically (steps 2), then:
(cd /tmp/ergon-gate-main && env -u OPENCODE_BIN PATH="$CLEANPATH" node scripts/test-all.mjs > /tmp/gate-main.log 2>&1; echo "main_suite_exit=$?")
comm -23 <(grep -a '^not ok' /tmp/gate-final.log | sort) <(grep -a '^not ok' /tmp/gate-main.log | sort) > /tmp/branch-attributable.txt
wc -l < /tmp/branch-attributable.txt                                         # MUST be 0

# 7) After the Lead-authorised merge: remote main proof.
git ls-remote https://github.com/cayleb-james2008/ergon.git refs/heads/main
gh api repos/cayleb-james2008/ergon/commits/main --jq '.sha+" | "+(.commit.message|split("\n")[0])'
git merge-base --is-ancestor <FINAL_SHA> <NEW_MAIN_SHA>; echo "main_contains_pack20=$?"   # expect 0
git log --oneline --merges -3 <NEW_MAIN_SHA>   # the Pack 20 merge commit must be named
```

### 5.3 Acceptance criteria (all must hold, or the gate is FAILED with the failing leg named)

- Scope checks **PASS** (all 5) + dry run **exit 0** with 7 chains scored end-of-chain + calibration null 0/2 / oracle 2/2.
- Final-tree suite **exit 0**, `TOTAL` quoted, **0 fail, 0 without a complete report**; environment named.
- Documented opt-in e2e **6/6 pass**.
- **Branch-attributable failure count = 0** (empty `comm -23` output).
- **Merged-tree proof = 0** before the gate and **remote main contains `<FINAL_SHA>`** after the merge.
- Honesty leg: claim checked and honest (genuine instance or written finding).
- If a leg fails: **FAIL** goes back to the maker with the exact finding; **the same step failing twice escalates to the Lead** — no third workaround.

### 5.4 What I will not accept as proof

The maker's own suite log without the environment named; exit 0 claimed without the `TOTAL` line; scope claims without per-day counts / committed evidence; stale dry-run evidence from before the stack/voices work; a "green" achieved by excluding checks without a manifest-recorded reason; any number not produced on the final pushed SHA.

---

## 6. Risks / open items for the Lead

1. **The branch moved after Team A's report.** Tip is `75948eb` (S1 merge of `b59b599` into `709bf0e`, pushed 18:34:49 ET). No S2–S5 commits exist on the remote as of 19:35 ET. The Lead should confirm the S1 push was the authorised go; the gate is on the final SHA, not on `709bf0e`.
2. **The suite is green today — the gate is about the new scope work, not about fixing a broken baseline.** Baseline proof: `main`, P20 and the S1 tip are all exit 0 / 0 fail / 0 incomplete in the documented environment (§3.2). If the final tree is not green, it is the maker's new work, and the branch-attributable diff will name it.
3. **Host hygiene blocks a clean gate seat.** `/tmp` at 98% inode use; six orphaned `opencode serve` v2 processes (~690 MB RSS total) from earlier suite runs. Clean `/tmp` old worktrees/dependency trees and kill the orphans before the gate; otherwise the fresh `npm ci` fails ENOSPC (as mine did once).
4. **Test-hermeticity bug (not a blocker):** `support/tests/smoke-cli.test.mjs` does not unset `OPENCODE_BIN`, so the full suite fails one case if `OPENCODE_BIN` is exported. Gate rule: full suite runs with `OPENCODE_BIN` unset; the e2e runs as the separate documented opt-in.
5. **Case-count conflict unresolved, measured numbers used.** 1,577 appears in the Job 03 brief; measured is 1,564 (P20) / 1,609 (`main`/S1) in the documented environment; 1,558 / 1,603 as-found. The repo's 1,451 recorded proof is a Windows run with 87 failures, not a green baseline.
6. **My seat vs the canonical seat.** My numbers come from a real `npm ci` + a hash-verified `.document-runtime` (repo verifier exit 0), not from Team A's scaffolding. The canonical gate seat must match that, or its numbers are not comparable.

---

## 7. Reproducibility

- Raw run extracts: `board/probes/ergon-checker-baseline-runs.txt`.
- Gate scope checker: `board/probes/ergon-checker-gate-checks.mjs` (usage: `node … <worktree-root>`; exit 0 = all scope legs pass). Run against the baseline `709bf0e` it correctly returns FAIL on week-fields / mon-carry-thu / chain-handlers / x1-x4-design and PASS on scorer-calibration.
- My throwaway artifacts (not durable): clone/worktrees under `/tmp/ergon-checker/{ergon,wt/*}`, full logs under `/tmp/ergon-checker/logs/`, dependency trees inside those worktrees. The durable record is this report + the two probe files.

*Written by `ergon-checker`, 2026-10-31 ~19:40 ET. I built none of the work above; every number quoted was produced by a command I ran today and is reproducible from the commands in §2–§5.*
