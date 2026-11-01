# MONDAY 07:45 ET LOOSE-ENDS SWEEP — runbook (pinned booking)

Owner: `upkeep-lead` (Team E). Backup: `lead`. Booking source: `board/BOARD.md` timed table
("Mon 2 Nov 07:45 — Monday loose-ends sweep"). Written 2026-10-31 18:30 ET.
**Last re-verified 2026-11-01T23:50Z (18:50 ET) by Lane 6:** §0 GitHub cadence current (armed Tue 3 Nov 09:00 ET / 14:00Z, Lead-owned); §1-§5 commands unchanged and still valid; §7 **added** for the now-ruled N5 post-16:20 step (Nox 20261101T233413Z §2). Nothing else changed.
**Hard deadlines:** findings file written by **08:30 ET**; chat report to `lead` by **08:45 ET**;
the futures 09:05 guard is Team C's and this sweep must not delay it.

**VERIFIED SCRIPT (use this, not ad-hoc typing):** `board/probes/team-e-e6/monday-dryrun.sh [SINCE_ISO]`
— 33 read-only commands, all `[exit=0]`, dry-run 2026-10-31 18:53-19:06 ET; canonical log
`board/probes/team-e-e6/monday-dryrun-corrected-final.log`; findings and gaps in
`board/reports/team-e-monday-dryrun.md`. Monday invocation:
`./monday-dryrun.sh '2026-11-02T01:00:00-05:00'`. The snippets in §§2-4 are the same commands for reference.

Constraints (non-negotiable): read-only on Nox's board (`~/Work/shared-workflows/lanes/board/**` — one
writer per file, not us); no trading action, no broker session, no wallet, no order; no spend; no
credential handling; one consolidated report, no drip-feed; never contact Cayleb directly — anything
needing him goes to `lead` in one batch. Closed projects stay out of the report.

---

## 0. The recurring GitHub pass — ARMED by the Lead (patched 2026-11-01 15:00 ET; old text kept as history)

**PATCHED 2026-11-01 15:00 ET (Lane 6, Lead instruction):** the scheduler-refusal / manual-fallback text below is history
only. The recurring pass is now **ARMED by the Lead**: "Every 2 days — GitHub estate read-only sweep", next occurrence
**Tuesday 3 Nov 2026 09:00 ET** (14:00Z), Lead-owned. The 07:45 sweep records it as **scheduled, not pending**; do not run
the manual fallback unless the armed sweep is missing at run time.

`schedule_create` was attempted for the next hygiene pass and was **refused by policy**:
`{"code":"subagent_session","message":"A delegated subagent cannot use scheduled tasks."}`
(2026-10-31T22:31Z). **SUPERSEDED 2026-11-01: the recurrence is armed in the Lead's session (see PATCHED note above);**
the manual procedure below is retained as a fallback only.

**Spec for `lead`'s main session (paste into `schedule_create`):**
- title: `GitHub hygiene sweep, 61 repos read-only`
- target: `this_session` (or `new_once` if the Lead prefers separate reports)
- rrule: `DTSTART;TZID=America/New_York:20261102T180000` newline `RRULE:FREQ=DAILY;INTERVAL=2`
- prompt: point it at `board/reports/team-e-phase1.md` §5 E6 and
  `board/probes/github-sweep-20261031/sweep.sh` + `NOTES.md`; require the read-only-first discipline,
  the rate-limit before/after logging, the honest partial-coverage rule, and a ≤30-line report to the Lead.
- **Note the weekday:** 2 Nov 2026 is a **Monday**, not a Sunday (next Sunday is 1 Nov 2026). The
  cadence is 2 days from pass 1 (Sat 31 Oct 18:00 ET), so **Mon 2 Nov 18:00 ET is the correct next
  occurrence**; if the Lead meant the literal Sunday, use `20261101T180000`.

**Manual procedure (if no task exists by Sun 1 Nov 18:00 ET):**
```bash
cd /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes
D=github-sweep-$(date +%Y%m%d); mkdir -p "$D" && cp github-sweep-20261031/sweep.sh "$D"/
cp github-sweep-20261031/repos.tsv "$D"/   # baseline for "changed since last pass"
cd "$D" && ./sweep.sh && cat sweep-timeline.log
# then diff: new/changed PRs, archived-flag changes, branch-count delta, repos added/removed
```
Done-condition: `repos.tsv` has all current repos (61 at the last pass), `sweep-timeline.log` has
rate-limit before/after, and a report exists naming coverage n/61 — or, if 429, the missing slice by name.

---

## 1. Pre-flight (07:30-07:45 ET)

```bash
date; TZ=America/New_York date '+%Y-%m-%d %H:%M %Z'          # EST after the 1 Nov DST change
head -3 ~/Work/shared-workflows/lanes/board/ops/STATUS.md    # sensor must be <15 min old
git -C ~/Work/shared-workflows/lanes/board log -1 --format='%h %ci %s'
```
- If the sensor is stale/down: record it as a finding, use the last STATUS.md, and do not claim
  liveness for any lane.
- Read `board/BOARD.md` job rows 01-19 and note any status the Lead changed overnight.
- Read `board/reports/` filenames + mtimes to see which team reported overnight.

## 2. Check A — Duet verdicts against outstanding packs (Ergon lane)

What "owed" means: (i) the Kimi redo handback (Job 05 waits on it — the correct quiet outcome is a
one-line "not arrived"); (ii) any Duet/Pack verdict that changes an outstanding pack (Pack 19 merge,
Pack 20 finish, Pack 18 follow-ons).

```bash
# handback arrival: a branch with "kimi" in its name, or a new remote ref
gh api 'repos/cayleb-james2008/ergon/branches?per_page=100' --paginate --jq '.[].name' | grep -i kimi
# overnight Ergon activity + any new PR
git ls-remote https://github.com/cayleb-james2008/ergon.git 'refs/heads/main' 'refs/heads/*kimi*'
gh pr list -R cayleb-james2008/ergon --state all --limit 15 --json number,title,isDraft,state,updatedAt
```
- Record: handback yes/no; newest ergon main sha + timestamp; any PR opened/merged/closed overnight.
- If the handback **has** arrived: do not audit it now — report "arrived, Job 05 successor is the
  remote-branch verification, then the alignment audit" and hand it to `lead`.
- If not: write exactly "not arrived" and move on (no re-raising, no speculation).

## 3. Check B — trading session/feed state ahead of the 09:05 guard (read-only)

Authoritative source for what must be true: `board/reports/team-c-phase1.md` §C3 (desktop sign-in
route, broker record access) and §C4; `board/reports/monday-futures-runbook.md` (Team C's runbook).
**Do not open a broker session, do not place/flatten anything** — the 09:05 guard and the five
attestations are Team C's, and any fresh sign-in is theirs to make.

```bash
# futures feed/desk data freshness on this machine (files only; no venue login)
ls -lt ~/Work/projects/*/data/*.jsonl 2>/dev/null | head -5
systemctl list-units --all --no-pager 2>/dev/null | grep -iE 'openbell|kalshi|memo|futures' | head -10
# C3: futures passive feed attaches through a Chromium tab — PROVE it, don't assume it
systemctl --user show openbell-live.service -p ActiveState,SubState --value | tr '\n' ' '; echo
journalctl --user -u openbell-live.service --since "2026-11-02T01:00:00-05:00" --no-pager \
  | grep -c 'feed attach/read error'          # Monday target: 0 after the tab is proven
curl -s --max-time 3 http://127.0.0.1:9222/json | grep -ci tradovate || true   # target: >= 1
# C1: Kalshi units are USER units — a system-scope check reports them missing (dry-run finding)
systemctl --user list-units 'openbell-kalshi*' --all --no-pager --plain
# C2: freshness = repo commit + the .jsonl/.jsonl.gz pair, NOT the local file mtime
gh api 'repos/cayleb-james2008/kalshi-shadow-data/commits?per_page=1' \
  --jq '.[0] | [.sha[0:8], .commit.author.date, (.commit.message|split("\n")[0])] | @tsv'
gh api 'repos/cayleb-james2008/kalshi-shadow-data/contents/data' \
  --jq '.[] | [.name, (.size|tostring)] | @tsv'
```
- **Feed finding framing (Lead, 19:0x ET):** tonight's `no trader.tradovate.com tab` warnings
  (2,281 today) are an **EXPECTED-weekend observation**, not a fault — the market is closed. Monday
  07:45 records it as a **PROOF requirement**: warning count trending to 0 and tradovate tab count ≥ 1
  after the tab is proven. `trading-lead` owns attestation (c); Team E does not open or sign in to any tab.
- Record: newest capture/session file + timestamp; unit enabled/active states exactly as printed;
  any stop/error marker; whether anything looks like a live order or wallet operation (must be none).
- Red flags to escalate to `lead` immediately (interrupt allowed for a money event/blocker): feed not
  updating, a live path enabled, a wallet/file move, or a futures unit that will not start before 09:05.

## 4. Check C — which agent surfaces actually resumed after the 01:00 ET reset

Rule: liveness = evidence, never labels. For each surface, require an artifact timestamped after
01:00 ET (commit, file mtime, mail, unit state), else write "not resumed (no artifact)".

```bash
# 1) Nox sensor: lane rows + mail since 01:00 ET
sed -n '1,14p' ~/Work/shared-workflows/lanes/board/ops/STATUS.md
ls -lt ~/Work/shared-workflows/lanes/board/inbox/2026-11/ 2>/dev/null | head -10
# 2) this programme's own reports: who wrote after 01:00 ET, what do they claim ran
ls -lt --time-style=+%Y-%m-%dT%H:%M /home/cayleb/Documents/deepseek-harness/default-workspace/board/reports/ | head -12
# 3) remote evidence of lane work after 01:00 ET
git ls-remote https://github.com/cayleb-james2008/ergon.git refs/heads/main
git ls-remote https://github.com/cayleb-james2008/dsh-fleet.git refs/heads/main refs/heads/next
git ls-remote https://github.com/cayleb-james2008/kalshi-shadow-data.git refs/heads/main
# 4) lane units + local lanes (laptop/desk machine routes are absent from here — say so)
systemctl --user list-units --all --no-pager 2>/dev/null | grep -iE 'fleet|tripwire|kalshi|openbell' | grep -vi dsh-subprocess | cut -c1-150 | head
```
- Surfaces to report on: laptop/Claude lane (Job 04), Kalshi lane (14), fleet lane (06-11), Ergon lane,
  Nox's own board/lanes, and the money-desk lane (must stay closed unless the programme reopens it).
- A meter reset is not evidence of resumption. If the only evidence is a status word, say "not verified".

## 5. Output and disposition

Write `board/reports/monday-2026-11-02-findings.md`:
```markdown
# Monday 2026-11-02 loose-ends sweep — findings (07:45 ET run)
Sensor: STATUS.md at <time>; BOARD.md read at <time>.
| # | Finding | Evidence (command + quoted output + timestamp) | Disposition (acted / handed / waiting) | Owner | If not acted, why |
```
- **Act on the findings the same morning** within Team E's remit (read-only, proposals, reporting):
  update `board/reports/status-digest.md`, queue proposals into the Lead's ask, name successors.
- Anything outside Team E's remit → one line in the Lead's batch; never contact Cayleb.
- Then report to `lead` (≤30 lines): what resumed, what did not, the trading/feed state, the Duet/Kimi
  position, and the successors. One consolidated report — no drip-feed.
- If the 07:45 run itself cannot happen (machine down, no session), the **backup owner is `lead`** and
  the finding must say so plainly; a missed sweep is reported as missed, never backdated.

## 6. Successor chain (no idle lanes)

1. Findings file (08:30) → 2. Lead's batch (08:45) → 3. Team C's 09:05 guard (owner trading-lead) →
4. Monday's outcome check 16:20 (Team C) → 5. Next hygiene pass (**Tue 3 Nov 2026 09:00 ET**, Lead-armed every-2-days read-only sweep; manual fallback retired) → 6. Next loose-ends sweep is the following Monday.

---

## 7. N5 — Codex extension-host stop, Monday 2 Nov POST-16:20 ET slot (RULED — executes in this slot only)

**Authority:** Nox **20261101T233413Z §2** — N5 ruled and closed as a scheduled act, replacing the open dependency question (Team C's G2.5 plan established the futures desk's CDP route is its own Chromium, PID 2960519; the Codex extension-host PID 2960775 is separate and NOT a desk dependency). Method A approved earlier (Lead ruling N5 + `APPROVED.md`); owner-approved window = **Monday 2 Nov after the 16:20 ET outcome check** (the 16:00 booking carries the N5 steer; execution is after Team C's outcome check, **N5 first**).
**Status at runbook write (2026-11-01T23:44Z / 18:44 ET, read-only):** PREPARED, **NOTHING STOPPED**. `ss -ltnp` → `127.0.0.1:9222` owned by `chromium pid=2960519`; extension-host `2960775` alive with PPID **2960519**; `APPROVED.md` present in the N5 archive dir; `openbell-tabkeeper.service` active (its log-invariant is the desk route, not touched).
**Hard scope:** stop **ONLY** the extension-host. **Never** touch Chromium 2960519, `:9222`, `openbell-tabkeeper`, the Chromium profile, or any desk process/tab. No credential read. Nothing is stopped before the slot.

### 7.1 At the slot — read-only PID re-identification FIRST (Chromium may have been restarted since 1 Nov; the script's constants are 2960519 / 2960775)
```bash
date; TZ=America/New_York date '+%Y-%m-%d %H:%M %Z'
ss -ltnp | grep ':9222'
pgrep -af 'extension-host' | grep -v pgrep
```
- PID pair unchanged (`chromium 2960519` owns :9222, host `2960775` PPID 2960519) → run the approved script as-is.
- Either PID changed (Chromium restart) → copy the script, update **only** the two constants `CHROME_PID`/`HOST_PID` to the values just read, `diff` the copy, run the dry-run first, record the diff in the result file. Method A and the acceptance rules are unchanged. If there is no extension-host process at all, record that and skip the kill — do not substitute a wider kill.

### 7.2 Execute (dry-run first, then apply — the script refuses `--apply` before the 16:20 ET window by its own gate)
```bash
cd /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/team-e-lane6/n5-codex-extension-host
./stop-codex-extension-host.sh            # dry-run: prints window, preconditions, actions, acceptance
./stop-codex-extension-host.sh --apply    # moves the 2 native-host manifests to the archive, TERMs the host only
```

### 7.3 Acceptance (Nox's conditions — read-only, quote each in the result file)
```bash
ss -ltnp | grep ':9222'                    # MUST still show chromium (same PID); if the extension-host respawns as a Chromium child, note its PID and LEAVE it — no broader kill
pgrep -af 'extension-host' | grep -v pgrep # expect none (or the noted respawn PID, left alone)
journalctl --user -u openbell-live.service -n 20 --no-pager | tail -5
                                           # last feed line must remain the "no trader.tradovate.com tab" class;
                                           # must NOT become "no Chromium on CDP port 9222"
cat /home/cayleb/Work/_archive/lane6-retirement-20261101/codex-extension-host/N5-RESULT.txt
```

### 7.4 File the execution evidence in the slot (the row closes on this, not before)
Write `board/reports/team-e-n5-result-20261102.md`: timestamp; PIDs before/after; the two manifests moved + sha256 lines; the `ss :9222` read-back; the quoted `openbell-live` journal line; any respawn note; the script's own output. Then report to `lead`. **Do not mark N5 closed until those read-backs are in the file.**

### 7.5 Reversal
Move the two manifests back from the archive dir to their original `NativeMessagingHosts/` dirs, then reload the browser so the extension reconnects. Reversal line is recorded in `N5-RESULT.txt`. The stop touches nothing else, so no other reversal exists.
