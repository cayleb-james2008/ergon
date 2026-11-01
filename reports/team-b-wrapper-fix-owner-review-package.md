# WRAPPER FIX + SECOND V3 ATTEMPT — owner review package (one combined ask)

Prepared 2026-11-01T17:58Z by `fleet-lead` (Team B, Lane 1) at Nox's ruling (181500Z §2, FIX direction — not
AMEND). **Nothing is installed, nothing is run, no window opens.** This is the package Nox can carry to the owner
as one combined ask.

## 1. The combined ask (one line for the owner)

> Approve (a) the reviewed fixed post-install **mode-normalisation step in the authorised install wrapper** (exact
> diff below, proposed wrapper sha256 `eb0ad024…`) and (b) **one further controlled run of the unchanged V3
> identity procedure** under the same conditions as before (same approved procedure bytes, same operator seat,
> same narrow sudoers/polkit drop-ins, same window/receipt rules), replacing the consumed one-invocation approval
> with this single combined authorisation.

## 2. Pinned inputs (pre/post hashes)

| Input | Value |
|---|---|
| Current installed wrapper (source of record, reviewed) | `/usr/local/bin/dsh-fleet-admin` sha256 **`f84075cf1ab0f1df71bcbf8dc2af3bb883e60f432d4c8d2478db9def0db3be91`** (identical to `ops/dsh-fleet-admin` at cf93107) |
| **Proposed patched wrapper** (this package's exact bytes) | `~/workspace/harness-build/scratch-tb-l1/proposed-dsh-fleet-admin` sha256 **`eb0ad02480282b9b89341beb991ce38e4b6ffcace5c755d945dbcf6f0f4fb888`** (`bash -n` clean; unified diff sha256 `62710ac2…`, +53 lines) |
| Procedure source (unchanged) | `652d69f59d4edfa62a7f702e26cfc5a3b101aa1e96284620f8b9d65ea32012b7` |
| Procedure bootstrap doc (unchanged) | `129972590e9e0323898c92249febd0d942e47a84263748824249e114221053f6` |
| Fleet caller / comparison core / manifest (unchanged) | `556ad026…` / `f211db39…` / `6fa03597…` |
| **82-pair source** (modes) | `/var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz` sha256 **`37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177`** (82 members, verified) + `identity-accepted-manifest.txt` (`6fa03597…`) |
| Independent procedure review (unchanged) | `de466966…` (source PASS WITH CONDITIONS; reviewer not the formal checker) |
| Narrow sudoers/polkit drop-ins (unchanged) | owner command block doc sha256 `88691743768cc71a79a46ff3c9e04462957676337934defe7f1ea0bb613cf4ac` (§A/§B/§D stand) |
| Window receipt scripts (unchanged) | `window-receipt-v3.sh 6c410a0e…`, runner `273fa8d3…` |

## 3. Exact wrapper diff (reviewed change: one fixed post-add normalisation step)

```diff
--- /home/cayleb/workspace/harness-build/dsh-fleet-integration/ops/dsh-fleet-admin	2026-10-31 19:22:49.349138514 -0400
+++ /home/cayleb/workspace/harness-build/scratch-tb-l1/proposed-dsh-fleet-admin	2026-11-01 13:20:26.140534527 -0500
@@ -56,6 +56,56 @@
   if [ -f "$SAVED/sha256" ]; then [ "$(sha256of "$patch")" = "$(/usr/bin/cat -- "$SAVED/sha256")" ]; else ! /usr/bin/grep -qxF -- "$ARM_NOTE" "$patch"; fi
 }
 
+
+# -- Effects of the identity fix: fixed post-install mode normalisation (reviewed change, one-time basis) ----------
+# The sanctioned installer clones from the pnpm content-addressable store (under umask 027): files arrive hard-linked
+# and with archive modes masked (0644->0640, 0755->0750), which the identity procedure's caller refuses. This step
+# de-links each member by atomic copy+rename, sets the archive member's exact mode, sets every package directory to
+# 0750, then re-verifies the whole tree and exits non-zero on any mismatch (fail closed; idempotent).
+normalise_installed_tree() { # $1 = hash-verified tarball, $2 = installed package root
+  local tgz=$1 root=$2
+  [ -f "$tgz" ] && [ ! -L "$tgz" ] || die "normalise: tarball missing or a symlink"
+  [ -d "$root" ] && [ ! -L "$root" ] || die "normalise: package root missing or a symlink"
+  /usr/bin/python3 -I -S -B - "$tgz" "$root" <<'PYNORM' || die "normalise: failed (tree left for inspection; re-run plugin-add then this step)"
+import os, stat, sys, shutil, tarfile
+tgz, root = sys.argv[1], sys.argv[2]
+with tarfile.open(tgz, "r:gz") as tf:
+    members = [(m.name[len("package/"):], m.mode & 0o7777) for m in tf.getmembers() if m.isfile()]
+if len(members) != 82:
+    raise SystemExit(f"normalise: expected 82 members, got {len(members)}")
+for rel, mode in members:
+    if mode not in (0o644, 0o755):
+        raise SystemExit(f"normalise: unexpected archive mode {mode:o} for {rel}")
+    p = os.path.join(root, rel)
+    try:
+        st = os.lstat(p)
+    except OSError as exc:
+        raise SystemExit(f"normalise: missing member {rel}: {exc}")
+    if not stat.S_ISREG(st.st_mode):
+        raise SystemExit(f"normalise: member is not a regular file: {rel}")
+    if st.st_nlink != 1 or stat.S_IMODE(st.st_mode) != mode:
+        tmp = p + ".norm-tmp"
+        with open(p, "rb") as srcf, open(tmp, "wb") as dstf:
+            shutil.copyfileobj(srcf, dstf)
+        os.chmod(tmp, mode)
+        os.replace(tmp, p)
+for dirpath, dirnames, _files in os.walk(root):
+    os.chmod(dirpath, 0o750)
+# fail-closed re-verify
+seen = 0
+for rel, mode in members:
+    p = os.path.join(root, rel)
+    st = os.lstat(p)
+    if not stat.S_ISREG(st.st_mode) or st.st_nlink != 1 or stat.S_IMODE(st.st_mode) != mode:
+        raise SystemExit(f"normalise: post-verify failed for {rel}")
+    seen += 1
+for dirpath, _dirnames, _files in os.walk(root):
+    if stat.S_IMODE(os.lstat(dirpath).st_mode) != 0o750:
+        raise SystemExit(f"normalise: post-verify directory mode wrong: {dirpath}")
+print(f"normalise: {seen} files single-link with archive modes; package dirs 0750")
+PYNORM
+}
+
 verb=${1:-}
 [ $# -gt 0 ] && shift
 
@@ -89,7 +139,8 @@
     [ "$actual" = "$sha" ] || { /usr/bin/rm -f -- "$tmp"; die "sha256 mismatch for $file"; }
     /usr/bin/chmod 0640 "$tmp"; /usr/bin/mv -f -- "$tmp" "$PKGS/$file"
     audit "plugin-add $file sha256=$sha via=${PKEXEC_UID:+pkexec}${PKEXEC_UID:-sudo}"
-    "$DSH" plugin --profile fleet add --save-exact --ignore-scripts "$PKGS/$file" ;;
+    "$DSH" plugin --profile fleet add --save-exact --ignore-scripts "$PKGS/$file"
+    normalise_installed_tree "$PKGS/$file" "$PROFILE/node_modules/@cayleb/dsh-fleet" ;;
   plugin-remove)
     name=${1:-}; [[ $name =~ ^(@[a-z0-9][a-z0-9._-]*/)?[a-z0-9][a-z0-9._-]*$ ]] || die "plugin-remove <package name>"
     audit "plugin-remove $name"
```

## 4. What the new step does (semantics)

After `dsh plugin --profile fleet add --save-exact --ignore-scripts` succeeds (the wrapper already hash-verified the
tarball copy), the fixed step:
1. reads the 82 `(archive-relative path, archive member mode)` pairs **from the already hash-verified tarball**
   (no network, no caller-supplied paths, no argument passthrough);
2. for each member under the fixed package root: if the file is hard-linked (`st_nlink != 1`) or its mode differs
   from the archive member mode, writes the content to a sibling temp file, sets the archive mode, and renames it
   over the original (de-links, bytes unchanged);
3. sets every directory of the package tree (root and subdirectories) to `0750`;
4. **re-verifies fail-closed** before returning: exactly 82 single-link regular files with the archive modes, every
   directory `0750` — any mismatch exits non-zero (the install verb fails; nothing is trusted).

Why this is the right direction (scratch-proven): the sanctioned installer clones from the pnpm content-addressable
store under the wrapper's `umask 027`, so files arrive **hard-linked and with modes masked** (`0644→0640`,
`0755→0750`); the V3 caller requires single-link + exact archive modes. The normalisation restores the contract's
strong claim instead of relaxing the comparison (the AMEND alternative is parked, not used).

## 5. Scratch re-proof of the exact proposed wrapper step (run 2026-11-01T17:50Z, no real install)

Fresh isolated profile, `dsh plugin add` under `umask 027` (the wrapper's environment), then the **function
extracted verbatim from the proposed wrapper** (sha256 `8a98e109…`):

```
install(umask027) rc=0
run 1  -> normalise: 82 files single-link with archive modes; package dirs 0750   rc=0
run 2  -> normalise: 82 files single-link with archive modes; package dirs 0750   rc=0   (idempotent)
pinned core compare -> CORE-RESULT: PASS {"archive_sha256":"37e3a542…","file_count":82}
negative: one mode tampered -> re-run repairs it, rc=0
negative: one member missing -> fails closed rc=64 ("normalise: missing member …; tree left for inspection")
```

## 6. What does NOT change (explicit)

- **The V3 procedure bytes, the caller, the comparison core and the manifest are unchanged** (pinned above).
- **The operator seat, execution route and window discipline are unchanged** (owner-present, one invocation,
  no-writer window, receipts, full-preserved-result rule).
- **The narrow sudoers/polkit drop-ins stand unchanged** (same command block, same lifetime rule: removed after
  the run + fresh checker).
- **The procedure's independent review stands** (`de466966…`, PASS WITH CONDITIONS); this package does not
  re-open it.
- Only the **one-invocation approval** is replaced by the owner's single combined authorisation line above.
- No change to any other verb, unit, profile, policy or service setting. The wrapper inherits the same
  root-owned 0755 install path; its other verbs are untouched.

## 7. Rollback / uninstall

- **Wrapper rollback:** reinstall the previous pinned bytes (`f84075cf…`, script excerpt attached to this package)
  with the same owner relay — a root act; the package tree is untouched by rollback.
- **Normalisation failure at runtime:** the step exits non-zero *after* the plugin-add; the tree is left for
  inspection. Recovery = re-run the wrapper's fixed `plugin-add` on the same accepted tarball (idempotent), which
  re-installs and re-normalises. No partial tree is ever treated as good.
- **Uninstall:** the fix adds no files outside the package tree and no state; removing the plugin via the fixed
  wrapper (`plugin-remove`) removes the normalised tree. No residual.
- **After the V3 run:** drop-ins removed per §D of the command block; preserved stage left to tmpfs clearing (or an
  owner-authorised removal).

## 8. Test plan

**A. Scratch (done — quoted in §5; repeat with the final installed wrapper bytes before the relay if wanted).**
**B. Real-install dry-run shape (after the owner approves the wrapper fix, before the V3 window):**
1. owner relay installs the proposed wrapper (root 0755, sha256 re-checked on host);
2. release act: stop fleet-dsh → fixed `plugin-add` of the accepted tarball (now with normalisation) → start;
3. read-only checks: wrapper hash, unit active, drop archive hash unchanged; the tree's mode/link state is exactly
   what the V3 caller will measure (no protected read required before the window);
4. only then schedule the V3 window (same receipts, same operator presence) and, after it, the Stage 2 installed
   output check by the distinct fresh checker.
**C. Stop conditions:** any wrapper hash mismatch, any normalisation non-zero, any unit state anomaly → stop, do
not open the window, report.

## 9. Status

Review package complete and self-contained; **proposed wrapper bytes exist only in scratch** (`eb0ad024…`), not
installed, not run. No window opens and no V3 retry happens before the owner's combined approval and Nox's relay
ordering. Stage disposition recorded (tmpfs clearing; digests recorded).
