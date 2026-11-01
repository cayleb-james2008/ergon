# TEAM E — E5a: ready-to-run execution pack for picks 8-12 (NOTHING EXECUTED)

**STATUS 2026-10-31 20:27 ET - SUPERSEDED FOR EXECUTION. REFERENCE ONLY.**
Nox's 00:20Z manager ruling (relayed by the Lead): Cayleb delegated the held-PR picks and the
repo-consolidation namings to Nox's best recommendation, and **execution is running on Nox's side, not
Team E's**. Nothing below may be executed by Team E. The command set and the 22:24Z baselines stay on
record as *reference* for whoever observes the result; the post-action verification reads remain valid
for reading remote state, never for writing it.

Built 2026-10-31 18:26 ET (22:26Z) by `upkeep-lead`, in response to the Lead's E5a order. (Original
intent: run when Cayleb returned number picks. That intent is void as of the ruling above.)
**No pick has been returned by Cayleb yet, so nothing below has been run.** This file is the exact
command set + the exact verification read for each item, plus a no-op proof for every item answered `no`.

Baselines used for the proofs (captured 22:24Z, stored in `board/probes/team-e-e5/`):
`baseline-pr-states.tsv` (7 PRs: all `open`/`draft=true`/`merged=false`), `baseline-branch-counts.tsv`,
`baseline-archive-targets.tsv` (all four targets `archived=false`, `open_prs=0`).

**Global safety rules for execution (apply to every block below):**
1. One pick at a time, exactly as named. No bundling, no "while I'm here".
2. After each write, a **separate** read proves the result (never trust the write call's own output).
3. Any verification that does not match its expected value: stop, report, do not retry a third way.
4. Archiving is **reversible** (`archived=false` restores it) and does not delete code.
5. These are the only write verbs authorised: `PATCH repos/<r> -F archived=true|false`. No merges by
   Team E (item 8 is Team A's), no deletes, no renames, no branch deletions (item 13 is `no`).
6. Log every command + output to `board/probes/team-e-e5/execution-<UTC timestamp>.log` (append-only).

---

## Item 8 — `yes` means: merge `ergon-lite` into `ergon`, then archive `ergon-lite`

**Team E does not execute this.** Per the Lead's confirmed split, Team A executes under Ergon's merge
regime (fresh-context checker PASS + merged-tree proof); Cayleb's "8 yes" is the authorisation. This
block is the handoff to paste to `ergon-lead` — Team A may adapt the mechanics, but the acceptance and
the delivered artifact are fixed.

```
HANDOFF → ergon-lead (item 8, Cayleb naming given)
Scope: bring ergon-lite's 19 files that are absent from ergon into ergon; then ergon-lite is archived.
Evidence that the content is real (not duplicates): comparing trees at 22:03Z,
  ergon-lite = 21 files; ergon main = 701 files; identical paths = 2; missing from ergon = 19
  (the whole lite/ subtree: lite/index.js, lite/client.js, lite/preferences.js, lite/proposals.js,
   lite/private-release.js, lite/updater.js, lite/cordis.patch.yml, lite/installer/{README,bootstrap.nsi,build.ps1},
   lite/package.json, lite/test/{client,preferences,private-release,proposals,updater}.test.js, lite/README.md,
   lite/UPDATER-CONTRACT.md, .env.example)
Suggested mechanics (adapt, keep the regime):
  git fetch origin
  git worktree add /tmp/ergon-lite-merge origin/main
  cd /tmp/ergon-lite-merge && git remote add lite https://github.com/cayleb-james2008/ergon-lite.git
  git fetch lite main && git merge --no-ff lite/main -m "merge: bring ergon-lite into ergon (Cayleb pick 8)"
  # resolve, run the full suite in the merged tree, then exercise the lite/ tests explicitly
Required before it counts: fresh-context checker (not the merger) runs the merged-tree proof with
  0 branch-attributable failures; record the suite totals and the merge commit sha.
The archive of ergon-lite happens ONLY after the merge is verified on the remote:
  gh api --method PATCH repos/cayleb-james2008/ergon-lite -F archived=true
```

**Team E's post-action verification reads (run after Team A reports; quote output in the report):**
```bash
# 1. the 19 previously-absent paths now exist on ergon's main
gh api 'repos/cayleb-james2008/ergon/git/trees/main?recursive=1' \
  --jq '[.tree[]|select(.type=="blob")|.path]|.[]' > /tmp/ergon-tree-after.txt
comm -23 <(sort /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/github-sweep-20261031/ergon-lite-tree.txt) \
         <(sort /tmp/ergon-tree-after.txt) | wc -l        # expect 0
# 2. the merge landed as a merge commit, nothing force-pushed away
gh api repos/cayleb-james2008/ergon/commits/main \
  --jq '{sha: .sha, parents: [.parents[].sha], msg: (.commit.message|split("\n")[0])}'
# 3. ergon-lite archived (only if that step was taken)
gh api repos/cayleb-james2008/ergon-lite --jq '{archived, pushed_at}'
```
**If ergon-lite's archive is *not* part of what Cayleb intended:** the merge still stands; leave
`ergon-lite` unarchived and say so. Do not archive on the team's own initiative.

---

## Items 9-12 — `yes` means: archive the repo (read-only, reversible)

Same verb for all four. Run the **precondition read** first, then the write, then the **verification
read** as a separate call.

### Item 9 — `openbell-meme-desk`
```bash
# precondition (expect: archived=false, open_prs=0)
gh api repos/cayleb-james2008/openbell-meme-desk --jq '{archived, pushed_at}'
gh api 'repos/cayleb-james2008/openbell-meme-desk/pulls?state=open' --jq 'length'
# write
gh api --method PATCH repos/cayleb-james2008/openbell-meme-desk -F archived=true
# verification (separate call; expect archived=true)
gh api repos/cayleb-james2008/openbell-meme-desk --jq '{archived, is_template, disabled}'
# rollback if ever needed
gh api --method PATCH repos/cayleb-james2008/openbell-meme-desk -F archived=false
```

### Item 10 — `memefly`
```bash
gh api repos/cayleb-james2008/memefly --jq '{archived, pushed_at}'            # expect archived=false
gh api --method PATCH repos/cayleb-james2008/memefly -F archived=true
gh api repos/cayleb-james2008/memefly --jq '{archived}'                       # expect true
```

### Item 11 — `meme-hedge-fund`
```bash
gh api repos/cayleb-james2008/meme-hedge-fund --jq '{archived, pushed_at}'    # expect archived=false
gh api --method PATCH repos/cayleb-james2008/meme-hedge-fund -F archived=true
gh api repos/cayleb-james2008/meme-hedge-fund --jq '{archived}'               # expect true
```

### Item 12 — `talos-os-archive` (the empty repo)
```bash
gh api repos/cayleb-james2008/talos-os-archive --jq '{archived, size, pushed_at}'  # expect archived=false
gh api --method PATCH repos/cayleb-james2008/talos-os-archive -F archived=true
gh api repos/cayleb-james2008/talos-os-archive --jq '{archived}'                   # expect true
```

**Why these are safe to archive:** none has an open PR (verified 22:24Z, `open_prs=0` for all four);
archiving preserves every byte of history and is one call to reverse; the unified `openbell` repo is
untouched by item 9 (different repo), and 92 of the old desk's 112 filenames already exist inside
`openbell`.

---

## No-op proofs — for every item Cayleb answers `no` (or skips)

Run these **after** executing the `yes` picks, in one batch, and diff against the 22:24Z baseline.
Expect: zero differences for `no` items.

```bash
cd /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/team-e-e5

# A. the 7 held PRs must be exactly as they were: open / draft=true / merged=false
: > current-pr-states.tsv
for spec in "kairos-cloud 6" "kairos-cloud 7" "kairos-cloud 10" "kairos-cloud 35" \
            "talos-workspace 23" "daedalus 1" "sophos 11"; do
  set -- $spec
  gh api "repos/cayleb-james2008/$1/pulls/$2" \
    --jq '[.base.repo.name, ("#"+ (.number|tostring)), .state, (.draft|tostring), (.merged|tostring)] | @tsv' \
    >> current-pr-states.tsv
done
diff baseline-pr-states.tsv current-pr-states.tsv && echo "NO-OP PROVEN: 7 held PRs untouched"

# B. item 13 (no prune): branch counts unchanged
: > current-branch-counts.tsv
for r in talos-workspace kairos-cloud agentic-resume memefly sophos; do
  printf '%s\t%s\n' "$r" "$(gh api "repos/cayleb-james2008/$r/branches?per_page=100" --jq 'length')" \
    >> current-branch-counts.tsv
done
diff <(grep -E 'talos-workspace|kairos-cloud|agentic-resume|memefly|sophos' baseline-branch-counts.tsv) \
     current-branch-counts.tsv && echo "NO-OP PROVEN: 0 branches deleted (158 expected across the 5)"

# C. archive targets that were answered no must still be archived=false
for r in openbell-meme-desk memefly meme-hedge-fund talos-os-archive; do
  printf '%s archived=%s\n' "$r" "$(gh api repos/cayleb-james2008/$r --jq '.archived')"
done
```

Also verify the estate-level invariant after any mix of picks:
`gh repo list cayleb-james2008 --limit 200 --jq '[.[]|select(.isArchived)]|length'` →
expect **5 + (number of `yes` archives)**; and total repos still **61** (archiving never removes a repo).

---

## Expected end-state table (fill in as picks are executed)

| Item | Pick | Action | Verification read | Expected | Actual | Verdict |
|---|---|---|---|---|---|---|
| 8 | yes | merge by Team A, then archive ergon-lite | 19 paths present on ergon main; ergon-lite `archived` | 0 missing; true | — | — |
| 9 | yes | archive openbell-meme-desk | `archived` | true | — | — |
| 10 | yes | archive memefly | `archived` | true | — | — |
| 11 | yes | archive meme-hedge-fund | `archived` | true | — | — |
| 12 | yes | archive talos-os-archive | `archived` | true | — | — |
| 1-7, 13 | no | nothing | PR/branch diff vs baseline | zero diff | — | — |

Nothing in this file has been executed. A fresh checker re-runs the verification reads before any pick
is called done.
