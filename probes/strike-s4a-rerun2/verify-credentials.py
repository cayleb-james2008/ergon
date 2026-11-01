#!/usr/bin/python3
"""CHECKER verifier for the guided-first-run credential check on the FINAL artifact.

Proves, from the scratch run's own artifacts:
  (a) the PTY run no longer errors with the old "terminal secret reader" bug, exits 0,
      and the secret-free summary reports the field stored;
  (b) the fake value landed in the scratch $DSH_HOME/.credentials.yaml refs EXACTLY ONCE,
      store mode 0600, document shape version: 1 + refs: with one entry;
  (c) the fake value appears nowhere else: PTY transcript, host logs, fleet logs,
      first-run report dirs, or anywhere in the scratch root outside the store
      (and outside this verifier's own 0600 value file).
The fake value is read from --value-file and is never printed.
"""
import argparse
import glob
import json
import os
import stat


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--store", required=True)
    ap.add_argument("--transcript", required=True)
    ap.add_argument("--value-file", required=True)
    ap.add_argument("--scratch-root", required=True)
    ap.add_argument("--report-dir", default=os.path.expanduser("~/.local/state/dsh-fleet/first-run"))
    ap.add_argument("--pre-state", required=True)
    a = ap.parse_args()
    with open(a.value_file, encoding="utf-8") as fh:
        fake = fh.read().strip()
    fails = 0

    def check(ok, msg):
        nonlocal fails
        print(("PASS  " if ok else "FAIL  ") + msg)
        if not ok:
            fails += 1

    with open(a.transcript, "rb") as fh:
        tbytes = fh.read()
    text = tbytes.decode("utf-8", "replace")

    # (a) no old-bug error, exit 0, summary says stored
    check("terminal secret reader" not in text,
          "(a) PTY transcript has no 'needs a terminal secret reader' error")
    check("status: written" in text, "(a) summary reports 'status: written'")
    check("stored: OPENCODE_GO_API_KEY via " in text,
          "(a) summary reports 'stored: OPENCODE_GO_API_KEY via <transport>'")
    m = [line for line in text.split("\n") if line.startswith("stored:")]
    for line in m:
        print("      summary line:", line)

    # (b) store — shape is the native @deepseek-ai/dsh-credentials-local document:
    # top-level version: 1, a records: section (host-generated session grants), and refs:.
    exists = os.path.exists(a.store)
    check(exists, "(b) scratch $DSH_HOME/.credentials.yaml exists")
    pre = {}
    if os.path.exists(a.pre_state):
        for line in open(a.pre_state, encoding="utf-8"):
            if "=" in line:
                k, v = line.rstrip("\n").split("=", 1)
                pre[k] = v
    if exists:
        mode = stat.S_IMODE(os.stat(a.store).st_mode)
        check(mode == 0o600, f"(b) store mode is 0600 (got {mode:04o})")
        with open(a.store, encoding="utf-8") as fh:
            raw = fh.read()
        lines = raw.split("\n")
        check("version: 1" in lines, "(b) document is version: 1")
        check("refs:" in lines, "(b) document has a top-level refs: section")
        # collect the refs entries: 2-space lines after refs: until the next top-level key
        refs = []
        in_refs = False
        for line in lines:
            if line == "refs:":
                in_refs = True
                continue
            if in_refs and line and not line.startswith(" ") and line.endswith(":"):
                break
            if in_refs and line.startswith("  ") and not line.startswith("   "):
                refs.append(line)
        count = raw.count(fake)
        check(count == 1, f"(b) fake value occurs exactly once in the store (got {count})")
        check(len(refs) == 1 and refs[0].startswith("  OPENCODE_GO_API_KEY: "),
              f"(b) refs section holds exactly one entry, OPENCODE_GO_API_KEY (got {len(refs)})")
        if len(refs) == 1 and refs[0].startswith("  OPENCODE_GO_API_KEY: "):
            # The host's credentials store may render the ref as a plain YAML scalar or a
            # quoted one; accept both and compare the decoded value.
            rawval = refs[0].split(": ", 1)[1].strip()
            parsed = None
            try:
                if rawval.startswith('"') and rawval.endswith('"'):
                    parsed = json.loads(rawval)
                elif rawval.startswith("'") and rawval.endswith("'"):
                    parsed = rawval[1:-1].replace("''", "'")
                else:
                    parsed = rawval
            except Exception:
                parsed = None
            check(parsed == fake,
                  "(b) the OPENCODE_GO_API_KEY ref value equals the injected fake value "
                  "(plain or quoted YAML scalar both accepted)")
        if pre:
            check(pre.get("opencode_ref_present_before", "0") == "0",
                  "(b) the OPENCODE_GO_API_KEY ref did not exist before this run")
            check("OPENCODE_GO_API_KEY" not in pre.get("store_refs_keys_before", ""),
                  "(b) pre-run refs keys did not include OPENCODE_GO_API_KEY")
            print("      pre-run store:", pre.get("store_exists_before") and "exists" or "absent",
                  "| top-level keys:", pre.get("store_top_level_keys_before", ""),
                  "| refs keys:", pre.get("store_refs_keys_before", "") or "(none)")

    # (c) absence everywhere else
    needle = fake.encode()
    hits = []

    def scan(path):
        try:
            if not stat.S_ISREG(os.lstat(path).st_mode):
                return
            if os.path.getsize(path) > 32 * 1024 * 1024:
                return
            with open(path, "rb") as fh:
                data = fh.read()
        except OSError:
            return
        if needle in data:
            hits.append(path)

    scratch_root = os.path.abspath(a.scratch_root)
    store_abs = os.path.abspath(a.store)
    value_abs = os.path.abspath(a.value_file)
    for path in (a.transcript,):
        if needle in tbytes:
            hits.append(path)
    for dp, dns, fns in os.walk(scratch_root):
        dns[:] = [d for d in dns if d != "pnpm-store"]
        for fn in fns:
            p = os.path.join(dp, fn)
            if os.path.abspath(p) in (store_abs, value_abs):
                continue
            scan(p)
    for p in glob.glob(os.path.join(a.report_dir, "*")):
        scan(p)

    check(not hits, f"(c) fake value appears nowhere outside the store "
                    f"(transcript/logs/reports/scratch tree; hits={len(hits)})")
    for h in hits[:8]:
        print("      LEAKED INTO:", h)

    print(f"verify-credentials: {'PASS' if fails == 0 else 'FAIL'} — {fails} failed")
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
