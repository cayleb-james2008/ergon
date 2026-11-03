---
from: claude-finish
to: nox
kind: completion report
re: the "finish the plugin" order of 3 Nov 2026
---

# Completion report: "finish the plugin" (3 Nov 2026)

## Status: IN PROGRESS. NOT DONE.

Per the standing definition, DONE means every workstream proven end to end by a fresh checker, the live install healthy, and the sitting bundle staged. That is not the case. Four things stop it, named here and not softened:

1. The live 0.3.6 install has NOT run. The safety classifier refused the install command with reason "[Production Deploy]". The live host is unchanged.
2. No fresh checker has run on the integrated release/0.3.6 as a whole, other than the whole-package check CHECK-PKG-0.3.6 (PASS as an install candidate, scratch only; details in section 3). Several workstreams are scratch-, fake- or fixture-proven only.
3. Two workstreams FAILED their last fresh check (W5B, W6C) and are not in the package. W7 is open on its owner dependency.
4. The owner sitting bundle is staged but nothing in it has been run by the owner.

Nothing was sent, posted, registered or installed on the live host by this work. No key, token or password value appears in this report.

## 1. Verdict table

| WS | Verdict | Branch / head | In 0.3.6 |
|---|---|---|---|
| W1 courier harness | PASS (round-2 courier check) | finish/courier-harness | yes |
| W3 relocation, delegator becomes singleton lane | PASS | finish/w3 (e81e4f2) | yes |
| W4 owner elevation completion, secure prompt, D1 kit | PASS | finish/w4 (a8611b8; the first mail named 9babcf1, a8611b8 is correct) | yes |
| W5A package pieces for 0.3.6 | PASS | finish/w5a (b4a02e5) | yes |
| W5B Option B, rubric final task, dispatch-a-banker | FAIL (liveness gate not met) | finish/w5b (f7962b3) | NO, excluded |
| W6A visible face, fleet / phone VM / Windows VM panels | PASS (functionality; look not graded, see 4.3) | finish/w6a (2f8b737) | yes |
| W6B contained Studio instances, fake-backed | PASS, fake-backed only; real-Studio path unproven | finish/w6b (700b648) | source only, not wired |
| W6BS real-bindings spike and containment proof | PASS for the containment proof; native embedding NOT complete (login wall) | finish/w6bs (628f49e) | source only, not wired |
| W6C agent-activity view | FAIL (round 3) | finish/w6c (b07c7a2) | NO, excluded |
| W7 B7 step-0 live drive | OPEN, owner dependency (see 2.7) | none | n/a |
| W8 CamoFox hosted browser | PASS (round 3) | finish/w8 (70d6740) | yes |
| W9 RSI as external participant (ext:rsi) | PASS as a BUILD pass only (see 4.1) | finish/w9 (9c0f76e) | yes, with the courier |
| BUNDLE owner sitting pack | staged, fresh checker re-run still needed | finish/bundle, not merged by design | pack, not package code |

Also built and checked in this order and merged into release/0.3.6: pa9-comms, l4a-elevate, d2a, ctrl (CTRL verdict PASS), d1b (PASS, ships disabled), c1 (token pre-expiry notice, PASS). W2 (G1 gate on the merged courier tree) closed as G1 CONDITIONAL on a8ced6b: no courier defect, conditional on integration, which the merge has since done; row 7 and the load-sensitive keep-alive race remain noted.

## 2. Per workstream: evidence, proven, not proven, blocked

Evidence paths are under /home/cayleb/workspace/harness-build/finish-docs/ (checks, mutation reports) and finish-state/<WS>/ (STATE.md, VERDICT.md).

### 2.1 W1 courier harness (finish-state/W1/STATE.md, CHECK-COURIER-r2.md)
- Proven: pa9-comms and d2a merged by real merge; the D2b courier pull cursor; duet suite 136/136 in 31 of 32 cold reruns; whole package equals base by failing name.
- Not proven: one cold first run gave 135/136 with the failing name not captured (unidentified flake). Couriers are files only; no unit installed; C0 proof, G40, G2 stay owner-present steps. Duet quiet-host 129/129 is still owed (also W9).

### 2.2 W3 relocation (finish-state/W3/VERDICT.md, CHECK-W3.md, MUTATION-W3.md)
- Proven (scratch, fake loopback model): singleton delegator lane, wake by labelled handoff, capability reduction, board view with no claim token, distillation canaries clean, deletion guard with 16 refusals under a pty. Checker 13/13. Relocation suite 131/131. 156 mutants, 108 of 111 survivors closed, 3 equivalent.
- Not proven: any real DeepSeek model behaviour; real session-store fields; live install, live lane spawn, live bus drive; deletion and archive on real data (owner-run). Switch script for item 15 of the pack is not built (blocked-on-branch).
- Carried: the relocated seat's no-re-park charter line (CHARTER-ADDENDUM-NO-REPARK.md) lands with the W3 merge and is unproven on a real host.

### 2.3 W4 owner elevation (finish-state/W4/VERDICT.md, CHECK-W4-r2.md, MUTATION-W4.md)
- Proven: D1 install kit with real entry program on scratch, whole-kit manifest pin, GTK4 prompt under Broadway, password-absence scan (0 hits in 23,079 samples). Mutation: 176 applied, 141 of 148 re-run survivors closed.
- Not proven: real Wayland/Hyprland (headless Hyprland does not start here); root-mode owner check beyond the test seam; D3 against a live journal; real install (owner-only, sudo).
- Disclosed incident: an early e2e run aimed at the live control socket once and made one elevate-outcome report (no secret). Fixed so every scene uses its own scratch socket.

### 2.4 W5A (finish-state/W5A/VERDICT.md, CHECK-W5A.md, MUTATION-W5A.md)
- Proven: install-0.3.6.sh on a real scratch host 17/17, must-keep gate 26/26 (refuses the reverted validate.js and a scratch-rooted mcp row), hazard guard refuses live targets, D3 defaults, duet limits 20/hour and 100 open, tmp-sweep. 191 mutants, 88 of 90 survivors closed.
- Not proven: real systemd (stop, systemd-run, sweep timer enable); installer refusal when the app's Host is the fleet writer (Part B row 9) and mailbox retention are not built.

### 2.5 W5B (finish-state/W5B/VERDICT.md, CHECK-W5B-r3.md): FAIL
- Why: liveness gate NOT MET. The scratch home had no model credential, so no lane spawned: neither liveness A (rubric final) nor B (command order, bank, close) ran. No code defect was found, and none was ruled out by a live drive.
- Blocked by: credential staging (stage-credential.mjs) is owner-run or a session the owner clears; a prior copy was refused by the classifier as credential materialization.
- Consequence: Option B is NOT in 0.3.6; the banking gap stays as filed. Mutation 154 mutants, 100 killed, 29 proven real gaps, later commits closed more; the final survivor count is unmeasured. Successor option: a W5B liveness re-run on a host with a model route; say if you want it.

### 2.6 W6A / W6B / W6BS / W6C: the visible face
- W6A (CHECK-W6A-r2.md, MUTATION-W6A-CLOSURE.md): PASS. 101/101 client tests, 116 real-Chromium renders 0 fail, real scratch DSH shell drive 322 assertions 0 fail, bundle byte-identical across 4 checkouts, 104 of 112 survivors closed. NOT proven: any real producer of live data (no /fleet/state, /fleet/events, /avd/*, /monitor/* server exists; live renders are against a stub of the wire contract); the config route is interim (localStorage). Open contradictions 1-5 in its STATE.md; the visual-pack gate (see 4.3).
- W6B (CHECK-W6B-r2.md): PASS on a fake launcher and fake Studio MCP only. 94/94 W6B suites; 189 mutants, 107 survived at first and the named safety survivors were fixed; no post-fix mutation score is claimed. NOT proven: any real Studio, login, container, StudioMCP tool list. Unwired and absent from the tarball.
- W6BS (CHECK-W6BS.md, STUDIO-CONTAINMENT-PROOF.md): PASS for the spike's job. Two real contained instances (street game, card game) each with own systemd unit, Xvfb, pasta and bwrap namespaces, Wine prefix, real StudioMCP host; 40/40 in 2 rounds, checker 38/38; owner's personal Studio observed read-only before, during and after with no violation. NOT done: Studio never passes its login screen ("Authenticated: NO", "Embedded Web Browser fail to load"), so no place opened and no real execute_luau ran. Containment is proven for tested paths only; one negative (an unlisted protected port) is detection, not prevention. Three checker doc corrections D1-D3 are not yet applied to the write-up. Owner items open: two non-owner Roblox accounts, an x11vnc viewer route (x11vnc is not installed), studio-confirm-login. Nothing published.
- W6C (CHECK-W6C-r3.md): FAIL. LOW but verdict-affecting: the bundle-freshness test is still checkout-path dependent (digit-leading CSS-module hash not normalised); and the maker's screenshot-vs-reference gate cannot pass (references 03, 04, mobile absent). Fix is small (normalise the digit-leading hash with a unit case, rebuild, new checker round); not done.

### 2.7 W7 B7 step-0 live drive
- Filed FAIL at 08:32 ET (no delegator ping sent; delegator parked). Later the same day the delegator leg PROVEN by a fresh checker, attempt 3 (CHECK-0.3.5-live-attempt3.md, 7 rows proven, 1 named limit): Nox's typed resume at 10:28 ET woke the delegator; one labelled TRANSITIONAL ping h9 (359 B, not the 106 B proof text) was pulled and acked by ext:delegator itself; bus.status on its own handoffs; events chain recomputed, no break.
- W7 STAYS OPEN. Remaining: CHECK-0.3.5-live.md, as I read it for this report, still shows FAIL (NOT PROVEN) from before that proof, and I found no combined final superseding it; the owner's G2 proof ping (the 106 B leg-3 text) is not sent; bus.hello live is a named scratch-only limit; whether the delegator re-parked after 15:54:57Z is unknown. Dependency: the owner.

### 2.8 W8 CamoFox (finish-state/W8/VERDICT.md, CHECK-W8-r3.md)
- Proven by the third fresh checker on 70d6740: all four owner criteria MET (real browser task watchable in the sidebar; logins survive host-home deletion and SIGKILL, negative control logs out; typed password in 0 of 31 readbacks, guard-off leaks; 403 and challenge-page sites route to Chromium) and capability inventory 32/32. Rounds 1 and 2 did not pass (round 2 found a HIGH, a CamoFox login erased by the next Chromium checkpoint, fixed). 175 mutants, 165 closed.
- Not proven: any language-model-driven agent (lane agents driven by the selftest op); a real protected site (Cloudflare etc.) or real https; iframe and shadow-DOM password fields; the product reclaim of dead displays live.
- UNREPAIRED INCIDENT: my own run of mutant D5 deleted the owner's real /tmp/.X11-unix/X0. The repair (ln -s X0_ /tmp/.X11-unix/X0) was refused by the classifier ("safety classifier" refusal, not worked around). At filing time X0 was still absent: new X clients on :0 cannot connect; existing ones keep working. Owner staged fix: ln -s X0_ /tmp/.X11-unix/X0 (unprivileged, reversible with rm). Clean fix: a Wayland session restart. I have not re-read the live state of /tmp/.X11-unix/X0 for this report.

### 2.9 W9 RSI external participant (finish-state/W9/VERDICT.md, CHECK-W9.md, RSI-REACHABILITY.md)
- Proven (BUILD pass): ext:rsi on the Duet pattern, closed-envelope fence both ways, 147/147 plus Duet 114/114 subset, 167 mutants with 68 real-gap survivors closed (tests only), checker real-drive registration on a scratch host (refused without confirm, ran after real `dsh-fleet confirm`), 59/59 handoff states, hostile corpus 21/21.
- Not done: the connectivity proof itself; nothing sent, registered or installed live. Route is relay-only through Nox's hourly loop. The meme desk shadow on outis-desktop is down (JUP_API_KEY empty); not a W9 deliverable.

### 2.10 BUNDLE owner sitting pack (finish-state/BUNDLE/STATE.md)
- Staged, NOT announced, nothing run: /home/cayleb/Documents/deepseek-harness/default-workspace/board/owner-sitting/PACK-20261103/ (index.md, SITTING.html, scripts, SHA256SUMS, VERIFICATION.md), 17 items, test suite 52/52 on a scratch copy, a fix pass after the fresh checker's first FAIL. A fresh checker re-run is still needed. Item 15 (the switch) is blocked on W3 (no owner-run switch script); item 7 refuses until Lane 6 writes d1-kit.pin and the fleet-elevate release is live; item 4 (PA picks) is yours to decide. The pack lacks owner items for the Studio spike and W8 criteria 2 and 3 beyond the gap lines. STUDIO-CONTAINMENT-PROOF.md must go into it; I did not touch finish/bundle.

## 3. Integration and the live install

- release/0.3.6 head d6e2e4a. Merged: pa9-comms, l4a-elevate, w4, d2a, courier-harness, ctrl, d1b (ships disabled), c1, w9, w3, w5a, w6a, w6b, w6bs, w8. Excluded: w5b, w6c, bundle.
- Integrated suite: 1927 tests, 1923 pass, 4 fail (3 are base's version-string tests; the 4th, a w4-closure pin count, was fixed and passed alone 5/5; not re-run in a full suite). Release plus relocation suite 213/213.
- Whole-package check, CHECK-PKG-0.3.6.md (fresh agent, scratch): verdict PASS as an install candidate. Installer real-shape install of 111 files (38 replaced, 73 added), idempotent, rollback byte-for-byte; boot gates (11 pins, bus-selfcheck, verdict-selfcheck, facade 15 tools); bus round trip 37/37; duet 23/23; elevate 20/20; face panels in a real DSH shell 16/16; CamoFox watch view 15/15. Findings: M1 fleet-browser fixed port 8796 (install only while 8796 is free); M2 face panels not wired though notes overstate it; the CONTROL audit lacks ext:rsi and fleet-browser states.
- Widened install shape: the installer writes three host-rooted files derived from the target (11-pin manifest, delegator preset, one appended fleet-browser patch in the profile overlay). That widens W5A's files-only design; derivation is hash-checked against reviewed artefacts.
- Studio is merged as source only: no client entry, files[], manifest pin, host RPC or control ops. The W6A/W6B token-file choice and owner policy values are open.
- Live install attempt (INSTALL36, finish-docs/INSTALL-0.3.6-OWNER-RUN.md): all four preconditions held (must-keep gate PASS, 0 non-fixture non-terminal orders, dry run 111 to install, unit active). The command `install-0.3.6.sh --target-live` was REFUSED by the safety classifier: "[Production Deploy]". Not retried, not worked around. Live host unchanged (MainPID 3671044, tree hash 791acbac... identical before and after, no new backup). Owner-run command, rollback and health checks are in the file above.
- Anomaly, not mine: /run/user/1000/fleet has been empty since 13:21:05 ET while the unit still listens on the orphan inodes; a restart recreates the sockets.
- Disclosed: the integrator ran a pkill on a generic `node --test --test-concurrency=2 --test-reporter=tap` pattern during an earlier shutdown, which could have killed another agent's test run; unverifiable.

## 4. Nox rulings of 3 Nov, as carried

### 4.1 W9 and the RSI connectivity proof (ruling 1)
W9 is accepted as a BUILD pass only. The sequence is: guarded 0.3.6 install lands W9 (ext:rsi registration and courier start go into the install runbook); then the handoff fires; Nox's hourly loop carries it to RSI's thread and attests the reply; `rsi-check` then upgrades round-trip-ok-unattested to proven. Nothing about the meme desk route changes until that ack exists.

FLAG, whose hand the registration confirm needs: `comms-register` of ext:rsi is Tier B (request, confirm the ticket, then the second call with the grant). The confirm (`dsh-fleet confirm`, via the secure prompt) is the OWNER's hand (RSI-OWNER-ITEMS A1; A2 token issue and A3 courier unit install are also owner acts). Nox or the executing session cannot confirm it. It must also be confirmed and consumed before the token request, since comms.grant is one policy value. This is not yet listed as an owner item in the staged sitting pack index; it needs adding to the install runbook (INSTALL-0.3.6-OWNER-RUN.md) and the pack.

### 4.2 W3 live install and G2 (ruling 2)
W3 live install steps ride the standing guarded-install go at the whole-package PASS. The owner's G2 proof ping (the byte-exact 106 B text) is a sitting item, last in sequence. Open: the standing go has not been exercised because the install itself is refused by the classifier, so the owner runs it.

### 4.3 W6B and the face look (rulings 3, 3a)
- W6B is a fake-backed PASS with the real-Studio path unproven. The queued next step is the real-bindings spike with the containment proof; W6BS did the containment proof but stops at the Studio login wall, so the spike is not finished.
- FACE VISUAL (Nox 3 Nov ~11:24 ET): the Lumen visual pack is scrapped. The game-style UI (React, Three.js, Tailwind) is for OpenBell's trading dashboard only, not the DSH face, not Ergon or RHL. The DSH face's direction is the clean professional component-library look: real components, flat colour, no gradients, no game styling; it does not wait on any external design pack.
- The face FUNCTIONALITY (panels, studios, wiring, the W6B model and its checks) is proven as built, to the limits in 2.6. Any Lumen-derived styling in the built bundle is NOT carried as the final look and was NOT graded (W6A's visual-pack gate was open and no owner grade was given; the accent was an interim mono swatch). The face is NOT visually done. Restyling to the professional library look is the PENDING VISUAL ITEM. Question for Nox: is that restyle in this mission or a successor item?

### 4.4 W10 dynamic workflows (ruling 3b)
Filed as the successor item: finish-state/W10/STATE.md, spec finish-docs/W10-DYNAMIC-WORKFLOWS-SPEC.md. NOT STARTED. It starts after the install seals and post-install health, goldfish spec pass first, then build in three slices (runtime core, control ops and resume, face view), each with a fresh checker, then integration into 0.3.7 or 0.3.6.x. Its three open questions for Nox: (a) does a workflow agent count as a roster lane for spend and cap purposes, or does a run have its own cap line; (b) which role may author workflow definitions; (c) is the default-method rule enforced or advisory. The spec's section 6 records your 11:14 ET rulings on all three (span of control 3-5 per lead with sub-lead levels; lanes author, Nox approves, versioned artefacts; advisory default with an easy start, no blocking validator). I list the questions as asked and flag them as already answered there; confirm they stand.

### 4.5 W7 and the named limits (ruling 4)
W7 stays OPEN with its owner dependency (2.7). C1 (token pre-expiry notice) is PASS. C2, C3 and C5 stay as named limits as ruled (I did not re-derive them here; the courier status register records C5/A4 FINAL, C0 NOT FINAL, and `comms.control-channel-trusted` default false until L4-A). Unproven real-host items: `agent.cancel` ending a wedged turn on a real model (CTRL ran scripted agents only; no real DSH agent or model); the relocated seat's no-re-park charter line (lands with the W3 merge, unproven on a real host).

## 5. Classifier refusals, named with their reason

| Refused action | Reason | Outcome |
|---|---|---|
| Live install `install-0.3.6.sh --target-live` (INSTALL36) | "[Production Deploy]" | not run, owner-run file written |
| W8: `ln -s X0_ /tmp/.X11-unix/X0` | safety classifier refusal (no reason text in the verdict) | not worked around; owner staged |
| W5B round 1: credential staging for liveness drives | credential materialization | blocked; owner-run |

## 6. What stops at the owner line

Install (sudo/production), the ext:rsi Tier B confirm, the X0 symlink, the 17 sitting items (G2 proof ping last), two Roblox accounts and the viewer route for Studio, staging a model credential for W5B liveness, the W7 combined final check after the owner-dependent steps, the PA picks (item 4), and the Q17/Q18 deletion items.

## 7. Remaining items, named (this is the "in progress" list)

1. Guarded live 0.3.6 install by the owner, then post-install health; then ext:rsi registration confirm, courier start, handoff fire, hourly loop carry and attest.
2. Studio: finish the login/embedding failure, real execute_luau reads, real wiring (client entry, files[], pin, host RPC, control ops), W6B real-bindings spike.
3. Face: restyle to the professional library look (Nox to say mission or successor); real producer for /fleet/state and friends; config route ruling; contradictions 1-5.
4. W6C: digit-leading hash fix, new checker round. W5B: liveness run with a model route.
5. Fresh checker re-run of the sitting bundle; add ext:rsi confirm and Studio/W8 owner items; switch script for W3.
6. W7 combined final check; quiet-host Duet 129/129; unidentified W1 cold-run flake.
7. W10 after the install seals.

## 8. Usage log (DeepSeek test calls, as reported by the workers; nothing invented)

Every worker that logged usage reports 0 DeepSeek/OpenCode calls and 0 Anthropic API calls: W1, W3 (all model behaviour was the loopback fake), W4, W5A, W5B, W6A, W6B, W6BS, W6C, W8, W9, BUNDLE, G1M, CHKPKG, INSTALL36, hazchk, and the mutation stages. Mutation, checker and drive counts are test runs, not model calls. The reports do not state the figure for the CTRL, C1, D1B and D2A builders; those STATE files carry no usage line, so I record "not reported" for them. Total DeepSeek test calls reported: 0. The consequence is that no workstream was exercised against a real DeepSeek model.
