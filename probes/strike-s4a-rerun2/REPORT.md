# S4 part A on the FINAL artifact — install proof + guided-first-run credential proof

**Verdict: ACCEPT (S4A PASS)** — 63 PASS / 0 FAIL / 0 BLOCKED, runner exit 0.

- Checker: `checker-install` (independent; own harness; nothing modified).
- Date: 2026-11-01 18:04–18:06 EST; fresh scratch root (removed after capture for secret hygiene); umask 027; own dynamic MCP port 35849.
- **Artifact: `strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz`, sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070`** — the final release.

## Artifact facts and delta (independently checked)

- **104 regular members, 101×0644 + 3×0755**, `package/package.json` = `@cayleb/dsh-fleet 0.3.0`.
- Delta vs `d03ea2ac` (reference = that artifact's installed tree, proven 104/104 by `first-run --status` in the previous S4A run): **exactly two changed members — `src/firstrun/capabilities.mjs` and `src/firstrun/credentials.mjs`**; 0 added, 0 removed; `rows.manifest.json` byte-identical (`1bcac986…`). This matches the stated scope.
- **Rails vs the 0.2.2 artifact** (extracted independently from the readable `/var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz`): **all 11 rail areas byte-identical file-for-file** — core 15/15, orders 17/17, policy 9/9, ledger 4/4, lessons 7/7, sandbox 4/4, egress 6/6, fsguard 4/4, housekeeper 4/4, client 2/2, presets 4/4. The "rails unchanged since 0.2.2" claim holds.

## Battery (all from my own reads)

| Step | Result |
|---|---|
| install only the tarball (`dsh plugin … add --save-exact --ignore-scripts`) | **rc=0**, installed 0.3.0, bundles `[dsh-base, dsh-web-app, @cayleb/dsh-fleet]` |
| reviewed normalise block + independent assert | rc=0; **104/104 regular, nlink=1, exact archive mode, sha256 match, 0 missing/extra, dirs 0750** |
| compose (`--dump-config` with scratch overlay) | **214 = 196 baseline + 18 inserted, 0 duplicate ids**, all rows enabled, overlay values applied |
| boot + control `{"op":"status"}` | **ok=true, integrity.ok=true, failures=[], lanes_allowed=true**, peer uid 1000 accepted/0 rejected, selftest on, WAL |
| **`first-run --status` vs tarball** | **status_files=104 = tarball 104, missing=0, extra=0, sha/mode_mismatch=0** |
| MCP facade (own port) | admin-socket token op (never printed), **initialize 200** (2025-03-26, `fleet-mcp 0.3.0-mcp`, session issued), 202, **tools/list 9 tools**, wrong bearer 401 |

## NEW — `first-run --credentials` under a real PTY (fake key)

Driven through a real PTY (`pty-credentials.py`): 4 prompts seen; the fake value sent at the first prompt, bare Enter (skip) at the other three. Host: the scratch 0.3.0 sibling of the same run.

- **(a) old bug gone:** exit 0; no `needs a terminal secret reader` anywhere; summary `status: written` + `stored: OPENCODE_GO_API_KEY via control-op`. Code evidence: the old d03ea2ac artifact threw `new Error('an interactive credential run needs a terminal secret reader')` when no reader was injected; the final artifact wires `createTTYSecretReader({input, output})` as the production path and sets raw mode **before** the prompt.
- **(b) stored correctly:** scratch `$DSH_HOME/.credentials.yaml` mode **0600**; document `version: 1` with a `refs:` section; **the fake value occurs exactly once**; `refs` holds **exactly one entry, `OPENCODE_GO_API_KEY`, equal to the injected value**. Pre-state proof: the store existed before the run (host-created `version`/`records`, a browser-session grant) with **no refs keys**; the run added the ref (pre-run `OPENCODE_GO_API_KEY` ref: absent).
- **(c) no leak:** the fake value appears **nowhere** outside the store — PTY transcript, host log, fleet logs, first-run report dirs, whole scratch tree: **0 hits**. The PTY transcript (filed verbatim) contains prompts and `accepted (not echoed)` but **no echoed value**; the summary is secret-free.

## Decisive lines (verbatim)

```
PASS  overlay profile composed 214 rows = 196 baseline + 18 inserted
PASS  no duplicate row ids in the composed profile (0 duplicates)
assert: archive=104 files; installed=104 files; missing=0; extra=0; problems=0; dirs_wrong_mode=0
first-run-status: name=@cayleb/dsh-fleet version=0.3.0 dshVersion=0.2.2-alpha.2
status_files=104 tarball_members=104 missing=0 extra=0 sha/mode_mismatch=0
PASS  (a) PTY transcript has no 'needs a terminal secret reader' error
PASS  (a) summary reports 'status: written'
PASS  (a) summary reports 'stored: OPENCODE_GO_API_KEY via <transport>'
PASS  (b) store mode is 0600 (got 0600)
PASS  (b) fake value occurs exactly once in the store (got 1)
PASS  (b) refs section holds exactly one entry, OPENCODE_GO_API_KEY (got 1)
PASS  (b) the OPENCODE_GO_API_KEY ref did not exist before this run
PASS  (c) fake value appears nowhere outside the store (transcript/logs/reports/scratch tree; hits=0)
verify-credentials: PASS — 0 failed
FAILS=0
S4A: PASS
```

## Caveats

- The host's `dsh-credentials-local` store pre-exists at boot with a `records:` section; the in-host control op added the `refs` entry (the store format is the native one, not the CLI's direct-write shape). Both are exercised correctly; the proof is about the value landing once in `refs` with mode 0600, which holds.
- Transport used: `control-op` (the in-host `first-run-credential-set`), i.e. the production path for a live host. The CLI's direct-write/staging fallbacks were not needed.
- pnpm environment notes unchanged (installation-pinned 11.28.5 via shim; store reuse — installed bytes proven by the sha256 comparison).
- Hygiene: scratch host stopped, scratch root removed, fake credential value and scratch UI tokens never filed; MCP bearer token never printed. No root actions; `/var/lib/fleet`, `fleet-dsh` and the owner profile untouched. S5 is checker-verify's task; I did not run it.

## Evidence

`/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/s4a-rerun2/` — 36 files + `SHA256SUMS` (all verify). Full transcript `logs/99-console.txt`; complete table `logs/30-full-table.txt`; artifact facts/delta `logs/00-final-artifact.txt`; rails check `logs/01-rails-vs-022.txt`; bug-fix evidence `logs/02-credentials-bug-fix-evidence.txt`; credentials proof `logs/23-*`, `24-*`, `25-pty-transcript.txt`, `26-store-metadata.txt`; core comparisons `logs/21-firstrun-status.txt` + `frstatus.json`, `logs/18-compose.txt` + `dump1.yml`.
