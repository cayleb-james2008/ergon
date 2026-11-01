#!/usr/bin/python3
"""CHECKER compose check: the scratch profile must compose every row the 0.3.0 bundle
patch inserts, with the overlay's scratch values applied.

Expected insert ids are derived from the TARBALL's own patch files (not from the dump),
so the comparison is against the artifact under test.
"""
import re
import sys
import tarfile

ROW = re.compile(r"^- id: (\S+)\s*$")
INSERT_ITEM = re.compile(r"^ {4}- id: (\S+)\s*$")
TOP = re.compile(r"^- ")

PATCH_FILES = ("package/cordis.patch.yml", "package/mcp.row.yml")
PRESET_PREFIX = "package/presets/"


def insert_ids_from(text):
    ids, in_insert = set(), False
    for line in text.splitlines():
        if TOP.match(line):
            in_insert = line.startswith("- insert:")
            continue
        if in_insert:
            m = INSERT_ITEM.match(line)
            if m:
                ids.add(m.group(1))
    return ids


def main():
    dump_path, tgz, root, dump0_path = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
    mcp_port = sys.argv[5] if len(sys.argv) > 5 else "8789"
    dump = open(dump_path, encoding="utf-8").read()
    rows = [m.group(1) for m in (ROW.match(l) for l in dump.splitlines()) if m]
    dump0 = open(dump0_path, encoding="utf-8").read()
    rows0 = [m.group(1) for m in (ROW.match(l) for l in dump0.splitlines()) if m]
    expected = set()
    with tarfile.open(tgz, "r:gz") as tf:
        for member in tf.getmembers():
            if not member.isfile():
                continue
            if member.name in PATCH_FILES or (member.name.startswith(PRESET_PREFIX) and member.name.endswith(".patch.yml")):
                expected |= insert_ids_from(tf.extractfile(member).read().decode("utf-8"))

    fails = []

    def check(ok, line):
        print(("PASS  " if ok else "FAIL  ") + line)
        if not ok:
            fails.append(line)

    check(len(rows0) > 100, f"baseline web-template profile composed {len(rows0)} rows")
    dupes = sorted({r for r in rows if rows.count(r) > 1})
    check(not dupes, f"no duplicate row ids in the composed profile ({len(dupes)} duplicates)")
    check(len(rows) == len(rows0) + len(expected), f"overlay profile composed {len(rows)} rows = {len(rows0)} baseline + {len(expected)} inserted")
    check(len(expected) >= 18, f"tarball bundle patches declare {len(expected)} inserted rows")

    lines = dump.splitlines()
    for rid in sorted(expected):
        try:
            idx = lines.index(f"- id: {rid}")
        except ValueError:
            check(False, f"row present: {rid}")
            continue
        nxt = lines[idx + 1] if idx + 1 < len(lines) else ""
        check(not nxt.strip().startswith("disabled: true"), f"row present and enabled: {rid}")

    check("name: '@cayleb/dsh-fleet'" in dump, "fleet bundle plugin row present (@cayleb/dsh-fleet)")
    checks = [
        ("fleet-core controlSocket is the scratch socket", f"controlSocket: {root}/ctl/control.sock"),
        ("fleet-core dbPath is the scratch DB", f"dbPath: {root}/fleet/state/fleet.db"),
        ("fleet-core fleetRoot is the scratch root", f"fleetRoot: {root}/fleet"),
        ("fleet-mcp owns the chosen port", f"port: {mcp_port}"),
        ("fleet-mcp statePath is scratch", f"statePath: {root}/fleet/state/mcp"),
        ("fleet-mcp adminSocket is scratch", f"adminSocket: {root}/ctl/mcp-admin.sock"),
        ("fleet-egress guardInProcess", "guardInProcess: true"),
        ("fleet-policy peerUid is this uid", "peerUid: 1000"),
    ]
    for label, needle in checks:
        check(needle in dump, f"overlay applied: {label}")

    print(f"check-compose: {'PASS' if not fails else 'FAIL'} — {len(fails)} failed")
    return 0 if not fails else 1


raise SystemExit(main())
