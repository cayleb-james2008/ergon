# Video Proof Sidecar — `ergon-desktop-app`

- **Surface / App:** `ergon-desktop-app` (Embedded React 19 SPA `/` + Raw-TCP HTTP/SSE server + Work Graph + FileViewer + `.history/` Diff + Audit Bundle + Settings)
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy 4.5.0 / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-desktop-app/2026-11-26-core-app-flow.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-desktop-app/2026-11-26-core-app-flow.mp4`
- **Video Metadata:** `56.55s` (`00:56`), `1,981,919` bytes (`1.89 MiB`), `1440x900`, `h264`, `20 fps`, uncut single Xvfb take (`DISPLAY=:91`)
- **Video SHA-256:** `1c4f5893957c737068bd4c5ba41d31190a3eeac5d11829431a95efd5b22fd40d`

## Exact Command & Inputs

- **Cold Launch Command:**
  `ERGON_DATA_DIR=/home/cayleb/Work/scratch/ultron-ergon/run-core/desktop-app-env/data ERGON_PORT=18912 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 /home/cayleb/Work/scratch/ultron-ergon/cargo-target/release/ergon`
- **Inputs Used:** Clean empty `DATA_DIR` with zero pre-populated state and `ERGON_NO_AUTO_MODEL=1` (honestly displays the unconfigured model banner rather than mocking an LLM). Uses Ergon's built-in deterministic **Weekly Client Digest** starter playbook (`seed_fixture: meeting-notes.md` from `src/skills/catalog.json`, explicitly labeled seeded starter fixture) and built-in **Reconcile Ledger** workflow fixture test, plus a live manual inline edit in FileViewerModal. Zero mocked API routes, zero secrets, zero browser console errors.

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous capture starts on `DISPLAY=:91`; cold-launches `ergon v0.38.0` on port `18912` with empty `DATA_DIR` and confirms `/api/version` |
| `00:03` | Opens real headed Chromium (`1440x900`, 1:1 DPI) to `http://127.0.0.1:18912/` showing cold workbench and honest unconfigured model banner |
| `00:06` | Switches to Today, previews dry-run steps, and launches **Weekly Client Digest** starter playbook (`summarize_meeting` -> `Weekly Client Digest.md` + Word `.docx` export) |
| `00:10` | Opens Workflows dialog (12 built-in workflows) and runs live `Test workflow` on `reconcile_ledger` (`PASS` status banner) |
| `00:15` | Inspects **Today** view (finished work items, activity stream, starter launchers, memory health) |
| `00:17` | Inspects **Work Graph** view (`#graph-svg` SVG provenance graph linking person, suite items, 2 finished jobs, and 2 produced files) |
| `00:20` | Opens **Command Palette** (`Ctrl+K`) and runs live full-vault search (`Weekly Client`) |
| `00:23` | Opens Right Inspector **Files** tab and opens `Weekly Client Digest.md` in **FileViewerModal** |
| `00:27` | Switches to **Edit** tab, appends a live verification note, and saves revision (creating a real `.history/` snapshot) |
| `00:29` | Clicks `SHA-256 Receipt (.md)` to export signed receipt (`ergon-receipt-weekly-client-digest-md.md`) |
| `00:32` | Switches Right Inspector to **History** tab to inspect `.history/` snapshot diff |
| `00:34` | Switches Right Inspector to **Audit** tab: verifies 2-receipt chain badge (`Verified (2)`), switches workspace policy to `Strict Client`, downloads `Bundle (.json)` (`ergon-audit-bundle-you-2026-11-27.json`), and runs `Pre-Flight Check` |
| `00:40` | Opens **Settings** modal, inspects Model/Security sections, toggles Appearance `Dark` -> `Light` (Settings-only theme switch) |
| `00:46` | Closes Chromium to reveal terminal underneath in the same continuous take: prints `Weekly Client Digest.md`, lists `.history/` and downloaded receipt + audit bundle, runs offline `ergon verify` (`PASS [audit_bundle] ... 2 receipts, 3 files`), and prints file SHA-256 sums |
| `00:54` | Stops child `ergon` process cleanly |
| `00:55` | Capture ends cleanly |

## Observed Notes

- Zero browser console or page errors (`console_errors: []`).
- Note on UI layout: `StarterPlaybooksBar` renders on the **Today** tab (not the empty Chat landing tab), so the flow switched cleanly to Today at `00:06` to launch the starter playbook.
- Downloaded artifacts verified on disk in the same uncut take:
  - `ergon-receipt-weekly-client-digest-md.md` (`660 B`)
  - `ergon-audit-bundle-you-2026-11-27.json` (`5.1 KB`, offline `ergon verify` -> `PASS`, `2 receipts, 3 files`, `sha256: 50877530c791...`)
  - `Weekly Client Digest.md` & `Weekly Client Digest.docx` in workspace
