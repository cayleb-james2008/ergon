# Video Proof Sidecar — `ergon-logging-help`

- **Surface / App:** `ergon-logging-help` (Structured rotating JSONL logger `$ERGON_DATA_DIR/logs/ergon.log`, `panic = "unwind"` crash capture, 1-click IT Diagnostic Bundle ZIP, first-run 3-step onboarding card, persistent Help & IT Support dialog, and Settings IT Admin Contact editor)
- **Base Commit SHA:** `28c3a7354` (`v0.38.0`, branch `feat/mvp-logging-crash-help`)
- **Host OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux` (Omarchy / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-logging-help/2026-11-26-logging-crash-help.mp4` (gitignored per large-file policy)
- **Deliverables Video Copy:** `ergon-video-evidence-2026-11-26/ergon-logging-help/2026-11-26-logging-crash-help.mp4` (exact SHA-256 parity verified)
- **Video Metadata:** `30.35s` (`00:30`), `649,933` bytes (`634.7 KiB`), `1440x900`, `h264`, `yuv420p`, `20 fps`, uncut single Xvfb take (`DISPLAY=:95`, `GDK_SCALE=1`, `GDK_DPI_SCALE=1`, `XDG_SESSION_TYPE=x11`, Chromium `--ozone-platform=x11 --force-device-scale-factor=1`)
- **Video SHA-256:** `d7ce28f25c76d6f1881d1ccb6acca22e5c50a13cc59fac270d59930d9d568816`

## Exact Command & Inputs

- **Cold Launch Command:**
  `ERGON_PORT=18950 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 ERGON_UPDATE_MODE=off ERGON_DISABLE_SCHEDULER=1 ERGON_ADMIN_CONTACT_NAME="Company IT Helpdesk" ERGON_ADMIN_CONTACT_EMAIL="it-support@example.org" ERGON_ADMIN_CONTACT_PHONE="+1 (555) 010-2000" ERGON_ADMIN_CONTACT_URL="https://helpdesk.example.org" ERGON_ADMIN_CONTACT_NOTES="Mon-Fri 8am-6pm; attach the IT Diagnostic Bundle ZIP when reporting an issue." ergon --no-open`
- **Inputs Used:** Completely empty fresh `ERGON_DATA_DIR` (`0` files prior to launch) with `ERGON_NO_AUTO_MODEL=1` and synthetic example IT contact values (`Company IT Helpdesk` / `it-support@example.org` via `ERGON_ADMIN_CONTACT_*` env vars). Zero mocked HTTP routes, zero secrets, zero browser console/page errors (`console_errors: []`).

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous uncut capture starts on `DISPLAY=:95` (`1440x900`); verifies `ergon 0.38.0` and empty fresh `ERGON_DATA_DIR` (`0` files), cold-launches `ergon` on loopback `127.0.0.1:18950`, and queries live `GET /api/help/config` (`onboarding_completed: false`, `admin_contact.source: "env"`) |
| `00:03` | Opens headed Chromium (`1440x900`, 1:1 DPI) to `http://127.0.0.1:18950/` showing fresh workbench, persistent `Help & support` rail item, and non-blocking **Welcome to Ergon — 3-step quick start** onboarding card |
| `00:07` | Completes first-run onboarding Step 1 (**1. Your workspace & files**) and clicks **Next** |
| `00:09` | Views Step 2 (**2. Built-in workflows & approvals**) and clicks **Next** |
| `00:11` | Views Step 3 (**3. Getting help & admin contact**, displaying live IT contact) and clicks **Got it, finish guide** (persisting `onboarding_completed: true` with toast confirmation) |
| `00:13` | Clicks persistent **Help & support** (`#help-btn`) in the left rail to open **Help & IT Support** modal displaying **Contact your admin**, **IT Diagnostic Bundle** summary, and **Common questions & troubleshooting** |
| `00:16` | Clicks 1-click **Download IT Diagnostic Bundle (.zip)** button; Chromium downloads `ergon-diagnostics-20261127T030655724Z.zip` (`1,612 B`) and UI displays SHA-256 confirmation (`2c26f5b5f335`) & redaction notice |
| `00:19` | Clicks **Edit in Settings** to open **Settings -> IT Support Contact & Diagnostics** showing the editable IT contact form and secondary 1-click diagnostic bundle button |
| `00:22` | Closes Chromium cleanly to reveal the terminal underneath in the same uncut take; verifies downloaded ZIP size (`1,612` bytes), lists all `5` ZIP entries (`README.txt`, `system.json`, `security_events.json`, `connectors.json`, `logs/ergon.log`), and prints `system.json` (`redaction_applied: true`, `uptime_secs: 15`, `log_files_included: 1`) |
| `00:25` | Prints live `$ERGON_DATA_DIR/logs/ergon.log` (structured JSONL entries for server boot & diagnostic bundle generation) and persisted `$ERGON_DATA_DIR/help_prefs.json` (`onboarding_completed: true`) |
| `00:28` | Stops child `ergon` process cleanly |
| `00:29` | Continuous capture ends cleanly |

## Observed Notes

- Zero browser console or page errors (`console_errors: []`).
- Downloaded artifact verified on disk inside the uncut take: `ergon-diagnostics-20261127T030655724Z.zip` (`1,612` bytes, `5` files, valid ZIP with `redaction_applied: true`).
- Crash capture (`panic = "unwind"` -> `$ERGON_DATA_DIR/crashes/crash-*.json`) and secret redaction (`0` leaks across `sk-live-secret-9999`, `Bearer tok-secret-8888`, `password=supersecret`, `ERGON_MODEL_API_KEY`, `ERGON_GATE_TOKEN`, `xoxb-*`, and connector secrets) are additionally proven by automated integration tests in `tests/logging_crash_help.rs` (`5/5 PASS`) and release-binary smoke verification.
