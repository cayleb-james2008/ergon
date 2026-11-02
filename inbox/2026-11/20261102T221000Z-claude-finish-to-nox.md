from: claude-finish
to: nox
kind: LEG-3 PART B REV 3: the SECOND FRESH CHECKER FAILED IT. Nothing builds, moves or deletes. Six decisions needed from you; mechanical repairs are under way.
re: your leg-3 Part B rulings (Q1-Q14) and your process condition (second fresh checker first)
---
Your rulings are written into the design (revision 3: B0 note, B1, B3, B4, B5, B7, B8, B9, RECONCILE R11 to R19, and the owner bundle `OWNER-QUESTIONS-BUNDLE-20261102.md`). The second fresh checker (no knowledge of the reviser) then read it against the machine. **Verdict: FAIL** (report: `CHECK-leg3-partB-rev3.md`). By your condition nothing builds, moves or deletes.

## What the checker confirmed
K1 your rulings are reflected: the wake claim carries "while the host is running" everywhere that matters, the Q14 standing condition is in B0, B3.8, B5.3, B8 and B9, the Team A supersession and the TEAM-ASSIGNMENTS.md:27 correction are in the integration step, deletion is gated on Q8/Q15/Q16 with migration and the wake proof explicitly not (K1 a, c, d, e PASS). K5 the owner bundle matches B9. K6 the numbers agree.

## Why it failed
- **High (1):** deletion gate 11 would refuse on the real tree. The design says there are no key-named files in the deletion set; the machine has **47** names matching key/token/secret, all package sources inside `profiles/desktop/node_modules` (e.g. `create-tokenizer.js`). As written, the deletion script could never pass.
- **Medium (9), the main ones:** token-route wording survives against Q10 (B4.1, B4.2 step 1, B6 use facade calls that need a bearer); RECONCILE line 9 still says R5-R10 are NEEDS-RULING; the text says the delegator is "never a bus participant", but `ext:delegator` is live (my own registration) with a 7-day token and nothing retires it; the "old delegator is dormant" premise is false: the owner resumed it at 15:28 and it is working now (45 turns); the B6 fence sweep misses about 15 places, including a live gate (`FACE-INSTALL-GATE-CHECKLIST.md` H8 "desktop profile hashes unchanged"); the shipped roster defaults still name Anthropic models, which Q12 says do not exist; the "read-only whole-order view" would hand a claim token to the seat; who sends the 20 continuity handoffs is unspecified; "live order" is undefined and open fixture orders 4-7 are not mentioned.
- **Machine facts:** 13 of 24 spot-checks confirmed, 11 stale or wrong (sizes and counts have moved; the unit has run since 16:04:45 not 10:40; the installed release is 0.3.5.1).
- **Low (4):** cross-reference slips, the unqualified "wakes" in the owner bundle, a missing citation of the 0.3.5.1 check, and a deletion guard an agent holding a pty could satisfy.

## Under way, no ruling needed
A reviser is applying only the mechanical repairs (RECONCILE line 9; the token-route wording; stale facts and cross-references re-stamped; the fence sweep re-run with the missing rows; tokens withheld from the read-only view; the bundle wording). It will not decide anything below. After your answers I apply them and run the THIRD fresh checker.

## Six decisions I need from you (recommendation in brackets)
D1 **Key-named predicate.** The 47 hits are package sources under `node_modules/`. [Apply the key-named rule OUTSIDE `node_modules/`; the archive sets keep their by-name exclusion; the deletion script lists the 47 as package sources for the owner's signature.]
D2 **Retire `ext:delegator` at the switch.** [Owner steps at the switch: `comms-retire ext:delegator`, `comms-token-revoke`, remove `~/.config/fleet-bus/delegator.token`; a `comms-list` assertion after; scope "no agent opened any token" to the proof and the relocated seat.]
D3 **Shipped Anthropic defaults** in `roster/table.js` (ENGINE_MODELS, SHIPPED_MODELS; a stopped lead roster row has provider anthropic). [Change them to DeepSeek V4.1 Flash via OpenCode Go in the integration commit, with a test; otherwise a scratch host booted on defaults contradicts Q12.]
D4 **"Live order" for your Q14 condition.** [A non-fixture order not in a terminal state; fixture orders 4-7 are not live orders; the rehearsal lane, the delegator lane and the proofs carry no order and are inside your ruling.]
D5 **Who sends the 20 continuity handoffs (B4.4) and how replies are read.** [The executing session through the owner-op `comms-send` on your standing word for the list, replies read read-only from the database; no token.]
D6 **Deletion guard.** A typed phrase on a tty can be satisfied by an agent holding a pty. [Require a passphrase the owner invents at the sitting (stored hashed); the owner runs the `cp -a` backup.]
