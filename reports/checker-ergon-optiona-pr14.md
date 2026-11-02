# Fresh-context checker — Ergon PR #14 (simulation-bank fold-in, Nox Option A)

**Checker:** `checker-ergon` (fresh context; built none of the work). **Seat:** independent checker; maker never checker.
**Subject:** Ergon PR #14 — https://github.com/cayleb-james2008/ergon/pull/14 — branch `docs/l5-simulation-bank`, head
`dae4bd698d31b41a280517dce2d6c1577511c347` (`dae4bd6`), base `main` `c3d86a5ec60cb328c433eaf546019de9f4a92a34`.
**Authority checked against:** Nox ruling `20261101T233413Z` §1 (Option A) as quoted in the Lead's brief and restated
verbatim in `board/reports/team-a-simulation-programme-batch5-foldin-decision.md` §3/§6 ("Conditions on Option A").
**Check window:** 2026-11-01T23:53:21Z → 2026-11-01T23:58:52Z (all reads/runs by this seat).
**Action taken:** read-only verification only. **No merge, no push, no PR edit, no branch edit.** No spend, no model
calls, no credential values, no image reads, no screenshots, no host input injection.

**VERDICT: ACCEPT** — every Option-A ruling condition is met and every required gate reproduces green from a fresh
clone; one non-blocking documentation nit is named in §6 (EXIT-CODES harness row's stream wording), which does not
violate any ruling condition.

`VERDICT: ACCEPT` · `RULING: 20261101T233413Z §1 Option A` · `PR: 14` · `HEAD: dae4bd698d31b41a280517dce2d6c1577511c347` ·
`BASE: c3d86a5ec60cb328c433eaf546019de9f4a92a34` · `CHECKED_AT: 2026-11-01T23:58:52Z` · `CHECKER: checker-ergon`

---

## 1. Ruling conditions vs. what was verified

| # | Option-A condition | Result | Evidence |
|---|---|---|---|
| C1 | Docs-only branch cut from **then-current `main`** | **PASS** | Base `c3d86a5` = `origin/main` at fetch time; commit parent of `dae4bd6` is exactly `c3d86a5`; 1 commit on branch; 0 merge commits. §2 |
| C2 | **New `docs/acceptance/simulation/` area only**, plus the **two `docs/EXIT-CODES.md` entry-point rows** | **PASS** | 21 changed files: 20 added under the new area + `docs/EXIT-CODES.md` modified (+2/−0 rows, nothing else). 0 files outside ruled scope. §3 |
| C3 | **No edits to `docs/acceptance/demo-company/`** | **PASS** | `git diff --name-only c3d86a5..dae4bd6 -- docs/acceptance/demo-company` → empty (0 lines). §3 |
| C4 | **No product code, no runtime** | **PASS** | 0 changed files under `src|scripts|tests|plugins|routing|suite|installer|profiles|support|vendor|…`, 0 `package*.json`, 0 executable/runtime files. Only `.md/.json/.jsonl/.mjs` under `docs/`. §3 |
| C5 | **README boundary text kept verbatim** | **PASS** | `## Boundary` section byte-identical source vs landed (`sha256 038fa2a06eee33d1c2deab95927eeefac8e4f72c3893d23ed52d7e1822b42ab8` both). §4 |
| C6 | **BANK SELF-TEST labels kept verbatim** | **PASS** | 7/7 artifacts named `BANK-SELF-TEST-*`, each carrying the exact label string; 7/7 byte-identical to the workspace bank. §4 |
| C7 | Fresh-context checker re-runs `validate.mjs`, harness `plan`/`self-test`/`verify`, exit-codes + package-contents tests **before merge** | **PASS** | All six reproductions run by this seat from a fresh GitHub clone: validate exit 0; plan/self-test/verify exit 0 (PASS — 4 files); label-stripped negative exits 1; exit-codes 45/45; package tests 19/19. §5 |
| C8 | The ruling **moves neither the RUN nor the package gates** | **PASS (untouched)** | Diff touches no RUN/package-gate file (`docs/acceptance/demo-company/`, package docs, product code all untouched); package tests green. The simulation-bank proof runs remain gated and are **not** waived by this PR. §7 |

**Judgement on the one disclosed scope deviation:** `docs/acceptance/simulation/batch-plan.md` was not named in the
Option-A paper's file enumeration but ships **inside** the ruled new area, **unmodified**. It is required: the shipped
`validate.mjs` reads it in its secrets/PII pass (line 240) and throws `ENOENT` without it (proven, §5.6). The deviation
is disclosed in the PR body and the maker's report. It is within C2 ("new area only"), justified, and not a violation.

---

## 2. Provenance (C1) — commands, results, timestamps

Working dir: `/home/cayleb/Documents/Codex/2026-10-23/task/ergon-phase0/repo` (the Ergon repo; GitHub
`cayleb-james2008/ergon`).

```
$ git fetch origin --prune                       # 2026-11-01T23:53:21Z, exit 0
From https://github.com/cayleb-james2008/ergon
   123bf77..c3d86a5  main       -> origin/main
$ git rev-parse origin/main
c3d86a5ec60cb328c433eaf546019de9f4a92a34
$ git log -1 --format='%H %cI %s' origin/main
c3d86a5ec60cb328c433eaf546019de9f4a92a34 2026-11-01T15:22:40-05:00 Merge pull request #13 from cayleb-james2008/docs/l5-pack-known-limits
```
`origin/main` = `c3d86a5` **before and after** the fold-in (re-checked 23:56:44Z), so `c3d86a5` was then-current `main`.

```
$ git ls-remote origin refs/heads/docs/l5-simulation-bank      # 2026-11-01T23:53:29Z
dae4bd698d31b41a280517dce2d6c1577511c347	refs/heads/docs/l5-simulation-bank
$ git fetch origin docs/l5-simulation-bank:refs/checker/pr14-head   # 23:53:33Z, exit 0
$ git log -1 --format='%H%n%cI%n%an <%ae>%n%s' refs/checker/pr14-head
dae4bd698d31b41a280517dce2d6c1577511c347
2026-11-01T18:45:11-05:00                       # = 2026-11-01T23:45:11Z, matches the claimed freeze instant
ergon-lead <ergon-lead@localhost>
docs(acceptance): fold the rehearsal simulation bank into docs/acceptance/simulation/ (Nox Option A)
$ git log -1 --format='parent=%P' refs/checker/pr14-head
parent=c3d86a5ec60cb328c433eaf546019de9f4a92a34
$ git rev-list --count c3d86a5..refs/checker/pr14-head      # 1
$ git rev-list --merges c3d86a5..refs/checker/pr14-head | wc -l   # 0
```

```
$ gh pr view 14 --json …                        # 2026-11-01T23:53Z and re-read 23:56:44Z
{"state":"OPEN","isDraft":false,"mergeable":"MERGEABLE","baseRefName":"main",
 "baseRefOid":"c3d86a5ec60cb328c433eaf546019de9f4a92a34",
 "headRefName":"docs/l5-simulation-bank",
 "headRefOid":"dae4bd698d31b41a280517dce2d6c1577511c347",
 "changedFiles":21,"additions":4977,"deletions":0,
 "createdAt":"2026-11-01T23:46:24Z","updatedAt":"2026-11-01T23:50:54Z"}
```

**No drift since the freeze:** at 23:56:44Z the remote branch head was still `dae4bd6`, `origin/main` still `c3d86a5`,
PR still OPEN / MERGEABLE / `changedFiles=21`, `headRefOid` unchanged, `updatedAt` unchanged (23:50:54Z — PR-body edit,
not a new commit). `dae4bd6` is not an ancestor of `origin/main` (not merged).

---

## 3. Scope (C2/C3/C4) — commands and outputs

```
$ BASE=c3d86a5ec60cb328c433eaf546019de9f4a92a34; HEAD=dae4bd698d31b41a280517dce2d6c1577511c347
$ git merge-base $BASE $HEAD
c3d86a5ec60cb328c433eaf546019de9f4a92a34
$ git diff --shortstat $BASE..$HEAD
 21 files changed, 4977 insertions(+)
$ git diff --name-status $BASE..$HEAD
M	docs/EXIT-CODES.md
A	docs/acceptance/simulation/README.md
A	docs/acceptance/simulation/SPEC.md
A	docs/acceptance/simulation/batch-plan.md
A	docs/acceptance/simulation/companies/cedar-hollow-clinic-admin/fixture.json
A	docs/acceptance/simulation/companies/ferngrove-school-office/fixture.json
A	docs/acceptance/simulation/companies/harborline-plumbing/fixture.json
A	docs/acceptance/simulation/companies/kelvinside-fabrication/fixture.json
A	docs/acceptance/simulation/companies/meridian-haulage/fixture.json
A	docs/acceptance/simulation/companies/northlight-architects/fixture.json
A	docs/acceptance/simulation/cross-company/collisions.json
A	docs/acceptance/simulation/divergence-cases.json
A	docs/acceptance/simulation/harness.mjs
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-batch-run/BANK-SELF-TEST-README.md
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-batch-run/BANK-SELF-TEST-acts.jsonl
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-batch-run/BANK-SELF-TEST-report.md
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-batch-run/BANK-SELF-TEST-summary.json
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-plan-harborline-plumbing/BANK-SELF-TEST-README.md
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-plan-harborline-plumbing/BANK-SELF-TEST-run-plan.json
A	docs/acceptance/simulation/self-test-output/BANK-SELF-TEST-plan-harborline-plumbing/BANK-SELF-TEST-run-plan.md
A	docs/acceptance/simulation/validate.mjs
```

```
$ git diff --name-only $BASE..$HEAD -- docs/acceptance/demo-company    # empty output
demo-company lines=0
$ git diff --name-only $BASE..$HEAD | grep -vE '^docs/acceptance/simulation/|^docs/EXIT-CODES\.md$'
outside_scope_count=0
$ git diff --name-only $BASE..$HEAD | grep -E '^(src|scripts|tests|plugins|routing|suite|installer|profiles|support|vendor|design|documents|evidence|examples|onboarding|templates|budget|diagnostics)/|^package(-lock)?\.json$|\.(mjs|js|ts|py|sh|ps1|cmd)$' | grep -v '^docs/acceptance/simulation/'
product_code_count=0
$ git diff --raw $BASE..$HEAD | awk '{print $1, $2}' | sort | uniq -c
     20 :000000 100644        # 20 new regular files
      1 :100644 100644        # 1 modified regular file
$ git diff --check $BASE..$HEAD                             # exit 0 — no whitespace errors
extensions: 10 json, 1 jsonl, 8 md, 2 mjs (no binary, no image, no symlink)
```

`docs/EXIT-CODES.md` is exactly the two entry-point rows, nothing else (`git diff --numstat` → `2 0`):

```
$ git diff $BASE..$HEAD -- docs/EXIT-CODES.md
@@ -66,6 +66,8 @@ …
 | `docs/acceptance/demo-company/run-evidence/r0-preflight.sh` | 0, 1, 2 | … |
+| `docs/acceptance/simulation/validate.mjs` | 0, 1, 2 | 0: the simulation bank is valid (`SIMULATION BANK OK`), or `--help`. 1: one or more problems, each named on stdout. 2: usage (an unknown argument); nothing read. |
+| `docs/acceptance/simulation/harness.mjs` | 0, 1, 2 | 0: a run plan or the labelled self-test was written under `--out`, or `verify` passed, or `--help`. 1: `verify` found a missing or misnamed BANK SELF-TEST label, or the named company or the output directory does not exist. 2: usage (no command, an unknown command or option, or a missing `--out`), usage text on stderr. |
```

**Independent GitHub-side cross-check** (`gh pr view 14 --json files`, 23:56:00Z): 21 files, all additions except
`docs/EXIT-CODES.md` (`2+ 0-`); path set identical to the local diff; total `additions=4977`, `deletions=0`.

**PR-body provenance claims verified:** all 20 `sha256` values listed in the PR body under "sha256 of the landed files at
`dae4bd6`" match the actual head files — 20/20 `MATCH` (23:56Z).

---

## 4. Labels and boundary (C5/C6)

Workspace bank source: `board/probes/team-a-simulation-programme/` (20 files; untouched — 0 files modified after
2026-11-01T23:40:00Z, nothing deleted).

**Byte-identity of the landed set** (source bank vs `dae4bd6`, `sha256sum` per file): 19/19 non-README files
`IDENTICAL` — `SPEC.md`, `batch-plan.md`, `divergence-cases.json`, `harness.mjs`, `validate.mjs`,
6× `companies/*/fixture.json`, `cross-company/collisions.json`, and all 7 `self-test-output/**` artifacts. `README.md`
is the only file that differs, by design and as disclosed (two hunks only: the `Use` command path, and an appended
`## Fold-in record` section after the boundary section).

**Boundary text — verbatim proof:** `## Boundary` section extracted from both READMEs, trailing blank lines normalized,
`cmp`:

```
sha256 source: 038fa2a06eee33d1c2deab95927eeefac8e4f72c3893d23ed52d7e1822b42ab8
sha256 landed: 038fa2a06eee33d1c2deab95927eeefac8e4f72c3893d23ed52d7e1822b42ab8
BOUNDARY TEXT VERBATIM OK
NON-BLANK LINES IDENTICAL
```
(The boundary paragraph is unchanged: no edit to `docs/acceptance/` from this bank; fold-in anticipated as a separate
Nox/Lead decision; batch 1 closes nothing on its own.)

**BANK SELF-TEST labels — verbatim proof:** exactly 7 artifacts (4 under `BANK-SELF-TEST-batch-run/`, 3 under
`BANK-SELF-TEST-plan-harborline-plumbing/`); 7/7 file names contain `BANK-SELF-TEST`; 7/7 contain the exact label
string `BANK SELF-TEST — simulation-bank harness output. NOT the G1 run. NOT an acceptance rehearsal. Not evidence.`
(checked with fixed-string grep); 7/7 byte-identical to the workspace bank.

---

## 5. Gates re-run by this checker (C7) — fresh clone, not the maker's worktree

Fresh throwaway clone **from GitHub** (not from the maker's checkout, not the maker's worktree):
`git clone --branch docs/l5-simulation-bank --single-branch https://github.com/cayleb-james2008/ergon /tmp/checker-ergon-pr14`
at 2026-11-01T23:54:02Z → `git rev-parse HEAD` = `dae4bd698d31b41a280517dce2d6c1577511c347`, clean tree (0 dirty
paths), `node v26.7.0`. All commands below run in `/tmp/checker-ergon-pr14`.

**5.1 `validate.mjs` — 2026-11-01T23:54:35Z**
```
$ node docs/acceptance/simulation/validate.mjs
SIMULATION BANK OK: 6 companies, 80 cast members, 12 chains, 28 divergence cases, 2 cross-company collisions (company sizes 6-22); 6/6 chain kinds, 12/12 divergence kinds covered; token-blind separation 239/240; secrets/PII scan clean
validate exit=0
```

**5.2–5.4 harness `plan` / `self-test` / `verify` — 2026-11-01T23:54:41Z** (`OUT=/tmp/checker-ergon/harness-g8hQjW`)
```
$ node docs/acceptance/simulation/harness.mjs plan --company harborline-plumbing --out "$OUT/plan"
plan written: /tmp/checker-ergon/harness-g8hQjW/plan/BANK-SELF-TEST-plan-harborline-plumbing
plan_exit=0
$ node docs/acceptance/simulation/harness.mjs self-test --out "$OUT/self"
bank self-test written: /tmp/checker-ergon/harness-g8hQjW/self/BANK-SELF-TEST-batch-run (76 acts, 6 companies)
selftest_exit=0
$ node docs/acceptance/simulation/harness.mjs verify --out "$OUT/self"
BANK SELF-TEST VERIFY: PASS — 4 files, every file labelled and named BANK-SELF-TEST
verify_exit=0
```
Extra (beyond the required set): `verify --out docs/acceptance/simulation/self-test-output` (the **checked-in**
artifacts) → `BANK SELF-TEST VERIFY: PASS — 7 files, every file labelled and named BANK-SELF-TEST`, exit 0.

**5.5 Negative fail-closed proof — 2026-11-01T23:54:51Z** (label stripped from a copy, repo untouched)
```
$ cp -a "$OUT/self/." /tmp/checker-ergon/negative-xqI2RT/
$ sed -i 's/BANK SELF-TEST//g' /tmp/checker-ergon/negative-xqI2RT/BANK-SELF-TEST-batch-run/BANK-SELF-TEST-summary.json
$ node docs/acceptance/simulation/harness.mjs verify --out /tmp/checker-ergon/negative-xqI2RT
BANK SELF-TEST VERIFY: FAIL (1)
  - BANK-SELF-TEST-batch-run/BANK-SELF-TEST-summary.json: missing the BANK SELF-TEST label
negative_verify_exit=1
```

**5.6 `batch-plan.md` necessity (deviation justification) — 23:55:44Z**, in a copy (`/tmp/checker-ergon/neg-batchplan`):
```
$ sed -n '240p' docs/acceptance/simulation/validate.mjs
allFiles.push('SPEC.md', 'README.md', 'batch-plan.md', 'divergence-cases.json');
$ rm docs/acceptance/simulation/batch-plan.md && node docs/acceptance/simulation/validate.mjs
Error: ENOENT: no such file or directory, open '…/docs/acceptance/simulation/batch-plan.md'
    at read (…/validate.mjs:22:24) … at file:///…/validate.mjs:240:16
validate_without_batchplan_exit=1
```

**5.7 Exit-codes test — 2026-11-01T23:54:35Z**
```
$ node --test tests/exit-codes.test.mjs
ℹ tests 45   ℹ pass 45   ℹ fail 0   ℹ cancelled 0   ℹ skipped 0   ℹ todo 0
exit_codes_exit=0
```
**Sensitivity control** (so the green is not vacuous): in a copy, deleting just the two new EXIT-CODES rows makes the
test fail — 44/45, `✖ every entry point in the repo is in the exit-code doc… AssertionError: entry points missing from
docs/EXIT-CODES.md` — proving the 45/45 run actually exercises both new rows.

**5.8 Package-contents + manifest + docs-truth tests — 2026-11-01T23:54:41Z**
```
$ node --test tests/package-contents.test.mjs tests/package-manifest.test.mjs tests/docs-truth.test.mjs
ℹ tests 19   ℹ pass 19   ℹ fail 0   ℹ cancelled 0   ℹ skipped 0   ℹ todo 0
package_tests_exit=0
```

**5.9 Direct exercise of the two new EXIT-CODES rows' code paths** (the suite does not run their usage cases, so this
seat checked the rows by hand, 23:57Z):
| Invocation | Observed | Row says |
|---|---|---|
| `validate.mjs --bogus` | exit 2, `unknown argument: --bogus` + usage on **stdout**, nothing read | 2: usage (an unknown argument); nothing read — **correct** |
| `validate.mjs --help` | exit 0, usage on stdout | 0: … or `--help` — **correct** |
| `harness.mjs` (no command) | exit 2, usage on **stdout** (163 B stdout / 0 B stderr) | 2: usage (no command …), "usage text on stderr" — **stream claim wrong here** (see §6) |
| `harness.mjs bogus` | exit 2, `unknown command: bogus` + usage on **stderr** | 2: usage (unknown command) — **correct** |
| `harness.mjs verify` (no `--out`) | exit 2 | 2: usage (missing `--out`) — **correct** |
| `harness.mjs verify --out <missing>` | exit 1 | 1: output directory does not exist — **correct** |
| `harness.mjs plan --company no-such-co` | exit 1 | 1: named company does not exist — **correct** |
| `harness.mjs --help` | exit 0, usage on stdout | 0: … or `--help` — **correct** |

**5.10 Hygiene:** independent regex scan over all 21 changed files → **0** secret-like hits (AWS keys, `sk-…`, GitHub/
Slack tokens, private-key headers, JWTs, secret/password/token assignments); **0** non-`.example` e-mail addresses;
`git diff --check` clean; no binary/image files in the diff (so no image reads were needed, and none were made).

---

## 6. Finding — one non-blocking documentation nit (does not violate the ruling)

`docs/EXIT-CODES.md`, the new `harness.mjs` row, ends: *"2: usage (no command, an unknown command or option, or a
missing `--out`), usage text on stderr."* For the **no-command** case the harness prints its usage to **stdout**, not
stderr (`harness.mjs` line 35: `usage()` defaults to `process.stdout`, exit 2). The unknown-command / unknown-option /
missing-`--out` paths do write usage to stderr (line 28–32 `argError`). All three codes (0/1/2) in the row are correct,
and the row exists, is tabled, and satisfies the exit-codes test; the ruling condition ("plus the two
`docs/EXIT-CODES.md` entry-point rows") is met. **Smallest fix if wanted (post-merge docs nit):** reword the stream
phrase, e.g. *"usage text on stderr for argument errors (the no-command usage prints on stdout)"*. This is the only
inaccuracy found; it is not a scope, label, boundary, gate, or runtime issue, so it does not change the verdict.

---

## 7. Independence and limits — what this certification does NOT cover

- **Not verified: Nox's primary ruling text.** No Nox `20261101T233413Z` filing is present on this host. Verification
  is against the ruling **as quoted in the Lead's brief**, corroborated verbatim by the recorded "Conditions on
  Option A" in `board/reports/team-a-simulation-programme-batch5-foldin-decision.md` §3/§6. If the primary text differs
  from that quote, this certification would need re-scoping.
- **Not waived / not verified: the simulation-bank proof runs** (DeepSeek acting agent + one Haiku comparison pass) —
  still gated, untouched by this PR, and not evidence of any RHL deployment claim.
- **Not certified: the RUN (Job 03 R0–R7) or the package gate.** This PR touches neither (diff proof + package tests
  green only show it breaks neither). The ruling explicitly moves neither gate.
- **Not run: full `scripts/test-all.mjs` and `scripts/lint.mjs`** — outside the ruling's required reproduction set; the
  maker reports pre-existing environment failures on this seat, and this checker did not rely on that claim.
- **Not verified: runtime behaviour beyond the documented exit paths** — the bank is input data (no imports, no network,
  no model calls); this was read, not exercised.
- **No merge, no push, no PR/branch/issue edit.** A local checker ref used for fetching was deleted; the Ergon repo
  working tree is unchanged (`main` still at the maker's checkout `123bf77`; only `origin/main` advanced to `c3d86a5`
  by the normal fetch). The verification clone at `/tmp/checker-ergon-pr14` is throwaway and stays at `dae4bd6`.
- **No spend, no credential values, no image reads/screenshots, no host input injection.**

---

## 8. Evidence paths (for re-verification)

| Path | What it is |
|---|---|
| `/tmp/checker-ergon-pr14` | Fresh clone of PR #14 head from GitHub, clean, `HEAD=dae4bd6`; all §5 gates run here |
| `/tmp/checker-ergon/harness-g8hQjW/` | Harness plan + self-test outputs and the `verify` PASS target (4 files) |
| `/tmp/checker-ergon/negative-xqI2RT/` | Label-stripped copy; `verify` exits 1 against it |
| `/tmp/checker-ergon/neg-doc/` | Copy with the two EXIT-CODES rows removed; exit-codes test fails 44/45 (sensitivity control) |
| `/tmp/checker-ergon/neg-batchplan/` | Copy with `batch-plan.md` removed; `validate.mjs` ENOENT at line 240 |
| `/tmp/checker-ergon/pr14-body.md` | PR #14 body; the 20 `sha256` provenance claims verified against head files |
| `/tmp/checker-ergon/bs.txt`, `bl.txt`, `boundary-src.txt`, `boundary-land.txt` | Boundary-section extraction used for the verbatim `cmp` |
| `board/probes/team-a-simulation-programme/` | Workspace bank source of record (20 files, untouched); comparison baseline for §4 |
| `board/reports/team-a-simulation-programme-batch5-foldin-decision.md` §3/§6 | Recorded Option-A conditions used to corroborate the ruling quote |

**One-line reason for the verdict:** docs-only branch cut from then-current `main`, exactly the ruled 20-file new area
plus the two EXIT-CODES rows, demo-company/product code/runtime untouched, README boundary text and all 7 BANK SELF-TEST
labels verbatim, and every required gate reproduced green by a fresh-context checker from a clean clone — one
non-blocking stream-wording nit in the harness EXIT-CODES row is named, nothing ruling-violating was found.

`VERDICT: ACCEPT` · `CHECKER: checker-ergon` · `CHECKED_AT: 2026-11-01T23:58:52Z` · `PR_HEAD: dae4bd698d31b41a280517dce2d6c1577511c347` · `PR_BASE: c3d86a5ec60cb328c433eaf546019de9f4a92a34`

---

## 9. PR #15 delta re-check — the §6 nit fix (follow-up, 2026-11-02T00:06:28Z–00:07:01Z)

**Subject:** Ergon PR #15 — https://github.com/cayleb-james2008/ergon/pull/15 — branch
`docs/l5-exit-codes-harness-usage`, head `cb21688a8b850d5f2f84c7f923246e0cd20f2dbd` (`cb21688`), base `main`
`8822d479ce694b9c569dd13300379b941f0c320a` = the **PR #14 merge commit** (`Merge pull request #14 from
cayleb-james2008/docs/l5-simulation-bank`; parents `c3d86a5` + `dae4bd6`; `git diff dae4bd6..8822d479` = 0 files, so the
base is exactly the accepted baseline). Read-only; no merge/push/edit.

**PR state** (`gh pr view 15`, 2026-11-02T00:06:29Z, re-read 00:07:01Z): `state=OPEN`, `isDraft=false`,
`mergeable=MERGEABLE`, base `main@8822d479`, head `docs/l5-exit-codes-harness-usage@cb21688`, `changedFiles=1`,
`additions=1`, `deletions=1`, created 00:04:06Z, `updatedAt=00:05:09Z`. Remote head `cb21688` and `origin/main=8822d479`
re-read at 00:07:01Z — no drift. Head commit parent = `8822d479`; 1 commit; 0 merge commits.

**Scope:** `git diff --shortstat 8822d479..cb21688` → `1 file changed, 1 insertion(+), 1 deletion(-)`;
`git diff --name-status` → `M docs/EXIT-CODES.md`; outside-`docs/EXIT-CODES.md` count **0**; GitHub API file list
independently = 1 file (`1+ 1- docs/EXIT-CODES.md`). The diff is exactly one row, codes column `| 0, 1, 2 |` unchanged:

```
- | `docs/acceptance/simulation/harness.mjs` | 0, 1, 2 | … 2: usage (no command, an unknown command or option, or a missing `--out`), usage text on stderr. |
+ | `docs/acceptance/simulation/harness.mjs` | 0, 1, 2 | … 2: usage (no command, an unknown command or option, or a missing `--out`); the no-command case prints the usage text on stdout, argument errors print it on stderr. |
```

**Stream behaviour reproduced at `cb21688`** (fresh GitHub clone `/tmp/checker-ergon-pr15`, `HEAD=cb21688`, clean tree;
2026-11-02T00:06:48Z):

| Invocation | exit | stdout | stderr | Corrected row says |
|---|---|---|---|---|
| `harness.mjs` (no command) | 2 | 163 B, first line `Usage:` | 0 B | "the no-command case prints the usage text on stdout" — **MATCH** |
| `harness.mjs bogus` | 2 | 0 B | 186 B, `unknown command: bogus` | "argument errors print it on stderr" — **MATCH** |
| `harness.mjs verify` (missing `--out`) | 2 | 0 B | 181 B, `--out is required` | argument error → stderr — **MATCH** |
| `harness.mjs plan --bogus` (unknown option) | 2 | 0 B | 189 B, `unknown argument: --bogus` | argument error → stderr — **MATCH** |
| `harness.mjs --help` | 0 | 163 B usage | 0 B | row's `0: … or --help` — **MATCH** |

**Exit-codes test at `cb21688`** (2026-11-02T00:06:49Z): `node --test tests/exit-codes.test.mjs` → `ℹ tests 45 ·
ℹ pass 45 · ℹ fail 0`, exit 0.

**Verdict for the delta: ACCEPT** — the PR is exactly the one-row correction of the §6 nit, no other file changed, the
corrected row now matches observed behaviour in every listed case, codes 0/1/2 unchanged, and the exit-codes test stays
45/45. No new findings. Fences honoured: read-only on origin, no merge/push/edit, no credential values, no image reads.

`PR15_VERDICT: ACCEPT` · `PR15_HEAD: cb21688a8b850d5f2f84c7f923246e0cd20f2dbd` · `PR15_BASE: 8822d479ce694b9c569dd13300379b941f0c320a` · `PR15_CHECKED_AT: 2026-11-02T00:07:01Z`
