# board/probes/checker-031 — evidence root for the fleet 0.3.1 acceptance battery

**Status: PREP ONLY (2026-11-02T00:53Z). No runs against the installed service, no install, no restart.**
The 0.3.1 candidate artifact does not exist yet; strike owns the build/install line. This directory holds the
independent checker's (`checker-031`) evidence once the Lead opens the candidate phase.

Plan of record: [`board/reports/checker-031-battery-plan.md`](../../reports/checker-031-battery-plan.md).

## Layout (one directory per leg; every command writes here with cwd + timestamp + exit code)

| Dir | Leg | Contents (expected) |
|---|---|---|
| `00-preflight/` | G — candidate/staging gates | tarball+sidecar+drop+dist hashes, release-tree git state, wrapper hash, `stage2-check.sh` hash, install log excerpt, unit state before the battery |
| `01-provenance/` | P — provenance | `tar` member/mode listing, `package.json` version, `first-run --status` (before), `compare-frstatus` output, derived-count lines from the install log, release-tree↔tarball file equality |
| `02-installed-output/` | I — installed-output acceptance | the unmodified `stage2-check.sh` fresh `OUT` (console.out, console.err, exit-code, pass table), identity before/after, control status before/after |
| `03-roster-doctrine/` | R — roster/doctrine | tarball roster listing, `SHIPPED_MODELS`, `policy-get rates` JSON, composed `dump-config` excerpts (`llm-pi-ai`, `subagent`, `fleet-mcp`, `fleet-roster`), rows facet, `order.stage_real` refusal + `order-list` readback |
| `04-mcp/` | M — MCP surface | listeners, `mcp-selftest --json`, unchanged accepted `mcp-verify.mjs` battery output, auth-refusal + action-log scope, token/PII scan |
| `05-rails-rollback/` | X — rails/rollback | 11 rail-area `diff -rq` outputs, full member-set diff vs 0.3.0, rollback artifact checks + install-script rollback branch |
| `06-verdict/` | verdict | `REPORT.md`, per-leg ACCEPT/REJECT table, `SHA256SUMS`, token scan, machine verdict line |
| `scripts/` | helpers | read-only helpers: `compare-frstatus.py` (accepted S4A pattern, adapted), `tar-members.py`, `admin-op.mjs`, and the accepted `mcp-verify.mjs` copied **unchanged** with its sha256 recorded |

## Rules for this evidence root

- Every file is created by the checker's own commands; no maker-supplied evidence is copied in as proof (it may be
  cited only as the accepted baseline).
- No credential value, bearer token, or owner key ever lands here; probes read the MCP token from the admin socket into
  memory and redact output. A token/PII scan of the whole root is part of the verdict.
- `SHA256SUMS` is regenerated at the end and verified (`sha256sum -c`).
- Timestamps are UTC (`date -u +%FT%TZ`); every command line, cwd and exit code is recorded in the leg's log.
- This directory is empty of evidence until the Lead opens the candidate phase; the presence of this README is the only
  prep artifact.
