# Video Proof: `skill-update_memory` — Update Memory (`update_memory`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-update_memory/2026-11-26-update-memory.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-update_memory/2026-11-26-update-memory.mp4`
- **Duration:** `21.55s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `648,363` bytes
- **SHA-256:** `d541154278fad8cd7cd874c8b2cd20cf320a31498c2192367980d633f71f2282`

## Inputs Used

- **Type:** Realistic non-sensitive standing preference text.
- **Arguments:** `note="Always format currency totals in USD rounded to two decimals and place summary figures first."`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18922` with fresh empty `memory.md`.
2. Opened 'What my agent has learned' (`#memory-btn`) confirming empty baseline memory.
3. Inspected `update_memory` in Workflows and ran it live via Typed Pipeline UI.
4. Re-opened 'What my agent has learned' confirming count updated to `1`, Healthy badge intact, and exact rule rendered.
5. Closed Chromium and verified `$ERGON_DATA_DIR/memory.md` contents and SHA-256 on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18922 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened 'What my agent has learned' showing baseline empty memory (0 rules) |
| `00:08` | Opened Workflows dialog and located update_memory |
| `00:13` | Configured typed pipeline step for update_memory |
| `00:14` | Saved routine 'Record Standing Finance Rule'; launching live Run now |
| `00:14` | Skill update_memory completed live (status=completed) |
| `00:15` | Re-opened 'What my agent has learned' showing persisted standing rule + Healthy status |
| `00:17` | Closed Chromium; returning to terminal for on-disk verification |
| `00:21` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Appended `- Always format currency totals in USD rounded to two decimals and place summary figures first.` to `$ERGON_DATA_DIR/memory.md`, verified live in both the Ergon Memory UI panel and on disk.
