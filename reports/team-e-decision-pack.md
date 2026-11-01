# DECISION PACK — Job 18 (GitHub estate), 31 Oct 2026

**STATUS 2026-10-31 20:27 ET - NO LONGER AWAITING PICKS.** Cayleb delegated the held-PR picks and the
repo-consolidation namings to Nox's best recommendation; execution is running on Nox's side (Nox 00:20Z
manager ruling). This pack remains on record as the evidence trail and the plain-language statement of
each item; Team E does not execute any of it. If the repos change, Team E records the change and does
not act on it.

For Cayleb. Built by `upkeep-lead` (Team E) from a fresh read-only sweep of all 61 repos,
2026-10-31 21:59-22:05Z (17:59-18:05 ET). Nothing in this pack has been done — every item is a
proposal, and nothing is merged, deleted, archived or renamed without you naming that exact repo or PR.

**How to answer:** give each item's number and one word — `yes` (do the thing it says), `no` (leave it
exactly as it is), `merge` (only where merging is the thing it says). Skip any item you don't care about;
skipped items are left alone, which is always safe. Example reply:
`1 no, 2 no, 3 no, 4 no, 5 no, 6 no, 7 no, 8 yes, 9 yes, 10 yes, 11 yes, 12 yes, 13 no`

**My recommendation in one line:** say **no** to all seven draft PRs (they are dormant research, not
live code) and **yes** to the four cleanups (8-12); the estate gets tidy without touching anything that
matters.

**What the sweep found first (so the pack rests on facts, not memory):**
- **61 repos**, 5 already archived, 0 forks, 4 public. The two repos that were UNKNOWN after the 31 Oct
  rate-limit (`daedalus`, `kairos-cloud`) were re-swept first and are now **resolved — no unknowns left**.
  No rate-limiting this pass (662 API points used of 5,000/hour).
- **Open PRs: exactly 7, and every one is a draft.** No PR has been opened or merged on any other repo
  since 29 Oct except: `openbell` #1 and #3 merged and #2 closed (this morning), `ergon` #6 merged / #5
  and #7 closed, `sophos` #8 merged, `agentic-resume` #9 merged, `gomining-capital-os` #2 closed.
- **338 branches**, of which **193 non-default branches have not been touched in over 14 days** and 11
  over 30 days. That is backlog, not breakage.
- No repo is missing, broken or dirty in a way that needs an urgent decision.

---

## 1. kairos-cloud PR #6 — merge it?

**One sentence:** "Fail closed on an entry quote we cannot date, depth or corroborate" — a safety fix for
the Kairos meme-trading bot: if the bot can't value an entry, it refuses the trade instead of guessing.

**Facts (read from GitHub):** draft, 33 commits, 36 files, +7,919/−136, tests green, mergeable clean,
untouched since 30 Sep. It targets the repo's default branch, which — unusually — *is itself* a feature
branch (`claude/kairos-meme-coin-indicators-it8aux`), because this repo has no `main`.
**Recommendation: no.** Kairos Cloud is dormant (no push since 8 Oct) and nothing runs it; merging now
adds history to a repo nobody is using. If you ever revive Kairos Cloud, say yes and it lands cleanly.
**If you pick nothing:** it stays a draft exactly as it is; nothing breaks.

## 2. kairos-cloud PR #7 — merge it?

**One sentence:** the independent verification review of PR #6 (v3, head 80b055e) plus a proposed design —
it is the audit trail for item 1, not new product code.

**Facts:** draft, 5 commits, 585 files, +68,656/−0 (almost all generated evidence documents), tests green,
untouched since 30 Sep.
**Recommendation: no.** It is #6's review; with #6 staying a draft there is nothing for it to verify on
the mainline.
**If you pick nothing:** stays a draft; nothing breaks.

## 3. kairos-cloud PR #10 — merge it?

**One sentence:** "Persist recognized ambiguous submissions before reconciliation" — a trading-safety fix
that writes down any submission the bot couldn't classify, so a restart can't lose track of it.

**Facts:** draft, 60 commits, 96 files, +42,848/−301, tests green, mergeable clean, untouched since 3 Oct.
**It is stacked on item 1:** its base branch is `buffy/daedalus-research`, which is PR #6's head. So a
"yes" here alone would not reach even the repo's default branch.
**Recommendation: no** (and it only becomes meaningful after #6).
**If you pick nothing:** stays a draft; nothing breaks.

## 4. kairos-cloud PR #35 — merge it?

**One sentence:** a threat model and historical evidence package for order-bound transaction
authorisation — documentation and evidence only, no source-code changes.

**Facts:** draft, 12 commits, 34 files, tests green, mergeable clean, untouched since 7 Oct. Stacked on
item 3 (its base is #10's head branch).
**Recommendation: no.** Pure documentation in a dormant repo; it is already preserved as a branch.
**If you pick nothing:** stays a draft; nothing breaks.

## 5. talos-workspace PR #23 — merge it?

**One sentence:** a four-file review-evidence note in the Talos workspace that cross-references the
kairos-cloud #35 authorisation draft — it is a paper trail, not a code change.

**Facts:** draft, 1 commit, 4 files, +539/−0, targets `main`, untouched since 7 Oct, no CI checks.
**Recommendation: no.** It is a note pointing at item 4; with item 4 staying a draft, the note is a
record, not a change.
**If you pick nothing:** stays a draft; nothing breaks.

## 6. daedalus PR #1 — merge it?

**One sentence:** "Daedalus v0.2: git-native ratchet architecture for recursive self-improvement" — a
design/architecture proposal for the Daedalus project.

**Facts:** draft, 4 commits, 96 files, +7,801/−1,137 — and it **cannot be merged as it stands**: GitHub
reports it is in conflict with `main` (`mergeable=false`, `mergeable_state=dirty`). A "yes" would first
cost 1-2 hours of conflict resolution, then a review, before anything could land.
**Recommendation: no.** It is a v0.2 design record; the 3 Oct conflict has sat for four weeks and
nothing depends on it. If you want the ideas live, the cheaper route is a fresh branch off today's
`main`, not a rescue of this PR — I can propose that separately.
**If you pick nothing:** stays a draft; nothing breaks.

## 7. sophos PR #11 — merge it?

**One sentence:** "compose Prime Agent hardening on the PR9 runtime baseline" — a security-hardening
draft layered on an earlier runtime PR.

**Facts:** draft, 34 commits, 50 files, +11,794/−216; **CI not green** (`mergeable_state=unstable`); its
base branch is `fix/combined-protocol-packaging`, an unmerged feature branch, not `master` — so a "yes"
would not put anything on the repo's mainline. `sophos` is one of your four public repos.
**Recommendation: no.** Unstable and stacked on an unmerged branch; merging would be noise.
**If you pick nothing:** stays a draft; nothing breaks.

---

## 8. Merge `ergon-lite` into `ergon` — your Ergon-is-one-project ruling

**One sentence:** the two Ergon repos should be one repo; `ergon-lite` holds the laptop/desktop "lite"
client, installer and updater work that is not yet in `ergon`.

**Facts:** `ergon-lite` = 21 files, last commit 29 Oct 2026 ("Add private release download verification"),
1 branch, 90 KB. Comparing file trees: **19 of its 21 files do not exist in `ergon`** — the whole `lite/`
subtree (NSIS installer, updater, preferences, private-release, five test files). So this is a real
content merge, not a tidy-up of duplicates. `ergon` is the live project (701 files, 93 branches, pushed
today 17:52 ET).
**Recommendation: yes** — it matches your standing ruling that Ergon is ONE project. It should be
executed by Team A (Ergon's owners) under your existing Ergon merge authorisation (fresh checker PASS +
merged-tree proof), then `ergon-lite` archived. I would only record the naming; I would not write to the
Ergon repo from the estate lane.
**If you pick nothing:** the lite client/installer/updater stay split across two repos — nothing breaks,
but Ergon stays two projects, against your ruling.

## 9. Archive `openbell-meme-desk` — superseded by the unified `openbell` repo

**One sentence:** the old standalone meme-desk repo has been folded into the unified `openbell` repo,
which merged its own PR #3 this morning (10:51 ET), so the old repo is now a duplicate.

**Facts:** `openbell-meme-desk` = 112 files, 1.6 MB, 1 branch, last push 28 Oct. The unified `openbell`
has a top-level `meme/` tree and **92 of the old repo's 112 filenames appear inside `openbell`** (the
content was reorganised, so paths differ). Archiving is reversible (GitHub can unarchive).
**Recommendation: yes.**
**If you pick nothing:** 1.6 MB of superseded code stays in the estate list and keeps appearing in every
future sweep.

## 10. Archive `memefly` — one of the two "old desk repos"

**One sentence:** an older paper-only meme-trading app ("PAPER ONLY · NO FUNDING · EXCHANGE EXECUTION
DISABLED") from before the OpenBell desk, with 22 branches and no activity since 8 Oct.

**Facts:** 2.2 MB, 22 branches of which 21 are stale (14+ days), last push 8 Oct 2026.
**Recommendation: yes** — best evidence match for one of the two old desk repos the brief says to
archive. If you did not mean this repo, name the one you meant and I will leave this alone.
**If you pick nothing:** it stays; nothing breaks.

## 11. Archive `meme-hedge-fund` — the second "old desk repo"

**One sentence:** the old Kairos meme-desk research workspace — its own README says the trading engine,
wallet handling and agent orchestration live *outside* this repo, so it is dated findings and sizing
notes only.

**Facts:** 138 KB, 1 branch, last push 8 Oct 2026. (Third candidate if you meant something else:
`autonomous-trading-team`, 2 MB, 11 branches, last push 14 Oct — it is a different thing: a trading-team
workspace, not a desk. I have not proposed it.)
**Recommendation: yes.** Note: the brief names "the two old desk repos" without naming them; item 10 and
this item are my evidence-based answer. If you meant different repos, name them and these stay put.
**If you pick nothing:** it stays; nothing breaks.

## 12. Archive `talos-os-archive` — an empty repo

**One sentence:** this repo exists only as a name: it has **zero commits and zero branches** yet is not
flagged as archived, so it is the one genuinely empty shell in the estate.

**Facts:** `isEmpty=true`, 0 branches, 0 commits, never archived (last "push" 14 Oct is the creation
event). It is the 61st repo that makes the count 61 rather than the 60 in the brief.
**Recommendation: yes** — archive it, or delete it if you prefer (archiving is safer and reversible).
**If you pick nothing:** a permanently empty repo stays in every sweep.

## 13. Delete stale branches (bulk approval) — **not recommended tonight**

**One sentence:** the estate carries 338 branches, and 193 non-default branches have not moved in over 14
days (11 over 30 days), concentrated in five dormant repos: `talos-workspace` 52, `kairos-cloud` 39,
`agentic-resume` 22, `memefly` 21, `sophos` 16.

**Facts:** "stale" does **not** mean "merged" — deleting an unmerged branch can lose work that only
exists there, and branches are gone for good once GitHub garbage-collects them. I have not yet proved
which are merged.
**Recommendation: no for now.** Instead, next cycle I run a per-branch merged/not-merged check on those
five repos (~300 read-only API calls) and bring you a second pack that recommends deleting **only the
confirmed-merged** ones, repo by repo. If you say "yes" here instead, I will treat it as approval for
that confirmed-merged set only — never for unmerged work.
**If you pick nothing:** branches accumulate; nothing breaks.

---

## Answer sheet (copy this back with your words)

```
 1 kairos-cloud #6      —  no
 2 kairos-cloud #7      —  no
 3 kairos-cloud #10     —  no
 4 kairos-cloud #35     —  no
 5 talos-workspace #23  —  no
 6 daedalus #1          —  no
 7 sophos #11           —  no
 8 ergon-lite → ergon   —  yes
 9 archive openbell-meme-desk — yes
10 archive memefly      —  yes
11 archive meme-hedge-fund — yes
12 archive talos-os-archive — yes
13 prune stale branches —  no
```

## Deliberately not in this pack

- **Ergon's own open PRs** and Pack 19/20 merges: those run under your standing Ergon merge
  authorisation (fresh checker + merged-tree proof). Different regime — the brief says do not mix them.
- **Visibility of your 4 public repos** (`sophos`, `dotz`, `industry-ai-suite`, `megavibe`): no action
  proposed; flagged here only so it is on the record that they are public.
- **The `daedalus` / `daedulus` spelling pair:** two genuinely different projects (different
  descriptions, different default branches). Renaming would break local paths for no gain tonight — FYI
  only.
- **The 5 already-archived repos:** correctly archived, nothing to do.
- Anything not named above: the sweep found no other decision that needs you.

## How this pack will be verified (when you answer)

Each pick is executed exactly as named, nothing bundled, and verified on GitHub's remote (not from my
local view): for a PR, the PR's `state`/`merged` after the action; for an archive, the repo's
`archived: true` flag after the action; for each item you said no to, a read proving it is untouched.
Maker never checker: a fresh checker re-runs those reads before anything is called done.

**Evidence for every fact above** is in `board/probes/github-sweep-20261031/` — `repos.tsv`,
`branches.tsv`, `branch-dates.tsv`, `open-prs.txt`, `held-prs-detail.tsv`, `held-prs-checks.tsv`,
`sweep-timeline.log`, plus tree comparisons (`ergon-lite-tree.txt` vs `ergon-tree.txt`,
`meme-desk-tree.txt` vs `openbell-tree.txt`).
