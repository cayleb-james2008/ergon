# LANE 2 — build queue (isolated worktree ready; no installs)

Filed 2026-11-01T17:12Z by `fleet-lead`. Scratch/isolated only; nothing installs from this lane.

## Workspace
- Worktree: `~/workspace/harness-build/scratch-tb-l2/build-effects` — branch **`tb/effects-outbox`** off
  `integration/0.2.2` @ `cf93107` (clean; integration tree untouched).
- Baseline: isolated-tree scratch acceptance **PASS 60/60** (`board/reports/team-b-lane2-scratch-acceptance.md`)
  — rerun it as the regression gate after every build step.

## Ordered queue
1. **Effects/Outbox (adopted pattern, write-first durability)** — design filed
   (`board/reports/team-b-lane2-effects-outbox-design.md`). Build: additive `effects` table + executor with
   retry/backoff + restart-recovery pass; wrap the consequential writes (board post/claim/release, task
   transitions, evidence/verdict/bank); crash-matrix tests (record-then-crash, send-then-crash, observe-then-crash,
   recovery-repeat); read-model pending/failed counts. **Done-condition:** crash/kill matrix green, no duplicate
   logical action, no lost intended action across restart; scratch Stage 2 suite stays 60/60.
2. **Bus package assembly on the merged base** — rebase/reassemble the `bus` branch content onto the integrated
   tree in an isolated worktree; keep 67-test evidence; no mission-branch writes without authorisation.
3. **Hands-code race fix** — identify the race from the preserved hands WIP/notes, fix in the isolated tree,
   one regression test per verified race; hands adoption itself stays Stage 8 (with the MCP exposure rails).
4. **Contract-doc amendment** — throwing-accessor known-limitation entry (Nox `072312Z`, proposal `80ad8715`,
   tree `96a52f5e`) + the adapter producer-selection note; docs only.
5. **Composed-path end-to-end acceptance prep** — script the composed path against the scratch host so the
   real-install acceptance (Lane 1, after identity) is a single run.

## Fences
No install, no service action, no writes to the installed tree or personal profile; maker-checker split applies;
each step's acceptance on the isolated tree, digest-level verdicts on the board.
