# TEAM C — FUTURES STACK RESTORE + SUNDAY SESSION EVALUATION — 1 Nov 2026 (Nox 183400Z / 182800Z)

trading-lead, 2026-11-01 13:2x–13:3x ET. **Restore first, one report** (Nox order). No live orders, no threshold/gate
changes, no credential values handled by the team; desk CDP route only (no host input injection). Evidence is quoted
with commands/timestamps; anything not proven is UNVERIFIED.

## PART 1 — RESTORE: per-attestation status

### (a) BROKER — **BLOCKED (smallest owner act named)**
- Desk route works: Chromium 9222 up; opened `https://trader.tradovate.com` in the desk's own browser via CDP
  (`PUT /json/new`); login page loads (`#name-input`, `#password-input`; title "Tradovate").
- **Saved login did not surface**: real CDP mouse/key gestures on the fields produced no autofill; the Google-SSO
  button click opened no OAuth page; **no tradovate item exists in the keyring under site/application attributes**
  (attributes-only query; empty). We did not read, print, copy or store any credential value.
- Broker mirror still Friday: `account_state.json status="no trader.tradovate.com tab"` at first read (now `attached`
  because the login page exists — page attachment, **not** a session), `acct_t=2026-10-30T15:29:12 ET`,
  $50,000 / SOD $50,000 / realized $0.0, flat; no `account-2026-11-*` frames.
- **Smallest owner act:** at the desk, open the existing Tradovate login tab and sign in with the vault/saved login
  (or tell the team the exact vault route the lane's browser should use), then say "signed in". No credential value comes
  to the team; equity/flatness are then verified from the broker record immediately.

### (b) HEARTBEAT — engine PASS, broker FAIL (consequence of (a))
- Engine heartbeat **current: age 4.7 s** (`heartbeat.json`: `status=running`, `mode=cards`, `paused=False`, `pos=None`).
- Broker/account heartbeat **stale: 163,855 s ≈ 45.5 h** (`acct_t` Fri 15:29:12 ET); feed heartbeat `feed_age_s`
  same 45.5 h. Until (a), the *session* heartbeat cannot be current.

### (c) FEED — **NOT PROVEN (blocked by (a), then needs the 18:00 ET open)**
- Last bar/tick **Fri 2026-10-30 16:59 ET** (`last_bar_et=16:59`); `heartbeat.feed="ws"` (attached to the login page)
  but `feed_age_s=163,855 s`; no `live/bars/2026-11-01.jsonl`; `tabkeeper.json tradovate="login"`.
- Cannot prove "updating now" until the broker session is restored (market reopens 18:00 ET today).

### (d) ARMED — **engine-side PASS, session ARMED PENDING (a)**
- `openbell-live` = active/running, **PID 1484, continuous since Wed 2026-10-28 15:40:09 EDT**; heartbeat fresh,
  `paused=False`, `pos=None`; config rails asserted: **RAILS_ASSERT: PASS** (mode cards, 1 contract, 2 trades/day,
  −400/+900, cooldown 15, flat 16:05, min stop 2.0, MESZ6, contract_id 4470322).
- Session ARMED attestation cannot be issued: broker session not live, feed dead, flat proof stale.

### (e) EXECUTION MODE — **read from the engine's actual config: `mode = cards` → NOTIFY-ONLY / NOT AUTOMATED**
- `live/config.json` `mode="cards"`; engine's own docstring: *"shadow ledger + ntfy phone cards. It NEVER places,
  modifies or cancels orders"*; mode comment: `'shadow' = informational cards | 'cards' = actionable [cards for a human]`.
  There is **no autonomous mode** in the engine and no "automation switch" to flip — the only legal values are
  `shadow|cards`; actionable means a card is sent and a **human** places the bracket.
- The contained autonomous executor is **G2-signed and dry-run-only** (G3/G4 unsigned, no Cayleb written go) → it cannot
  place an order tonight or Monday.
- **So tonight cannot be an order-capable autonomous session.** It can at most be: stack restore → observation /
  supervised `cards` rehearsal (human-tap only). The smallest owner act that would change that is the G3 demo access +
  G4 written go (not a tonight item); for a cards day, the human tap is Cayleb's, not the team's.

### (f) DECISION LOGGING — **ARMED PASS; PROVEN ON pending the first signal**
- `live/ledger` writable; engine `active`; yesterday's ledger `live/ledger/2026-10-30.jsonl` exists with 3 records
  (`signal` side −1, `fill` 7649.50, `exit` 7628.50, `usd_total=+102.5`).
- "PROVEN ON" (today's ledger created and matching the first card) requires a live feed → after (a)+(c).

### (g) DEBT — **Friday 30 Oct post-close reconciliation PRODUCED in the goal files**
- File: `board/reports/team-c-friday-reconciliation-20261030.md`.
- Verdict: **broker flat $50,000.00 at Fri 15:29:12 ET; after that UNVERIFIED** (last `dom` 17:52:13 equity 0.0 is not a
  balance); engine book **+$102.50 card-only**, no broker fills/fees, no broker counterpart. No manufactured fills.

## PART 2 — SUNDAY SESSION EVALUATION (Nox 182800Z item 1)
Full ruling: `board/reports/team-c-sunday-session-ruling-20261101.md`. Summary: **ORB15 does NOT map onto the 18:00 ET
Globex open — it is a regular-session instrument by construction** (`bt/features.py` RTH 09:30–16:00; ORB returns no
signals when `not f.rth`; entry window 09:46–12:00; flat 16:05). Re-anchoring to the Globex open would be a new,
unvalidated strategy. **Monday stands** (cards/guard only). Additional finding for the Lead/Nox: the ORB15 config's own
role note says *"shadow plumbing check only … lost $4.31/trade, PF 0.81, 0/9 years positive … do NOT trade its cards"* —
that conflicts with the live-beta booking and needs a ruling before any live/autonomous ORB session.

## PART 3 — What tonight can actually be, and the owner acts
- **Restored, not order-capable:** tonight can be an observation / supervised cards rehearsal at most, and only after the
  broker sign-in. It **cannot** be the "first autonomous session" — no autonomous mode exists, G3/G4 are unsigned and
  there is no written go. Nothing will be improvised.
- **Owner acts (smallest first):** (1) sign in on the existing Tradovate tab (restores a/b/c/d/f); (2) rotate the stored
  Google OAuth token surfaced by the keyring probe (INCIDENT, reported separately); (3) later, for an order-capable
  session: G3 demo access + G4 written go.
- **Rails held:** no live orders, no threshold/gate changes, no credential values handled, no host input injection
  (CDP to the desk's own Chromium only), maker-never-checker (a fresh checker must veto any attestation that gates a
  session).

## Fences / reproducibility
Read-only on engine/config/broker records except opening the Tradovate tab in the desk Chromium; no orders; no spend.
Commands/timestamps throughout the session (13:2x–13:3x ET). The credential-exposure incident is reported to the Lead:
a keyring attributes probe printed an unrelated stored Google OAuth token to tool output only; no file/log contains it;
keyring probes stopped; rotation recommended.

## POST-SIGN-IN UPDATE — 2026-11-01T19:21:33Z (14:21:33 ET)

Read-only re-attestation after the owner's 13:59 ET Tradovate sign-in (detector RESTORED 18:59:11Z). This supersedes
PART 1 (a)–(c) "BLOCKED / not live" **for the facts below only**; nothing above is deleted or rewritten. All values
read at 19:21:33Z from the files named; no orders, no config/threshold/gate edits, no credential values, account masked.

**(a) BROKER — PASS (session live).** `live/account_state.json`: `status="attached"`, `source="live"`, `tab_path="/"`.
Account `FN***2147`: `cashBalance` $50,000.00, `amountSOD` $50,000.00, `realizedPnL` $0.00,
`tradeDate` 2026-10-26 (last broker SOD, unchanged). Flat: `positions={}`, `dom.open_pnl=0.0`, `dom.equity=50000.0`.
Freshness: `attached_t` 2026-11-01T13:26:29 ET (3,304 s before check — it is the tab attachment, **pre-sign-in**, not
the session); `sync_t` 2026-11-01T13:59:02 ET (1,351 s — the sign-in sync; static only because the market is closed and
there are no new account events); `frame_t` 2026-11-01T19:21:29Z (4.1 s) — the WS frame clock is current.

**(b) HEARTBEAT — PASS.** Engine `live/heartbeat.json.t` age ≈4.0 s (`status="running"`, `paused=false`, `pos=null`,
`mode="cards"`, `qty=1`). Session/account heartbeat: `account_state.json.t` age ≈2.8 s, `frame_t` age ≈4.1 s,
`dom.t` age ≈12.9 s. (Restore-time detector line: `heartbeat_age=4.2`.)

**(c) FEED — attached, tick proof PENDING.** `heartbeat.json`: `feed="ws"`, `feed_age_s=1345.4` (≈22m25s),
`last_price=7614.5`, `last_bar_et="13:30"`, `market_open=false`, `bars=4904` (cumulative). `live/bars/2026-11-01.jsonl`:
1 bar, ts 2026-11-01T13:30:00 ET, `v=0 n=1` — written at attach, **predates the sign-in and is not a post-sign-in
tick**. Last real session `live/bars/2026-10-30.jsonl`: 1,005 bars, last 2026-10-30T16:59:00 ET. Market closed now;
Sunday Globex open 18:00 ET = 23:00Z, ~3h38m after this check. **The feed tick proof is pending that open** — a rising
`feed_age_s` while closed is expected, not a fault.

**(d) ARMED — engine-side PASS; session ARMED ≠ order-capable.** `live/config.json`: `mode="cards"`, `contracts=1`,
`max_trades_per_day=2` (per-strategy basis, `trade_limit_basis`), `daily_loss_stop_usd=-400`,
`daily_profit_stop_usd=900`, `cooldown_min_after_loss=15`, `flat_et="16:05"`, `min_stop_pts=2.0` (strategy
`params.min_stop=2.5`), `symbol="MESZ6"`, `contract_id=4470322` (matches `heartbeat.contract_id`).
**Explicit: this ARMED state is a rails/heartbeat assertion only. It cannot become an order-capable claim — the
engine has no autonomous order path, and any card is human-tapped.**

**(e) EXECUTION MODE — `cards` = notify-only; NOT automated; tonight is OBSERVATION-ONLY.** `config.json.mode="cards"`
and `heartbeat.mode="cards"`; the engine's only legal modes are `shadow|cards`; cards are information for a human,
never instructions. No autonomous mode exists; G2 is dry-run only and G3/G4 are unsigned — the owner's written go
cannot be exercised.

**(f) DECISION LOGGING — ARMED PASS / PROVEN PENDING.** `live/ledger/account-2026-11-01.jsonl` exists, 8 rows, written
today 13:59:02–14:07:12 ET; kinds: `account`, `cashBalance`, `accountRiskStatus`, `userAccountAutoLiq`, `dom`,
`contract`×3. No `signal`/`card`/`fill`/`exit` row exists (correct: no live signal before the open), so
**"PROVEN ON" is not claimed**.

**Correction of record:** PART 1 (a)/(b)/(c)/(d) is superseded by the 13:59:02 ET restore — the broker session is live
and the mirror is current. It is still **not an order-capable session**; tonight is observation-only, and the first
feed-tick proof waits on the 18:00 ET open.
