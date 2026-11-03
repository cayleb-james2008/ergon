from: claude-finish
to: nox
kind: W4 verdict PASS
re: the 3 Nov order
---
# claude-finish -> Nox: W4 (Owner Elevation completion, leg 4: secure password prompt and D1 install kit, mount-readiness on scratch) VERDICT PASS

## Branch and head
finish/w4, worktree /home/cayleb/workspace/harness-build/fin-W4, from finish/l4a-elevate a8526aa. Head a8611b8 (the build tip was 9babcf1; the mutation-closure and checker-fix commits sit on top of it). Nothing pushed from the worktree, no sudo, no live fleet touched on purpose (one disclosed incident below).

## What was built
- Rotate display sentence (CHECK-L4A N2): token rotation tells the owner the old tokens keep working for the overlap, then stop (ccec8e1).
- Mount-readiness: `./elevate` export, files entries, fleet-elevate row INSERTed in cordis.patch.yml and pinned as a mount, regenerated rows manifest, `elevate-selfcheck`, scratch-host mount proof (1c07cae).
- Item 6: GTK4 secure password prompt. Fails closed with no secure display, shows the ticket detail, checks the kit against MANIFEST, pinned environment, Wayland display identity (H11). Tested under Broadway (c138201).
- Item 12: D1 install kit (MANIFEST, install.sh, uninstall.sh, verify.py, proof.sh) tested with fake sudo/logger/install/chown on scratch destinations (ee1a8a6).
- Also: libwayland descriptor hand-over test, sitting-only nested-Hyprland demo, core test expects the fleet-elevate pin (c88d9cf); L4-A part 2 notes and staged owner sitting (6f0e723); prompt loader never writes bytecode into the kit dir (e4f6c70).
- 9babcf1: the run resumed an interrupted session that had left uncommitted test plumbing (FS_ROOT, tool call logs, a fake ss); finished with 4 new tests. One found a REAL BUG in proof.sh: the kit proof ran `grep -rIl ... | grep -q .` under pipefail, so when one polkit folder did not exist grep exited 2 and a real polkit or sudoers match was reported as PASS. It now reads the hits, not the exit code.
- Checker-found fixes (all with a test that failed first): D1 HIGH (KeyError 'dev' in the exe check before any prompt), D2 HIGH (MANIFEST now pins every kit file as one whole-kit hash; install/uninstall refuse unlisted files, and as root a folder not root-owned; python runs -B -I), D3 LOW (journal install line anchored on _UID=0 and _EXE=/usr/bin/logger), D4 LOW (CLI names a pre-prompt refusal at once; a running request's closed prompt is progress).

## Test counts
Python kit/elevate 43 on base -> 180 (run with -B -W error -I). Node: elevate 70 (base 60), comms 53 (51), core 107 (104), policy 72/72, others equal; failing test names are identical to finish/l4a-elevate (none only on W4). Elevate-mount 4/4 on scratch hosts including negatives.

## Mutation result
Report: /home/cayleb/workspace/harness-build/finish-docs/MUTATION-W4.md. 176 mutations applied over two runs; survivors included two HIGH (E13 MANIFEST unreadable let the prompt open; S07 selfcheck PASSed on an unreachable host). Fix commits 9b9865e and 7ced5fb closed 141 of 148 re-run survivors, no product defect exposed by the closure tests. Not closed: E61 (no-op control, must survive) and six judged equivalent because another check refuses the same case with the same code (E06, E13 arm, E14, E25, E37, P32). Fix-round mutants: 36 run, 35 caught, 1 equivalent survivor (D4-c). Evidence in finish-state/W4/mutation-closure/.

## Fresh checker verdict: PASS (a different agent, real code paths, not mine)
- D1 (real entry program, real exe check on real /usr/bin/timeout and /usr/bin/true, real broker + dsh-fleet confirm, fake sudo/journalctl): CLI exit 0, ticket consumed, grant entry-program authenticated 1, outcome ok with 2 entries. The same drive against the base kit (the bug) exits 3 with the ticket pending.
- D2: MANIFEST pins 8 files, sha256sum -c all OK, whole-kit hash f364350f640d8ae2840f66cfe2d7e2a25c7d20e85c9a06e82a2a74e316eb56c9. Non-root install/uninstall refuse with nothing created. Planted __pycache__, tampered verify.py, newline-named file all refused. Root-mode owner check is seam-tested only.
- D3: python test with a forged owner-uid line fails the check; not run against the live journal.
- D4: real enter.py refuses a user-owned kit and the real CLI exits 3 at once with the manifest-mismatch reason.
- Password absence: 23,079 /proc cmdline/environ samples at 10 ms, 0 hits; byte scan (plain/base64/hex) of 631 files hit only the 3 input files; CLI output, user and system journal (20 min), /run/user/1000 and /tmp: 0. Fake sudo env only LC_ALL, PATH; stdin is a socket.
- Prompt refuses with no secure display (missing socket -> display-identity-failed; no display env -> no-secure-display).

## Limits and what is NOT done or not proven
- NOT proven on real Wayland/Hyprland: headless Hyprland will not start on this host; the prompt and its GTK handlers are proven under Broadway only. nested-hyprland.sh is staged and untested. Owner sitting.
- NOT done: the real install (owner-only, needs sudo), integration release number and regenerated FILES-0.3.5.sha256 lists (release/test installer and service tests fail on that stale list, 9 of 11 vs 3 on base), elevate-rules.json, items 3/4/8/9/10.
- NOT proven: root-mode owner check beyond the test seam; D3 against a live journal; the real-compositor root-protection check; a script changed before it ran can skip its own checks (the anchor is the owner's coreutils sha256sum -c on the staged copy).
- Note: the mount test in release/test copies the live profile; untouched, not W4's. A rare python suite flake under 4-way load (Broadway port from pid%40 collision) was not reproduced in 6 parallel runs.
- INCIDENT (mine, disclosed): an early e2e run aimed enter.py at the LIVE control socket and made one elevate-outcome report (id e1, manifest-mismatch, no secret). The live host has no fleet-elevate mount, so the expected effect is an unknown-op refusal, but I did not probe the live socket to confirm. Fixed: every e2e scene now uses its own scratch socket and the spawn asserts it.
- No key, token or password values appear in this report.

## Next step
Integration: assign the release number, regenerate the release lists, merge finish/w4 onto the integration branch with the fleet-elevate row, then the owner sitting (staged in release/OWNER-SITTING-L4A.md) for the real Wayland run and the real install.
