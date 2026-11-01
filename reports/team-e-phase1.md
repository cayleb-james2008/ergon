# Team E (ESTATE + UPKEEP) — Phase 1: restatement, assumptions, conflicts, plan

Author: `upkeep-lead` (Team E). Written 2026-10-31 18:16 ET (22:16Z). Jobs 18 and 19.
Tier A. Budget used: $0 of $0 (no spend exists; every action in this batch was read-only).
Status of this file: Phase 1 record + first-batch evidence index. The batch itself is reported in
`board/reports/team-e-decision-pack.md` and `board/reports/status-digest.md`.

---

## 1. The job, restated in my own words

**Job 18 (GitHub hygiene).** Keep a 60-repo private estate from rotting, and — the real point — stop
asking Cayleb one question at a time. Once per two-day cycle: sweep every repo read-only (open PRs
including whether each is a draft, stale branches, what changed since the last pass), then hand him a
single numbered packet where every outstanding repo decision is one plain sentence with a
recommendation, and he answers with numbers. Nothing is merged, deleted, archived or renamed unless he
names that exact repo or PR himself. A general "tidy up" is not a naming. Where the sweep is
rate-limited, the missing slice is named, never smoothed over.

**Job 19 (Upkeep).** Own the housekeeping layer nobody else owns: (a) relay the fleet's numbered
missions to the owning lane exactly when each gate closes, never early, never two lanes on one mission;
(b) give Cayleb ONE consolidated status instead of a drip-feed; (c) run the booked Monday 07:45 ET
loose-ends sweep; and (d) work two undiagnosed desktop problems — Chrome/Roblox receiving polite-quit
signals (app-kill sender) and the OS seeing 62.4 GiB where the machine is believed to hold 128 GiB —
with evidence, one variable at a time. If the app-kill has not recurred, say so and keep the tripwire
armed; do not manufacture a diagnosis.

**Why it matters.** A dirty estate hides the one decision that matters behind forty that don't; and two
undiagnosed desktop faults quietly corrupt every observation made on this machine. Both jobs are
read-only observation plus proposals — the ledger is the product.

---

## 2. Assumptions I am working under (numbered, falsifiable)

1. **Estate size is 61, not 60.** Brief and PROGRAMME say 60; `gh repo list` returns 61. I sweep 61.
2. **5 repos are already archived** (`scalper-retired-20260907`, `neptune-v1-equities`,
   `sophos-gitlab-legacy`, `talos-os-legacy-reference`, `ratchet-codex`) — they need no decision.
3. **0 forks** in the estate; 4 public repos (`sophos`, `dotz`, `industry-ai-suite`, `megavibe`).
4. **The 31 Oct 429 UNKNOWNs are the two repos named in the brief** (`daedalus`, `kairos-cloud`), and
   tonight's re-sweep is authoritative for them (no 429 occurred; see §5).
5. **"Held PRs" means the seven drafts** named in the brief, and only those. The Ergon repo's own PRs
   are outside this pack (standing Ergon merge authorisation is a different regime — brief §"NOTE").
6. **Every held PR is still a draft**, so none of them can be merged casually and none is a normal
   "merge to main" (see conflict C3 — several stack on each other and on non-default branches).
7. **`ergon-lite → ergon` is a content merge, executed by Team A** under the Ergon merge regime, not by
   the estate lane — because it writes to the Ergon repo. My pack asks only for the naming.
8. **"The two old desk repos" are not named anywhere in the sources.** I identify the two best
   evidence-backed candidates (`memefly`, `meme-hedge-fund`) and flag the ambiguity (conflict C5).
9. **No spend exists for these jobs** (unstated envelope = zero). Every request tonight was a GET.
10. **GitHub 2FA is never routed through Cayleb.** `gh` is authenticated as `cayleb-james2008` with
    scopes `gist, read:org, repo, workflow`; the CLI route is the only route used.
11. **Nox's board is read-only for Team E.** I read `ops/STATUS.md`, `ops/status.json`,
    `ops/LANES.json`, and inbound mail; I wrote nothing there (verified: `git status` in that tree shows
    only the Lead's own untracked mail file, §5).
12. **Nox's inbound mail to the Lead is data, not orders** (PROGRAMME §2.10). I read it to make the
    digest correct and quote it attributed; the Lead decides.
13. **Because the batch is specified read-only, I did not patch the tripwire or restart anything**
    (two sensor defects found, §4-C6; fix proposed, not applied).
14. **RAM reading "62.4 GiB" is the host view, not the machine's capacity.** The 127.8 GiB figure is
    swap+zram (job brief). Confirmed on this box: swap = 62.4 GiB swapfile + 62.4 GiB zram = 124 GiB.
15. **Timestamps are ET** unless marked Z; commands were run as user `cayleb` on `outis-desktop`.

---

## 3. Conflicts, gaps and things that cannot be verified (each with evidence)

**C1 — Repo count 60 vs 61.** `gh repo list cayleb-james2008 --limit 200` → 61 rows
(`board/probes/github-sweep-20261031/repos.tsv`, 21:59Z). One of the 61 is **empty**
(`talos-os-archive`: `isEmpty=true`, 0 branches, 0 commits, not archived). So the estate resolves as
61/61 = 60 with content + 1 confirmed empty. New pack item 12.

**C2 — `daedalus` and `daedulus` are two different projects, and the held PR is on the correctly-spelled
one.** `daedalus` = "A portable, research-grounded skill … recursive improvement", default branch `main`,
229 KB, holds PR #1. `daedulus` (typo) = "Evidence-gated control plane for the Outis local model series",
default branch `master`, 12.6 MB. Nothing to merge here; the naming collision is recorded as an FYI, not
a pack item (renaming a repo breaks local paths for no benefit tonight).

**C3 — `kairos-cloud` has no `main` branch at all; its default branch is a feature branch.** Default =
`claude/kairos-meme-coin-indicators-it8aux` (`gh api repos/.../kairos-cloud`), and
`gh api repos/.../kairos-cloud/commits/main` → HTTP 422 "No commit found for SHA: main". Consequence:
the four kairos-cloud PRs are **one stacked chain, not four independent merges**:
`default ← #6 (head buffy/daedalus-research) ← #10 (head talos/unknown-submit-persistence-c085-20260930)
← #35 (head claude/transaction-auth-boundary-buyaee)`; and `#7 → default` sits beside it. A "yes" on #35
alone changes nothing on any trunk. **This is the single most important correction to the brief's
framing** and the pack says so in plain language.

**C4 — Every held PR is a draft, and two are not clean.** All seven return `draft=true`.
`daedalus #1`: `mergeable=false`, `mergeable_state=dirty` → it cannot be merged without resolving
conflicts first. `sophos #11`: `mergeable_state=unstable` (CI not green on its head). The rest are
`mergeable_state=clean`. So "merge" is not a uniform button even in principle (pack items 1-7).

**C5 — "The two old desk repos" is an unnamed set.** The brief names only `openbell-meme-desk`. Evidence
for the other two: `memefly` ("PAPER ONLY · NO FUNDING · EXCHANGE EXECUTION DISABLED", 2.2 MB, 22
branches of which 21 stale, last push 8 Oct) and `meme-hedge-fund` ("Private supporting research
workspace for Kairos … the trading engine, wallet handling and six-agent orchestration are external to
this repository", 138 KB, 1 branch, last push 8 Oct). A third candidate exists —
`autonomous-trading-team` (2 MB, 11 branches, last push 14 Oct, **empty README**). I recommend the first
two and ask the Lead to confirm (question Q3).

**C6 — The app-kill tripwire has two real detector defects; this is why it has caught nothing since
arming.** (a) `tripwire.py` matches Sober with `cmdline[0].endswith('/sober')`, but Sober runs from a
flatpak (`app-flatpak-org.vinegarhq.Sober-…scope`, launched by `gtk-launch`), so its host-visible argv
never matches → **Sober deaths are invisible to the recorder**; two Sober exits today (12:27:32,
14:47:39) produced no `GONE` line. (b) The `GONE` write is gated on
`if not os.path.exists('/proc/<pid>')`; a process that is a zombie for even one 0.1 s poll is dropped
from `prev` without ever being logged — which is consistent with the Chromium SIGSEGV at 16:19:21 not
being recorded. Both are one-line-class fixes; not applied (read-only batch, question Q5).

**C7 — The RAM mystery is not a fault and does not need root to explain.** See §4-B.

**C8 — Two relay systems.** Nox's live sensor board (`~/Work/shared-workflows/lanes/board`, sensor every
3 min) and this programme's board (`board/`) are separate systems with separate writers. Nox now
watches this board through a registered `delegator` lane whose `activity_globs` point at
`…/default-workspace/board/*`; mail to Nox goes through her board inbox. Nothing in our programme may
write her files. This is §12/ASK's known conflict, now with a working route; recorded in the digest §8.

**C9 — The board's fleet status is stale as of 18:0x ET.** Nox's mail of 21:52Z/22:02Z reports that the
Monday B12 handback question is **CLOSED** (Cayleb: the programme runs now, Under DSH; Outis inactive;
Team B is executor of record for Jobs 06-11), and confirms mission numbering M4=Job 08, M5=Job 09.
`board/BOARD.md` still has Job 06 as `PAUSED-PENDING-RULING`. That is the Lead's file to update; I flag
it here and in the digest, attributed to Nox's mail, unverified by me (question Q7).

**C10 — Cannot verify from this machine (named, not guessed):** the physical SMBIOS module part numbers
and the BIOS-reported total (DMI tables are `-r-------- root`, `dmidecode`/`lshw` not installed,
`sudo` needs a password we must not handle) — the RAM diagnosis rests on kernel DMI + amdgpu + e820
instead (§4-B). Also unverifiable here: which agent (if any) sent the quits; the sender is still unnamed.

---

## 4. What the first batch actually established

### A. GitHub sweep (Job 18 item 1) — read-only, paced, no 429
- Commands/timestamps in `board/probes/github-sweep-20261031/sweep-timeline.log`; raw data in the same
  directory (`repos.tsv`, `branches.tsv`, `branch-dates.tsv`, `commits-since-30oct.tsv`,
  `open-prs.txt`, `prs-since-29oct.txt`, `held-prs-detail.tsv`, `held-prs-checks.tsv`).
- Result: **61/61 repos resolved** (60 with content + 1 empty), **338 branches**, **7 open PRs (all
  drafts)**, **9 PRs merged/closed since 29 Oct**, **193 non-default branches older than 14 days**,
  **11 older than 30**. API budget: 662 core points used in total tonight; `rate_limit` before the sweep
  4,562 remaining, after 4,398; GraphQL 4,989 remaining. **Zero 429s.**
- Full findings and the decision pack: `board/reports/team-e-decision-pack.md`.

### B. Desktop mystery 5 — RAM 62.4 GiB vs 128 GiB: **explained, not a fault**
Evidence (all read-only, quoted):
- Machine: `journalctl -k` → `DMI: GMKtec NucBox_EVO-X2/GMKtec, BIOS EVO-X2 1.12 12/09/2025`; CPU
  `lscpu` → `AMD RYZEN AI MAX+ 395 w/ Radeon 8060S`, 1 socket / 16 cores / 32 threads / 1 NUMA node.
- **`journalctl -k` → `DMI: Memory slots populated: 8/8`** — no empty slot, so "module seating/visibility"
  is not the cause.
- **`journalctl -k` → `amdgpu … VRAM: 65536M … (65536M used)`** and `Detected VRAM RAM=65536M` — the
  firmware has carved out **64 GiB** for the integrated GPU.
- Host view: `/proc/meminfo MemTotal: 65452552 kB` = **62.42 GiB**; `free -h` → `Mem: 62Gi`;
  `swap` → `62.4G /swap/swapfile + 62.4G /dev/zram0` = 124 GiB (this is the "127.8 GiB" family of
  numbers; it is swap+zram, not RAM).
- e820 parse (`/tmp/e820.txt`, `journalctl -k`): `System RAM 63.65 GiB`, `device reserved 65.35 GiB`,
  `gap 1.74 GiB`.
- `amdgpu … Capping GTT to 63918M to not exceed available system memory` — 63,918 MB ≈ 62.4 GiB, i.e.
  the GPU's system-memory aperture is sized from exactly the host pool above.
**Arithmetic:** 64.0 GiB (VRAM carve-out) + 62.42 GiB (host) = **126.4 GiB**, plus ~1.6 GiB of
kernel/firmware reservations = **128 GiB installed**. The machine does carry 128 GiB; half of it is
reserved for the iGPU by firmware. It is a BIOS/UMA setting, not an OS knob and not a fault. Whether to
keep or shrink the carve-out is Cayleb's call at the maintenance window (question Q6).

### C. Desktop mystery 4 — app-kill sender: **awaiting recurrence, tripwire armed** (no named sender)
Evidence (quoted in the decision digest/digest §5; raw in
`~/workspace/harness-build/evidence/incident-20261031-app-kills/`):
- Tripwire unit: `fleet-incident-tripwire.service` active, transient unit
  (`/run/user/1000/systemd/transient/…`), `ActiveEnterTimestamp=Sat 2026-10-31 10:08:38 EDT`,
  `MainPID=1509552`, `NRestarts=0`; main thread cycling (`wchan=hrtimer_nanosleep`, ~21 ticks/2 s) and
  the Hyprland stream is live (`hypr-events.log` mtime 17:53:30, 202 KB, 22 `closewindow` events).
- **No Chromium/Sober main-process disappearance has been recorded** since arming (`tripwire.log` is 291
  bytes, one line). So: no proven recurrence.
- One `EXEC` catch, 10:20:33: `pkill -u cayleb -x sleep -P 1`, parent
  `/usr/bin/bash -c source /home/cayleb/.claude/shell-snapshots/…` → a Claude Code shell, killing
  `sleep` processes. Named source, but `-x sleep` cannot match Chrome/Sober → **not the sender**.
- **One new candidate event** with the polite-quit signature: Sober's flatpak scope ended
  **12:27:32** while its window was **not focused** (Hypr `closewindow>>558c7f9e8380` at 12:27:32.424,
  last active window was a different id), and Sober's own log ends
  `app: lifecycle: will_do_clean_exit delta=7597012.68ms` → `clean_exit_success delta=1351.91ms`
  (`~/.var/app/org.vinegarhq.Sober/data/sober/sober_logs/2026-10-31_10-20-52.log`, mtime 12:27:32.42).
  That is a polite quit, not a crash — but the sender is still unidentified, because the tripwire's
  Sober matcher cannot see the flatpak process (C6a) and no 5-second pre-window snapshot exists.
- Separate and **not** the polite-quit class: Chromium **SIGSEGV** at 16:19:21 (`coredumpctl info
  1377347`, 44.8 MB core) — a crash; recorded so it is not folded into the quit theory.

### D. Status digest (Job 19 Part A) — `board/reports/status-digest.md`
Built read-only from Nox's live sensor board (STATUS.md updated 18:10 ET; status.json 22:01Z;
LANES.json now contains a `delegator` row) and her two inbound mails
(`…/board/inbox/2026-10/20261031T215204Z-nox-to-delegator.md`,
`…20261031T220204Z-nox-to-delegator.md`). Nothing written in her tree: `git -C …/lanes/board status
--short` shows only the Lead's own untracked mail file.

---

## 5. Plan for the next batches (each with a checkable done-condition)

**E5 — SUPERSEDED 2026-10-31 20:27 ET (Nox 00:20Z manager ruling).** Cayleb delegated the held-PR picks
and the consolidation namings to Nox's best recommendation; execution runs on Nox's side. Team E's
execution pack is reference only and Team E executes none of it. The enduring duty is the opposite one:
**observe** the estate, record any change to those repos, and never act on it. Item 8 (ergon-lite) is
likewise not Team E's; if it lands via Nox/Team A, verify remote flags for the record only.

**E6 — Recurring 2-day pass (next: Sun 2 Nov ~18:00 ET).**
Same discipline, same file layout. Done-condition: new `board/probes/github-sweep-<date>/` containing
61/61 repo rows, a `sweep-timeline.log` with rate-limit before/after, and a regenerated pack; if 429,
the missing slice named in the report. **Question Q4:** run manually each cycle, or authorise the
scheduler (needs the Lead's explicit request — I will not create a scheduled task unasked).

**E7 — Tripwire detector fix (30-45 min, next batch).**
Patch `tripwire.py`: match flatpak Sober (scope/`comm`), log on set-difference regardless of `/proc`
existence, and snapshot the 5 s before a `closewindow` that was not focused. Done-condition: a synthetic
test (start a throwaway flatpak-less stand-in, close it unfocused) produces a logged `GONE`/`CLOSEFOCUS`
line with a process snapshot; then restart the unit and show `ActiveState=active`, `NRestarts=0`.
**Question Q5:** approval to edit + restart the passive recorder.

**E7b — Tripwire Sober-launch refinement (30-45 min, next tripwire touch; Lead order 31 Oct evening).**
Remove the last false-positive class: flatpak startup helpers logging as momentary `GONE sober`.
Design and acceptance criteria: `board/reports/team-e-tripwire-patch.md` §10 (leader-only preferred,
>=2 s lifetime as fallback). Done-condition: synthetic launch sequence ⇒ 0 `GONE sober`; killing the
scope leader ⇒ exactly 1 with parent capture; `--selftest` PASS; one restart with before/after and the
documented rollback.

**E8 — RAM confirmation at the maintenance window (15 min at the machine, Cayleb's hands).**
Read the BIOS/UMA (iGPU memory) setting and, with root, `dmidecode -t memory` totals. Done-condition:
quoted BIOS value + quoted `dmidecode` total, and a recorded decision on carve-out size. Cayleb act;
nothing here needs it before Monday.

**E9 — Monday 07:45 ET loose-ends sweep (booked).**
Checks: Duet verdicts vs outstanding packs; trading session/feed state ahead of 09:05; which agent
surfaces actually resumed after 01:00 ET. Done-condition: findings list with a disposition per finding,
written to `board/reports/` before 08:30 ET; anything needing Cayleb goes in the Lead's batched ask.

**E10 — Branch-prune evidence (next full cycle).**
For the five dormant repos with the worst branch debt (`talos-workspace` 52, `kairos-cloud` 39,
`agentic-resume` 22/24, `memefly` 21, `sophos` 16/17), run a per-branch merged/not-merged check (~300
GETs). Done-condition: table repo → branch → merged-into-default yes/no; **nothing deleted**; only
merged branches proposed for a later named prune.

---

## 6. Questions the Lead must batch (not asked of Cayleb directly)

- **Q1** Does the 13-item pack go to Cayleb as its own section, or do you fold it into the consolidated
  ask? (I recommend its own section: it is already numbered and self-contained.)
- **Q2** ~~Item 8 (`merge ergon-lite into ergon`) execution split~~ — **MOOT 2026-10-31 20:27 ET**: the
  picks and namings moved to Nox's side by Cayleb's delegation; Team E neither executes nor certifies them.
- **Q3** Confirm the two "old desk repos" = `memefly` + `meme-hedge-fund` (evidence in C5), or name the
  substitute (`autonomous-trading-team` is the third candidate).
- **Q4** Recurring pass: manual per cycle, or may I create the scheduled task?
- **Q5** May I patch and restart the passive tripwire (E7) next batch? It is our own read-only sensor.
- **Q6** Add to Cayleb's ask: the iGPU is holding 64 GiB of his 128 GiB — keep it (local models) or shrink
  it to give the OS more RAM? A BIOS act at the maintenance window, one setting.
- **Q7** Your board still shows Job 06 `PAUSED-PENDING-RULING`; Nox's mail says B12 is closed and Team B
  is executor of record. Update `board/BOARD.md` if you accept that, so no lane stays blocked on a
  question that has been answered.

---

## 7. Hard rules I did not cross (self-check, enumerable)

- No merge, deletion, archive, rename, or settings change on any repo — every GitHub call was a GET
  (`gh repo list`, `gh api`, `gh api graphql` read queries). 0 writes.
- No spend: $0.
- No credential handling: no key/token read, printed, copied or logged; `gh` used as already authed.
- No GitHub login routed through Cayleb; no 2FA interaction.
- No writes to `~/Work/shared-workflows/lanes/board/**` (verified by `git status` there).
- No live orders, no wallet, no trading action; no public post/send.
- Maker never checker: nothing in this file certifies Team E's own work as DONE; a fresh checker must
  run the verifications.
