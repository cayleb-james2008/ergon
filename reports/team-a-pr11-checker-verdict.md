# Team A PR #11 — independent checker verdict

**Checker:** fresh-context, independent (built/staged nothing under review). **Reported:** 2026-11-01T16:49Z.
**Subject:** `cayleb-james2008/ergon` PR #11, branch `docs/l5-run-evidence`, head
`5cde3129f3acce0f309e54c5b7520fd6b3d6a388`, base `5c2eaed70b15384655449c4b5426c7802b2ef39d`, 42 files,
+940/−21. **Maker report read:** `board/reports/team-a-job03-package-assembled.md` (batch 3) and the PR body.

**Overall verdict: PASS — safe to merge as docs-only.** Two non-blocking corrections are recorded below:
(1) the claim that `5c2eaed` is *current main* is stale — current main is `848a41b`, two commits ahead; the PR still
merges cleanly (independently tested). (2) The `r0-preflight.sh` docs claim "no network calls", but the script's
`getent hosts` is a resolver call. Neither affects the docs-only property or merge safety; if either is a hard gate for
the Lead, treat this as HOLD and rebase / fix the wording first.

**Method:** `gh` API reads; a fresh shallow clone in `/tmp/ergon-pr11-verify-20261101T164139Z`; a `git archive` of the
head plus a detached worktree at the head for the git-dependent test; a local `git merge --no-commit` into current main
(aborted, no remote write). No repo or workspace writes except this report. No images read. No merge, push or spend;
`gh` used the existing CLI login, and no secret values were read or printed.

---

## 1. Per-item result

| # | Item | Result | Evidence |
|---|---|---|---|
| 1 | Base/head/ancestry | **PASS** | PR base OID `5c2eaed`; head `5cde3129`; `git rev-parse 5cde3129^` = `5c2eaed`; head is a descendant of base; head is not an ancestor of main; only `origin/docs/l5-run-evidence` contains it. |
| 2 | "Base `5c2eaed` is current main" | **FAIL (factual, non-blocking)** | Current main is `848a41ba2aca2bc73a2bef5ba96237df4e07b97b`; `5c2eaed..main` = 2 commits (`7e33ed6`, `848a41b`), net diff `installer/ui/index.html` only. Head was authored 16:36:31Z; main moved 16:37:44Z; PR created 16:39:41Z. Local merge into `848a41b` is clean (exit 0, 0 conflicts), so this is a stale statement, not a merge defect. |
| 3 | File count and diff | **PASS** | `git diff --shortstat 5c2eaed 5cde3129` → `42 files changed, 940 insertions(+), 21 deletions(-)`; GitHub API `changedFiles: 42, additions: 940, deletions: 21`. |
| 4 | Docs-only, no product/fixture-binary/test change | **PASS** | All 42 paths start `docs/`; 0 binary files (`git diff --numstat` has no `-` rows); no change to `package.json`, `tests/`, `docs/acceptance/demo-company/fixtures/`, `scripts/`, `installer/` or `support/`. Two new standalone tools under `docs/` (`check-skeleton.mjs`, `r0-preflight.sh`), not wired into the product or tests. |
| 5 | PR state | **PASS** | Final snapshot 16:47:29Z: `OPEN`, `MERGEABLE`, `mergeStateStatus: CLEAN`; `test` check success 16:42:51Z, CodeRabbit success 16:40:52Z. The 16:40 first read was `UNSTABLE` only because `test` was still in progress. |
| 6 | Run-evidence skeleton structure | **PASS** | `pass-1/` + `pass-2/`, 8 step dirs each (R0-pre, R1–R6, R7-close), 16 step dirs total, 16 `notes.md`, 4 R7-close files per pass (state-record, host-teardown, pass-diff, notes). |
| 7 | 10 required note fields each | **PASS** | Independent count over all 16 `notes.md`: 16/16 contain all 10 fields (`Step, Pass, Owner, Operator action, Expected, Actual, Word, Evidence files, Timestamp (UTC), Blocker / finding`); 0 missing. |
| 8 | Pass index blocks | **PASS** | Both `pass-1/index.md` and `pass-2/index.md` carry the R0–R7 verdict table (8 rows), `## Secrets check`, and `## Checker sign-off`; both say "This is a skeleton. No pass has been recorded." |
| 9 | `check-skeleton.mjs` | **PASS** | Ran on the head archive/worktree: `SKELETON OK: 2 passes, 16 step directories, 10 required note fields each, secrets scan clean` (exit 0). Negative test (remove one field / inject `password = …`) → `SKELETON PROBLEMS (2)`, exit 1. |
| 10 | `r0-preflight.sh` smoke | **PASS** | 6/6 `PASS`, `R0 PREFLIGHT: PASS`, exit 0 (tool self-test; output dirs discarded, not R0 evidence). |
| 11 | `r0-preflight.sh` fail-closed | **PASS** | No `--repo`/`--package`/project id → 3 named blocking FAILs, `R0 PREFLIGHT: FAIL (3 blocking)`, exit 1. |
| 12 | `r0-preflight.sh` "no network calls" claim | **FAIL (minor, non-blocking)** | The script line 110 runs `getent hosts "$URLNAME"`; on this host that returned public IPs for `example.com` via the resolver (`hosts: … resolve files myhostname dns`). The header and README/R0-PROVISIONING say "no network calls"; DNS resolution is a network call. Fix the wording to "no HTTP/API calls; DNS resolution only" before using this as R0 evidence. |
| 13 | `R0-PROVISIONING.md` G1–G7 / P1–P10 | **PASS** | 7 gate rows G1–G7 with owner; 10 step rows P1–P10, each with owner, command/action, evidence file and pass condition; no-secret, cost/authorisation and stop-rule sections present. |
| 14 | F1 fix in `PACKAGE.md` | **PASS** | §3 now has 22 full 64-character evidence hashes; §2 has 6 full artefact hashes; 28/28 match the files. The old sentence claiming the full values "repeat in `board/reports/team-a-job03-package-assembled.md`" is removed. PR #10 merge commit `6bcd4eb` is recorded in the metadata row and §8. |
| 15 | Governance ADOPT record | **PASS (design-only); provenance UNVERIFIED** | `docs/acceptance/ERGON-PACKAGE-GOVERNANCE-PATTERNS.md` is Markdown only: no code block, no source file, no dependency, no `package.json` change; it explicitly says "DESIGN ADOPTED. CODE NOT ADOPTED. NOTHING IMPORTED, VENDORED OR RUN." The Nox/Lead ADOPT provenance is a relay recorded in the doc and the maker report; it is not independently verifiable from the repo (no Nox ruling file in-tree). |
| 16 | `EXIT-CODES.md` / demo-company README pointers | **PASS** | Two new entry-point rows for `check-skeleton.mjs` and `r0-preflight.sh`; README gains the `R0-PROVISIONING.md` and `run-evidence/README.md` pointer rows; `tests/exit-codes.test.mjs` passes 45/45. |
| 17 | 28/28 full hashes | **PASS** | Inline parser over PACKAGE.md §2/§3 + `sha256sum`: `checked 28 hashes, mismatches 0, unique 28`. |
| 18 | 90 relative links / 28 markdown files | **PASS** | Inline link checker over the 28 changed `*.md`: `checked 90 relative links across 28 markdown files; missing 0`. |
| 19 | Exit-codes test | **PASS** | `node tests/exit-codes.test.mjs` → tests 45, pass 45, fail 0, exit 0. |
| 20 | Package-contents test | **PASS** | `node tests/package-contents.test.mjs` → tests 5, pass 5, fail 0, exit 0. |
| 21 | Evidence-freshness test | **PASS** | In a real git checkout at head: tests 6, pass 6, fail 0, exit 0. (In the plain archive it is 5 pass + 1 skipped because the test needs a git checkout; the maker ran in a worktree, and the git-checkout run is 6/6.) |
| 22 | Docs-truth test | **PASS** | `node tests/docs-truth.test.mjs` → tests 12, pass 12, fail 0, exit 0. |
| 23 | No RUN claim | **PASS** | Every added template is a placeholder (`<fill>`, `@PASS@`, `<templated at filing…>`); README/index say "SKELETON ONLY / no pass has been recorded / the RUN has not started"; the only "run" claims found are negative statements. No added line claims a pass, a host run or observed evidence. |
| 24 | C1–C6 stay PENDING / no unruled contract applied | **PASS** | `CONTRACT-C1-C6-PENDING.md` is unchanged by this PR and remains "RECORDED, NOT APPLIED… PENDING Nox" for C1–C6; the governance record and R0 gates G2/G3 point to the same pending state; no contract row is applied. |
| 25 | No main touch | **PASS** | Head `5cde3129` is not on `origin/main`; `git branch -r --contains` shows only `origin/docs/l5-run-evidence`; main tip `848a41b` is unrelated installer work; PR is still OPEN. No merge/push performed by this checker. |
| 26 | Governance doc scope (Lead's question) | **PASS — acceptable in a docs-only PR** | It is beyond the three named files, but it adds Markdown design only: no code, dependency, build/config or behaviour change; it is topically the same Job 03 governance work and is disclosed in the PR body and maker report. It does **not** break the docs-only property. Splitting it is a review-scope preference, not a correctness requirement. |

---

## 2. Exact re-run commands and results (timestamps UTC)

All commands were run against the head archive or the detached head worktree at
`/tmp/ergon-pr11-verify-20261101T164139Z/head-wt`; the repo was a fresh shallow clone. The only workspace write is this
report; scratch is under `/tmp`.

### 2.1 Skeleton validator

```
$ node docs/acceptance/demo-company/run-evidence/check-skeleton.mjs
SKELETON OK: 2 passes, 16 step directories, 10 required note fields each, secrets scan clean
exit=0
```

- First run 2026-11-01T16:41:54Z; re-run 16:47:30Z. Same output, exit 0.
- Negative control: copied the skeleton to `/tmp/skel-neg-*`, deleted `- Timestamp (UTC):` from
  `pass-1/R2/notes.md`, appended `password = supersecret123` to `pass-1/R3/notes.md`:
  `SKELETON PROBLEMS (2): … missing field "- Timestamp (UTC):" … contains a secret-like assignment`, exit 1.
  Restored copy → exit 0. This shows the validator is not a no-op.

### 2.2 Preflight smoke

```
$ SM=$(mktemp -d); PKG=$(mktemp); echo dummy > "$PKG"
$ printf 'project_id: smoke\nregion: smoke\n' > "$SM/project-id.txt"
$ bash docs/acceptance/demo-company/run-evidence/r0-preflight.sh "$SM" \
    --repo "$PWD" --package "$PKG" --url rehearsal.example.com
PASS  host fingerprint written
PASS  package hashed: dummy2-DZUuFK.tgz d3eb539a…
PASS  run config accepted (route budget-proxy)
PASS  no company state on the checked paths
PASS  url name recorded: rehearsal.example.com
PASS  company project id recorded
R0 PREFLIGHT: PASS
exit=0
```

- 2026-11-01T16:42:48Z and re-run 16:47:47Z; both 6/6, exit 0. This is a tool self-test on the checker host, not R0
  evidence; output dirs discarded.

### 2.3 Preflight fail-closed

```
$ FC=$(mktemp -d)
$ bash docs/acceptance/demo-company/run-evidence/r0-preflight.sh "$FC"
PASS  host fingerprint written
FAIL  package hash not recorded (R0 blocking: pass --package TARBALL)
FAIL  run-config check not run (R0 blocking: pass --repo PATH to the pinned checkout)
PASS  no company state on the checked paths
FAIL  company project id not recorded (R0 blocking: write id + region only, no secret)
R0 PREFLIGHT: FAIL (3 blocking)
exit=1
```

- 2026-11-01T16:42:48Z and 16:47:47Z; both exit 1 with 3 named blocking items. `bash -n` → exit 0.

### 2.4 Hashes (28/28)

Inline Node script (Appendix A.1): parse the 6 artefact rows in §2 and the 22 evidence rows in §3 of
`docs/acceptance/demo-company/PACKAGE.md`, `sha256sum` each referenced file.

```
checked 28 hashes, mismatches 0, unique 28
exit=0
```

- 2026-11-01T16:42:32Z and 16:47:47Z; same result. No truncated hashes remain in §2/§3; the 6 artefact hashes are full
  64-character values.

### 2.5 Relative links (90/28/0)

Inline Node script (Appendix A.2): collect the 28 changed `*.md` files, extract relative Markdown link targets, resolve
them against the file's directory, skip `http(s)`, `mailto:`, `tel:` and `#` anchors.

```
checked 90 relative links across 28 markdown files; missing 0
exit=0
```

- 2026-11-01T16:42:32Z and 16:47:47Z; same result.

### 2.6 Repo tests

```
$ node tests/exit-codes.test.mjs        → tests 45, pass 45, fail 0, exit 0   (16:41:54Z / 16:47:30Z)
$ node tests/package-contents.test.mjs  → tests 5,  pass 5,  fail 0, exit 0   (16:41:56Z / 16:47:30Z)
$ node tests/evidence-freshness.test.mjs→ tests 6,  pass 6,  fail 0, exit 0   (16:42:32Z / 16:47:30Z; git checkout)
$ node tests/docs-truth.test.mjs        → tests 12, pass 12, fail 0, exit 0   (16:41:57Z / 16:47:30Z)
```

The plain `git archive` extraction has no `.git`; `evidence-freshness` skips its one git-dependent test there
(5 pass / 1 skip). Running it in the detached worktree at the same head gives 6/6, which is the maker's claim and the
meaningful result.

### 2.7 Merge test into current main (no remote write)

```
$ git checkout -B tmp-merge-test origin/main       # 848a41b
$ git merge --no-commit --no-ff 5cde3129
Automatic merge went well; stopped before committing as requested
merge_exit=0; conflicts=0
$ git merge --abort; git checkout main
```

- 2026-11-01T16:47:47Z. The 42-file docs change merges into current main with no conflicts. No merge commit was
  created and nothing was pushed.

### 2.8 Secret scan

- `check-skeleton.mjs` scans the run-evidence tree and reports "secrets scan clean".
- A broad regex scan over all 42 changed files (`sk-ant-`, `ghp_`, `github_pat_`, JWT, private key, secret-like
  assignments, `service_role`) found **0 hits**. A diff-line scan only hits the detector's own regex literal in
  `check-skeleton.mjs` (`/service_role[_ ]key/i`), not a credential.

---

## 3. Branch / PR facts (raw)

```
$ gh pr view 11 --repo cayleb-james2008/ergon --json state,mergeable,mergeStateStatus,baseRefOid,headRefOid,changedFiles,additions,deletions
{"state":"OPEN","mergeable":"MERGEABLE","mergeStateStatus":"CLEAN",
 "baseRefOid":"5c2eaed70b15384655449c4b5426c7802b2ef39d",
 "headRefOid":"5cde3129f3acce0f309e54c5b7520fd6b3d6a388",
 "changedFiles":42,"additions":940,"deletions":21}

$ git rev-parse origin/main
848a41ba2aca2bc73a2bef5ba96237df4e07b97b
$ git merge-base --is-ancestor 5cde3129 origin/main && echo YES || echo NO
NO
$ git branch -r --contains 5cde3129
  origin/docs/l5-run-evidence
```

- Head commit: `5cde3129f3acce0f309e54c5b7520fd6b3d6a388`, parent `5c2eaed70b15384655449c4b5426c7802b2ef39d`,
  authored 2026-11-01T11:36:31-05:00 (16:36:31Z) by "Team A (ergon-lead)".
- PR created 2026-11-01T16:39:41Z; CI `test` started 16:39:48Z, completed 16:42:51Z success; CodeRabbit completed
  16:40:52Z success.
- Current main `848a41b` is 2 commits ahead of the PR base; net diff is one file, `installer/ui/index.html`
  (+5/−5). The branch was cut before those commits; the merge test is clean.
- No approving review is recorded (`reviewDecision: ""`); this repo has no branch-protection API available to this
  token (403 on `/protection` and `/rulesets`), and the Lead is the merger. Not a defect for a docs-only PR.

---

## 4. Scope ruling — the governance-patterns doc

The doc is **acceptable in this docs-only PR**. Reasons:

1. It adds a single Markdown design record; no code, dependency, build script, config or fixture is added, and
   `package.json` is unchanged.
2. It is topically the same Job 03 package/governance work the PR already touches; the PR body and the maker report both
   disclose it as a fourth item beyond the three named files.
3. It explicitly says design adopted / code not adopted / nothing imported, vendored, linked or run; the diff confirms
   that (all 42 paths are docs; no new source or dependency).
4. It does not weaken the docs-only property or change the review surface of the three named files.

Splitting it is a **review-scope preference**, not a correctness requirement. If the Lead wants the PR to contain only
the three named files, split it; otherwise it can merge as-is. The one thing not independently verifiable from the repo
is the Nox/Lead ADOPT provenance (the record is a relay; no Nox ruling file is in-tree) — that is marked UNVERIFIED, not
a docs-only violation.

---

## 5. Run / contract / main checks

- **No run claim:** every template is unfilled; the README and both pass indexes say the skeleton is not evidence and no
  pass has been recorded; the RUN is stated as not started. The only "run" references are negative ("has not run",
  "not run", "templates only").
- **C1–C6:** unchanged and still PENDING Nox; no contract row is applied. `CONTRACT-C1-C6-PENDING.md` is not in the
  diff.
- **Main:** head is not on main, only on `origin/docs/l5-run-evidence`; PR is open; no merge or push by this checker.

---

## 6. Findings and disposition

| # | Finding | Severity | Disposition |
|---|---|---|---|
| F1 | "Base `5c2eaed` is current main" is stale: current main is `848a41b` (2 commits, net one installer file). The PR base OID is correct and the merge test is clean. | Minor factual / process | Non-blocking for docs-only merge. If the Lead's merge policy requires base == current main, rebase first (that would be a process HOLD). |
| F2 | `r0-preflight.sh` says "no network calls", but line 110's `getent hosts` is a resolver call (verified: `example.com` resolves to public IPs). | Minor wording / safety-claim accuracy | Non-blocking for merging docs. Fix the wording to "no HTTP/API calls; DNS resolution only" before the tool is used as R0 evidence. |
| F3 | The Nox/Lead ADOPT provenance in the governance doc is a relay, not independently checkable from the repo. | UNVERIFIED | Not a docs-only violation; the Lead is the relay. |
| F4 | In a plain archive, `evidence-freshness` skips 1 git-dependent test (5 pass / 1 skip); in a real git checkout it is 6/6. | Expected test behaviour | No action; the maker's 6/6 claim is correct in a checkout. |

No blocking FAIL. The only FAIL rows are F1 (stale statement) and F2 (minor wording); neither changes the docs-only
property, the file set, the tests, or mergeability.

---

## 7. Overall verdict

**PASS — safe to merge as docs-only.**

Decisive evidence: 42/42 files under `docs/`, 0 binaries, no product/test/fixture/package change; all four repo tests
pass (45/45, 5/5, 6/6, 12/12); skeleton validator passes and fails closed on tampering; preflight passes 6/6 and fails
closed 3/3; 28/28 hashes and 90/90 relative links; PR `OPEN/MERGEABLE/CLEAN` with CI green; local merge into current
main has 0 conflicts; head is not on main; no run claim; C1–C6 still PENDING.

**If the Lead treats either correction as a gate:** HOLD, rebase onto `848a41b` (or re-pin) and correct the "no network
calls" wording; then this becomes a clean PASS with no caveats. Otherwise, merge as-is.

---

## Appendix A — verification scripts

### A.1 28-hash checker (Node ESM, run from the head worktree)

```js
import fs from 'node:fs'; import path from 'node:path'; import crypto from 'node:crypto';
const base = path.join(process.cwd(), 'docs/acceptance/demo-company');
let checked = 0, bad = [], seen = [];
for (const line of fs.readFileSync(path.join(base, 'PACKAGE.md'), 'utf8').split('\n')) {
  const m = line.match(/^\|\s*(?:\[([^\]]+)\]\(([^)]+)\)|`([^`]+)`)\s*\|.*?`([0-9a-f]{64})`\s*\|/);
  if (!m) continue;
  const rel = m[2] || m[3], want = m[4];
  const p = [path.join(base, rel), path.join(base, 'evidence', rel)].find(fs.existsSync);
  if (!p) { bad.push([rel, 'MISSING']); continue; }
  const got = crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
  checked++; seen.push(rel);
  if (got !== want) bad.push([rel, `want ${want} got ${got}`]);
}
console.log(`checked ${checked} hashes, mismatches ${bad.length}, unique ${new Set(seen).size}`);
for (const b of bad) console.log('MISMATCH', ...b);
process.exit(bad.length ? 1 : 0);
```

### A.2 Relative-link checker (Node ESM, run from the head worktree)

```js
import fs from 'node:fs'; import path from 'node:path'; import { execSync } from 'node:child_process';
const root = process.cwd();
const files = execSync('git diff --name-only 5c2eaed 5cde3129 -- "*.md"', { cwd: root, encoding: 'utf8' })
  .trim().split('\n');
let links = 0, missing = [];
for (const f of files) {
  const t = fs.readFileSync(path.join(root, f), 'utf8');
  const re = /\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
  let m;
  while ((m = re.exec(t))) {
    let x = m[1];
    if (/^(https?:|mailto:|tel:|#)/.test(x)) continue;
    x = x.split('#')[0]; if (!x) continue;
    links++;
    const p = path.resolve(path.dirname(path.join(root, f)), decodeURIComponent(x));
    if (!fs.existsSync(p)) missing.push([f, x]);
  }
}
console.log(`checked ${links} relative links across ${files.length} markdown files; missing ${missing.length}`);
for (const x of missing) console.log('MISSING', ...x);
process.exit(missing.length ? 1 : 0);
```

---

## Appendix B — raw log

Full command output for the final batch is at `/tmp/pr11-final-verify.log` (scratch, host-local). Key lines:

```
2026-11-01T16:47:29Z  PR state: OPEN / MERGEABLE / CLEAN; base 5c2eaed; head 5cde3129; 42 files; +940/−21
2026-11-01T16:47:30Z  SKELETON OK: 2 passes, 16 step directories, 10 required note fields each, secrets scan clean
2026-11-01T16:47:30Z  exit-codes 45/45; package-contents 5/5; evidence-freshness 6/6; docs-truth 12/12
2026-11-01T16:47:47Z  checked 28 hashes, mismatches 0, unique 28
2026-11-01T16:47:47Z  checked 90 relative links across 28 markdown files; missing 0
2026-11-01T16:47:47Z  R0 PREFLIGHT: PASS (6/6, exit 0); FAIL (3 blocking, exit 1)
2026-11-01T16:47:47Z  merge into 848a41b: exit 0, conflicts 0
```

**No merge, no push, no branch modification; no spend; no new credentials handled (`gh` used the existing CLI login,
no secret values read); no image reads.**
