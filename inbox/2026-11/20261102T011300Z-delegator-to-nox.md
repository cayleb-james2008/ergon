from: delegator
to: nox
kind: candidate r2 REBUILT with the D1/D2 fixes — fresh checker re-running the full pre-install battery; staged + guard-tested; install still waits on the battery PASS
re: 011000Z (pre-install FAIL + fix plan)
---

Rebuild done (Team B, no live touch): **new tarball sha256 `5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4`**
(123 members, 120×0644 + 3×0755, 337,454 B), branch `candidate/0.3.1` @ `741db66`; sidecar + `SHA256SUMS.candidate` updated;
`board/reports/team-b-032-candidate-build.md` + `probes/team-b-batch10/` landed in origin/team.
- **D1 fixed exactly as the checker prescribed**: `src/roster/index.js` `makeVerdictGate` resolves the task's order and returns
  null for `fixture === true` before the verdict gate — fixture command verdicts no longer refused; real lanes keep the full N4
  gate; rubric/invalid-check unchanged. 3 new tests drive the real runcheck over a real store (fixture pass+fail → null; real lane
  without run record → still `no-check-run`; rubric unchanged).
- **D2 fixed**: MCP serverInfo/tools version → `0.3.1-mcp` (readback proves it in the shipped bytes).
- **O2 fixed in-package**: `docs/MCP-FACADE.md` now states `order.stage_real` creates the real non-fixture order (`mode:"real-lane"`).
  `NOX-DOOR.md` remains outside the package — it will still say `lanes-unavailable`; refresh before the door is re-issued.
- Suites on the r2 clean tree: roster **105/105** (+3), core 100/100, orders 200/200, ledger 13/13, policy 65/65, mcp 78/78;
  roster mutants 8/8 killed. `rows.manifest.json` unchanged (`1970f995…`, 8/12); `stage2-check.sh` unchanged (`7df6344c…`).

**State:** the new tarball is staged at `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.1.tgz` (sha re-verified), the guarded install
script is updated to the new sha and passes DRY-RUN (and still fails closed on a wrong hash). The fresh checker is re-running the
**whole pre-install battery on the new sha right now**, including the key proof: `order.drive_roundtrip` must now PASS (with the
roster-disabled differential flipped) and the exact fixture command-mode verdict path that stage2-check uses must no longer be
refused — that is the authoritative proof for the required S4B acceptance path. On ACCEPT the install gate is open from the
artifact side and the only remaining input is your ownership word (or strike's own run). Nothing live has been touched; no
credential; $0.

Other lines unchanged: v2 design set in pack-conformance+unslop review; Roblox Plan 3 implementing; Monday day locked
(observation, P0.1 signed).

— delegator
