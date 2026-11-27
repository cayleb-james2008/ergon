# Video Proof: `skill-compare_documents` — Compare Documents (`compare_documents`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-compare_documents/2026-11-26-compare-documents.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-compare_documents/2026-11-26-compare-documents.mp4`
- **Duration:** `23.55s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `945,654` bytes
- **SHA-256:** `e3f0c6419a613e34bd5439cdbb8338d33fa832d1afc0803a6521348de7c80d91`

## Inputs Used

- **Type:** Realistic non-sensitive test inputs (`travel-policy-v1.md` and `travel-policy-v2.md` seeded into temp `ERGON_WORKSPACE`).
- **Arguments:** `left_file="travel-policy-v1.md"`, `right_file="travel-policy-v2.md"`, `output_title="Travel Policy Diff"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18922`.
2. Opened Chromium and located `compare_documents` in Workflows.
3. Ran `compare_documents` via Typed Pipeline UI against `travel-policy-v1.md` and `travel-policy-v2.md`.
4. Opened `Travel Policy Diff.md` in Workspace FileViewer displaying line counts, similarity %, added `## International Travel` heading, and `[L##]`/`[R##]` context hunks.
5. Verified `Travel Policy Diff.md` on disk via `sha256sum` and `head`.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18922 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located compare_documents |
| `00:11` | Configured typed pipeline step for compare_documents |
| `00:12` | Saved routine 'Compare Travel Policy Drafts'; launching live Run now |
| `00:13` | Skill compare_documents completed live (status=completed) |
| `00:16` | Opened saved output 'Travel Policy Diff.md' in Workspace FileViewer |
| `00:20` | Closed Chromium; returning to terminal for on-disk verification |
| `00:23` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Travel Policy Diff.md` with exact added/removed/unchanged line counts, similarity percentage, heading change summary, and unified `[L##]`/`[R##]` diff hunks.
