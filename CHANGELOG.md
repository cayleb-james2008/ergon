# Changelog

Each release is tagged `vMAJOR.MINOR.PATCH` and published on GitHub Releases with
binaries and a `SHA256SUMS` file. The running app shows its version and can update
itself from the newest release (see README, "Versions and updates").

## 0.35.0 - 2026-11-26

Added parallel DAG pipeline execution, `for_each` batch fan-out, `deliver` connector steps, persisted `agent_snapshot` context compaction, and live `token_delta` SSE streaming across the Rust backend (`src/pipelines.rs`, `src/lib.rs`, `src/sse.rs`, `src/server.rs`, `tests/dag_pipelines_v035.rs`) and the React workbench (`frontend/src/lib/api.ts`, `frontend/src/lib/webEngine.ts`, `frontend/src/shell/PipelineBuilder.tsx`, `frontend/src/shell/CheckpointCard.tsx`, `frontend/src/chat/LiveRunTimeline.tsx`).

In `src/pipelines.rs` and `src/server.rs`, `PipelineStepSpec` adds `depends_on` topological scheduling (up to `4` concurrent branches with cycle detection), `for_each` batch fan-out (up to `50` items with `step_id[i]` `child_items` and `{{steps.<id>.items}}` template access), `when` `on_all_ok` and `expr:<path> == <val>`/`!=` conditions, and `deliver` steps wired through `connectors::invoke_connector` that park `Strict Client` or approval-gated holds as `paused_checkpoint` runs resumed when the linked `connector_delivery` decision settles. `POST /api/pipelines/validate` returns `{ ok, valid, errors, topological_order }` (`HTTP 400` on cycles or invalid step references).

In `src/sse.rs` and `src/lib.rs`, `RunEventBus::publish_token_delta` streams `token_delta` SSE frames (`MAX_BUFFERED_EVENTS = 1024`) from `call_model_with_usage_streaming` with `application/json` fallback, `run_loop` compacts conversation history under `context_budget_chars` (`80,000` default) while keeping the system prompt, last four turns, and `WriteLedger` (`MAX_COMPACTED_STEPS = 48`), and `AgentSnapshot` (`messages`, `written`, `budget_used`) persists across paused and resumed pipeline runs.

In the workbench, `PipelineBuilder.tsx` adds `depends_on` dependency chips, `for_each` batch configuration, the `deliver` step editor, `on_all_ok` and `expr:` run conditions, and inline **Validate DAG** feedback. `CheckpointCard.tsx` and `LiveRunTimeline.tsx` surface parallel branch counts, `child_items` progress pills, outbound delivery hold details, preserved `agent_snapshot` context counts, and live `#live-token-delta-preview` streaming, with full IndexedDB parity in `webEngine.ts` (`ergon-web.html`). Verified post-final-edit: `clippy` 0 warnings, `x86_64-pc-windows-msvc` `xwin` check 0 errors, `274/274` Rust tests (`6/6` `dag_pipelines_v035`), `gate 101/101`, `graph 6/6`, `replay 20/20`, `sloplint 0/0/0` (on both `frontend/src` and rendered shell), `8.31 MiB` (`8,713,056 B`) binary, `162.94 KiB` (`166,851 B`) raw / `37.26 KiB` (`38,151 B`) gzip shell (`17` `/chunks/*`), `7.62 ms` boot, `22.68 MB` peak RSS, `3,699.0 req/s` (`9.25 ms` p95, `0/9,272` errors), `5/5` SSE, and `44/44` fuzz.

## 0.34.0 - 2026-11-26

Added full-vault chunked hybrid search and exact-decimal multi-file table aggregation across the Rust backend (`src/search.rs`, `src/tables.rs`, `src/tools.rs`, `src/lib.rs`, `src/server.rs`, `tests/vault_search_v034.rs`) and the React workbench (`frontend/src/lib/api.ts`, `frontend/src/lib/webEngine.ts`, `frontend/src/shell/CommandPalette.tsx`, `frontend/src/shell/WorkspaceFiles.tsx`).

`PersonIndex` (`src/search.rs`) removes the 512-character file indexing cutoff and indexes every `.xlsx` sheet, `.pdf` page, `.pptx` slide, `.docx`, and text, Markdown, CSV, TSV, or JSON file into overlapping line-bounded chunks (`VaultChunk`) persisted incrementally per person at `vault_index.json` (`0600` on Unix) with SHA-256 and mtime skip checks. Queries (`GET /api/search?q=...&mode=hybrid|bm25|exact&path_prefix=...`) score chunks with BM25, deterministic 256-dimensional n-gram vectors (or optional OpenAI-compatible remote embeddings), and Reciprocal Rank Fusion, returning ranked passages (`chunks[]`) with `page`, `sheet`, `line_start`, `line_end`, `heading`, and `index_stats`. `POST /api/vault/reindex` rebuilds the vault index on demand, while `search_vault` and `aggregate_vault` (`src/tools.rs`, `src/tables.rs`) give the agent spotlighted passage search and fixed-point `i128` rollups across glob-matched CSV, TSV, and multi-sheet `.xlsx` files (`POST /api/vault/aggregate`).

In the workbench, `CommandPalette.tsx` and `WorkspaceFiles.tsx` add a `Hybrid`, `BM25`, and `Exact` mode toggle, live chunk and file counts, one-click `Reindex Vault`, passage jump buttons (`Page N`, `#Sheet`, `Lstart-Lend`) that open `FileViewerModal` directly to the active line range and workbook sheet, a Multi-file table rollup dialog (`#vault-aggregate-dialog`) with CSV save and open, and matching IndexedDB parity in `webEngine.ts` (`ergon-web.html`). Verified post-final-edit: `clippy` 0 warnings, `x86_64-pc-windows-msvc` `xwin` check 0 errors, `268/268` Rust tests (`5/5` `vault_search_v034`), `gate 97/97`, `graph 6/6`, `replay 20/20`, `sloplint 0/0/0` (on both `frontend/src` and rendered shell), `8.19 MiB` (`8,590,176 B`) binary, `162.92 KiB` (`166,825 B`) raw / `37.28 KiB` (`38,172 B`) gzip shell (`17` `/chunks/*`), `10.07 ms` boot, `21.93 MB` peak RSS, `4,070.9 req/s` (`8.14 ms` p95, `0/10,195` errors), `5/5` SSE, and `44/44` fuzz.

## 0.33.0 - 2026-11-26

Added a Client, Engagement, and Retainer hierarchy with LLM token and cost metering, time-saved attribution, and retainer budget gates across the Rust server (`src/records.rs`, `src/lib.rs`, `src/pipelines.rs`, `src/receipts.rs`, `src/audit_bundle.rs`, `src/search.rs`, `src/tools.rs`, `src/server.rs`, `tests/clients_metering_v033.rs`) and the React workbench (`frontend/src/`, `frontend/src/lib/webEngine.ts`).

Each person workspace now tracks clients and retainers (`clients.json` with `0600` permissions on Unix) through `GET /api/clients`, `POST /api/clients`, `POST /api/engagements`, `POST /api/engagements/switch`, and `GET /api/metering/summary`. PDF exports fall back to the active client `brand_header`, and `GET /api/config` plus `PUT`/`POST /api/config` accept per-model input, output, and cached token pricing overrides (`model_pricing.json` `0600`) layered over built-in model rates.

Model calls in `call_model_with_usage` and deterministic pipeline steps record prompt, completion, total, and cached token counts, latency, USD spend, estimated minutes saved, and billable value (`usage_ledger.jsonl` `0600`, `work.json`, and v2 `ChainedReceipt` entries with canonical `usage_digest` while keeping legacy receipt hashes unchanged). Live runs and pipelines emit `usage` deltas on `step_done`, `run_finished`, `pipeline_step_done`, and `pipeline_finished` SSE events, and pre-run or mid-run spend checks park a `budget_gate` decision in Today whenever a metered engagement reaches its retainer cap.

In the UI, `PersonSelect.tsx` adds a Client and Engagement selector with quick creation, `Paper.tsx` adds the Retainer burn and billable ROI card (`#retainer-roi-card`), receipt-row usage summaries, and plain-language `budget_gate` approval cards, `SettingsPanel.tsx` adds the Model pricing overrides editor, and `webEngine.ts` mirrors all client, metering, SSE usage, pricing, and budget gate flows in IndexedDB. Verified post-final-edit: `clippy` 0 warnings, `x86_64-pc-windows-msvc` `xwin` check 0 errors, `263/263` Rust tests (`6/6` `clients_metering_v033`), `gate 93/93`, `graph 6/6`, `replay 20/20`, `sloplint 0/0/0` (on both `frontend/src` and rendered app), `8.08 MiB` (`8,467,296 B`) binary, `6.65 ms` boot, `18.62 MB` peak RSS, `5,777.9 req/s` (`3.46 ms` p95, `0/14,453` errors), `5/5` SSE, and `44/44` fuzz.

## 0.32.0 - 2026-11-26

Hardened hybrid pipelines, document and table handling, atomic file editing, and intake/outbound connectors across the Rust server, tests (`tests/hardening_v032.rs`), and React workbench (`frontend/src/` and `frontend/src/lib/webEngine.ts`).

Pipeline recovery on startup (`src/pipelines.rs`) now transitions runs flagged `cancel_requested` to `cancelled`, marks corrupt `paused_checkpoint` runs lacking a pending checkpoint as `interrupted`, and marks stale `running` steps as `failed`. Async execution updates `current_step_index` so `{{steps.prev.*}}` templates resolve, blocks self, forward, and circular `{{steps.*}}` references under bounded depth (`16`), expansion (`128`), and output (`256 KB`) caps, avoids UTF-8 boundary panics in `extract_step_metrics`, and serializes `/resume`, `/retry`, and `/cancel` atomically inside `mutate_run_http` with `409 Conflict` on concurrent races. `PipelineBuilder.tsx`, `RoutinesPanel.tsx`, `CheckpointCard.tsx`, and `state.ts` validate prompts, choices, step IDs, and template references before saving and guard checkpoint and abort buttons against double submission.

Table loading and file editing (`src/tables.rs`, `src/docs.rs`, `src/file_history.rs`, `src/server.rs`, `src/tools.rs`) now reject invalid UTF-8 and null bytes on CSV, TSV, and JSON loads (`HTTP 400`), preserve ragged extra columns, enforce spreadsheet dimension caps (`200,000` rows, `1,024` columns, `2,000,000` cells) and zip entry budgets, bound PDF export (`200` pages, `2 MB` input), and serialize `PUT` and `PATCH /api/files/content` plus `patch_file` under `HISTORY_LOCK` with SHA-256 verification. `FileEditor.tsx` and `WorkspaceFiles.tsx` detect malformed CSVs with a raw text fallback, preserve unsaved drafts across external `409` SHA changes, and enforce unsaved-change guards across tab switches, file changes, and agent launches.

Connector intake and egress (`src/connectors.rs`, `src/tripwire.rs`, `src/server.rs`, `src/tools.rs`, `src/skills.rs`) reject symlinks and `..` traversal in `validate_watch_dir` and intake reads, enforce `1 MB` (`HTTP 413`) webhook body limits, add `X-Ergon-Timestamp` (`300s` window), `{ts}.{body}` HMAC, and `X-Ergon-Delivery` replay protection with constant-time token comparison, expand secret redaction to JSON-escaped, URL, Base64, hex, and partial secret echoes while keeping `token_hint`, block literal private/metadata IPs in `check_egress`, and enforce `Air-Gapped` and `Strict Client` rules across `settle_delivery_decision` and `invoke_connector`. `ConnectorsPanel.tsx` validates `watch_dir`, outbound URLs, headers, secret keys, and simulator JSON locally, clears transient secrets before async submission and on modal close, and adds delivery log error retry and empty states. Verified post-final-edit: `clippy` 0 warnings, `x86_64-pc-windows-msvc` `xwin` check 0 errors, `257/257` Rust tests (`11/11` `hardening_v032`), `gate 89/89`, `graph 6/6`, `replay 20/20`, `sloplint 0/0/0` (on both `frontend/src` and rendered app), `7.95 MiB` (`8,340,320 B`) binary, `10.01 ms` boot, `17.92 MB` peak RSS, `4,821.3 req/s` (`4.99 ms` p95, `0/12,060` errors), `5/5` SSE, and `44/44` fuzz.

## 0.31.0 - 2026-11-26

Added unattended intake triggers and policy-gated outbound connectors (`src/connectors.rs`, `src/server.rs`, `src/tools.rs`, `src/records.rs`, `src/tripwire.rs`, `frontend/src/shell/ConnectorsPanel.tsx`, `frontend/src/views/Paper.tsx`, `frontend/src/lib/api.ts`, and `frontend/src/lib/webEngine.ts`). Signed inbound webhooks (`POST /api/hooks/:hook_id`) verify `X-Ergon-Signature` or `X-Ergon-Hook-Token` in constant time, write payloads up to `1 MB` to `workspace/inbox/`, and start linked routines or pipelines with `inputs.trigger_file`. Loopback-configured `folder_watch` rules reject sensitive and internal data paths, wait for two steady `(size, mtime)` checks under `25 MB`, deduplicate with a SHA-256 intake manifest, and copy files to `workspace/intake/`.

Added `http_webhook`, `rest_api`, and `smtp_relay` outbound connectors plus the `invoke_connector` agent tool with host-side credential isolation in `0600` `connector_secrets.json`, `{{secret:key}}` header substitution after `tripwire::check_egress` checks, active-run secret redaction, `Air-Gapped` blocking, forced `Strict Client` approval cards (`connector_delivery` in Today), and a recorded `DeliveryLog`. Verified post-final-edit: `clippy` 0 warnings, `x86_64-pc-windows-msvc` `xwin` check 0 errors, `246/246` Rust tests (`3/3` `connectors_v031`), `gate 84/84`, `graph 6/6`, `replay 20/20`, `SLOP 0/0/0`, `7.92 MiB` (`8,303,456 B`) binary, `5.94 ms` boot, `16.96 MB` peak RSS, `8,194.5 req/s` (`2.29 ms` p95, `0/20,493` errors), `5/5` SSE, and `44/44` fuzz.

## 0.30.0 - 2026-11-26

Added the Interactive Deliverable Studio (`FileEditor.tsx` inside `FileViewerModal`, `WorkspaceFiles.tsx`, `Paper.tsx`, `api.ts`, and `webEngine.ts`) with inline split Markdown editing, editable CSV grids, optimistic-lock `PUT /api/files/content` and `PATCH /api/files/content` (`409 Conflict` detection plus automatic `.history/` revisions), live Table Query / Group-By rollups (`POST /api/tables/query` + `src/tables.rs` exact fixed-point `i128` queries across `.csv`, `.tsv`, `.xlsx#SheetName`, and `.json`), and expanded upload `accept` filters covering `.pdf,.docx,.xlsx,.pptx`.

Added surgical `patch_file` and `query_table` agent tools (`src/tools.rs`, `src/lib.rs`), pure-Rust native PDF 1.4 export (`markdown_to_pdf`), and multi-sheet formula `.xlsx` workbooks (`sheets_to_xlsx` emitting `<f>SUM(...)</f>` formulas and cached totals in `src/docs.rs`). Verified post-final-edit: `clippy` 0 warnings, `x86_64-pc-windows-msvc` `xwin` check 0 errors, `243/243` Rust tests (`6/6` `studio_tables_pdf`), `gate 79/79`, `graph 6/6`, `replay 20/20`, `SLOP 0/0/0`, `7.77 MiB` (`8,147,808 B`) binary, `4.3 ms` boot, `16.84 MB` peak RSS, `8,194.8 req/s` (`2.31 ms` p95, `0/20,493` errors), `5/5` SSE, and `44/44` fuzz.

## 0.29.0 - 2026-11-26

Resumable hybrid workflow pipelines (`steps_v2`) execute multi-step combinations of zero-LLM deterministic skills, AI agent runs, structured human checkpoints, and document exports asynchronously under `/api/pipelines/{run,runs,runs/:id,resume,retry,cancel}`, persisting execution state to `pipeline_runs.json` across restarts and streaming `pipeline_*` events over `/api/runs/stream` (`src/pipelines.rs`, `src/routines.rs`, `src/tools.rs`, `src/lib.rs`, `src/server.rs`).

Starter playbooks now launch live `steps_v2` deterministic skill pipelines against workspace files instead of canned preview text (`src/skill_catalog.rs`, `src/skills/catalog.json`), and `request_checkpoint` parks ad-hoc agent runs until answered in Today (`Paper.tsx`), inline chat (`Chat.tsx`), or Routines (`PipelineBuilder.tsx`, `CheckpointCard.tsx`) with full browser-only parity in `ergon-web.html` (`frontend/src/lib/webEngine.ts`).

## 0.28.0 - 2026-11-25

Zero-setup starter playbooks bundle three verified workflows (`Weekly Client Digest`, `Repo Release Audit`, and `Research Brief + Citation Pack`) inside the compressed built-in skill catalog (`src/skills/catalog.json`, `src/skill_catalog.rs`). Launch, preview ordered dry-run steps, or run any starter directly into a signed Client Handoff ZIP archive in one click from Today or Recurring routines (`GET /api/playbooks/starters`, `POST /api/playbooks/starters/launch`, `frontend/src/shell/StarterPlaybooksBar.tsx`), with offline `ergon-web.html` parity (`frontend/src/lib/webEngine.ts`).

Client handoff ZIP bundling (`src/audit_bundle.rs`, `src/docs.rs`, `src/server.rs`) now borrows slices via `zip_write_refs` without cloning the `audit_bundle.json` buffer, reuses the cached `PersonIndex` receipt chain when available, streams workspace leak scans through one shared read buffer, and drops temporary bundle buffers immediately for HTTP exports. Moving static skill dry-run fixtures into compressed `catalog.json` keeps the stripped release binary at `7.497 MB` (`7,861,088 B`).

## 0.27.0 - 2026-11-25

One-click client handoff bundling packs a verified `CLIENT_SUMMARY.md`, tamper-evident `audit_bundle.json`, `SHA256SUMS`, and offline verification instructions (`VERIFY.txt`) into a self-contained ZIP archive or directory. Run `ergon export-handoff [--out PATH] [--title TITLE] [--workspace DIR] [--force] [--json]` from the CLI (`src/main.rs`), call `POST /api/export/handoff` (`src/server.rs`), or use the Client handoff card in the right inspector (`frontend/src/shell/InspectorDrawer.tsx`, `frontend/src/lib/webEngine.ts`).

Every handoff export runs a deterministic pre-check across workspace deliverables and generated summaries for exposed host machine paths, API keys, `.env` credential snippets, private keys, and SSN patterns, returning `file:L##` fix hints and blocking export (`409 Conflict` or CLI exit `1`) under `Strict Client` unless `--force` is passed (`src/audit_bundle.rs`).

Warm HTTP accept pool initialization drops from four worker threads to two (`src/server.rs`), and `frontend/e2e/probe-graph.mjs` replaces fixed graph settle waits with deterministic `svg#graph-svg` selector waits.

## 0.26.0 - 2026-11-25

Offline evidence verification lets auditors and clients check exported audit bundles (`.json`), client deliverable packs (`.md`), and signed playbook packs (`.ergon-playbook.json`) without a running server or model gateway. Run `ergon verify <bundle-or-pack> [--workspace <dir>] [--json]` from the command line (`src/main.rs`, `src/audit_bundle.rs`), call `POST /api/verify/evidence` (`src/server.rs`), or use the Verify evidence card inside the right inspector (`frontend/src/shell/InspectorDrawer.tsx`, `frontend/src/lib/webEngine.ts`).

Switching the release profile to `panic = "abort"` with an isolated `--internal-pdf-text` worker subprocess for untrusted PDF parsing (`Cargo.toml`, `src/docs.rs`, `src/main.rs`), moving the twelve built-in skill definitions into a compile-time compressed `src/skills/catalog.json` (`build.rs`, `src/skill_catalog.rs`), factoring `person_idx`, and replacing remaining inline error JSON allocations with `respond_err` bring the stripped release binary down to `7.46 MB` (`7,824,224 B`).

## 0.25.0 - 2026-11-25

Workspace audit policy profiles let you switch between `Fast Internal`, `Strict Client`, and `Air-Gapped` in one click from the Audit tab in the right inspector. `GET`, `PUT`, and `POST /api/workspace/policy` save the choice to `workspace_policy.json` (`src/lib.rs`, `src/server.rs`, `frontend/src/shell/InspectorDrawer.tsx`, `frontend/src/lib/webEngine.ts`).

Under `Strict Client`, `GET /api/files/pack` returns `409 Conflict` when exports reference stale or broken `[file:L##]` citations, and host home or temp paths in packs and audit bundles redact to `~/`. Under `Air-Gapped`, `fetch_url`, `research`, and `host_fetch_url` are blocked in `src/tools.rs` and `src/skills.rs`.

Pre-compressing the twelve embedded `.rn` skill sources in `build.rs`, dropping the unused `gzip` feature from `ureq`, optimizing `serde` and `serde_core` for size in `Cargo.toml`, converting `PersonIndex` token posting lists to `Box<[u32]>`, and tightening `BufReader` and `JSON_SCRATCH` capacities trim the release binary below `8.40 MB` and keep warm RSS under `16.0 MB`.

## 0.24.0 - 2026-11-25

Added automated `audit_memory_health` and `heal_memory` endpoints (`GET /api/memory`, `GET /api/memory/health`, `POST /api/memory/heal`) across `src/skills.rs`, `src/server.rs`, `frontend/src/shell/Panels.tsx`, `frontend/src/views/Paper.tsx`, and `frontend/src/lib/{api,webEngine}.ts` to detect duplicate preferences, contradicting standing rules, and broken `[file:L##]` or workspace file references. Surfaced `Healthy` and issue-count pills plus one-click Resolve and Prune (`#memory-heal-btn`) in the memory modal, navigation rail, and Today desk card.

Switched the embedded workflow VM to `default-features = false, features = ["std"]` with native diagnostic span formatting, tuned `[profile.release]` `opt-level = "z"` while keeping `opt-level = 3` on hot HTTP, JSON, and SHA-256 crates, and capped idle `BufReader` (`4 KB`) and `JSON_SCRATCH` (`32 KB`) slack for `< 16.5 MB` peak RSS.

## 0.23.0 - 2026-11-25

Extended routine enrichment (`count_missed_windows`, `enrich_routine_with_verification` in `src/scheduler.rs`, `src/server.rs`, `src/lib.rs`) to track missed cadence windows (`missed_windows`) and receipt-chain verification (`last_run_verified`: `"verified"` or `"drift"`, `last_run_receipt_sha256`), and added `POST /api/routines/:id/catchup` and `POST /api/scheduler/catchup` with `webEngine.ts` parity. Added `Verified` and `Drift` SHA-256 pills (`data-routine-sha-badge`), `N missed` badges, and one-click Catch up buttons in Recurring routines (`frontend/src/shell/RoutinesPanel.tsx`).

Replaced per-request `serde_json::Value` DOM trees on `/api/version`, `/api/status`, `/api/files`, `/api/receipts`, and `/api/search` (`src/server.rs`, `src/search.rs`) with static bytes and reusable thread-local `JSON_SCRATCH` streaming serialization, raising keep-alive throughput past `5,000 req/s` under a `< 9.0 MB` release binary.

## 0.22.0 - 2026-11-25

Extended `GET /api/files/pack?summary=1` (`.md`, `.html`, and `preflight=1&summary=1` in `src/server.rs`, `src/lib.rs`, `frontend/src/views/Paper.tsx`, `frontend/src/lib/{receiptExport,webEngine}.ts`) to union deliverable files and verify `[file:L##]` citations across summarized finished runs in one pass, keeping the strictest per-citation drift status, annotating `## Verified Work Summary` rows, and emitting `X-Ergon-Pack-Summary-Runs`, `X-Ergon-Pack-Summary-Citations`, and `X-Ergon-Pack-Summary-Stale` headers. Added `#summary-citation-gate` freshness pills and stale-warning export toasts in Today Finished.

Tuned `[profile.release]` (`opt-level = "s"` on `ergon-rs` while preserving `opt-level = 3` on `serde`, `serde_json`, `itoa`, `ryu`, `memchr`, `sha2`, `digest`, `block-buffer`, `cpufeatures`), shrank `PersonIndex` chain slack and snippet read buffers (`1024 B`), and trimmed HTTP worker stack size (`512 KB`).

## 0.21.0 - 2026-11-25

Added read-only deterministic `POST /api/playbooks/:id/preview` and `POST /api/routines/:id/preview` (`src/routines.rs`, `src/server.rs`, `frontend/src/lib/{api,receiptExport,webEngine}.ts`, `frontend/src/state.ts`, `frontend/src/shell/RoutinesPanel.tsx`) to expand ordered playbook steps, matched built-in workflows, predicted workspace file impacts (with `.history/` revision counts), and the `Reversible via .history/` safety badge without touching disk or calling an LLM. Added one-click Preview and the inline Dry-run preview drawer (`#playbook-preview-drawer`, `#playbook-rollback-badge`) in Recurring routines.

Refreshed all six Light and Dark hero and gallery screenshots (`docs/screenshots/{hero,run-diff,worktree-lanes}-{light,dark}.png`, `README.md`) against a configured local gateway fixture so the status badge shows `Ready`.

## 0.20.0 - 2026-11-25

Added `GET /api/playbooks/:id/export` and `POST /api/playbooks/import` (`src/routines.rs`, `src/server.rs`, `frontend/src/lib/{api,receiptExport,webEngine}.ts`, `frontend/src/state.ts`, `frontend/src/shell/RoutinesPanel.tsx`) with `ergon-playbook-v1` schema validation, `<= 256 KB` payload cap (`413` on oversize), canonical SHA-256 checksum verification (`X-Ergon-Playbook-SHA256`), and `webEngine.ts` parity. Added one-click Export and Import playbook (`#playbook-import-btn`) controls in Recurring routines.

Replaced fixed sleeps (`waitForTimeout`) in `frontend/e2e/replay.mjs` and `frontend/e2e/cases.mjs` with explicit DOM and route settlement waits (`waitFor` plus `requestAnimationFrame`) for deterministic `20/20` replay passes on pinned CPUs.

## 0.19.0 - 2026-11-25

Enabled HTTP/1.1 persistent Keep-Alive connections (`handle_connection` and zero-copy `IoSlice` vectored writes in `src/server.rs`, keeping immediate `Connection: close` on 4xx, 5xx, and SSE) and restored `opt-level = 3` on `serde`, `serde_core`, `itoa`, `ryu`, `memchr`, and `digest` while keeping cold crates (`lopdf`, `rustls`, and the workflow VM) at `opt-level = "z"`. Added a single-token `O(limit)` fast path in `PersonIndex::search` (`src/search.rs`) to avoid 5,000-entry substring scans on exact token queries (`GET /api/search?q=...`).

Extended `GET /api/files/pack?summary=1` (`format=md|html`) to compile recent finished receipts into a `## Verified Work Summary` table alongside file SHA-256 digests and `[file:L##]` citation ratios (`100% Verified` vs `Stale Line Drift`), and added Client Summary (`.md`) and Summary (`.html`) export buttons in Today's Finished header.

## 0.18.0 - 2026-11-25

Extended `verify_text_citations_against` and `PackPreflight` (`src/tools.rs`, `src/server.rs`, `frontend/src/lib/{receiptExport,webEngine}.ts`, `frontend/src/shell/InspectorDrawer.tsx`) with live SHA-256 digests (`file_sha256`), live line excerpts (`line_excerpt`), and `.history/` or baseline SHA-256 drift detection (`stale` vs `verified` status). Added `GET /api/files/pack?format=html` (`build_workspace_handoff_html`) and `#export-handoff-html-btn` to generate standalone printable HTML deliverables with a SHA-256 manifest, citation freshness gate, optional delta, and strict CSP (`default-src 'none'; style-src 'unsafe-inline'`).

Configured `[profile.release.package."*"] opt-level = "z"` (keeping `ergon-rs`, `serde_json`, and `sha2` at `opt-level = 3`), and added `SearchCache::get_if_fresh`, `audit_bundle::build_audit_bundle_json_bytes_with_chain`, and zero-alloc `compute_workspace_manifest_sha256` hashing so warm `/api/audit/bundle` calls borrow cached receipt chains without duplicating 5,000-entry allocations.

## 0.17.0 - 2026-11-25

Added `file_history::diff_texts` and `GET /api/receipts/compare?a=<id>&b=<id>` (`src/file_history.rs`, `src/server.rs`, `frontend/src/shell/{InspectorDrawer,WorkspaceFiles}.tsx`, `frontend/src/lib/{receiptExport,webEngine}.ts`) to compute line-by-line summary deltas (`+added / -removed / % similarity`), added/removed deliverable files, and unified Markdown diff blocks. Extended `GET /api/files/pack` with `delta_a`/`delta_b` and `delta_file`/`rev_a`/`rev_b`, plus Append Delta to Client Pack (`.md`) buttons in `InspectorDrawer` and `FileHistoryDiffView`.

Added scored fuzzy search (`fuzzyScore` in `frontend/src/shell/CommandPalette.tsx`) in `Cmd+K` across workspace files, extracted `[file:L##]` source citations, multi-step playbooks and routines, workflows, and receipts.

## 0.16.0 - 2026-11-25

Added `verify_text_citations` and `POST /api/citations/verify` (`src/tools.rs`, `src/server.rs`, `frontend/src/chat/Entry.tsx`, `frontend/src/shell/WorkspaceFiles.tsx`, `frontend/src/shell/InspectorDrawer.tsx`, `frontend/src/lib/webEngine.ts`) and wired automatic `[file:L##]` cross-file verification into `GET /api/files/pack?preflight=1` and Client Pack (`.md`) exports. Rendered clickable `[file:L##]` citation pills in finished receipts, replies, and Pack Pre-Flight summaries that scroll and highlight the cited line range inside `FileViewerModal`.

Enabled fat LTO (`lto = "fat"` in `Cargo.toml`) and pre-gzipped `assets/index.html` into `.rodata` (`index.html.gz`), bringing the release binary below `11.8 MB`. Added streaming `audit_bundle::build_audit_bundle_json_bytes` serializing 5,000+ `receipt_chain.entries` directly into `ShaWriter` without allocating `5,000` `Value::Object` maps, keeping peak `VmHWM` under `29 MB` on 5,000 receipts and 500 files.

## 0.15.0 - 2026-11-25

Pre-compressed lazy `.js` and `.css` chunks at compile time via `miniz_oxide` inside `build.rs` and served them zero-copy from `.rodata` (`src/ui.rs`, `src/server.rs`), removing runtime `CHUNK_CACHE` heap allocations while inflating on the fly for non-gzip clients.

Added streaming `receipts::load_chain` (`src/receipts.rs`, `src/search.rs`, `src/audit_bundle.rs`, `src/server.rs`) parsing `work.json` and `receipts.json` one object at a time (~2 KB buffer) and streamed `compute_canonical_bundle_sha256` directly into `Sha256` without cloning 5,000-entry arrays. Upgraded run saving into one-click Promote to Playbook (`extractPlaybookFromRun`) and added `GET /api/files/pack?preflight=1` with inline Pre-Flight Check.

## 0.14.0 - 2026-11-25

Split `PaperFace`, `GraphFace`, `FileViewerModal`, `InspectorDrawer`, `SettingsPanel`, `SkillsPanel`, `RoutinesPanel`, `CommandPalette`, and secondary `Panels` into lazy-loaded `/chunks/*.js`, `.woff2`, and `.css` assets (`build.rs`, `src/ui.rs`, `src/server.rs`, `frontend/vite.config.ts`, `frontend/src/App.tsx`, `frontend/src/chat/LiveRunTimeline.tsx`, `frontend/src/views/usePaper.ts`) served with RFC 1952 `gzip`, `ETag` + `304`, `Vary: Accept-Encoding`, `Cache-Control: public, max-age=31536000, immutable`, and strict security headers. This reduced initial `GET /` payload from `1,054 KB` (~375 KB gzip) to `148,263 B` (`144.8 KiB`) raw and `33,291 B` (`32.5 KiB`) gzip while keeping `npm run build:web` a single self-contained `ergon-web.html`.

Fixed `handleResolveDecision` and `WaitingEntry` pending decision matching so approvals emit immediate `"Approved"` toasts and refresh Today and the active thread in parallel (`Promise.all`), and added one-click Load sample workspace (`#starter-pack-btn`) and Package & Verify (`.md`) (`#package-verify-btn`). Removed duplicate `receipt_lower: Vec<String>` allocations from `PersonIndex` (`src/search.rs`), capped snippet previews at 512 chars, shrunk inverted token posting lists, and set HTTP worker pool stack size to `2 MB`.

## 0.13.0 - 2026-11-25

Extended routines into sequential Workspace Playbooks (`steps`, optional `expected_inputs`, and `pack_deliverables` in `src/routines.rs`, `src/server.rs`, `frontend/src/shell/RoutinesPanel.tsx`, `frontend/src/lib/webEngine.ts`) executed step by step in one shared thread context via `POST /api/routines/:id/run`, with optional `"Playbook Deliverable Pack.md"` assembly. Added one-click presets (`Month-End Close (3 steps)`, `Contract Audit + Pack (2 steps)`), interactive step chaining (`+ Chain step`), and `"Auto-bundle Deliverable Pack (.md)"`.

Reused a single token scratch buffer across 10,000-receipt and 500-file index builds (`src/search.rs`), replaced temporary `HashSet` query intersections with sorted `binary_search` narrowing (`narrow_candidates`), and routed filtered `list_receipts` and `list_files` queries through inverted token indexes. Extracted sandboxed workspace tools, URL egress fetchers, `export_workspace_file`, and honesty-check helpers into `src/tools.rs`, upgraded cached `DecoratedPage` pre-compression in `src/tripwire.rs` to level 9, consolidated Geist font weights, targeted `es2022` in Vite, and pruned dead `WorkspaceFilesPanel`.

## 0.12.0 - 2026-11-25

Extracted the embedded HTTP server, request parser, security gates, and domain route handlers (`try_route_system`, `try_route_files_and_audit`, `try_route_knowledge_and_routines`, `try_route_runs_and_chat`) out of `src/lib.rs` into `src/server.rs` while keeping public crate exports intact. Standardized config, thread store, routine store, scheduler run history, and `.history/` manifests and blobs onto unique-temp atomic writes (`atomic_write_bytes` and `atomic_write_bytes_with_mode` preserving `0o600` on saved API keys), and added `GET /api/files/pack` with `webEngine.ts` parity.

Added one-click Save as routine actions on finished deliverables (`FinishedEntry` in `Entry.tsx`) and finished receipt rows in Today (`PaperFace` in `Paper.tsx`), one-click Client Pack (`.md`) batch downloads, multi-file upload queue chips (`Chat.tsx`, `detectBatchWorkflow` in `words.ts`), `<LazyMotion features={domAnimation} strict>`, memoized workbench panes, and fast `/api/receipts?limit=200` initial loading in Today.

## 0.11.0 - 2026-11-25

Added `[profile.release]` (`opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`), served `assets/index.html` with pre-compressed RFC 1952 `gzip`, `ETag` + `304 Not Modified`, `Vary: Accept-Encoding`, `Cache-Control: no-cache`, and security headers (`nosniff`, `DENY`, `Referrer-Policy: no-referrer`, CSP). Replaced per-connection `thread::spawn` with a warm worker pool, enabled `TCP_NODELAY`, coalesced headers and body with `BASE_HEADERS_BLOCK`, removed `TcpStream::try_clone()` and per-header `.to_lowercase()` allocations in `read_request`, switched `sessions`, `runtimes`, `people`, and `decorated` locks to `RwLock`, and included precomputed thread `state` in `GET /api/threads` (`Store::list_json`).

Added context-aware workflow auto-detection (`detectWorkflowForFile`, `detectWorkspaceWorkflow`), Unified and Side-by-side diff toggle in `FileHistoryDiffView`, fixed `/api/receipts/verify` chain-count parsing, wired SHA-256 receipt export into `FinishedEntry`, added Copy summary, and isolated the live elapsed timer inside `LiveElapsedClock`. Rejected duplicate `Host` headers (`HTTP 400`), bounded `LiveRun::steps` (`MAX_LIVE_STEPS = 128`), blocked hidden dotfiles in `.history/` (`validate_history_path`), and guarded `export_workspace_file` against overwriting workspace symlinks.

## 0.10.0 - 2026-11-25

Removed the duplicate `1.2 MB` `assets/phone.html` asset and `ui::PHONE_HTML`, serving one responsive SPA (`assets/index.html`, `972 kB`) across `/`, `/phone`, desktop, tablet, and mobile. Built the 3-pane workspace with top bar, workspace pill (`#person-select`), view switcher (`Chat`, `Today`, `Work graph`), collapsible Left Rail, centered Thread with four business deliverable starter cards, and collapsible Right Inspector Drawer (`Files`, `History & Diff`, `Audit & Offers`).

Rendered waiting approval cards (`WaitingEntry`) inline inside the conversation thread with Approve, Hold / Keep, Revise, and Preview diff, plus one-click Word (`.docx`), Excel (`.xlsx`), SHA-256 Receipt (`.md`/`.json`), and Audit Bundle (`.md`/`.json`) exports. Pruned unused wrapper libraries, kept Settings-only Light and Dark switching, fixed `--input` border contrast (`>= 3:1`), and enforced DOM Range zero-text-clipping and zero-overflow UI gates (`sloplint --scope app --render` `0 FAIL`). Rejected conflicting `Content-Length` and `Transfer-Encoding: chunked` headers (`HTTP 400`), enforced case-insensitive `Host`/`Origin` checks, bounded active HTTP connections (`MAX_ACTIVE_CONNS = 128`, `HTTP 503`), guarded `POST /api/files/upload` against symlink targets with atomic `FILE_TMP_SEQ` writes, and skipped loading `PersonRuntime` in `start_scheduler_thread` when no `routines.json` exists.

## 0.9.0 - 2026-11-24

Added document readers (`src/docs.rs`) for PDF (including CID and ToUnicode fonts), Word `.docx` (Markdown with headings, lists, tables, and links), Excel `.xlsx` (each sheet as CSV with dates, booleans, and cached formula values; `book.xlsx#Sheet` selects a sheet), and PowerPoint `.pptx` (slides and speaker notes), wired through `read_file`, `search_files`, built-in workflows (`host_workspace_read`), the file viewer, and the search index. Added `export_file`, `POST /api/files/export`, and Save as Word / Save as Excel in the file viewer for styled `.docx` and typed formula-safe `.xlsx` exports validated against LibreOffice, python-docx, and openpyxl.

Switched uploads to byte-exact base64 up to `25 MB` (keeping the `1 MB` cap on all other routes), fixed previews vs downloads over `2 MiB`, added explicit `offset` windows (~5,800 chars) in `read_file`, skipped ledger summary rows ("Total", "Subtotal") and flagged possible duplicate charges in `reconcile_ledger`, added bounded backoff retries on remote `408`/`425`/`429`/`5xx`/`52x` and dropped connections, and deduplicated `memory.md` entries while truncating prompts past ~10,000 chars. Verified with 16 tests in `tests/documents_v090.rs` (including zip-bomb and malformed-PDF safety), 200 Rust tests, and Playwright (`69 + 6 + 20`).

## 0.8.0 - 2026-11-24

Added OWASP AppSensor-style security tripwires (`src/tripwire.rs`, `GET /api/security`, `POST /api/security/policy`): 70+ perimeter decoy resources (`/.env`, `/.git/config`, `/wp-login.php`, `/api/login`, `/api/internal/*`) returning plausible decoys with HMAC-verifiable `sk-ergon-...` honeytokens, `robots.txt` and page-source breadcrumbs, and a decoy `ergon_role` cookie. Presenting a honeytoken or tampering with the cookie is refused and banned on remote addresses with decaying fail2ban-style scoring (loopback is never locked out), and refused cross-site, DNS-rebinding, and bad gate-token requests are logged.

Added per-run prompt bait keys, Spotlighting boundaries on tool output, an egress guard (`fetch_url`, research workflow, `host_fetch_url`) blocking leaked secrets and vetting resolved redirect IPs against metadata and private networks, threat-score termination at 100, and memory/library supply-chain checks holding suspicious `update_memory` notes and read-plus-fetch skills as Today decisions. Added SHA-256 hash-chained `security_events.json`, Settings -> Security controls, per-inline-script SHA-256 CSP, `X-Frame-Options: DENY`, `nosniff`, `Referrer-Policy: no-referrer`, COOP/CORP, `Permissions-Policy`, and `Cache-Control: no-store` on API JSON. Fixed raw JSON decision labels in Today so far and root permission handling in `update::tests::errors_name_files_never_folders`. Verified with 18 `tests/tripwire.rs` proofs, 13 tripwire unit tests (including RFC 4231 HMAC), and Playwright (`69 + 6 + 20`).

## 0.7.0 - 2026-11-22

Added the background routine scheduler (`src/scheduler.rs`, `PATCH /api/routines/:id`, `GET /api/routines/:id/history`) with a `15s` cadence ticker (`ERGON_DISABLE_SCHEDULER=1` opt-out, SSE `routine_fired`), UTC calendar math (`every 15m`, `hourly`, `daily 09:00`, `weekly mon`, or `interval_secs`), pause/resume toggles, `Next run` countdowns, and `routine_history.json`. Added traversal-proof `.history/` file version snapshots (`src/file_history.rs`, `GET /api/files/history`, `POST /api/files/restore`) with SHA-256 verification, LCS unified diffs, and `pre-restore` recovery inside `FileViewerModal`.

Added tamper-evident JSON and Markdown Field-Book Audit Bundles (`src/audit_bundle.rs`, `GET /api/audit/bundle?format=json|md`), a mobile-safe Workspace Switcher (`POST /api/people/switch`, `WorkspaceSwitcherPanel`) outside mobile Vaul drawers with browser-edition IndexedDB parity, and standalone chromeless desktop app launch flags (`--app`/`--open`, `--no-open`, `--host`, `--port`, `--help`, `--version`, `ERGON_NO_OPEN=1`) with `x86_64-pc-windows-msvc` builds.

## 0.6.0 - 2026-11-22

- Hardened agent honesty loop (`src/lib.rs::run_loop`, `run_agent_live`): detects tasks naming a workspace tool/workflow or workspace file and rejects premature zero-tool `{"done": true}` replies (forcing real tool invocation against the workspace before finishing, and recording an explicit caution + `finished: false` receipt if a model still runs no tools).
- Eliminated host filesystem path disclosure in workflow & workspace tool errors (`src/skills.rs::host_workspace_{read,write,list}`, `src/lib.rs::list_dir`) so missing-file and I/O errors report only workspace-relative paths.
- Aligned numeric parsing in `clean_dataset` (`parse_num`) with `reconcile_ledger` to parse accounting parenthesized negatives (`($12.34)`, `($1,250.50)`) accurately in dataset profiling and outlier detection.
- Verified end-to-end across 3 distinct fictional company workspaces (`northstar-freight`, `cedarbrook-dental`, `ironwood-mechanical`) with all workflows and `EC-01..EC-08` adversarial edge cases against live local Ollama.

## 0.5.0 - 2026-11-22

- Expanded built-in workflow library from 4 to 12 deterministic employee-grade workflows (`reconcile_ledger`, `summarize_meeting`, `audit_contract`, `triage_inbox`, `clean_dataset`, `compare_documents`, `draft_brief`, `extract_tables`, plus all 4 original workflows) with `src/skill_catalog.rs`, `GET /api/skills`, and sandboxed script test harness `POST /api/skills/test`.
- True Server-Sent Events live run stream (`GET /api/runs/stream`) emitting `run_started`, `step_start`, `step_done` (with elapsed ms), `file_written`, `approval_parked`, and `run_finished` frames plus active cancellation (`POST /api/runs/:id/cancel`).
- High-scale indexed search & caching (`GET /api/search?q=...`, paginated `GET /api/receipts`, cached `GET /api/files`): per-person `RwLock` inverted index proven under 5,000 receipts + 500 workspace files with 30 concurrent HTTP clients + 5 SSE subscribers (`p95 < 150ms`).
- Cryptographically chained SHA-256 Field-Book receipt export (`GET /api/receipts/:id/export?format=md|json`, `GET /api/receipts/verify`) with per-file SHA-256 workspace digests, plus atomic Standing Recurring Routines (`GET/POST/DELETE /api/routines`, `POST /api/routines/:id/run`).
- Power-user Desk UX & Local-First Browser Edition: Global `Cmd+K` / `Ctrl+K` Command Palette, multi-file drag-and-drop upload, upgraded `FileViewerModal` (CSV numeric sort/filter/paging, Markdown section jump, JSON tree, SHA-256 export), Built-in Workflows & Routines dialogs lifted cleanly outside mobile Vaul drawer (`390x844` & `320x568` verified), and hardened 32-vector file path fuzzing.

## 0.4.0 - 2026-11-21

- Live local model connectivity out of the box: keyless loopback OpenAI-compatible endpoints (`127.0.0.1`, `localhost`, `[::1]`), automatic startup probing of running local gateways (`:8790`, `:8789`, `:11434`, opt-out via `ERGON_NO_AUTO_MODEL=1`), and persisted live model configuration (`GET/POST /api/config`, saved to `data/config.json` with `0600` permissions on Unix).
- Real Workspace File Viewer & upload: `GET /api/files`, `GET /api/files/content?path=...(&download=1)` jailed strictly through `sandbox_path_resolving` (refusing `..` traversal and symlink escapes with HTTP 403), and same-site guarded `POST /api/files/upload`. Every saved file chip and Open button across Chat, Today, Work Graph, and Workspace files opens a live previewer (tabular CSV view, structured Markdown, formatted JSON, plain text, plus real file Download).
- Live run step streaming: `GET /api/runs/status` streams active tool executions into the Chat active-run card in real time as tools complete.
- Unified Settings dialog (`Model, Workspace & Appearance`): consolidates model provider switching, Light/Dark theme `SegmentedControl` (keeping main rail chrome uncluttered), and version/update controls in one rail-footer dialog.
- Real Local-First Browser Edition (`npm run build:web`): compiles an IndexedDB-backed standalone client-side workspace with deterministic CSV reconciliation, Markdown summarization, standing memory rules, approval-gated deletion, optional BYOK LLM (`sessionStorage` only), and one-click connect to local `ergon` (`127.0.0.1:8787`).

## 0.3.0 - 2026-11-21

- Personal Employee Agent Desk (House Frontend + Furrow, light-default with full dark parity): persistent 256px left desk rail (Vaul drawer under 768px), `/` conversation filter with keyboard hint, `Cmd/Ctrl+K` global composer jump, and self-hosted Geist + Geist Mono variable/static woff2 fonts.
- Today's Desk: Morning Brief with rolling counters, Tremor 14-slot task outcome `Tracker`, consequential `Waiting on you` approval queue, proactive repeatable-app `Offers`, finished work ledger with file-type glyphs, and `Today so far` chronological activity stream (`/api/today`).
- Chat & Field-Book Entries: role-tailored starter task cards with skill/tool chips, live active-run ledger (`TextShimmer` + `BorderTrail`), Field-Book Result Cards with saved-file glyphs and Cult UI `Expandable` raw tool-step inspector, and unified responsive `/phone` desk.
- Update strip hardening: loopback-only `Update now`, focus preservation across busy/staged/dismissed states, `Show update` recovery control, and plain-language failure banners with raw diagnostics behind `Details`.
- Verification: 101 Rust unit & E2E tests plus expanded Playwright UI gate (`gate`, `probe-graph`, and worst-case `replay`).

## 0.2.0 - 2026-11-19

- New front end: Chat, Today and Work Graph faces and a `/phone` face; plain-language step ledger; amber "Waiting on you" state; light and dark themes; keyboard and screen-reader support.
- Safety: cross-site guard (Host, Origin, `Sec-Fetch-Site`, JSON-only POST), bounded request heads, `ERGON_ALLOWED_HOSTS`.
- Honesty: a parked deletion is never reported as done; plain answers and fully failed runs are not counted as finished work; recovered store files are named in the page and `/api/status`.
- `--version`, `GET /api/version`, and self-update from GitHub releases (`ERGON_UPDATE_MODE`).
- Dependencies: Rust 1.100, embedded deterministic workflow runtime.

## 0.1.0 - 2026-11-17

- First single-binary build: one binary serving the web shell, agent loop, tools, per-person memory and built-in workflows.
