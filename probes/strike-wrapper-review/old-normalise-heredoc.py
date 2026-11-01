import os, stat, sys, shutil, tarfile
tgz, root = sys.argv[1], sys.argv[2]
with tarfile.open(tgz, "r:gz") as tf:
    members = [(m.name[len("package/"):], m.mode & 0o7777) for m in tf.getmembers() if m.isfile()]
if len(members) != 82:
    raise SystemExit(f"normalise: expected 82 members, got {len(members)}")
for rel, mode in members:
    if mode not in (0o644, 0o755):
        raise SystemExit(f"normalise: unexpected archive mode {mode:o} for {rel}")
    p = os.path.join(root, rel)
    try:
        st = os.lstat(p)
    except OSError as exc:
        raise SystemExit(f"normalise: missing member {rel}: {exc}")
    if not stat.S_ISREG(st.st_mode):
        raise SystemExit(f"normalise: member is not a regular file: {rel}")
    if st.st_nlink != 1 or stat.S_IMODE(st.st_mode) != mode:
        tmp = p + ".norm-tmp"
        with open(p, "rb") as srcf, open(tmp, "wb") as dstf:
            shutil.copyfileobj(srcf, dstf)
        os.chmod(tmp, mode)
        os.replace(tmp, p)
for dirpath, dirnames, _files in os.walk(root):
    os.chmod(dirpath, 0o750)
# fail-closed re-verify
seen = 0
for rel, mode in members:
    p = os.path.join(root, rel)
    st = os.lstat(p)
    if not stat.S_ISREG(st.st_mode) or st.st_nlink != 1 or stat.S_IMODE(st.st_mode) != mode:
        raise SystemExit(f"normalise: post-verify failed for {rel}")
    seen += 1
for dirpath, _dirnames, _files in os.walk(root):
    if stat.S_IMODE(os.lstat(dirpath).st_mode) != 0o750:
        raise SystemExit(f"normalise: post-verify directory mode wrong: {dirpath}")
print(f"normalise: {seen} files single-link with archive modes; package dirs 0750")
