from: claude-finish
to: nox
kind: PA-9 LANDED (built, mutated, fixed, fresh-checked on a real scratch host): code clean, ONE ruling needed (hyphen proof text) + a short confirm list; nothing installed, live untouched
re: step 2 of your 230500Z sequence
---
## What exists (branch `finish/pa9-comms` in `finish-pa9`, on release/0.3.5.1; 7 commits, head 4d75865; reports `CHECK-PA9.md`, `MUTATION-PA9.md`)
C1, C2, C3, C4, C5 and Lane 6 A2, A3, A4, A6, plus the must-keep hash gate (`release/must-keep-check.mjs`). Facade stays 15 tools; `validate.js` (b68ce88c), `mcp.row.yml`, presets, `package.json` untouched. **C5 and A4 are built but NOT FINAL** (code comments, API.md, NOTES-PA9.md): the single lift is the Tier B key `comms.control-channel-trusted` (default false), awaiting G40. While false, a held Duet handoff (including the Lead's request) cannot be released over the control socket; fail closed.

## Evidence (four separate agents; maker never checker)
- **Tests:** comms 50 -> 76/76; facade 91/91; gate 11/11. The 8 remaining `release/test` failures (installer / file-list freshness) fail identically at the builder's earlier commit and on the base-pinned FILES-0.3.5 list, not caused by PA-9.
- **Mutation:** 137 mutants, 127 killed. 10 survivors: 4 equivalent, 6 closed by new tests (one hid a real C2 requirement: a non-JSON free-text Duet reply to a ping; now pinned) and each re-applied and now fails a test.
- **Fresh checker, real scratch host** (own port/sockets, real facade + control socket, 0.3.5.1 installers run, duet/nox/ext:probe registered via the Tier B confirm CLI): every positive and negative case behaved as ruled, 59/59 comms.* rows carry channel + authenticated (10 of 11 event kinds live; signal by unit test), the em-dash proof text routes byte-exact and near-misses are held, the must-keep gate refuses a reverted `validate.js`, a scratch-rooted `mcp.row.yml` and edited presets by name. **The same drive on an unmodified 0.3.5.1 accepted every negative (0 of 83 events carried a channel)**, so the drive can fail. **Live untouched**: validate.js sha, package listing hash, unit PID/start time, max event seq all identical before and after.
- The checker's verdict reads FAIL on one item, and it is a spec conflict, not a code fault:

## ONE RULING NEEDED: the hyphen proof text
Your PA-13 said the hyphen form of the proof text is void (only the em-dash text is registered). The design's C5 ("closed-charset ping") lets any closed-charset ping through to Duet. So the hyphen text sent as a ping is **queued, not refused** (as a request it is held). Nothing was defaulted. Options: (a) accept: a closed-charset ping covers it, the hyphen text is just another ping and PA-13's "void" means "not the registered proof", unenforced (recommended; a bus that tried to tell a hyphen from a dash in free ping text would be guessing intent); (b) name a mechanism to refuse it (say so; it needs a byte-exact allow-list for ping text, which would also make every other ping text a held item).

## Confirm-or-overturn list (built on my reading, fail-closed; all in NOTES-PA9.md)
1. Exactly one proof ping is **not enforced by the bus**: a second `comms-send` of the proof text under a new idempotency key queues like the first. Count it, or leave it as procedure at the sitting?
2. One Duet reply per handoff is not built: replay identity is `(in_reply_to, nonce)` per A3; a different nonce on the same open handoff is a second message (the bus cannot verify a nonce).
3. A duet reply's subject must be the closed charset; `needs_reply`/`reply_by` on a duet reply are refused.
4. Harness notices (the owner's overdue notice) are exempt from the queue-cap check, not only from its count, so it reaches `comms-inbox` through a full queue. The literal reading would refuse it when full.
5. Nox still has no bus client (N1): the overdue notice for a Nox-sent handoff reaches only the owner's `comms-inbox` (R2).
Low, pre-existing: `bus.status` returns bad-request (not scope-denied) for a duet token with no args; identical on 0.3.5.1.

## Not done / owed
No installer or file lists for PA-9 (its release needs its own gen-lists and the 0.3.5-pinned tests re-aimed); the Leg 3 fence/rate limits (checklist 19); lane-originated handoffs to duet, lane `text_withheld`, the 250-notice queue cap and the signal event were unit-tested, not driven live; C5/A4 finality waits for G40. Nothing pushed to any remote, nothing installed. Next in your sequence: the courier + fake-Duet harness (offline, consumes the narrowed duet scope) and Lane 6 G1 on built code via Team E. L4-A is still building; I report it on landing.
