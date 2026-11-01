# Team A — Lane 5 (Job 03) batch report: rehearsal package assembled + R0–R7 filed

**Batch:** A-1 … A-5 (Lane 5, Ergon Job 03). **Date:** 2026-11-01 08:29–09:00 ET. **State:** prep only — the rehearsal
has **not** run, no main writes, no spend, no credentials. Announcement message sent to the Lead 08:40 ET; Lead rulings
received 08:41 ET (crew depth, merge route, prep-only, host not yet).

**Result in one line:** R0–R7 labels filed, the Job 03 rehearsal package indexed and integrity-verified on `main @
dffb44ef`, the one-page run sheet written, C1–C6 recorded as PENDING with the open arbitration; branch + PR #10 up for
a fresh-context checker. The run is unblocked only by Lane 1's installed output (plus the gates named in the run sheet).

## 1. What was produced (branch `docs/l5-acceptance-r0-r7`, commit `86e874ac`)

| # | Item | File | Status |
|---|---|---|---|
| A-1 | R0–R7 stage labels filed into the repo (Nox ruling `20261101T050327Z` §2 gap rule), sourced from the lane note with the source marked and repo grounding added per step | `docs/acceptance/demo-company/REHEARSAL-R0-R7.md` | Done on branch; checker pending |
| A-2 | Job 03 demo-company package assembled from the Nox-pinned artefact set on `dffb44ef`: six named artefacts + `evidence/dry-run-standin/` indexed with SHA-256, honesty labels, harness refs, and the not-in-package boundary. Nothing rebuilt | `docs/acceptance/demo-company/PACKAGE.md` | Done on branch; checker pending |
| A-3 | Rehearsal package completed for the RUN: gates, owner map, exact commands, per-step evidence, `order-e`/`order-f` + DONE-BAR/EVIDENCE-INDEX/KNOWN-LIMITATIONS references, stop rules, successor (one page) | `docs/acceptance/demo-company/RUN-SHEET.md` | Done on branch; checker pending |
| A-4 | C1–C6 frozen contract set recorded as **PENDING Nox confirmation** + the open arbitration (ADD-only `main[key]` W1 view vs documented reshape of the chat view); nothing applied | `docs/acceptance/demo-company/CONTRACT-C1-C6-PENDING.md` | Recorded; ruling open |
| A-5 | Write-area declaration | below §5 | Filed |
| — | Pointer section in the demo-company README | `docs/acceptance/demo-company/README.md` | Done on branch |

Branch: `docs/l5-acceptance-r0-r7`, base `dffb44ef` (origin/main), commit `86e874ac5cc01e5ab5fca73d5d19f9c48487af01`,
5 files, +494 lines, docs only (no product code, no fixture or committed-evidence change). New-file SHA-256:
PACKAGE.md `f83a3b40d620fbda…`, REHEARSAL-R0-R7.md `987fb156508244d6…`, RUN-SHEET.md `b9bef871472071d3…`,
CONTRACT-C1-C6-PENDING.md `ef7a308c94405eaa…`.

**PR #10 OPEN, MERGEABLE:** https://github.com/cayleb-james2008/ergon/pull/10 (base `main`, head
`docs/l5-acceptance-r0-r7`; remote branch tip = `86e874ac`; verified with `gh pr view 10 --json …`,
2026-11-01 08:59 ET). Team A pushed a branch + PR only — **no main write** (Lead ruling 08:41 ET).

## 2. Verification — observed, not asserted (commands + results + timestamp)

All run on the branch worktree at `dffb44ef`/`86e874a`, 2026-11-01 08:43–08:59 ET. Outputs are from the run logs
`/tmp/l5-package-verify-20261101T084440.log`, `/tmp/l5-package-verify2-20261101T084646.log` and the direct commands
below.

| Check | Command | Result |
|---|---|---|
| Base pin | `git ls-remote origin main` | `dffb44efa8c67d2bbb37eafc1af97c525fb5b9db refs/heads/main` — matches the ruling's pin |
| Run config | `node fixtures/check-run-config.mjs` | `accepted: employeeModel claude-haiku-5-5, agentModel claude-haiku-5-5, route budget-proxy` — EXIT=0 |
| Cast voices | `node fixtures/check-voices.mjs` | `99 written lines checked (85 fixtures + 14 anchors); blind separation 99/99` … `ALL VOICES PASS` — EXIT=0 |
| Chains well-formed | `node fixtures/chains/chains.mjs --check` | `"ok": true`, `"problems": []`, 7 chains, kinds `attachment-not-forwarded, casual-approval-as-formal, chains-collide, detail-dropped, number-transposed, urgency-inflated` — EXIT=0 |
| Chain rubric separates | `node fixtures/chains/chains.mjs --calibrate` | `"separatesAll": true`; every chain `oracle caught` / `chargeAheadExample charged-ahead` / `nullAnswer missed` — EXIT=0 |
| Seeded past stands up | `node fixtures/setup.mjs $ROOT`; `node fixtures/dump-state.mjs $ROOT`; `node fixtures/teardown.mjs $ROOT` | setup EXIT=0 (104 tickets, 133 replies, 9 stack artefacts, 10 devices); dump `{"sha256":"558df9edb29c8c5b…","bytes":315258}`; teardown `{"empty": true}` — EXIT=0 |
| Week dry run (stand-in) | `node fixtures/dry-run-week.mjs $ROOT $OUT` | EXIT=0 in **1m29.6 s**; summary: `"label":"STAND-IN OUTPUT … not model capability"`, `clocks.live={"acts":0}`, `clocks.stateAdvanced={"acts":152,"simulatedDays":5,"wallSeconds":89.51}`, `chains.scoredChains=7`, `everyChainScoredAtChainEnd=true`, `hopsRecorded=24`; `modelsUsed` both sides `stand-in … no model was called` |
| Wall-clock estimate | `node fixtures/estimate-wallclock.mjs $OUT/summary.json` | per-day serial/parallel estimates printed; basis line says assumed Haiku-class figures, to be replaced by measurement — EXIT=0 |
| Stack artefacts | `node fixtures/check-stack.mjs $OUT` | `artefacts 17; tools named 8; named but never touched: []; all checks passed` — EXIT=0 |
| Repo guard on new docs | `node tests/package-contents.test.mjs` | `tests 5 / pass 5 / fail 0` — EXIT=0 |
| Doc links | link-check over the 5 files | `checked 87 relative links across 5 files; missing 0` — EXIT=0 |
| PR state | `gh pr view 10 --json …` | `{"state":"OPEN","base":"main","head":"docs/l5-acceptance-r0-r7","mergeable":"MERGEABLE","commits":["86e874ac…"]}` |

What this proves: the pinned artefact set is intact and executes on `dffb44ef`, and the docs set is self-consistent
(it does not break the package-contents guard). What it does **not** prove: any model capability, any live operation,
or any rehearsal step. The rehearsal run remains blocked (§4).

## 3. The package as assembled (what the checker will look at)

- Pinned artefact set (all `docs/acceptance/demo-company/`): COMPANY.md `6ef0f314…`, CAST.md `42baea0e…`, SEED.md
  `5fbd8e33…`, SCENARIOS.md `44cc9f87…`, COMPETENCY.md `8d7701e2…`, RUN-REPORT-TEMPLATE.md `5b46b876…`; evidence
  `evidence/dry-run-standin/` (16 files, hashes in PACKAGE.md §3) and `evidence/batch-standin/` (6 files); supporting
  WEEK.md/SELF-IMPROVEMENT.md/TEMPLATE-HARDENING.md/README.md + `fixtures/`.
- Harness references with their holds stated: `order-e/` is **GROUNDWORK ONLY** (H1 no API key on record, H2 the ruled
  package does not exist on main, H3 clean user not created), `order-f/` is **SKELETON ONLY** with DONE-BAR **HOLD**
  (seven bar lines Nox-side).
- Stand-in honesty: live acts 0; state-advanced only; 314 model calls through the budget proxy in the committed week
  stand-in; chains 7/7 scored once at their end; "expectation not met" rows are stand-in limits kept visible; fleet
  audit pending. Batch stand-in: 0 live acts, 1768 state-advanced acts / 56 simulated days, 8 companies, 72
  stand-in-limit findings, TC-1 recorded as a proposal only.
- Not in the package (named, not smoothed): the ruled DSH-era product package (main still builds the OpenClaw 1.5.5
  baseline; `package.json` name `openclaw-personalized-agent-baseline`), C1–C6 freeze, Windows-only surfaces
  (companions, stand-in/pending), phone/voice surface (PACKAGE-PENDING), warehouse board + Supabase wiring (work-PC
  window `RHLCO-WKS-09`), the seven done-bar lines (Nox HOLD).

## 4. Blockers needing a ruling (recommended answer each)

| # | Blocker | Recommended answer |
|---|---|---|
| B1 | **Checker for PR #10** — maker never checker; a fresh-context reviewer must PASS the branch before it merges | Lead spawns the fresh-context checker on PR #10 (similar scale to the Job 02 gate; the branch is docs-only). Merge by Lead/Nox after PASS. |
| B2 | **C1–C6 freeze + client-shape arbitration** — recorded as PENDING; nothing may be built | Nox confirms the freeze set (C1 freeze alpha.2 + delta review; C2 pin alpha.2; C3 slots+locale, no plugin-manager; C5 Linux bundle + Windows companions; C6 private tarball) and rules ADD-only `main[key]` W1 view vs reshape. Until then Job 03 stays prep/assembly. |
| B3 | **The RUN needs Lane 1's installed output** — G1 in the run sheet; the ruled package does not exist on main (order-e H2) | Lane 1's install closing is the successor trigger. No host purchase or run scheduling before then (Lead ruling d). |
| B4 | **Host/URL/Supabase + owner acts** — not schedulable yet | Do not raise until G1; then route the host purchase (€8/mo CPX22-class), DNS name and Supabase org/project confirmation through Nox as owner acts. |
| B5 | **Crew depth** — Nox follow-up 20261101T132614Z grants leads crews | My runtime fixed delegation depth at 0 (`subagent`/`subagent_fork` remaining levels 0; `spawn_teammate` is Lead-only), so I cannot spawn. Lead confirmed direct execution 08:41 ET. Recorded; no crew spawned. |

## 5. A-5 write-area declaration (Team A / Lane 5, ergon repo only)

Owned and actively used:
- `/home/cayleb/workspace/harness-build/a-l5-seat` — **new worktree** from `dffb44ef`, branch
  `docs/l5-acceptance-r0-r7` (this batch's only authoring seat; no other lane needs it).
- `/home/cayleb/workspace/harness-build/ergon-main-merge` — main worktree, clean @ `686fca8`; used read-only this
  batch (fetched `origin/main` = `dffb44ef`; its fetch refspec is pinned to `team`, so `main` is fetched explicitly).
- `/home/cayleb/workspace/harness-build/p20-seat` — branch `duet/p20-demo-company` @ `462277c` (closed job; not touched).
- `/home/cayleb/workspace/harness-build/p21-seat` — branch `duet/p21-fixes-scaleup` @ `4387b51` (closed job; not touched).
- `/home/cayleb/workspace/harness-build/ref/ergon-team` — board-sync clone on branch `team` (read-only this batch).
- Workspace board: `board/reports/team-a-*.md`, `board/probes/team-a-*`. Nothing written to
  `~/Work/shared-workflows/lanes/board/**` (Nox board, read-only) or to another team's files.

Experimental node_modules symlinks used only to run the fixture checks were removed before commit; the branch carries
docs only.

## 6. Successor

1. **Now:** Lead spawns the fresh-context checker on PR #10; on PASS Lead/Nox merges the branch.
2. **On Lane 1 installed output (G1):** the run sheet becomes executable — the rehearsal RUN (R0–R7) with the
   fresh-context rehearsal checker named by the Lead; Team A drives the run, records per-step evidence, fixes findings
   and re-runs until a full pass is clean or a finding is named a genuine blocker.
3. **In parallel, no idle:** prepare the run's evidence directory skeleton and the R0 provisioning checklist against the
   actual host once named; hold C1–C6 and the arbitration until Nox rules (no build).

**Evidence state:** all quotes above are command output from this batch, captured 2026-11-01 08:43–08:59 ET; the run
logs are on disk at the paths named in §2. Unverified items are marked PENDING/HOLD/BLOCKED — nothing in this report is
upgraded to "done" without the named gate.

---

# Batch 2 (Lead-approved 2026-11-01 09:0x ET): run-evidence skeleton + R0 provisioning checklist

**Scope:** Lead approved item (a) only — run-evidence skeleton + R0 provisioning checklist, generic host, no spend, no
host purchase. Job 04 (b) skipped (out of topology, Windows VM not existing). Job 05 (c) one-line standing check.
PR #10 remains under the Lead's fresh checker; **nothing was committed to that branch** — batch 2 lives on the
workspace board and folds into the repo as a follow-up docs branch only after PR #10 merges.

## Produced

| Artefact | Path | What it is |
|---|---|---|
| Run-evidence skeleton | `board/probes/team-a-job03-run-evidence-skeleton/` | Two passes (`pass-1/`, `pass-2/`), 8 step dirs each (R0-pre, R1–R6, R7-close), `notes.md` template per step with the 10 required fields, pass `index.md` (run metadata, R0–R7 verdict table, secrets check, checker sign-off block — Team A never signs it), R0-pre evidence templates, R7-close state-record/teardown/pass-diff templates, README with the word vocabulary and rules. **Templates only; no pass recorded.** |
| Preflight tool | `.../r0-preflight.sh` | Generic-host R0 preflight: host fingerprint, package SHA-256, run-config acceptance (budget-proxy only), absence of company state, URL name (offline), project id; writes `preflight-summary.txt`; read-only except its output dir; no sudo, no network, no secret reads. |
| Skeleton validator | `.../check-skeleton.mjs` | Structure + required fields + secret-pattern scan; exits non-zero on any problem. |
| R0 provisioning checklist | `board/probes/team-a-job03-r0-provisioning.md` | Gates G1–G7, generic host spec, P1–P10 steps with owner + exact commands + evidence + pass condition, no-secret rules, cost/authorisation, stop rules, successor. |

## Verification (observed, 2026-11-01 09:08–09:10 ET)

- `node check-skeleton.mjs` → `SKELETON OK: 2 passes, 16 step directories, 10 required note fields each, secrets scan clean` — EXIT=0.
- `bash -n r0-preflight.sh` → EXIT=0 (shellcheck not installed on this host; syntax checked only and said so).
- Smoke test: `bash r0-preflight.sh /tmp/<smoke> --repo /home/cayleb/workspace/harness-build/a-l5-seat --package /tmp/<dummy>.tgz --url rehearsal.example.com` →
  `PASS host fingerprint written / PASS package hashed: … 5adfbaa7de63771c… / PASS run config accepted (route budget-proxy) / PASS no company state on the checked paths / PASS url name recorded / PASS company project id recorded` → `R0 PREFLIGHT: PASS` EXIT=0. This is a **tool self-test on this desktop, not R0 evidence** — the files were smoke-test outputs and were discarded.
- Fail-closed test (no `--repo`, no `--package`, fresh dir) → `R0 PREFLIGHT: FAIL (3 blocking)` EXIT=1 with each missing item named; the tool cannot manufacture a clean R0.
- Job 05 one-line standing check (2026-11-01 09:10 ET): `git ls-remote --heads origin | wc -l` → **98 remote branches**, branches containing `kimi` → **0**, matching PRs → none; `gh pr list` shows no Kimi handback. **Kimi handback: still not arrived.**
- PR #10 state at 09:10 ET: `{"head":"86e874ac5cc01e5ab5fca73d5d19f9c48487af01","mergeable":"MERGEABLE","state":"OPEN"}` — head unchanged since filing; the Lead's checker verdict is pending.

## Status / successor

- Batch 2 = **prep complete, unexecuted**: no host named, no spend ($0), no credentials, no rehearsal run.
- The R0 checklist and skeleton are **not in the repo** by design: folding them in now would change the tree the checker
  is reviewing on PR #10. Successor (after PR #10 PASS + merge): docs-only branch adding
  `docs/acceptance/demo-company/run-evidence/` (the skeleton) and `R0-PROVISIONING.md`, then the checker scope for that
  fold-in comes from the Lead.
- The RUN stays unscheduled until Gate G1 (Lane 1's installed output) and the Lead schedules it; then `r0-preflight.sh`
  runs on the real host into both passes' `R0-pre/` dirs and the operator's clock starts at R1.

---

# Batch 3 — fold-in branch after PR #10 merged (2026-11-01 ~11:36–11:50 ET, post-restart)

**Context:** session restarted twice (memory-plugin crash; lane state on disk). PR #10 merged by the Lead as
**6bcd4eb1b813ae674712d73ac8c25b060ea66c7a** at 14:37:25Z (main was `dffb44ef`); fresh-checker PASS recorded on the
docs-only branch. Lead GO: docs-only fold-in branch now — run-evidence + R0-PROVISIONING + the F1 doc fix; branch + PR
only, never main; RUN stays unscheduled until G1.

## What was done

- New worktree `~/workspace/harness-build/a-l5-foldin`, branch **`docs/l5-run-evidence`**, created from the merge commit
  `6bcd4eb`, then **rebased onto current `origin/main` `5c2eaed`** (main moved after the merge: installer Midnight Slate
  retheme + text-button accent fix, 4 files — `installer/ui/index.html`, `installer/ui/setup.css`,
  `scripts/generate-theme.mjs`, `support/server/dashboard.html`; no overlap with docs). Rebase clean.
- **Run-evidence fold-in** into the repo: `docs/acceptance/demo-company/run-evidence/` — pass-1/pass-2 with R0-pre,
  R1–R6, R7-close; 10-field `notes.md` per step; per-pass `index.md` (run metadata, R0–R7 verdict table, secrets check,
  checker sign-off block); R7 state-record/host-teardown/pass-diff templates; `check-skeleton.mjs`;
  `r0-preflight.sh` (mode 755).
- **`docs/acceptance/demo-company/R0-PROVISIONING.md`** — gates G1–G7, generic host spec, P1–P10 owner/commands/
  evidence, no-secret rules, cost/authorisation, stop rules.
- **F1 fix** (checker finding on PR #10): `PACKAGE.md` §3 now lists the **full 64-character SHA-256 values** for all 22
  evidence files (plus the 6 artefact hashes), re-measured on the merged tree; the false claim that the values repeated
  in the board report is removed; PR #10 merge commit recorded; §8 describes both prep branches; §7 pin sentence
  updated. `docs/EXIT-CODES.md` gains the two new entry points (required by `tests/exit-codes.test.mjs`);
  `demo-company/README.md` gains the two pointer rows.
- **Nox ADOPT record** (relayed by the Lead 2026-11-01): `docs/acceptance/ERGON-PACKAGE-GOVERNANCE-PATTERNS.md` —
  gate→eval→mandate kernel, observe-by-default initiative, versioned prompt-config staging with a separate approver.
  Design adopted, **code NOT**: no prototype import/vendor/run, no dependency, no `package.json` change; each pattern
  names the gap it fills (grounded in review gate/build loop/SELF-IMPROVEMENT/HUMAN-REVIEW) and its landing point in the
  open Job 03 W0/W2 work; Jobs 01/02 and the RUN untouched. Flagged to the Lead as beyond the three named files in case
  they want it split.

## Verification (observed on the rebased tree, 2026-11-01 11:40–11:50 ET)

| Check | Command | Result |
|---|---|---|
| Skeleton | `node docs/acceptance/demo-company/run-evidence/check-skeleton.mjs` | `SKELETON OK: 2 passes, 16 step directories, 10 required note fields each, secrets scan clean` — exit 0 |
| Preflight smoke | `bash …/r0-preflight.sh <tmp> --repo <tree> --package <dummy> --url rehearsal.example.com` | 6/6 `PASS`, `R0 PREFLIGHT: PASS` — exit 0 (tool self-test; outputs discarded, not evidence) |
| Preflight fail-closed | same without `--repo`/`--package` | `R0 PREFLIGHT: FAIL (3 blocking)` — exit 1 |
| PACKAGE.md hashes | script recomputing each stated hash | `checked 28 hashes, mismatches 0` |
| Links | script over the 28 committed markdown files | `checked 90 relative links across 28 committed markdown files; missing 0` |
| Entry points | `node tests/exit-codes.test.mjs` | 45 pass / 0 fail |
| Package contents | `node tests/package-contents.test.mjs` | 5 pass / 0 fail |
| Evidence freshness | `node tests/evidence-freshness.test.mjs` | 6 pass / 0 fail |
| Docs truth | `node tests/docs-truth.test.mjs` | 12 pass / 0 fail |

Commit `5cde3129f3acce0f309e54c5b7520fd6b3d6a388`; branch pushed; **PR #11 OPEN / MERGEABLE**
(`gh pr view 11 --json …`: base `main`, head `docs/l5-run-evidence`, headOid `5cde3129…`); 42 files, +940/−21, docs only.
No main write.

## Status

- PR #11 awaits the Lead's fresh-context checker; Lead merges on PASS (Team A never merges).
- F1 is fixed in-branch (full hashes verify 28/28).
- RUN unscheduled until Lane 1 reports the installed output (G1); C1–C6 + W1 still PENDING Nox; $0 spend; no host;
  no credentials.

---

# Batch 3 close-out — PR #11 merged; two queued corrections (2026-11-01 11:52 ET)

**PR #11 MERGED by the Lead:** `gh pr view 11 --json state,mergeCommit,mergedAt` → `state MERGED`, merge commit
**`6a455f915714ac49958e9d8c878478d50cd8debe`**, `mergedAt 2026-11-01T16:50:20Z`, merged by `cayleb-james2008`. Merge
parents: `848a41b` + `5cde3129` (verified: `git log -1 --pretty='%H %P' origin/main`); `git merge-base --is-ancestor
5cde3129 origin/main` → branch commit IS an ancestor. Merged tree carries 37 files under
`docs/acceptance/demo-company/run-evidence/`, plus `R0-PROVISIONING.md` and
`ERGON-PACKAGE-GOVERNANCE-PATTERNS.md`; `PACKAGE.md` on main has 28 full 64-hex hash values. Checker PASS recorded by
the Lead (42/42 files under docs/, tests 45/45 5/5 6/6 12/12, skeleton OK + fails closed on tampering, preflight 6/6 +
3-blocking, 28/28 hashes, 90/90 links, merge clean); the checker accepted the governance-patterns doc in the same PR as
not breaking the docs-only property. Main had moved `5c2eaed → 848a41b → 6a455f9` during the fold-in; the merge was
clean.

## Queued non-blocking corrections (do at the next docs touch, before the R0 run)

1. **Preflight "no network calls" wording is imprecise.** The docs/script say the preflight makes no network calls, but
   `getent hosts` (r0-preflight.sh, step 5, `--url` path) IS a local DNS resolution. Reword wherever it appears to:
   *"no outbound service calls; one local DNS resolution for the hostname (`getent hosts`), used only to record the URL
   name — the real reachability proof is R3 from the second device."* Files: `docs/acceptance/demo-company/run-evidence/README.md`
   (tool row), `docs/acceptance/demo-company/R0-PROVISIONING.md` §4 and §6, `docs/acceptance/demo-company/run-evidence/r0-preflight.sh`
   header comment, and the workspace copy in `board/probes/team-a-job03-run-evidence-skeleton/`.
2. **Branch-base freshness.** The fold-in PR was cut from `5c2eaed` while main moved to `848a41b`/`6a455f9` (merge was
   still clean, checker noted it). For the next branch: fetch main explicitly (`git fetch origin
   main:refs/remotes/origin/main --force` — this clone's refspec is pinned to `team`) and cut from the then-current tip;
   never assume the base recorded in a report is current.

Both are documentation-only; neither blocks the merge, the RUN, or any gate. They are recorded here so the next docs
change carries them rather than a third PR being opened for them.
