# S4B — platform acceptance check (Stage 2) against the INSTALLED 0.3.0 host

Verdict: **PASS** — `EXIT=0`, `summary: pass=60 fail=0`, `STAGE 2: PASS` (165 s).
Checker: checker-install (independent; ran the plugin's own UNMODIFIED check; no root actions; did not modify the check or the installed profile).

Artifact under test (installed): `@cayleb/dsh-fleet 0.3.0`, tarball sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070` (drop copy byte-identical to release `dist/`), 104 members. Installed-tree readback: version 0.3.0, 104 files, modes {0644,0755}.
Check: `strike-release/checks/stage2-check.sh`, sha256 `7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95` (matches the expected hash — same script that passed S1).
Command run exactly: `OUT=<this dir> FLEET_CLI=<release>/package/bin/dsh-fleet.mjs bash <release>/checks/stage2-check.sh`.

Pre-run identity: fleet-dsh active, InvocationID baf7bf93…, MainPID 510804; control status ok/integrity ok, selftest on, lanes allowed; `first-run --status` = 0.3.0 / 104 files.
Post-run identity (independent, after the check's own 2.11 SIGKILL/restart): **active**, InvocationID **bc98f82d…**, MainPID **518634** (check's own line: `MainPID 510804 -> 518634; kill rc=0; up again at +3s`), control status ok/integrity ok/failures [], lanes allowed; `first-run --status` = 0.3.0 / 104 files. The check's EXIT trap did not need to start the unit (0 `trap: starting` lines).

## Staging note (not a product defect)
The drop dir had the 0.3.0 tarball but **no conventional `<tgz>.sha256` sidecar**; the digest was published only in `SHA256SUMS.strike` (a filename the check does not read), and the check's leg 2.1a hard-requires `<tgz>.sha256`. I restored `cayleb-dsh-fleet-0.3.0.tgz.sha256` from the independently verified digest (8173d13e…, also matching the published SHA256SUMS.strike line and the installed-tree provenance) before the run — recorded in `00-staging-before.txt` (before/after listing, content, hashes). The check, tarball and installed profile were not modified. With the sidecar present, 2.1a printed `tarball cayleb-dsh-fleet-0.3.0.tgz (sha256 matches sidecar)` and passed.
Also: the named install record `.../s3-install-0.3.0/INSTALL-0.3.0` is not a separate file; the verdict `INSTALL-0.3.0: PASS` is the last line of `install-20261101T230709Z.log` in that dir (recorded in `41-provenance.txt`).

## Files
- `stage2-check.out` — full check output (the script's own tee); `console.out` byte-identical; `console.err` empty (0 bytes).
- `exit-code.txt`, `run-timestamps.txt` — EXIT=0, 18:08:56→18:11:41 EST, 165 s.
- `00-staging-before.txt` — missing-sidecar capture + the restoration record.
- `01-meta-before.txt` — command, check sha256, release tree head/branch/status, package.json + FLEET_CLI hashes, tarball hashes, service before.
- `10-identity-before.txt` + `10-firstrun-status-before.json` — installed identity before.
- `20-firstrun-status-after.json` — installed identity after.
- `30-post-run-service-proof.txt` — systemctl + control status + restart evidence after the run.
- `40-pass-table.txt` — all 60 PASS lines verbatim.
- `41-provenance.txt` — drop vs dist tarball, tarball vs release-tree package, installed CLI hashes, install-record location.
- `SHA256SUMS` — sha256 of every file here (except itself).
