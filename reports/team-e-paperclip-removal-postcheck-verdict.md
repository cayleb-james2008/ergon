# TEAM E / LANE 6 — POST-APPLY INDEPENDENT VERDICT: Paperclip/Pantheon move-removal

**Verdict: PASS** — removal complete, reversible, nothing lost.
**Checker:** fresh-context, independent; preserved, moved and removed nothing it checked; READ-ONLY (wrote only this file).
**Check window:** 2026-11-01 15:07:22Z → 15:16:29Z (10:07–10:16 ET).
**Subject:** move-removal applied 2026-11-01T15:01:47Z, per `board/reports/team-e-lane6-batch3-20261101.md` and `/home/cayleb/Work/_archive/lane6-retirement-20261101/REMOVAL-RECORD.txt`.
**Host:** `/home` = `/dev/mapper/root[/@home]`, btrfs, `st_dev=30` (all paths below on one filesystem).

Read-only discipline: no `mv`/`rm`/`ln`/`systemctl start|stop|restart` was run; the only write is this file. `~/.config/paperclip-pantheon/credentials.env` was **not** read (existence noted only). `systemctl daemon-reload` was **not** re-run.

---

## Summary table

| # | Check | Result |
|---|---|---|
| 1 | Old paths absent | **PASS** |
| 2 | Archive holds both trees, byte-exact, nothing deleted | **PASS** |
| 3 | 16 archived symlinks gone; no dangling links elsewhere | **PASS** (1 stale `.desktop` residual, outside scope — see R1) |
| 4 | `~/.paperclip` untouched (3,250,983,090 B, 84 dumps) | **PASS** |
| 5 | Preservation archive re-hashes clean 25/25 | **PASS** |
| 6 | Service state + 0 live consumers | **PASS** |
| 7 | REMOVAL-RECORD + workable reversal | **PASS** |
| 8 | Unrelated live services untouched | **PASS** |

---

## 1. Old paths ABSENT — PASS

```
$ for p in /home/cayleb/.local/share/paperclip-pantheon /home/cayleb/.local/share/paperclip-studio-lane; do [ -e "$p" ] && echo "$p EXISTS" || echo "$p ABSENT"; done
/home/cayleb/.local/share/paperclip-pantheon ABSENT
/home/cayleb/.local/share/paperclip-studio-lane ABSENT

$ stat /home/cayleb/.local/share/paperclip-pantheon
stat: cannot statx '/home/cayleb/.local/share/paperclip-pantheon': No such file or directory
```
- `ls -la ~/.local/share | grep -i paperclip` → no entries.
- Also probed and ABSENT at the four other plausible old locations: `~/Documents/paperclip-*`, `~/paperclip-*`, `~/Work/paperclip-*`.
- No stray second copy anywhere: `find /home/cayleb -maxdepth 4 \( -name paperclip-pantheon -o -name paperclip-studio-lane \) -not -path '*/_archive/*'` → only `/home/cayleb/.config/paperclip-pantheon`, which is a **pre-existing 5,908 B config dir** (mtime Oct 18 02:02, contains the `RETIRED` marker) — not a tree copy, not created by the move.

## 2. Retirement archive holds both trees, byte counts EXACT — PASS

Method is the one the removing script itself used (`du -sb`, apparent size) — cross-checked three ways.

```
$ du -sb /home/cayleb/Work/_archive/lane6-retirement-20261101/paperclip-pantheon
16185660895	/home/cayleb/Work/_archive/lane6-retirement-20261101/paperclip-pantheon
$ du -sb /home/cayleb/Work/_archive/lane6-retirement-20261101/paperclip-studio-lane
3094249713	/home/cayleb/Work/_archive/lane6-retirement-20261101/paperclip-studio-lane
```

| tree | this check | recorded | delta |
|---|---|---|---|
| paperclip-pantheon | 16,185,660,895 | 16,185,660,895 | **0** |
| paperclip-studio-lane | 3,094,249,713 | 3,094,249,713 | **0** |

Contents are real, not a shell: 263,557 files / 40,353 dirs / 351 symlinks (pantheon); 24,216 files / 2,816 dirs / 59 symlinks (studio); 304,261 and 27,091 total entries.

**Nothing deleted — three independent proofs:**
1. **Byte identity** above; a single unlink would move the total.
2. **Same-filesystem rename:** source and destination are both `st_dev=30` on `/dev/mapper/root[/@home]`, so `mv` was `rename(2)` — same inodes, no copy-then-delete window. Archived tree inodes: `69095228` (pantheon), `71615222` (studio); the trees' mtimes still read Oct 14–17, i.e. inode metadata survived intact.
3. **Content identity:** all 80 worktree paths in `manifests/worktrees.tsv` resolve inside the archive (`found=80 missing=0`), and all 5 primary repo HEADs match the pre-move manifest exactly:

```
zeus-hq            manifest=bea1e961600b disk=bea1e961600b MATCH
muse-os            manifest=cb876890a532 disk=cb876890a532 MATCH
asteria-roblox     manifest=e97e4590c6cd disk=e97e4590c6cd MATCH
helikon-card-game  manifest=0f19b7bcaadc disk=0f19b7bcaadc MATCH
project-olympus    manifest=90a28267c1d3 disk=90a28267c1d3 MATCH
```
All five `.git` dirs readable; `git fsck --connectivity-only` returns only ordinary unreferenced objects, no corruption.

*Note on counting (not a defect):* a naive `find -type f -printf '%s' | awk` sum gives 16,242,921,891 / 3,905,777,548 — **higher**, because `du -sb` de-duplicates multiply-linked inodes (50 and 388 such inodes respectively). The recorded 16,185,660,895 / 3,094,249,713 are `du -sb` figures and reproduce exactly; the two methods are consistent, not contradictory.

## 3. The 16 archived symlinks are GONE — PASS

Manifest `.../paperclip-pantheon/manifests/symlinks.tsv` = 16 lines (3 `~/.local/bin` + 13 `~/.claude/skills`), matching the removal record.

```
gone=16 still_present=0
manifest entries: bin=3 skills=13 total=16
```

Sampled by name from the manifest, all absent: `~/.local/bin/{pantheon-ctl,yt-uploader,studio-lane}`; `~/.claude/skills/{paperclip, olympus-risk-containment--f8e6f01002, checkpoint-non-replay--eab6c194ca, research--24af9e9fc3, find-skills--ee7fac7d87, pantheon-governance--2a501d884b, ponytail--70f5f3482b, paperclip-converting-plans-to-tasks, roblox-networking--1ecae7a0b9, roblox-luau--83627f8f26, lane-scoped-mcp--c5d203c480, roblox-datastores--1c57db29a0, diagnosing-bugs--3cf67b7b88}` — 13/13 `gone`.

Dangling sweep (not just `-e`, used `-xtype l`):
```
~/.local/bin  dangling=0    ~/.claude/skills dangling=0
~/.local/share/applications dangling=0   ~/.config/autostart dangling=0
~/.config/systemd/user dangling=0
```
No symlink anywhere under `~/.local ~/.config ~/.claude ~/Documents` still targets `paperclip-pantheon`/`paperclip-studio-lane`. Pre-existing dangling links found elsewhere are unrelated (2,173 flatpak repo objects, chromium/Codex `SingletonLock`, wine `dosdevices` in the studio prefix, and 12 `~/.dsh/profiles/tauri/*` links left by the **earlier** dsh-tauri removal and already disclosed in that archive's `RESTORE.md`) — none removed or created by this move.

**R1 (residual, outside authorised scope):** `~/.local/share/applications/paperclip-pantheon.desktop` (655 B, mtime Oct 15 13:55) survives and is now a **dead launcher** — `Exec=/home/cayleb/.local/share/paperclip-pantheon/bin/pantheon-desktop` MISSING, `Icon=.../paperclip-pantheon.svg` MISSING. It is a menu entry that silently does nothing. Not a live consumer, no data at risk, trivially reversible. The 16-link scope never included `.desktop` files, so this is a **plan-scope residue, not an execution failure**. Recommend either deleting it or moving it into the retirement archive on the next pass.

## 4. `~/.paperclip` UNTOUCHED — PASS

```
$ du -sb /home/cayleb/.paperclip
3250983090	/home/cayleb/.paperclip
$ find /home/cayleb/.paperclip -name 'paperclip-*.sql.gz' -type f | wc -l
84
```
Matches the D1 decision byte-for-byte (3,250,983,090 B, 84 `paperclip-*.sql.gz` dumps). `instances/` intact; top-level mtime Oct 13 20:06 (pre-move). Left in place deliberately, not an oversight — recorded in `DECISIONS.md` §1.

## 5. Preservation archive still re-hashes clean — PASS

```
$ cd /home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon && sha256sum -c SHA256SUMS.txt
... 25 lines ...
skills/paperclip-skills-instance.tar.zst: OK
wip/zeus-hq/01-zeus-hq_/untracked.tar.zst: OK
=== exit: 0 ===    (real 0m3.547s)
```
25/25 `OK`, zero failures, exit 0. Includes 5 bundles + skills tar + 19 WIP patch/tarball pairs. This is a genuinely independent re-hash (the archive was never touched by my session); it confirms the `mv` 14 minutes earlier did not disturb the preservation copy — and that the script's pre-move gate (§1 of the apply log, `PASS sha256sum -c (25/25)`) was real.

## 6. Services + live consumers — PASS

```
paperclip-pantheon.service   enabled=disabled   active=inactive   (drop-ins auth-url.conf, retired.conf; inactive (dead))

muse-render.service          static / inactive      muse-review.service   static / inactive
yt-uploader.service          static / inactive      roblox-hub.service    static / inactive
studio-lane.service          static / inactive      art-lane.service      static / inactive
forge-local-worker.service   static / inactive
```
7 dependent units static/inactive — matches `manifests/units.tsv` (8 rows incl. main).

`forge-local-worker.timer`: `enabled=enabled active=active` (OnCalendar `*:0/10`) **and 0 references to the moved tree** — its `.service` runs `/usr/bin/python3 /home/cayleb/Work/studio-art/tools/forge_local.py worker` with `WorkingDirectory=/home/cayleb/Work/studio-art`; `grep` for `paperclip-pantheon|paperclip-studio-lane` in `forge-local-worker.*` → `0`. The timer stays useful; the worker never touched the trees.

Additional hardening found: every retired unit carries a `ConditionPathExists=!%h/.config/paperclip-pantheon/RETIRED` drop-in and that marker is present (Oct 18 02:02), so these units are double-gated (disabled *and* condition-failed).

**No live consumer** — `/proc` scan of **654** pids over `cwd`, `exe`, `root` and `cmdline` (the D8 method, not `pgrep`):
```
CMDLINE pid=135748 cmd=bash -c echo "=== /proc cwd/exe/root/cmdline scan for old tree paths ===" ...
TOTAL HITS: 1
```
The single hit is **the scanning shell's own command string** (its script text contains the search patterns) — self-referential, not a consumer. Zero hits in `cwd`/`exe`/`root`. This independently reproduces the apply log's `0 consumers`.

Also clear of old paths: user `crontab` (none), `~/.config/autostart` (none), shell rc files (none).

## 7. REMOVAL-RECORD.txt + reversal — PASS

`/home/cayleb/Work/_archive/lane6-retirement-20261101/REMOVAL-RECORD.txt` exists (624 B, mtime 10:01) and names: the timestamp `2026-11-01T15:01:47Z`, actor `cayleb (Lane 6)`, authority (checker PASS + Lead go), marker path, archive path, both moved byte counts, `symlinks unlinked: 16 / 16`, the kept-in-place decision (`~/.paperclip 3.1G`), and the reversal (mv back + relink from `symlinks.tsv`). Every field is corroborated by my own independent measurements above.

**Reversal is genuinely workable — tested, not assumed:** for all 16 manifest rows, the recorded link *target* exists inside the archive at the corresponding path (targets are absolute `/home/cayleb/.local/share/paperclip-pantheon/...`, which the `mv`-back restores):
```
relink targets resolvable in archive: ok=16 bad=0
```
So `mv` back + `relink from symlinks.tsv` restores 16/16 links — I verified the payload each link would point at, not merely that a link name was recorded. The 2 runtime-based skills (`paperclip` 10 files, `paperclip-converting-plans-to-tasks` 1 file) are present **inside the archived tree** and return with it; the skills tar covers the other 11/13 (248 entries). Since the move was a same-device rename, `mv` back is a single atomic `rename` — fast and lossless even at 19 GB.

## 8. Unrelated live services untouched — PASS

```
user   chrome-bridge            enabled / active
user   openbell-kalshi-poll     enabled / active
user   outis-control            enabled / active
system fleet-dsh                disabled / active   (active (running) since 2026-10-31 23:46:27 EDT; 11h ago)
```
All four running. **Correction to my own first pass:** `fleet-dsh` is a **system** unit (`/etc/systemd/system/fleet-dsh.service`), so a `systemctl --user` query reports `not-found` — that was my scope error, not a dead service. Confirmed alive at system scope with its process (`node /var/lib/fleet/npm/bin/dsh --profile fleet --port 3091`, uptime 11h25m). The batch-3 report's "all active" claim holds.

---

## Residual observations (non-blocking, no verdict impact)

- **R1** — `~/.local/share/applications/paperclip-pantheon.desktop`: dead "Paperclip" launcher (Exec + Icon both into the moved tree). Outside the 16-link scope; cosmetic; recommend removing or archiving it.
- **R2** — one symlink *inside* the archive dangles while archived by design: `paperclip-pantheon/_retired-20261015/bin/pantheon-connect-go.symlink -> /home/cayleb/.local/share/paperclip-pantheon/bin/pantheon-connect-go` (self-referential absolute path from an earlier retirement). Resolves again on `mv` back; harmless where it sits.
- **R3** — disk free 586 G before and after this check (`df -h /home`: `used=1.3T free=586G`); the 19.28 GB moved on the same filesystem, so no space was consumed by the move and none is at risk.

None of R1–R3 contradicts the report. R1 is a scope gap in the *plan*, not a failure of the *move*.

## Overall verdict: PASS

Complete — both trees gone from original paths and present in the retirement archive at **byte-identical** counts (16,185,660,895 / 3,094,249,713, delta 0), with 80/80 worktrees and 5/5 HEADs matching the pre-move manifest.
Reversible — same-device rename (inodes preserved), record complete, and all 16 relink targets verified present in the archive.
Nothing lost — no `rm` of any tree in the script (`mv` only; `rm -f` only on symlinks, targets preserved), preservation archive independently re-hashed 25/25 OK, `~/.paperclip` untouched at 3,250,983,090 B / 84 dumps, no live consumer, services quiet, unrelated services running.

**PASS is warranted. No failing leg.** Follow-up, optional: retire the stale `paperclip-pantheon.desktop` launcher (R1).

---

### Reproduce (all read-only)
```bash
R=/home/cayleb/Work/_archive/lane6-retirement-20261101
P=/home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon
du -sb "$R/paperclip-pantheon" "$R/paperclip-studio-lane"       # 16185660895 / 3094249713
ls /home/cayleb/.local/share/paperclip-pantheon                 # No such file
cd "$P" && sha256sum -c SHA256SUMS.txt                           # 25/25 OK
du -sb /home/cayleb/.paperclip; find /home/cayleb/.paperclip -name 'paperclip-*.sql.gz' | wc -l
systemctl --user is-enabled paperclip-pantheon; systemctl --user is-active paperclip-pantheon
for p in /proc/[0-9]*; do for f in cwd exe root; do readlink "$p/$f"; done; tr '\0' ' ' < "$p/cmdline"; done | grep paperclip
```
