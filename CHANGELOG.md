# Changelog

Each release is tagged `vMAJOR.MINOR.PATCH` and published on GitHub Releases with
binaries and a `SHA256SUMS` file. The running app shows its version and can update
itself from the newest release (see README, "Versions and updates").

## 0.21.0 - 2026-11-25

- **Playbook Dry-Run Preview & Reversible File-Impact Summary (`src/routines.rs`, `src/server.rs`, `frontend/src/lib/{api,receiptExport,webEngine}.ts`, `frontend/src/state.ts`, `frontend/src/shell/RoutinesPanel.tsx`):**
  - Added read-only deterministic `POST /api/playbooks/:id/preview` and `POST /api/routines/:id/preview` expanding ordered playbook steps, matched built-in workflows, predicted workspace file impacts (with `.history/` revision counts), and the `Reversible via .history/` safety badge without touching disk or calling an LLM.
  - Added 1-click **Preview** + inline **Dry-run preview** drawer (`#playbook-preview-drawer`, `#playbook-rollback-badge`) inside Recurring Routines with full `webEngine.ts` standalone parity.
- **Visual Polish & Verified Benchmarks (`docs/screenshots/{hero,run-diff,worktree-lanes}-{light,dark}.png`, `README.md`):**
  - Refreshed all 6 Light/Dark hero & gallery screenshots against a configured local gateway fixture so the status badge shows green **`Ready`** (zero unconfigured banner).

## 0.20.0 - 2026-11-25

- **1-Click Verified Playbook Pack Export & Import (`.ergon-playbook.json` + SHA-256 Checksum) (`src/routines.rs`, `src/server.rs`, `frontend/src/lib/{api,receiptExport,webEngine}.ts`, `frontend/src/state.ts`, `frontend/src/shell/RoutinesPanel.tsx`):**
  - Added `GET /api/playbooks/:id/export` and `POST /api/playbooks/import` with strict `ergon-playbook-v1` schema validation, `<= 256 KB` payload cap (`413` on oversize), and canonical SHA-256 checksum verification (`X-Ergon-Playbook-SHA256`), plus 100% standalone `webEngine.ts` parity.
  - Added 1-click **Export** on routine/playbook cards and **Import playbook** (`#playbook-import-btn`) in Recurring Routines with live SHA-256 verification feedback.
- **Deterministic Playwright Replay Hardening (`frontend/e2e/replay.mjs`, `frontend/e2e/cases.mjs`):**
  - Replaced arbitrary `waitForTimeout(1800)` / `waitForTimeout(400)` / `waitForTimeout(500)` sleeps with explicit DOM/route settlement waits (`waitFor` + `requestAnimationFrame` paint sync) for deterministic `20/20` replay passes on pinned CPUs.

## 0.19.0 - 2026-11-25

- **HTTP Keep-Alive, `-O3` Hot-Path Split & `O(limit)` Token Search (`Cargo.toml`, `Cargo.lock`, `src/server.rs`, `src/search.rs`):**
  - Enabled HTTP/1.1 persistent Keep-Alive connections (`handle_connection` + `respond_bytes` zero-copy `IoSlice` vectored writes, preserving immediate `Connection: close` on 4xx/5xx/SSE) and restored `opt-level = 3` on `serde`/`serde_core`/`itoa`/`ryu`/`memchr`/`digest` while keeping heavy cold crates (`rune`, `lopdf`, `rustls`) at `opt-level = "z"`.
  - Added single-token `O(limit)` fast path in `PersonIndex::search` eliminating 5,000-entry substring scans on exact token queries (`GET /api/search?q=...`).
- **1-Click Multi-Run Retainer / Client Summary Pack (`.md` & `.html`) (`src/server.rs`, `frontend/src/lib/{receiptExport,webEngine}.ts`, `frontend/src/views/Paper.tsx`):**
  - Extended `GET /api/files/pack?summary=1` (`format=md|html`) to compile recent finished receipts into a `## Verified Work Summary` table/list alongside file SHA-256 digests and verified `[file:L##]` citation ratios (`100% Verified` vs `Stale Line Drift`).
  - Added 1-click **Client Summary (.md)** (`#export-summary-md-btn`) and **Summary (.html)** (`#export-summary-html-btn`) buttons in Today's **Finished** header with full `webEngine.ts` standalone parity.

## 0.18.0 - 2026-11-25

- **Citation Freshness Pre-Flight Gate & 1-Click Verified Client Handoff (`.html`) (`src/tools.rs`, `src/server.rs`, `frontend/src/lib/{receiptExport,webEngine}.ts`, `frontend/src/shell/InspectorDrawer.tsx`):**
  - Extended `verify_text_citations_against` & `PackPreflight` with live SHA-256 digests (`file_sha256`), live line excerpts (`line_excerpt`), and `.history/` / baseline SHA-256 drift detection (`stale` vs `verified` status, `100% Verified` vs `Stale Line Drift` gate).
  - Added `GET /api/files/pack?format=html` (`build_workspace_handoff_html`) and `#export-handoff-html-btn` (**Client Handoff (.html)**) generating standalone printable HTML deliverables with SHA-256 manifest, live citation freshness gate, optional delta, and strict CSP (`default-src 'none'; style-src 'unsafe-inline'`), with 100% `webEngine.ts` parity.
- **Release Binary & Warm Audit-Bundle Peak RSS Diet (`Cargo.toml`, `src/search.rs`, `src/audit_bundle.rs`, `src/server.rs`):**
  - Configured `[profile.release.package."*"] opt-level = "z"` (keeping `ergon-rs`, `serde_json`, and `sha2` at `opt-level = 3`), cutting cold VM/parser/TLS bloat from the release binary.
  - Added `SearchCache::get_if_fresh` + `audit_bundle::build_audit_bundle_json_bytes_with_chain` and zero-alloc `compute_workspace_manifest_sha256` hashing so warm `/api/audit/bundle` calls borrow cached receipt chains with zero duplicate 5k-entry allocation.

## 0.17.0 - 2026-11-25

- **1-Click Run / Receipt Comparison & Delta Client Pack (`src/file_history.rs`, `src/server.rs`, `frontend/src/shell/{InspectorDrawer,WorkspaceFiles}.tsx`, `frontend/src/lib/{receiptExport,webEngine}.ts`):**
  - Added `file_history::diff_texts` and `GET /api/receipts/compare?a=<id>&b=<id>` computing exact line-by-line summary deltas (`+added / -removed / % similarity`), added/removed deliverable files, and unified Markdown diff blocks.
  - Extended `GET /api/files/pack` with `delta_a`/`delta_b` (run delta) and `delta_file`/`rev_a`/`rev_b` (`.history/` snapshot delta) and added 1-click **Append Delta to Client Pack (.md)** buttons inside both `InspectorDrawer` (**Compare runs**) and `FileHistoryDiffView`, with 100% `webEngine.ts` standalone parity.
- **Unified Fuzzy `Cmd+K` Jump Across Files, Citations, Playbooks & Receipts (`frontend/src/shell/CommandPalette.tsx`):**
  - Added scored fuzzy search (`fuzzyScore`) across workspace files, extracted `[file:L##]` source citations (`onOpenFile("file#L##")`), multi-step playbooks & routines (showing step counts), workflows, and receipts.

## 0.16.0 - 2026-11-25

- **Cross-File Citation Verification & Clickable `[file:L##]` Source Inspector (`src/tools.rs`, `src/server.rs`, `frontend/src/chat/Entry.tsx`, `frontend/src/shell/WorkspaceFiles.tsx`, `frontend/src/shell/InspectorDrawer.tsx`, `frontend/src/lib/webEngine.ts`):**
  - Added `verify_text_citations` & `POST /api/citations/verify`, and integrated automatic `[file:L##]` cross-file citation verification directly into `GET /api/files/pack?preflight=1` and `Client Pack (.md)` exports (flagging missing files or out-of-range line references before export).
  - Rendered clickable `[file:L##]` citation pills inside finished receipts, replies, and Pack Pre-Flight summaries that open `FileViewerModal` in line-numbered Source Inspector mode scrolled & highlighted directly to the cited line range (`webEngine.ts` 100% synced).
- **Binary & Peak RSS Diet (`Cargo.toml`, `build.rs`, `src/ui.rs`, `src/audit_bundle.rs`, `src/server.rs`):**
  - Enabled fat LTO (`lto = "fat"`) in `[profile.release]` and pre-gzipped `assets/index.html` into `.rodata` (`index.html.gz`) inflated once on demand, trimming the release binary below `11.8 MB`.
  - Added zero-DOM streaming `audit_bundle::build_audit_bundle_json_bytes` serializing 5k+ `receipt_chain.entries` directly into `ShaWriter` & response bytes without allocating `5,000` `Value::Object` maps, slashing Peak `VmHWM` under 5k receipts + 500 workspace files below `29 MB`.

## 0.15.0 - 2026-11-25

- **Pre-Gzipped `.rodata` `/chunks/*` (`build.rs`, `Cargo.toml`, `src/ui.rs`, `src/server.rs`):**
  - Pre-compressed all lazy `.js` and `.css` chunks at compile time via `miniz_oxide` inside `build.rs` and served zero-copy directly from `.rodata` (eliminating runtime `CHUNK_CACHE` heap allocations and trimming release binary size), with transparent on-the-fly inflation fallback for non-gzip clients.
- **Streaming Receipts & Zero-Clone Audit Bundle RSS Diet (`src/receipts.rs`, `src/search.rs`, `src/audit_bundle.rs`, `src/server.rs`):**
  - Added streaming `receipts::load_chain` parsing `work.json` and `receipts.json` one JSON object at a time (~2 KB transient buffer instead of materializing two full `Value` DOMs) and streamed `compute_canonical_bundle_sha256` directly into `Sha256` without cloning 5k-entry arrays, reducing Peak `VmHWM` under 5k receipts + 500 workspace files.
- **1-Click Playbook Promotion & Deliverable Pre-Flight Check (`frontend/src/`, `src/server.rs`, `frontend/src/lib/webEngine.ts`):**
  - Upgraded run saving into 1-click **Promote to Playbook** (`extractPlaybookFromRun` auto-extracting tool steps, input globs like `*.csv`/`*.md`, and `pack_deliverables`), and added `GET /api/files/pack?preflight=1` + inline **Pre-Flight Check** verifying non-empty deliverables, row/line counts, and SHA-256 digests prior to Client Pack export (`webEngine.ts` 100% synced).

## 0.14.0 - 2026-11-25

- **Initial-Shell `React.lazy` Code-Splitting & Compile-Time `/chunks/*` (`build.rs`, `src/ui.rs`, `src/server.rs`, `frontend/vite.config.ts`, `frontend/src/App.tsx`, `frontend/src/chat/LiveRunTimeline.tsx`, `frontend/src/views/usePaper.ts`):**
  - Split `PaperFace`, `GraphFace`, `FileViewerModal`, `InspectorDrawer`, `SettingsPanel`, `SkillsPanel`, `RoutinesPanel`, `CommandPalette`, and secondary `Panels` into lazy-loaded `/chunks/*.js` + `.woff2` + `.css` assets (+ shared vendor chunk) embedded at compile time via `build.rs` and served with cached RFC 1952 `gzip`, strong `ETag` + `304`, `Vary: Accept-Encoding`, `Cache-Control: public, max-age=31536000, immutable`, and strict security headers (`nosniff`/`DENY`).
  - Reduced initial `GET /` shell from `1,054 KB` (`~375 KB` gzip) down to `148,263 B` (`144.8 KiB`) raw / `33,291 B` (`32.5 KiB`) gzip (-91% initial shell wire transfer) while keeping `npm run build:web` 100% single-file (`ergon-web.html`, `0` external requests).
- **Deterministic Approval UX & Zero-Effort Onboarding (`frontend/src/App.tsx`, `frontend/src/chat/Entry.tsx`, `frontend/src/chat/Chat.tsx`, `frontend/src/shell/InspectorDrawer.tsx`):**
  - Fixed `handleResolveDecision` & `WaitingEntry` pending decision matching so approvals emit immediate deterministic `"Approved"` feedback toasts and refresh Today + active thread in parallel (`Promise.all`).
  - Added 1-click **Load sample workspace** (`#starter-pack-btn`) for empty workspaces and 1-click **Package & Verify (.md)** (`#package-verify-btn`) combining hash-chain verification (`/api/receipts/verify`) with SHA-256 Client Deliverable Pack download.
- **Peak RSS Diet & Thread Stack Tuning (`src/search.rs`, `src/server.rs`):**
  - Eliminated duplicate `receipt_lower: Vec<String>` allocations from `PersonIndex` via zero-alloc `contains_icase` field checks, capped file snippet previews at 512 chars, called `shrink_to_fit()` on inverted token posting lists, and tuned HTTP worker pool stack size to `2 MB`.

## 0.13.0 - 2026-11-25

- **Multi-Step Workflow Chaining & 1-Click Workspace Playbooks (`src/routines.rs`, `src/server.rs`, `frontend/src/shell/RoutinesPanel.tsx`, `frontend/src/lib/webEngine.ts`):**
  - Extended routines into sequential **Workspace Playbooks** (`steps`, optional `expected_inputs`, and `pack_deliverables`) executed step-by-step in one shared thread context via `POST /api/routines/:id/run`, optionally assembling `"Playbook Deliverable Pack.md"` via shared `build_workspace_pack`.
  - Added 1-click Playbook presets (**Month-End Close (3 steps)**, **Contract Audit + Pack (2 steps)**), interactive step chaining (`+ Chain step`), and `"Auto-bundle Deliverable Pack (.md)"` in `RoutinesPanel.tsx`, with matching standalone browser shim parity in `webEngine.ts`.
- **Indexed Search Speed & `src/tools.rs` Modularization (`src/search.rs`, `src/tools.rs`, `src/lib.rs`, `src/tripwire.rs`):**
  - Reused one token scratch buffer across 10k-receipt/500-file index builds in `src/search.rs`, replaced temporary `HashSet` query intersections with zero-alloc sorted `binary_search` narrowing (`narrow_candidates`), and routed filtered `list_receipts` / `list_files` queries through the inverted token indexes (verified up to 10,000 receipts + 500 files).
  - Extracted sandboxed workspace tools, URL egress fetchers, `export_workspace_file`, and honesty-check helpers out of `src/lib.rs` into cohesive `src/tools.rs`, and upgraded cached `DecoratedPage` pre-compression in `src/tripwire.rs` to level 9.
- **Frontend Bundle Diet (`frontend/src/fonts.css`, `frontend/vite.config.ts`, `frontend/src/shell/WorkspaceFiles.tsx`):**
  - Consolidated Geist weights (`400`, `500 600` on `Geist-Medium.woff2`) + `Geist Mono`, targeted `es2022` in Vite, and pruned dead `WorkspaceFilesPanel`, shrinking embedded `assets/index.html` to `1,054 KB` raw / `~375 KB` gzip (`~366 KiB`).

## 0.12.0 - 2026-11-25

- **Modular HTTP Server & Atomic Persistence (`src/server.rs`, `src/lib.rs`, `src/routines.rs`, `src/scheduler.rs`, `src/file_history.rs`):**
  - Extracted the embedded HTTP server, request parser, security gates, and domain route handlers (`try_route_system`, `try_route_files_and_audit`, `try_route_knowledge_and_routines`, `try_route_runs_and_chat`) out of `src/lib.rs` into a cohesive `src/server.rs` module while keeping all public crate exports intact.
  - Standardized config, thread store, routine store, scheduler run history, and `.history/` manifests/blobs onto unique-temp atomic writes (`atomic_write_bytes` / `atomic_write_bytes_with_mode` preserving `0o600` on saved API keys).
  - Added `GET /api/files/pack` (with corresponding `frontend/src/lib/webEngine.ts` standalone parity) to assemble verified SHA-256 Client Deliverable Packs (`.md`) with sandbox containment and symlink rejection.
- **Zero-Effort User Value & Frontend Bundle / 60fps Optimizations (`frontend/src/`, `rune/used/`):**
  - Added 1-click **Save as routine** actions directly on finished deliverables (`FinishedEntry` in `Entry.tsx`) and finished receipt rows in `Today` (`PaperFace` in `Paper.tsx`), plus 1-click **Client Pack (.md)** batch deliverable downloads in `FinishedEntry` and `InspectorDrawer` Audit tab.
  - Enhanced composer drag-and-drop / multi-file uploads (`Chat.tsx`, `detectBatchWorkflow` in `words.ts`) with interactive file queue chips (inspect / remove individual files) and 1-click multi-file batch workflow actions (`Reconcile N sheets`, `Compare N documents`, `Summarize N files`).
  - Switched `motion/react` to `<LazyMotion features={domAnimation} strict>` + `m` across `App.tsx`, `InspectorDrawer.tsx`, `text-shimmer.tsx`, and `border-trail.tsx` (trimming JS & single-file HTML bundle size), memoized heavy workbench panes (`PaperFace`, `InspectorDrawer`, `CommandPalette`, `FinishedEntry`), and replaced `Today`'s eager `?limit=5000` fetch with fast `/api/receipts?limit=200` initial load + debounced server search and on-demand load-all.

## 0.11.0 - 2026-11-25

- **Faster Runtime, Compression & Release Binary Diet (`Cargo.toml`, `src/lib.rs`, `src/tripwire.rs`, `src/search.rs`):**
  - Added `[profile.release]` (`opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`), shrinking the compiled binary while boosting throughput.
  - Served embedded `assets/index.html` with pre-compressed RFC 1952 `gzip` (`Accept-Encoding: gzip`), strong `ETag` + `304 Not Modified`, `Vary: Accept-Encoding`, `Cache-Control: no-cache`, and full security headers (`X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, CSP).
  - Replaced per-connection `thread::spawn` with a warm elastic worker pool, enabled `TCP_NODELAY`, coalesced HTTP headers + body into a single socket write with preformatted `BASE_HEADERS_BLOCK`, eliminated `TcpStream::try_clone()` and per-header `.to_lowercase()` allocations in `read_request`, converted `sessions`/`runtimes`/`people`/`decorated` locks to `RwLock`, added zero-alloc fast paths in `SearchCache::list_receipts` and `list_files`, and included precomputed thread `state` in `GET /api/threads` (`Store::list_json`) so the frontend skips up to 40 N+1 `/api/threads/:id` fetches on load.
- **Zero-Friction User Velocity & Side-by-Side Diff (`frontend/src/`):**
  - Added context-aware workflow auto-detection (`detectWorkflowForFile`, `detectWorkspaceWorkflow`) that binds starter cards to real workspace files, surfaces a 1-click suggested workflow bar immediately after drag-and-drop/file upload, adds a 1-click contextual workflow button in `FileViewerModal`, and exposes recommended actions in `Cmd+K`.
  - Added an instant **Unified | Side-by-side** diff toggle in `FileHistoryDiffView` before restoring `.history/` snapshots, fixed `/api/receipts/verify` chain-count parsing in `InspectorDrawer`, wired server-side SHA-256 receipt export into chat `FinishedEntry`, added a 1-click **Copy summary** deliverable action, and isolated the live elapsed timer inside `LiveElapsedClock` to eliminate 4x/sec full-app React re-renders during active runs.
- **Hardening (`src/lib.rs`, `src/file_history.rs`):**
  - Rejected duplicate `Host` headers with HTTP `400 Bad Request`, bounded `LiveRun::steps` (`MAX_LIVE_STEPS = 128`), blocked hidden dotfiles in `.history/` (`validate_history_path`), and guarded `export_workspace_file` against overwriting workspace symlinks.

## 0.10.0 - 2026-11-25

- **Unified Responsive Desktop Workbench:** Eliminated the duplicate 1.2 MB `assets/phone.html` asset and `ui::PHONE_HTML`, serving one unified responsive SPA (`assets/index.html`, 972 kB — a ~59% reduction from 2.39 MB combined) cleanly across `/`, `/phone`, desktop, tablet, and mobile.
- **Claude-Desktop-caliber 3-Pane Workspace:** Rebuilt the primary workspace around a serene personal employee desk: slim top bar with active company/workspace pill (`#person-select`), view switcher (`Chat`, `Today`, `Work graph`), collapsible Left Rail (Conversations, `/` search, Scheduled Routines quick drawer with 1-click cadence presets), centered Thread with 4 business deliverable starter cards (Word `.docx`, Excel `.xlsx`, Contract Audit, Executive Brief), and a collapsible Right Workspace Inspector Drawer (`Files`, `History & Diff`, `Audit & Offers`).
- **Inline Approvals, File Diffs & Instant Deliverables:** Waiting approval cards (`WaitingEntry`) now render directly inside the conversation thread with **Approve**, **Hold / Keep**, **Revise**, and inline **Preview diff** without jumping tabs; completed deliverables offer 1-click **Export Word (.docx)**, **Export Excel (.xlsx)**, **SHA-256 Receipt (.md/.json)**, and **Audit Bundle (.md/.json)**.
- **Lean, Accessible Design System & Gates:** Pruned unused wrapper libraries (`motion/react`, `MorphingDialog`, `SlidingNumber`, `BorderTrail`, `TextShimmer`, `AnimatedBackground`, `Expandable`, `Tracker`, `@dnd-kit`, `recharts`, `date-fns`, `jotai`, `lodash.throttle`, `tunnel-rat`), kept `#67` Settings-only Light/Dark switching and `#155` zero `"Rune"` user-facing jargon, fixed `--input` border contrast (`>= 3:1`), and enforced DOM Range zero-text-clipping and zero-overflow UI gates (`sloplint --scope app --render` 0 FAIL).
- **Backend HTTP & Filesystem Hardening (`src/lib.rs`, `src/skills.rs`):** Rejected conflicting/invalid `Content-Length` and `Transfer-Encoding: chunked` requests with HTTP `400`, enforced case-insensitive `Host`/`Origin` matching, bounded active HTTP connections (`MAX_ACTIVE_CONNS = 128` returning HTTP `503`), guarded `POST /api/files/upload` against symlinked targets with atomic `FILE_TMP_SEQ` writes across all workspace/export/workflow paths, and skipped loading `PersonRuntime` in `start_scheduler_thread` when no `routines.json` exists.

## 0.9.0 - 2026-11-24

- **Reads the files employees actually have (`src/docs.rs`):** PDF (page by page, including CID/ToUnicode fonts from Word, Google Docs and browsers), Word `.docx` (as Markdown with headings, numbered and bulleted lists, tables, link targets), Excel `.xlsx` (each sheet as CSV with real dates, booleans and cached formula values; `book.xlsx#Sheet` selects a sheet) and PowerPoint `.pptx` (slides plus speaker notes). One cached extractor serves the agent's `read_file` and `search_files`, all 12 built-in workflows (`host_workspace_read`), the file viewer and the search index — `audit_contract` on a Word/PDF contract and `reconcile_ledger` on an Excel workbook now just work.
- **Share-ready deliverables:** new `export_file` tool, `POST /api/files/export`, and **Save as Word / Save as Excel** in the file viewer: Markdown → styled `.docx` (Heading styles, real bullet/numbered lists, tables with header rows and alignment, bold/italic/code, live hyperlinks, quotes, code blocks) and CSV/TSV → `.xlsx` (typed numbers with thousands/currency/percent formats, ISO dates as real dates, leading-zero IDs kept as text, bold frozen header, autofilter, fitted column widths). Validated by LibreOffice, python-docx and openpyxl.
- **Fixed: uploading a PDF, Office file or image silently corrupted it** (the desk read every file as UTF-8 text). Uploads now travel byte-exact (base64) and may be up to 25 MB (the upload route alone gets a larger body limit and a longer body deadline; every other route keeps its 1 MB cap). Downloads of files over 2 MiB work again (the limit is for previews only).
- **Long files are read in full:** `read_file` returns ~5,800-character windows with an explicit `offset` to read on, instead of silently cutting at the tool-result limit.
- **`reconcile_ledger` on real workbooks:** sheet summary rows ("Total", "Subtotal", …) are skipped and listed instead of doubling the ledger; a new **Possible Duplicate Charges** check flags the same date, vendor and amount twice in one file (rows with different references are never flagged).
- **Resilient model calls:** HTTP 408/425/429/5xx/52x and dropped connections to remote endpoints are retried with backoff (honouring `Retry-After`, capped, inside a 150 s budget); a local gateway that is not running still fails fast.
- **Tidy memory:** a note already in memory is not added again; prompts carry the person's identity plus their most recent learning once memory grows past ~10,000 characters (the file keeps everything).
- Verification: 16 new tests (`tests/documents_v090.rs`) on fixtures from real producers (python-docx, pandoc, openpyxl re-saved by LibreOffice, python-pptx, reportlab, Chromium — regenerate with `tests/fixtures/docs/make_fixtures.py`), including zip-bomb and malformed-PDF safety, byte-exact HTTP uploads, and stub models that fail with 503/400; mutation-checked. 200 Rust tests and the Playwright UI gate (69 + 6 + 20) pass.

## 0.8.0 - 2026-11-24

- **Security Tripwires — honeytraps throughout (`src/tripwire.rs`, `GET /api/security`, `POST /api/security/policy`):** OWASP AppSensor-style detection points with automatic, proportionate responses. Perimeter: 70+ decoy resources (`/.env`, `/.git/config`, `/wp-login.php`, `/api/login`, `/api/internal/*`, …) answering with plausible decoys that carry unique HMAC-verifiable `sk-ergon-…` honeytokens (Canarytokens-style attribution of every placement), `robots.txt` + page-source breadcrumbs, and a decoy `ergon_role` cookie; presenting a honeytoken or tampering with the cookie is refused and banned with escalating, decaying fail2ban-style scoring (remote addresses only — loopback is never locked out). Refused cross-site/DNS-rebinding requests and wrong gate tokens are now recorded and reported instead of silently dropped.
- **Agent-side honeytraps:** each run's system prompt carries a unique bait key and a content-is-data rule; tool output is fenced off with a per-run random boundary (Microsoft Spotlighting) and flagged when it carries injection phrasing. A new egress guard checks every outbound fetch (agent `fetch_url`, the research workflow, library `host_fetch_url`): bait keys and real secrets never leave in any encoding, an SSRF-safe ureq resolver vets RESOLVED addresses on every redirect hop (cloud metadata and Ergon's own API always refused; private networks only with the person's own words or a one-tap allow). Sandbox escapes aimed at credentials or Ergon's data stop the run; weaker signals (phantom `bash`/`send_email` tools, injected content) add to a per-run threat score that stops the run at 100.
- **Memory & library supply-chain traps:** library (non-built-in) skills can no longer rewrite a person's memory; suspicious `update_memory` notes and skills that both read workspace/memory data and fetch URLs are held as Today decisions instead of being written/published to every colleague.
- **One plain Today card per incident** with the likely source and a one-tap follow-up (quarantine the file, block the site, allow the address, remember the note, publish the skill), most urgent first; Settings → Security shows armed traps, recent events, scanner bans and allowed/blocked addresses.
- **Tamper-evident security log:** `security_events.json` is SHA-256 hash-chained, coalesced and capped; breaks are detected and reported (HT1); the Field-Book Audit Bundle now carries a verified `security` section and `integrity.security_events_sha256`.
- **Response hardening:** desk pages carry a CSP pinning their own inline scripts by SHA-256, plus `X-Frame-Options: DENY`, `nosniff`, `Referrer-Policy: no-referrer`, COOP/CORP and `Permissions-Policy`; API JSON is `Cache-Control: no-store`.
- Fixed: the "Today so far" stream showed decision kinds as raw JSON (`Waiting on you ("delete")`).
- Fixed: `update::tests::errors_name_files_never_folders` failed when run as root (directory permissions are not enforced for root); it now probes and skips honestly.
- Verification: 18 new end-to-end tripwire proofs (`tests/tripwire.rs`) against a stub model that plays a hijacked agent, 13 tripwire unit tests (including the RFC 4231 HMAC vector), and the Playwright UI gate (69 + 6 + 20 checks) under the new CSP.

## 0.7.0 - 2026-11-22

- **Background Routine Scheduler & Atomic Run History (`src/scheduler.rs`, `PATCH /api/routines/:id`, `GET /api/routines/:id/history`):** Real background cadence ticker (`15s` default, opt-out via `ERGON_DISABLE_SCHEDULER=1`, SSE `routine_fired` broadcast), exact UTC calendar math (`every 15m`, `hourly`, `daily 09:00`, `weekly mon`, or `interval_secs`), pause/resume toggles, live `Next run` countdowns, and atomic per-person `routine_history.json` records with linked receipts.
- **Automatic File Version History, Unified Diff & Reversible Restore (`src/file_history.rs`, `GET /api/files/history`, `POST /api/files/restore`):** Traversal-proof `.history/` revision snapshots recorded automatically on file upload, `write_file`, workflow write, and approved delete, with SHA-256 blob verification, LCS unified diff hunks, and atomic `pre-restore` version recovery inside `FileViewerModal`.
- **Tamper-Evident Field-Book Audit Bundle (`src/audit_bundle.rs`, `GET /api/audit/bundle?format=json|md`):** One-click signed JSON and Markdown audit export combining verified SHA-256 receipt chains, workspace file digests, standing memory digest, routine history, and independent offline verification instructions.
- **Mobile-Safe Workspace Switcher (`POST /api/people/switch`, `WorkspaceSwitcherPanel`):** Lifted outside mobile Vaul drawers (`z-50` over `z-40`) with Settings and `Cmd+K` access, plus full v0.7.0 browser-edition parity (`IndexedDB` file history/diff/restore, routine scheduler/history, audit bundle export).
- **Standalone Linux/Windows/macOS Chromeless Desktop App (`src/main.rs`):** Launches Ergon directly in a dedicated chromeless desktop window (`chromium`/`google-chrome`/`brave` `--app=` on Linux, `msedge.exe`/`chrome.exe` `--app=` on Windows, Chrome/Edge `--app=` on macOS) with CLI flags `--app`/`--open`, `--no-open`, `--host`, `--port`, `--help`, `--version`, automatic headless/CI/systemd suppression (`ERGON_NO_OPEN=1`), and verified `x86_64-pc-windows-msvc` release builds.

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
