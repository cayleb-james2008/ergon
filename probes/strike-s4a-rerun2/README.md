# S4A on the FINAL artifact 8173d13e — install proof + guided-first-run credential proof

Verdict: **S4A PASS** — **63 PASS / 0 FAIL / 0 BLOCKED** (runner exit 0). Checker: checker-install.
Artifact: `strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz`, sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070`.
Facts re-confirmed: 104 regular members (101×0644 + 3×0755), `@cayleb/dsh-fleet 0.3.0`.
Independent delta vs d03ea2ac (reference = d03ea2ac installed tree proven 104/104 earlier): **exactly two changed members** — `src/firstrun/capabilities.mjs`, `src/firstrun/credentials.mjs`; 0 added/removed; `rows.manifest.json` unchanged. Independent check that the rails are byte-identical to the 0.2.2 artifact (extracted from the readable `/var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz`): **all 11 rail areas match file-for-file** (core 15, orders 17, policy 9, ledger 4, lessons 7, sandbox 4, egress 6, fsguard 4, housekeeper 4, client 2, presets 4).
Fresh scratch root `/tmp/checker-s4a.BD0g5m` (removed after evidence capture for secret hygiene); umask 027; own dynamic MCP port 35849; pnpm 11.28.5 (DSH installation pin).

Battery (same as previous S4A runs, plus the new credentials check):
- install only the tarball → rc=0, installed 0.3.0, bundles `[dsh-base, dsh-web-app, @cayleb/dsh-fleet]`;
- reviewed normalise block → rc=0; independent assert 104/104 exact archive modes, nlink=1, 0 missing/extra, dirs 0750;
- compose **214 = 196 + 18**, 0 duplicate ids, all rows enabled, overlay values applied;
- host **ok=true / integrity.ok=true / failures=[] / lanes_allowed=true**, peer uid 1000, selftest, WAL;
- **`first-run --status` 104/104** = tarball, missing=0 extra=0 sha/mode_mismatch=0;
- MCP: initialize 200 (2025-03-26, fleet-mcp 0.3.0-mcp), notifications 202, tools/list 9 tools, wrong bearer 401;
- **NEW: `first-run --credentials` under a real PTY** with a fake key (4 prompts; value at the first, skips after):
  - (a) exit 0, no `needs a terminal secret reader` error, summary `status: written`, `stored: OPENCODE_GO_API_KEY via control-op`;
  - (b) the fake value is in the scratch `$DSH_HOME/.credentials.yaml` **refs exactly once** (`OPENCODE_GO_API_KEY`), store mode **0600**, document `version: 1` with a `refs:` section; pre-run store existed (host-created `version/records`, no refs keys) and had **no** OPENCODE_GO_API_KEY ref — the run added it;
  - (c) the fake value appears **nowhere** outside the store: PTY transcript, host log, fleet logs, first-run report dirs, scratch tree — 0 hits. PTY transcript filed verbatim (no value, no echo).

Harness: `run-s4a.sh` (battery), `pty-credentials.py` (real PTY driver; value read from a 0600 file, never printed), `verify-credentials.py` (a/b/c proofs incl. pre-state), `mcp-probe.py`, `compare-frstatus.py`, `check-compose.py`.
Key logs: `logs/99-console.txt` (full transcript), `logs/30-full-table.txt` (complete table), `logs/00-final-artifact.txt` (digest + delta), `logs/01-rails-vs-022.txt`, `logs/02-credentials-bug-fix-evidence.txt` (old had the throw; final wires the default reader and sets raw mode before the prompt), `logs/23-*`/`24-*`/`25-pty-transcript.txt`/`26-store-metadata.txt`, `logs/21-firstrun-status.txt` + `frstatus.json`, `logs/18-compose.txt` + `dump1.yml`, `logs/20-status.txt`, `logs/22-mcp-probe.txt`, `SHA256SUMS`.
Hygiene: scratch host stopped; scratch root removed; fake value and scratch UI tokens never filed (transcript verified clean, other logs scrubbed); MCP bearer token never printed. Nothing outside the scratch environment was touched.
