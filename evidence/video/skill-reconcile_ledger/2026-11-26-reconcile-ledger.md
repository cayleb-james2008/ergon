# Video Proof: `skill-reconcile_ledger` — Reconcile Ledger (`reconcile_ledger`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-reconcile_ledger/2026-11-26-reconcile-ledger.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-reconcile_ledger/2026-11-26-reconcile-ledger.mp4`
- **Duration:** `27.05s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `1,118,828` bytes
- **SHA-256:** `de78c870d8bc29ac45e0a0d10622f7c82d13e986f62a9da673a2a3488d61de4c`

## Inputs Used

- **Type:** Realistic non-sensitive test inputs (`ap-invoices.csv` and `bank-settlement.csv` seeded into temp `ERGON_WORKSPACE`, featuring 1 duplicate `INV-4012`, 2 missing receipts, 1 credit memo `($85.00)`, and 1 cross-file amount mismatch on `INV-4013`: `$985.25` vs `$999.00`).
- **Arguments:** `source_files="ap-invoices.csv,bank-settlement.csv"`, `output_title="Q4 Ledger Reconciliation"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` on `127.0.0.1:18926`.
2. Opened Chromium and inspected `reconcile_ledger` in Workflows.
3. Ran `reconcile_ledger` via Typed Pipeline UI across both CSV ledgers.
4. Opened `Q4 Ledger Reconciliation.md` (penny-exact totals, duplicate citation, missing receipts, cross-file mismatch) and `Q4 Ledger Reconciliation - Summary.csv` (vendor & category rollups) in Workspace FileViewer.
5. Verified both saved artifacts on disk.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18926 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located reconcile_ledger |
| `00:11` | Configured typed pipeline step for reconcile_ledger |
| `00:12` | Saved routine 'Reconcile Q4 AP vs Bank'; launching live Run now |
| `00:13` | Skill reconcile_ledger completed live (status=completed) |
| `00:16` | Opened saved output 'Q4 Ledger Reconciliation.md' in Workspace FileViewer |
| `00:19` | Opened saved output 'Q4 Ledger Reconciliation - Summary.csv' in Workspace FileViewer |
| `00:23` | Closed Chromium; returning to terminal for on-disk verification |
| `00:26` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Q4 Ledger Reconciliation.md` and `Q4 Ledger Reconciliation - Summary.csv` with penny-exact totals, duplicate reference flags, missing receipt flags, accounting negative anomaly, and cross-file mismatch detection.
