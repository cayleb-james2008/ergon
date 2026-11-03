---
from: claude-finish
to: nox
kind: W6C verdict
re: the 3 Nov order
---
# W6C (Visible face leg 1 part C: agent-activity view and shared-workspace shape): verdict FAIL

Branch finish/w6c (worktree /home/cayleb/workspace/harness-build/fin-W6C), over release/0.3.5.1 with finish/w6a merged in. HEAD on disk now b07c7a2 (fixer pass 3). The first build report named eadae52 (13 commits); fixer commits e3bedb6, 10d4715, 8f06ba0 and b07c7a2 came after it.

## What was built
The agent-activity view and the shared-workspace shape, registered in the DSH shell as the "Agents" panel: shared-session care fence in the roster, face read model, attention, redaction, host RPC on exact POST routes under /api/fleet, activity page source (types, view model, sources, panel, styles, copy), packaged rehearsal recording, client fleetTransport, client.js rebuilt. R2 real-shell render (src/face/live/shell-render.mjs, scratch host, private headless Chromium) passes 21 of 21 checks at 390 dark, 1440 dark and 1440 light. Booted-host liveness check boot-face 21/21 on W6C; the same script on the unmodified base fails 9 checks, so the proof can fail.

## Tests
After fixer pass 3: face + client suites 140 tests, 140 pass (4 new in that pass). Full suite via find glob: 1137 tests, 1110 pass, 27 fail, all in known environmental families (fsguard, sandbox, orders child scripts, firstrun manifest, roster gate/mutate); the firstrun report-hash failures are identical at the earlier commit. The full before/after failure-set diff was not re-run.

## Mutation
234 mutations applied (/home/cayleb/workspace/harness-build/finish-docs/MUTATION-W6C.md). Survivors found, including W23 (a role=checker lane with no kind, the shape checkflow.js:68 really creates, was shown as an agent row) and the roster wiring group E06-E10. Fix pass: 92 closed. Not closed, judged equivalent: K05, K06 (String(req.id) coercion), E11 (laneOf string check) and the like. Fixer pass 3 mutants M5a, M5b, M6 all killed.

## Fresh checker
Verdict FAIL (round 3, finish-docs/CHECK-W6C-r3.md, on b07c7a2). PASS: round-2 MEDIUM block_reason redaction (real board, secret walk of the served result clean); round-2 LOW "Workers" label with parked lanes; booted-host liveness; real drive at 390 and 1440 dark/light; care fence and lane-message; 30-state maker render harness; no gradient; no fixture shown as real.
Remaining defects:
1. LOW, verdict-affecting: the bundle freshness test (client.test.mjs, "committed client.js is what the pinned DSH preset builds from index.js") is still checkout-path dependent. normalize-build.mjs does not match a digit-leading CSS-module hash (7 characters with a leading underscore). It failed in one of two other checkout paths the checker tried and passed in the other, so it is path luck, and in a clean checkout it is a new failing test by name against base. The bundle itself is fresh (byte-equal under a looser normalisation). A round-1 fix reported closed was not.
2. NOT PASSABLE AS WRITTEN: the maker screenshot-vs-reference gate. Only references 01 and 02 exist; 03, 04 and the mobile reference are owner-supplied and absent. Reference 02 is a sidebar session list while the page is a main-pane view, so the comparison covers tone, density, border and type only.

## Limits, not done, not proven
- The checker verdict is FAIL; W6C is not accepted.
- The screenshot gate is NOT PASSING (reference set incomplete).
- Working/silent lanes on real data, elevations, Tier B holds and lane-message completion need a real model turn; seen only through fixtures and real-board unit tests. In the checker's run the lane stayed idle.
- Owner's grade of the look, the accent swatch, W6B contained Studio instances and W5 package manifests are out of scope and ungraded.
- Free-text fields read but not planted: participants[].display, stalls, unstaffed, holds[].tool, elevations[].classification.
- The light-theme dark Activity surface is a leg 1 decision, unchanged.
- Usage: 0 Anthropic API calls; no completed DeepSeek/OpenCode call observed.

## Next step
Fix the digit-leading hash normalisation with a digit-leading unit case, rebuild, send to a new fresh checker round; the screenshot gate waits on the owner's references 03, 04 and mobile.
