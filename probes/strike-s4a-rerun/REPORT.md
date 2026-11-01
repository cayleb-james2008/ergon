# S4 part A re-run — package-alone install proof on the CORRECTED release

**Verdict: ACCEPT (S4A PASS)** — 49 PASS / 0 FAIL / 0 BLOCKED, runner exit 0.

- Checker: `checker-install` (independent; own harness; nothing changed in the artifact or the tree).
- Date: 2026-11-01 17:45–17:46 EST; fresh scratch root `/tmp/checker-s4a.lRmJgn` (host stopped at teardown, root kept); umask 027 (fleet-faithful); own dynamic MCP port **33123**.
- **Artifact: `strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz`, sha256 `d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab`** — the corrected build that will be installed.

## Artifact facts (re-confirmed on this digest)

- **104 regular members**, **101×0644 + 3×0755**, `package/package.json` = **`@cayleb/dsh-fleet 0.3.0`**, 21 `files[]` entries.
- Independent delta against the previous `b60019a0…` build (using the b60019a0 installed tree from the prior run as reference, whose 104/104 sha+mode equality to b60019a0 was itself proven by `first-run --status`): **exactly one changed member — `bin/rows-manifest.mjs`; 0 added, 0 removed; `rows.manifest.json` byte-identical (`1bcac986…`)**. This matches the stated change scope; the shipped manifest is untouched.

## Battery (identical to the prior S4A run, nothing weakened)

| Step | Result |
|---|---|
| fresh throwaway `DSH_HOME`, profile from default web template | `dump0` = 196 rows |
| install only the tarball (`dsh plugin … add --save-exact --ignore-scripts`) | **rc=0**, installed 0.3.0, bundles `[dsh-base, dsh-web-app, @cayleb/dsh-fleet]` |
| reviewed normalise block + independent assert | rc=0; **104/104 regular, nlink=1, exact archive mode, sha256 match, 0 missing/extra, dirs 0750** |
| compose | **214 = 196 baseline + 18 inserted, 0 duplicate ids**, all enabled, overlay values applied |
| host status | **ok=true, integrity.ok=true, failures=[], lanes_allowed=true**, peer uid 1000, selftest on, WAL |
| **`first-run --status` vs tarball** | **status_files=104 = tarball 104, missing=0, extra=0, sha/mode_mismatch=0** |
| MCP (own port 33123) | token op ok (never printed), **initialize 200** (2025-03-26, `fleet-mcp 0.3.0-mcp`, session issued), 202, **tools/list 9 tools**, wrong bearer 401 |

## Decisive lines (verbatim)

```
PASS  overlay profile composed 214 rows = 196 baseline + 18 inserted
PASS  no duplicate row ids in the composed profile (0 duplicates)
normalise-archive: @cayleb/dsh-fleet 0.3.0 104 regular members
assert: archive=104 files; installed=104 files; missing=0; extra=0; problems=0; dirs_wrong_mode=0
first-run-status: name=@cayleb/dsh-fleet version=0.3.0 dshVersion=0.2.2-alpha.2
status_files=104 tarball_members=104 missing=0 extra=0 sha/mode_mismatch=0
PASS  status ok=true
PASS  integrity.ok=true
PASS  lanes_allowed=true
PASS  initialize HTTP 200 protocol=2025-03-26 server=fleet-mcp 0.3.0-mcp session=issued
PASS  tools/list HTTP 200 9 tools: fleet.status, session.create, …
FAILS=0
S4A: PASS
```

## Caveats

- Same environment notes as the first S4A run: the DSH installation's pinned pnpm 11.28.5 is used through a one-binary shim (the desktop PATH shim cannot run from a harness CLI child); pnpm reused its content-addressed store (`duration_s=0`) but the first-run manifest comparison proves installed bytes == this exact tarball, sha256 for sha256.
- `tools/call session.create` was not exercised (known builder bug, outside the required proof). No root actions; `/var/lib/fleet`, `fleet-dsh` and the owner profile untouched. S4B (real install) still awaits the owner act.
- Scratch host's ephemeral web-UI tokens were redacted from captured logs; the MCP bearer token is handled in memory only and never printed or written.

## Evidence

`/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/s4a-rerun/` — 27 files + `SHA256SUMS` (all verify). Full transcript `logs/99-console.txt`; complete table `logs/30-full-table.txt`; artifact facts + delta `logs/00-corrected-artifact.txt`; core comparison `logs/21-firstrun-status.txt` + `frstatus.json`; corpus `logs/18-compose.txt` + `dump1.yml`.
