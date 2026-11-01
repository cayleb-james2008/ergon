# S4B — Stage 2 acceptance check against the INSTALLED @cayleb/dsh-fleet 0.3.0

**Verdict: ACCEPT** — `EXIT=0`, `summary: pass=60 fail=0`, **`STAGE 2: PASS`** (165 s).

- Checker: `checker-install` (independent). Ran the plugin's own **UNMODIFIED** acceptance check; no root actions; did not modify the check or the installed profile.
- Date: 2026-11-01 18:08:56 → 18:11:41 EST, host `outis-desktop`, as `cayleb` (uid 1000).
- Installed artifact: `@cayleb/dsh-fleet 0.3.0`; tarball sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070` (drop copy byte-identical to release `dist/`), 104 members; installed-tree readback version 0.3.0, 104 files, modes {0644,0755}.
- Check: `strike-release/checks/stage2-check.sh`, sha256 **`7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95`** — matches the expected hash (the same script that passed S1).
- Command, exactly as specified:
  `OUT=<evidence> FLEET_CLI=<release>/package/bin/dsh-fleet.mjs bash <release>/checks/stage2-check.sh`

## Result (verbatim)

```
summary: pass=60 fail=0 (evidence: /home/cayleb/workspace/harness-build/evidence/strike/s4b-stage2-installed)
STAGE 2: PASS
```
```
EXIT=0
trap: cancelled 70 of 1082 fixture orders (append-only rows remain by design)
```

Re-derived counts: `^PASS ` = **60**, `^FAIL ` = **0**, no `NOT-RUN`; `console.err` 0 bytes; `stage2-check.out` byte-identical to `console.out`. Duration 165 s.

## Installed identity before → after

| | before | after |
|---|---|---|
| unit | active, InvocationID `baf7bf93…`, MainPID **510804** | **active**, InvocationID `bc98f82d…`, MainPID **518634** |
| control `{"op":"status"}` | ok:true, integrity.ok:true, failures [], lanes_allowed:true, selftest:true, events 44411 | ok:true, **integrity.ok:true**, failures [], lanes_allowed:true, selftest:true, events 58439 |
| `first-run --status` | 0.3.0, **104 files**, modes {0644,0755} | 0.3.0, **104 files**, modes {0644,0755} |

The check's own 2.11 leg SIGKILLed and restarted the unit: `MainPID 510804 -> 518634; kill rc=0; up again at +3s`. The EXIT trap did not need to start it (`trap: starting` lines = 0). No manual intervention was required.

## Decisive PASS lines

```
PASS pre — cli /home/cayleb/workspace/harness-build/strike-release/package/bin/dsh-fleet.mjs; control socket answers status; selftest on
PASS pre-iface — all assumed ops and tools recognised
      tarball cayleb-dsh-fleet-0.3.0.tgz (sha256 matches sidecar); ./store -> ./src/orders/standalone.js
PASS 2.1a — 1000 orders + 1000 tasks, exactly one winner each: {…"one_winner":2000…}
PASS 2.1b — 1000 orders + 1000 tasks, exactly one winner each: {…"one_winner":2000…}
PASS 2.11-kill — fleet-dsh SIGKILLed mid-burst and back up (MainPID 510804 -> 518634)
PASS inv — 416 rows: open<=>unclaimed, … attempts<=3, done orders have every task done+banked; …
```

## Provenance cross-checks

- Drop tarball == release `dist/` tarball, sha256 `8173d13e…` both.
- All **104 shipped tarball members are byte-identical to the release tree's `package/`** files (the tree has tests/notes beyond `files[]`; none shipped).
- `FLEET_CLI` (`bin/dsh-fleet.mjs`, sha256 `3d839901…`) is byte-identical in the release tree, the tarball, and the **installed** tree (from the in-host `first-run --status` readback). `package.json` installed sha `62aa36d3…` = tree sha.
- So the acceptance check ran the installed 0.3.0 host with exactly the shipped/installed CLI.

## Staging notes (not product defects; both recorded before acting)

1. **Missing conventional sidecar.** `/var/lib/fleet-drop` had `cayleb-dsh-fleet-0.3.0.tgz` but **no `<tgz>.sha256`**; the digest was published only in `SHA256SUMS.strike`, which the check does not read (all prior releases have the conventional sidecar). The check's leg 2.1a hard-requires it (otherwise `FAIL 2.1a` for lack of a hash-checked tarball). I restored `cayleb-dsh-fleet-0.3.0.tgz.sha256` from the independently verified digest (`8173d13e…`, also matching the published `SHA256SUMS.strike` line and the installed-tree provenance) **before** the run; the check, tarball and installed profile were not modified. Logged in `00-staging-before.txt` (before/after listing, exact command, content, hashes). The check then printed `tarball cayleb-dsh-fleet-0.3.0.tgz (sha256 matches sidecar)`.
2. **Install record filename.** The Lead's message cites `.../s3-install-0.3.0/INSTALL-0.3.0: PASS`; that path is not a separate file. The verdict line `INSTALL-0.3.0: PASS` is the last line of `install-20261101T230709Z.log` in that directory (recorded in `41-provenance.txt`).

## Evidence

`/home/cayleb/workspace/harness-build/evidence/strike/s4b-stage2-installed/` — 54 files + `SHA256SUMS` (all verify).
Key: `stage2-check.out` (full output), `console.err` (0 B), `exit-code.txt`, `run-timestamps.txt`, `00-staging-before.txt`, `01-meta-before.txt`, `10-identity-before.txt` + `10-firstrun-status-before.json`, `20-firstrun-status-after.json`, `30-post-run-service-proof.txt`, `40-pass-table.txt` (all 60 PASS lines), `41-provenance.txt`, `README.md`.

Fences respected: no root actions; the check was run as-is; the installed profile and the service were not modified; secrets absent (no token/credential values in this evidence).
