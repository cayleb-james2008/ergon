# TEAM C — BATCH 7 (resumed session) — 2026-11-01T23:34Z–23:50Z (18:34–18:50 ET)

Owner: `trading-lead`, Team C (Tier S throughout). Report to the Lead only. **Spend $0.** No live order, no wallet
operation, no credential value, no public post, no host input injection, no image read. Working board: `board/`.
All evidence is read-only; every command + result + timestamp is quoted below. Unverified is reported unverified.

## 0. Status per item
| Item | Status | One line |
|---|---|---|
| **C1** Monday trading decision input | **DONE (decision input)** | One honest path exists (cards/observation day, human tap only if the ORB15 ruling lands); fully-autonomous cannot land before Monday 09:45. §1 |
| **C2** Fresh tick proof | **DONE — PASS** | First fresh bar exactly 23:00:00Z (18:00:00−05:00) with n=815 tick-prints; 43 bars to 23:42Z; `feed_age_s` 0.1–0.8 s. §2 |
| **C2b** Observation outcome record (Nox chase) | **DONE — filed** | Landable report `board/reports/team-c-globex-open-outcome-20261101.md`; what the open did / what the desk saw / what did NOT happen and why. §2 |
| **C3** Monday runbook final | **DONE — v7, RE-SIGN REQUIRED** | `board/reports/monday-futures-runbook.md` now v7 (sha256 `512752e4…`, 407 lines); v6 signed text preserved byte-identical; P0.1 must be re-signed on v7 before 09:05 ET. §3 |
| **C4** Kalshi read-only check | **DONE** | 3 units active+enabled; `readiness.py` re-run on captured data for both owed self-check days **9 PASS / 1 NA / 0 FAIL, exit 0**; two-scorer AGREE 6,853/0; WP10 167 tests pass; data fresh to 23:39:09Z. §4 |
| **C5** Job 13/15 state (plan only) | **DONE (plan)** | momo machine route = **UNKNOWN** (no machine route; external lane, mail-by-hand); cutover pre-check checklist with what must be true + the one finding that changes the cutover picture. §5 |
| **N5** dependency question | **ANSWERED + RULED** | Desk CDP route = its own Chromium PID 2960519 on :9222; extension-host PID 2960775 is a child with no socket, not a desk dependency. Nox closed N5 233413Z §2. §6 |

---

# 1. C1 — THE MONDAY TRADING QUESTION (decision input; no action taken)

## 1.1 The governing facts, each with its source
1. **The engine has no automated mode.** `live/config.json` `mode="cards"`; the only legal values are `shadow|cards`;
   cards are information for a human, never instructions. Verified tonight: `heartbeat.mode="cards"`,
   `config["mode"]="cards"` (23:43Z). The engine "NEVER places, modifies or cancels orders".
2. **The autonomous executor is dry-run only.** G2 signed with conditions: `live/exec` commit `0893497`,
   digest `32dd2bea4bb0296b4d24f8e595efa8b197cd2bec699bcb6f418b0fc8e13384f9`; 75 tests; `mode: dry-run`; no enable
   artefact, no CDP transport, no order path. G2.5 (CDP/DOM) is a **plan only**; G3 (demo venue) and G4 (pre-arming) are
   **unsigned**. (`board/reports/team-c-g2-outcome-20261101.md`, `team-c-g2p5-plan-20261101.md`,
   `team-c-autonomous-path-design-v4.md` §D.)
3. **The written go exists but cannot be exercised.** Nox 20261101T184800Z §B records the owner's written go (his tap +
   typed confirmation, 1 Nov 13:32 ET) for "the OpenBell futures desk to run its autonomous sessions under his 31 Oct
   automation ruling … tonight's 18:00 ET Globex session is its first application, and it carries forward to Monday's
   session without a re-ask"; condition (ii) was that Team C's Sunday ruling map ORB15 onto the Globex open — **that
   condition failed** (ruling: ORB15 is RTH-anchored, does not map), so "tonight is observation-only and the go's first
   application becomes Monday's session". The same filing says the go "does not retro-sign the gates".
4. **The strategy is RTH-anchored**: `bt/features.py:23-24` RTH 09:30–16:00; `bt/strategies.py:118` class `ORB`
   ("close outside the first N-minute RTH range"), `:133` `if not f.rth: return []`; entry window 09:46–12:00; flat 16:05.
5. **ORB15's own role note says do not trade its cards** — "lost $4.31/trade, PF 0.81, 0 of 9 years positive … do NOT
   trade its cards" — and the Nox/Cayleb ruling on that contradiction is **still open** (delegator-to-Nox 233600Z item 6).
6. **Tonight proves the rails/session/feed restoration holds**: broker attached $50,000 flat, feed live, day rolled at
   18:01 ET (C2). Monday's attestations can therefore pass *if the desk route stays up* — but passing attestations is not
   the same as having an order path.

## 1.2 The concrete paths to a Monday session that trades inside the rails — and where each one stops
| Path | What it is | What it needs | Boundary that stops it |
|---|---|---|---|
| **P1. CARDS DAY, HUMAN TAP** (the only path that can trade Monday) | Engine emits ORB15 cards 09:46–12:00; **a human places the bracket** in the signed-in Tradovate tab inside the rails. Team observes/logs only. | (i) P0.1 signed on runbook v7 before 09:05; (ii) guard + five attestations + (c2) binding all PASS; (iii) **the ORB15 ruling on record** (deliberate beta exception); (iv) a human present/authorised to tap 09:46–12:00. | Stops at any FAIL/UNVERIFIED attestation (no window); stops with **no live position** if the ORB15 ruling is absent (cards logged as observations only); stops for Cayleb on any fresh credential/2FA demand; no tap = no trade. **No autonomous fill is possible or claimed.** |
| **P2. FULLY AUTONOMOUS (G3/G4)** | Contained executor drives the Tradovate ticket via CDP inside the rails. | G2.5 page capture (needs a signed-in tab **outside** 09:05–16:20) → G3 demo package (needs **demo environment + demo access — Cayleb/Nox owner assets**) → G4 (G1–G3 signatures + `go_id` written go + kill-path drill + fresh firm-page re-read), each with a fresh Tier S checker **before** the step. | **Cannot land by Monday 09:45** — G2.5 cannot be captured before the Monday post-16:20 window, and G3's demo assets do not exist. Any shortcut = crossing the gate (forbidden, and would void the design's containment proof). |
| **P3. HUMAN-IN-LOOP EXECUTOR** (semi-automated, executor+human confirm) | A transport-bearing executor with a human confirm per trade. | The CDP transport + guards are exactly what **G2.5→G3** bind; the signed G2 tree is frozen (no transport) and `live/exec` must not be touched after the sign. | **Not available today**: it is new code behind a new gate; my fence for this batch is explicitly "no code change to the engine". Cannot be improvised on a trading morning. |
| **P4. GLOBEX/OVERIGHT SESSION** (trade the 18:00 open) | Re-anchor ORB15 to Sunday/Globex open. | A new preregistration (new anchor, session, sample) + its own frozen forward record + the normal gates. | **Ruled out** (Sunday ruling): ORB15 returns zero signals outside RTH; re-anchoring is a new, unvalidated strategy. No Sunday session. |
| **P5. TEAM TRADES BY HAND / SUBSTITUTE DATA** | A person or the team places discretionary orders, or uses shadow data as if live. | — | **Forbidden**: out of role (the team never trades), breaks the attestation frame, and would be a manufactured claim. Not a path. |

## 1.3 Which path is honest today
**P1 is the only honest Monday path, and only in two layers.** Layer A (always available): **guard → five attestations →
(c2) binding → 09:45 window opened for OBSERVATION → 16:05 flat proof → 16:20 outcome** — a fully evidenced observation
day, `0 fills claimed`, which is a real result (it proves the desk route and decision log work). Layer B (only if the
ORB15 ruling lands and a human is present): a **human-tapped** ORB15 card inside the rails — still not an autonomous
fill, and still needing the ruling. **Anything that claims "the desk trades Monday autonomously" is false on today's
engine**; the written go authorises autonomous operation in principle but cannot be applied to an engine with no
autonomous mode.
**P2 is the path to the owner's actual objective (first autonomous fills) — and it cannot be reached by Monday 09:45.**
Its critical path is: G2.5 capture (Monday ≥16:20) → G3 demo package + demo assets → G4. No honest compression exists.

## 1.4 The Monday decision (one paragraph, for the Lead)
> Monday is a **cards/observation day, not an autonomous session**: the engine has no automated mode (`mode=cards`;
> `shadow|cards` are the only legal values), G2.5/G3/G4 are unsigned and G2 is dry-run-only, so the owner's recorded
> written go (Nox 184800Z §B) cannot be exercised — it authorises autonomous running in principle, not against an engine
> that has no such mode, and it does not retro-sign the gates. The only way Monday can produce a live fill inside the
> rails is a **human-tapped** ORB15 card, and that requires the still-open Nox/Cayleb ruling on ORB15's own "do not trade
> its cards" role note; absent the ruling, any card is logged as an observation and no position is opened. The runbook v7
> now encodes exactly this gate, and the day still cannot start unless a fresh checker signs P0.1 on the v7 text before
> 09:05 ET. What the day can prove honestly: guard, five attestations, post-09:30 binding, window observed, 16:05 flat,
> 16:20 completeness as PASS-empty. What it must not claim: autonomous fills, an order-capable session, or that a missing
> signal is a failure.

## 1.5 What it needs from Nox / Cayleb (smallest acts)
- **Nox (one line, blocks P1 Layer B):** rule the ORB15 contradiction — "run the beta as a deliberate exception (cards are
  live-tradeable by a human)" **or** "hold ORB15: cards observation-only until the record is re-cleared". Until then Monday
  is observation-only by the book.
- **Cayleb (only for Layer B):** be able to place a bracket at the desk 09:46–12:00 if a card fires and the ruling allows
  it — his act, not the team's. (Nothing else is needed from him for the observation day.)
- **Cayleb + Nox (P2, not a Monday item):** name the **demo environment and who holds the demo access** for G3; later, the
  G4 `go_id` written go. No credential crosses the team.
- **Lead:** spawn a fresh Tier S checker to re-sign P0.1 on runbook v7 before 09:05 ET (see §3.2).

---

# 2. C2 — FRESH TICK PROOF: PASS (read-only)

**Primary reading 2026-11-01T23:43:15Z** (18:43:15 ET EST, UTC−5), from `~/Work/projects/openbell/live/`.
Full field-by-field proof: `board/probes/team-c/tick-proof-20261101T2343Z.md`.
Observation outcome: `board/reports/team-c-globex-open-outcome-20261101.md` (the landable filing for Nox 233413Z §5).

- `heartbeat.json`: `t` age **0.16 s**, `et` `2026-11-01T18:43:15-05:00`, `status running`, `mode cards`, `feed "ws"`,
  `feed_age_s` **0.8** (0.1 at 23:38Z), `market_open` **true**, `last_bar_et "18:42"`, `last_price 7641.5`, `bars 4947`,
  `pos null`, `paused false`, `contract_id 4470322`; `lag` median **56.2 ms** / p90 85.1 ms.
- `bars/2026-11-01.jsonl` (44 rows): row 0 = pre-open attach (13:30 ET, n=1); **row 1 = 1793574000 =
  2026-11-01T23:00:00Z = 2026-11-01T18:00:00−05:00**, o 7626.5 / h 7638.75 / l 7626.5 / c 7636.0, **v 3219, n 815**,
  `src tv-ws`; then 43 consecutive one-minute bars to **23:42:00Z (18:42−05:00)**, last close 7642.5. Range since open
  7625.5–7642.5.
- File mtimes: `heartbeat.json` 18:43:15.734 ET; `bars/2026-11-01.jsonl` 18:43:00.601 ET (4,795 B); `account_state.json`
  18:43:12.887 ET (`frame_t` age 1.2 s); `ledger/account-2026-11-02.jsonl` 18:43:06.916 ET.
- Desk day rolled at the open: `logs/engine-2026-10-28.log` → `2026-11-01 18:01:00,109 openbell.engine INFO session day
  2026-11-02: pnl 0.00 trades 0`.
- **What did NOT happen** (the outcome record states all of it): no signal (RTH-anchored: `if not f.rth: return []`), no
  card (`cards_today(2026-11-02)=0`), no decision ledger file (`live/ledger/2026-11-02.jsonl` absent = 0 signals), no
  order/fill/position (`pos null`, `positions {}`, `mode=cards` = notify-only, G2 dry-run), no Sunday session (ruled).
- **Verdict: PASS.** Proven, not inferred; nothing was touched to produce it.

---

# 3. C3 — MONDAY RUNBOOK FINAL: v7 (and the Tier S consequence)

## 3.1 What changed (`board/reports/monday-futures-runbook.md`, v7, 407 lines)
1. **§0 new** — tonight's verified post-open baseline (C2) + four material deltas, with the explicit statement that it
   **does not pre-pass any Monday attestation**.
2. **§0b new** — the **five 09:15 attestations stated individually**: (a) broker session + equity from a verified record;
   (a2) account FLAT by positive surface; (b) engine heartbeat current; (c1) tab+feed alive (provisional) → **(c2)
   binding post-09:30 bar**; (d) engine ARMED + rails + config-to-process binding; (e) execution mode — each with its
   command/evidence source, tonight's baseline and Monday's PASS bar. (Plus decision-log ARMED → PROVEN ON / PASS-empty.)
3. **§0c new** — **owner / backup / fallback**: owner `trading-lead`; backup `lead` (may run read-only attestations, no bar
   relaxed); P0.1 signer fresh `trading-checker`; desktop-route failure → BLOCKED-ON-MACHINE, no improvisation, escalate;
   absent ORB15 ruling → observation-only, 0 fills claimed.
4. **~09:40 mode re-verify** — expected baseline values added (`mode=cards`, running, qty 1, contract 4470322,
   `paused=false`, `pos=null`); "re-verified" must include the quoted (c2) post-09:30 bar, not just the mode line.
5. **09:45 window** — **ORB15 HUMAN-TAP GATE** added: the window opening authorises observation, not a live position; a
   human tap additionally needs the ORB15 ruling on record; absent it, cards are logged as observations and 16:20 records
   OBSERVATION-ONLY / 0 fills.
6. **16:20 item 5** — Friday reconciliation updated: produced as a document; the state after the last verified frame
   (Fri 15:29:12 ET) stays UNVERIFIED until Monday's first fresh `cashBalance` frame.
7. **P0.1 row** — now requires a signoff that **pins the v7 sha256**, explicitly stating that a signoff pinning v6 does
   NOT satisfy P0.1 for v7.
8. **"What this runbook cannot prove"** — two v7 limits added (it cannot authorise a live card; it cannot substitute for
   the P0.1 signature).
**No attestation command was rewritten** — diff of the v6 body vs the v7 body shows only the additions above and the item-5
replacement (`diff` run 2026-11-01T23:47Z).

## 3.2 The Tier S consequence — P0.1 must be re-signed (name it, do not hide it)
- The existing P0.1 signature (`board/probes/team-c/monday-2026-11-02-precheck-signoff.md`, `trading-checker`, 00:12Z) is
  **pinned to the v6 text**: `board/reports/monday-futures-runbook.md`, v6, 320 lines, sha256
  `027ca4e4cc1427e3300e2d36e0eda4380bfb3d8410c32ba8847533347adb4eff`, and it says: *"If the file's sha256 differs at 09:05
  on Monday, this signature is void."* **That v6 hash was verified byte-identical tonight before editing** (2026-11-01T23:45Z,
  `sha256sum` = `027ca4e4…`, 320 lines).
- The **v6 signed text is preserved byte-identical** at `board/probes/team-c/runbook-v6-signed-copy-027ca4e4.md`
  (same `sha256sum` `027ca4e4…`, 320 lines) so the signed artefact remains reproducible; the signed file itself is now v7.
- **v7 final hash: `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`** (407 lines, computed 2026-11-01T23:48Z).
  Any further edit voids it again — this is the last edit for the batch.
- **Required before 09:05 ET Monday: a fresh Tier S checker (veto) reads v7, runs what it can read-only, and writes a
  fresh signoff pinning `512752e4…`** at `board/probes/team-c/monday-2026-11-02-precheck-signoff.md` (or a clearly named
  v7 successor, with the v6 file kept as history). **If no v7 signature exists at 09:05, the gate rule applies: the day
  does not start.** The Lead owns spawning that checker; this is the one thing that must not be forgotten overnight.

## 3.3 The v6 gaps found (quoted, now fixed in v7)
1. *"This file is the pre-check evidence for P0.1: no live step runs until the trading-checker signs P0.1."* — but after
   any edit that signature silently stopped covering the file; v6 had no rule for its own revision. → v7 header + P0.1 row.
2. *"5. **Friday's missing post-close reconciliation (owed).**"* — stale: the reconciliation was produced 1 Nov. → v7 item 5.
3. *"~09:40 — MODE STEP + RE-VERIFY … re-run (b), (c2), (d), (e) and write 're-verified 09:40, mode cards'"* — no baseline
   values and no requirement to quote the (c2) bar. → v7 adds both.
4. The 09:45 window had **no gate between "a card exists" and "a live position exists"** — with ORB15's role note saying
   "do NOT trade its cards", that was the one unsafe ambiguity left. → v7 §09:45 human-tap gate.
5. No single place stated owner/backup/fallback in one sentence (it was spread across BOOKINGS + FAILURE DISPOSITION). → §0c.

---

# 4. C4 — KALSHI SHADOW: READ-ONLY STATE (no order, no gate movement, no flip)

## 4.1 Unit state — enabled vs active (both checked, 2026-11-01T23:34Z)
```
$ systemctl --user is-active openbell-kalshi-poll openbell-kalshi-ref openbell-kalshi-shadow
active
active
active
$ systemctl --user is-enabled openbell-kalshi-poll openbell-kalshi-ref openbell-kalshi-shadow
enabled
enabled
enabled
```
All three are **active (running) and enabled** (survive reboot). The futures units were checked in the same pass:
`openbell-live`, `openbell-dash`, `openbell-tabkeeper` all active+enabled; unit descriptions confirm the shadow has **no
network, no orders** for `-shadow`, and "no orders, no credentials" for `-poll`/`-ref`.

## 4.2 `readiness.py` re-run on captured data — the two owed self-checks (WP9, 10 checks)
Both re-runs are **LATE re-runs** on the captured data (the armed instants are in the past); they are labelled as such and
are not substitutes for an armed instant. Commands run from the lane worktree with `--out`/`--scratch` under
`board/probes/team-c/kalshi-readiness-rerun-20261101T2335Z/` (nothing written into the lane tree).
```
$ python3 -I research/kalshi15/readiness.py --day 2026-10-31 --out … --scratch …
readiness 2026-10-31  read at 2026-11-01T23:35:24Z  observed 2026-10-31T12:33:55Z..2026-11-01T00:00:00Z
PASS capture_validator  5 files, 3357672 records (…trade=3309005), errors 0
PASS gap_ledger         unhealthy-book share worst 0.029% (HYPE); gap records 6 (book 63s trades 37s); starts 3 restarts 2 crashes 0
PASS ref_health         worst 99.975% (BNB); venue connects 34 disconnects 27 errors 20; clock -2.678..2.448 ms stratum [2]
PASS settlement_reconcile 486/486 result-vs-rule, 414/414 shadow-vs-result, mismatches 0
PASS proxy_wrong_side   (report only; per-coin wrong-side 1/44–3/45)
PASS replay_determinism byte-identical over the common prefix: decisions 16390, orders 121991 lines (177.3 s)
PASS chain_integrity    decisions 16417 lines head df21488b774447f7; orders 122097 lines head 427d4a9bcc4b8981
NA   nox_reconcile      no time overlap with Nox's clean cycles (by construction)
PASS counts             healthy settled windows 134–135/coin; ~20.0–20.2 days to the 2,000 bar
PASS heartbeats         poll=0.6s ref=0.5s shadow=0.4s; errors_1h poll=0 shadow=0
EXIT=0
```
```
$ … readiness.py --day 2026-11-01 …
readiness 2026-11-01  read at 2026-11-01T23:39:09Z  observed 2026-11-01T00:00:00Z..2026-11-01T23:39:09Z
PASS capture_validator  5 files, 5734940 records (book=80987 fees=13 gap=32 market=19426 schedule=1 settled=855 status=284 trade=5633342), errors 0
PASS gap_ledger         worst unhealthy-book share 0.106% (XRP); gap records 32; starts 0 restarts 0 crashes 0; longest gap 60s
PASS ref_health         worst 100.000%; venue connects 3 disconnects 3 errors 3; clock -23.961..41.934 ms stratum [2]
PASS settlement_reconcile 846/846 result-vs-rule, 846/846 shadow-vs-result, mismatches 0, unsettled 0
PASS proxy_wrong_side   per-coin wrong-side 0/91–3/91 (report only)
PASS replay_determinism byte-identical over the common prefix: decisions 16417, orders 122097 lines (175.5 s)
PASS chain_integrity    decisions 16444 lines head a1ad8d023f2dd024; orders 122097 lines head 427d4a9bcc4b8981
NA   nox_reconcile      no Nox file for the day (by construction)
PASS counts             healthy settled windows 135–136/coin; ~20.0–20.1 days to 2,000
PASS heartbeats         poll=0.4s ref=1.5s shadow=0.9s; errors_1h poll=0 shadow=0
EXIT=0
```
**Both: 9 PASS / 1 NA / 0 FAIL, exit 0.** One observation, not a failure: both replays produce **+9 decision lines** over
the live ledger on the common prefix (16,399 vs 16,390; 16,426 vs 16,417). Readiness passes on the common prefix, and the
same +9 shape was recorded in the DST adjudication §8.4 as a pre-existing observation to be looked at on its own.

## 4.3 The two owed self-checks — what they were, and their status now
- **20:07 ET rollover check (00:07Z)**: the armed run fired 2026-11-01 00:06:34–00:10:02Z and captured the before/after
  artefacts; its criterion-3 summary printed `HEAD_NOT_ADVANCED_OR_UNREADABLE` because of a **parse defect in the script**
  (it tried `json.load` on a multi-line snapshot), not a ledger problem. Corrected basis from its own readiness logs:
  decisions 5,401 lines/head `135dd8d9…` → 5,419/`a3fa6b75…` → 5,428/`019a1499…`; orders unchanged 39,655/`97eb89fd…`
  (no new order events in the window); **no count went backwards, chain_integrity PASS at both instants**. The fix was
  applied to the DST copy only; **the armed rollover artefacts were not rewritten**. Tonight's re-run of that day PASSES
  (§4.2, first block) — the seam itself is healthy; the recorded defect is procedural and is reported as such.
- **03:12 ET DST check**: **CLOSED** as **INCOMPLETE, independently signed** (`board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md`;
  checker verdict `board/reports/team-c-dst-checker-verdict.md`, 14:20Z, PASS on the adjudication): the 05:50Z leg ran and
  PASSED; the 07:15Z armed leg **never ran** because the DSH app instance owning its subprocess scope was torn down 26m40s
  before the instant (journal-quoted "on client request"; the caller is UNVERIFIED). The capture stack itself crossed the
  fallback cleanly (no restart/crash; books continuous; EST −0500 after). The 07:15Z instant is **unrecoverable** and is
  recorded as such; the late re-run proves the system healthy afterwards, not the missing instant. Tonight's second re-run
  PASSES. **Carry to Monday's lane:** any armed run launched from a DSH tool call dies with the app instance — use a
  `systemd-run --user --transient` unit + watchdog for future armed instants.

## 4.4 Two-scorer agreement (figures confirmed from the file, 2026-11-01T23:44Z)
`research/kalshi15/results/scorer_agreement.json` (`date_utc 2026-10-31T17:39:32Z`):
**`AGREE: 6853 rows each, 0 differences; summaries and the late side file identical (tol 1e-9, integers exact); six
latency rows {2,5,10,30,60,90} (2 and 5 informational)`**; history: 6788 rows (pre-amendment 2) → 6853 (post-amendment 2,
11:40 ET) → 6853 (six latency rows + late side file, after Nox Amendment 1, 13:45 ET); pins `X_milli 100`,
`done_lines 2560`, `theta [0.03,0.06,0.1,0.15]`, `L_star 60` (the L\*=30s proposal is still FOR RULING, not applied).

## 4.5 WP9 / WP10 status
- **WP9 readiness**: built and exercised; first run 9 PASS/1 NA; tonight's two re-runs independently reproduce 9 PASS/1 NA/
  0 FAIL on today's captured data (`§4.2`). The one NA is structural (`nox_reconcile` — no overlap with Nox's clean
  cycles; his copy's push stalled 31 Oct, so check 8 stays NA by construction, not by failure).
- **WP10 live path**: present and **shipped OFF by construction**: `live/kalshi_exec/exec.json` =
  `{"enabled": false, "environment": "demo", "role": "desktop"}`; the single choke point `guard.require_live()` fails
  closed on **six** conditions (enabled + matching scope hash, exact phrase env, unexpired GO.json with scope hash,
  verdict.json with a passed gate, no STOP file, demo unless both files say prod). **Ran the package suite now:**
  `PYTHONDONTWRITEBYTECODE=1 python3 -m pytest -q live/kalshi_exec -p no:cacheprovider` → **`167 passed in 1.08s`, exit 0**
  (2026-11-01T23:44Z). No gate movement: the gate is the frozen forward record (freeze target Tue 3–Wed 4 Nov).

## 4.6 Data freshness (2026-11-01T23:42–23:44Z)
- `live/kalshi/capture/2026-11-01/`: `books.jsonl` 325,838,623 B mtime 18:42:54 ET; `trades.jsonl` 2,528,842,976 B
  18:42:51 ET; `markets.jsonl` 45,130,167 B 18:42:47 ET; `events.jsonl` 18:38:49 ET; `gaps.jsonl` 01:30:53 ET (gaps only
  accrue when they happen).
- `live/kalshi/ref/2026-11-01/`: per-coin files mtime 18:42:54.999 ET (all nine coins).
- Heartbeats: poll **0.1 s**, ref **1.7 s**, shadow **0.9 s** old. readiness read cutoffs: capture books 23:39:09Z,
  ref 23:39:10Z, ledger tick 23:38:00Z.
- Nox's collector copy remains stale (last push 31 Oct, per lane RESUME) → check 8 NA, stated not passed.

## 4.7 What this section does NOT do
No order, no flip, no enable flag, no threshold/parameter change, no gate movement, no key/token read, no write into the
lane tree. `$51.39` untouched; the live path stays `enabled:false`.

---

# 5. C5 — JOB 13 / JOB 15 STATE (plan only; nothing executed)

## 5.1 Job 13 (momo v5 machine route): **route = UNKNOWN**
What was tried tonight (all read-only):
1. `~/.ssh/config` → **no `Host` entries** (no configured remote).
2. `tailscale status --json` → self `outis-desktop`; peers: `Cayleb-Desk` (windows, **offline** 255 d), `outis` (windows,
   **offline** 152 d), `Nothing Phone` (online), `muse` (online), `srv1360751` (offline 255 d). **No Grok/RSI desk node.**
3. `mount` → no network filesystems (nfs/cifs/sshfs/9p absent).
4. Nox's board `ops/LANES.json` → `{"id":"grok","name":"Grok / RSI desk","kind":"external"}`; the file's own note:
   *"external (no machine route: mail to it is handed to Nox)"*. **This is the authoritative answer: there is no machine
   route by design.**
5. Desktop host layer check (`~/Work/projects/memedesk-host`): `memedesk.target` active, `memedesk-sync.timer` active, but
   `memedesk-desk.service` **failed** (`desk-start: refused: settings NOT READY`, `JUP_API_KEY empty → FAIL`, exit 78).
   Its own RUNBOOK: *"The desktop runs no live trader. The Grok trader is the only live one."* → the desktop tree is a
   **shadow/code copy, not a view of the live desk**; archive of `~/Work/projects/openbell-meme-desk/data/` (ledger.jsonl
   3.9 MB, live/ledger.jsonl 3.3 MB) is a **31 Oct-synced copy**, not live data — must not be used as ground truth.
**Could not read** (state plainly): the desk machine's `data/status.json`, `data/strategy/live.log`, `data/live/ledger.jsonl`,
deployed `exit_rule.json`, the live trader's environment and process table. Nothing was reconstructed from the chat thread;
the unverified message still sitting in that thread is treated as data, never as an order.
**Route therefore = the hand-carried board-mail path only** (draft request filed at `board/reports/team-c-desk-data-request.md`,
NOT sent; Lead→Nox→desk by hand). Job 13 stays BLOCKED-ON-MACHINE until Nox relays that answer.

## 5.2 Job 15 (OpenBell cutover) — the checklist of what must be true before any cutover (nothing executed)
Pre-cutover source of truth: Nox's `openbell-cutover-prechecks-2026-10-31.md` (desk-state half) + repo `docs/CUTOVER.md`
(repo-hygiene half). Status below is **verified here** vs **must be done at the desk** (unverifiable from this seat).
| # | Must be true | Status tonight |
|---|---|---|
| 1 | **Cutover target = merged master `6f582f8`** | **VERIFIED in the lane clone**: `git log --oneline -1 github/master` → `6f582f8 Merge pull request #3 from cayleb-james2008/stack/3-review` (`~/Work/projects/openbell-kalshi`, remote-tracking ref; not re-fetched tonight). **Finding (material):** the LIVE engine's checkout `~/Work/projects/openbell` is on branch `research/kronos-stage1-verdict` @ `0893497` with modified files, its local `master` is `fd82bb1` and it has **no remote** and no `6f582f8` object — i.e. **the deployed tree is not the cutover target**; the cutover means merging/checking out `6f582f8` on the live checkout, with the G2 `live/exec` tree still frozen. |
| 2 | **Deployed exit-rule file byte-checked + startup-log line** | **UNVERIFIABLE from here** (desk machine). Repo facts verified: merged master carries the seed `meme/desk/seed/live/exit_rule.json`, **sha256 `ff19993f4b6b56cf73993c408148eae8e1260de2a9d00bc8838394090f0572cd`** = variant B, `maxHoldH 0.5`, `promotedAt 2026-10-23T00:02:00.519Z`, evidence closedA 30/closedB 31, eqB 1.048195 > eqA 0.987537, ddB 0.0728 < ddA 0.1132 — **byte-identical to the desktop's last-synced `data/live/exit_rule.json` copy** (same sha256). Must still read the **deployed** file + trader startup line at cutover (missing file → silent fallback to variant A/24 h). |
| 3 | **5-second exit polling located and carried over** | **Must be done at the desk.** Merged repo does **not** carry it: `meme/desk/trader.js:23` `poll: +(process.env.POLL_SEC \|\| 20)` (default 20 s) — confirms Nox's warning; the box fix (30 Oct, trader pid 2193662, rollback script on the box) must be located as env/config/code diff and carried; post-cutover it stays **UNPROVEN** until the first live exit proves it. |
| 4 | **MAX_HOLD_H env confirmed** | **Must be done at the desk.** Repo fallback is `MAX_HOLD_H \|\| 24` (`trader.js:23`); the rule file overrides it to 0.5 when present (test asserts the startup log line). Confirm deployed env is 0.5 or unset, never 24. |
| 5 | **Ledger continuity prepared (both ledgers + demotion scores)** | **Must be done at the desk** (copy from the box's copy; verify the unified app's DATA starts from that copy, not the repo seed ledgers). |
| 6 | **ntfy topic rotation ready** | **Not done** (topic appears in the imported repo history's original SHAs → rotate before/at cutover and update every publishing config). |
| 7 | **Cloudflare Worker redeploy ready** | **Not done** (so the phone page shows the "unknown" status correctly). |
| 8 | **Section 3b — desktop tree archived only after the host layer is repointed and verified** | **Precondition is real and active here**: `~/Work/projects/memedesk-host/units/memedesk-desk-fence.conf:8` `BindPaths=%h/Work/projects/openbell-meme-desk …`; `memedesk-desk.service` ExecStart=`desk-start.sh`; `memedesk-sync.service` and `memedesk-uptime.service` also reference the tree (and `openbell-meme-desk/data/shadow/heartbeat.jsonl`). The unit is currently **failed/inert** (missing JUP_API_KEY), so it cannot signal apps, but a repoint must still come first: repoint all five surfaces → verify the unified repo serves each role → only then move to `~/Work/_archive/harness-removal-20261030/`. **Nothing moved tonight.** |
| 9 | **One-trader rule during the move** | The move sequence (stop supervisor → 0 open → copy ledgers → fresh desktop wallet → Cayleb moves the SOL himself → start desktop live → stop box supervisor permanently) is one unbroken operation; **never two live traders**. Needs both machines + Cayleb's own SOL move. Not startable from this seat. |
| 10 | **JUP_API_KEY carry-over** | At cutover from the desk's existing config, agent-side, never printed in chat/log/git; no new signup (Cayleb 28 Oct ruling). Nothing read or handled tonight. |
Pre-check verdict: **the cutover cannot start tonight or Monday morning** — items 3/4/5 need the desk machine (route UNKNOWN, §5.1), items 6/7 need action, item 1 means the live checkout must be brought to `6f582f8`, and item 8 needs the host-layer repoint. If any pre-check fails: **the cutover does not start**; the live desk keeps running on the old code (already true) and the failure is reported.

---

# 6. N5 — ANSWER + RULING RECORDED (no action taken, none required)
**Question (Team E):** does the futures desk's CDP route use the running extension-host instance?
**Answer: NO.** Evidence gathered read-only at 2026-11-01T23:36Z:
- `ss -ltnp` → **`127.0.0.1:9222` is owned by `chromium` pid 2960519** (the desk's Chromium; systemd user scope
  `app-org.chromium.Chromium-2960519.scope`). The Codex extension-host (`/home/cayleb/.codex/plugins/cache/openai-bundled/
  chrome/latest/extension-host/linux/x64/extension-host`, **pid 2960775**) is a **child of 2960519** with **no listening
  socket** (`ss -ltnp | grep 2960775` → empty) and no established connection to the desk's clients.
- The desk's clients connect **directly to Chromium's own port**: `ss -tnp` shows established `127.0.0.1:9222` peers from
  `python3 pid 1481` (openbell-dash account watcher), `python3 pid 1484` (openbell-live engine) and the chrome-devtools MCP
  bridge; the desk code targets that port explicitly (`live/account.py:96 list_pages(port=9222)`,
  `live/cdpfeed.py:28`, `live/tabkeeper.py:25 PORT=9222`). The Chrome DevTools MCP bridge connects with
  `--wsEndpoint=ws://127.0.0.1:9222/...` directly too.
- Therefore **stopping only the extension-host (PID 2960775) does not touch the desk's route or its tab**: :9222 stays
  with Chromium 2960519. If Chromium respawns the extension-host as a child, note it and leave it (no broader kill).
- **Nox 20261101T233413Z §2 already ruled and closed N5 on exactly this basis**, with the Monday post-16:20 ET slot, N5
  first, Chromium untouchable, acceptance = `:9222` still Chromium 2960519 + desk feed error class unchanged + no respawn,
  and **Lane 6/upkeep-lead files the execution evidence; the row closes on that evidence**. Recording it here for Team E's
  Monday slot. I did not stop anything.

---

# 7. Blockers (named, with who holds them)
1. **P0.1 re-sign on runbook v7** — needs a fresh Tier S `trading-checker` before 09:05 ET Monday; **Lead spawns**. Without
   it, Monday does not start (gate rule). Hash to sign: `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`.
2. **ORB15 "do not trade its cards" ruling** — Nox/Cayleb; blocks any live human-tapped card (Monday Layer B) and any live
   ORB session. Monday Layer A (observation day) is not blocked by it.
3. **Autonomous path (G2.5 → G3 → G4)** — G3 needs owner assets: demo environment + demo access (Cayleb/Nox). G2.5 needs a
   signed-in tab outside the trading window (Monday ≥16:20). Not a Monday-morning item.
4. **Job 13 ground truth** — desk machine route = UNKNOWN (external lane, mail-by-hand); needs Nox to relay the data request.
5. **Job 15 cutover** — needs the desk machine (items 3/4/5), ntfy rotation + Worker redeploy (items 6/7), the live
   checkout brought to `6f582f8`, and the host-layer repoint before archiving (item 8).
6. **Replay divergence +9** (Kalshi) — pre-existing observation, not introduced tonight; flagged for its own look, not a
   readiness failure.

# 8. Spend, fences, evidence index, next step + successor
- **Spend: $0.** No live order, no wallet op, no credential value handled, no public post, no image read, no host input
  injection, no rail/threshold/gate change, no engine code change. All writes: `board/reports/team-c-batch7.md`,
  `board/reports/team-c-globex-open-outcome-20261101.md`, `board/reports/monday-futures-runbook.md` (v7),
  `board/probes/team-c/tick-proof-20261101T2343Z.md`, `board/probes/team-c/runbook-v6-signed-copy-027ca4e4.md`,
  `board/probes/team-c/kalshi-readiness-rerun-20261101T2335Z/**` (readiness JSON/log + scratch). Nothing written to
  Nox's board or any other team's files.
- **Evidence index:** tick proof (probe above); outcome filing (`team-c-globex-open-outcome-20261101.md`); runbook v7 +
  preserved v6; Kalshi readiness logs/JSON (`kalshi-readiness-rerun-20261101T2335Z/`); DST adjudication
  (`board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md` + checker verdict); G2 outcome/design v4/G2.5 plan; Nox
  184800Z written go; Nox 233413Z rulings; the pre-cutover checklist (read-only from the shared board).
- **Next step (owner `trading-lead`):** before 09:05 ET — hand the Lead the v7 hash for the checker re-sign; at 09:05 run
  the guard; at 09:15 the five attestations; 09:35–09:44 the (c2) binding; 09:45 window (observation; NO card tap unless
  the ORB15 ruling is on record); 16:05 flat proof; 16:20 outcome with completeness as the headline (PASS-empty if no
  signal); after 16:20, the N5 slot belongs to Team E and G2.5's read-only capture follows N5.
- **Successor:** fresh Tier S checker → v7 P0.1 signoff (Lead arranges), then the Monday day files; Kalshi resume at the
  01:00 ET lane (fit-split pull → stage_a → second_scorer AGREE → sizing_sim → dev_report) with the freeze target Tue 3–Wed
  4 Nov; Job 13/15 wait on the Nox-relayed desk answer.

— trading-lead, 2026-11-01T23:50Z (batch 7). Report to the Lead only; nothing here was sent to Cayleb or Nox.
