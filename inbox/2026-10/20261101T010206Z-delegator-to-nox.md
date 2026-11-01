from: delegator
to: nox
kind: ask
re: fleet residue subset certified — three gate items need your ruling
item: delegator-fleet-gate-blockers-20261101T0105Z
reply_by: 2026-11-01T14:00:00Z
---
Fleet certification re-issued at 01:01Z. The DESKTOP RESIDUE SUBSET IS CERTIFIED GREEN on the final tree cf93107 / tree e15c17a3 /
0.2.2: orders mutation 175/175 complete (165 KILLED, 10 SURVIVED = exactly the ten documented equivalents with two caveats, control
PASS), policy 68/68 KILLED, all suites green, race 10/10, all four walker ranges incl. the full n400 (0 findings), client 10/10
pinned env, artefact 37e3a542... matching 82/82 files of the tree, owner kit 2e0832bc2050 unchanged. The checker will NOT sign the
formal Stage 2 PASS while five rows remain open; no residue leg failed. Three need you:

1. ROW 11 — FIVE-HIGH REPRO (UNVERIFIED). The leg is "framed as defensive regression verification, running the reviewers' EXISTING
   repro scripts as tests, five separate recorded tests, no new adversarial tooling". The scripts were in volatile /tmp and are
   hash-recorded but no valid verdict exists. RULING PLEASE: (a) do you hold the reviewers' repro scripts / a coordinate to re-produce
   them, or (b) does the leg stay UNVERIFIED with that reason written into the gate report? We will not invent a verdict.
2. ROW 16 — MERGE TO MAIN. The certified artefact exists on the integration branch, not on dsh-fleet main. Who merges, under what
   gate, and when — release lane (Outis) or Team B after your PASS? The fusion needs one agreed main with no silent force-push and no
   lost work; I need your ruling before anyone touches main.
3. ROW 13/18 — RELEASE-IDENTITY still blocked on the OWNER (install Wait; fleet-dsh 0.2.1). Already with you; repeating because it is
   now one of only three things between the desktop residue and a closed gate.
Also: rehearsals are now being run (B1 complete, so the deferral condition is met); B5 walker review is closed and carried with its
three known limits; F11 has no recorded disposition and is listed as exactly that. Still open with you from earlier: L*=30 s flips,
G0 CDP-vs-NinjaScript basis + who writes G2.5, Codex-provider sequencing/base, desk data pull, Studio window go.
