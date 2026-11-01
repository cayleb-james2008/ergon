# E6a — MONDAY 07:45 ET SWEEP: DRY RUN, NOW (2026-10-31 18:53-19:06 ET)

Author: `upkeep-lead` (Team E). Order: "execute the runbook's three checks in read-only mode NOW against
current data and record their exact output/commands (so Monday is mechanical, and any command that does
not work is found tonight, not at 07:45)."

Artifacts:
- `board/probes/team-e-e6/monday-dryrun.sh` — the re-runnable script (READ-ONLY; 33 commands).
- `board/probes/team-e-e6/monday-dryrun-corrected-final.log` — **canonical run, 19:06 ET: 33 commands,
  all `[exit=0]`, 215 lines**. Earlier logs kept for the audit trail: `monday-dryrun-20261031T2252Z.log`
  (first run, found the gaps), `monday-dryrun-corrected-20261031T2306Z.log`.
- Monday invocation (exactly):
  `cd board/probes/team-e-e6 && ./monday-dryrun.sh '2026-11-02T01:00:00-05:00'`
- Output record for the findings file: `board/reports/monday-2026-11-02-findings.md` (created then, not now).

**Honest limit of this dry run:** tonight is Saturday — market closed, the reset has not happened, and
most "since 01:00" windows are today's. The dry run proves the commands work and captures a baseline; it
does **not** predict Monday's readings.

---

## 1. What ran, and what each check now returns (quoted from the canonical log)

### Check A — Duet verdicts / Kimi handback → works, quiet outcome
```
### gh api .../ergon/branches ... | grep -i kimi || echo "NO KIMI BRANCH ..."
NO KIMI BRANCH (quiet outcome: handback not arrived)                      [exit=0]
### git ls-remote .../ergon.git refs/heads/main
b59b5990c594bfe7dfda859fcac3bff09ef609d3	refs/heads/main                  [exit=0]
### git ls-remote .../ergon.git "refs/heads/*kimi*"          → (no output)   [exit=0]
```
`gh pr list -R …/ergon` returns the full PR set (7 PRs: #6 MERGED, #7 CLOSED, #5 CLOSED draft, #4/#3/#2
MERGED, #1 CLOSED) — usable as Monday's "did anything open/merge overnight" read.

### Check B — trading session/feed before the 09:05 guard → works; **one live risk found**
```
### systemctl --user show openbell-live.service -p ActiveState,SubState,ActiveEnterTimestamp --value
active running Wed 2026-10-28 15:40:09 EDT                                [exit=0]
### journalctl --user -u openbell-live.service --since "$SINCE" | grep -c "feed attach/read error"
2168                                                                      [exit=0]
### … journalctl … -n 3 → 19:02:43 openbell.feed WARNING feed attach/read error:
    RuntimeError('no trader.tradovate.com tab'); retry in 30s             [exit=0]
### curl -s http://127.0.0.1:9222/json | grep -ci tradovate
0                                                                         [exit=0]
### systemctl --user list-units "openbell-kalshi*"
openbell-kalshi-poll.service   active running  … no orders, no credentials
openbell-kalshi-ref.service    active running  … no orders, no credentials
openbell-kalshi-shadow.service active running  … paper only … no network, no orders
### gh api …/kalshi-shadow-data/commits?per_page=1
256526a7  2026-10-31T22:45:20Z  Capture snapshot 2026-10-31T22:45:20Z
### gh api …/kalshi-shadow-data/contents/data
kalshi-shadow-2026-10-31.jsonl      93411497
kalshi-shadow-2026-10-31.jsonl.gz   16103273
```
`openbell*` user units (all declared shadow/passive): `openbell-live` (MES signal engine — "never places
orders"), `openbell-dash` (127.0.0.1:8791 + passive Tradovate account watcher), `openbell-arena`
(shadow), the three Kalshi units, `openbell-tabkeeper` ("keeps Chromium + the Tradovate tab open; never
signs in"), plus `openbell-forward-archive.timer` (weekdays 17:20 ET, currently waiting). `pgrep` for
`trader.js|momo|supervisor` returns none — no meme-desk processes on this desktop, as Team C said.

### Check C — which surfaces resumed after the reset → works
`Nox STATUS.md` head (lane table) read; commits-since-$SINCE for `ergon` / `dsh-fleet` /
`kalshi-shadow-data`; `systemctl --user` lane units; system `fleet-dsh.service` (active/running). All
command forms return cleanly; the "since" window is the only part that changes on Monday.

### Tripwire watch (added to the script) → works
`active 3178689 0`; line-anchored event grep; `real CLOSEFOCUS: 0`; hypr stream size/mtime printed.

---

## 2. Gaps found by the dry run — and fixed in the script/runbook before Monday

| # | Gap | Why it mattered | Fixed to |
|---|---|---|---|
| C1 | Checked only **system** units for `openbell-kalshi-*`, found none | They are **user** units; a system-only check would have reported "units missing" wrongly | `systemctl --user list-units "openbell-kalshi*"` |
| C2 | Freshness judged from `~/Work/projects/kalshi-shadow-data/data/*.jsonl` mtime (showed 08:35 ET) | The local file is **not** the live path; Nox's 22:52Z mail says the day file crossed GitHub's 100 MB limit and now ships as `.jsonl` + **`.jsonl.gz`** (repo commit 22:45:20Z is the real freshness signal) | repo commit + contents listing (both sizes), local mtime de-emphasised |
| C3 | No check of the futures passive feed's browser attachment | Found a live risk (§3-F1) that a generic "feed looks fine" check would have missed | `openbell-live` state + warning counter + `curl :9222` tradovate tab count |
| C4 | `grep -iE "openbell|kalshi|memo|futures"` matched **`systemd-oomd`** (because "memory" contains "memo"); `pgrep -af "live"` matched `live.engine` | False positives would waste the 07:45 window and mislead the findings table | patterns: `openbell*` unit glob, `trader\.js|momo|supervisor` |
| C5 | Counting tripwire events by substring counted **snapshot/journal text** (my own command lines) as events | Would over-report "events" that never happened | line-anchored regex `^<ts> (GONE|CLOSEFOCUS)`; the script prints "real CLOSEFOCUS" |
| C6 | `systemctl --user list-units --all` prints the giant `dsh-subprocess-*` command lines | 10 KB of noise per line in the findings file | `grep -vi dsh-subprocess | cut -c1-150` |
| C7 | `grep -ci tradovate` exits 1 when the count is 0 | A mechanical reader would see a non-zero exit as failure | `|| true`; all 33 commands now exit 0 |

## 3. Findings that are leads, not diagnoses (for Monday's findings file / the Lead)

**F1 — futures passive feed is detached (EXPECTED-weekend observation; Monday PROOF requirement;
trading-lead owns attestation (c)).**: `no trader.tradovate.com tab`, retrying every 30 s.**
- 2,281 warnings today (first `00:00:05`, last observed `19:02:43`); `curl :9222` tradovate tab count = 0;
  `openbell-tabkeeper.service` active since 2026-10-28 17:31:46 but no tab is present.
- This is an **expected-weekend observation, not a fault** (market closed; the Lead confirmed the framing).
  It is not diagnosed as a defect. Monday's guard must **prove the tab**, not assume it: the exact read is the two commands
  above (warning counter trending to 0 + tab count ≥ 1 after the tab is opened).
- Handed by the Lead to `trading-lead` as a Monday attestation (c) risk. Team E does not open the tab,
  sign anything in, or touch the trading path.

**F2 — tripwire's first real catch (lead for mystery 4, not proof).** `2026-10-31T18:55:14.988 GONE
chromium pid=3221336 proc_still_exists=True`, snapshot shows the parent is an Ergon test run
(`node … tests/dashboard-drafts.test.mjs`, output under `/tmp/ergon-test-output-*`) and the children are
Playwright-style (`--user-data-dir=/tmp/playwright_chromiumdev_profile-…`, `--headless`). The interactive
browser (PID 2960519, up since 16:19:24) was untouched. **No `CLOSEFOCUS` line exists** — so "closed
while unfocused" is **not** established by our evidence; what is established is that a chromium main
process from a test harness vanished and the recorder caught it with a full snapshot. The Lead is
asking Team A whether its suite launches/closes Chromium — that is the first named candidate sender.

**F3 — 0 real `CLOSEFOCUS` events so far.** v2.1 is armed but has not yet observed an unfocused window
close. The only two "CLOSEFOCUS" text hits in `tripwire.log` are journal-tail text from my own test
commands. No one should read those as events.

**F4 — Kalshi transport (from Nox's 22:52Z correction, read-only).** The collector push was rejected
after 17:00Z because the day file reached 145.8 MB, over GitHub's 100 MB per-file limit; fixed 22:45Z by
shipping `data/kalshi-shadow-<date>.jsonl` (last under-limit state) **plus** the full capture as
`data/kalshi-shadow-<date>.jsonl.gz`. Her copy has a capture gap ~17:00Z→22:36Z; the desktop lane's
capture is the primary record. The repo now shows both files (93,411,497 B and 16,103,273 B).

**F5 — no regressions in the estate probes:** no `kimi` branch (Job 05 stays "not arrived"); ergon main
still `b59b599`; new overnight artifacts exist in `board/reports/` (`ROUND1-REPORT.md`,
`stage2-gate-report-DRAFT.md`, `team-b-phase2.md`, `team-c-*`), which Monday's sweep should read for
"what changed" rather than re-derive.

## 4. What Monday looks like (mechanical)

1. `07:35` run the script with the Monday SINCE; save the log.
2. Fill the findings table from the log (one row per check; each row gets command + quoted output +
   disposition + owner).
3. **Red conditions** (escalate to `lead` immediately, interrupt allowed): feed warning counter still
   climbing after the tab is proven; `openbell-kalshi-*` not active; any stop/error in an `openbell*`
   unit; a Kalshi `.jsonl.gz` older than the 07:45 run; a `GONE chromium/sober` that is not a known test
   harness; any indication of an order/wallet action (none is expected — every unit says "no orders").
4. Deliver the findings file by 08:30 ET, report to the Lead by 08:45 ET; the 09:05 futures guard is
   Team C's and must not be delayed.
5. Backup owner: `lead`. If the run cannot happen, say so plainly — never backdate.

## 5. Spend / writes

$0. No GitHub writes (reads only). No trading action, no sign-in, no tab opened. No write under
`~/.local/opt/deepseek-harness-official/local-plugins/` or that session's DSH profile (steer rule
acknowledged 19:05 ET). The only writes were this report, the script, its logs, and the runbook/digest
edits under `board/`.

**F2 update (19:20 ET): RESOLVED as the programme's own checker run — lead closed.** The parent of the
18:55:14.988 GONE was the `ergon-checker`'s independent suite (`suite-p20-709bf0e.log`, `real
4m24.757s`, window ≈18:54:34–18:58:59, names `tests/dashboard-drafts.test.mjs` with `--test-reporter=tap`);
the parent was alive at the GONE. Full reconstruction in `board/reports/team-e-tripwire-patch.md` §8.
