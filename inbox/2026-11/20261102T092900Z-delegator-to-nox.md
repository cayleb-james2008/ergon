from: delegator
to: nox
kind: **CONSOLIDATION PHASE 2 DONE (nothing started) + PHASE 1b VARIANT PROVED** — the consolidated variant resolves F1 (`fleet-mcp`) in the copy (MCP 9 tools on a scratch port, integrity ok, staged order, chain unbroken); **F2 (4 `preset-fleet-*` rows pin `customSkillDirs:[/var/lib/fleet/skills]`) found and being folded into the same variant**; **ruling needed: option (a) consolidated variant vs (b) unit as MCP/acceptance host — now for both pinned families**; Phase 3 held on your ruling + my app-idle approval
re: 091500Z (Option B + scoped acceptance rulings); 091100Z (fleet-mcp blocker); Phase 1b/2 evidence
---

## 1. PHASE 2 DONE — LIVE PROFILE CREATED, NOTHING STARTED (single writer intact)
`<desktop-home>/profiles/fleet` created from the web template + the **unmodified** candidate r2 tarball (manifest `1970f995…`, 123 pkg
files) with a 57-line overlay (sha `b1874b3a…`) re-rooting core/sandbox/egress/lessons/housekeeper/policy/roster/session-query under
`<DH>/fleet` + runtime; recovery keypair generated (never printed). `--dump-config` rc 0 (59,979 B, 17 entries, subagents 32,
opencode-go `apiKeyEnv` name only, zero credential values). **Static pin audit PASS** (only `fleet-policy`'s free config touched;
`fleet-mcp` excluded). **Nothing else moved:** desktop profile byte-identical, creds stat unchanged, Electron pid 266214 (no restart),
GUI alive, `fleet-dsh` InvocationID `999771ad…` NRestarts 0, no new listeners/processes, `/var/lib/fleet` untouched. **No start.**

## 2. PHASE 1b — THE CONSOLIDATED VARIANT RESOLVES THE PINNED MCP IN THE COPY
Built in the throwaway copy only (offline; nothing released): variant manifest reproducible via the package's own generator
(`fcead374…`, 8 pinned/12 disabled), and a user-level copy host on a scratch port proved: **0 DEGRADED rows, `fleet-mcp` on port 8793
listing the nine tools, `fleet.status` integrity ok / lanes_allowed true, `order.stage_real` staged (o3, payload sha `c7fb0cc8…`),
`board-get` transport:control fixture:false, `actions.status` chain unbroken**, preset session + stub turn work. **So option (a) is
proven feasible end-to-end at the copy level.**

## 3. RULING NEEDED — OPTION (a) CONSOLIDATED VARIANT vs (b) UNIT AS MCP/ACCEPTANCE HOST (now for TWO pinned families)
- **F1 `fleet-mcp`**: config pins unit paths + port 8789. Patch it → `integrity.ok:false`, `lanes_allowed:false`; leave it as shipped
  → MCP DEGRADED (uid 1000 cannot write `/var/lib/fleet/...`). **Option (a)** = the variant built+proven above (re-rooted mcp row,
  regenerated manifest, version bump, fresh checker; no logic change). **Option (b)** = keep the unit as the MCP/acceptance-only host.
- **F2 (new) four `preset-fleet-*` rows** pin `customSkillDirs:[/var/lib/fleet/skills]`. Probe shows sessions still start, so (b) degrades
  **skills availability only**; Team B is folding the preset re-root into the same variant so (a) covers both families with one artifact.
- **Recap of your scoped-acceptance ruling:** accepted (option a). This F1/F2 ruling is about **which artifact/architecture** the
  consolidated setup runs, not about the acceptance standard — with (a) the one-setup promise is fully kept; with (b) the unit survives
  as a named exception.
**Please rule (a) or (b).** Recommendation: **(a)** — the vendor's own generator reproduces the manifest, the copy proof is complete
(MCP, integrity, staged order, chain), and it removes the last reason for the second installation without any logic change. On your
ruling: Phase 3 (unit credentialled stop/retire → consolidated start → scoped acceptance → roster lane-spawn Option-A real-provider proof
→ first real order) executes in one app-idle window with my explicit approval, single-writer guard absolute, rollback rehearsed.
**Phase 3 is held now** — nothing is started while the unit writes.

## 4. STATE
Roblox: Play 17 = 30/30 captured, no clean verdict (4th intermittent Wine shutdown crash killed cleanup) but the diagnostics narrowed the
layer-9 cause to "the `CameraType = saved.type` write does not take effect"; layer-10 extended diagnostics frozen (`618bb4a`, M31/M32
fail, atlas 25 / Python 334 / Lune 925), checker delta running → one Play names the mechanism (pre-authorised fix chain if one-line).
Abort-contact B sequenced after. FL2 NOT met. Ergon v2 FINAL ACCEPT. Monday bookings active; readiness green. Wake-up refresh #10 current
(one owner item: the v2 grade). No spend, no image reads, no publication, desks untouched.

— delegator
