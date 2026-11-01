# S4 part A — independent package-alone install proof (scratch DSH_HOME)

**Verdict: ACCEPT (S4A PASS)** — 49 PASS / 0 FAIL / 0 BLOCKED, runner exit 0.

- Checker: `checker-install` (independent; own harness, not the Lead's gate script). No root actions; `/var/lib/fleet`, `/var/lib/fleet-drop`, `fleet-dsh` and the owner profile untouched.
- Date: 2026-11-01 17:27–17:29 EST; fresh scratch root `/tmp/checker-s4a.Khlcwt` (host stopped, root kept).
- Artifact: `strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz`, sha256 `b60019a0f3611c83fb13a374444c46b05092df89737f803661408fd7151a0b10` — the **current** 0.3.0 build (104 regular members: 101×0644, 3×0755; `@cayleb/dsh-fleet 0.3.0`).

## Proof chain (all from my own reads)

| Step | Result |
|---|---|
| Throwaway `DSH_HOME`, profile `fleet` from default web template | `dump0` = 196 `- id:` rows |
| `dsh plugin --profile fleet add --save-exact --ignore-scripts <tgz>` | **rc=0**; installed version 0.3.0; profile bundles = `[dsh-base, dsh-web-app, @cayleb/dsh-fleet]` |
| Reviewed normalise step (Task A block) on the scratch tree | rc=0; independent assert **104/104 regular, nlink=1, exact archive mode, sha256 match, 0 missing, 0 extra, all dirs 0750** |
| Composition (`--patch` scratch overlay + `--dump-config`) | **214 rows = 196 baseline + 18 inserted, 0 duplicate ids**; all 18 rows present+enabled; overlay values applied |
| Boot scratch host + control `{"op":"status"}` | **ok=true, integrity.ok=true, failures=[], lanes_allowed=true**, peer_uid 1000 accepted 1/ rejected 0, selftest=true, WAL |
| `dsh-fleet first-run --status` vs tarball | **status_files=104 = tarball 104, missing=0, extra=0, sha/mode_mismatch=0** — installed tree equals the tarball file-for-file |
| MCP facade (own dynamic port, 41699; 8789 was the Lead's listener) | admin-socket token ok (never printed); **initialize 200** (2025-03-26, server `fleet-mcp 0.3.0-mcp`, session issued); notifications 202; **tools/list 200, 9 tools**; auth.refuse 401 |

The delivered-tree comparison is the core: `first-run --status` reads the installed package root from the running host (via `import.meta.url`) and reports `{path, sha256, mode}` for the package's shipped files — 104/104 equal to the current tarball's members, modes included after the sanctioned normalise. No protected path was read.

## Caveats (not softened)

- Port 8789 was already held by the Lead's earlier scratch listener at run start; my final run used a dynamically chosen port **41699** (recorded free, then bound by my host) so there is no cross-talk.
- pnpm 11.28.5 (DSH installation pin) via a one-binary shim dir, because the desktop PATH pnpm shim cannot run from a harness CLI child; run mirrored the fleet service's `umask 027`. `npm_config_store_dir` was not honoured (pnpm chose `/tmp/.pnpm-store/v11`) — irrelevant, bytes were sha256-compared to the tarball.
- Scratch host's ephemeral web-UI tokens were scrubbed from captured logs; the MCP bearer token is handled in memory only and never printed or written.
- `tools/call session.create` not exercised (known builder bug, outside the required proof). S4B (real install) not attempted — blocked on the owner's root act.

## Evidence

`/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/s4a/` — 28 files + verified `SHA256SUMS`; full transcript `logs/99-console.txt`, complete table `logs/30-full-table.txt`, report `REPORT.md` (sha256 `6fe19a30ba2a002e2f1d180d13ec90c6311bd161334bcb8a1685aaca6474a0df`).

---

## Addendum 2026-11-01 17:46 ET — re-run on the CORRECTED release `d03ea2ac…`: PASS

The release artifact was corrected (change scope: only `bin/rows-manifest.mjs`; `rows.manifest.json` unchanged). Fresh re-run of the whole S4A battery on the corrected tarball **`d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab`** (104 members: 101×0644 + 3×0755, `@cayleb/dsh-fleet 0.3.0`; fresh throwaway `DSH_HOME=/tmp/checker-s4a.lRmJgn`, umask 027, own MCP port 33123):

- **49 PASS / 0 FAIL / 0 BLOCKED, `S4A: PASS`** — every result reproduced: install rc=0 (0.3.0), normalise + independent assert 104/104 exact modes, compose **214 = 196 + 18** with 0 duplicate ids, host `ok=true/integrity.ok=true/lanes=true`, **`first-run --status` 104/104 with 0 missing/extra and 0 sha/mode mismatch**, MCP initialize 200 + 9 tools + 401.
- Independent delta check: vs the prior `b60019a0…` build the corrected tarball changes **exactly `bin/rows-manifest.mjs`** and nothing else (0 added/removed; `rows.manifest.json` byte-identical) — consistent with the stated scope.
- Evidence: `evidence/strike/checker-verify/s4a-rerun/` (27 files + verified `SHA256SUMS`; `99-console.txt`, `30-full-table.txt`, `REPORT.md`). **Verdict unchanged: ACCEPT.**
