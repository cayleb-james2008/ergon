# INSTALLER-FIX PROOF — post-add mode normalisation makes the pinned comparison core PASS

Filed 2026-11-01T16:07Z by `fleet-lead` (Team B, Lane 1). Scratch only: no install on the real service, no
protected read, no service action, no retry of the consumed V3 run. Evidence logs:
`~/workspace/harness-build/scratch-tb-l1/raw/fix-proof-normalise.log` (`b0432359…`),
`…/fix-proof-install-umask027.log` (`f473cec8…`), `…/fix-proof-install-umask022.log` (`a411dfb3…`).

## 1. What the failure was

The V3 caller requires (a) every installed file to be a **single-link regular file**, and (b) each installed
file's mode to **equal its tar member mode exactly** (`identity_snapshot_core.py:165,169`). The authorised
install path runs the fixed wrapper, which sets `umask 027`, and the sanctioned `dsh plugin add` clones files
from the pnpm content-addressable store. Result on a fresh scratch install: **82/82 files hardlinked**
(nlink 2–6) and **58/82 mode diffs** (0644→0640, 0755→0750).

## 2. Two candidate repairs, both tested in scratch

| Experiment | Result |
|---|---|
| **A. umask 022 for `dsh plugin add` alone** (fresh profile, existing pnpm store) | **INSUFFICIENT** — still 58 mode diffs (the store was populated under umask 027; clone preserves store modes) and still hardlinks. The install-time umask does not control the modes that actually land. |
| **B. fixed post-add normalisation** (delink each file by atomic copy+rename, set the archive member mode, then chmod package-tree directories 0750) | **CORE-RESULT: PASS** — the pinned `identity_snapshot_core.compare_open_tree` accepts the tree: 82 files, archive sha256 `37e3a542…`, single-link, archive-exact modes, dirs 0750 (ownership field passed with the scratch uid/gid). Log: `pre: files 82 hardlinked 82 mode-diffs 58` → `normalisation: delinked 82 modes-fixed 58` → `CORE-RESULT: PASS {"archive_sha256":"37e3a542…","file_count":82}` |

So the repair that keeps the **strong archive-identical claim** is the normalisation pass, not the umask change
alone. (The real fleet install apparently landed single-link files — the caller's single-link check passed and it
failed at modes — while scratch installs hardlink; the normalisation handles both cases idempotently.)

## 3. Proposed fixed wrapper change (for review; NOT implemented here)

Add a fixed post-install step to the authorised wrapper's `plugin-add` path, running as the fleet principal
(the wrapper already does), with **no caller-supplied arguments**:

1. after `dsh plugin add --save-exact --ignore-scripts "$PKGS/$file"` returns 0;
2. read the 82 `(<relpath>, <mode>)` pairs from the **already hash-verified** tarball (the wrapper has it; fixed
   code, no network);
3. for each file under the fixed package root: if `st_nlink != 1` or mode != archive mode → write content to a
   sibling temp file, `chmod` the archive mode, `rename` over the original (de-links and sets the exact mode,
   bytes preserved);
4. `chmod 0750` every directory of the package tree (root and subdirectories), ownership already fleet;
5. re-verify before returning: 82 files, every file single-link with its archive mode, every directory 0750;
   any mismatch exits non-zero (fail closed, same discipline as the current wrapper).
6. keep the wrapper's `umask` policy for everything else — the normalisation step makes the mode outcome
   deterministic regardless of store state.

**Scope/risk:** this is a security-relevant install-path change (the wrapper). It needs its own review, the
owner's approval, a wrapper reinstall (root act), and then the affected release install act; only after that a
second V3 run (fresh approval) can use it.

**Caveat recorded:** scratch proofs run as uid/gid 1000:1000; the real caller checks ownership 951:950. The
normalisation runs as fleet so ownership is untouched; the pinned core's ownership leg is unchanged (it passed
in the real run — the failure was modes).

## 4. Recommendation

**Preferred repair = the normalisation step (§3).** It restores the contract's strong claim (archive-identical
modes, single-link, fleet-owned) instead of weakening the comparison. The policy alternative (v4 amendment,
`board/reports/team-b-identity-v4-amendment-draft.md`) is drafted for Nox's single-read decision.
