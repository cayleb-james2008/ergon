import json, os, stat, sys, shutil, tarfile
tgz, root = sys.argv[1], sys.argv[2]
with tarfile.open(tgz, "r:gz") as tf:
    all_members = tf.getmembers()
    members = [(m.name[len("package/"):], m.mode & 0o7777) for m in all_members if m.isfile()]
    # The member count is derived from the hash-verified archive, not pinned to a release: a pinned count blocks every
    # later release (0.3.0 ships 104 members). The archive's own identity is the binding check.
    if not members:
        raise SystemExit("normalise: the archive has no regular file members")
    manifest = tf.extractfile("package/package.json")
    if manifest is None:
        raise SystemExit("normalise: the archive has no package/package.json")
    try:
        meta = json.loads(manifest.read().decode("utf-8"))
    except Exception as exc:
        raise SystemExit(f"normalise: package/package.json is not JSON: {exc}")
    if meta.get("name") != "@cayleb/dsh-fleet":
        raise SystemExit(f"normalise: archive package name is {meta.get('name')!r}, not '@cayleb/dsh-fleet'")
    if not isinstance(meta.get("version"), str) or not meta["version"]:
        raise SystemExit("normalise: archive package.json has no version")
    if any(not rel or rel.startswith("/") or ".." in rel.split("/") for rel, _m in members):
        raise SystemExit("normalise: archive member path is not a clean relative path")
    print(f"normalise-archive: @cayleb/dsh-fleet {meta['version']} {len(members)} regular members")
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
