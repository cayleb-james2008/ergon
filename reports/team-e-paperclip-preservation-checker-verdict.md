# TEAM E — PAPERCLIP / PANTHEON PRESERVATION — INDEPENDENT CHECKER VERDICT

**Checker:** fresh-context, independent (did not create, move, archive or receipt any of this material).
**Subject:** `board/probes/team-e-lane6/paperclip-preservation/RECEIPT.md` (Lane 6, 2026-11-01 09:35 ET).
**Archive under test:** `/home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon/`
**Run window:** 2026-11-01 09:36–09:53 ET (14:36–14:53Z). Every command below was executed by me; nothing is quoted from `verify-restore.log`.
**Mode:** READ-ONLY. No deletion, no move, no service action, no credential file read. Temp space used: `/tmp/checker-te.qtnh2t` (clones, extractions). Only file written: this verdict.

**Environment:** Linux (host `cayleb`), git 2.x, zstd v1.5.7, /home 1.9T total / 587G free.

---

## VERDICT TABLE

| # | Leg | Result |
|---|---|---|
| 1 | Archive present, size, `sha256sum -c` | **PASS** (25/25 OK, 0 failures, rc=0) |
| 2 | 5 bundle `verify` + fresh clone refs/HEAD/commit counts | **PASS** (5/5 verify; 80/80 branches identical SHA; counts 1/52/259/12/5 match) |
| 3 | Tracked patches `git apply --check` | **PASS** (8/8, plus 8/8 byte-identical to live diffs) |
| 4 | `zstd -t` untracked archives | **PASS** (11/11 OK, 0 corrupt — tested all 11, not a sample; skills archive OK) |
| 5 | Skills payload + manifests (80 worktrees / 16 symlinks / units / 25 hashes) | **PASS with caveat** (13/13 payloads intact on disk; archive itself carries 11/13; unit files 1/8 copied) |
| 6 | Source untouched: trees, units, processes, listeners, removal-not-started | **PASS** |
| 7 | Discrepancy register (incl. 22.1G vs 19G) | **RECORDED — delta explained** (D1) |

**OVERALL: PASS — the preservation copy is restore-verified; the removal (move, never delete) may proceed.**

No preservation leg is broken. Six receipt numbers are wrong or imprecise (D1–D6) and must be corrected before the retirement record is written; three of them (`~/.paperclip`, unit copies, 2 skill links) change *what the estate means*, not what is lost.

---

## 1. ARCHIVE + HASH — PASS

```bash
cd /home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon
du -sb .                      # 7512240681  (7.0 GiB — matches receipt's "7.0 GB")
du -sh bundles wip skills manifests logs
# 253M bundles / 6.8G wip / 140K skills / 48K manifests / 28K logs
wc -l SHA256SUMS.txt          # 25
sha256sum -c SHA256SUMS.txt   # rc=0
# OK lines: 25   FAILED lines: 0
(cd "$A" && sha256sum -c SHA256SUMS.txt.sha256)   # SHA256SUMS.txt: OK  rc=0
```
Composition of the 25 hashed artifacts: **5 bundles + 8 tracked patches + 12 `.tar.zst`** (11 untracked + 1 skills) — matches receipt §1 "25 primary artifacts".
Sidecar `SHA256SUMS.txt.sha256` = `c1308bfd…7eb418 SHA256SUMS.txt`, verifies OK.

Timestamps: 09:37:13 → 09:37:16 ET.

---

## 2. BUNDLES — PASS

`git bundle verify` needs a repo context; the first attempt inside the archive dir returned `error: need a repository to verify a bundle` (and `$?` was masked by a pipe). Re-run correctly:

```bash
WORK=$(mktemp -d /tmp/checker-te.XXXXXX); git init -q "$WORK/scratch"; cd "$WORK/scratch"
for b in "$A"/bundles/*.bundle; do git bundle verify "$b"; echo "EXIT=$?"; done
```
Result for **all five**: `… is okay` + `The bundle records a complete history` + `EXIT=0` (sha1).

Refs in bundles (branch refs only): `asteria-roblox 21`, `helikon-card-game 7`, `muse-os 20`, `project-olympus 7`, `zeus-hq 25` → **80 branches**, matching receipt §2.

Fresh clone, one per repo, then differential comparison:

```bash
git clone -q --no-checkout "$A/bundles/$r.bundle" "$WORK/restore/$r"
git bundle list-heads "$A/bundles/$r.bundle"      # refs/heads/*
git -C "$WORK/restore/$r" for-each-ref --format='%(refname:lstrip=3) %(objectname)' refs/remotes/origin
```

| repo | bundle branches | clone branches | missing | extra | SHA mismatch | objects absent | bundle HEAD | commits `--all` |
|---|---|---|---|---|---|---|---|---|
| zeus-hq | 25 | 25 | 0 | 0 | 0 | 0 | `bea1e961…` | 1 (receipt 1) |
| muse-os | 20 | 20 | 0 | 0 | 0 | 0 | `cb876890…` | 52 (receipt 52) |
| asteria-roblox | 21 | 21 | 0 | 0 | 0 | 0 | `e97e4590…` | 259 (receipt 259) |
| helikon-card-game | 7 | 7 | 0 | 0 | 0 | 0 | `0f19b7bc…` | 12 (receipt 12) |
| project-olympus | 7 | 7 | 0 | 0 | 0 | 0 | `90a28267…` | 5 (receipt 5) |

Clone HEAD SHA == bundle HEAD SHA for all five, and each also equals the `master` row of `manifests/worktrees.tsv`. Commit counts reproduce the receipt exactly (1/52/259/12/5). The only extra clone ref is `origin/HEAD` (expected, as the receipt says).
Additional: all 5 source repos report **0 remotes** (`git -C <src> remote -v`), confirming "local-only, no remotes".

Timestamps: 09:38:25 (verify), 09:40:12 → 09:40:13 (clones), 09:48:27 (ref/SHA diff).

---

## 3. TRACKED PATCHES — PASS (all 8, not a sample)

```bash
git -C "$WORK/restore/<repo>" checkout -q --force <recorded HEAD>
git -C "$WORK/restore/<repo>" apply --check "$A/wip/<repo>/<slug>/tracked.patch"
```
| repo | worktree | recorded HEAD | apply --check |
|---|---|---|---|
| asteria-roblox | `01-asteria-roblox_` | `e97e4590c6` | **OK (rc=0)** |
| asteria-roblox | `08-PAN-175-…` | `8eef14727b` | **OK (rc=0)** |
| muse-os | `01-muse-os_` | `cb876890a5` | **OK (rc=0)** |
| muse-os | `02-PAN-118-…` | `a7251fcddf` | **OK (rc=0)** |
| muse-os | `05-PAN-124-…` | `cb5048dbf3` | **OK (rc=0)** |
| muse-os | `07-PAN-165-…` | `1506f9c47c` | **OK (rc=0)** |
| muse-os | `08-PAN-251-…` | `3a19fa6490` | **OK (rc=0)** |
| muse-os | `20-PAN-92-…` | `2f550aa30e` | **OK (rc=0)** |

Stronger check: `sha256sum tracked.patch` vs `git -C <live worktree> diff HEAD --binary | sha256sum` → **IDENTICAL for 8/8** (e.g. muse-os main `9c8364d43d1e4342`, asteria main `ae051e1bae56538b`). The archived patches are bit-for-bit the live WIP state, not merely appliable.

Timestamp: 09:41:56 → 09:41:57 ET; sha comparison 09:52:44 ET.

---

## 4. UNTRACKED ARCHIVES — PASS (0 corrupt)

`zstd -t` on **all 11** (not a sample — total 7.25 GB compressed, ran in 2 s):

| archive | bytes | `zstd -t` |
|---|---|---|
| asteria-roblox/01-asteria-roblox_ | 4,868,585,077 | OK |
| asteria-roblox/08-PAN-175-… | 2,118,467,153 | OK |
| helikon-card-game/01-helikon-card-game_ | 148,783,714 | OK |
| muse-os/01-muse-os_ | 71,058,885 | OK |
| muse-os/02,05,07,08,20 (5 task worktrees) | 47,058 / 47,058 / 47,058 / 46,533 / 47,032 | OK ×5 |
| project-olympus/01-project-olympus_ | 562,143 | OK |
| zeus-hq/01-zeus-hq_ | 38,511,798 | OK |
| skills/paperclip-skills-instance.tar.zst | 140,910 | OK |

`CORRUPT_COUNT=0`. Timestamp 09:42:31 → 09:42:33 ET.

Restore test (beyond integrity), zeus-hq (smallest meaningful):
```bash
tar --zstd -C "$WORK/x/zeus" -xf "$A/wip/zeus-hq/01-zeus-hq_/untracked.tar.zst"   # rc=0
# 343 tar members, 52,702,545 bytes extracted
# recorded untracked.nul = 295 entries; 24 of them are collapsed DIRS (trailing "/")
# 72 extra files all lie under those 24 dirs; 295 - 24 + 72 = 343  -> archive is a SUPERSET
sha256sum spot-check on 3 files: src == archive (MATCH)
```

---

## 5. SKILLS PAYLOAD + MANIFESTS — PASS WITH CAVEAT

`manifests/symlinks.tsv` = **16 rows = 13 Claude skill links + 3 `~/.local/bin` links** (receipt §2 correct). All 16 targets currently resolve (`[ -e ]` → EXISTS).

`skills/paperclip-skills-instance.tar.zst` (137.6 KiB on disk ≈ receipt "140 KB") extracts to 148 files / 866,053 bytes. Per-link content comparison (file count + byte total + sha256 of the whole tree):

| link | archive | live source | result |
|---|---|---|---|
| olympus-risk-containment, checkpoint-non-replay, research, find-skills, pantheon-governance, ponytail, roblox-networking, roblox-luau, lane-scoped-mcp, roblox-datastores, diagnosing-bugs | present | present | **11/11 byte-identical** |
| **`paperclip`** | — | `runtime/node_modules/@paperclipai/server/skills/paperclip` | **not in skills archive** |
| **`paperclip-converting-plans-to-tasks`** | — | `runtime/node_modules/@paperclipai/server/skills/…` | **not in skills archive** |

**Caveat (D6):** receipt §1 says the skills tar is "the payload that the 13 Claude skill symlinks point into" — it is **11 of 13**. The other two live in the runtime tree and survive only because the tree is **moved, never deleted** (they are also re-derivable from the npm package). A restore performed from the preservation archive alone reproduces 11/13 skill links; the 2 remainder need the retirement copy. Nothing is lost under the move plan.

Manifest completeness:
- `worktrees.tsv` = **80 rows**; `wip/…/meta.tsv` = **80 files**; every row's **live HEAD == recorded SHA (80/80, 0 drift, 0 missing)**; recorded dirty pairs (`staged/untracked`) sum exactly to live `git status --porcelain` counts for all 11 dirty worktrees (e.g. asteria 5/24 → 29; PAN-175 2/58 → 60).
- `tracked-changes.tsv` = 8 rows (matches the 8 patches).
- `units.tsv` = 8 rows with enabled/active state. **Unit file copies: only `paperclip-pantheon.service.unit` + `paperclip-pantheon.service.d/{auth-url.conf,retired.conf}`** are in `manifests/` — receipt §1 ("unit copy + drop-in") is accurate, §2 ("Unit copies preserved in manifests/") overstates it for the other 7 units (D5). Those 7 remain in `~/.config/systemd/user/` and are outside the removal scope.
- 25 SHA256'd artifacts as in §1.

---

## 6. SOURCE UNTOUCHED — PASS

```bash
du -sb $HOME/.local/share/paperclip-pantheon      # 16,185,660,895  (du -sh: 16G)
du -sb $HOME/.local/share/paperclip-studio-lane   #  3,094,249,713  (du -sh: 3.0G)
```
Both trees present, owner `cayleb:cayleb`. No paperclip/pantheon path exists inside `/home/cayleb/Work/_archive/lane6-retirement-20261101/` (that dir holds only the earlier `dsh-tauri` batch, 547M, plus `RESTORE.md`) → **removal not started**. The 16 symlinks are still present and resolving.

Units (`systemctl --user is-enabled` / `is-active`):
`paperclip-pantheon disabled/inactive`; `muse-render, muse-review, yt-uploader, roblox-hub, studio-lane, art-lane, forge-local-worker` all `static/inactive` → matches `units.tsv` and receipt §5.
Timers: all `inactive/disabled` except **`forge-local-worker.timer` = active/enabled**; its `ExecStart=/usr/bin/python3 /home/cayleb/Work/studio-art/tools/forge_local.py worker`, `WorkingDirectory=/home/cayleb/Work/studio-art` — **0 references** to the paperclip tree, so it is not a writer to it.

Processes / listeners (precise, not `pgrep`):
- `for p in /proc/[0-9]*; do readlink p/{cwd,exe,root}; done` filtered on both tree paths → **matches=0**.
- No `/proc/*/cmdline` referencing the tree except my own shell. (Note: `pgrep -af paperclip` matches *any* shell whose command string contains the word — it flagged my own invocation and an unrelated fleet shell. See D8.)
- `ss -ltnp`: every listening `node` pid has a `cwd` unrelated to the tree (ergon-library-code, outis-pilot, chrome-bridge, dsh runtime, …) → **no listener on the estate**.

`remove-paperclip.sh` (read only): default DRY-RUN; `--apply` additionally requires `CHECKER-PASS.md`; it moves (never deletes) both trees, unlinks only the 16 symlinks, `daemon-reload`s, writes `REMOVAL-RECORD.txt`. **Path mismatch (D7):** the script requires `$PRES/CHECKER-PASS.md` (the *archive* dir), while receipt §7 says the marker goes in the *probe* dir. This check did **not** create either marker.

---

## 7. DISCREPANCY REGISTER

**D1 — 22.1 GB vs 19 GB: delta is EXPLAINED (corrects the maker's "unexplained").**
Measured trees: 16.19 GB + 3.09 GB = **19.28 GB** (du 16G + 3.0G). A third paperclip path exists that the receipt never mentions: **`/home/cayleb/.paperclip` = 3,250,983,090 B (du 3.1G)**, containing `instances/default/data/backups/` with **84 `paperclip-*.sql.gz` DB dumps** (2026-10-13 → 2026-10-18).
`19.28 + 3.25 = 22.53 GB ≈ inventory 22.1 GB`. So the 30 Oct inventory counted `~/.paperclip`; today's receipt and the move plan do not. **Not a loss** (nothing deletes it), but the estate is 22.5 GB, not 19 GB, and removal will leave 3.1 GB of paperclip state behind. Lead must decide: leave in place (recommended: it is outside both trees) or add to the move.
Also confirmed as stated: "measured today 19G" is **correct**.

**D2 — "6.8 GB raw → 6.8 GB stored" is imprecise.**
Stored = 7,246,203,509 B = **6.75 GiB** (so "6.8 GB" only in GiB). Raw tar-stream content = **8,301,721,600 B = 7.73 GiB** (`zstd -dc | wc -c`). Compression ratio ≈ 0.87, not ~1.0 as the receipt implies.

**D3 — per-worktree "files/GB" figures count *list entries*, not files.**
`untracked.nul` collapses untracked directories to one trailing-slash entry (git behaviour). Examples: zeus "295 files/37 MB" → archive holds **343 members / 52,992,000 B** (271 plain files); olympus "6 files/552 KB" → **~60 files / 7,546,880 B**; asteria main "1,938 files/4.6 GB" → **5,560,606,720 B**; PAN-175 "2.0 GB" → 2,374,717,440 B; helikon "142 MB" → 225,566,720 B. Archives are a **superset** of the recorded lists in every case.

**D4 — main-worktree archives duplicate nested worktrees.** They include `.paperclip/worktrees/**` (e.g. asteria main carries a 98 MB PAN-175 zip and 9 copies of a 19.6 MB `rojo` binary). Harmless (more coverage), explains the large asteria archive.

**D5 — unit copies: 1 of 8.** Only `paperclip-pantheon.service` (+2 drop-ins) has a file copy; the other 7 units are enumerated in `units.tsv` only. They live in `~/.config/systemd/user/` and are not touched by the move.

**D6 — skills tar covers 11 of 13 links** (see §5).

**D7 — `CHECKER-PASS.md` location disagrees between receipt §7 (probe dir) and `remove-paperclip.sh` line 9/32 (archive dir).**

**D8 — `pgrep` is not a trustworthy process gate.** Receipt §2 and the removal script rely on `pgrep -af "paperclipai|paperclip-pantheon"`; it matches any shell/editor mentioning the string (it matched my own checker shell and a fleet shell). The `/proc` cwd/exe/cmdline scan is the reliable test I applied: 0 live consumers.

---

## 8. WHAT THIS VERDICT DOES AND DOES NOT SAY

- Says: every artifact the removal would depend on is present, hashes clean, and **restores** — 5/5 repos to identical refs/SHAs/commit counts from fresh clones, 8/8 WIP patches byte-identical and appliable, 11/11 untracked archives compression-valid and one fully restored as a superset, 13/13 skill payloads intact on disk.
- Says: the source is exactly in the recorded state (80/80 HEADs, dirty counts, no process, no listener, no active unit, no retirement move yet).
- Does **not** certify anything inside the credential-bearing paths (`instance/`, `private/`, `state/`, `.env`-class, studio-lane `prefix/`) — those were not read or enumerated by content; the move keeps them credential-blind, which is the correct behaviour.
- Does **not** authorise `--apply`. It records PASS; the marker placement (D7) and the `~/.paperclip` decision (D1) are the Lead's.

## 9. POST-APPLY CHECKS THE LEAD SHOULD REQUIRE
1. `CHECKER-PASS.md` present at the path the script actually tests (`$PRES/CHECKER-PASS.md`).
2. Both trees present under `lane6-retirement-20261101/` with the same `du -sb` byte counts (16,185,660,895 / 3,094,249,713) and `REMOVAL-RECORD.txt` written.
3. 16 links gone, `daemon-reload` done, `~/.paperclip` (3.1 GB) still where the Lead decided it should be.
4. Re-run `sha256sum -c` on the preservation archive after the move (it should be untouched by it).

— end of verdict —
