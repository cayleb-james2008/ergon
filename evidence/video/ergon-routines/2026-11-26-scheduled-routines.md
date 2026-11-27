# Video Proof: `ergon-routines` — Scheduled Routines & Starter Playbooks (`ergon-routines`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **FAIL**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/ergon-routines/2026-11-26-scheduled-routines.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-routines/2026-11-26-scheduled-routines.mp4`
- **Duration:** `41.05s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `1,158,535` bytes
- **SHA-256:** `ab94f4d26bc5575581295744fde7af08ff746bad24c003e0c97120756f6090ab`

## Inputs Used

- **Type:** Realistic non-sensitive test inputs (`daily-invoices.csv` and `meeting-notes.md` seeded into clean temp `ERGON_WORKSPACE`), cold-launched with `ERGON_SCHEDULER_TICK_MS=2000`, `ERGON_NO_AUTO_MODEL=1`, and no external LLM API key.
- **Routines Tested:** Built-in starter playbook `Weekly Client Digest` (`summarize_meeting` -> `.docx` export) and custom scheduled routine `Every-10s Ledger Check` (`every 10s`, deterministic `reconcile_ledger`).

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18928` with `ERGON_SCHEDULER_TICK_MS=2000` and no API key.
2. Opened Scheduled Routines panel showing `Scheduler active (2s tick)` pill.
3. Previewed and launched Starter Playbook `Weekly Client Digest` -> saved `Weekly Client Digest.md` and `Weekly Client Digest.docx`.
4. Created recurring deterministic routine `Every-10s Ledger Check` (`every 10s`), tested Pause/Resume toggle and zero-side-effect Dry-Run Preview.
5. Waited 12.5 seconds live (uncut) to verify automatic scheduled execution: observed `FAIL` bug where background scheduler skips even deterministic zero-LLM `steps_v2` routines when `!eff.model_configured()` (`src/server.rs:1029`), surfacing `1 missed`.
6. Triggered `Catch up` on the overdue routine, expanded Execution History showing the recorded run, opened `Scheduled Ledger Audit.md` in Workspace FileViewer, and verified all saved files on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18928 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Routines panel (status pill: 'Scheduler active (2s tick)') |
| `00:07` | Previewed Starter Playbook 'Weekly Client Digest' (dry-run steps + .history/ badge) |
| `00:08` | Launched Starter Playbook 'Weekly Client Digest' -> saved .md and .docx |
| `00:10` | Created scheduled routine 'Every-10s Ledger Check' (cadence: every 10s, deterministic reconcile_ledger) |
| `00:11` | Toggled routine to Paused |
| `00:12` | Resumed routine to Active |
| `00:12` | Opened zero-side-effect Dry-run Preview on scheduled routine |
| `00:13` | Waiting 12s live to verify background scheduler auto-execution (ERGON_SCHEDULER_TICK_MS=2000) |
| `00:27` | BUG OBSERVED: Background scheduler did NOT auto-fire deterministic routine without API key (1 missed) |
| `00:28` | Clicked 'Catch up' on overdue routine -> deterministic skill executed and cleared missed window |
| `00:29` | Expanded Execution history drawer showing recorded routine run |
| `00:34` | Opened saved output 'Scheduled Ledger Audit.md' in Workspace FileViewer |
| `00:37` | Closed Chromium; returning to terminal for on-disk verification |
| `00:40` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Starter Playbook launch, Pause/Resume, Dry-Run Preview, Catch-Up execution, Run History, and workspace deliverables (`Weekly Client Digest.md`, `Weekly Client Digest.docx`, `Scheduled Ledger Audit.md`) all worked; **marked `FAIL` honestly** because automatic background scheduling silently failed to fire a deterministic zero-LLM `steps_v2` routine while the UI claimed `Scheduler active (2s tick)` (`src/server.rs:1029`).

## Bugs / Blockers Observed

- Background scheduler did NOT auto-fire deterministic zero-LLM `steps_v2` routine (`every 10s`) after 12.5s even though UI displayed `Scheduler active (2s tick)`. Root cause: `src/server.rs:1029` (`start_scheduler_thread`) skips all routines when `!eff.model_configured()`, leaving the routine overdue (`1 missed`) instead of executing deterministic skills.
- Secondary bug observed at `00:29`–`00:30` after clicking `Catch up`: the single catch-up run incremented `Every-10s Ledger Check` to `2 runs` (double-counted because `routines::record_run` is called both inside `src/pipelines.rs:2297` and `src/server.rs:531`) and marked its receipt verification badge `Drift` instead of `Verified` (`Execution history` shows `1 recorded`).
