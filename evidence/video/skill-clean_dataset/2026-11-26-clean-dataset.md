# Video Proof: `skill-clean_dataset` — Clean Dataset (`clean_dataset`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-clean_dataset/2026-11-26-clean-dataset.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-clean_dataset/2026-11-26-clean-dataset.mp4`
- **Duration:** `27.55s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `1,106,124` bytes
- **SHA-256:** `e6a211292ea9f2bbae578c1d96caff4cb688d568a513c14e998f5bb7693b2c94`

## Inputs Used

- **Type:** Realistic non-sensitive test input (`q4-vendor-orders.csv` with 11 rows: mixed-case headers, duplicate `ORD-204`, `N/A`/`none`/`null` tokens, 1 ragged row, accounting credit `($45.00)`, and outlier `$14,850.00`).
- **Arguments:** `input_file="q4-vendor-orders.csv"`, `key_column="Order ID"`, `output_title="Q4 Cleaned Orders"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18921` with isolated workspace containing `q4-vendor-orders.csv`.
2. Opened Chromium and inspected `clean_dataset` in the Workflows dialog.
3. Created and ran a typed pipeline step for `clean_dataset`.
4. Opened both `Q4 Cleaned Orders.csv` (normalized `snake_case` interactive table) and `Q4 Cleaned Orders - Diff Report.md` (deduplication, ragged-row repair, null normalization, and `|z| >= 2.5` outlier metrics) in Workspace FileViewer.
5. Verified both saved artifacts on disk in the cold-launch terminal.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18921 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located clean_dataset |
| `00:11` | Configured typed pipeline step for clean_dataset |
| `00:13` | Saved routine 'Clean Q4 Vendor Orders'; launching live Run now |
| `00:13` | Skill clean_dataset completed live (status=completed) |
| `00:16` | Opened saved output 'Q4 Cleaned Orders.csv' in Workspace FileViewer |
| `00:20` | Opened saved output 'Q4 Cleaned Orders - Diff Report.md' in Workspace FileViewer |
| `00:24` | Closed Chromium; returning to terminal for on-disk verification |
| `00:27` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Q4 Cleaned Orders.csv` (deduplicated snake_case CSV) and `Q4 Cleaned Orders - Diff Report.md` (1 duplicate removed, ragged row repaired, null tokens standardized, `$14,850.00` outlier flagged).
