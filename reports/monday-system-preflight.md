# MONDAY SYSTEM PREFLIGHT — read-only checks run 2026-10-31 22:33 ET (Sun 02:33Z)

Purpose: catch environmental problems tonight rather than at Monday's 07:45/09:05 bookings. Nothing was changed except /tmp scratch
cleanup (recorded below). No service restarted, no config touched, no spend.

## Disk / memory
- root/home: 1.9T, 68% used, **598G available**. Kalshi capture is ~116 MB/day (2026-10-31 folder 116M; capture total 465M) => months
  of headroom; the Tue-Wed freeze and Monday capture are not disk-constrained.
- /tmp: was 93% inodes / 53% space (inode pressure from tonight's build trees). Cleaned programme scratch entries
  (`ergon-*`, `team-*`, `fc-*` older than 2026-10-31 20:30 ET; 5,356 paths, no process cwd/open-fd under them) =>
  **inodes 89% -> 65% (374k free), space 53% -> 42%**. Active cwds in /tmp (`/tmp/.wine-1000/...`, `/tmp/outis-e2e-...`,
  `/tmp/claude-1000/...`) were verified and left untouched.
- memory: 62 GiB total, 26 used, **35 available** — no memory pressure for Monday's suites.

## Remote state (verified live, not from memory)
- `ergon` main = **686fca8** (Pack 20 + Job 02 + J2 follow-ups, checker-proofed). No drift.
- `dsh-fleet` origin main = **4a8a1d3**; `integration/0.2.2` = **cf93107** (certified tree). Local clone main = abfb842 (the struck
  local-only docs commit; correctly NOT pushed and not in any remote ref).
- `openbell` master = **6f582f8** (cutover target). No drift.

## Services / armed jobs
- `fleet-dsh.service` active; `fleet-incident-tripwire.service` active (v2.2, parent capture); Kalshi user units
  poll/ref/shadow = active/active/active; futures engine (pid 1484) + dash (pid 1481) running.
- DST runner alive (pid 3542392, elapsed 2h10m) waiting on 05:50Z then 07:15Z; DST artefact dir ready.
- Monday futures runbook v6 sha256 = **027ca4e4…** — still exactly the P0.1 signed hash (signature not void).

## Scheduled deliveries in this session (6)
01:00 ET Kalshi resume + laptop-lane check; 07:45 ET loose-ends sweep; 09:05 ET futures guard+preflight; 16:05 ET flat+outcome;
DST verification ping Sun 02:25 ET (07:25Z); GitHub sweep every 2 days (first Mon 18:00 ET).

## Residual risks for Monday (named, not hidden)
- Release-identity install still blocked on the owner's Wait (fleet gate cannot close).
- Feed: "no trader.tradovate.com tab" is expected on a weekend; Monday 09:35-09:44 binding bar proof must actually run.
- Laptop lane: not reachable from this desktop; if the laptop session does not resume at 01:00, Job 04 is reported plainly as
  not resumed.
