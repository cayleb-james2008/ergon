# Changelog

Each release is tagged `vMAJOR.MINOR.PATCH` and published on GitHub Releases with
binaries and a `SHA256SUMS` file. The running app shows its version and can update
itself from the newest release (see README, "Versions and updates").

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
