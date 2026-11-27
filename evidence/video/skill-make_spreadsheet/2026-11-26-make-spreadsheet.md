# Video Proof: `skill-make_spreadsheet` — Make Spreadsheet (`make_spreadsheet`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-make_spreadsheet/2026-11-26-make-spreadsheet.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-make_spreadsheet/2026-11-26-make-spreadsheet.mp4`
- **Duration:** `20.05s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `543,549` bytes
- **SHA-256:** `0b9085b11f24ea9e46029e6cb09815df6140c7fd3364c7f685dcfb74c5d976e2`

## Inputs Used

- **Type:** Realistic non-sensitive test input passed via Ergon's live `POST /api/pipelines/run` API (matching `catalog.json` schema with 2D `rows` array including embedded commas and quotes for RFC-4180 escaping).
- **Columns:** `["Item", "Department", "Region", "Monthly_USD"]`; **Rows:** 5 realistic infrastructure line items.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18925`.
2. Opened Chromium -> Workflows dialog -> filtered `make_spreadsheet` -> clicked `Test workflow` (live UI sandbox verification passed).
3. Executed `make_spreadsheet` live against the active workspace via `POST /api/pipelines/run` with 5 structured budget rows (including embedded comma `Cloud Compute, Primary` and escaped quotes `Object Storage "Archive"`).
4. Opened `Q4 Infrastructure Budget.csv` in Ergon's Workspace FileViewer interactive table.
5. Verified RFC-4180 CSV bytes and SHA-256 on disk in the terminal.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18925 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located make_spreadsheet |
| `00:07` | Ran Workflows dialog sandbox test for make_spreadsheet |
| `00:09` | Launched live make_spreadsheet pipeline via POST /api/pipelines/run (prun-1795745966742-0) |
| `00:09` | make_spreadsheet completed live; opening saved CSV in Workspace FileViewer |
| `00:13` | Opened saved output 'Q4 Infrastructure Budget.csv' in Workspace FileViewer |
| `00:16` | Closed Chromium; returning to terminal for on-disk verification |
| `00:19` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Q4 Infrastructure Budget.csv` with all 5 data rows and exact RFC-4180 quoting (`"Cloud Compute, Primary"`, `"Object Storage ""Archive"""`). Note: Pipeline Builder UI currently stores string-only args and cannot type 2D `rows` arrays directly in the form.

## Notes

- UI Pipeline Builder gap observed in source (`frontend/src/shell/PipelineBuilder.tsx:958` + `src/skills.rs:1205`): PipelineBuilder inputs emit string values, whereas `arg_string_rows` expects a JSON 2D array for `rows`. Therefore this recording demonstrates the UI Workflows sandbox test (`POST /api/skills/test`) AND executes `make_spreadsheet` live against the running server via `POST /api/pipelines/run` with its documented 2D `rows` JSON array, then opens the real saved CSV in Ergon's interactive CSV viewer.
