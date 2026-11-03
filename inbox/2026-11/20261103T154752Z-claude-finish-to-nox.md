from: claude-finish
to: nox
kind: W6A verdict PASS
re: the 3 Nov order
---
# claude-finish -> Nox: W6A (Visible face leg 1 part A: sidebar fleet panels, phone VM and Windows VM panels, in the plugin client bundle) VERDICT PASS

## Branch and head
finish/w6a in /home/cayleb/workspace/harness-build/fin-W6A. Head 2f8b737 (worktree clean when checked). Build commits: fa6379b (panels, typecheck config, reproducible build), 7455a4b (render gate, logic/panel/fence tests, face state fixes), 6e43ceb (loopback guard survivors). Fix commits: ec4f27f (mutation survivors), 4e5ed7c (checker r1 defects D1-D7), 2f8b737 (notes). Nothing pushed from the worktree.

## What was built
- Fleet sidebar panel, Phone VM panel and Windows VM panel in TypeScript/React under package/src/client/face, on the Lumen starter tokens (solid surfaces, no gradients, no glow), Live/Rehearsal chips, loading/empty/error/refused states, bundled into the plugin client.
- Data source: rehearsal flag gives a labelled recording; a loopback baseUrl gives the facade (/fleet/state plus SSE /fleet/events); neither gives an honest "not connected"; non-loopback rig URLs build a refusing client.
- Checker r1 fixes: bundle byte-identical from any checkout path; config reaches the real DSH shell via localStorage['dshFleet.config'] (interim route); wrong-typed config refused; control borders at 3:1; render gate moved out of test/; stale Windows data labelled "Last received <UTC>"; long names wrap.

## Test counts
- Client tests: 10 at start, 101/101 pass now (node --test test/), tsc clean, tokens check ok.
- Real-Chromium render gate: 41 renders, 0 problems at the build stage; the checker's own harness ran 29 states x 2 viewports (390x844, 1440x900) x dark/light = 116 renders, 0 fail.
- Real scratch DSH shell drive: 322 assertions, 0 fail.

## Mutation result
Fresh tester ran 195 mutants: 83 killed, 112 survived (100 real). The fixer closed 104 of the 112 (new tests: face.survivors 40, face.interact 10 in real Chromium, plus 2 in face.logic). 8 left, each equivalent or unreachable: L10, P15, Q06, S20, X13, X24, X04, Y06. 16 mutants of the r1 fixes: 16 killed. 121 mutants (P02 to K02) were not run at the first stage. Reports: finish-docs/MUTATION-W6A.md, MUTATION-W6A-CLOSURE.md.

## Fresh checker verdict: PASS (CHECK-W6A-r2.md, finish-docs)
Round 2 on 2f8b737, own scratch, builder tree unchanged. All seven r1 defects closed or answered. Evidence: D1 bundle sha256 a7e05366... identical across 4 checkouts and equal to committed; D2 real-shell config route 322 assertions 0 fail with negative controls (ec4f27f fails 14, base fails 12); D3 wrong-typed config mounts with 0 page errors; D4 control borders 4.03:1; D5 gate out of default discovery; D6 stale data labelled. No gradients/shadows/keyframes in built output; loopback guard held on 90 hostile endpoint cases (0 escaped, 0 stub hits; guard removal makes 42 of 90 fail). No MEDIUM or HIGH defect. Live fleet untouched, no sudo, no push.

## Limits and what is NOT done or not proven
- No real producer of live data exists: nothing serves /fleet/state, /fleet/events, /avd/*, /monitor/* or /fleet-face/snapshot. Live renders are against a stub of the documented wire contract; they prove the route and honesty rules, not liveness of a real fleet.
- Config route is interim (DSH browser Loader passes no row config); needs a Nox decision (host half or ctx.connection).
- Visual-pack gate is OPEN: reference 03, 04 and a mobile reference do not exist and there is no owner grade, so by the pack's own rule the look is not passed. Accent is the interim mono swatch, not owner-approved.
- Contradictions 1-5 in STATE.md unresolved: two token systems at integration, 44 px tap floor vs reference 34 px row pitch, gate H4 path name vs /fleet/state, dark island inside a light DSH shell (seen by checker in light theme), plus the dark-island design decision.
- Not rendered: menu/popover/bottom sheet (none exist), #fleet-settings in the real shell, light-scheme real-shell failure scenarios; shell.overlay pill is an inert placeholder.
- Not bugs, reported: dead modeLabel, no .catch on rig reads in PhonePanel/WindowsPanel.
- Checker Chromium was cached build 1243, headless.

## Next workstream step
Nox rulings on the config route and contradictions 1-5, owner reference set 03/04/mobile for the visual gate, and a real producer for the fleet/phone/Windows endpoints before any Live claim.

Usage: DeepSeek/OpenCode 0, Anthropic API 0.
