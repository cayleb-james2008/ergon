# Video Proof: `skill-summarize_meeting` — Summarize Meeting (`summarize_meeting`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-summarize_meeting/2026-11-26-summarize-meeting.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-summarize_meeting/2026-11-26-summarize-meeting.mp4`
- **Duration:** `23.55s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `954,578` bytes
- **SHA-256:** `09d560359abb69e8ea6e7beb76e8658ce6a3a0adbf4bf746d7c09540ca8b64b5`

## Inputs Used

- **Type:** Realistic non-sensitive test input (`q4-release-sync.md` seeded into temp `ERGON_WORKSPACE`).
- **Arguments:** `notes_file="q4-release-sync.md"`, `output_title="Q4 Release Sync Recap"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18920`.
2. Opened Chromium and inspected `summarize_meeting` in Workflows.
3. Ran `summarize_meeting` via Typed Pipeline UI against `q4-release-sync.md`.
4. Opened `Q4 Release Sync Recap.md` in Workspace FileViewer displaying participants, `[L##]`-cited decisions, owner/due-date action table, open blockers, and per-owner follow-up drafts.
5. Verified `Q4 Release Sync Recap.md` on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18920 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located summarize_meeting |
| `00:11` | Configured typed pipeline step for summarize_meeting |
| `00:12` | Saved routine 'Summarize Q4 Release Sync'; launching live Run now |
| `00:13` | Skill summarize_meeting completed live (status=completed) |
| `00:16` | Opened saved output 'Q4 Release Sync Recap.md' in Workspace FileViewer |
| `00:20` | Closed Chromium; returning to terminal for on-disk verification |
| `00:23` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Q4 Release Sync Recap.md` with extracted attendees, line-cited decisions, action item owners & due dates, open questions/blockers, and follow-up drafts.
