# Video Proof: `skill-audit_contract` — Audit Contract (`audit_contract`)

- **Repository:** `cayleb-james2008/ergon`
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux (Omarchy 4.5.0 / Arch Linux)`
- **Result:** **PASS**
- **Capture Mode:** Single continuous, unedited screen recording (`Xvfb :92 1440x900x24`, `ffmpeg x11grab 20fps H.264 yuv420p`)
- **Worktree Video Path:** `evidence/video/skill-audit_contract/2026-11-26-audit-contract.mp4`
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-audit_contract/2026-11-26-audit-contract.mp4`
- **Duration:** `23.55s`
- **Resolution / Codec:** `1440x900`, `h264` (`yuv420p`)
- **File Size:** `918,172` bytes
- **SHA-256:** `cb77186ebb7565173c79adf1bdfcbba8b9af04c8f388ab7d81b36c38bb0d54fe`

## Inputs Used

- **Type:** Realistic non-sensitive test input (`northwind-saas-msa.md`, seeded into clean temp `ERGON_WORKSPACE`).
- **Arguments:** `contract_file="northwind-saas-msa.md"`, `required_clauses="termination,liability,confidentiality,data protection,renewal,governing law"`, `output_title="Northwind MSA Audit"`.

## Steps Demonstrated (Uncut)

1. Cold-launched `ergon` (`v0.38.0`) on `127.0.0.1:18920` with isolated `ERGON_DATA_DIR` and `ERGON_WORKSPACE`.
2. Opened real Chromium on `DISPLAY=:92` to the live Ergon web workbench.
3. Opened Workflows dialog and located `audit_contract`.
4. Created and ran a typed deterministic pipeline step executing `audit_contract` against `northwind-saas-msa.md`.
5. Opened the generated `Northwind MSA Audit.md` in Ergon's Workspace FileViewer showing `[L##]`-cited HIGH/MEDIUM risks and missing clauses (Confidentiality, Data Protection).
6. Closed Chromium and verified `Northwind MSA Audit.md` on disk with `ls -la`, `sha256sum`, and `head`.

## Key Timestamps (`MM:SS`)

| Timestamp | Event |
|---|---|
| `00:00` | Recording started (Xvfb :92 1440x900 @ 20fps) |
| `00:01` | Cold launch ready on http://127.0.0.1:18920 (shown in terminal) |
| `00:04` | Opened real Chromium to live Ergon UI |
| `00:06` | Opened Workflows dialog and located audit_contract |
| `00:11` | Configured typed pipeline step for audit_contract |
| `00:12` | Saved routine 'Audit Northwind MSA'; launching live Run now |
| `00:13` | Skill audit_contract completed live (status=completed) |
| `00:16` | Opened saved output 'Northwind MSA Audit.md' in Workspace FileViewer |
| `00:20` | Closed Chromium; returning to terminal for on-disk verification |
| `00:23` | On-disk artifact verification complete & Ergon stopped |

## Verifiable Output Observed

Saved `Northwind MSA Audit.md` flagging missing clauses (`confidentiality`, `data protection`) and `[L##]`-cited risks (90-day auto-renewal lock-in, unilateral amendment, unlimited liability, Net 90 payment terms).
