# Team A — batch 7 report (resumed session; 2026-11-01 ~23:35Z → ~23:55Z)

Lane: Team A / `ergon-lead` (Jobs 01–05). Reports to: the Lead. Spend this batch: **$0** (no purchases, no model calls, no
generation service, no credential handling, no image reads). Priority for this batch was reset by the Lead on Nox ruling
`20261101T233413Z` §1: execute the simulation-bank fold-in **before** the parts-bin shortlist; both are done.

**Status summary**

| Item | Status | One-line evidence |
|---|---|---|
| **P0 Simulation-bank fold-in (Nox Option A)** | **EXECUTED — PR #14 open, head frozen `dae4bd6`, awaiting a fresh checker (Lead spawns)** | branch cut from `main` `c3d86a5`; 21 files, +4,977, 0 deletions; validate OK; harness plan/self-test/verify PASS; exit-codes 45/45; package/docs tests 19/19 |
| **A1 Parts-bin shortlist** | **DELIVERED (proposal only)** | `board/reports/team-a-parts-bin-shortlist.md` — 12 rows, licence line + evidence URL + take/leave + gate path each; nothing admitted/vendored/purchased |
| **A2 Job 02 merge-state check** | **CLEAR — Job 02 (and Pack 19/20 + J2 follow-ups) merged and ancestor of `main`; no open Job 02 follow-ups** | `gh api compare` → `status=ahead behind_by=0` for `c196c25`, `b59b599`, `1c71898`, `686fca8`; checker file says Jobs 01/02 fully closed |
| **A3 Job 03 G1 check** | **G1 NOT satisfied as written; the *Lane-1 artefact* leg is now concrete (installed 0.3.0 + install record + tarball sha) but the Ergon package / fresh host / recorded `dsh` version are missing** | install record `INSTALL-0.3.0: PASS`; tarball sha `8173d13e…`; `first-run-status.json` `"dshVersion": null`; PACKAGE.md §5 item 1 "Absent"; no host |
| **A4 Job 04 Monday prep** | **ASSEMBLED (r2), nothing executed** | `board/reports/team-a-job04-monday-resume-pack-r2.md`; laptop branch `laptop/acceptance-battery` @ `62a277e`, **behind `main` by 90**, ahead 1; no route to laptop |

---

## P0 — Simulation-bank fold-in, Nox Option A (EXECUTED, not merged, not self-certified)

### Ruling applied (verbatim conditions)
Nox `20261101T233413Z` §1: docs-only branch cut from then-current `main`; new `docs/acceptance/simulation/` area ONLY, plus
the two `docs/EXIT-CODES.md` entry-point rows; no edits to `docs/acceptance/demo-company/`, no product code, no runtime;
README boundary text and the BANK-SELF-TEST labels kept verbatim; fresh-context checker re-runs `validate.mjs`, the
harness `plan`/`self-test`/`verify`, and the exit-codes/package-contents tests before merge. The ruling moves neither the
RUN nor the package gates; the simulation-bank proof runs (DeepSeek acting agent + one Haiku comparison pass) remain
gated before any RHL deployment claim — tracked here, not waived.

### Git state (verified this batch)

| | |
|---|---|
| Base | `main` **`c3d86a5ec60cb328c433eaf546019de9f4a92a34`** (`Merge pull request #13 …`), fetched explicitly: `git fetch origin main` → `origin/main = c3d86a5ec60cb…` |
| Branch | **`docs/l5-simulation-bank`**, cut from `origin/main` (`git switch -c docs/l5-simulation-bank origin/main`) |
| Head (frozen) | **`dae4bd698d31b41a280517dce2d6c1577511c347`** (`dae4bd6`), committed 2026-11-01T23:45:11Z |
| Remote head | `git ls-remote origin refs/heads/docs/l5-simulation-bank` → `dae4bd698d31b41a280517dce2d6c1577511c347` (push was `* [new branch]`) |
| PR | **https://github.com/cayleb-james2008/ergon/pull/14** — `state=open`, `draft=false`, `mergeable=true`, base `main@c3d86a5`, head `docs/l5-simulation-bank@dae4bd6`, `changed_files=21`, `additions=4977`, `deletions=0`, created `2026-11-01T23:46:24Z` |

Diff scope proof:

```
$ git diff --name-status origin/main..HEAD | wc -l
21
$ git diff --name-only origin/main..HEAD -- docs/acceptance/demo-company | wc -l
0
$ git diff --shortstat origin/main..HEAD
 21 files changed, 4977 insertions(+)
```

Landed: 20 added files under `docs/acceptance/simulation/` (`SPEC.md`, `README.md`, `batch-plan.md`,
`divergence-cases.json`, six `companies/<id>/fixture.json`, `cross-company/collisions.json`, `validate.mjs`, `harness.mjs`,
seven `self-test-output/BANK-SELF-TEST-*` files) + `docs/EXIT-CODES.md` (two rows only).

### Provenance and preservation

- Copied with `cp -a` from the workspace bank `board/probes/team-a-simulation-programme/`; a `sha256sum` diff of source vs
  landed files (excluding the one edited README) is **empty — 19/19 byte-identical**; `batch-plan.md` is byte-identical too
  (`fd32983a…` both sides). The landed set is **20/20 files**.
- **Scope note (stated, not smoothed):** the Option-A paper enumerated the fixture/validator set and did not name
  `batch-plan.md`, but the shipped `validate.mjs` reads it in its secrets/PII pass (line 240 — `allFiles.push('SPEC.md',
  'README.md', 'batch-plan.md', 'divergence-cases.json')`). Dropping it makes the validator throw `ENOENT` (observed), so it
  is included **unmodified** for the validator to run exactly as written. Nothing was deleted from the workspace bank.
- **Only content change in the landed set:** `README.md` — the `Use` command path now reads
  `node docs/acceptance/simulation/validate.mjs` (was the workspace path), plus an appended **fold-in record** section below
  the boundary section.
- **Boundary text kept verbatim** (checked by extracting the source `## Boundary` section and the landed one and diffing):
  `BOUNDARY PARAGRAPH VERBATIM OK`. **Labels kept verbatim:** all **7/7** self-test artifacts carry `BANK-SELF-TEST` in the
  file name and the "NOT the G1 run … not evidence" label in content.
- Provenance: each landed file's sha256 is in the PR body and in PR #14's description (`docs/acceptance/simulation/*`).

### Verification run by the maker on the frozen head (checker must reproduce; maker never certifies)

| # | Command | Result (quoted) | When |
|---|---|---|---|
| 1 | `node docs/acceptance/simulation/validate.mjs` | exit 0 — `SIMULATION BANK OK: 6 companies, 80 cast members, 12 chains, 28 divergence cases, 2 cross-company collisions (company sizes 6-22); 6/6 chain kinds, 12/12 divergence kinds covered; token-blind separation 239/240; secrets/PII scan clean` | 23:44Z |
| 2 | `node docs/acceptance/simulation/harness.mjs plan --company harborline-plumbing --out /tmp/bank-plan` | exit 0 — `plan written: /tmp/bank-plan/BANK-SELF-TEST-plan-harborline-plumbing` | 23:44Z |
| 3 | `node docs/acceptance/simulation/harness.mjs self-test --out /tmp/bank-selftest` | exit 0 — `bank self-test written: /tmp/bank-selftest/BANK-SELF-TEST-batch-run (76 acts, 6 companies)` | 23:44Z |
| 4 | `node docs/acceptance/simulation/harness.mjs verify --out /tmp/bank-selftest` | exit 0 — `BANK SELF-TEST VERIFY: PASS — 4 files, every file labelled and named BANK-SELF-TEST` | 23:44Z |
| 5 | negative proof: copy of (3), `sed` strips the label from `BANK-SELF-TEST-summary.json`, `verify` | **exit 1** — `BANK SELF-TEST VERIFY: FAIL (1) - …: missing the BANK SELF-TEST label` (fail-closed proven) | 23:44Z |
| 6 | `node --test tests/exit-codes.test.mjs` | **45 tests, 45 pass, 0 fail** (both new entry points found and tabled; no unlisted entry point) | 23:45Z |
| 7 | `node --test tests/package-contents.test.mjs tests/package-manifest.test.mjs tests/docs-truth.test.mjs` | **19 tests, 19 pass, 0 fail** | 23:45Z |
| 8 | `node scripts/lint.mjs` | `852 files checked, 0 findings` | 23:45Z |
| 9 | `node scripts/test-all.mjs` on branch `dae4bd6` vs base `c3d86a5` in a throwaway worktree | both: `TOTAL 1487 cases: 1476 pass, 6 fail, 0 cancelled, 5 skipped, 0 todo; 158/158 checks ran`; **failing-name sets identical** at base and head → **0 branch-attributable**. The 6 are pre-existing environment failures on this seat (installed OpenClaw policy/validator ×3, Twilio runtime ×1, `outis-live-smoke` CLI ×1 — needs the opencode binary). Not caused by, and not claimed by, this docs-only change. | 23:45–23:47Z |

Note on the suite: the seat does not provide the documented gate environment (no OpenClaw install/opencode binary), so the
full-suite number is **not** presented as a green suite; the targeted tests in items 6–7 are the ruling's required
reproduction set and they are green.

### Checker handoff (fresh-context checker — the Lead spawns it; this lane does not self-certify)

Clean checkout of **`dae4bd6`** (throwaway clone/worktree; no repo writes needed):

```bash
git fetch origin && git switch --detach dae4bd6
git diff --name-only c3d86a5..dae4bd6 | wc -l                            # expect 21
git diff --name-only c3d86a5..dae4bd6 -- docs/acceptance/demo-company    # expect empty
node docs/acceptance/simulation/validate.mjs                            # exit 0 + SIMULATION BANK OK: … line
OUT=$(mktemp -d)
node docs/acceptance/simulation/harness.mjs plan --company harborline-plumbing --out "$OUT/plan"
node docs/acceptance/simulation/harness.mjs self-test --out "$OUT/self"
node docs/acceptance/simulation/harness.mjs verify --out "$OUT/self"    # exit 0, PASS 4 files
node --test tests/exit-codes.test.mjs                                   # expect 45/45
node --test tests/package-contents.test.mjs tests/package-manifest.test.mjs   # expect pass
```

Preservation checks for the checker: (a) the landed `README.md` boundary paragraph is byte-identical to the source bank's
(§Boundary, non-blank lines diff empty); (b) every file under `self-test-output/` has `BANK-SELF-TEST` in its name and the
"not the G1 run … not evidence" label in content (7/7 now); (c) the two `docs/EXIT-CODES.md` rows exist and tabled codes
match the scripts' explicit codes (the test proves it both ways).

**Not claimed:** the PR is **not merged**; no fresh checker has run it; the merge decision is the Lead's. Retire the throwaway
base worktree `/tmp/ergon-base-c3d86a5` before any further suite comparison.

---

## A1 — Parts-bin shortlist (delivered; proposal only)

Deliverable: **`board/reports/team-a-parts-bin-shortlist.md`** (standalone, on top of the annex
`team-a-design-pack-ui-library-parts-bin-annex.md`). 12 rows — 8 Ergon Midnight Slate (S1–S8), 4 fleet-pack provisional
(S9–S12) — each with licence line + evidence URL, pack, TAKE (interaction pattern), LEAVE (aesthetics colliding with the
unslop gate / Midnight Slate flat discipline) and the curation-gate path **proposal → all-states check → screenshot compare
→ fresh checker**. Headline licence lines: Vengeance UI **MIT CLEAN** (re-fetched raw `LICENSE` HTTP 200 this batch);
Animate UI **MIT + Commons Clause, in-app only** (re-fetched `LICENSE.md` HTTP 200; Commons Clause bars redistributing the
components as a pack — Lead ruling R1); Uiverse **MIT per element CLEAN with re-checkable provenance** (R2); Forge UI
(forgeui.in, **not** Roblox ForgeGUI) **usage-only, patterns only** (R4); Uilora **commercial, not usable, shape study
only** (R3). Gate state for every row: **G1 only** — nothing admitted, vendored, purchased or committed; S7/S8 need the R2
provenance record; S9 is blocked by R1 if the fleet pack is ever distributed as a components bundle. Ergon rows stay behind
the owner's grading of the design comps (the component-layer admission is the next build step after grading, not this batch).

---

## A2 — Job 02 merge-state check (CLEAR)

**Is Pack 19 merged to `main`? YES — and so are Pack 20, Job 02 and its J2 follow-ups.** Ancestry proven from the remote
this batch (each short SHA compared against current `main` `c3d86a5`):

```
$ gh api repos/cayleb-james2008/ergon/compare/<sha>...main --jq '"status=\(.status) ahead_by=\(.ahead_by) behind_by=\(.behind_by) merge_base=\(.merge_base_commit.sha[0:12])"'
c196c25 -> status=ahead ahead_by=26 behind_by=0 merge_base=c196c2574e38   # Job 02 merge (Pack 21 + scale-up + residuals)
b59b599 -> status=ahead ahead_by=57 behind_by=0 merge_base=b59b5990c594   # Pack 19 integrated (the Job 02 brief's 19+20 precondition)
1c71898 -> status=ahead ahead_by=39 behind_by=0 merge_base=1c7189866b41   # Pack 20 merge (Job 01)
686fca8 -> status=ahead ahead_by=24 behind_by=0 merge_base=686fca8d5fba   # J2-F1/F2/F3 follow-ups merge
```

`status=ahead` with `behind_by=0` and `merge_base == <sha>` means each commit is an ancestor of the remote `main` tip. The
checker's own post-merge records say the same: `board/reports/ergon-checker-postmerge-job02.md` — merge commit `c196c25`,
parents `1c71898` + `4387b51`, merged tree green (suite 1,616/1,607/0 fail/9 skipped/exit 0, 0 branch-attributable);
`board/reports/ergon-checker-postmerge-j02-followups.md` — main `@686fca8`, "**Jobs 01 and 02 are fully closed, including
their follow-ups** … **Ergon `main` is checker-proven**".

**Job 02 next executable step:** there is **none inside Job 02** — its done-condition is met and merged: Pack 21
(E-2/E-6/E-1/S-2/S-4 + F-2/F-4/F-5 dispositions) + template-hardening scale-up built with a full batch run reported
(8 companies / 6 sectors / 1–2 week horizons → 1,768 acts, 56 simulated days, 0 live acts, 72 stand-in-limit findings,
1 template candidate TC-1 as a proposal only; `evidence/batch-standin/batch-report.md`), residuals (a) `scripts/check.mjs`
**retired and executed** (`33ea4f9`, 45/45 exit-codes tests) and (b) migration-gate doc sentence + declared pre-deployment
preflight landed (`bb7900c`, 51/51). The successor that consumes it is **Job 03** — and its next executable step remains
**blocked on G1** (§A3), not on Job 02.

**Open Job 02 follow-ups: none.** `ergon-checker-postmerge-j02-followups.md` records J2-F1/F2/F3 into `a61fdd5` (merged
`686fca8`) and PASS on the post-merge proof; its only "not closed" content is the Job 03 gap list (package, work-PC fold-in,
host), which belongs to Job 03. The two non-blocking corrections in the BOARD Job 03 row (preflight "no network calls"
wording; cut the next branch from then-current main) are Job 03 docs touches, not Job 02 work — **and the second one was
followed in this batch** (branch cut from then-current `main` `c3d86a5`).

---

## A3 — Job 03 G1 check (PREP ONLY; RUN not started)

**Verdict: G1 is NOT satisfied as written.** The installed `@cayleb/dsh-fleet` 0.3.0 gives the *Lane-1 artefact* leg real,
quotable evidence — but three of G1's conditions are still missing. G1's text (RUN-SHEET.md): "**Installed output ready** —
the ruled DSH-era Ergon package is installed on the fresh host by Lane 1's ruled path, and Lane 1's install record exists —
install record + package tarball sha256 + `dsh` version".

What is true now (evidence):
- **Install record exists** (Lane-1 leg): `/home/cayleb/workspace/harness-build/evidence/strike/s3-install-0.3.0/install-20261101T230709Z.log` — last line **`INSTALL-0.3.0: PASS`**; `plugin-add` went `@cayleb/dsh-fleet 0.2.2` → `0.3.0`; `READBACK version=0.3.0 files=104 tarball=104 missing=0 extra=0`.
- **Tarball sha256:** `cayleb-dsh-fleet-0.3.0.tgz` = **`8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070`** (installed identity, R0 read: `installed_version: 0.3.0`, `bytes: 287772`, `verified: true`, source = journald install audit line `2026-11-01T18:07:09-05:00 outis-desktop dsh-fleet-admin[510715]: plugin-add cayleb-dsh-fleet-0.3.0.tgz sha256=8173d13e… via=sudo`).
- **Finish-line evidence:** S4B installed-outcome **ACCEPT 60/60** (`board/reports/strike-checker-s4b-stage2-installed.md`, STAGE 2: PASS, exit 0) and the Ergon rehearsal on the installed 0.3.0 **S8-030 ACCEPT 113/113** (`board/reports/strike-checker-verify-s8-030.md`; R0 identity 0.3.0, 4 clean / 4 pending-named, 0 failed). S5-installed also ACCEPT. Installed unit tonight: `fleet-dsh` active, `MainPID 518634`, `InvocationID bc98f82d…`.

What is missing (why the gate is not satisfied):
1. **"The ruled DSH-era Ergon package"** — what is installed is the **fleet/platform plugin** `@cayleb/dsh-fleet` 0.3.0, not Ergon-as-a-plugin (the three-piece package: management software + DSH baseline + Ergon plugin). `docs/acceptance/demo-company/PACKAGE.md` §5 item 1 still says **"Absent"**; `main`'s `package.json` is still the OpenClaw 1.5.5 baseline. Job 03's package build is blocked on Nox's C1–C6 freeze + W1 ownership ruling.
2. **"On the fresh host"** — the 0.3.0 install is on **`outis-desktop`** (this desktop, as user `cayleb`), not a fresh cloud host. `R0-PROVISIONING.md`: "**No host exists yet, no purchase, no spend, no credentials handled.**"
3. **"`dsh` version"** — the install record's own status has **`"dshVersion": null`** (`…/s3-install-0.3.0/first-run-status.json`), and `dsh --version` from this shell returns `[dsh] Harness CLI not found. Please run DeepSeek Harness Desktop to install it first.` The package does pin the contract in metadata — `engines.dsh = "0.2.2-alpha.2"` — but the *running harness version* is not recorded anywhere G1 can quote.

So: the **Lane-1 install leg is now dischargeable**, but G1 as written needs (a) the Ergon-on-DSH package artefact, (b) a
fresh host, (c) a recorded harness version — or a Nox ruling that the proven 0.3.0 platform install is the installed output
G1 means (only Nox can make that ruling; this lane does not).

### Gates G2–G7 + C1–C6/W1 status (one table)

| Gate | Text (RUN-SHEET.md) | Status tonight | Evidence / missing piece |
|---|---|---|---|
| **G1** | Installed output ready (Lane 1) | **PARTIAL — not satisfied** | install record + tarball sha + 0.3.0 install proven, **but** Ergon package absent, not on a fresh host, `dshVersion: null` |
| **G2** | Contract frozen — C1–C6 confirmed by Nox | **PENDING Nox** | CONTRACT-C1-C6-PENDING.md: every C row "PENDING Nox"; nothing in force |
| **G3** | Client-shape arbitration closed (ADD-only `main[key]` W1 vs documented reshape + C4 page) | **PENDING Nox** | CONTRACT-C1-C6-PENDING.md §arbitration: both paths "PENDING — do not build" |
| **G4** | Host + company data (dedicated host imaged, DNS/URL, Supabase project created, config by secure route) | **ABSENT** | R0-PROVISIONING §1: "No host exists yet, no purchase, no spend, no credentials handled"; no host fingerprint / URL record / project id anywhere in `board/probes/` (templates only) |
| **G5** | Fresh-context rehearsal checker named + acceptance battery agreed | **PARTIAL** | S8-030's `checker-verify` is a fresh-context checker that built none of it and ran a 113-check verifier on the installed 0.3.0 (ACCEPT 113/113) — the *acceptance-battery* leg exists; a checker for the **R0–R7 operator run** is still the Lead's to name |
| **G6** | Run config accepted — models + sole route pinned | **PASS (re-verified this batch)** | `node docs/acceptance/demo-company/fixtures/check-run-config.mjs` → exit 0, `accepted: employeeModel claude-haiku-5-5, agentModel claude-haiku-5-5, route budget-proxy` (2026-11-01T23:41Z) |
| **G7** | No secrets in the pack — evidence rules hold; host secure values entered by owner | **PASS at pack level (prep)** | bounded pattern scan of `docs/acceptance/demo-company/**` (Anthropic/OpenAI/GitHub/AWS keys, private-key blocks, Slack/phone patterns) → **0 hits**; the evidence rules/no-secrets sections exist (RUN-SHEET §"Evidence pack layout", R0-PROVISIONING P9). The host-side half can only be true once values are entered at R0 — not claimed |
| **C1–C6** | Contract freeze set | **ALL PENDING Nox** | CONTRACT-C1-C6-PENDING.md table; nothing applied, no build |
| **W1** | ADD-only `main[key]` view vs documented reshape | **PENDING Nox — do not build** | same file, arbitration table |

**Fact-base flag for Nox's C2 ruling:** C2's written premise ("the 0.2.2 package is not installed; `fleet-dsh` still
0.2.1") is now stale — the installed, checker-proven release is **0.3.0** (`engines.dsh` still `0.2.2-alpha.2`, `dshVersion`
unrecorded). The re-pin decision should be taken against the 0.3.0 reality, not the 0.2.2-text premise. The RUN stays
unscheduled; nothing started.

---

## A4 — Job 04 Monday prep (assembled; nothing executed)

Deliverable: **`board/reports/team-a-job04-monday-resume-pack-r2.md`** (refreshes the 2026-10-31 handoff). Key current facts:
- Laptop branch **`laptop/acceptance-battery`** tip **`62a277e7b8ac7c0cbc9df990a7bdf45afaeda034`** (remote, verified this batch); checkpoint `docs/acceptance/order-e/CHECKPOINT.md` on that branch.
- Branch vs current `main` (`c3d86a5`): **diverged — ahead_by 1, behind_by 90**, merge base `8771371` (it was cut from `8771371`). Any merge must first refresh onto then-current `main`; the refresh route (merge vs rebase) is the Lead's decision, and the repo convention is merge commits/no force.
- Resume order carried exactly: checkpoint §Resume 1–6 (self-tests 40/40 + 19/19; fresh review of the stopped 5-reviewer workflow; `node scripts\test-all.mjs` with the bundled toolchain, baseline 1,529/1,412/92 same 29 checks; commit on the laptop branch; consolidated report; shared-tools refresh after merge) then Job 04 items 6–7 (mobile emulator + cross-device extension; .NET **SC-5** `support/app/ScanSetup.cs` + **FP-5** `SetupWindow.cs`/`ProviderCore.cs`/`ProviderConfigBridge.cs`, with the exact build/check entry points and the "source-text assertions are not a build" rule).
- **No route to the laptop** exists from this desktop; no laptop action taken. Holds (no spend, no credential handling, maker-never-checker, R1 banned, Sonnet 5.5 / Haiku sub-agents policy) are recorded in the pack.

---

## Blocked / successors / queue (unchanged except where noted)

| Lane | State | Blocked on | Successor |
|---|---|---|---|
| Fold-in PR #14 | OPEN, head `dae4bd6`, mergeable | **Lead spawns the fresh-context checker**; merge is the Lead's | checker verdict → Lead merge |
| Job 03 RUN | PREP ONLY | **G1** (Ergon package + fresh host + recorded `dsh` version), **G2/G3** (Nox C1–C6 + W1), **G4** (host purchase), **G5** (run checker named) | the RUN when gates true |
| Design packs / parts bin | PROPOSAL only | **owner grading of the design comps** (standing), then component-layer admission S1–S8 through the gate; fleet rows provisional + S9 R1 boundary | Lead/Nox confirm fleet-pack rows |
| Job 04 | PREP done (r2) | laptop lane dark until Mon 2 Nov 01:00 ET; **no machine route** | laptop session → Lead's checker |
| Job 05 | STANDING | no handback (`laptop` branches exist but 0 kimi/redo — re-checked via `git ls-remote --heads`) | re-check each cycle |
| Simulation-bank proof runs (DeepSeek acting agent + one Haiku comparison pass) | **TRACKED, GATED** | separate proof runs; not waived by the fold-in | before any RHL deployment claim |
| Nox 004500Z Hallmark intake mission | QUEUED — not pulled (Lead: "behind the roster/finish work") | Lead's ordering | next batch |

**Spend:** $0. No purchases, no model calls, no generation service, no credential values read/written, no image reads, no
public posts, no host input. Only files written this batch: this report, the two Team-A reports above, plus the PR branch
contents (all under the ruled scope). The throwaway base worktree used for the suite control run is
`/tmp/ergon-base-c3d86a5` (disposable; remove with `git worktree remove`).

## Artefacts filed this batch

| Path | What |
|---|---|
| `board/reports/team-a-batch7.md` | this report (self-contained) |
| `board/reports/team-a-parts-bin-shortlist.md` | A1 shortlist (proposal only) |
| `board/reports/team-a-job04-monday-resume-pack-r2.md` | A4 refreshed laptop-lane resume pack |
| PR #14 https://github.com/cayleb-james2008/ergon/pull/14 | fold-in (head `dae4bd6`, base `c3d86a5`) |
