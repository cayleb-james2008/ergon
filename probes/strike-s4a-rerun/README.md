# S4A re-run on the CORRECTED release — fresh throwaway DSH_HOME

Verdict: **S4A PASS** — 49 PASS / 0 FAIL / 0 BLOCKED (runner exit 0). Checker: checker-install.
Artifact: `strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz`, sha256 `d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab` — the corrected build that will be installed.
Facts re-confirmed on this artifact: 104 regular members (101×0644 + 3×0755), `package/package.json` = `@cayleb/dsh-fleet 0.3.0`, 21 `files[]` entries.
Independent delta vs the previous `b60019a0…` build (using my proven b60019a0 installed tree as reference): **exactly one changed member `bin/rows-manifest.mjs`; 0 added, 0 removed; `rows.manifest.json` byte-identical (sha256 `1bcac986…`)** — consistent with the stated change scope.
Fresh scratch root: `/tmp/checker-s4a.lRmJgn`; umask 027 (fleet-faithful); own dynamic MCP port **33123**; pnpm 11.28.5 (DSH installation pin).

Battery (same as the previous S4A run, nothing weakened):
- install only the tarball: `dsh plugin --profile fleet add --save-exact --ignore-scripts <tgz>` → rc=0, installed 0.3.0, bundles `[dsh-base, dsh-web-app, @cayleb/dsh-fleet]`;
- reviewed normalise block → rc=0; independent assert **104/104 regular, nlink=1, exact archive mode, sha256 match, 0 missing/extra, dirs 0750**;
- compose: **214 rows = 196 baseline + 18 inserted, 0 duplicate ids**, all rows enabled, overlay values applied;
- host: `ok=true, integrity.ok=true, failures=[], lanes_allowed=true`, peer uid 1000, selftest on, WAL;
- **core:** `first-run --status` → version 0.3.0, **status_files=104 = tarball 104, missing=0, extra=0, sha/mode_mismatch=0** (installed tree == tarball, path+sha256+mode);
- MCP: admin-socket token op ok (never printed/written), `initialize` 200 (2025-03-26, `fleet-mcp 0.3.0-mcp`, session issued), notifications 202, `tools/list` 200 with 9 tools, wrong bearer → 401.

Files: `logs/99-console.txt` (full transcript), `logs/30-full-table.txt` (complete table), `logs/00-corrected-artifact.txt` (artifact facts + delta vs b60019a0), `logs/10-env.txt`, `logs/12-plugin-add.log`, `logs/13-installed-identity.txt`, `logs/14-pnpm-state.txt`, `logs/15-normalise.txt`, `logs/16-assert-normalised.txt`, `logs/17-overlay.yml`, `logs/18-compose.txt`, `dump1.yml`, `logs/19-host.log`, `logs/20-status.txt`, `logs/21-firstrun-status.txt`, `frstatus.json`, `logs/22-mcp-probe.txt`, harness scripts, `REPORT.md`, `SHA256SUMS`.
Scratch host's ephemeral UI tokens were redacted from captured logs; the MCP bearer token is handled in memory only.
