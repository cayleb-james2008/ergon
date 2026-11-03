---
from: claude-finish
to: nox
kind: W3 verdict
re: the 3 Nov order
---

# W3 verdict PASS: relocation execution, the delegator becomes the singleton fleet lane (design Part B rev 3e), build and scratch rehearsal

Branch finish/w3, worktree /home/cayleb/workspace/harness-build/fin-W3 (from release/0.3.5.1 1f1f740). Head e81e4f2 (build head 133417d, then the survivor-fix commits on top). Everything ran on scratch hosts or synthetic data. The live host was not touched and nothing sent the proof text.

## What was built
- Delegator seat (B3.2/B3.3): singleton role outside ROLES, fleet-delegator preset (pinned count 9 to 10), charter file, read-only board view with no claim tokens, D3 DeepSeek shipped defaults.
- Scratch rehearsal (B4.1) and delegator wake proof (B4.2) on a booted scratch host with a fake loopback model and labelled test pings.
- B3.4 distillation, pack-verify, extended writers scan and host guard, pre-deletion checker (B5.4); B3.7 transitional Nox mail relay (read-only fleet.db); B4.4 continuity key and grader.
- Owner-run deletion script with a tty-only scrypt passphrase guard (sixteen refusals of B8 item 22 tested under a pty), archive-build.sh and archive-verify.sh, post-switch gate (A12, A10', H10', pin digest), reviewed deletion list, b8-verify, transcript-audit, continuity pipeline rehearsal on the scratch delegator lane (C18 act enforced).

## Tests
- Seat suite 19/19, its 19 mutants all killed. Package suite: 991 tests, 986 pass, 5 fail, the same 5 that fail on base (first-run x3 version string, client x2 need react).
- Relocation suite release/relocation/test: 131/131 (was 47-48 before the survivor stage).
- Scratch wake proof: PASS, 41 checks, including the C18 act observed for real (ACK written inside the lane, write outside refused).

## Mutation result
156 mutations applied by a fresh tester (MUTATION-W3.md, /home/cayleb/workspace/harness-build/finish-docs/). 45 killed, 111 survived. Fix stage: 108 of the 111 now fail a named test when mutated (107 plus W02 own mutation; proof in finish-state/W3/survivors-proof.mjs and .jsonl). 3 not closed, all equivalent: S08, S15, L07. One real defect found and fixed earlier in the resume (distill per-section byte budget, 133417d); no other defect exposed by the survivor tests.

## Fresh checker verdict: PASS (CHECK-W3.md, finish-docs/)
13 checks, all PASS: boot with 10 pinned; singleton lane (second spawn delegator-exists, builder asking the principal reserved-principal); wake by labelled handoff over the control socket with T0..T3 from read-only fleet.db, facade unchanged at 15 tools; scratch restart; capability reduction (40 tool calls, write outside lane, ../, sibling lane and sandbox_permissions escalation refused; removed tools unknown); board view has no token at any depth (positive controls show the scan can fail); D3 defaults and ENGINE_MODELS block identical to base; distillation with planted canaries 0 hits and no credential file opened; 4 of 4 guard mutants killed; at least one drive fails on base; tampered pin or role refuses the seat; live fleet untouched (events.seq 2189 before and after). Four minor findings, none blocking: second handoff within the cooldown is picked up in about 2.5 minutes; release/test shows 4 more failures than base from a stale FILES-0.3.5.sha256 (integrator step); SESSION-MAP wording; pack curation is still a human step. Note: the checker ran at e81e4f2.

## Limits, and what is NOT done / not proven
- The model in every proof is a loopback fake. What a real DeepSeek model does is not tested. DeepSeek/OpenCode Go calls 0, Anthropic calls 0.
- Not done (per STATE.md B7 table): live bus drive (needs owner), P1 wake proof and the 20 continuity sends on the live host (owner sends the proof), scratch-home switch test with a second Electron instance (needs Nox go, starts a GUI), install/lane-spawn/lane-start on the live host (needs Nox go), deletion and archive on real data (owner-run), B6 records edits, integration commit (gen-lists, version bump, VARIANT.md, --expect-pinned carriers: integrator).
- Not proven: real session-store JSON fields vs the metadata reader; whether the seat can use the "escalation available" text (checker showed the host guard refuses it); pack-verify REDACTIONS check accepts any capitalised line with no pipe (observation, untightened); the 8 mutation-sensitive seat behaviours are covered offline by the fake, not by a real model.

## Next step
Integrator stage (gen-lists, version bump, carriers), then Nox go for the live install and the owner's G2 proof ping.
