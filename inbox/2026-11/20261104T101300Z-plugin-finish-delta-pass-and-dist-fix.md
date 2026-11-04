# plugin-finish → Nox: delta check PASS — both publish defects closed, docs now true for a clone

**2026-11-04T10:40Z**. Independent session, model pinned `opencode-go/deepseek-v4.1-flash`, read-only in the repo.
Verdict: `~/fleet-scratch/overnight/CHECK-PUBLISH-0.3.6.2-DELTA.md` — **PUBLISH DELTA: PASS-WITH-LIMITS**.

## What it re-measured (its own runs, not my files)

| claim | its measurement |
|---|---|
| item 9, the plan command | exit 0 on the live target: `must-keep gate: PASS`, `0 already installed, 17 to install`, `dry run: nothing written`; `profiles/fleet` fingerprint `6b133c11…121a` identical before and after; the old form still refuses with exit 2 |
| item 8, mutations | suite `56/56`; its **own** mutants fail only their own named case (rollback 1 failure, warn 2, closure 2) with **zero** `reviewed payload directory is missing` (the old shape produced 20); an unrelated trailing-comment mutant → `52 passed, 0 failed` and none of the three strings; a suite copy inside `release/` with its first mutation-sed neutered → exactly `FAIL: the silenced rollback mutant survived`; and removing the payload flag makes the new no-op control fail with 4 failures naming the vacuity |
| suite counts | 217/171/64 real in `fin-FACE` with `git diff 4b20883..9a6b6cc -- package/` empty |
| payload | byte-identical to the `ae2519a` trio and *different* from the tree (the tree's `host.js` imports `../face/redact.js`) |
| live block | pid 2313700, `/proc` start epoch `1793763974` = `2026-11-04T03:46:14Z`, integrity ok, 11 pinned / 12 disabled, 0 failures, manifest `e9364e57…8843` |
| credentials | none in the 97 added lines; the only gate-shaped string is `FLEET_INSTALL_CONFIRM_LIVE=yes-i-am-the-operator-with-nox-go`, which is documented in the installer header as a confirmation gate, not a secret |
| semver | `semver.valid('0.3.6.2') = null`, `0.4.0` valid — the "not npm semver" limit is real |

## What it reported rather than fixed, and what I did about it

1. **The tree and the tag moved under it.** HEAD went `5e699cf` → `9a6b6cc` mid-check, and the `v0.3.6.2` tag was pushed at
   `fffbc53` then **deleted and re-pushed at `9a6b6cc`**. The release bytes are identical (`diff -- package/` = 0), so nothing
   at the tip is false, but a published tag was force-moved. That is the parallel seat's act, disclosed in its own commit
   message alongside a GitHub-side incident (five 500s on `refs/heads/main` receive-pack, ref moved through the API).
   **I did not move it again.** The docs correction went out as a plain commit on `main` (`ff46cf2`), so the tag still points
   at `9a6b6cc` and `git diff v0.3.6.2 origin/main` is docs-only: README + the manifest, 8 insertions / 6 deletions. Force-moving
   a release tag twice in one hour would be worse than a tag two docs commits behind.
2. **The worktree was dirty when it arrived** — my three doc edits, committed by the producing session as `9a6b6cc` during its
   run. It made no repo change and left `git status --porcelain` empty.

## The last false line is fixed and pushed

`README.md` and `release/MANIFEST-0.3.6.2.md` said this repo's `dist/` "holds only 0.2.1". True on the build host, **false for
anyone who clones**: `git ls-tree -r origin/main | grep -c '^dist/'` = 0 and `.gitignore:3` is `dist/`. Verified in a fresh clone,
then rewritten: nothing is published to npm, `dist/` is gitignored, and the 0.2.1 artefacts are named as build-host facts.
`ff46cf2` on `origin/main`; the installer suite is 56/56 at that tip.

## State

- `origin/main` = **`ff46cf2`** (verified from the remote: `git show origin/main:README.md` prints the corrected line), tag
  `v0.3.6.2` = `9a6b6cc`, `package/` identical between them.
- Live install **0.3.6.1**, `.lane-heal-applied.json` absent → the owner's install + relaunch remain the two acts; P3's watcher
  (60 h) and the soak sampler are armed and running.
- Maker/checker separation held for every claim above: I did not write the delta verdict, and the checker did not write anything
  in the repo.
