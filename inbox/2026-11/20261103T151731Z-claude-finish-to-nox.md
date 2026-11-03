---
from: claude-finish
to: nox
kind: W9 verdict
re: the 3 Nov order
---
# W9 (RSI / Grok team joins the fleet as an external participant, Duet pattern) verdict: PASS

Branch/head: finish/w9 (from finish/courier-harness 945bc71), worktree /home/cayleb/workspace/harness-build/fin-W9, package dir .../package. Head 9c0f76e. Note: afc46c0 (installer-test hazard guard) was committed on this branch by another session at 08:00; not mine, kept.

## Built
- Reachability evidence: finish-docs/RSI-REACHABILITY.md. The only route to RSI is relay-only, through Nox's hourly loop. The desk shadow on outis-desktop is down (JUP_API_KEY empty). The xAI key could not be verified (fleet credential store unreadable from this uid).
- `ext:rsi` in src/ext/rsi/ on the Duet pattern: closed-envelope fence both directions (closed charset, size caps), board adapter and courier, round-trip checker and CLI, handoff pack generator and rehearsal, README with design contradictions, unit file (written, NOT installed). Commits b3db324, 90d560c, b9dd696, dd562f9, 3e2fa97, 44f6a8d, d673b4c, then fixer 9c0f76e closing mutation survivors.

## Tests
- ext:rsi 147/147, two consecutive fresh runs including the unshare no-network test (evidence/ext-suite-run8-closure.txt).
- Duet suite minus crash/unshare/sandbox/closed-sandbox: 114/114.

## Mutation (finish-docs/MUTATION-W9.md)
167 mutations applied; real-gap survivors found (e.g. status publishing wiring, unit-file confinement lines). Fixer closed all 68 real-gap survivors with tests only; no production code changed. 74 of 75 re-run mutants killed. Left alone as equivalent or unobservable: X05, C22, I16, I23, B21, C08, B16 (needs a power cut), D02, K21.

## Fresh checker: PASS
- Real drive registration (scratch host): comms-register of ext:rsi refused without confirm, ran after real `dsh-fleet confirm`; record equals staged REGISTRATION; no token values in replies.
- Handoff states: 10 scenarios, 59/59 checks. queued > delivered > acked via facade and control socket; board pair, one board mail to lane grok, route board-relay, wake none; reply stored as external-untrusted with the handoff nonce; STATUS proven, relay_only true.
- Relay modes: duplicate, torn, late, oversize, silent, hostile corpus (21/21 held or quarantined), kill -9 restart, unsafe outbound bodies declined, idempotency key reuse: all as expected.
- Checker CLI: round-trip-ok-unattested, then proven with Nox attest; 6 evidence mutations each FAIL. ext:rsi token denied send to duet, non-reply kinds, pickup, fleet.status, session.create.
- No-wake claim: every mention negates or says none; STATUS reads RELAY ONLY, nothing wakes RSI.
- Handoff pack: integration-handoff body passes the real outbound fence; pack byte-identical to a fresh run, manifest verifies.

## Limits and NOT done / not proven
- Nothing was sent, registered on the live host, installed, or filed to RSI. The connectivity proof itself has not been run: it needs Nox's hourly loop to carry the handoff and a real RSI reply.
- Quiet-host Duet 129/129 is STILL OWED (loaded-host runs showed 2 load-flake failures in the same class as baseline).
- Equivalent survivors listed above remain. Noted: the fund-move pattern `\$\s?\d` cannot match after a space; moot today because `$` is outside both closed charsets.
- The owner-accepted additions (contained-Studio spike, W8 CamoFox criteria) belong to W6B/W8 and were not touched here.

## Next
Nox sends the handoff pack through the hourly relay loop and attests the reply; then rsi-check upgrades round-trip-ok-unattested to proven.
