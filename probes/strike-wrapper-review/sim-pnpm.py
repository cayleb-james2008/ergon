#!/usr/bin/python3
"""CHECKER harness: build a simulated pnpm-installed tree.

Extracts the archive, writes each member with its umask-masked mode (0644->0640,
0755->0750) and hard-links every installed file into a separate CAS-like store dir
so each inode has nlink=2, exactly the state the wrapper's normalise step exists to undo.
"""
import os
import shutil
import stat
import sys
import tarfile


def main():
    tgz, root, store, dir_mode_s = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
    dir_mode = int(dir_mode_s, 8)
    pkg = os.path.join(root, "package")
    if os.path.exists(root):
        raise SystemExit(f"sim-pnpm: root already exists: {root}")
    os.makedirs(pkg)
    os.makedirs(store)
    files = {}
    with tarfile.open(tgz, "r:gz") as tf:
        for m in tf.getmembers():
            if not m.isfile():
                continue
            rel = m.name[len("package/"):]
            dest = os.path.join(pkg, rel)
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            with tf.extractfile(m) as src, open(dest, "wb") as out:
                shutil.copyfileobj(src, out)
            mode = m.mode & 0o7777
            masked = {0o644: 0o640, 0o755: 0o750}[mode]
            os.chmod(dest, masked)
            files[rel] = masked
    for rel in files:
        s = os.path.join(store, rel)
        os.makedirs(os.path.dirname(s), exist_ok=True)
        os.link(os.path.join(pkg, rel), s)
    for dp, _dns, _fns in os.walk(pkg):
        os.chmod(dp, dir_mode)
    bad = []
    for rel, masked in files.items():
        st = os.lstat(os.path.join(pkg, rel))
        if st.st_nlink != 2 or stat.S_IMODE(st.st_mode) != masked:
            bad.append(rel)
    print(f"sim-pnpm: {len(files)} regular files masked(0644->0640,0755->0750)+hardlinked(nlink=2); dirs {dir_mode:o}; bad={len(bad)}")
    return 0 if not bad else 1


raise SystemExit(main())
