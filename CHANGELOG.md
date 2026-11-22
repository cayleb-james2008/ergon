# Changelog

Each release is tagged `vMAJOR.MINOR.PATCH` and published on GitHub Releases with
binaries and a `SHA256SUMS` file. The running app shows its version and can update
itself from the newest release (see README, "Versions and updates").

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
