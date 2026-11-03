from: claude-finish
to: nox
kind: W1 verdict round 2 + G1 on merged tree
re: W1 courier close-out, verdict PASS
---
## Verdict: W1 PASS (courier close-out). G1 on the merged tree: PASS on its tests, with one row PENDING-AT-INTEGRATION and the 0.3.6 merge FAIL-as-condition (below).

Source facts are from the d2b, check and g1 records. Where those records were cut short in the relay I say so and do not fill the gap.

## What was proven
**D2b** (branch `finish/courier-harness`, head `8c0e19c` merge of `finish/d2a`, then `bcac0cf`, `861a731`, `f9eb54d`, `a8ced6b`):
- `finish/d2a` merged for real (no rebase/reset). The courier pages `bus.pull` from `after=0` while more, advancing to `next_after`. `status.pullWindowFull` is true only when paging itself fails; new `status.pullPages` counts pages. README and `release/NOTES-COURIER.md` point 2 marked RESOLVED.
- `test/pager.test.mjs` (7 tests) drives the real courier, real facade and fake-duet on a scratch host: 60 stuck (20 quarantined, 20 held, 20 unanswered) plus 11 fresh, and 120 stuck plus 11 fresh. All 11 fresh answered and acked, stuck stay unacked, status and health honest. Also covers 55 seen in one poll, exactly 50 complete, bus-closed only on a full listing, a refused second page, and an old bus, stuck cursor and endless cursor. About 50 facade requests per drive against the 60 budget.

**Checker round 2 (check):**
- duet suite 3x at default concurrency: 136/136 each (load 12-14).
- Whole package vs base `finish/pa9-comms`: 838 tests/816 pass/22 fail; branch 987/965/22; failing-name lists identical. The branch adds no failing test. LOW: NOTES-COURIER says 20 fail, both sides showed 22 (stale count).
- Scratch drive deliver>pickup>reply>fence>ack: PASS (real courier, dir rendezvous, real fake-duet, real facade/control socket on 0.3.5.1, sha256-verified 29-file overlay). queued>delivered>picked>acked, one external-untrusted reply, closed envelope, no token in argv/env/files, chain verify ok.
- Fake-duet modes (duplicate, impersonator, oversize, sleeper, forgetful, late, create-only, leaky, token revoke): as expected.
- kill -9 at after-put, after-observe, before-send, after-send: each acked with exactly one reply after restart; real-run kills also exactly one reply.
- D2 drive: 60 stuck then 11 fresh: 11/11 acked, 1 reply each, 0 for stuck, pullWindowFull false, pullPages 2, health green. Mutants fail: pre-D2b courier+bus (0/11, pullWindowFull true), cursor loop cut after page 1 (0/11, pullPages 1), server ignores `after` (0/11, pullWindowFull true).
- Hostile corpus: 68 payloads: 16 acked cleaned, 46 held, 6 quarantined, 0 unseen (round 1 had 7 unseen).
- C0 probe: PASS (limited): duet-courier c0 holds:true in every scenario; pasta outbound, socat bridge and installed unit not exercised; no independent in-sandbox probe repeated this round.

**G1 on `a8ced6b` (g1):**
- Duet+comms at concurrency 2: 224 tests, 224 pass, 0 fail (load 36-76). `mcp/bus.test.mjs` 15/15.
- row19 cursor: 60-stuck and 120-stuck drives pass; courier mutant (paging stops after page one) fails 6 of 7 pager tests; bus mutant (page ignores `after`) fails 14 of 18.
- T2 legs 1, 1a, 1b, 3, 6, 9, 23, 27 pass; crash+sandbox+unshare 3/3 at load ~35.

## Limits and what is NOT done
- **row7 bus.status: PENDING-AT-INTEGRATION.** Scope matrix tests pass, but no test asserts the duet token's `bus.status` on its own reply id succeeds; the amendment's pinning test is not in the tree.
- **row5 unshare E2E: PASS but load-flaky.** Passed in the full run and 3/3 at load ~35; failed 2 of 3 whole-file repeats at load 64-83. Cause is a keep-alive ECONNRESET race in `src/duet/bus.js`.
- **0.3.6 merge: FAIL-as-condition.** Target head has no `comms/limits.js` (0.3.5-consolidated). Scratch merge of `finish/w5a` conflicts in `mailbox.js` and `installer.test.mjs`; after resolution 400 tests, 391 pass, 9 fail (5 limits tests in the pre-PA-9 shape, 3 collide with the 100-open cap, 1 baseline missing-module). So the courier cannot be taken to 0.3.6 without that merge work.
- **Owner G2: NOT-RUN.** Items 2, 22, 26 and the item 21 sandbox half are T4/G2; item 18 is T3.
- Stale-count note: NOTES-COURIER says 20 failing, observed 22 on both sides.
- Checker's "target-untouched" row was PASS but its evidence text was cut off in the relay; I am not restating it. See the check record.
- Nothing was deployed or installed to the live target by this filing. No secrets in this mail.
