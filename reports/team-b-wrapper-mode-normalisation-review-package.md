# WRAPPER-CHANGE REVIEW PACKAGE — mode-normalisation step for the authorised install path

Prepared 2026-11-01T16:40Z by `fleet-lead` (Team B, Lane 1) so it is ready the moment Nox rules **FIX**.
**Nothing here is implemented, staged, installed or run.** The install path is security-relevant and gets its
own independent review + owner approval before any change. Files: this package + proof
`board/probes/team-b-installer-mode-fix-proof-20261101T1607Z.md`.

## 1. Why (one paragraph)

The approved V3 identity comparison requires each installed file to be a **single-link regular file with the
exact archive member mode**. The sanctioned install path (`dsh plugin add` invoked by the authorised wrapper)
clones from the pnpm content-addressable store: under the wrapper's `umask 027` the store lands files at
`archive_mode & ~0o027` and clones them as **hardlinks**. Scratch proof: umask-027 install → 82/82 files
hardlinked, 58/82 mode diffs; umask-022 alone is insufficient (store decides); a fixed post-add normalisation
makes the pinned comparison core PASS. This package covers only the normalisation step.

## 2. Where the change goes

File: `ops/dsh-fleet-admin` (installed root-owned at `/usr/local/bin/dsh-fleet-admin`; reviewed source digest
today `f84075cf1ab0f1df71bcbf8dc2af3bb883e60f432d4c8d2478db9def0db3be91`). Verb: `plugin-add`, after the
existing `"$DSH" plugin --profile fleet add --save-exact --ignore-scripts "$PKGS/$file"` succeeds.
No new verb, no new argument, no caller input; the wrapper already holds the root-checked, hash-verified tarball
copy at `$PKGS/$file`.

## 3. Proposed fixed code (review target — sketch, not applied)

```bash
# after the plugin-add call succeeds, fixed normalisation of the package tree it just created.
# Deterministic: modes come from the tarball the wrapper already hash-verified; no network, no caller args.
normalise_tree() {  # $1 = verified tarball path, $2 = installed package root, $3 = fleet uid, $4 = fleet gid
  local tgz=$1 root=$2 uid=$3 gid=$4 rel mode tmp
  [ -f "$tgz" ] && [ ! -L "$tgz" ] || die "normalise: tarball missing or a symlink"
  [ -d "$root" ] && [ ! -L "$root" ] || die "normalise: package root missing or a symlink"
  # file modes from the tarball (single fixed parse, no extraction)
  while IFS=$'\t' read -r mode rel; do
    case $mode in 644|755) ;; *) die "normalise: unexpected archive mode $mode for $rel" ;; esac
    local p="$root/$rel"
    [ -f "$p" ] && [ ! -L "$p" ] || die "normalise: member missing or not a regular file: $rel"
    tmp=$(/usr/bin/mktemp -p "$(/usr/bin/dirname -- "$p")" .norm.XXXXXX) || die "normalise: mktemp"
    /usr/bin/cat -- "$p" > "$tmp" || { /usr/bin/rm -f -- "$tmp"; die "normalise: copy failed: $rel"; }
    /usr/bin/chmod "$mode" -- "$tmp" || { /usr/bin/rm -f -- "$tmp"; die "normalise: chmod failed: $rel"; }
    /usr/bin/mv -f -- "$tmp" "$p" || die "normalise: rename failed: $rel"   # de-links, bytes preserved
  done < <(/usr/bin/tar -tvzf "$tgz" | …fixed parse → "MODE<TAB>package-relative-path"…)
  # directories: package root and every subdirectory to 0750 (assets stay fleet-owned; root fs policy unchanged)
  /usr/bin/find "$root" -type d -exec /usr/bin/chmod 0750 {} +
  # fail-closed re-verify before returning success
  #   82 members, each single-link (st_nlink == 1), each mode == archive mode, each dir 0750, owner fleet
  verify_normalised "$root" "$tgz" "$uid" "$gid" || die "normalise: post-verify failed"
}
```

Notes for the reviewer: the tar listing parse must be fixed-string based (`tar -tvzf` → awk on `$1,$6`), reject
unexpected modes rather than guess; all paths stay under the fixed package root; staging files are created in
the same directory as the target (same filesystem, atomic `mv`); no `chown` needed (installer runs as fleet);
no glob expansion outside the root; every failure leaves the tree as it was except completed members — and the
verb then exits non-zero, so the caller re-runs `plugin-add`/rollback rather than trusting a partial tree
(add a `--force` re-add path in the release runbook).

## 4. Tests the reviewer should require (all scratch, no real install)

1. **Normal path:** fresh scratch profile, `dsh plugin add` (umask 027 environment), then the normalisation →
   82/82 single-link, archive-exact modes, dirs 0750; pinned `identity_snapshot_core.compare_open_tree` PASS
   (already proven in the scratch proof; re-run with the final code).
2. **Idempotency:** run the normalisation twice → second run changes nothing, exit 0.
3. **Already-correct tree:** tree installed with umask 022 and single-link (if achievable) → no-op, PASS.
4. **Failure paths (fail closed):** a missing member, an unexpected archive mode, a symlinked member, a
   non-writable target dir → exit non-zero, no partial "success", tree left inspectable.
5. **Scope:** no writes outside the fixed package root; no change to files owned by anyone else; no change to
   the stage/unit/sudoers/polkit/profile.
6. **Byte preservation:** sha256 of every member before/after normalisation unchanged (content untouched, only
   mode/linkage changed).

## 5. Risk register

| Risk | Mitigation in the sketch |
|---|---|
| Partial normalisation on failure | fail closed; release runbook re-runs a full `plugin-add` (idempotent) and normalisation before any comparison |
| Wrong modes applied | modes come only from the hash-verified tarball; unexpected value aborts |
| Symlink/traversal | all paths fixed under the package root; symlink members refused; temp files created beside targets |
| Atomicity | copy+chmod+rename per file, same directory/filesystem; never edits in place |
| Widening | normalisation only narrows/equals archive modes (0644/0755); dirs 0750; no exec additions |
| Wrapper surface | no new verb/argument; fixed code path only on `plugin-add` success |

## 6. Sequence after Nox rules FIX (each is a separate gate)

1. Implement + review this change; new wrapper source digest pinned.
2. Owner approval + wrapper reinstall (root act) — one relay, same pattern as the previous install block.
3. Re-run the release install act (0.2.2 through the revised wrapper) under a fresh approved operation.
4. Confirm installed tree: 82/82 single-link + archive modes + 0750 dirs (read-only check from a permitted,
   owner-authorised vantage — the same question the V3 run answers).
5. Then the **second V3 run** with a fresh owner approval (v3 approval consumed) + one operator relay of the
   §A/§B drop-ins (digest `88691743…`) + receipts (`window-receipt-v3.sh` `6c410a0e…` / runner `273fa8d3…`).

## 7. Status

**Prepared, not implemented.** Ready for an independent reviewer the moment Nox rules FIX. If Nox rules AMEND
instead, the v4 draft (`board/reports/team-b-identity-v4-amendment-draft.md`) is the alternative and this
package is parked (still needed for the hardlink class: amendment alone does not de-link).
