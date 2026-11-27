# Ergon Video Proof Inventory (`INDEX.md`)

- **Repository:** `cayleb-james2008/ergon`
- **Target Commit:** `28c3a7354` (`v0.38.0`, branch `main`)
- **Evidence Branch:** `evidence/video-proof-2026-11-26`
- **Host OS:** Linux `outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC x86_64` (Omarchy 4.5.0 / Arch Linux)
- **Linked Video Deliverables Folder:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/`
- **Large Video Storage Policy:** `git-lfs` is not installed on this host. Raw `.mp4` files are excluded from git via `.gitignore` (`evidence/video/**/*.mp4`), stored both in the local worktree (`evidence/video/<surface>/`) and mirrored to the linked deliverables folder above, and verified by exact SHA-256 digest, duration, byte size, and sidecar `.md` logs committed in this branch.
- **Inventory Status:** Initial pre-recording inventory committed BEFORE any video recording takes place. Statuses, video links, SHA-256 digests, and timestamps will be populated strictly from real, continuous, unedited screen recordings.

---

## Recording Standard

- Every video is a **single continuous, uncut screen capture** (`Xvfb 1440x900x24` + `ffmpeg -f x11grab -framerate 20 -c:v libx264 -preset veryfast -crf 24 -pix_fmt yuv420p`) showing cold launch through real interaction to a verifiable saved output on screen.
- Zero cuts, zero speedups, zero hidden loading/errors/retries.
- Realistic, clean, non-sensitive inputs only (labeled explicitly as realistic test inputs or seeded fixtures; external APIs are never mocked and passed off as live).
- Failed or blocked flows are kept uncut and marked `FAIL` or `BLOCKED` with exact timestamps and root causes.

---

## Complete Surface & App Inventory

### 1. Core Application, CLI, Installer & Companion Surfaces (Lane 1A — `inventory-video`)

| # | Surface / App ID | Component & Code Paths | Category | Recording Plan (Display `:91`, Ports `18910-18919`) | Status | Video / Sidecar | Linked Deliverables Path | SHA-256 |
|---|---|---|---|---|---|---|---|---|
| 1 | `ergon-installer-cli` | `install.sh`, `src/main.rs` (`--version`, `--help`, `--no-open`, `ergon-serve`, `export-handoff`, `verify`) | CLI & Installer | Fresh install into isolated temp `HOME`/`PREFIX`/`DATA_DIR` (`--no-service`), cold CLI launch, status check, `export-handoff`, `verify` | `PENDING` | `evidence/video/ergon-installer-cli/2026-11-26-install-and-cli.md` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-installer-cli/2026-11-26-install-and-cli.mp4` | `TBD` |
| 2 | `ergon-desktop-app` | Embedded SPA `/` (`assets/index.html`, `assets/chunks/*`, `src/server.rs`, `src/sse.rs`, `src/ui.rs`, `src/file_history.rs`, `src/receipts.rs`, `src/audit_bundle.rs`, `src/search.rs`, `src/tripwire.rs`) | Desktop Web App | Cold launch of `ergon` -> Chromium UI -> seed/upload workspace file & run starter playbook/workflow -> inspect Today, Work Graph, FileViewer, History & Diff, Audit Bundle/Policy, Command Palette search, Settings (Light/Dark + Security + Update) -> verifiable saved file & receipt on screen | `PENDING` | `evidence/video/ergon-desktop-app/2026-11-26-core-app-flow.md` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-desktop-app/2026-11-26-core-app-flow.mp4` | `TBD` |
| 3 | `ergon-connectors-studio` | Studio Tables (`src/tables.rs`), Office/PDF Export (`src/docs.rs`), Client Vault & Invoicing (`src/server.rs`, `src/records.rs`), Connectors (`src/connectors.rs`), MCP (`src/mcp.rs`) | Studio, Vault, Billing & Connectors | Cold launch -> Studio CSV inline edit & `/api/tables/query` rollup -> PDF 1.4 & XLSX export -> Client & Engagement creation, time entry, invoice PDF/JSON generation -> Connectors modal (inbound webhook + watched folder + SMTP/IMAP config & local MCP server discovery/inbound JSON-RPC) -> verifiable saved outputs on screen | `PENDING` | `evidence/video/ergon-connectors-studio/2026-11-26-studio-vault-connectors.md` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-connectors-studio/2026-11-26-studio-vault-connectors.mp4` | `TBD` |
| 4 | `ergon-phone-and-web-standalone` | Mobile responsive route `/phone` (`src/ui.rs`) and standalone offline single-file HTML desk (`frontend/dist/ergon-web.html`, `frontend/src/lib/webEngine.ts`) | Mobile & Offline Web Surfaces | Cold launch `/phone` at mobile viewport + open standalone `frontend/dist/ergon-web.html` locally in Chromium -> run offline workflow & verify saved/downloaded result | `PENDING` | `evidence/video/ergon-phone-and-web-standalone/2026-11-26-phone-and-standalone.md` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-phone-and-web-standalone/2026-11-26-phone-and-standalone.mp4` | `TBD` |
| 5 | `ergon-frontend-prototype` | Sibling standalone prototype outside this git repo (`/home/cayleb/Work/projects/ergon-frontend`, `server.mjs` + `dist/`, last modified 2026-11-17/18, targeting local Ollama `qwen2.5:3b`) | Companion Prototype (Out-of-Repo) | Inspect and document live cold launch behavior (static desk shell serves via Node `server.mjs`; live Ollama agent route requires Ollama on `127.0.0.1:11434`, which is not running on this host) | `PENDING` | `evidence/video/ergon-frontend-prototype/2026-11-26-companion-prototype.md` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-frontend-prototype/2026-11-26-companion-prototype.mp4` | `TBD` |

### 2. All 12 Built-In Workflows / Skills (Lane 1B — `skill-recorder`)

| # | Surface / Skill ID | Script & Spec | Deterministic | Outputs | Recording Plan (Display `:92`, Ports `18920-18929`) | Status | Video / Sidecar | Linked Deliverables Path | SHA-256 |
|---|---|---|---|---|---|---|---|---|---|
| 6 | `skill-write_document` | `src/skills/write_document.rn` | Yes | `markdown` | Cold launch -> run `write_document` with realistic sections -> open saved `.md` on screen | `PENDING` | `evidence/video/skill-write_document/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-write_document/` | `TBD` |
| 7 | `skill-make_spreadsheet` | `src/skills/make_spreadsheet.rn` | Yes | `csv` | Cold launch -> run `make_spreadsheet` with realistic rows -> open saved `.csv` on screen | `PENDING` | `evidence/video/skill-make_spreadsheet/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-make_spreadsheet/` | `TBD` |
| 8 | `skill-research` | `src/skills/research.rn` | No (HTTP fetch) | `markdown` | Cold launch -> run `research` against reachable URL / record honest behavior & saved output or fetch block | `PENDING` | `evidence/video/skill-research/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-research/` | `TBD` |
| 9 | `skill-update_memory` | `src/skills/update_memory.rn` | Yes | `memory` (`memory.md`) | Cold launch -> run `update_memory` -> verify updated `memory.md` & memory health on screen | `PENDING` | `evidence/video/skill-update_memory/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-update_memory/` | `TBD` |
| 10 | `skill-reconcile_ledger` | `src/skills/reconcile_ledger.rn` | Yes | `markdown`, `csv` | Cold launch -> run `reconcile_ledger` on realistic invoice/expense CSVs -> open reconciliation `.md` & `.csv` | `PENDING` | `evidence/video/skill-reconcile_ledger/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-reconcile_ledger/` | `TBD` |
| 11 | `skill-summarize_meeting` | `src/skills/summarize_meeting.rn` | Yes | `markdown` | Cold launch -> run `summarize_meeting` on realistic meeting notes -> open summary `.md` | `PENDING` | `evidence/video/skill-summarize_meeting/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-summarize_meeting/` | `TBD` |
| 12 | `skill-audit_contract` | `src/skills/audit_contract.rn` | Yes | `markdown` | Cold launch -> run `audit_contract` on realistic vendor MSA -> open `[L##]` risk table `.md` | `PENDING` | `evidence/video/skill-audit_contract/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-audit_contract/` | `TBD` |
| 13 | `skill-triage_inbox` | `src/skills/triage_inbox.rn` | Yes | `markdown` | Cold launch -> run `triage_inbox` on realistic inbox export -> open Urgent/Action/FYI triage `.md` | `PENDING` | `evidence/video/skill-triage_inbox/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-triage_inbox/` | `TBD` |
| 14 | `skill-clean_dataset` | `src/skills/clean_dataset.rn` | Yes | `csv`, `markdown` | Cold launch -> run `clean_dataset` on messy CSV with duplicates/outliers -> open cleaned `.csv` & diff report `.md` | `PENDING` | `evidence/video/skill-clean_dataset/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-clean_dataset/` | `TBD` |
| 15 | `skill-compare_documents` | `src/skills/compare_documents.rn` | Yes | `markdown` | Cold launch -> run `compare_documents` on two policy drafts -> open `[L##]`/`[R##]` diff `.md` | `PENDING` | `evidence/video/skill-compare_documents/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-compare_documents/` | `TBD` |
| 16 | `skill-draft_brief` | `src/skills/draft_brief.rn` | Yes | `markdown` | Cold launch -> run `draft_brief` grounded in workspace files + `memory.md` -> open cited brief `.md` | `PENDING` | `evidence/video/skill-draft_brief/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-draft_brief/` | `TBD` |
| 17 | `skill-extract_tables` | `src/skills/extract_tables.rn` | Yes | `csv` | Cold launch -> run `extract_tables` on structured Markdown/log input -> open extracted `.csv` | `PENDING` | `evidence/video/skill-extract_tables/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/skill-extract_tables/` | `TBD` |

### 3. Scheduled Routines, Playbooks & DAG Pipelines (Lane 1B — `skill-recorder`)

| # | Surface ID | Component & Code Paths | Category | Recording Plan (Display `:92`, Ports `18920-18929`) | Status | Video / Sidecar | Linked Deliverables Path | SHA-256 |
|---|---|---|---|---|---|---|---|---|
| 18 | `ergon-routines` | `src/routines.rs`, `src/scheduler.rs`, `RoutinesPanel.tsx`, `StarterPlaybooksBar.tsx` | Scheduler & Playbooks | Cold launch -> create/preview/run scheduled routine & starter playbook -> trigger catch-up & inspect run history + saved deliverable | `PENDING` | `evidence/video/ergon-routines/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-routines/` | `TBD` |
| 19 | `ergon-pipelines` | `src/pipelines.rs`, `PipelineBuilder.tsx`, `CheckpointCard.tsx`, `LiveRunTimeline.tsx` | Hybrid & DAG Pipelines | Cold launch -> validate & run multi-step DAG pipeline (`depends_on`, `for_each`, checkpoint pause/resume) -> inspect saved outputs & receipts | `PENDING` | `evidence/video/ergon-pipelines/` | `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-pipelines/` | `TBD` |

### 4. Backend Subsystems & External-Dependency Surfaces (Audited & Tracked)

| # | Subsystem / Surface | Code Paths | Live Demonstration Scope / Constraint | Status |
|---|---|---|---|---|
| 20 | Embedded HTTP/SSE Server & Gate Header | `src/server.rs`, `src/sse.rs` | Exercised live in every app recording (`/`, `/phone`, `/chunks/*`, `/api/*`, `/api/runs/stream`). | `PENDING` |
| 21 | Tripwire, Sandbox & Security Controls | `src/tripwire.rs`, `src/tools.rs`, `SecuritySection.tsx` | Inspected live in Settings Security section & `ergon export-handoff` leak pre-check. | `PENDING` |
| 22 | Self-Update Checker & Binary Swap | `src/update.rs`, `Update.tsx` | Live status check (`GET /api/update`) shown in UI/CLI; **live remote binary download/swap is `BLOCKED`** because HEAD `28c3a7354` is already the latest `v0.38.0` release (no newer published release asset exists on GitHub). | `BLOCKED (partial)` |
| 23 | Live Paid / Remote LLM API Chat | `src/lib.rs` (`/api/chat`, `ERGON_MODEL_API_KEY`) | **Live external LLM completion is `BLOCKED`** in clean isolated sessions without external provider credentials (never mocked as live). Unconfigured honest `HTTP 503` refusal behavior will be shown transparently. | `BLOCKED` |
| 24 | Live Third-Party Mail & OAuth2 Providers | `src/connectors.rs` (external SMTPS, IMAPS, OAuth2 token refresh) | **Live external Gmail/M365/SMTP/IMAP delivery is `BLOCKED`** without external mail accounts or OAuth2 client secrets. Configuration UI and local loopback behavior (if tested) will be labeled explicitly as local-only, never as third-party live delivery. | `BLOCKED` |
| 25 | Windows & macOS Native Desktop Launchers | `src/main.rs` (`msedge.exe`, macOS `open`), `.github/workflows/release.yml` | **Native Windows/macOS execution is `BLOCKED`** on this Linux x86_64 host. | `BLOCKED` |

---

## Flows That Cannot Be Demonstrated Live (Pre-Recording Disclosure)

This section will be finalized after all recordings complete, summarizing every external-credential or platform-gated flow and any recorded failure honestly.
