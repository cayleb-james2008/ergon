from: delegator
to: nox
kind: **CONSOLIDATION PHASE 1 COMPLETE + ONE MATERIAL BLOCKER needing your ruling** — `fleet-mcp` is integrity-pinned, so it cannot be re-rooted to a user-level home without breaking integrity (and left as shipped it degrades under uid 1000); options: (a) consolidated variant build (Team B's recommendation) or (b) keep the unit as MCP/acceptance-only host; Phase 1b offline prototype running; Phase 2/3 still held on your two earlier rulings
re: 090500Z (consolidation plan; acceptance delta + interpretation rulings); Phase 1 proof
---

## 1. PHASE 1 PROOF (offline, throwaway credential-less copy; no live write, restart, unit stop or credential value)
- `fleet --from-default-profile web` -> profile bundles [dsh-base, dsh-web-app]; `plugin --profile fleet add cayleb-dsh-fleet-0.3.1.tgz` ->
  rc 0, bundles [..., @cayleb/dsh-fleet]; installed tree **byte-identical to the tarball** (123 files, diff rc 0), rows manifest
  `1970f995...`, package.json `631eaeb1...`.
- `--dump-config` (59,749 B): **15 fleet-* rows compose**; fleet-roster with re-rooted paths; **subagent 32**; llm-pi-ai
  `opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY }` **with no value**; overlay re-roots all non-pinned rows under `<home>/fleet` and
  sockets under `$XDG_RUNTIME_DIR`.
- Scratch user-level host: control `status ok / integrity ok / rows 8-12 / failures [] / lanes_allowed true`; `policy-get roster.models`
  = doctrine (lead sonnet-5-5, builder+researcher opencode-go/deepseek-v4.1-flash, checker haiku-5-5); `policy-get rates` has the
  opencode-go row (0.15/0.6/0.003); `providers-get` route present, credential absent in the copy (correct); `first-run --status`
  0.3.1/123 files; state layout `<home>/fleet/{state,log,work,checks,backups,dsh,state/mcp}` + fleet.db + mcp/token created.
- **Untouched:** profiles/desktop/**, `.credentials.yaml`, `~/.dsh/**`, `/var/lib/fleet/**`, unit files, OpenCode dirs, strike door.
  Exact Phase-2 delta is file-by-file in the report; rollback script included.

## 2. THE ONE MATERIAL BLOCKER — RULING NEEDED
**`fleet-mcp` is integrity-pinned** (name + config hashes). Patching its config to a user-level home -> `integrity.ok:false`,
`lanes_allowed:false`, `failures:["rows fleet-mcp: config changed"]`. Leaving the shipped row -> MCP **DEGRADED**
`EACCES mkdir /var/lib/fleet/state/mcp|log` (uid 1000 cannot write unit paths) and not listening. **Every other fleet row re-roots
cleanly.** Options:
- **(a) RECOMMENDED — build a consolidated variant:** `mcp.row.yml` re-rooted to `<desktop-home>/fleet` + runtime socket/port,
  `rows.manifest.json` regenerated with the new hashes, version bump, fresh checker; no logic change, offline-safe, artifact released
  only after your ruling. This keeps the product promise (one setup, one plugin) and Team B is prototyping it in the throwaway copy now.
- **(b) Keep the `fleet-dsh` unit as the MCP/acceptance-only host** — an explicit exception to "one setup" for MCP + the unmodified
  acceptance; the fleet profile would still be the plugin's home for boards/roster, with the unit serving MCP.
**Please rule (a) or (b).** Phase 2 (live `fleet` profile create) and Phase 3 (unit stop/retire + consolidated start) stay held until
you rule, together with the two earlier asks: the **acceptance delta** (scoped consolidated acceptance vs unit as acceptance-only
exception) and the **interpretation** (sibling `fleet` profile under the desktop home = "the one setup"?). Phase 1b is producing the
scoped-acceptance command set so that ruling lands on evidence.

## 3. STATE
Roblox: layer-9 diagnostics delta checker-ACCEPTED; atlas Play 17 running now in the current recovered opening to name the camera-restore
cause (pre-authorised one-line-fix chain if it is one-line). Abort-contact B sequenced after. FL2 NOT met. Ergon v2 FINAL ACCEPT.
Monday bookings re-armed/verified; readiness green. No spend, no image reads, no publication, desks untouched.

— delegator
