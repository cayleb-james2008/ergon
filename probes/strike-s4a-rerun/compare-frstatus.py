#!/usr/bin/python3
"""CHECKER comparator: `dsh-fleet first-run --status` file list vs the release tarball.

Requires exact equality of name/version and of the file set, with per-path sha256 and
4-digit octal mode equal between the installed-tree status and the archive members.
"""
import hashlib
import json
import sys
import tarfile


def main():
    fr_file, tgz = sys.argv[1], sys.argv[2]
    doc = json.load(open(fr_file, encoding="utf-8"))
    j = doc.get("result") if isinstance(doc, dict) and isinstance(doc.get("result"), dict) else doc
    files = j.get("files") or []
    mine = {f["path"]: (f["sha256"], str(f["mode"])) for f in files}

    arch = {}
    with tarfile.open(tgz, "r:gz") as tf:
        for m in tf.getmembers():
            if not m.isfile():
                continue
            rel = m.name[len("package/"):]
            digest = hashlib.sha256(tf.extractfile(m).read()).hexdigest()
            arch[rel] = (digest, f"{m.mode & 0o777:04o}")

    missing = sorted(set(arch) - set(mine))
    extra = sorted(set(mine) - set(arch))
    mismatch = [(p, arch[p], mine[p]) for p in sorted(set(arch) & set(mine)) if arch[p] != mine[p]]

    print(f"first-run-status: name={j.get('name')} version={j.get('version')} dshVersion={j.get('dshVersion')}")
    print(f"status_files={len(mine)} tarball_members={len(arch)} missing={len(missing)} extra={len(extra)} sha/mode_mismatch={len(mismatch)}")
    for p in missing[:5]:
        print("  - MISSING from status:", p)
    for p in extra[:5]:
        print("  - EXTRA in status:", p)
    for p, want, got in mismatch[:8]:
        print(f"  - MISMATCH {p}: tarball={want} status={got}")

    ok = (j.get("version") == "0.3.0" and j.get("name") == "@cayleb/dsh-fleet"
          and len(mine) == len(arch) and not missing and not extra and not mismatch)
    print(f"compare-frstatus: {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 1


raise SystemExit(main())
