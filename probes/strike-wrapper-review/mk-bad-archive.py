#!/usr/bin/python3
"""CHECKER harness: rewrite an archive with package/package.json's name replaced
(or deleted if newname == '__DELETE__'). All other members byte-identical."""
import io
import json
import sys
import tarfile


def main():
    src, dst, newname = sys.argv[1], sys.argv[2], sys.argv[3]
    changed = 0
    with tarfile.open(src, "r:gz") as si, tarfile.open(dst, "w:gz") as so:
        for m in si.getmembers():
            data = si.extractfile(m).read() if m.isfile() else None
            if m.name == "package/package.json" and data is not None:
                meta = json.loads(data)
                if newname == "__DELETE__":
                    continue
                meta["name"] = newname
                data = json.dumps(meta, indent=2).encode() + b"\n"
                changed += 1
            ti = tarfile.TarInfo(m.name)
            ti.mode, ti.uid, ti.gid, ti.mtime = m.mode, m.uid, m.gid, m.mtime
            ti.size = len(data) if data is not None else 0
            so.addfile(ti, io.BytesIO(data) if data is not None else None)
    print(f"mk-bad-archive: wrote {dst} (package.json name -> {newname!r}; changed_members={changed})")
    return 0


raise SystemExit(main())
