# o15 builder lane → Nox — OPENBELL (t37): what is live, what is proven, what is named

**2026-11-04 13:47 ET (17:47Z).** Movement evidence for the OpenBell stream, with the live reads this lane took. A
fresh-context checker is verifying the four items against the artefacts and will add its verdict as a follow-up filing
and as evidence post 324 on the o15 board. Everything below was read live in this session.

## The live desk state (read at 13:44 ET)

```
$ systemctl --user is-active openbell-dash.service openbell-executor.service openbell-live.service \
    memedesk-desk.service openbell-kalshi-poll.service openbell-kalshi-ref.service openbell-kalshi-shadow.service \
    openbell-tabkeeper.service openbell-sessionwatch.service
active active active active active active active active active
$ curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8791/
200                                  <- the OLD dashboard is still the live one. No cutover was run.
```

**The recurring queues (the "hourly loop"):**

| timer | last | next |
|---|---|---|
| `openbell-futures-loop.timer` | 13:07 ET | **14:07 ET** (hourly cycle) |
| `memedesk-sync.timer` | 13:42:54 ET (52 s before the read) | 13:44:54 ET |
| `openbell-day-end-accounting.timer` | — | **16:25 ET today** (idempotent, rehearsed) |
| `openbell-forward-archive.timer` | 3 Nov 17:20 ET | 17:20 ET today |
| `openbell-session-guard.timer` | — | **Thu 5 Nov 09:02 ET** (arms the executor) |
| `openbell-session-seal.timer` | — | **Thu 5 Nov 09:03 ET** (second seat seals the attestation) |

`openbell-futures-loop.service`, `openbell-session-guard.service` and `openbell-session-seal.service` are **inactive
between their timer fires** — that is the design, not a fault (the timer is what runs them).

## The four items

**(a) Watchover Phase 2 — fixture build + fresh-checker render gate; the old dashboard stays live until you grade.**
The render gate was run by a fresh checker over `renders/20261104T1055Z` → **PASS-WITH-KNOWN-SPEC-CONFLICT** (446 rows,
420/26 = 24 A8 + 2 A29-iii, provenance re-derived independently; files
`~/fleet-scratch/overnight/checker-watchover-verdict-run7.md`, `-run8.md`, `-1055Z.md`). **The old dashboard is the live
one** (8791 → 200 above) and no cutover was run. **The honest gap, stated not upgraded:** the **fixture-build verifier
(A1) was silent in 3 attempts**, so the fixture build is evidenced *standing-by-log*, not by a checker verdict. No
shipped render-set note was edited (corrections go to `RESOLUTION.md`).

**(b) Futures loop + executor liveness for Wednesday, and today's formal seal record.** The loop's timer fired at 13:07
ET and fires again at 14:07; the executor is **active and observation-only** (not armed, no order placed — the broker
account is flat: realizedPnL 0.0, netPos 0, trades 0, placed []). The day's formal seal record is filed:
`board/reports/SEAL-FUTURES-2026-11-03.md` **§14** (SS1-13 untouched), which states what actually happened on 4 Nov plus
the refusal decomposition — and corrects §13's own closing prediction that the acts were "pending owner/Nox writes" (they
were pending nobody). The seal itself is the **second seat's** act (09:03 ET tomorrow; the maker may not check its own
attestation).

**(c) Meme desk 4-hour iterations + the h78 ack chase.** The desk is **running** (`memedesk-desk.service` active; the
sync timer firing every two minutes). **The h78 ack has NOT landed** — the bridge's own status
(`board/rsi/STATUS.json` + `STATUS.md`, written by the ext:rsi courier at **18:23:32Z**):

```
reachable: never-proven (no ack has ever been observed)
last_ack:  null
pending:   h78, age 974 min
route:     board-relay, relay_only: true, wake: none
courier:   green, 1875 polls, last poll 18:23:32Z
```

h78 (nonce `n48b18b472810458e`, created 02:09:30Z) is **delivered** (seq 78, state delivered) and has never been acked;
the RSI board mail carries it and no `ACK` line has been filed. **So `route=chat` cannot honestly be recorded yet** —
the criterion says to record it once RSI's ack lands. The chase continues through the Nox hourly loop. The blocker is
structural: RSI's side is **relay-only** (nothing wakes RSI; the loop carries the message in and files the answer by
hand). The desk's shadow locks are unchanged — `DESK_MODE=shadow`, `SHADOW=1`, no `CONFIRM_LIVE`, no `WALLET_KEYFILE` —
so exactly-ONE live trader holds (RSI's grok-bot-vm).

**(d) Kalshi exact-verdict fix before any first promotion.** The Kalshi desk is alive at **paper fidelity only**:
poll/ref/shadow active since 31 Oct, 468 decisions/hour (whole-minute ticks), 0 errors/hour, and
`exec.json = {enabled:false, environment:demo}` — placement is **off by construction**; `guard.require_live()` measured
→ `GuardRefused('not_enabled')` at check 1 of six; `GO.json` and `verdict.json` are **absent** (they are the owner's/Nox's
artefacts) and no row in the order events carries a venue order id. **So no first promotion can have run before the
fix**, and the fix plus its partial-match rejection test and baseline checker re-run are being verified now.
Two findings recorded earlier and **not** patched: "hourly DECIDE" is not a distinct artefact (468/hr is the whole-minute
rate), and the code defaults `--balance 24.09` while the owner order says $51.39 — arming as-is would size against
$24.09, which is a decision, not a bug to fix silently.

## Your acts (smallest form)

1. **Grade the Watchover render set** (`renders/20261104T1055Z`) — until you do, the old dashboard stays live and the
   cutover does not run.
2. **The two Nox rulings** the pack asks for (A29-iii front scope; A8 viewport vs S5.3 at 900 px) and the **live-trader
   decision** for the meme desk.
3. **Kalshi arming** needs GO.json + verdict.json + the phrase — owner/Nox artefacts.

No trading action, no arming, no rail edit, no hosted row read, no destructive SQL, no credential value printed.

— o15 builder lane (`builder:o15-builder`, order o15, task t37)
