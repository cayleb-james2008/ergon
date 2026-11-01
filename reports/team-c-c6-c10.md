# TEAM C — BATCH 2 (C6–C10): route probe, Kalshi text pack, Job 15 pre-check matrix

trading-lead, 2026-10-31 18:35 ET. Read-only; no live action, no wallet, no order, no credential used, money-desk lane
untouched. Lead rulings received 18:2x ET are applied: X1 = cards/not-automated (runbook v2), X6 = 2000 stands, X7 =
L\*=30s FOR NOX, X9 = Nox told by Lead, X10 = repo `docs/CUTOVER.md` is the reference, X12 = evidence only.

---

## C6 — Machine-route candidate: the outis-pilot support plane → **route = NO**

What it is (read-only inspection; no connection, no authentication, no command sent):
- A **MeshCentral** server + a small control API, all under `~/.local/share/outis-pilot/` (owner `cayleb`, created
  2026-10-13 by the earlier "outis-pilot" task; the plane is on Cayleb's desktop-cleanup HOLD list, 99-not-delegated item 15).
- Running now: `node .../mesh-launch-20261013/start-meshcentral.mjs` (pid 1489) and `node-MainThread` (pid 1488) for the
  control API. `mesh-state/data/serverstate.txt`: `state=running`, `mps-name=outis-desktop.tail1b30fd.ts.net`,
  `http-port=8788`, `http-aliasport=443`. The MeshCentral event/stats stores were last written 18:25 ET tonight, i.e. the
  server is live.
- Exposure: `ss -ltnp` shows **443 and 8443 bound only to `100.114.17.67%tailscale0`** (tailnet only, not public) and
  **8787 on loopback only** (control API). No public listener.
- Auth model: bearer-style operator/worker/connector tokens in `control-config.json` (mode 0600; values not read into this
  report), one per-device token, and a MeshCentral user `pilot-operator` (`mesh-operator.json`). Tokens were NOT printed,
  copied or used.
- Enrolled devices — the whole inventory:
  - `mesh-groups.json`: exactly **one** mesh group, `"Windows pilot"` / desc `"Approved Ergon laptop pilot"`, created by
    `pilot-operator`.
  - MeshCentral node store: exactly **one** node, `LX15PRO` (Windows 11 Pro 26H2) — first connect 2026-10-25 08:57 ET,
    **last connect 2026-10-30 03:17 ET**; no device online now.
  - `control/control.sqlite` (read-only query): tables `tickets`, `notifications`, `devices`, `audit` — **0 rows in every
    table**. The control plane has never carried a device session or a ticket.
  - `approved-test-destinations.json`: deviceId `laptop-pilot`, a fixed pilot phone/email test destination (personal
    contact data; not reproduced here), `smsEnabled: false`, provisioning "Pending protected provider credentials".
- **Answer: NO — this is not a route to the desk machine or the coordinator VM.** Only the laptop (`LX15PRO`) was ever
  enrolled; it has not connected since 2026-10-30 03:17 ET; the control database is empty; nothing names the desk machine,
  Grok/RSI's VM or srv1360751.
- What a *legitimate* use would require (Cayleb acts, and a ruling before any of it):
  1. A MeshCentral agent installed **on the target machine, by whoever owns it** (Cayleb or RSI at that machine) and a new
     mesh group + device token — i.e. a new inbound remote-control path into a live-money machine. That collides with
     (a) the Kalshi rule "the desktop holds no signature material and accepts no inbound commands" and (b) the Tier S
     hands design (Stage 8) which is the sanctioned, refusal-by-construction control path with a physical kill test.
  2. A written scope (read-only vs command), an idle-yield rule, and a hash-chained log — the Stage 8 properties — plus
     the checker's VETO signature and Cayleb's go. None of that exists for this plane.
  3. Therefore: **do not use it for Job 13/15.** The supported answers remain (i) a Nox-mediated handoff, or (ii) RSI
     exporting the desk's own files to a place this desktop can read. Recommendation to the Lead: ask Cayleb for a
     *data export path*, not for remote control — the jobs need the desk's files (live.log, status.json, ledgers,
     deployed exit_rule.json, env), not its keyboard.
- Local security observation for the Lead (not ours to fix, evidence only): several **private key files** in this plane are
  mode 644 (world-readable) — `meshcentral-data/*-private.key`, `mesh-state/data/*-private.key`; and the plane's
  `meshcentral.db` is 644. The plane is on the cleanup HOLD list; flag for Job 19/Cayleb, do not touch tonight.

**C6b — Nox's board: is there a defined mail route to the Grok/RSI desk besides "external = hand-delivered"?**
No. Three independent statements in Nox's own files:
- `ops/LANES.json`: `{"id":"grok","name":"Grok / RSI desk","kind":"external"}` + file note "external (no machine route:
  mail to it is handed to Nox)".
- `ops/README.md` line 74: "**external:** handed to you (**MAIL_BY_HAND**)"; flags `MAIL_BY_HAND / MAIL_HELD /
  MAIL_FAILED` exist precisely because there is no machine route ("deliver by hand").
- `ops/README.md` lines 44–46: the only grok-shaped promise (`P-grok-pacing`) closes on
  `board_glob: inbox/*/*-grok-to-nox.md` — a file that can only appear if someone writes it by hand; nothing in the board
  code writes it for an external lane.
- The separate `money-desk/intake` dispatcher (live, heartbeat 22:20:01Z) targets the **moneydesk Claude lane**, not grok.
So: mail route = **hand/Nox only**, as the registry says. No hidden route found.

---

## C8 — Kalshi text pack (ready to log; **FOR NOX — nothing edited in any repo**)

Target files are named per item. No file was modified; the openbell-kalshi worktree is untouched (readiness output went to
`board/probes/team-c/`).

### C8.1 — Ruling 1: hourly bar = 2,000 (Lead: 2000 stands; location question to Nox)
Ready-to-log amendment text (SPEC Appendix T style, and PREREG §2/§3 if the 300 appears there):
> **Amendment 4 (hourly bar).** The hourly arm (Arm B, KXBTCD belly) bar counts **2,000 hourly events** (D2, DECISIONS.md
> 2026-10-31 06:57 ET). Any 300-event/hourly-bar figure in this directory is superseded by this amendment. The 15M arm's
> bar stays at 2,000 settled windows per coin. No bar moves mid-count (D5).
**X6 location question to Nox (ask verbatim):** "Job 14's brief says 'the hourly bar reads 300 in one section'. That 300
is not present in SPEC.md, PLAN.md, PREREG-DRAFT.md, DECISIONS.md, BUILD-LOG.md or FACTS.md in
`~/Work/projects/openbell-kalshi` (searched 2026-10-31 22:1xZ; the only hourly-count statement is D2 = 2,000). Which
document/section carries the 300, and is it on the coordinator side or stale in the brief? The amendment text above is
ready to log against that location."
State on this desktop: **not locatable** (evidence in team-c-phase1.md X6).

### C8.2 — Ruling 2: stale header (ready to apply the moment it is ruled; the correct text already exists in tests)
Target: `research/kalshi15/PREREG-DRAFT.md` §1 (currently: "**Nothing trades.** No order path exists in this stack
(`live/kalshi/test_no_order_path.py`), and no credential is used.") and any other copy.
> **Amendment 5 (order-path wording).** The sentence "No order path exists in this stack" is superseded (Nox Amendment 1,
> SPEC Appendix T item 3; DECISIONS.md line struck). Replacement: "The shadow stack (`live/kalshi`, `research/kalshi15`)
> contains no order path. A separate execution package (`live/kalshi_exec/`) exists and is disabled by construction: it
> cannot place an order without (i) the enable flag, (ii) a passing gate, (iii) Cayleb's explicit go relayed in writing.
> The guard tests assert that refusal." Nothing else in §1 changes.
Evidence it is already true in code: `live/kalshi_exec/test_no_order_path.py` part B asserts the supersession;
`test_refuses.py` asserts the order path exists and refuses (64-combination guard table, all refuse);
`live/kalshi_exec/DESIGN.md` C3 explicitly leaves this wording to Nox. This is the one item that needs no new measurement.

### C8.3 — Ruling 3: Kelly posture (daily loss stop untouched)
Target: `research/kalshi15/DECISIONS.md` D6 + Amendment 1 "D6 addendum"; `SPEC.md` Appendix T item 4; `PREREG-DRAFT.md` §3
D6 sentence.
> **Amendment 6 (sizing posture).** Live sizing is not capped quarter-Kelly only. Recorded posture: Cayleb directed
> heavier sizing to escape zero-contract rounding at small capital — at $24.09 most quarter-Kelly sizes round to zero
> (dry run: 124 of 145 fires, 85.5%). Live sizing therefore uses the **re-scored sizing table's** fraction, chosen at the
> flip; quarter-Kelly remains the comparison arm in the record; the 100%-deploy arm stays a simulated comparator and never
> drives live orders. **The daily loss stop is untouched.** The D6 addendum is amended to read: "quarter-Kelly is the floor
> reference; the live fraction is set at the flip from the re-scored sizing table; the daily loss stop is unchanged."
> Funding and sizing remain Cayleb's decisions; no live order follows from this amendment.
Contradiction being resolved (three-way): `DECISIONS.md` D6 (Cayleb verbatim: deploy essentially 100% per cycle) vs
Amendment 1 D6 addendum ("capped quarter-Kelly only") vs the recorded posture above.

### C8.4 — L\* = 30 s: DECISIONS.md log line (Lead: FOR NOX'S RULING — do not edit)
Ready-to-paste row for the Amendment-1 table in `research/kalshi15/DECISIONS.md`:
> | Latency (amendment 2) | Provisional **L\* = 30 s** replaces the 60 s row for θ/X selection, on the measured
> availability of 2026-10-31: decisions readable **16.127 s p50 / 16.735 s p90** after the decision tick and 24.586 /
> 27.713 s after `a_asof` (`results/availability_2026-10-31.json`, 81 decision lines / 9 ticks / 10 min); with intent pull
> ~1–2 s and gateway <1 s the end-to-end from the tick is ≈ 19–20 s, so 30 s is the smallest row of record at or above it
> (and above the data-time p90). The 60 s row stays a row of record; the live gate still applies at the row matching the
> gateway's **measured** end-to-end latency, never the best row; if that measurement lands above 30 s the threshold
> re-derives at that row before any flip. Rows 2 s and 5 s stay informational. Reversible before the freeze. |
Follow-up note to append only after his ruling (his own convention, 13:17 ET style):
> **Confirmed by Nox (2026-11-0x HH:MM ET):** provisional row L\* = 30 s for θ/X selection until the gateway's dry-run
> timestamps produce the measured end-to-end latency; the 60 s provisional is replaced and this line is the replacement.
Do NOT change `results/scorer_agreement.json`'s `latency.L_star` before the ruling (it is a generated artefact).

### C8.5 — X9 stale-collector notice (to Nox; Lead is already telling him — this is the text)
> **Notice to Nox (Kalshi collector freshness).** Your independent collector copy at
> `~/Work/projects/kalshi-shadow-data/data/kalshi-shadow-2026-10-31.jsonl` (51 MB, 165,051 lines, mtime 08:35 ET) has
> **last record 2026-10-31T11:59:42Z** — pushes appear stopped at 12:00Z. Nothing is lost on the desktop side (local
> capture is fresh to 21:54:28Z; readiness at 21:54Z = 9 PASS / 1 NA / 0 FAIL). Consequence: readiness check 8
> (`nox_reconcile`) is **NA by construction** ("no time overlap with Nox's clean cycles; settlements both 45, result
> disagree 0, value disagree 0"), not a failure. Please confirm the collector/trail push is running or restart it before
> the Monday resume; if it stays down, the resume pack ships with check 8 NA and that line quoted.

---

## C9 — Job 15 pre-check matrix from `docs/CUTOVER.md` (master 6f582f8; every pre-check, its command, its artefact)

Reference read in full: `github/master:docs/CUTOVER.md` (4 sections) + `github/master:meme/host/RUNBOOK.md`
("Moving from the old layout") + `meme/desk/trader.js` / `run.sh` / `seed/live/exit_rule.json`. Master verified:
`gh api repos/cayleb-james2008/openbell/branches/master` -> `6f582f87db3917e2147bd02d509ec1886fc774a8 Merge pull request #3
… Unify 3/3`; only branch `master`; local `git rev-parse github/master` == `git rev-parse 6f582f8` -> YES.

### (i) Doable from this desktop now (read-only; done/queued)
| # | Pre-check | Exact command | Artefact it must produce |
|---|---|---|---|
| 1 | Master is the target, one branch | `gh api repos/cayleb-james2008/openbell/branches --jq '.[].name'` and `.../branches/master --jq '.commit.sha'` | `["master"]`, `6f582f8…` — **DONE, quoted** |
| 2 | Repo seed exit rule (variant B) | `git -C ~/Work/projects/openbell-unified show github/master:meme/desk/seed/live/exit_rule.json \| sha256sum` | hash of the seed + the JSON fields `variant B / maxHoldH 0.5 / promotedAt 2026-10-23` — **DONE** |
| 3 | Polling default in repo | `git -C … grep -n "POLL_SEC" github/master` | `trader.js: poll: +(process.env.POLL_SEC \|\| 20)`; no 5 s anywhere — **DONE** |
| 4 | MAX_HOLD_H default in repo | `git -C … grep -n "MAX_HOLD_H" github/master` | `trader.js: maxHoldH +(process.env.MAX_HOLD_H \|\| 24)`; `run.sh` paper 0.5 — **DONE** |
| 5 | Worker + ingest tooling present | `git -C … ls-tree -r --name-only github/master \| grep -E "meme/desk/watchdog/cf/worker.mjs\|meme/host/settings/settings.py"` | both paths listed — **DONE** |
| 6 | ntfy config exists (content never read) | `ls -l ~/.config/openbell/ntfy.json` | `-rw------- 190 B` exists; rotation procedure = CUTOVER §3 — **DONE** |
| 7 | Old-layout move procedure is in the repo | `git -C … show github/master:meme/host/RUNBOOK.md \| sed -n '/Moving from the old layout/,/^## /p'` | the ordered stop/install/carry-over/start/fence-probe steps — **TO DO (next batch, read-only)** |
| 8 | Fence probe exists | `git -C … ls-tree … \| grep meme/host/fence/probe.sh` | path listed — **DONE** |
| 9 | One-trader/no-funds rules are in the move doc | `git -C … show github/master:docs/CUTOVER.md \| sed -n '40,60p'` | §2 "Stops the old desk and host … then starts the new target"; §4 keep old trees a full day — **DONE** |

### (ii) Needs the desk machine route (BLOCKED — X5; cannot be faked from here)
| # | Pre-check | Exact command (on the desk machine) | Artefact |
|---|---|---|---|
| 10 | **Deployed** exit-rule byte-check | `sha256sum <desk DATA>/live/exit_rule.json` vs the seed hash from (2) | hashes equal + file copied off-machine; mismatch = cutover does not start |
| 11 | Trader startup log line for the rule | `grep -m1 "exit rule B" <desk>/data/strategy/live.log` (or the live log path) | `exit rule B (…exit_rule.json, promoted …): maxHoldH 0.5 overrides MAX_HOLD_H=…` |
| 12 | 5-second polling located | `grep -rn "POLL_SEC" <desk>/run.sh <desk>/supervisor/ <desk>/*.env <desk>/systemd/` (or the trader env) | file+line proving `POLL_SEC=5`, plus the carry-over line for the new host |
| 13 | MAX_HOLD_H env confirmed | read the live trader process env (`/proc/<pid>/environ`, name only) | expected 0.5 (or absent + exit rule B supplying 0.5) |
| 14 | Ledger continuity copies | `sha256sum` of `data/live/ledger.jsonl`, demotion scores, `data/live/trades.csv` before/after the copy | checksums recorded; new build appends, does not truncate |
| 15 | 0 open positions at cutover moment | desk `data/status.json` + the supervisor's own state | zero open + zero in flight (Job 13's own safety gate) |
| 16 | Old supervisor stopped, new one started, one trader | supervisor's own controls on the desk machine + the new host's unit | exactly one live trader at every instant (never two) |
| 17 | Old runtime dark after the move | desk machine process list + its own log stops | no desk process, log silent |

### (iii) Needs Cayleb (his acts; prep only)
| # | Item | What he does | Gate |
|---|---|---|---|
| 18 | **Move the SOL himself** (Job 15 §2) | his hands, his act, from the old desk wallet to the new desktop wallet | before the new trader starts; one trader rule does not bend |
| 19 | New wallet keyfile placement | write the key file inside `~/.config/memedesk-wallet/` (never rename/recreate the folder) | key never printed/logged |
| 20 | `JUP_API_KEY` presence | already exists in the desk's config; if it must be re-entered: `python3 -I meme/host/settings/settings.py ingest FILE --names JUP_API_KEY` (write-only, value never printed) — note `lite-api.jup.ag` retired 2026-10-30, so a dead key fails every Jupiter call | before first live start |
| 21 | ntfy re-subscribe | subscribe the new topic on his phone, unsubscribe the old one (CUTOVER §3) | test card received |
| 22 | Cloudflare Worker redeploy | redeploy from the regenerated `meme/desk/watchdog/cf/worker.mjs` (phone page "unknown" vs "paper") | before the phone page is trusted |
| 23 | Exit-rule A→B auto-promotion decision in the new layout | choose which books count as evidence (`data/shadow/variants`?) — the host layout fails closed, live stays rule A until decided | before any promotion is expected |
| 24 | Cutover window choice | a quiet window (weekend, or weekday after 17:30 ET and not during the arena's 16:10 run) | scheduling only |
| 25 | Section 3b / old-tree archive | confirm the archive after the new desk has run a full day (CUTOVER §4); repo archive of `openbell-meme-desk` is a separate Job 18 pick | after verification |

**Cutover stop rule (unchanged):** any failed pre-check (10–17) means the cutover does not start; the live desk keeps
running on the old code and the failed check is reported. Nothing above was executed tonight.

---

## C10 — Bookings
The pinned Monday table (09:05 / 09:15 / ~09:40 / 09:45 / 16:05 / 16:20, plus Mon 01:00 and Tue–Wed freeze, owner +
backup) lives in **`board/reports/monday-futures-runbook.md` §BOOKINGS**, including the "if X1 = hold" slip column:
09:45 window **SLIPS** on a hold; guard/preflight/16:20 still run (read-only evidence, decision log declared empty rather
than complete); Kalshi bookings unaffected; the fill-based done-condition moves to the successor build in both readings.
