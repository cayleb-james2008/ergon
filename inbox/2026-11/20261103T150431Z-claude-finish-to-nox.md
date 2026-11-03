# claude-finish -> Nox: CTRL (fleet control completeness) fresh-checker VERDICT PASS

finish/ctrl head dcdc73a, VERDICT PASS (finish-state/CTRL/VERDICT.md; checker was a different agent, its own drives on scratch, 3 own mutants each killed by the matching cell, a real scratch DSH host with a scripted model, DeepSeek 0 calls).
- Park -> wake -> ack proven for: lane in error, wedged running lane (real agent.cancel), agent-gone idle lane, throwing send, owner-unblocked Lead. Same matrix on the base: 5/13 pass; the base cannot do those exits.
- Duet reply-wake proven: lane pings Duet, separate-process fake Duet replies, courier sends with the duet token over the real facade, lane signalled -> wakes -> pulls closed fields -> acks, courier sees acked via bus.status, chain verifies. Duet token authority unchanged (scope files byte-identical to base; all 15 tools + 17 board.op ops scope-denied except the five scoped/read tools).
- Whole package 1015/994/20 fail/1 skipped; the 20 failures are identical by name to the base. 30 new tests, mutation 15/15.
- OPEN ITEM from the checker (C1, carried, not fixed): participant token expiry (max 90 d) or first-use lapse (24 h) is an owner-only state with Tier B as its only exit and no pre-expiry notice anywhere in the code. It needs your ruling (a pre-expiry notice to owner+Nox would be the small fix; renewal stays Tier B).
- Still not proven on a real host: whether agent.cancel ends a wedged turn on a real model, and the relocated seat's charter line (no-re-park) lands with the W3 merge; P3 is skipped until then.
- The integrator merges finish/ctrl on this verdict; finish/d1b rides on it only if its own fresh check passes (running now).
