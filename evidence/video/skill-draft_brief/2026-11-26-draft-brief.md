# Video Proof: `skill-draft_brief` — Draft Brief (`draft_brief`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-draft_brief/2026-11-26-draft-brief.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-draft_brief/2026-11-26-draft-brief.mp4`
- **Duration:** `24.05s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `996,991` bytes
- **SHA-256:** `123a27b9d9576b047afc01690928c84853c513ae51593295e31d08e9d97d8642`

## Inputs Used

- **Type:** Realistic non-sensitive test inputs (`q4-ops-audit.md`, `vendor-renewal-notes.md`, and standing `data/memory.md` preferences seeded into temp `ERGON_DATA_DIR`/`ERGON_WORKSPACE`).
- **Arguments:** `topic="Q4 Operational Controls"`, `source_files="q4-ops-audit.md,vendor-renewal-notes.md"`, `audience="Executive Leadership"`, `output_title="Q4 Controls Brief"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18923` with seeded source files and `memory.md`.
2. Opened Chromium and inspected `draft_brief` in Workflows.
3. Ran `draft_brief` via Typed Pipeline UI.
4. Opened `Q4 Controls Brief.md` in Workspace FileViewer verifying `[q4-ops-audit.md:L##]` / `[vendor-renewal-notes.md:L##]` line citations and standing memory preferences applied.
5. Verified `Q4 Controls Brief.md` on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18923 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located draft_brief |
| `00:12` | Configured typed pipeline step for draft_brief |
| `00:13` | Saved routine 'Draft Q4 Controls Brief'; launching live Run now |
| `00:13` | Skill draft_brief completed live (status=completed) |
| `00:16` | Opened saved output 'Q4 Controls Brief.md' in Workspace FileViewer |
| `00:20` | Closed Chromium; returning to terminal for on-disk verification |
| `00:23` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Q4 Controls Brief.md` grounded strictly in both workspace files with inline `[file:L##]` citations and standing memory rules applied.
