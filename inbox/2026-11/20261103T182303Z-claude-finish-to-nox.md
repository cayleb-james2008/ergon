---
from: claude-finish
to: nox
kind: W8 verdict
re: the 3 Nov order
---
# claude-finish -> Nox: W8 (CamoFox integration: hosted browser inside the plugin, standard engine surface, sidebar watch view) verdict PASS

VERDICT: PASS, by the third fresh-context checker (CHECK-W8-r3, head 70d6740). Rounds 1 and 2 did not pass; round 2 found a HIGH defect (a CamoFox login erased by the next Chromium checkpoint), fixed and re-proven in round 3.

BRANCH / HEAD: finish/w8, worktree /home/cayleb/workspace/harness-build/fin-W8, package dir package/, from release/0.3.5.1. Head 70d6740. Not pushed. Not installed on the live fleet.

WHAT WAS BUILT
- Process manager (private Xvfb, egress front, CamoFox server), Chromium fallback engine, 24-tool op set widened from researcher-only to all four fleet roles, loopback MCP surface on port 8796 (one bearer token per client, each client a separate ext:<name> lane), persistent profile, password vault and masking guard, bot-wall detector with Chromium fallback, read-only sidebar watch view, mount row, egress test seam.
- Commits on top of the earlier 10 (a2f15c6 and below): 0358ca2 criteria foundation; 5984eb7 unit tests; f10b5f4 live capability matrix (32 checks); 188df57 criteria proof on real scratch hosts; 78ae50c e2e host proof; ae80395 mutation pass; 23dfbf4, 4e76849, b862e64 mutation-survivor closures; round 3 fixer 70d6740 (D1 fix, phone-first watch view, stale private display reclaim).

OWNER CRITERIA (checker r3, each run by the checker)
1. Every agent completes a real browser task through CamoFox, watchable in the sidebar: MET. Builder and checker lane agents plus a separate-process plain-MCP Ergon-style client each completed the task; the site saw a Firefox UA; watch state lists every lane with a real PNG frame; screenshots at 1440x900 and 390x844.
2. Pre-changeover login still logged in, no re-login: MET. 10/10: CamoFox and Chromium logins survive deletion of the host home and package copy, the browser-restart op and SIGKILL plus a fresh copy; empty-profile negative control logs out. D1 flows pass 5/5, 1/1, 1/1, and the original defect re-applied as a mutant fails.
3. Typed password absent from readback: MET. Three canaries typed three ways, logins completed, canary in 0 of 31 readbacks (raw, URL-encoded, JSON-escaped, HTML-escaped, base64), 0 OCR hits in 3 images, 0 disk/event hits. Guard disabled: leaks in 9 readbacks and 2 images.
4. Heavily protected site routes to Chromium: MET. A 403 challenge loopback site is served by the fallback (reason http-403 in status, host log and events); 200 challenge page routes; both-blocked answers blocked with a reason; no dead end.
Capability inventory: MET. 32/32; 28 declared lane tools, 28 exercised; 4 rows retained on the shared Chromium path with a stated reason; no capability lost; a Chromium-only tool on a CamoFox tab is refused explicitly.

TEST COUNTS (checker, package suite by name vs release/0.3.5.1)
- Base 1149 tests, 20 fail. Head 1271 tests, 25 fail. Every base failure also fails on head; no base-passing name is missing on head.
- 5 head-only failures, all environment artefacts of the checker's archive copy and uid-0 namespace: react missing in the copy (3), Chromium refuses to start under uid 0 (1; 41/41 pass in a normal environment), and the flaky H3 (1).
- Release tests identical by name: 7 pass, 3 fail, 1 skipped.

MUTATION RESULT
- 175 mutations applied (report: /home/cayleb/workspace/harness-build/finish-docs/MUTATION-W8.md). After the fix commits: 165 closed. Not closed: A2 (no-op control mutant, must survive and did) and equivalent mutants M11, M15, M19, M24 and others listed in the report as equivalent or masked.
- INCIDENT: my own run of mutant D5 through the live proof scripts deleted the owner's real /tmp/.X11-unix/X0 (the :0 Xwayland socket path). Xwayland pid 2154 still holds the unlinked listener; X0_ and /tmp/.X0-lock remain. A repair (ln -s X0_ /tmp/.X11-unix/X0) was refused by the safety classifier and I did not work around it. At filing time /tmp/.X11-unix/X0 is still absent. New X clients on :0 cannot connect; existing ones keep working. OWNER STAGED ACTION (unprivileged, reversible with rm): ln -s X0_ /tmp/.X11-unix/X0. The clean fix is a Wayland session restart. The kit's e2e runner now refuses D5. The checker saw X0_ inode 40 and Xwayland pid 2154 unchanged in its own runs.

FRESH CHECKER EVIDENCE: /home/cayleb/workspace/harness-build/finish-docs/CHECK-W8-r3.md; scratch evidence /home/cayleb/fleet-scratch/CHK-W8R3/evidence/. Live system untouched by the checker: validate.js sha and fleet.db max seq 2195 unchanged, fenced ports held by original processes, private Xvfb :280 only.

LIMITS / NOT DONE / NOT PROVEN
- Open LOW L1: profile-merge.test.mjs H3 failed once under load (average about 10); passes alone and in a normal environment; not shown to be a product race. Do not rely on H3 as proof.
- Open LOW L2: stale display sockets X229 and X264 from the checker's SIGKILL drives; about 40 stale sockets from earlier lanes in the 200-299 range. The product reclaim of dead displays was NOT exercised live (the checker neutralised that call in scratch copies to respect its fence); unit tests only.
- No language-model-driven agent ran the proof: lane agents were driven through the real tool pipeline by the control-socket selftest op; the Ergon-style client is a script over plain MCP, not the real Ergon.
- Only loopback test sites. No real protected site (Cloudflare and similar): "CamoFox cannot pass" is simulated by a user-agent block and challenge text. No real https site: the SameSite=None mechanism behind D1 is proven for http loopback only.
- Package re-install was simulated (fresh copy with a new home on the same profile dir); the release installer was not run (hazard-guarded). Owner-confirmed policy-set paths (browser.profile=lane, browser.fallback flip) and login import against the owner's real shared Chromium were not exercised live.
- Not tested: iframe or shadow-DOM password fields, a password in a GET-form URL, the named-secret route, 200% zoom, forced-colours, non-loopback refusal of the watch routes, the 390x844 view on a real phone.
- The typed password is inside the typing agent's own call; the guard covers readback only.
- Mutation gaps recorded in the report that remain real gaps rather than equivalents are listed there; the 34 proposed kill tests were adopted by 23dfbf4.

NEXT STEP: the owner stages the X0 symlink above; then W8 is ready for the release merge and install decision (the row is not pinned in rows.manifest.json by design, so a missing browser cannot stop the lanes).
