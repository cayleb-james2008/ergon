# Video Proof Sidecar — `ergon-backup-restore`

- **Surface / App:** `ergon-backup-restore` (Gap 2: Automated Full-Instance Backup, Offsite Replication, `ergon verify`, Catastrophic `DATA_DIR` Wipe, Atomic Verified `ergon restore`, and Settings UI Backups & Recovery)
- **Baseline Tag / Branch:** `v0.38.0` (`feat/mvp-backup-restore`, base `28c3a7354`)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-backup-restore/2026-11-26-backup-restore.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `~/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-backup-restore/2026-11-26-backup-restore.mp4`
- **Video Metadata:** `55.40s` (`00:55`), `1,916,561` bytes (`1.83 MiB`), `1440x900`, `h264`, `yuv420p`, `20 fps`, uncut single Xvfb take (`DISPLAY=:94`, `GDK_SCALE=1`, `GDK_DPI_SCALE=1`, `XDG_SESSION_TYPE=x11`)
- **Video SHA-256:** `0d96ae6cb54f42650c16bec3839ae7ee902de8b295e1e443ee730b464f209fa9`

## Exact Commands & Inputs

- **Cold Launch Command:**
  `ERGON_DATA_DIR=./demo-data ERGON_PORT=18941 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 ergon --no-open`
- **Verify, Wipe, Restore & Re-Verify Commands Shown Live:**
  - `ergon verify ./offsite-vault/ergon-backup-20261127-030235-330Z.zip`
  - `rm -rf ./demo-data && ls -la ./demo-data`
  - `ergon restore ./offsite-vault/ergon-backup-20261127-030235-330Z.zip --data ./demo-data`
  - `ergon verify ./offsite-vault/ergon-backup-20261127-030235-330Z.zip --data ./demo-data`
- **Inputs Used:** Clean empty `./demo-data` with `ERGON_NO_AUTO_MODEL=1`. Created real workspace artifacts via the built-in **Weekly Client Digest** starter playbook (`seed_fixture: meeting-notes.md`, explicitly labeled seeded fixture -> produces `Weekly Client Digest.md`, `Weekly Client Digest.docx`, routine, and 2 SHA-256 Field-Book receipts) plus a live manual FileViewer edit appending `## Pre-Wipe Disaster Recovery Marker` (creating a `.history/` revision). Configured Backups & Recovery (`Scheduled`, `12h`, keep `7`, offsite mounted folder `./offsite-vault`) and triggered **Run Backup Now** from Settings UI. Zero mocked API routes, zero secrets, zero browser console errors (`console_errors: []`).

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous uncut capture starts on `DISPLAY=:94` (`1440x900`, 1:1 DPI); cold-launches `ergon 0.38.0` on loopback port `18941` with empty `./demo-data` and confirms `/api/version` |
| `00:03` | Opens real headed Chromium (`1440x900`, `--ozone-platform=x11 --force-device-scale-factor=1`) to `http://127.0.0.1:18941/` |
| `00:06` | Switches to **Today** and launches **Weekly Client Digest** starter playbook producing `Weekly Client Digest.md`, `Weekly Client Digest.docx`, scheduled routine, and chained SHA-256 receipts |
| `00:09` | Opens Right Inspector **Files** -> `Weekly Client Digest.md` in FileViewer -> edits inline to append `## Pre-Wipe Disaster Recovery Marker` and saves revision (creating `.history/` snapshot) |
| `00:15` | Opens **Settings** -> **Backups & Recovery**, configures schedule (`Scheduled`, `12h`, keep `7`, offsite `Mounted folder` -> `./offsite-vault`), saves config, and clicks **Run Backup Now** -> displays live verified snapshot `ergon-backup-20261127-030235-330Z.zip` (`14 files`, `41.3 KB`, `sha256 f69d02e95a0b`, offsite copy verified) with toast |
| `00:23` | Closes Chromium to reveal terminal underneath in the same uncut take: lists `./demo-data/backups` and `./offsite-vault` (`.zip` + `.sha256` sidecars) and runs offline `ergon verify` -> `PASS [backup_archive]` (`14 files`, `181115 bytes`) |
| `00:25` | Stops running Ergon server via PID and **visibly wipes `./demo-data` completely** (`rm -rf ./demo-data` -> `ls: cannot access './demo-data': No such file or directory`) |
| `00:28` | Runs `ergon restore` from `./offsite-vault/ergon-backup-20261127-030235-330Z.zip` into `./demo-data` -> `PASS [restore] restored 14 files (181115 bytes) into ./demo-data (verified on disk)` |
| `00:30` | Runs `ergon verify "$BACKUP" --data ./demo-data` -> `PASS [backup_archive]` confirming every restored file on disk matches manifest SHA-256, and prints restored `## Pre-Wipe Disaster Recovery Marker` from `Weekly Client Digest.md` |
| `00:33` | Restarts `ergon` on loopback port `18941` pointing at the restored `./demo-data` |
| `00:36` | Reopens Chromium to `http://127.0.0.1:18941/`, opens `Weekly Client Digest.md` showing exact restored content, `Pre-Wipe Disaster Recovery Marker` section pill, and `History & Diff 1` revision |
| `00:48` | Opens Right Inspector **Audit** tab confirming the SHA-256 receipt chain (`Verified (2)`) survived the full wipe and restore intact |
| `00:50` | Opens **Settings** -> **Backups & Recovery** confirming the restored schedule (`Every 12h`, `12` hours, keep `7`, `./offsite-vault`) survived intact |
| `00:53` | Closes Chromium, stops restarted server cleanly via PID, and prints final `PASS` confirmation |
| `00:55` | Continuous uncut capture ends cleanly |

## Observed Notes

- Zero browser console or page errors (`console_errors: []`).
- Verified snapshot produced during the recording:
  - Archive: `ergon-backup-20261127-030235-330Z.zip` (`41.3 KB` compressed, `181,115` bytes raw across `14` files)
  - Archive SHA-256: `f69d02e95a0be9cf9240fce576c7340e800fc84427ef5fe3a3a7b92a2e78dfe5`
  - Manifest SHA-256: `c01e751367915f6e1f31ffa6e6af62d83921b062292152da114cbdb91d2ff168`
- Both local (`./demo-data/backups`) and offsite (`./offsite-vault`) copies matched bit-for-bit, and complete post-wipe recovery restored all workspace deliverables, `.history/` revisions, scheduled routines, `backup_config.json`, and the verified 2-entry Field-Book receipt chain.
