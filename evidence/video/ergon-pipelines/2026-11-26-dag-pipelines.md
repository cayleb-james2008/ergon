# Video Proof: `ergon-pipelines` — Multi-Step DAG Pipelines (`ergon-pipelines`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/ergon-pipelines/2026-11-26-dag-pipelines.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-pipelines/2026-11-26-dag-pipelines.mp4`
- **Duration:** `24.75s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `1,219,309` bytes
- **SHA-256:** `957fdb759ed45e5c1241794e3cd3f976ac5293274f53c1ca51f8e8a2c3f7184f`

## Inputs Used

- **Type:** Realistic non-sensitive test inputs (`q4-invoices.csv`, `q4-sync-notes.md`, `ops-standup.md`, and standing `memory.md`).
- **DAG Topology (5 steps, 4 stages):** Stage 1 parallel fan-out (`reconcile` + 2-item `for_each` batch `summarize_batch`) -> Stage 2 join (`human_gate` Human Checkpoint with `on_all_ok`) -> Stage 3 (`draft_brief` grounded in reconciled ledger) -> Stage 4 (`export` Word `.docx`).

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18929` with isolated workspace and zero LLM key.
2. Opened Routines -> New routine -> Typed Pipeline Composer (`#pipeline-builder`) and authored the 5-step Diamond DAG with `depends_on`, `for_each` batch fan-out, Human Checkpoint, `draft_brief`, and `.docx` export.
3. Clicked `Validate DAG` showing live server-verified topological stages.
4. Saved and launched the pipeline (`Run now`).
5. Observed parallel Stage 1 completion (`reconcile` + 2-item `summarize_batch`) and live pause at `human_gate` (`Waiting on you`).
6. Selected `Approve with audit note` in the CheckpointCard UI and clicked `Resume Job`, completing `draft_brief` and Word `.docx` export.
7. Opened `DAG Executive Signoff.md` in Workspace FileViewer and verified `.md` + `.docx` (Zip/OOXML) outputs on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18929 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:10` | Validated 5-step Diamond DAG + for_each in UI (DAG valid — Stages: Stage 1: reconcile + summarize_batch -> Stage 2: human_gate -> Stage 3: brief -> Stage 4: export_docx) |
| `00:11` | Saved DAG pipeline routine; clicking Run now |
| `00:12` | Stage 1 parallel fan-out + 2-item for_each completed; paused at human_gate (prun-1795746229120-0) |
| `00:12` | Displayed live Human Checkpoint card showing completed parallel steps & batch children |
| `00:14` | Selected 'Approve with audit note' and clicked Resume Job |
| `00:14` | Resumed DAG completed Stage 3 (draft_brief) + Stage 4 (.docx export) -> status=completed |
| `00:17` | Opened saved output 'DAG Executive Signoff.md' in Workspace FileViewer |
| `00:21` | Closed Chromium; returning to terminal for on-disk verification |
| `00:24` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Completed 5-step zero-LLM Diamond DAG (`depends_on`, `for_each`, checkpoint pause/resume, `.docx` export), saving `DAG Ledger Audit.md`, batch summaries, `DAG Executive Signoff.md`, and `DAG Executive Signoff.docx`.
