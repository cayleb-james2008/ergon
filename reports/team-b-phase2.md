# TEAM B — PHASE 2 REPORT (T1–T4)

Owner: `fleet-lead` (Team B / DSH FLEET). Written 2026-10-31 18:35 ET (22:35Z). Window: 18:20–18:35 ET.
Fences held: throwaway copies under `/tmp` only; **no mission-branch writes, no merge/deploy/install, no force-push, no spend, no credential touch, no write to `~/Documents/Codex/**`**. Repo worktrees untouched (`dsh-fleet` clean at `abfb842`; `dsh-fleet-next` clean at `0769797`).
Companion: `board/reports/stage2-gate-report-FINAL.md` (T2 output). Prior evidence: `board/reports/team-b-phase1.md`, `board/probes/team-b-seed1546.md`.

---

## T1. Orders-mutation survivors — triage of all 10 (candidate `0769797`)

Base and method: the recorded survivor list is `evidence/r6-gates/mutate-orders.txt` (15:05:12 ET, run on `next`@`5184978`): 174 mutants, 164 KILLED, 10 SURVIVED. I extracted the candidate tree with `git archive 0769797` (no worktree, no ref written) and **re-ran those 10 mutants against the candidate**:

```
$ cd /tmp/team-b-b2/next-0769797/package/src/orders
$ TMPDIR=/tmp/x MUTATE_WORKERS=2 python3 -I test/mutate.py 23 24 41 42 70 89 110 116 152 156
23 SURVIVED       roll-up without final task
24 SURVIVED       roll-up with unfinished tasks
41 SURVIVED       maker records verdict
42 SURVIVED       double checker verdict
70 SURVIVED       R2-1 any checker pass rolls up a command done-when
89 SURVIVED       R2-4 the external verdict does not restart the lease
110 SURVIVED      R3-5 a fixture skill tries to land
116 SURVIVED      R3-9 a filing with no task says banked
152 SURVIVED      R5-2 a fault leaves the Lead's lease unsettled
156 SURVIVED      R5-4 a run on an order that ended is a fault
   (18:24:45 ET; sha256 4094ab084d0f4376…)
```

So all 10 still survive on the release candidate (they are not a `5184978` artefact). Each one is a **second-defence mutant**: the rule it breaks is still enforced by another guard in the candidate, which is why no test can observe the mutation. I verified each shadowing guard in the candidate source (paths relative to `package/src/`; line numbers from the `0769797` worktree). **No survivor is a real gap → no regression test was added** (a test for an equivalent mutant would be theatre; Job 06 item 3's "one regression test per verified high" binds the five highs, not equivalent mutants).

| # | Survivor (mutated rule) | Verdict | Shadowing defence verified in `0769797` (one line) |
|---|---|---|---|
| 23 | roll-up without final task | **equivalent** | `final === undefined` is unreachable: decomposition inserts the harness-owned final task (`orders/tasks.js:147`), `tasks_one_final` makes it unique (`orders/schema.js:155`), and it is non-replaceable (`tasks.js:104,111`); an order with no tasks is caught by the `tasks.length === 0` guard. |
| 24 | roll-up with unfinished tasks | **equivalent** | shadowed by the banking guard (`orders/orders.js:432`): only a done task can be banked — `markBanked` refuses non-done (`orders/tasks.js:796`), the wire pre-checks done (`orders/banking.js:27`), and the final banks only on its done transition (`tasks.js:467`). |
| 41 | maker records verdict | **equivalent** | the verdict path requires the check-window holder first (`tasks.js:534` `requireHolder` → `orders/leases.js:14` claimant+token), and `beginCheck` refuses maker-as-checker and sets `claimant = checker` (`tasks.js:441,449`); a maker can never be that holder. |
| 42 | double checker verdict | **equivalent** | the DB `UNIQUE (task_id, round, source)` (`orders/schema.js:205-212`) makes the second insert raise, and `insertVerdict` maps it to the same `verdict-exists` (`tasks.js:512-518`); after the first verdict the task also leaves `checking` (`not-checking`, `tasks.js:533`). |
| 70 | R2-1 any checker pass rolls up a command done-when | **equivalent** | R5-9 refuses any non-finisher checker on a command-mode final (`tasks.js:442-445`; `FINISHER_CHECKER` `orders/kit.js:277`), so a checker pass on a command final is always the finisher's and `finisherPassed` (`orders/orders.js`, rollUp) is the same predicate. |
| 89 | R2-4 the external verdict does not restart the lease | **equivalent** | both branches that change state settle the lease themselves — `applyFail` (`tasks.js:495`), `markDone` (`tasks.js:477`); the remaining branch (external pass before the checker's pass) changes no lease input (verdict + post only), so the removed settle is redundant. |
| 110 | R3-5 a fixture skill tries to land | **equivalent** | `land()` refuses fixtures itself (`lessons/lessons.js:475`, `land-refused`), and the caller only lands promoted non-`lesson` rows; the mutated line is a second defence. |
| 116 | R3-9 a filing with no task says banked | **equivalent** | `banks` only gates the banking notice (`lessons.js:271,289`); with no task the board handler gets `orders.getTask('') === null` and banks nothing (`orders/banking.js:42-45`), and the answer's `banked` is board-derived and false for `task === ''` (`lessons.js:297-300`). |
| 152 | R5-2 a fault leaves the Lead's lease unsettled | **equivalent** | the fault's state change emits a `transition` notice naming the order (`orders/kit.js:183-192`), and the policy's level-triggered settle runs on every such notice (`orders/store.js:125-141` `watchOrders`; `policy/port.js:42`), so `dropFinal`'s explicit settle is redundant. |
| 156 | R5-4 a run on an order that ended is a fault | **equivalent** | ending an order closes every live task and check window (`orders/orders.js:315-319` `endTasks`, called by `failRow`; `orders.js:434` for `done`), so the finisher's token clause (`orders/finisher.js:220,232-233`) sees the ended window first and `windowClosed` is true without the TERMINAL check. |

This matches the mutation harness's own header (`package/src/orders/test/mutate.py`, lines 4–16), which already documents these ten as known equivalent survivors and records the R5 additions. The open question is only whether the **gate report** may cite that header as the disposition or must re-derive it per survivor — put to Nox in T4 (§4).

**Suite run after the triage** (unmutated candidate copy, same harness, same suites + walker range — the control the mutation runner requires):

```
$ TMPDIR=/tmp/x MUTATE_WORKERS=1 python3 -I test/mutate.py control
-1 PASS           control (no mutation)
   (18:26:11 ET; sha256 7e0cd33915a1c6a6…)
```

No test files were modified anywhere (nothing to add: 0 verified gaps). Candidate's own new mutant rows (1 orders + 3 policy, added by `0769797`) are **not** covered by this triage: they need Nox's full candidate mutation run (his in-flight run), and the earlier narrowed safe subset killed 27/27 selected mutants (`task/mission1/targeted-mutants.txt`).

## T2. Stage 2 gate report — DRAFT skeleton

Written to **`board/reports/stage2-gate-report-FINAL.md`**. One row per gate leg with evidence path + timestamp + verdict (`PASS` / `FAIL` / `UNVERIFIED — reason` / `PENDING — Nox's candidate run`), the open-leg list, the seed-1546 disposition text the Lead approved, the client-suite environmental disposition and exactly what would make it verifiable, the five-high no-valid-verdict wording, and the service-state unverifiable-with-reason wording. It carries a **DRAFT — NOT A PASS** banner in the title and header and says explicitly: no row becomes PASS until (a) Nox's independent candidate run lands where a row depends on it, and (b) a fresh-context checker **with veto** certifies the report. This seat never certifies it.

## T3. Job 07 — Stage 3 red-team package: assembly plan (gap list, no writes)

**Nox's item-7 spec** (`board/probes/nox-rulings-20261031T215204Z.md` §3): the package at `evidence/stage-3/redteam-package/` with a one-page index; the ten attacks (a)–(j) **as run on the real install**, each with command, refusal observed, no-execution evidence; the grant legs (single use, 10-minute expiry, wrong args/agent/order, Tier B labelled Tier A, no relay path); guard-ordering leg 3.3; action-log chain + fail-closed 3.4; ledger legs 3.5; the reviewers' attack scripts with findings and fixes; the known-limits list as written. Verdict issues against that index line by line.

**What exists today** (observed):
- `evidence/stage-3/` contains only `build/` (11:33–12:07 ET: release manifests, dist byte-check, npm-pack, signal-scope lint, before/after hashes). **No `redteam-package/`, no index, no per-attack records.**
- `checks/stage3-check.sh` exists and already encodes the legs: `3.1a`–`3.1j` (the ten attacks), `3.2a`/`3.2a-agent`/`3.2a-label`/`3.2a-nogrant`/`3.2a-replay`, `3.2e-confirm`, `3.2-fresh`, `3.2g-owner`, `3.2g-owner-confirm`, `3.2-noexec`, `3.2-polkit`, `3.2-tb-expiry`, `3.2-tb-expiry-confirm`, `3.3a-c`, `3.4a-c`, `3.5a-d`/`3.5c-ctl`/`3.5d-usage`; the tty run is described in its header (up to 9 prompts). On `next` the script is +160 lines vs main; the runnable version must be the one from the merged/0.2.2 tree.
- Reviewer attack scripts and findings: `evidence/review-results/review_{liveness,security,contract}.txt` and `verify_*.txt` exist; the reviewers' original repro scripts are in volatile `/tmp/claude-1000/...` and are **not** in the repo (RESUME says copy before /tmp clears) — the narrowed repro batch produced no valid verdict (`task/mission1/READY-FOR-RECHECK.md`).
- Design/known-limits source material: `docs/RESUME.md` ("what the Stage 3 materials still need"), `docs/ASSUMPTIONS.md`, `docs/fleet-plugin-design.md` §7; the historical scratch Stage 3 adapter is **not** an unchanged-gate pass (Outis audit).

**Gap list, producer, sequence** (each item named with the blocking dependency):

| # | Piece | Producer | Depends on | Notes |
|---|---|---|---|---|
| 1 | `redteam-package/` dir + `INDEX.md` one-pager, line-numbered per Nox's spec | Team B (draft) | — | Can be written as a template now; contents stay empty until runs exist |
| 2 | The ten attacks (a)–(j) **on the real install**: command, refusal code/message, no-execution evidence | Team B runs `checks/stage3-check.sh`; fresh checker witnesses | **Job 06 closed**: 0.2.2 packaged, installed, identity-verified; `FLEET_CLI` set; service up | The script already implements the ten; the run is the evidence. One run at a time |
| 3 | Grant legs (single use, 10-min expiry, wrong args/agent/order, Tier B labelled Tier A, no relay) | Team B (same run: `3.2a*`, `3.2e-confirm`, `3.2-tb-expiry*`) | #2 | Four of these are the manual/tty legs (Cayleb) |
| 4 | Guard-ordering leg 3.3 (`3.3a-c`) | Team B (same run) | #2 | |
| 5 | Action-log chain + fail-closed 3.4 (`3.4a-c`) | Team B (same run) | #2 | log chain verify needs the service vantage |
| 6 | Ledger legs 3.5 (`3.5a-d`, `3.5c-ctl`, `3.5d-usage`) | Team B (same run) | #2 | |
| 7 | Cayleb's `S3_CONFIRM=tty` terminal run (~10 min, up to 9 polkit prompts) — legs 3.2b, 3.2d, 3.2g, owner policy confirm, one Tier A confirm, expired-ticket confirm (must be refused) | **Cayleb** (prep + operator sheet by Team B) | #2 (materials land first) | Never simulated; verified by its recorded output |
| 8 | Reviewers' attack scripts + findings/fixes | Team B copies the existing scripts from volatile `/tmp` **if still present**, else records them as unavailable with hashes from `READY-FOR-RECHECK.md` | — | Do not rewrite or re-run the blocked broad request |
| 9 | Known-limits list (engines, key visible to an engine's own tools, edge-triggered classes now level-triggered, walker aggregate-envelope claim, client suite, service vantage) | Team B | — | Drafted from `docs/RESUME.md` + gate report |
| 10 | One-page index wired to every artefact path + Nox's line-by-line verdict | Team B → Nox | #1–#9 | Nox's verdict is same-day once the package is complete |
| 11 | Fresh checker with **veto** on the gate package | Lead's spawn | #1–#10 | Tier S-adjacent: pre-signoff before the Stage 3 gate is called closed |

Sequence: **(1) template now → (2–6, 8, 9) after Job 06 install → (7) Cayleb's tty run on the materials → (10) Nox verdict → (11) fresh checker.** Nothing in this table is started before Job 06 closes; no mission-branch writes are needed for any of it (evidence lives on disk; the package is copied in only when the gate report is committed after the gate).

**T3b. Single-plugin adoption map (nine) — repo/branches reality.** Lead asked for this map while Nox pins the canonical list. Starting fact confirmed: **none of the nine exists in the repo or any branch** — `git grep -l` at `main` and `next` returns 0 hits for `dsh-android`, `windows-remote-ssh`, `dsh-remote-web-ui`, `dsh-im`, `dsh-archive-manager`, `sage-mem`, `dsh-api-dashboard`, `dsh-ops-mcp`; `dsh-telegram` appears only in two docs. The only fleet code is the plugin's own layer (`package/src/{core,sandbox,egress,fsguard,orders,lessons,housekeeper,client}`, plus `roster`/`bus` on the `bus` branch), and `package/package.json` lists only those files. Candidate names come from Outis's adoption map (`ENVELOPE-AUDIT-OWNERSHIP.md`, itself labelled "not verified pins") and Nox's 31 Oct sweep; the map is:

| Adoption (Nox's list) | Candidate source (latest known) | In repo/branches | Intended home | Stage |
|---|---|---|---|---|
| Remote web UI layer | `@linxin666/dsh-remote-web-ui` (from `zhu1090093659/dsh-web`), LAN/tailnet only, quick tunnel off | none | vendored bundle row + package `files` | operator-path (after S5, before S9) |
| Messaging bridge | `xmanrui/dsh-im` (one bridge) | none | vendored plugin row | operator-path |
| Session archive manager | `MichengAI/dsh-archive-manager` | none | vendored plugin row | operator-path |
| Subscriptions / engine providers | `V1ki/dsh-plugin-subscriptions` + `lcestou/dsh-oh-my-claude` wrapped; routes OpenRouter/NIM/OpenCode Go/Zen/Antigravity/Claude/ChatGPT | none (fleet's own `src/engines/` also does not exist yet) | `src/engines/` + wrapped rows; Job 10 owns the providers | Job 10 (after S5) |
| sage-mem store | `gezi-wen/sage-mem` (Apache-2.0 to verify) behind quarantine/promotion; no OpenViking | none | vendored plugin row | operator-path |
| Telegram surface base | `hi-wenw/dsh-telegram-channel` (historical v1.2 named `@ashafizullah/dsh-telegram@0.5.1` — **conflict**) | docs only | `src/phone/` on the vendored base; Job 11 | Job 11 (after S5) |
| API dashboard meters | `133563825as-ai/dsh-api-dashboard` (normalise `lane_meter`) | none | vendored plugin row + `lane_meter` contract | operator-path (needs S5 meter rows) |
| Android emulator rig | `ZSeven-W/dsh-android` | none | **device-control adoption → Stage 8 hands scope** (Tier S), not packaging alone | Stage 8 (Job 09) |
| Windows VM guest remote SSH | `pureexe/dsh-windows-remote-ssh` wrapped for the guest | none | **Stage 8 hands scope** (Tier S) | Stage 8 (Job 09) |

Conflicts to name now: (i) the Telegram base has two generations (v1.2 name vs sweep name) — Job 11 cannot pin its vendored base until Nox's canonical list lands; (ii) the two device-control adoptions are new, absent from all job briefs, and are Tier S material — they belong in Stage 8 with per-participant route probes (Job 08 rule), not bolted on at packaging; (iii) the engine providers' route list now includes subscription routes whose terms/feasibility are unverified (Stage 4b design fact 4: unresolved → API-key default, flagged); (iv) the packaging target is an install unit that must also carry the fleet's own modules (hands, phone, digest, engines), so "vendor the nine" is not the whole package — it is the third-party layer of one umbrella.

## T4. Nox questions still open after the 17:52 rulings — TEXT FOR THE LEAD TO SEND

> **UPDATE 18:45 ET (Nox batch-2): ALL FIVE ANSWERED.** (1) canonical adoption list supplied (see `team-b-phase1.md` §10(a) — 10 adoptions + automation row 11); (2) service-state legs ruled **desktop-vantage** — evidence filed at `board/probes/team-b-desktop-vantage-20261031T1847Z.md`; (3) `abfb842` ruled **B — strike from the record, no push**, exact record wording in `team-b-phase1.md` §6; (4) seed-1546 **accepted** as a fixture-completion gap with the walker-raise-own-review + final-tree-green condition (green recorded 18:45 ET in the gate report §2); (5) 10-survivor disposition **approved**. The text below is retained as the record of what was asked; it is no longer a live send list. New live item from the Lead's 18:58 ET steer: Nox must rule **sequencing and base for the Codex engine provider (Stage 4b)** before that build starts.

1. **Canonical single-plugin adoption list.** Cayleb's 17:59 property names nine adoptions (remote web UI, messaging bridge, session archive manager, subscriptions/engine providers, sage-mem, Telegram surface base, API dashboard meters, `ZSeven-W/dsh-android`, `pureexe/dsh-windows-remote-ssh`). Two generations of names collide (v1.2: `@ashafizullah/dsh-telegram@0.5.1`, MCP-SDK facade, adopted GUI engine; 31 Oct sweep: `hi-wenw/dsh-telegram-channel`, `dsh-ops-mcp`, custom hands). Please pin the canonical list with exact versions/commits and where each lives (vendored row vs module vs wrapper), and confirm the two device-control adoptions are scoped into Stage 8 hands with per-participant route probes rather than packaging alone. *Our map of the nine against the repo: none exists today — 0 grep hits in main/next; evidence in §T3b.*
2. **Service-state vantage.** The desktop lane's system-bus queries return `Operation not permitted` (checker and Outis audit). Does your `muse` vantage cover the Stage 2 service-state legs (live `fleet-dsh` state, installed-package identity), so they can be recorded as verified rather than unverified? If yes, the exact command/route and what you will record; if no, we write the legs unverified-with-reason.
3. **`abfb842` ruling.** Live: local `main` = `abfb842` (docs-only, +41 lines, 3 files, author Cayleb 16:12:41 ET), no remote ref contains it; content preserved in `task/mission1/owner-steering-docs.patch`; local worktree clean; FF push verified possible. Choose: (A) push docs-only FF to `origin/main` (recommended, exact commands ready in `team-b-phase1.md` §6), (B) strike the local commit after archiving the patch, or (C) branch+PR with review. No remote write will be made without your word.
4. **Seed-1546 gate wording.** Lead approved recording it as a fixture-completion gap, not a package defect, evidenced by the four-way differential (candidate package + pre-fix walker stays red; pre-fix package + only the walker-raise hunk goes green; deterministic 3/3 red on `5184978`). Confirm the gate report may use that wording, and confirm the walker leg is not PASS until your independent candidate run lands **and** a fresh checker with veto certifies.
5. **10-survivor disposition.** The candidate's 10 orders-mutation survivors are the harness's documented second-defence set; I re-ran all 10 on `0769797` (10/10 SURVIVED) and verified each shadowing guard in current source (table in §T1) — **zero real gaps, zero tests added**. Does the gate report cite that triage as the disposition (plus your candidate mutation run for the new mutant rows), or do you want a different form of survivor record?

---

## Status, spend, next

- **T1** complete (10/10 equivalent, evidence quoted; 0 tests added by design).
- **T2** draft skeleton written, clearly marked DRAFT/NOT-A-PASS.
- **T3** plan + gap list + adoption map complete; no writes to mission branches.
- **T4** five questions drafted as text for the Lead.
- Spend **$0**. No gate passed, no job closed, nothing self-certified. Next: consume Nox's candidate verdict → fill the gate-report rows → fresh checker with veto → Job 07 (Stage 3 package) only after the gate closes.
