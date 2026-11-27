# Video Proof Sidecar — `ergon-connectors-studio`

- **Surface / App:** `ergon-connectors-studio` (Deliverable Studio Tables, Office/PDF Export, Client Vault/CRM/SOW/Invoicing, Inbound Webhooks, Watched Folders, IMAP/SMTP/OAuth2 Config, Local Stdio MCP Discovery & Gated Inbound MCP)
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy 4.5.0 / Arch Linux)
- **Result:** `PARTIAL` (Studio fixed-point query, `.xlsx` export, Client CRM/SOW, Time Tracking, Invoice PDF 1.4 / XLSX / JSON generation, Inbound Signed Webhook, Watched Folder, SMTP/OAuth2 config persistence, Local Stdio MCP discovery, and Gated Inbound MCP all **PASS**; 1 real UI defect caught uncut at `00:13` where `Save as PDF` is enabled on `.csv` files in `FileViewerModal` and returns `HTTP 400`)
- **Worktree Video Path:** `evidence/video/ergon-connectors-studio/2026-11-26-studio-vault-connectors.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-connectors-studio/2026-11-26-studio-vault-connectors.mp4`
- **Video Metadata:** `56.55s` (`00:56`), `2,781,812` bytes (`2.65 MiB`), `1440x900`, `h264`, `20 fps`, uncut single Xvfb take (`DISPLAY=:91`)
- **Video SHA-256:** `345a5db9f2aee86b050c174ad232e4ca446d1c28b9bc1bb8abd3071d5b7f28a9`

## Exact Command & Inputs

- **Cold Launch Command:**
  `ERGON_DATA_DIR=/home/cayleb/Work/scratch/ultron-ergon/run-core/connectors-studio-env/data ERGON_PORT=18913 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 /home/cayleb/Work/scratch/ultron-ergon/cargo-target/release/ergon`
- **Inputs Used:**
  - **Seeded realistic CSV test input:** `q4-field-expenses.csv` (5 rows across `Field Ops`, `Admin`, and `Project Mgmt` totaling `$3,155.50`).
  - **Realistic UI inputs:** Client `Cascade Industrial Supply`, engagement `Q4 Operations Retainer` (`$185/hr`, `$5,000` budget), SOW rule, billing contact (`ap@cascade-supply.example`), `3.5 hr` billable time entry (`$647.50`).
  - **Local loopback stdio MCP test server:** `python3 /home/cayleb/Work/scratch/ultron-ergon/run-core/local_mcp_echo.py` (speaks real JSON-RPC 2.0 `initialize` + `tools/list` over stdio; explicitly labeled local test server).
  - **Config-only mail entries:** SMTP Relay & OAuth2 saved with non-routable RFC-2606 test host (`smtp.example.invalid`, `auth.example.invalid`) and IMAP form inspected only — **no live external mail server or external OAuth2 credentials were used or mocked**.
  - Zero real secrets on screen; inbound MCP bearer token stayed in shell variable `$TOKEN` without echoing.

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous capture starts on `DISPLAY=:91`; seeds `q4-field-expenses.csv` into clean workspace and cold-launches `ergon v0.38.0` on port `18913` |
| `00:03` | Opens real headed Chromium (`1440x900`, 1:1 DPI) to `http://127.0.0.1:18913/` |
| `00:05` | Opens `q4-field-expenses.csv` in Deliverable Studio Edit view, runs fixed-point `GROUP BY Department SUM(Amount)` query (`Field Ops: 2115.75`, `Project Mgmt: 540`, `Admin: 499.75`), and saves `q4-department-rollup.csv` |
| `00:13` | **Observed UI defect (kept uncut):** Clicking `Save as PDF` (`#file-save-pdf`) on `.csv` triggers `HTTP 400 Bad Request` (`src/tools.rs:1365`: PDF export accepts `.md`/`.txt` only, yet `WorkspaceFiles.tsx:1379` renders the button enabled for `.csv`). Clicking `Save as Excel` (`#file-save-as`) succeeds and writes/downloads `q4-field-expenses.xlsx` (`3.7 KB`, valid Excel 2007+) |
| `00:16` | Creates Client `Cascade Industrial Supply` and Engagement `Q4 Operations Retainer` via Workspace Switcher |
| `00:18` | Opens CRM & SOW drawer, adds SOW rule and billing contact (`Elena Vance`, `Controller`), and saves |
| `00:21` | Switches to Today, logs `3.5 hr` billable time entry (`$647.50`), opens Invoices & Billing drawer, generates `INV-0001` (writes real PDF 1.4, XLSX, and JSON invoices on disk), and clicks `Mark Issued` |
| `00:27` | Creates target routine `Field Intake Triage` for inbound connector binding |
| `00:31` | Opens Connectors modal -> creates signed Inbound Webhook (`Field Intake Hook`) -> runs live Simulate Payload -> writes real JSON payload into `workspace/inbox/hook-*.json` |
| `00:34` | Configures local Watched Folder (`Local Intake Drop`, `*.md`) and opens IMAP Inbox (`imap_poll`) form for inspection (cancelled without external credentials) |
| `00:38` | Configures Outbound SMTP Relay + OAuth2 settings (config-only persistence with `smtp.example.invalid`; no external mail sent) |
| `00:41` | Registers local stdio MCP server (`Local Ops MCP`) and runs live `Discover Tools` over stdio JSON-RPC 2.0 (`Discovered 2 tools on "Local Ops MCP"`: `list_equipment_inventory`, `lookup_work_order`) |
| `00:45` | Closes Chromium to reveal terminal in the same uncut take: prints `q4-department-rollup.csv`, verifies `q4-field-expenses.xlsx`, lists generated invoice trio (`inv-0001-2026-11.{json,pdf,xlsx}` — PDF confirmed `PDF document, version 1.4, 1 page(s)`), lists `workspace/inbox/hook-*.json`, and invokes gated read-only Inbound MCP (`POST /api/mcp/inbound` `tools/list`) with live JSON-RPC response |
| `00:54` | Stops child `ergon` process cleanly |
| `00:55` | Capture ends cleanly |

## Bugs / Gaps Logged Honestly

1. **Defect at `00:13` (`WorkspaceFiles.tsx:1379` vs `src/tools.rs:1365`):** `FileViewerModal` renders `#file-save-pdf` (`Save as PDF`) enabled when viewing a `.csv` file, causing `POST /api/files/export` with `format: "pdf"` to fail with `HTTP 400` (`"export_file: PDF export takes a .md or .txt file, not q4-field-expenses.csv"`). Native PDF 1.4 generation itself works cleanly (verified via Invoice PDF `inv-0001-2026-11.pdf`, `4.4 KB`, `PDF document, version 1.4, 1 page(s)`).
2. **External mail/OAuth2 boundary:** Live third-party SMTPS/IMAPS/OAuth2 delivery cannot be demonstrated without external mail accounts and credentials; only UI configuration and local loopback/stdio MCP transports were demonstrated live.
