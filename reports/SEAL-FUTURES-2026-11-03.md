# SEAL RECORD — FUTURES DESK, 3 NOVEMBER 2026 (ET)

Filed: 4 Nov 2026 ~00:20 ET by the OPENBELL stream lead (model `opencode-go/deepseek-v4.1-flash`).
Sealed day: **Tuesday 3 November 2026**. Scope: the futures signal engine, the ORB15 executor, the rails, the hourly
self-improvement loop, and the Kalshi shadow desk's liveness as it stood at the seal read.
This is a **record of state read from disk**, not a desk day: the formal desk record (`TUESDAY-20261103-DESK-RECORD.md`)
already stands as **NO-DESK-DAY**, and nothing here backfills it.

## 1. What the day was

| fact | value | source |
|---|---|---|
| Engine (`openbell-live.service`) | running, feed `ws`, `market_open true`, `pos null`, `day_pnl 0.0`, `bars 6422`, last bar 23:07 ET | `live/heartbeat.json` |
| Engine trades today | 0 | `live/heartbeat.json` (`trades 0`) |
| Executor (`openbell-executor.service`) | running since 11:14:32 ET, `trades 0`, `placed []`; **live venue** — see §7 correction: the running config is `mode: live`, `venue: tradovate` | unit state + `exec/state/day-2026-11-03.json` + the executor's own `exec/exec_config.json` (§7) |
| Attestation | **PASS** (five legs, no failures), attested 11:12:25 ET by `builder-claude`, inspected by a **different seat** (`inspector-seat-claude-sonnet`, record digest `d4544c47…b41372`) | `exec/state/attestation-2026-11-03.json` |
| Enable window | 09:29:20 → **17:00:00 ET**; halt at 17:00:01 `leg_failed:armed` (expiry, as designed) | `exec/state/enabled.json`, `day-2026-11-03.json` |
| Rails | -$400 / +$900 / 2 trades / 15-min cooldown / flat 16:05 — **unchanged, not edited** | `exec/exec_config.json` sha256 `c924d3fd…315eff`, mtime 2026-11-01 11:39 |
| Owner's own trades | 3 ES contracts, 07:20:08 → 07:22:11 ET, **+$445.28 net** — owner P/L, **not** engine/executor performance and not a rail event | `day-2026-11-03.json.owner_activity`, reconciled from the account ledger |
| Strategy loop | 13 cycles today, all `HOLD`; today's cause `FEED_SILENCE`; 18 cycles in the chain total | `live/loop/cycle-log.jsonl` |

The desk never traded the engine's money: 0 fills, 0 closed trips, 0 net. The only fills on the account were the
owner's manual trades, recorded separately and never summed into engine P&L.

## 2. Loop seal (hash chain)

```
cd ~/Work/projects/openbell-futures-loop && python3 -m live.loop.cycle --status
FRESH: newest cycle 2026-11-03T23:07:00-05:00 kind=cycle decision=HOLD; chain ok
```

- Chain verified end to end at seal time: **ok** (no broken link).
- Chain head `record_hash f0f5395465ccad871bf0bf154ab338c475b60148363c86a7ea57bb9631a21335`,
  `prev_hash 8eaa0d69844885bbec5e02b6da455ad5b9017b8e25aaaca04a969436c845d986`.
- Today's cycle reasons (verbatim from the log): `R-FEED: today's session is FEED_SILENCE (excluded from the streak;
  never a parameter change)`; `R-SILENT: last complete sessions under orb15-v1: 2026-10-30=SIGNAL_NO_EXECUTOR,
  2026-11-02=SIGNAL_NO_EXECUTOR, 2026-11-03=FEED_SILENCE -> not fired`; `R-FILLS: 0 fill(s) / 0 closed under orb15-v1,
  net $+0.00 … (judging needs >= 20)`.
- The loop wrote only under `live/loop/` (units: `ProtectSystem=strict`, `ReadWritePaths=…/live/loop`).

## 3. Evidence digests (sha256)

```
c140da5e91eca934aa13c2810ca743ea00603580627d26052cfc2de7345e89db  openbell-executor/live/exec/state/day-2026-11-03.json
abc868a7c62aa75b6fe08ded2de7048c4adddb0a532bd6bfbba0ee3bc1794561  openbell-executor/live/exec/state/attestation-2026-11-03.json
4e852ae7cc69c98a3a59d3e14fc24eaeebd477a46cf2f674dec1e22c50638858  openbell-executor/live/exec/state/enabled.json
6750772214c4d82281afd45a4d1556f17986321e57c82c0b79d1bcc084644215  openbell-futures-loop/live/loop/cycle-log.jsonl
6bde84d90a3c96c2a8513f2d84bbf83f9a47f996b3da8041dbad9dadaeb27b18  openbell/live/heartbeat.json (23:08:40 ET read)
c924d3fd7bf5433829058475b0a4937e5ee88bd978de10c49968376aba315eff  openbell/live/exec/exec_config.json   (rails)
9b82d26815b7720d0d7ec017270c3698bc50c07f158fe6d0622f5a8f3fd3c45b  openbell/live/exec/deny.json
f1696ea31a72afbe0cdbf6dec8edb0a98a6b7215b1f2279136cbd814ae4e3b27  openbell/live/exec/rails.py
```

Kalshi shadow (liveness read at the same minute): poll `errors_1h 0`, `gaps_today 0`, `req_rate_60s 1.667`,
`req_total 487,958`; shadow `decisions_today 1944`, `errors_1h 0`, chain heads decisions seq 40,986 / orders seq 273,989;
`heartbeat-poll.json` sha256 `1ccf83ff…26b109`, `heartbeat-shadow.json` sha256 `6d545a79…5ee3a8`. Paper only; no orders.

## 4. Open item recorded, not hidden

`exec/state/flat_proof.json` at 23:09 ET reads `{"source":"panel","positive":false,"statement":"not flat or unreadable"}`.
The engine heartbeat reports `pos: null` (no position) and the day state reports no open order, so the desk is flat by
both trading records; the panel probe could not confirm it. Recorded as an **unproven flat at the seal read**, not as
flat. It does not affect the day's P&L (zero) or the rails.

## 5. Wednesday 4 November — what liveness requires (nothing armed by this record)

1. A **fresh day enable** (`go_id` + expiry) from Nox/the owner — today's `enabled.json` expired at 17:00:00 ET and the
   executor halted at 17:00:01 `leg_failed:armed`. There is **no dry-run net** under it (§7): a valid enable arms a real
   broker path through the Tradovate page.
2. A **fresh five-leg attestation** for the new day, then a **different-seat inspect** (the 3 Nov record shows the shape:
   attest by one seat, inspect by another).
3. The **R11 price-limit record** written inside 09:00–09:15 ET (today's was the desk-bar stopgap, `state/r11/…`).
4. No unit restart, no config change, no rail change by any agent until those three exist.
5. The hourly loop needs nothing: `openbell-futures-loop.timer` is active and fires at :07 each hour; the dead-man
   check (`--status`, exit 3 when the newest cycle is older than 70 min) is the liveness test.

## 6. What this seal does NOT claim

- Not a traded day, not a desk day, not a runbook day.
- Not a strategy verdict: 0 fills under `orb15-v1` is below the 20-fill judging bar, and today is `FEED_SILENCE`
  (excluded from the streak, never a parameter change).
- No rail, config, deny-list, credential or unit was touched by this stream. Every read above is read-only.

## 7. Correction, filed 2026-11-03 23:58 ET (same stream, after a second read)

The first filing of this seal said the executor was `mode dry-run`. **That was wrong, and it was wrong in the
direction that understates risk.** Corrected here, with the original wording left in place above so the error is
visible rather than quietly fixed.

**What was read the first time.** `/home/cayleb/Work/projects/openbell/live/exec/exec_config.json` — `mode: dry-run`,
sha256 `c924d3fd7bf5433829058475b0a4937e5ee88bd978de10c49968376aba315eff`, the mock CDP ticket map, no `venue` key.

**What the running process actually reads.** `python3 -m live.exec_run run --live-root …/openbell/live`, working
directory `~/Work/projects/openbell-executor`, and `live/exec_run.py:24` loads `PKG / "exec_config.json"` — i.e.
`/home/cayleb/Work/projects/openbell-executor/live/exec/exec_config.json`, sha256
`4e6969d401d65bcf43402d3a21c20ac1689a00d9af9cf911c56e0366f43cc7a5`, **`mode: live`, `venue: tradovate`**, with the
`cdp_tradovate.*` selectors pinned from the 2 Nov captures. `--live-root` supplies only the desk config
(`<live-root>/config.json`) and the price-limit sidecar; it does not supply the executor's mode.

**How the two copies differ, exactly.** Flattened key comparison: the only value that differs is `mode`
(`dry-run` / `live`); the executor copy is a strict superset adding `venue`, the whole `cdp_tradovate.*` selector map,
`account_name`, `loop.*`, the `paths.*` state names, `holidays_et`, the `attest.*` thresholds, `signal.*` and
`gap_pct_max`. **Every rail value is identical in both copies** — daily loss -400, profit +900, 2 trades/day, 15-min
cooldown, flat 16:05, min stop 2 pt, entry window 09:46–12:00, phase challenge — so the rails claim in §1 stands. What
the §3 digest identified was the desk copy, not the running one; both digests are now recorded here.

**`mode` is not an execution switch.** No code path reads `cfg["mode"]` to pick a recorder instead of a venue. It is
read in exactly one place: `attest.execution_mode` (`live/exec/attest.py:93`), where `mode != "live"` **fails** the
attestation leg (`exec_config_mode_not_live`). So setting `dry-run` would not have stopped a send — it would only have
made the day unattestable. The venue chosen is `TradovateVenue` because `cfg["venue"] == "tradovate"`
(`exec_run.py:31`), a real CDP transport to the broker page on port 9222; the `RecordingVenue` stub is used only by
the tests and the G2 dry-run harness.

**What actually kept orders off the broker today** was the gate chain, not a mode:

| gate | state at 23:57 ET | effect |
|---|---|---|
| enable artefact | `enabled: true`, `expires_et 2026-11-03T17:00:00-05:00` | **expired** — `load_enable_artefact` refuses |
| day state | `halt {reason: leg_failed:armed}` at 17:00:01 | no new entries by design |
| attestation | five legs PASS, 11:12:25 ET, different-seat inspect `d4544c47…` | valid for 3 Nov only |
| trades / placed | `0` / `[]` | nothing was sent |

**Consequence for Wednesday, stated plainly.** Re-enabling the desk for 4 Nov arms a **real** broker path; there is no
dry-run layer between a valid enable + passing attestation and the Tradovate ticket. The only honest reason nothing
traded today is the expired enable and the halt. Anyone re-enabling should read it in those terms.

**Also flagged, not edited.** (a) `live/exec/__init__.py` still describes the package as "disabled by construction:
mode dry-run, `state/enabled.json` absent by default, no venue that can send unless a CDP transport is injected" —
stale for this deployment on all three counts. Docstring correction is a code change and waits for Nox. (b) Two
`exec_config.json` copies with different modes exist in two trees; a unit started without `--live-root` pointing at
the desk, or a config copied between trees, would silently change which file the attestation grades. Worth a single
canonical config path.

## 8. Wednesday's first cycle, verified 00:23 ET (4 Nov)

The liveness claim in §5.5 is now tested rather than asserted — the first cycle of the new day ran on schedule:

```
cd ~/Work/projects/openbell-futures-loop && python3 -m live.loop.cycle --status
FRESH: newest cycle 2026-11-04T00:07:00-05:00 (16 min ago) kind=cycle decision=HOLD; chain ok   (exit 0)
```

- `openbell-futures-loop.timer` fired at **00:07:00 ET**; the oneshot service finished clean (`ActiveState=inactive`,
  `Result=success`), consuming 2.6 s CPU.
- New log row: `cycle_at 2026-11-04T00:07:00-05:00`, `day 2026-11-04`, `decision HOLD`, `cause IN_PROGRESS`,
  `calibration_ok true` — `IN_PROGRESS` is Wednesday's session not having closed yet, i.e. no data to judge, not a fault.
- Chain still `ok` end to end; the dead-man check (`--status`, exit 3 when the newest cycle is older than 70 min) is
  the liveness test the 09:00–09:15 R11 window will rely on.
- Executor unchanged and alive (`active (running)`, 12 h): `mode: live`, `venue: tradovate`, enable expired 17:00,
  day halted `leg_failed:armed`, `trades 0`, `placed []`. Nothing armed by this record.

So: **the loop is live for Wednesday and needs nothing from anyone. The executor needs the four acts in §5 before it
can trade, and trading it will be real money (§7).**

## 9. Day-handover check, 00:45 ET (4 Nov) — the halt does not carry over

Re-read after midnight because §8's executor line described the 3 Nov day file. The 4 Nov day file was created by the
executor's own tick at **00:42:03 ET**, and it is clean:

```
live/exec/state/day-2026-11-04.json  {"halt": null, "trades": 0, "placed": [], "open": null, "pending": null}
live/exec/state/day-2026-11-03.json  {"halt": {"reason": "leg_failed:armed", "t": 1793743201.29}, ...}
```

So the `leg_failed:armed` halt that stopped 3 Nov is **gone**, not carried: yesterday's protection was the expired
enable plus that halt, and today only the enable remains. `enabled.json` still carries `expires_utc 1793743200`
(2026-11-03T17:00 ET), and `live/exec/guard.py:load_enable_artefact` refuses on `disabled:expired` before any other
gate, so the desk is still safe — but it is now **one leg from armed, not two**. Worth stating plainly before 09:00:
refreshing that file without a fresh attestation and a different-seat inspect would arm a live-venue executor whose
last five-leg attestation was 3 Nov.

Also observed, same check: `flat_proof.json` still reads `positive:false, "not flat or unreadable"` (panel probe,
00:42), and the executor's own attestation note at 00:20:00 ET was `obs:broker,armed — Not armed: broker, armed`. So
today's blockers as of 00:45 ET are: **enable expired** (hard), **broker ticket page not ready** (attestation leg),
**armed/not-flat** (attestation leg). `openbell-live` engine, `openbell-executor` and `openbell-dash` are all
`active (running)`; `:8791` still serves the OLD dashboard.

Second consecutive cycle, 01:08 ET: `FRESH: newest cycle 2026-11-04T01:07:00-05:00 (1 min ago) kind=cycle
decision=HOLD; chain ok` (exit 0). Two Wednesday cycles on schedule, chain intact — the timer and the dead-man check are
both doing what §5.5 says, and the loop needs nothing from anyone before the 09:00–09:15 R11 window.

## 9. Day rollover correction, 01:15 ET — the halt did not carry into 4 November

§7 says "what kept orders off the broker today was the gate chain — expired enable, halt `leg_failed:armed`". True of
3 Nov. It is **not** true of 4 Nov: the engine's first tick of the new day wrote `day-2026-11-04.json` at 00:42:03 with
`halt: null, trades 0, placed []`. The halt is per-day and cleared with the date.

So Wednesday's chain is narrower than the day I sealed:

| gate | 3 Nov | 4 Nov (now) |
|---|---|---|
| enable artefact | expired at 17:00 after the halt | **expired** (`expires_et 2026-11-03T17:00:00`, `guard.py:load_enable_artefact` → `disabled:expired`) |
| intraday halt | `leg_failed:armed` | **null — gone** |
| attestation | 3 Nov record, inspected | **day-scoped**: `liveloop.py:232` calls `attest.usable(path, self.day, pkg_sha)`, so a 4 Nov record is required |

Nothing is armed; `trades 0`, `placed []`. But the margin is two procedural gates (an unexpired enable *and* a fresh
inspected attestation) rather than three, and both are things a human writes. Nobody should read yesterday's halt as a
standing protection. The four acts in §5 are the whole of Wednesday's safety.

## 10. R11 is a 15-minute morning act with a working tool — appended 04 Nov 01:40 ET

§5 lists "the R11 record inside 09:00–09:15 ET" as one of Wednesday's four acts, and `live/exec/ARMING.md` §0 says
*"No generator exists in the tree"*. **That sentence is stale** and I checked it instead of repeating it:

- `live/exec_r11.py` exists — commit `a111119`, *"exec: R11 record builder (schema 2, matches the design's validation
  numbers)"*, written 05:52 ET on 3 Nov, eighteen minutes **after** `ARMING.md` (05:34). So the runbook predates its
  own tool.
- It builds a schema-2 record in one command, and the 09:00–09:15 constraint is the **rails'**, not the tool's: the
  record must be *read* inside that window, so the command runs there, not earlier.
- Proof, run in memory at 01:38 ET (nothing written to `state/r11/`): a 4 Nov record built for `MESZ6` at a simulated
  `read_at` of 09:05 ET returns **no reasons** from `rails._r11_reasons` at 09:06 and at 11:00 — the gate passes. The
  same builder with a `read_at` of 10:30 ET returns `['price_limit:read_outside_guard']` — the gate refuses. So the
  generator's output is exactly what the rail wants, and the window rule bites in both directions.
- The exchange-published CME fixing is the design's reference (preferred). The bar-derived stopgap
  (`--from-bars --accept-stopgap`) remains the Nox-ruled fallback and is labelled in `ref_source`.

Consequence for Wednesday: R11 moves from "a component nobody has" to a 15-minute act — read the ES December fixing,
run the builder inside the window, then attest. The other three acts in §5 are unchanged, and nothing is armed tonight.

`live/exec/ARMING.md` §0 now carries this correction inline (documentation only; `exec_config.json` was already modified
before this stream arrived and was **not** touched — it is the running executor's own config).

---

## §11 — Wednesday pre-flight tool, and two arm acts that were already done (04 Nov, 01:40 ET)

**New, read-only artefact:** `~/fleet-scratch/overnight/wednesday/preflight.py` — one command for the desk at 09:00:
`python3 ~/fleet-scratch/overnight/wednesday/preflight.py [--day YYYY-MM-DD] [--json]`. It calls the executor's own
`guard.load_enable_artefact`, `guard.flat_proof_ok`, `attest.usable` and `rails._r11_reasons` (no re-implementation, no
writes, no broker connection, no price-limit record created) and prints one line per arming act with PASS / FAIL / TODO
and the executor's own refusal string. Exit 0 only when every hard gate is PASS.

Run against today (04 Nov, 01:36 ET) it returns, independently of this record's prose:

| act | state | what it reports |
|---|---|---|
| 1 enable | **FAIL** | `guard refused: disabled:expired` |
| 2 day state | PASS | `halt null, trades 0, placed 0` |
| 3 attestation | **TODO** | `attestation_missing` for 2026-11-04 |
| 4 flat proof | **FAIL** | not positive |
| 5 R11 record | **TODO** | `state/r11/cme_limits-2026-11-04.json` missing (must be read 09:00–09:15 ET) |
| 6 second seat | **TODO** | today's attestation names no inspecting seat |

Run against 3 Nov it is not all-red, which is the point of quoting it: attestation **PASS** (`usable for 2026-11-03`),
R11 **PASS** (`SET, read_at 2026-11-03T14:02:00Z inside 09:00–09:15, price outside the refuse bands`), day state **FAIL**
with the historical `halt leg_failed:armed`. So the tool discriminates rather than printing one verdict.

**Two acts in `live/exec/ARMING.md` were already complete and the runbook still lists them as work:** §2 desk config is
`"mode": "automated"` (heartbeat agrees) and §4 executor config is `"mode": "live"` (`4e6969d4…3cc7a5`, the file the
running process reads). Following §2/§4 literally would re-edit a digest-bound config, invalidating
`state/static_proof.json` and forcing a fresh proof plus re-attestation, for no gain. Corrected inline in `ARMING.md`
(append-only block, documentation only; no config value was changed by this stream). Wednesday is four acts: enable,
attest, second-seat inspect, R11 inside its window — plus a flat proof the panel does not currently give.

## §12 — first cycle of 4 Nov verified on schedule (02:09 ET)

`openbell-futures-loop.timer` fired at **02:07:00 ET**; the oneshot unit exited `status=0/SUCCESS`; `--status` reports
`FRESH: newest cycle 2026-11-04T02:07:00-05:00 (2 min ago) kind=cycle decision=HOLD; chain ok`; next fire 03:07.
`openbell-live`, `openbell-executor` and `openbell-dash` all `active`. Day state for 2026-11-04 remains
`halt: null, trades 0, placed []`, `enabled.json` remains `disabled:expired`, `flat_proof` remains not positive.
Two consecutive hourly cycles have now run clean on the new day (01:07 and 02:07), which is the whole of what the loop
can prove by itself: **the loop is alive; the executor is not armed, and nothing in this stream armed it.**

## 11. Correction, 4 Nov 02:30 ET — the live desk has been in `mode: automated` since yesterday morning, not `cards`

Found by reading the live snapshot instead of trusting the runbook: `~/Work/projects/openbell/live/config.json`
(mtime **2026-11-03 07:40**) carries `"mode": "automated"`, and the engine (`openbell-live.service`, started 09:28:59 on
3 Nov) reports `mode: automated` in both its live snapshot and `live/heartbeat.json` (6,620 bars, `pos: null`,
`day_pnl 0.0`, `market_open: true`).

`live/exec/ARMING.md` §0 preconditions and §2 still say *"Desk config: `mode: cards` -> `automated`"* and *"nothing below
has been done"*. **That is stale for the mode.** Consequences, in order of importance:

- The engine sends **no tap cards** in this mode; the executor is the only order path. Anyone expecting cards, or
  "fixing" the desk by flipping it back to `cards`, is changing the safety story without changing the gate story.
- Wednesday is therefore **one gate away**, not two steps away: R11 record, enable, attestation + inspect. Nothing else
  stands between the desk and a real broker path (§7 already says a valid enable arms a real path).
- Today's protection is unchanged and was re-verified at 02:30 ET: `enabled.json` is the 09:29/3 Nov artefact and
  `guard.load_enable_artefact` raises `disabled:expired`; `day-2026-11-04.json` is `halt: null, trades 0, placed []`;
  `intents-2026-11-03.jsonl` is the newest intent file, so **no entry has been attempted today at all**.

This corrects this stream's own earlier summary, which said the desk ran "mode cards". It never did tonight; it has been
automated since 3 Nov 07:40 and has placed nothing, because the executor's gates — not the engine mode — are what stop
it. R11 (see §10) is the one gate that must be created inside 09:00–09:15 ET.

## §13 — Slice B completion, 4 Nov 07:50 ET (openbell2 mission, verification append)

This section completes Slice B (the deferred futures sealed-record slice from openbell-progress.md) as authorised accounting work. §§1-12 are untouched. What follows is a fresh verify-against-disk, not a re-write.

**3 Nov final numbers (re-verified 4 Nov):**
- Days cells: engine 0 trades, day_pnl 0.0; executor trades 0, placed []; attestation 3 Nov attested 11:12:25 ET PASS five legs, different-seat inspect `inspector-seat-claude-sonnet` digest `d4544c47…b41372`; enable 09:29:20→17:00:00 ET, halt 17:00:01 `leg_failed:armed`; rails -$400/+$900/2 trades/15-min cooldown/flat 16:05 unchanged.
- Owner manual +$445.28 (3 ES contracts 07:20:08→07:22:11 ET) remains owner P&L, never engine P&L.
- Cycle chain head for 3 Nov `record_hash f0f5395465ccad871bf0bf154ab338c475b60148363c86a7ea57bb9631a21335`, `prev_hash 8eaa0d69844885bbec5e02b6da455ad5b9017b8e25aaaca04a969436c845d986`; chain `ok`.

**FEED_SILENCE cause (the 3 Nov zero-trade explanation):**
- Engine up 07:40:53, feed OK 07:45, then tabkeeper log: browser_down 08:43:43, tradovate tab opened 08:44:13 (state missing), Tradovate logged out card 08:44:43, recovery 11:11:45. Gap 08:43→11:12 ≈ 2h29m.
- Opening range window 09:30–09:45 (570–585): 0/15 live bars. Entry window to 12:00: 48/135 live bars. 2 Nov had full 15/15 + 135/135 by contrast.
- `live/loop/funnel.py` (COVER_MIN 0.80) labels this FEED_SILENCE at lines 142/146 — the classification is CORRECT. It is not a strategy indictment (strategy_caused false) and not an executor refusal. Zero trades is fully explained by the feed gap; no signal could form (trigger null, range null).

**§7 correction stands:** the running executor is `mode: live` / `venue: tradovate` (its own `exec_config.json` 4e6969d4…), not the desk copy's dry-run. What kept orders off the broker was the gate chain (expired enable + halt), not a dry-run net. Both digests stand as recorded in §7.

**4 Nov re-verification (07:48 ET):** day rolled clean at 00:42:03 (`day-2026-11-04.json` halt null, trades 0, placed []); flat_proof turned POSITIVE (panel "no positions" 07:48) after being `positive:false` at every prior read; enabled.json still expired 3 Nov 17:00; no 4 Nov attestation/R11 yet; bars 469 tv-ws contiguous 00:00→07:47 (no gaps >5m measured), heartbeat feed ws age 0.1, bars 6942, last 7663.0, pos null, day_pnl 0.0. Loop 07:07 IN_PROGRESS chain ok. Preflight 07:48: 1 FAIL expired, 2 PASS, 3 TODO, 4 PASS, 5 TODO, 6 TODO.

**Slice B verdict:** the 3 Nov sealed record is complete and accurate for its day. No numbers needed correction beyond §7's already-filed correction. Today's desk is live but unarmed pending the booked 09:05/09:15 acts (owner/Nox writes).

## §14 — Slice B closed: what actually happened on 4 Nov, and why the desk traded nothing (openbell2 mission, 4 Nov 12:30 ET)

§§1–13 are untouched. This is a fresh verify-against-disk append. §13 said "today's desk is live but
unarmed pending the booked 09:05/09:15 acts (owner/Nox writes)". **That prediction was falsified by
the afternoon, and the record has to say so.**

**§13's closing sentence was wrong in one respect: the acts were not merely "pending owner/Nox
writes" — they were pending NOBODY.** Measured on disk at 11:20 ET: `state/enabled.json` was still the
3 Nov artefact (`expires_et 2026-11-03T17:00:00-05:00`, `expires_utc 1793743200`); there was **no**
`state/attestation-2026-11-04.json`; `state/r11/` held **only** `cme_limits-2026-11-03.json`. There is
no systemd timer, cron entry or script on this machine that performs the 09:05 guard or the 09:15
preflight — `live/exec_run.py` says it outright: *"Nothing starts it automatically. The owner starts
it."* The booked acts are **manual and were missed.** That, not the feed, is what cost 4 Nov its trades.

**A real signal fired and the executor refused it — correctly.** The engine's own ledger
(`live/ledger/2026-11-04.jsonl`) records: signal 09:47:00 px 7734.25 (orb15_L, stop 15.0 / target 30.0)
→ fill 09:48 px 7734.5 → **exit 09:51 px 7719.25, pts -15.25, usd -78.75, why "stop", mae 16.5,
mfe 0.0, 4 bars**. The executor's chain (`state/intents-2026-11-04.jsonl` seq 0, `intent_id
ob-ff59e358f4a1bffe`, side long qty 1 MESZ6, `decision_ref sig-1793803620119`) recorded
**`allow: false`, `reasons ["observation_only:armed","price_limit:missing","illiquid_or_gap","third_party","disabled"]`**.
`state/day-2026-11-04.json` stayed `halt null, trades 0, placed []`, and the broker confirms it:
`cashBalance` `realizedPnL 0.0`, `amount 50445.28` (SOD identical), `position netPos 0` — **flat, no
fill, no order on the broker.** The -$78.75 is the **engine's shadow book**, never broker P&L.

**The refusal decomposes into THREE rail causes, each measured by calling the desk's own code**
(not read from a document), and 3 of the 5 listed reasons are one root cause:
1. `disabled:expired` — `guard.load_enable_artefact(state/enabled.json, now)` raises it. This ONE
   failure cascades: `attest.armed` leg fails → `bad=["armed"]` → `observe_why=["armed"]` →
   `observation_only:armed`; and `_rail_state` sets `enabled = static_proof_ok = not armed.reasons`
   → `third_party` + `disabled`. **Three of five reasons, one cause.**
2. `price_limit:missing` — no 4 Nov R11 sidecar. `exec_config.r11.guard_window_et` is
   `["09:00","09:15"]` and `rails._r11_reasons` requires the record's `read_at` to fall inside it on
   the trade day; `exec_r11.py` stamps `read_at` = the moment it runs, so a record built after 09:15
   yields `price_limit:read_outside_guard`. **The only way to make a late record "valid" is to
   back-date `read_at`, which falsifies a safety record. Not done.**
3. `illiquid_or_gap` — `exec_io.gap_state("2026-11-04")` returns **`"gapped"`** on the desk's own code:
   previous regular-session close **7596.50** (3 Nov 15:59 ET) vs today's first 09:30 bar open
   **7683.75** = **+1.1486%**, over `rails.gap_pct_max 1.0`. R12 fails closed on anything but a
   positively `normal` gap, so **even a fully armed desk would have refused every entry today.**
   (`gap_state("2026-11-03")` = `"normal"` — the comparison is a control, not an artefact.)

**The 3 Nov FEED_SILENCE cause did NOT recur.** Feed healthy all day: 675 bars, contiguous, ws age
0.0, last 7712.0, `market_open true`; tabkeeper `tradovate: ok`, `session: ok`, `lane_logout false`;
sessionwatch `state ok`. The 3 Nov cause (tab/browser_down 08:43→11:12) stays exactly as §13 records
it and is now fully separated from 4 Nov's cause.

**The 09:05/09:15 acts were the gap, and they are now closed mechanically.** Built
`~/fleet-scratch/overnight/wednesday/arm_session.py`: it performs the booked acts in order using the
executor's OWN functions — build today's R11 record via `live.exec_r11.build` **inside** the
09:00–09:15 ET window, write a fresh `enabled.json` (owner's go id + checker sig + same-day expiry),
run the five-leg attestation via `live.exec_run attest`, then verify with `guard.startup`. It
**deliberately does not perform the fourth act** (the second-seat seal): `attest.inspect` refuses the
maker as checker and the executor's README says *"the real control is who is allowed to run
`inspect`"*, so a script that attested AND sealed would hollow out a deliberate control. Installed as
`openbell-session-guard.service`/`.timer` (trading-lead seat, **09:02 ET** Mon–Fri) and
`openbell-session-seal.service`/`.timer` (lead seat, **09:03 ET**), both **enabled**.

**Proven, not asserted.** `test_arm_session.py` → **16/16** checks in a sandbox; `test_session_units.py`
→ **12/12** checks driving the **real units' own ExecStart lines through `systemd-run`** against a
/tmp copy: R11 written and **accepted by `rails._r11_reasons`**; enable with today's expiry;
attestation **PASS, failed []**; sealed by a different seat; **`attest.usable` → True** against the
executor's package digest `c90a8f3f…`; **`guard.startup` → every gate passed**; and **the rails ALLOW
an in-rails intent** — the step that was impossible on 4 Nov — while refusing the same intent without
the sidecar. Live state verified untouched afterwards (`state/r11/` still holds only the 3 Nov file;
live `enabled.json` mtime still 3 Nov 09:29). **No rail was edited; no order was placed.** The honest
limit, recorded in the unit file itself: both seats run on this one host, so this preserves the
procedural maker≠checker gate and named-seat accountability but is not two independent humans.

**Slice B verdict (final).** The 3 Nov sealed record is complete and accurate for its day; §7's
correction stands. 4 Nov is now sealed as a **zero-trade day whose cause was the missed morning
arming acts plus three independent rails (expired enable, closed R11 window, 1.1486% open gap) — not
feed, not strategy.** Real broker P&L for 4 Nov: **$0.00**. The engine's paper book: **-$78.75** (one
signal, stopped out in 4 bars, mae 16.5 / mfe 0.0) — a loss, recorded as a loss.
