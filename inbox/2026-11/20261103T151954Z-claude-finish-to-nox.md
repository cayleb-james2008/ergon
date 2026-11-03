---
from: claude-finish
to: nox
kind: W6B verdict
re: the 3 Nov order
---
# claude-finish -> Nox: W6B verdict PASS (fake-backed model; real-Studio path NOT PROVEN)

W6B = Visible face (leg 1) part B: multiple contained Roblox Studio instances in the sidebar, each with its own Studio MCP connection, game and agent team.

## Branch and head
finish/w6b at 700b648 (from release/0.3.5.1). Clean worktree: /home/cayleb/workspace/harness-build/fin-W6B.

## What was built
- Original build (earlier run, re-verified from a clean worktree this pass, no new code this pass): b77911f contained Studio instance model, binding guards, MCP client, supervisor, fake Studio MCP server and fake launcher, with tests; 9776f91 Studios panel (components, store, register), render-pack and live harness, feed error level, rpcHandlers; e6292ad remote redaction closes an orphan-lane name leak, defence in depth in the panel, api tests, attention layout, honest restart message.
- Backend in package/src/face/studios/: model, registry, binding guards, Studio MCP client, supervisor, fake servers, api, rpc handlers. Panel in package/src/client/face/studios/: instance list, per-instance detail (game, account label, container, MCP connection, team lanes, activity feed), Start, Stop, Connect MCP, Disconnect MCP when writes are allowed.
- Mutation fix round: a7ecc86 owner's place refused by file name in any directory (D1), .ROBLOSECURITY cookie masked at a line start (D2); 15bd2e0 tests for the mutation survivors, harness follows TMPDIR (D3, D4); c0e5546 fakes fail cleanly, tests cannot hang under a mutant, stdio group-kill test.
- Checker-defect fix: 700b648 fail-closed registry load and re-check at start/connect, complete-policy guard, attention text no longer doubled, no tracked symlink.

## Tests
- Maker, fresh this pass (at e6292ad): src/face/studios 30/30; src/client/face/studios 35/35; live harness ok at 1440 and 390, no page errors; render pack 46 renders (390x844, 1440x900, every state), 0 measurement failures (no horizontal overflow, buttons >= 44 px, contrast >= 4.5:1). 56 earlier screenshots in finish-state/W6B/shots, compared by eye with the reference set.
- Checker, round 2 on 700b648: W6B suites 94/94 and client 72 pass (1 skipped, build tooling not installed, skipped on base too). Whole package against release/0.3.5.1 by test name: base 803 tests / 785 pass / 17 fail; W6B 960 / 942 / 17; identical failing names (environment-bound, same on base), 0 base tests missing, 0 status changes, 157 new tests all pass.

## Mutation result
Fresh tester, 189 mutants on c0e5546's predecessor 4fc8e49 tree: 82 killed, 107 survived (7 equivalent, 14 safety-rule unpinned), raw score 43.4 percent. Verdict then: suites did not yet prove the containment rules. Named survivors included R01 (registry.add checked the binding against no other instance) and D1 (owner-place clash checked by path only, tripwire by file name only; reproduced). Both are fixed (a7ecc86, 15bd2e0) with tests; the checker's round-2 drive confirms clashes are refused at registry.add and an owner place by file name in another directory is refused. A post-fix mutation re-run score is in finish-state/W6B/mutation-rerun* ; this mail does not claim a number for it.

## Fresh checker verdict: PASS (round 2; reports finish-docs/CHECK-W6B.md, CHECK-W6B-r2.md)
Round 1 defects re-driven, all CLOSED: MEDIUM persisted registry loaded without checkBinding (chain2.mjs 41/41; same probe fails 41-way on c0e5546); LOW tracked symlink (no mode-120000 in git ls-files, git status clean); LOW doubled attention text (one clause at 390 and 1440).
Checks PASS: three instances isolated, drive.mjs 90/90 (separate connections, state, credentials, canary redaction, crash/stop isolation); owner-session binding refusal (owner stand-in saw 0 non-list calls); render 87/87; maker harness pack 46 renders 0 failures, live 31 ok; reference diff by eye (family match, not a pixel diff because the reference set has no panel states); live untouched (validate.js shas, service active, listener set, owner Studio pids unchanged, port 13469 never touched; events max seq not re-read).
Observation (LOW, not a defect): the tripwire treats a studio with no name as not foreign; cheap hardening for integration is to count an unnamed studio as foreign.

## Limits: what is NOT done / NOT proven
- Real-Studio path NOT PROVEN: fake launcher and fake Studio MCP only. No real Studio, real login, real container (systemd-run + pasta + Vinegar + Xvfb), real StudioMCP tool list or get_studio_state text, G0 (DSH connection RPC), real DSH shell, two-account persistence, light/dark seam, 200 percent zoom, forced-colours render.
- The module is still unwired and absent from the tarball (client index, bundle, package.json exports, rows.manifest, registerControlOp, host RPC registration are integration work). Owner policy values (place dir, flatpak id) are for the integration to supply.
- NOT done, owner-accepted addition 1 (contained-Studio spike): two REAL bound instances (street game, card game) with their own Studio MCP connections, agent team labels and sidebar status; and the containment proof document (instances active, Cayleb's personal Studio session state before and after, any point containment failed). Nothing here is that proof; a PASS on the fake model says nothing about isolation on the real machine. Studio control stays MCP-only, nothing published.
- Owner-accepted addition 2 (W8 CamoFox criteria) is a separate workstream and is not touched by this verdict.

## Next step
Run the real-bindings spike (addition 1): bind the two real instances, drive MCP work in both, and write the personal-session before/after containment proof; integrate the wiring then. W6B is not "done" until that proof exists.
