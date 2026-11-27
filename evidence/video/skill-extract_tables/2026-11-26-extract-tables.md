# Video Proof: `skill-extract_tables` — Extract Tables (`extract_tables`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-extract_tables/2026-11-26-extract-tables.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-extract_tables/2026-11-26-extract-tables.mp4`
- **Duration:** `27.05s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `1,083,248` bytes
- **SHA-256:** `3091baa077e8a42f6e582832702ab5cabc5b2753bb1a8c6db5058b923ab49031`

## Inputs Used

- **Type:** Realistic non-sensitive test input (`quarterly-service-report.md` containing two Markdown pipe tables separated by prose, including a quoted comma cell `North, East`).
- **Arguments:** `source_file="quarterly-service-report.md"`, `output_title="Service SLA Export"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18924`.
2. Opened Chromium and inspected `extract_tables` in Workflows.
3. Ran `extract_tables` via Typed Pipeline UI against `quarterly-service-report.md`.
4. Opened both extracted CSV spreadsheets (`Service SLA Export.csv` and `Service SLA Export - Table 2.csv`) in Workspace FileViewer.
5. Verified both RFC-4180 CSV files on disk via `sha256sum` and `head`.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18924 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located extract_tables |
| `00:11` | Configured typed pipeline step for extract_tables |
| `00:12` | Saved routine 'Extract Service SLA Tables'; launching live Run now |
| `00:13` | Skill extract_tables completed live (status=completed) |
| `00:16` | Opened saved output 'Service SLA Export.csv' in Workspace FileViewer |
| `00:20` | Opened saved output 'Service SLA Export - Table 2.csv' in Workspace FileViewer |
| `00:23` | Closed Chromium; returning to terminal for on-disk verification |
| `00:27` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Service SLA Export.csv` (properly quoting `"North, East"`) and `Service SLA Export - Table 2.csv` (release owner table).
