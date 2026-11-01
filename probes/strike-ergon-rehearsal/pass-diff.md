# Pass diff — second from-scratch pass (R7)

Generated: 2026-11-01T22:07:38Z

- Pass 2 (second) compared with pass 1 (first).
- Seed-state digest: pass 1 `558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1` · pass 2 `558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1` (identical)
- Verdict tables identical: **yes**
- Differences: none (ids, timestamps and the mktemp root differ by design and are not compared)

## Verdict table — pass 1

| Step | Word | Named piece / notes |
|---|---|---|
| R0 | clean | read-only install proof: control socket status/integrity/policy, fleet-dsh unit, installed package identity digest + authorised wrapper, repo pin, repo r0-preflight. Dedicated fresh host/DNS/Supabase are owner acts, named out of scope. |
| R1 | clean | fixture company/cast/seed stood up in a fresh mktemp root through the product's own modules; canonical state dump hash matches the pinned digest in both passes. The installer first-run UI is not in the installed plugin (stand-in named). |
| R2 | pending | fixture profile + onboarding session per cast member; onboarding driven to active through the product protocol with fixture answers (stand-in for the employee's own words); one real fleet agent session per cast member on the installed plugin. The model-quality assistant leg is PENDING: no provider credential (owner act). |
| R3 | pending | company host origin recorded from the fixture config and shape-checked; the fixture origin is a made-up name that does not resolve and the host binds 127.0.0.1 only. Hosted non-localhost URL + DNS + second-device reachability PENDING (not built; owner acts out of scope). |
| R4 | pending | host-side device registry row for the warehouse lead's phone recorded as a stand-in (operator-registered, approved). The phone surface is PACKAGE-PENDING (no phone client; a phone cannot use the fleet join — F5), so the step is pending with the missing piece named. |
| R5 | clean | scenario (a)+(b) ticket round trip through the product's control service (ids, states, receipts, device readback, recorded product limits F3/F4/F6) AND a real warehouse-board-shaped task as a fleet order on the installed plugin (create -> decompose -> claim -> built -> check -> verdict -> bank -> done). Stand-ins named: Get Help/phone UI, scripted board. |
| R6 | pending | voice/phone surface PACKAGE-PENDING (P20-F5): no voice button, dictation client or phone surface on the installed plugin. Supporting fixture check only: the cast's written voices separate blind; that is not the voice surface and the step stays pending. |
| R7 | clean | state record (company/cast/tickets/board/order/digests) and teardown of the fresh company root; the second from-scratch pass runs R0-R7 into a new root and its verdict table is diffed (R7 is clean only when the tables match). |

## Verdict table — pass 2

| Step | Word | Named piece / notes |
|---|---|---|
| R0 | clean | read-only install proof: control socket status/integrity/policy, fleet-dsh unit, installed package identity digest + authorised wrapper, repo pin, repo r0-preflight. Dedicated fresh host/DNS/Supabase are owner acts, named out of scope. |
| R1 | clean | fixture company/cast/seed stood up in a fresh mktemp root through the product's own modules; canonical state dump hash matches the pinned digest in both passes. The installer first-run UI is not in the installed plugin (stand-in named). |
| R2 | pending | fixture profile + onboarding session per cast member; onboarding driven to active through the product protocol with fixture answers (stand-in for the employee's own words); one real fleet agent session per cast member on the installed plugin. The model-quality assistant leg is PENDING: no provider credential (owner act). |
| R3 | pending | company host origin recorded from the fixture config and shape-checked; the fixture origin is a made-up name that does not resolve and the host binds 127.0.0.1 only. Hosted non-localhost URL + DNS + second-device reachability PENDING (not built; owner acts out of scope). |
| R4 | pending | host-side device registry row for the warehouse lead's phone recorded as a stand-in (operator-registered, approved). The phone surface is PACKAGE-PENDING (no phone client; a phone cannot use the fleet join — F5), so the step is pending with the missing piece named. |
| R5 | clean | scenario (a)+(b) ticket round trip through the product's control service (ids, states, receipts, device readback, recorded product limits F3/F4/F6) AND a real warehouse-board-shaped task as a fleet order on the installed plugin (create -> decompose -> claim -> built -> check -> verdict -> bank -> done). Stand-ins named: Get Help/phone UI, scripted board. |
| R6 | pending | voice/phone surface PACKAGE-PENDING (P20-F5): no voice button, dictation client or phone surface on the installed plugin. Supporting fixture check only: the cast's written voices separate blind; that is not the voice surface and the step stays pending. |
| R7 | clean | state record (company/cast/tickets/board/order/digests) and teardown of the fresh company root; the second from-scratch pass runs R0-R7 into a new root and its verdict table is diffed (R7 is clean only when the tables match). |

## Differences

None. The second pass reproduced the same words, the same named pending pieces and the same hard-check results from a fresh company root.

The two pass indexes render the same verdict table (`pass-1/index.md`, `pass-2/index.md`): the driver's DONE-WHEN extracts the `VERDICT-TABLE` blocks from both and diffs them, recording the result in `verdict-table.diff` at the pack root.
