# Video Proof: `skill-write_document` — Write Document (`write_document`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-write_document/2026-11-26-write-document.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-write_document/2026-11-26-write-document.mp4`
- **Duration:** `23.55s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `860,723` bytes
- **SHA-256:** `6e28a0e992cf3756177afc08b945ce163efcb0ea31efdc19706e3515aa723d60`

## Inputs Used

- **Type:** Realistic non-sensitive structured Markdown sections.
- **Arguments:** `title="Q4 Readiness Report"`, `sections` (Executive Summary, Verification Metrics, Next Week Priorities).

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18923`.
2. Opened Chromium and inspected `write_document` in Workflows.
3. Ran `write_document` via Typed Pipeline UI.
4. Opened `Q4 Readiness Report.md` in Workspace FileViewer displaying rendered Markdown headings and section blocks.
5. Verified `Q4 Readiness Report.md` bytes and SHA-256 on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18923 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located write_document |
| `00:11` | Configured typed pipeline step for write_document |
| `00:12` | Saved routine 'Write Q4 Readiness Report'; launching live Run now |
| `00:13` | Skill write_document completed live (status=completed) |
| `00:16` | Opened saved output 'Q4 Readiness Report.md' in Workspace FileViewer |
| `00:20` | Closed Chromium; returning to terminal for on-disk verification |
| `00:23` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Q4 Readiness Report.md` with formatted `# Q4 Readiness Report` heading and structured Markdown sections.

## Notes

- UI Pipeline Builder note: `PipelineBuilder.tsx` captures `sections` in a single-line text input and `src/skills.rs:1191` splits string inputs on commas, so comma-free Markdown headings and section blocks were entered directly in the UI.
