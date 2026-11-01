# Pre-install assurance: the 0.3.0 artifact's rails are byte-identical to the Stage-2-accepted 0.2.2 artifact

Run 2026-11-01T22:55Z by the strike Lead from the two shipped tarballs (no source trees, no assumptions):
- A = `cayleb-dsh-fleet-0.2.2.tgz` sha256 `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177` (the artifact whose installed
  result passed the plugin's own Stage 2 acceptance check 60/60, S1 this session)
- B = `cayleb-dsh-fleet-0.3.0.tgz` sha256 `d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab` (the corrected release
  staged for the fleet install)

## Result
- **Rails byte-identical**: `src/core`, `src/orders`, `src/policy`, `src/ledger`, `src/lessons`, `src/sandbox`, `src/egress`,
  `src/fsguard`, `src/housekeeper`, `src/client` and all four `presets/*.patch.yml` are identical in A and B (`diff -rq` empty for
  each). The board/leases/ledger/policy/containment code that the acceptance check exercises is the exact code Stage 2 accepted.
- **`rows.manifest.json` byte-identical** (sha256 `1bcac9862a9b6c9723c27b193b3a9f22ab9b07f9d30e7fef862d01d194d59ee9`).
- **22 files added**: `src/firstrun/` (13), `src/mcp/` (7), `bin/mcp-selftest.mjs`, `mcp.row.yml` — the guided first run and the MCP
  facade, both additive.
- **4 files changed**: `bin/dsh-fleet.mjs` (the documented `first-run` dispatch in front of fleet-core's CLI), `bin/rows-manifest.mjs`
  (the shipped generator's handling of an additional bundle-patch file; its output is byte-identical to the shipped manifest),
  `cordis.patch.yml` (one inserted row, `fleet-firstrun`), `package.json` (version, exports/files/bin entries, the extra bundle patch).
- No removed files. No change in member modes (101×0644 + 3×0755 in both artifacts).

## What this means
The acceptance check's rails legs run against identical code on the installed 0.3.0 output; the only new boot surface is the two
inserted rows (`fleet-firstrun`, `fleet-mcp`), whose behaviour was already proven on real scratch boots (integrity ok, lanes
allowed, first-run --status, 15/15 MCP PASS). Residual install risk is therefore the install/normalise step itself, already
exercised end to end on throwaway profiles by the independent S4A check (49/49) and the wrapper's own normalise logic
(104/104, negative controls fail closed).

Reproduce: extract both tarballs and `diff -rq A/package B/package`; loop `diff -rq` over the ten rail directories and `presets`.
