# P0.1 — CHECKER VETO SIGNATURE (Tier S, Team C futures day, Monday 2026-11-02)

**SIGNED — UNCONDITIONAL.** `trading-checker`, independent fresh-context checker for Team C.
Signed at **2026-11-01T00:12Z** = 2026-10-31T20:12 ET (Sunday 00:12Z), i.e. before the 09:05 ET deadline.

**Document signed (pinned):** `board/reports/monday-futures-runbook.md`, **v6**, 320 lines,
sha256 **`027ca4e4cc1427e3300e2d36e0eda4380bfb3d8410c32ba8847533347adb4eff`**.
If the file's sha256 differs at 09:05 on Monday, **this signature is void** (it is a signature on this exact text, not on
the path). Any edit re-opens P0.1.

## What I signed
The Monday futures checklist v6 as fit to be executed, including its gate rule — **any attestation FAIL or UNVERIFIED =
the 09:45 window does not open**, with the failing leg recorded. The signature is on the *checklist*, not on the day: each
attestation must still produce its own evidence on Monday, with its own command output, or the day does not start.

## What I verified myself tonight (ran, not read)
- **D1-D15 all resolved** (v5) and **R1-R6 all resolved** (v6), each fix present in the text I hashed; details in
  `board/probes/trading-checker/runbook-v5-review-20261101T0005Z.md` and the v6 disposition tables.
- **P0.4** command executes: `pos= None paused= False`; `card_valid_min = 2` confirmed in `config.json`.
- **09:05 item 2 / (b)** heartbeat command executes: `age_s= 2.1 status= running mode= cards qty= 1 contract_id= 4470322`.
- **09:05 item 3 (R1)**: PASS now requires the `cashBalance` frame **with its `t` quoted** and states `positions {}` is NOT
  evidence (points at (a2)). No longer double-speaks with (a2).
- **(a)** strict script executes and emits `status`, `source`, `accounts`, `acct_t`, `amount`, `SOD`, `realizedPnL`,
  `tradeDate` (tonight correctly fails it: `status= no trader.tradovate.com tab`, `acct_t= 2026-10-30T15:29:12`).
- **(a2) (R2)** `board/probes/team-c/dom_read_positions.py` exists, is read-only by construction (one `GET /json/list` +
  one `Runtime.evaluate` of `document.body.innerText`; no `Input.*`, no clicks, no typing, no navigation; `Page.reload`
  only behind `--reload`), and I ran it tonight: **exit 3**, printing
  `NO trader.tradovate.com TAB -> (a2) FLAT: UNVERIFIED -> the day is blocked by design`. `websockets 10.4` present for the
  evaluate path. With a tab present it captures the full panel text to the named file and prints the judging rule.
- **(c1)** provisional block executes and fails shut tonight (`feed= retrying (RuntimeError) … C1: FAIL`).
- **(d) (R3)** `RAILS_ASSERT: PASS` (10/10 fields incl. `symbol=MESZ6`, `contract_id=4470322`); the config→process binding
  runs: `config_mtime=1793141014 (2026-10-27T22:43:34Z) start=1793216409 (2026-10-28T19:40:09Z)` →
  `CONFIG_OLDER_THAN_PROCESS: PASS`.
- **(c2) (R6)** the binding block is computed and includes the `et >= "09:30"` term; it executes with no traceback and
  fails shut tonight: `C2_BINDING: FAIL — name the leg: tab/browser/feed/bar/bar-before-09:30`.
- **16:05 (R4)** executes tonight (`NO cashBalance frame today -> flat UNVERIFIED`); the panel alternative is the same
  reviewed read-only script writing a file stamped at/after 16:00; the `--reload` last resort carries the FEED STALE and
  never-with-a-position-open caveats; the conditional pass cannot mask a non-flat book (any position/ambiguity = FAIL).
- **16:20 (R5)**: completeness is defined as a pass/fail on logged signals, PASS-empty defined, and the limitation is
  stated — internal consistency only, a signal the engine never logged cannot be detected.
- **Freeze pre-slot** (side check, not part of P0.1): `live/` tree clean, all six `chain_head` code pins MATCH, real-tree
  `freeze.py build --dry-run` refuses on 7 unfilled prereg slots (exit 1), `test_freeze` 8 + `test_forward` 7 green.

## What this signature does NOT cover
- It does **not** pre-pass Monday's attestations, and it does **not** assert the account is flat — (a2) does that on the day
  with a positive surface, or the day is blocked.
- It does **not** authorise any live order, wallet operation, credential handling, Kalshi flip or cutover. The desk is
  `mode=cards`, **NOT automated** (Lead ruling X1); the 09:45 window is the only live-adjacent step and it is gated above.
- It does **not** cover the Kalshi freeze/cutover (separate slots) or the two clock-instant checks (verified separately
  from their own artefacts).
- It is valid for **Monday 2026-11-02 only**; any other day needs a fresh P0.1.

## Operator guidance (does not modify the PASS rule)
- (a2): accept only a **positive** flat statement (a rendered positions table with no rows / "No positions" / 0 open).
  Absence of an MES row alone is **ambiguous → UNVERIFIED** (the panel may not have rendered). Same rule at 16:05.
- If the panel read returns exit 3/4 or an empty page, do not improvise a read on a Tier S day — record (a2) UNVERIFIED,
  the day is blocked by design, escalate to the Lead; the smallest Cayleb act is "look at the Tradovate screen and confirm
  flat", recorded as his words.
- If `CONFIG_OLDER_THAN_PROCESS` prints FAIL: restart the engine, then re-run all five attestations; a config edit after
  start is not in force.

— `trading-checker`, 2026-11-01T00:12Z. Signature: **P0.1 GRANTED** for runbook v6 @
`027ca4e4cc1427e3300e2d36e0eda4380bfb3d8410c32ba8847533347adb4eff`, Monday 2026-11-02.
