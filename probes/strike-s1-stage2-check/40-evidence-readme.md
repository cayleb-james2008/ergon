# S1 evidence pack — Stage 2 acceptance check against the installed @cayleb/dsh-fleet

Checker: checker-install (independent fresh-context). Verdict: **ACCEPT**.

- Command: `OUT=<this dir> FLEET_CLI=<tree>/package/bin/dsh-fleet.mjs bash <tree>/checks/stage2-check.sh`
- Result: `EXIT=0`, `summary: pass=60 fail=0`, `STAGE 2: PASS`; duration 157 s.
- Check script sha256: `7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95` (matches expected)
- Tree: `/home/cayleb/workspace/harness-build/dsh-fleet-integration`, branch `integration/0.2.2`, commit `cf93107bc6e31ed265332b005d75b63d8acb300e`, clean before AND after.
- package/package.json sha256: `ef93187aec289d70bd29446c66c5b63abe44f78f4d2342871af2338274b4f52c`
- Tarball used (newest `cayleb-dsh-fleet-*.tgz` in /var/lib/fleet-drop): `cayleb-dsh-fleet-0.2.2.tgz`, sha256 `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177` (matches sidecar; byte-identical to accepted-tree dist tarball).
- Post-run service: `fleet-dsh` active/running, MainPID 427587, InvocationID `9e3929d0d99c45ca98ff0d01fd275eb6`; control `status` -> ok:true, result.integrity.ok:true, integrity.failures [] . The check's own 2.11 SIGKILL/restart left the unit up; its EXIT trap therefore did not need to start it.

## Files
- `stage2-check.out` — full check output (the script's own tee); `console.out` byte-identical; `console.err` empty (0 bytes).
- `exit-code.txt`, `run-timestamps.txt` — exit status and start/end epoch+ISO, duration.
- `00-meta-before.txt` — command line, identity of tree/commit/script/package.json, service-before, tool versions.
- `01-tarball-provenance.txt` — drop vs dist tarball sha256 + cmp; notes on `dsh-fleet-admin version`.
- `10-identity-baseline.txt` — read-only identity snapshot taken during the run (baseline).
- `20-post-run-service-proof.txt` — post-run systemctl facts, control-socket status (ok/integrity), admin version/status.
- `30-pass-table.txt` — all 60 PASS lines verbatim.
- `ctl.jsonl`, `pre-status.json`, `pre-cli-status.txt`, `2.*`, `inv-*.jsonl`, `cleanup-cancel.jsonl` — per-leg raw evidence produced by the check.
- `SHA256SUMS` — sha256 of every file in this pack (except itself).

## Limitations / anomalies (nothing softened)
- The installed package tree under /var/lib/fleet is mode 0750 fleet:fleet and is **not readable as cayleb**; this pack therefore records service facts and the wrapper's fixed verbs only, not installed-tree contents. No guess was made about file bytes.
- `dsh-fleet-admin version` prints the DSH pin `0.2.2-alpha.2`, NOT the plugin version. Plugin version 0.2.2 is evidenced by the tarball the check selected (sha256 matches sidecar) and by the accepted-tree package.json.
- A proposal-only identity snapshot caller is staged at `/run/dsh-fleet-id-cf93107/` (header: "PROPOSAL ONLY ... do not ... run ... without the separate reviewed owner authorization"). It was **not run** by this checker; the owner/Lead must authorise it. Its manifest pins 82 members and archive sha256 37e3a542… (same tarball).
- `run-timestamps.txt` has a duplicated `start_epoch` line (cosmetic wrapper artifact; both values identical: 1793569402).
- No FAIL, no NOT-RUN, no SKIP lines. No FAIL string in any PASS verdict.
