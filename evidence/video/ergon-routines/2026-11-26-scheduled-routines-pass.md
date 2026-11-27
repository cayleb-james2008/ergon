# Video Proof Sidecar — `ergon-routines` (`PASS` Re-Record)

- **Surface / App:** `ergon-routines` — Scheduled Routines, Zero-LLM Background Auto-Execution & Single Catch-Up Verification (MVP Gap 4)
- **Repository:** `cayleb-james2008/ergon`
- **Baseline Tag / Branch:** `v0.38.0` (`fix/mvp-routines-scheduler-catchup`, base `28c3a7354`)
- **Host OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux` (Omarchy / Arch Linux)
- **Result:** **`PASS`** (replaces prior uncut `FAIL` recorded against `28c3a7354`)
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :96 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`, `GDK_SCALE=1`, `GDK_DPI_SCALE=1`, `XDG_SESSION_TYPE=x11`, `--ozone-platform=x11 --force-device-scale-factor=1`)
- **Worktree Video Path:** `evidence/video/ergon-routines/2026-11-26-scheduled-routines-pass.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `~/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-routines/2026-11-26-scheduled-routines-pass.mp4`
- **Video Metadata:** `35.60s` (`00:35`), `892,982` bytes (`872 KiB`), `1440x900`, `h264`, `yuv420p`, `20 fps`
- **Video SHA-256:** `a98e07763a4cd81c488fe33aa2bce6632c32fd06b1a4c1dc6ba81b4bc749f589`

## Exact Commands & Inputs

- **Cold Launch Command:**
  `ERGON_NO_AUTO_MODEL=1 ERGON_SCHEDULER_TICK_MS=5000 NO_OPEN=1 ERGON_DATA_DIR=./data ERGON_WORKSPACE=./workspace ergon --port 18962 --no-open`
- **Status Verification Shown Live:**
  `curl -s http://127.0.0.1:18962/api/status | jq '{version, model_configured, scheduler_enabled}'` -> `{"version": "0.38.0", "model_configured": false, "scheduler_enabled": true}`
- **Inputs Used (Seeded vs. Live):**
  - **Seeded workspace inputs (non-sensitive):** `daily-invoices.csv` (4 realistic vendor invoice rows with 1 duplicate reference and 1 missing receipt) and `meeting-notes.md` (weekly ops sync notes).
  - **Seeded paused overdue routine:** `Catch-Up Ledger Check` (`every 15m`, `enabled: false`, `created_ms` backdated 35 minutes, deterministic `steps_v2` running `reconcile_ledger` -> `Catchup Ledger Audit.md`) so clicking **Resume** live in the UI immediately surfaces `2 missed` and the **Catch up** button.
  - **Live UI actions:** Launched Starter Playbook `Weekly Client Digest`, created recurring zero-LLM deterministic routine `Every-8s Ledger Check` (`every 8s`, `reconcile_ledger` -> `Scheduled Ledger Audit.md`) directly via the Routines Typed Pipeline Composer, waited uncut for automatic background execution with NO model API key configured, triggered single manual **Catch up** on `Catch-Up Ledger Check`, expanded Run history on both routines, and opened both saved Markdown outputs in the Workspace FileViewer. Zero mocked API routes, zero secrets, zero browser console errors (`console_errors: []`, `page_errors: []`).

## Steps Shown & Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Continuous uncut capture starts (`Xvfb :96`, `1440x900 @ 20fps`, 1:1 DPI); cold launch of `ergon` on `http://127.0.0.1:18962` with `ERGON_NO_AUTO_MODEL=1` and `ERGON_SCHEDULER_TICK_MS=5000`; `/api/status` confirms `model_configured: false`, `scheduler_enabled: true` |
| `00:04` | Opens real headed Chromium (`1440x900`, `--ozone-platform=x11 --force-device-scale-factor=1`) to live Ergon UI showing unconfigured model banner (`Setup needed`) |
| `00:06` | Opens **Scheduled routines** panel displaying `Scheduler active (5s tick)` pill |
| `00:07` | Previews Starter Playbook **Weekly Client Digest** (dry-run steps + `Reversible via .history/` badge) |
| `00:08` | Launches Starter Playbook **Weekly Client Digest** -> saves `Weekly Client Digest.md` and `Weekly Client Digest.docx` (`1 run`, `Verified`) |
| `00:11` | Creates recurring zero-LLM deterministic routine **Every-8s Ledger Check** (`every 8s`, `reconcile_ledger` -> `Scheduled Ledger Audit.md`) via the UI Typed Pipeline Composer and waits uncut without clicking anything |
| `00:20` | **Fix #1 verified live:** Background scheduler tick automatically fires **Every-8s Ledger Check** with NO model key configured (`model_configured: false`) -> SSE live refresh updates card to **`1 run`** and green **`Verified`** SHA-256 receipt badge (`0 missed`) |
| `00:21` | Pauses `Every-8s Ledger Check` and clicks **Resume** on seeded overdue **Catch-Up Ledger Check** -> immediately displays **`2 missed`** badge and **Catch up** button |
| `00:22` | **Fix #2 verified live:** Clicks **Catch up** once -> clears both missed windows, updates card to **EXACTLY `1 run`** (no double-counting) and green **`Verified`** badge (no false `Drift`) |
| `00:24` | Expands **Execution history** on **Catch-Up Ledger Check** (`1 recorded`, `Ok`, linked SHA-256 receipt `rcpt-...`) |
| `00:26` | Expands **Execution history** on **Every-8s Ledger Check** (`1 recorded`, `Ok`, `scheduled`, linked SHA-256 receipt) |
| `00:28` | Closes Routines modal, opens Workspace Inspector, and opens real saved **`Scheduled Ledger Audit.md`** in FileViewer (`4` line items, `$2,018.25` combined total, `1` duplicate reference, `1` missing receipt) |
| `00:30` | Opens real saved **`Catchup Ledger Audit.md`** in Workspace FileViewer |
| `00:32` | Closes Chromium to reveal terminal underneath in the same uncut take; prints live `GET /api/routines` JSON (`run_count: 1`, `history_count: 1`, `missed_windows: 0`, `last_run_verified: "verified"` for both routines) and `ls -1 ./workspace` on-disk deliverables, confirms `0` browser console/page errors, and stops `ergon` cleanly |
| `00:35` | Continuous uncut capture ends cleanly |

## Verifiable Output Observed

- **Fix #1 (Zero-LLM Scheduled Auto-Execution):** `Every-8s Ledger Check` fired automatically on the background scheduler tick with `ERGON_NO_AUTO_MODEL=1` (`model_configured: false`), wrote `Scheduled Ledger Audit.md` (`1,249` bytes) and `Scheduled Ledger Audit - Summary.csv` (`211` bytes), recorded `run_count: 1` and `history_count: 1` (`trigger: "scheduled"`), and verified cleanly (`last_run_verified: "verified"`).
- **Fix #2 (Single Catch-Up Count & Receipt Verification):** Clicking **Catch up** once on overdue `Catch-Up Ledger Check` (`2 missed`) wrote `Catchup Ledger Audit.md` (`1,247` bytes) and `Catchup Ledger Audit - Summary.csv` (`211` bytes), cleared `missed_windows` to `0`, recorded **EXACTLY `1` run** (`run_count: 1`, `history_count: 1`, `trigger: "catchup"`), and verified cleanly (`last_run_verified: "verified"`, zero false `Drift`).
- **Browser Health:** `0` console errors, `0` uncaught page errors.
