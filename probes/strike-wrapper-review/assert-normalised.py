#!/usr/bin/python3
"""CHECKER harness: independently assert a normalised tree equals the archive.

For every archive regular member: exists, is a regular file, nlink==1, exact archive
mode, sha256 of installed bytes == sha256 of archive bytes. No extra files. Every
directory (including the package root) mode 0750. Exit 0 only if all hold.
"""
import hashlib
import os
import stat
import sys
import tarfile


def main():
    tgz, pkg = sys.argv[1], sys.argv[2]
    arch = {}
    with tarfile.open(tgz, "r:gz") as tf:
        for m in tf.getmembers():
            if m.isfile():
                rel = m.name[len("package/"):]
                data = tf.extractfile(m).read()
                arch[rel] = (m.mode & 0o7777, hashlib.sha256(data).hexdigest(), len(data))

    problems = []
    found = {}
    for dp, _dns, fns in os.walk(pkg):
        for fn in fns:
            p = os.path.join(dp, fn)
            rel = os.path.relpath(p, pkg)
            st = os.lstat(p)
            found[rel] = st
            if not stat.S_ISREG(st.st_mode):
                problems.append(f"not a regular file: {rel}")
                continue
            if st.st_nlink != 1:
                problems.append(f"nlink={st.st_nlink} (want 1): {rel}")
            if rel not in arch:
                problems.append(f"extra file not in archive: {rel}")
                continue
            mode, sha, size = arch[rel]
            if stat.S_IMODE(st.st_mode) != mode:
                problems.append(f"mode {stat.S_IMODE(st.st_mode):04o} != archive {mode:04o}: {rel}")
            if st.st_size != size:
                problems.append(f"size {st.st_size} != archive {size}: {rel}")
            with open(p, "rb") as fh:
                h = hashlib.sha256(fh.read()).hexdigest()
            if h != sha:
                problems.append(f"sha256 mismatch: {rel}")

    missing = sorted(r for r in arch if r not in found)
    dirs_bad = []
    for dp, _dns, _fns in os.walk(pkg):
        m = stat.S_IMODE(os.lstat(dp).st_mode)
        if m != 0o750:
            dirs_bad.append((os.path.relpath(dp, pkg) or ".", f"{m:04o}"))

    print(
        f"assert: archive={len(arch)} files; installed={len(found)} files; "
        f"missing={len(missing)}; extra={len([r for r in found if r not in arch])}; "
        f"problems={len(problems)}; dirs_wrong_mode={len(dirs_bad)}"
    )
    for item in missing[:5]:
        print(f"  - missing: {item}")
    for problem in problems[:15]:
        print(f"  - {problem}")
    for rel, m in dirs_bad[:5]:
        print(f"  - dir mode {m} (want 0750): {rel}")
    return 0 if not missing and not problems and not dirs_bad else 1


raise SystemExit(main())
