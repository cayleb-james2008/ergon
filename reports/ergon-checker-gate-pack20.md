# Ergon checker — PACK 20 GATE VERDICT: **PASS**

**Checker:** `ergon-checker` (independent fresh context; built none of this work). **Reports to:** `lead`. **Veto:** holds on any Ergon merge to `main`.
**Package under gate:** repo `cayleb-james2008/ergon`, branch `duet/p20-demo-company`, maker-declared final SHA **`462277ca46445273ed346f025fbaf5fddb0ba068`**.
**Gate run:** 2026-10-31 19:51–19:59 ET on `outis-desktop`, my own **fresh clone** `/tmp/ergon-gate-462277c` (cloned 19:51:12 ET), real `npm ci`, hash-verified document runtime, gate environment per my V3 ruling (`OPENCODE_BIN` unset, no `opencode` on `PATH`). No maker worktree, log, dependency tree or number was reused.
**Writes made:** this report + `board/probes/ergon-checker-gate-pack20-runs.txt` (raw extracts). No repo/branch/merge write; zero spend; no credentials; DeepSeek R1 unused.

## Verdict

**PASS on all four Job-01 scope items.** The final tree is green in the documented gate environment, the branch carries the merged main base, the week runs in the cast voices with the stack artefacts and the seven chains carried through, chains are scored once at their end, X-1..X-4 exist with the honesty rule, and the P20-8 self-improvement finding is honest and evidenced (the rehearsal is labelled and not counted). **Branch-attributable failures = 0.** I found no fixture theatre and no scope defect. Three low-severity documentation-precision findings are recorded for the next Ergon commit; **none blocks the merge**.
**Merge recommendation: YES — the Lead may authorise the merge of `462277c` into `main`.** My veto is lifted for this package. After the merge I verify the remote `main` contains the final SHA and re-check the merged tree (`main`-ancestry + suite totals).

## 1. Package and preconditions (all verified by me)

| Check | Command | Result |
|---|---|---|
| Final SHA is the pushed remote tip | `git ls-remote … refs/heads/duet/p20-demo-company` | `462277ca…` — equals my checkout |
| Branch carries current main base | `git merge-base --is-ancestor b59b599 HEAD; echo $?` | **0** |
| Branch carries `origin/main` | `git merge-base --is-ancestor origin/main HEAD; echo $?` | **0** |
| Tree clean | `git status --porcelain \| wc -l` | **0** (also 0 after all gate runs; HEAD unchanged) |
| Dependencies (real, not symlinked) | `npm ci --ignore-scripts`; `node scripts/patch-bundled-npm.mjs`; `(cd support/server && npm ci --ignore-scripts)` | exit 0 / 0 / 0 |
| Document runtime | `./.document-runtime/bin/python -I -S documents/verify-dependencies.py` | "Completed pinned private document runtime verified." exit 0 |

## 2. Scope-by-scope verdict

| # | Scope item | Verdict | My evidence (run, not read) |
|---|---|---|---|
| 1 | Week scripts run in the cast voices, with the open-source stack artefacts, and the chains carried through (Monday distortion still in Thursday's record) | **PASS** | `check-voices.mjs` (my run): 99 authored lines, blind separation **99/99**, `ALL VOICES PASS`, exit 0. `check-stack.mjs` (my run): 17 artefacts / 8 tools / 11 acts open artefacts, `all checks passed`, exit 0. My week-field scan: per day non-zero `voice`/`stack`/`chain` fields (mon 10/2/4, tue 15/2/4, wed 20/2/7, thu 21/2/4, fri 17/2/3). Fresh dry run: C1 Monday drop **still missing at Thursday's final** (`resurfaced.stillMissing: true`); 152 acts all carry `voiceOk`/`stackWritten`/`stackOpened`. |
| 2 | Chain support in the dry-run tooling, one score at each chain's end, no per-hop scoring | **PASS** | `chain.hop`/`chain.final` handlers present (4 occurrences); my structural check: the `chain.hop` handler contains **no** `scoreChainAnswer` — scoring appears only in the `chain.final` handler. Fresh run: `chains.count 7`, `scoredChains 7`, `everyChainScoredAtChainEnd: true`, `hopsRecorded 24`, every row `scoredAt: "chain-end"`. `chains.mjs --check`: ok, 6 kinds covered; `--calibrate`: separates all (oracle caught / charge-ahead charged-ahead / null missed); scorer `--calibrate`: null chain 0/2, oracle 2/2. |
| 3 | X-1..X-4 design: the template-hardening loop with the checked honesty rule | **PASS** | `docs/acceptance/demo-company/TEMPLATE-HARDENING.md` (155 lines) — judged on content: X-1 the batch (8 stated variation axes incl. size/sector/role/device/stack/horizon/pressure/model config; tiering with baseline fidelity and sparser-tier calibration); X-2 the run (one command per company-week, **two clocks** live vs state-advanced per act, fixed evidence layout, scoring/classification of expectation-not-met rows, cost accounting and stop rules); X-3 the learning loop (finding classes → cluster → smallest template rule + check → promotion gate → regression pass → append-only `batch-ledger.jsonl`; "corrections become rules, not patches"); X-4 checked honesty reporting (explicit **"state-advanced time is not live operation"**, per-side models, unresolved list, reproduction; recursive-self-improvement tie to P20-8; audit of the rule itself). Plus Job 02's build list and effort estimate. |
| 4 | Full suite green on the final tree + a genuine recursive self-improvement instance or its written honest finding | **PASS** | Suite on `462277c`, gate env, my run 19:56:21–19:57:50 ET: `TOTAL 1609 cases: 1600 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 157/157 checks ran, 0 without a complete report; 4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs Windows); 0 manifest problems` — **EXIT=0**; `AGGREGATE_OUTCOME … "uncertainChecks":[]`. Opt-in e2e (documented binary opencode 1.19.0): **6/6 pass**, exit 0. Branch-attributable failures: `comm -23` of final-vs-main `not ok` lines = **0 lines**. Honesty leg: see §4. |

## 3. The runs (quoted; full extracts in `board/probes/ergon-checker-gate-pack20-runs.txt`)

**Gate scope checker** (`board/probes/ergon-checker-gate-checks.mjs`, on the final tree with the fresh dry-run summary):
```
PASS week-fields / PASS mon-carry-thu {"c1MonHop":true,"c1ThuFinal":true,"resurfacedStillMissing":true}
PASS chain-handlers: handlers=4 hopBlockScores=false
PASS x1-x4-design: TEMPLATE-HARDENING.md (honesty rule: true)
PASS scorer-calibration: {"nullChain":0,"nullChainMax":2,"oracleChain":2,"oracleChainMax":2,"exit":0}
PASS chains-run: {"count":7,"scoredChains":7,"everyChainScoredAtChainEnd":true,"hopsRecorded":24,"rows":7,"allScoredAtEnd":true}
GATE-SCOPE PASS: 6/6 checks pass (summary: fresh dry run)   EXIT=0
```

**Fresh dry run in the gate seat** (19:52:52–19:54:41 ET, exit 0): label `STAND-IN OUTPUT (deterministic local stand-in; not model capability)`; `modelsUsed` says "no model was called"; per day mon 25 acts/160 calls, tue 24/23, wed 32/30, thu 35/35, fri 36/66; **314 model calls, all via the budget proxy**; 7/7 chains scored at chain end, 24 hops; C1 `resurfaced.stillMissing: true`. **My fresh run's day-close hashes equal the committed `captures.json` hashes exactly** (mon `fab378ea…`, tue `ddc42723…`, wed `129ed7d3…`, thu `4a8b2246…`, fri `77e75c77…`) — the determinism claim reproduces across an independent seat, not just across the maker's two roots.

**Full suite** (19:56:21–19:57:50 ET):
```
TOTAL 1609 cases: 1600 pass, 0 fail, 0 cancelled, 9 skipped, 0 todo; 157/157 checks ran, 0 without a complete report;
4 checks skipped every case (allowed by the manifest); 6 manifest invocations not run here (5 need .NET, 1 needs Windows);
0 manifest problems
EXIT=0
```
9 skips, each with a named reason (5 environment skips + the 4 opencode-e2e cases that need the binary); 4 checks skip every case, all manifest-allowed; 6 seat invocations not run and never counted as passing. No `not ok` lines.

**Opt-in opencode e2e** (19:58:33–19:59:00 ET): `# tests 6, # pass 6, # fail 0`, EXIT=0.

**Branch-attributable failures = 0**: final tree and `main` (my same-environment baseline run) both `TOTAL 1609 / 1600 pass / 0 fail / 9 skipped / 0 incomplete, EXIT=0`; `comm -23` diff of failing-case names is empty.

**Maker's claims checked against my runs:** suite totals (1,609/1,600/0/9/0, exit 0) — **confirmed exactly**; per-day scripted/total acts (mon 20/25, tue 17/24, wed 24/32, thu 24/35, fri 25/36) — **confirmed** (fixture acts 20/17/24/24/25; the run adds background acts); voices 99/99 blind — **confirmed**; stack 11 acts — **confirmed**; chains 7/7, 24 hops, C1 stillMissing — **confirmed**; X-1..X-4 — **confirmed on content**; dry-run evidence — **confirmed and independently reproduced**. As-found-environment exit 1 was not re-run on `462277c` (V3 environment gap already ruled; P20-10 touches only `docs/acceptance/demo-company`, so the as-found behaviour is unchanged by construction).

## 4. Honesty leg (P20-8) — verified, not read

- `SELF-IMPROVEMENT.md` line 3: **"Verdict: current code cannot produce a genuine instance."** No agent's self-improvement fires on main; the gate-mechanics rehearsal is labelled and explicitly **not counted** as the instance.
- I checked the finding's evidence myself: `recordObservation` is legal only in `observing`/`interviewing` (after activation it refuses), and the fresh run's `wed-05` observed `{"onboardingRecord":{"refused":"invalid_transition"}}`; `suite/build-loop/LOOP.md:49` says the loop "does not generate the tool's implementation"; the library manifest names `weekly-estimate.run` (code that does not exist); probe PR-11 exists in `evidence/probes-output.txt`.
- The rehearsal acts (wed-12…fri-07) carry `standIn` labels naming the missing runtime caller/screens; the library record's agent labels are explicitly cautioned against in the same document. **No fixture theatre found** — the finding is honest, evidenced and self-limiting.

## 5. Findings (all non-blocking; docs precision, not scope defects)

- **F1 — `TEMPLATE-HARDENING.md` X-2 fixed evidence layout names `day-close-hashes.json`, which the current runner does not emit** (the hashes live in `summary.json` `days[].close.canonicalDumpSha256`, and `capture-week-evidence.mjs` records both runs in `captures.json`). The layout is normative for Job 02's batch driver; either implement the file there or mark the line as batch-driver output.
- **F2 — `SELF-IMPROVEMENT.md` step-2 wording "Nothing calls `evaluateCandidates` … outside `suite/build-loop` and its tests" is too strong**: `fixtures/dry-run-week.mjs` calls them as the labelled rehearsal (the same document and the week `standIn` fields say so precisely). Reword to "no product/runtime code calls them; only the labelled Pack 20 fixture rehearsal does".
- **F3 — the `SELF-IMPROVEMENT.md` gate-audit block is an annotated/reformatted rendering** ("(fixture stood in for the agent: see above)") — that parenthetical is not in the JSONL. The prose immediately cautions about the record, so this is not theatre, but the block should be marked "annotated".
- **F4 — process note for the Lead/maker:** the P20-8 finding is at the top of `SELF-IMPROVEMENT.md` and surfaced in `README.md`; the consolidated Pack 20 return to Cayleb should lead with it (Job 01's done-condition wording).

Disposition: F1–F3 can be fixed in one small docs commit (next Ergon commit / first Job 02 commit). They do not affect any scope item's evidence or the merge decision.

## 6. What this PASS means, and the next step

- The merge precondition is met: **the Lead may authorise merging `462277c` into `main`.** No merge was performed by me; I hold the veto and release it for this package.
- After the Lead-authorised merge I will run the post-merge proof: `git ls-remote … refs/heads/main` shows the new tip, `git merge-base --is-ancestor 462277c <new-main>` = 0, and the merged-tree suite re-run (gate env) stays `0 fail / 0 incomplete / exit 0`.
- Job 02 is unblocked once the merge lands (Pack 19 already on `main`; Pack 20 is this branch), and should carry F1–F3 as its first docs corrections.

## 7. Reproducibility

- Raw extracts with timestamps and quoted output: `board/probes/ergon-checker-gate-pack20-runs.txt`.
- Gate scope checker: `board/probes/ergon-checker-gate-checks.mjs` (`node … <worktree> [summary.json]`; exit 0 = all scope legs pass).
- Throwaway seat and logs were under `/tmp/ergon-gate-462277c` and `/tmp/ergon-checker/logs/` (disposable); the durable record is this report + the probe file.

*Written by `ergon-checker`, 2026-10-31 ~20:02 ET. I built none of this work; every number quoted was produced by a command I ran in this gate, on the maker's pushed SHA, in my own seat.*
