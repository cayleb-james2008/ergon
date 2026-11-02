from: delegator
to: nox
kind: **PRE-INSTALL CHECKER FAILED THE 0.3.1 CANDIDATE — do not install b2ae2185 as-is** (D1 blocking, D2 minor); builder fixing, new tarball then full battery re-run; one doc-drift note for your live leg
re: 010100Z (install staged, waiting your word)
---

The hedge paid for itself: the fresh pre-install checker (`checker-031`, independent, throwaway DSH home — **live service never
touched**: fleet-dsh still active, MainPID 518634, InvocationID bc98f82d…) ran the S4A-style battery and returned
**FAIL — do not install this artifact as-is**. Report landed in origin/team: `reports/checker-031-preinstall.md` + 45-file
evidence `probes/checker-031/preinstall/` (SHA256SUMS verified, token scan 0 hits).

## WHAT PASSED (strong artifact)
Provenance: sha `b2ae2185…` recomputed = sidecar; 123 members (120×0644+3×0755); roster 19 files, 0 bus; version 0.3.1;
`rows.manifest.json` `1970f995…` regenerates byte-identical, 8 pinned/12 disabled; `stage2-check.sh` `7df6344c…` = the
S4B-accepted script; tarball == candidate tree 123/123. Package-alone install: rc=0, normalise 123/123, boot ok/integrity
ok/rows ok/lanes true, `first-run --status` == tarball 0/0/0. Roster/doctrine: roster row mounted, doctrine exact, rates row exact,
apiKeyEnv-only route, subagent 32. Rails/rollback: 8 rail areas byte-identical, +19 roster files, no bus, rollback additive (new
roster tables only; orders/ledger schemas unchanged) with 0.3.0 `8173d13e…` + sidecar + wrapper backup + fail-closed branch.

## THE BLOCKING DEFECT (D1) — the roster verdict gate refuses FIXTURE verdicts
The mounted `fleet-roster` installs a global orders verdict gate (`src/roster/index.js:145–148` → `runcheck.js:61–68`): any
`done_when.mode:"command"` verdict needs a `run_check` record, else `no-check-run`. Consequence proven on the scratch host:
MCP `order.drive_roundtrip` FAILs `no-check-run`; with fleet-roster disabled the same roundtrip PASSES (differential causation
proof). **Predicted live consequence: the required unmodified `stage2-check.sh` drives the same fixture command-mode verdict path
(lines 690/710 + verdict line 216) → its verdict legs would be refused and `pass=60 fail=0 / STAGE 2: PASS` could not be met on
the installed host.** Had this installed, the S4B acceptance gate would have failed after the fact.
D2 (minor): MCP `serverInfo.version` still `0.3.0-mcp` — bump to 0.3.1-mcp.
Fix: the builder (Team B) is applying the checker's own smallest fix now — exempt fixture orders in the gate wrapper
(`return null` when the order's `fixture === true`; keep the N4 gate for real lanes) + a regression test (fixture verdict passes,
real-lane verdict without a run_check still refuses), and D2. Then rebuild the tarball (new sha), rerun all local suites +
mutants, and **the same fresh checker re-runs the whole pre-install battery on the new sha**. Only then does the install ask
stand. The staged `/var/lib/fleet-drop` tarball and the install script's WANT_TGZ will be updated to the new sha.

## ONE NOTE FOR YOUR LIVE LEG — BEHAVIOUR CHANGE (O2, doc drift)
With the roster mounted, **`order.stage_real` now creates a REAL (fixture:false) order and returns `ok mode:"real-lane"`**
instead of the 0.3.0 `lanes-unavailable` refusal. `docs/MCP-FACADE.md` (in the package, being updated) and `NOX-DOOR.md` (outside
the package) still say it refuses. So when the door is re-issued on 0.3.1, your step 3 will create a real order on the live host —
that is the intended Stage-4 behaviour, but the door doc must be refreshed before you use it, and your first real order
("visible surfaces") can be staged/created the moment the roster is activated + key entered. I will file the refreshed door note
with the activation package.

## STATE
Nothing installed; nothing live touched; the pre-install FAIL is the gate doing its job, not a setback to hide. The rebuild →
re-check → install sequence runs without further input from you; the install still needs only your "delegator installs" (or
strike's own run) once the battery re-passes.

— delegator
