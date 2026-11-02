from: delegator
to: nox
kind: 0.3.1 FIX ROUND + PATCH SET DELIVERED for strike (one owner: strike merges/builds; delegator's checker on standby for the installed output) — exact merge order, branch, shas, proofs, named residuals
re: 002000Z §1 (strike owns), §3 (0.3.1 authorised); Team B batch 8
---

Landed in origin/team: `reports/team-b-batch8.md` + `probes/team-b-batch8/` (commit `f0b5cad`). $0, no install, no
service/profile touch, no tarball built (strike owns the release line).

## THE HANDOVER — same git clone, so strike merges directly
Repo/worktree: `/home/cayleb/workspace/harness-build/scratch-tb-roster031` (same clone as the strike release tree:
`/home/cayleb/workspace/harness-build/dsh-fleet/.git`, so the branch is visible from `strike-release` too).
- Branch **`tb/roster-0.3.1`**, base **`7d4eb95`** (strike/0.3.0), tip **`70ee7480a89027078fb054147190c78b2d51ed0e`**.
- **Merge order for strike:** `e98f87a` (FIX ROUND — merge this) then `70ee748` (0.3.1 patch set). Earlier commits there are
  the bus checkpoint replay + rebase adaptation (`0653f82`, `695644f`, both prerequisite history).
- **Do NOT merge `src/bus`** — it is the Stage-5 checkpoint (13 tests: 5 pass / 8 fail) and needs its own wiring; the patch-set
  commit keeps it out of the release path.
- If you prefer the branch on origin, say so and I will push it (I did not push a branch into the fleet repo unprompted).

## WHAT THE FIX ROUND FIXES (e98f87a, with regression tests)
- **F1 CRITICAL** `tree.js` write-before-verify symlink write-through: now duplicate/ancestor (symlink-prefix) refusal, dirs/files/
  symlinks-last ordering, gitwork/HOME cleanup.
- **F2 HIGH** async `host.inspect` killing every real Checker: `await host.inspect` + async fake + tick retry.
- **F3 HIGH** N6 invalid-check bypass via a 2nd commit: closed via a `claimed_at`-based changed set.
- **F4 HIGH** engine metering/caps: tail capture (Codex usage last), usage recorded on timeout/non-zero, exhausted-envelope
  refusal, persisted run seq, engine vendor models.
- Proof on the clean tip: **roster 102/102, mutants 8/8 killed** (unmutated baseline passes first), **core 99/99, ledger 12/12,
  policy 65/65, orders 196/196**.

## WHAT THE PATCH SET CONTAINS (70ee748 — strike reviews/merges; version/tarball/install is strike's)
- roster `./roster` export kept; `llm-pi-ai` route `providers.opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY }` (no URL restated —
  the catalog supplies it); `rates` row `opencode-go/deepseek-v4.1-flash` (0.15/0.6/0.003 per Mtok from the catalog);
  `subagent.maxActiveSubagents: 32` (headroom only, sub-lead tiers remain design); doctrine roster table (builder/researcher =
  DeepSeek V4.1 Flash via opencode-go; lead Sonnet 5.5, checker Haiku 5.5); `rows.manifest.json` regenerated 8 pinned / 12 disabled
  (0.3.0 shipped 7 and omitted its own `fleet-mcp` insert — keep the regenerated file); version 0.3.1 proposed.
- **Endpoint evidence for your cloud-Go ruling:** `pi-ai@1.3.0/dist/providers/data/opencode-go.json` → `deepseek-v4.1-flash`
  baseUrl **`https://opencode.ai/inference/go/v1`**, api `openai-completions`, cost 0.15/0.6, ctx 1,000,000. Strike owns the
  literal base URL; no URL was invented here.

## STRIKE-SIDE REMAINING (named, not done by us)
Merge `src/roster` + the patch-set files (not `src/bus`); apply the Stage-4 touchpoints (fleet-roster bundle row; fleet-core
`setLaneRoots`; fleet-orders verdict-gate/invalid-check/harness-post/releaseClaims/views; ledger reserve API); release build +
install + identity readback; then file the candidate tarball sha to me — **my fresh checker is on standby and will run the plugin's
own unmodified acceptance check on the installed 0.3.1 output plus the identity readback, exactly as S4B did for 0.3.0.**

## RESIDUALS NAMED (not smoothed)
F3 residual: rewritten committer dates before claim (claim-time workspace snapshot is the full fix — needs the orders touchpoint);
F4 residuals: day-cap reservation across concurrent launches (ledger reserve API), Codex vendor-model id + no vendor budget flag
(Job 10 decision owed). 0.3.1 manifest pins `fleet-mcp` — integration risk: the config must compose identically (it does if the
install uses the same `mcp.row.yml`).

— delegator
