# Team A — Pack 20 merge record (Job 01 close-out)

Recorded by `ergon-lead` 2026-10-31 20:06–20:08 ET, on the Lead's explicit authorisation ("PASS ACCEPTED — THE PACK 20
MERGE IS AUTHORISED"). Maker never checker: the PASS under which this merge happened is `ergon-checker`'s, not mine; the
merged-tree proof is the checker's to run, not mine.

## The merge (executed exactly as ordered)

- Branch merged: `duet/p20-demo-company` @ **`462277ca46445273ed346f025fbaf5fddb0ba068`** ("P20-10: week scripts in the
  cast voices with stack artefacts, and the chains carried through the week"; its history includes the S1 merge
  `75948eb` of main `b59b599`).
- Target: `main` at **`b59b5990c594bfe7dfda859fcac3bff09ef609d3`** (Pack 19 integrated) — verified against the remote
  (`git ls-remote --heads origin main`) immediately before the merge; no third-party advance.
- Merge style: **merge commit, `--no-ff`** (no squash, no rebase of main, no force-push). The repo's own history
  convention is merge commits with descriptive messages (`Merge duet/p19-… (Pack 19, checker PASS 31 Oct)`), so this
  follows it.
- **Merge commit: `1c7189866b41755b92bc57c8b85d91649b529bcd`** — message
  `Merge duet/p20-demo-company (Pack 20, checker PASS 31 Oct) into main`.
  - parents: **`b59b5990c594bfe7dfda859fcac3bff09ef609d3`** and **`462277ca46445273ed346f025fbaf5fddb0ba068`**
    (`git cat-file -p HEAD` — tree `f2b08d8ab16ee225fce996928267c0b0c4e23c5e`).
- **Tree identity:** the merge tree equals the P20 branch tree — `git diff --stat 462277c HEAD` is empty; `HEAD^{tree}` =
  `462277c^{tree}` = `f2b08d8ab16ee225fce996928267c0b0c4e23c5e`. So the merge introduces the checker-passed tree and
  nothing else.
- **Ancestry:** `git merge-base --is-ancestor 462277c HEAD` → **exit 0**; `git merge-base --is-ancestor b59b599 HEAD`
  → **exit 0**.
- Working tree clean before the merge and after it; **no other change was included in the push** (the checker's four
  non-blocking docs findings F1–F4 are deliberately deferred to the next Ergon commit, as ordered).

## The push (result)

```
$ git push origin main
To https://github.com/cayleb-james2008/ergon.git
   b59b599..1c71898  main -> main
$ git ls-remote --heads origin main
1c7189866b41755b92bc57c8b85d91649b529bcd	refs/heads/main
```

**Remote `main` tip after the push: `1c7189866b41755b92bc57c8b85d91649b529bcd`** (was `b59b599`). Push exit 0; no
force; nothing else pushed.

## Handoff to the checker (post-merge proof)

`ergon-checker` was notified with the new main SHA. Post-merge proof requested: remote main tip = `1c71898`, ancestry of
`462277c` and `b59b599` in main, and the merged-tree full suite in the documented gate environment (`OPENCODE_BIN` unset,
no opencode on PATH) with **0 fail / 0 incomplete / exit 0**. If that proof fails: stop, report, no revert and no
re-merge without the Lead. Successor Job 02 unblocks on the proof.

A local corroborating run on the merged tree (`/home/cayleb/workspace/harness-build/ergon-main-merge`, gate env) was
started for early warning only; its totals do not substitute for the checker's proof and any disagreement is reported.
