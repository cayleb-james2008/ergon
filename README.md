# Ergon

[![latest release](https://img.shields.io/github/v/release/cayleb-james2008/ergon)](https://github.com/cayleb-james2008/ergon/releases)

An AI agent desk for every employee — personal to each person, it learns how
they work and **finishes their digital work autonomously, end to end**.
The person states a task in plain words; the agent works it through real
sandboxed tools and built-in workflows, and the finished result lands as a verified
Field-Book receipt. No approval ceremony in the core loop.

A single Rust binary serves the whole product: the embedded web desk (`/` and
`/phone`), the agent loop, the sandboxed tool layer, per-person standing
memory, and the embedded workflow runtime. No external runtime or database server.

## Install

```sh
./install.sh [--prefix DIR] [--data DIR] [--no-service]
```

Installs the binary (building it with cargo if no prebuilt binary is
present), writes a launcher (`ergon-serve`) that carries the model key
and data dir, and registers a systemd user service where one is
available. Paths are made absolute and embedded safely — a prefix
with spaces, or a data path containing shell metacharacters, installs
and runs correctly.

## Run

```sh
export PATH="$HOME/.local/rust/bin:$PATH"
CARGO_BUILD_JOBS=2 cargo build --release
./target/release/ergon             # desktop app window on http://127.0.0.1:8787 (auto-detects local gateways)
./target/release/ergon --no-open   # headless server mode (never opens a desktop window)
```

Launching `ergon` directly (or double-clicking `ergon.exe` on Windows, or passing `--app`) opens a chromeless desktop app window (`chromium`/`google-chrome`/`brave --app=` on Linux, `msedge.exe`/`chrome.exe --app=` on Windows) while serving the local workspace on `127.0.0.1:8787`. Window launch is automatically suppressed in headless shells, CI, systemd services, or when `--no-open` / `ERGON_NO_OPEN=1` is set.

On startup (unless `ERGON_NO_AUTO_MODEL=1` or an explicit `ERGON_MODEL_BASE_URL` / `ERGON_MODEL_API_KEY` / `data/config.json` is present), Ergon automatically probes running local OpenAI-compatible gateways on loopback (`127.0.0.1:11434/v1` first, then `127.0.0.1:8790/v1`, then `127.0.0.1:8789/v1`) and connects with no API key required. You can switch providers or enter a remote OpenAI-compatible endpoint and API key live inside **Settings** (`GET/POST /api/config`, persisted in `data/config.json`).

Without a reachable local gateway or `ERGON_MODEL_API_KEY`, the UI shows an honest "not connected to a model yet" banner and `/api/chat` refuses (HTTP 503) — Ergon never fabricates an answer.

## Config (env)

| Var | Default |
| --- | --- |
| `ERGON_HOST` / `ERGON_PORT` | `127.0.0.1` / `8787` |
| `ERGON_ALLOWED_HOSTS` | — (comma-separated host names, with or without ports, case-insensitive; extra Host names accepted when no gate token is set, e.g. a LAN name for `/phone`. Loopback is always accepted) |
| `ERGON_DATA_DIR` | `./data` (people, threads, `config.json`, per-person workspaces) |
| `ERGON_WORKSPACE` | `<data>/workspace` — the default person's sandbox root |
| `ERGON_MODEL_BASE_URL` | auto-probed local gateway (`:11434` / `:8790` / `:8789`), else `https://opencode.ai/zen/go/v1` |
| `ERGON_MODEL_ID` | auto-detected gateway default, else `deepseek-v4.1-flash` |
| `ERGON_MODEL_API_KEY` | — (optional on loopback `127.0.0.1`/`localhost`/`[::1]`; required for remote endpoints) |
| `ERGON_NO_AUTO_MODEL` | `0` (set to `1` to disable startup local gateway auto-probing) |
| `ERGON_NO_OPEN` / `ERGON_OPEN` | `0` (`ERGON_NO_OPEN=1` or `--no-open` disables desktop window launch; `ERGON_OPEN=1` or `--app` forces it when a display is present) |
| `ERGON_GATE_TOKEN` | — (when set, every request must carry the matching `X-Ergon-Gate` header; this is the hosting layer's gate — Ergon itself has no accounts or passwords) |
| `ERGON_FETCH_ALLOW_PRIVATE` | `0` (set to `1` to let the agent open loopback/private-network addresses without asking; by default only addresses named in your own task, or allowed once from Today, open) |
| `ERGON_FETCH_ALLOW_HOSTS` | — (comma-separated hosts the agent may always open even on a private network, e.g. `wiki.corp,10.0.0.5`) |
| `ERGON_TRIPWIRE_NO_BAN` | `0` (set to `1` to record scanners without banning them) |
| `ERGON_TRUST_PROXY` | `0` (set to `1` behind a local reverse proxy so bans and alerts use the right-most `X-Forwarded-For` address) |

## How it works

- **Desk shell** — self-contained single-file bundle in `assets/index.html`
  served over `/` and `/phone` with `gzip` compression, strong `ETag` + `304 Not Modified`,
  self-hosted **Geist** and **Geist Mono** fonts, light-first House Frontend tokens,
  and full dark-mode parity. A 256px left rail (a gesture-friendly Vaul drawer under
  768px and on `/phone`) holds the active person desk, primary **New chat** action,
  `/` conversation search filter, gliding conversation list, **Scheduled routines**,
  **Workflows**, **What my agent has learned**, **My suite**, **Add a person**, and
  **Settings** (live Model provider config, Light/Dark `SegmentedControl`, Security
  tripwires, Audit Bundle, and Update controls), paired with a collapsible right
  **Workspace** inspector (`Files`, `History` side-by-side diff & restore, `Audit`).
  The top command bar switches across **Chat**, **Today** (with live pending-decision
  count), and **Work Graph**, plus `Cmd/Ctrl+K` for instant command palette search.
- **Workspace File Viewer & Upload** — `GET /api/files`, `GET /api/files/content?path=...(&download=1)` (jailed through `sandbox_path_resolving` to block `..` traversal and symlink escapes), and `POST /api/files/upload`. Clicking any saved file pill or **Open** button across Chat, Today, Work Graph, or Workspace files renders real CSV tables, structured Markdown, formatted JSON, or plain text with one-click Download.
- **Chat & Field-Book entries** — empty chats present role-tailored starter
  task cards with skill/tool chips, and the composer supports direct file
  upload into the active workspace. Active runs stream live tool steps via
  `GET /api/runs/status` inside a shimmering step card; completed runs settle
  into ruled Field-Book Result Cards with clickable saved-file pills,
  `Geist Mono` step counts, and an expandable raw tool-step inspector.
- **Today's Desk** — combines the 48-hour Morning Brief, a 14-slot Tremor
  run-outcome `Tracker`, the **Waiting on you** consequential decision queue,
  proactive **Offers from your agent**, finished work with file-type glyphs,
  and the **Today so far** chronological event stream (`/api/today`).
- **Work Graph** — deterministic four-band SVG map (`People`, `Suite items`,
  `Finished work`, `Files`) drawn at 1:1 CSS scale (`>=13px` labels from
  768px up) paired with a `Map & list` / `List only` switch and an accessible
  structured list.
- **People** — every employee gets their own workspace, memory and
  threads (`people.json` + one data directory per person). People can be
  created in the UI; a new person's agent starts from a seeded memory.
  Browser sessions are pinned to a person and never leak across people;
  session ids are 16 random bytes and a forged or unknown cookie is
  replaced, never adopted.
- **Agent loop** (`src/lib.rs`) — `POST /api/chat {thread_id, message}`.
  A non-empty `thread_id` that does not exist gets 404 `no such thread`
  (an empty id starts a new thread).
  The model replies with one JSON object per turn —
  `{"tool": name, "args": {...}}` or `{"done": true, "summary": "..."}` —
  the loop executes the tool for real, feeds the result back, and
  repeats (cap 24 steps).
- **Tools & 12 Built-in Workflows** — `list_dir`, `read_file`, `write_file`, `search_files`, `move_file` (all sandboxed to the person's workspace), `fetch_url` (HTML→text), and twelve deterministic sandboxed workflows (`write_document`, `make_spreadsheet`, `research`, `update_memory`, `reconcile_ledger`, `summarize_meeting`, `audit_contract`, `triage_inbox`, `clean_dataset`, `compare_documents`, `draft_brief`, `extract_tables`, inspectable via `GET /api/skills` and testable in an ephemeral sandbox via `POST /api/skills/test`), plus `browse_library`, `install_item`, `publish_skill`, `delegate`, `delete_file` (which parks a decision instead of deleting directly), and `flag_decision`.
- **Live SSE Streaming, Background Routine Scheduler, File History & Audit Bundles** — `GET /api/runs/stream` streams real-time tool steps, elapsed ms, file writes, approvals, and `routine_fired` events with `POST /api/runs/:id/cancel`; `GET /api/search?q=...` and paginated `GET /api/receipts` / `GET /api/files` use a per-person `RwLock` inverted index tested at 5,000 receipts + 500 workspace files under 30 concurrent clients; `GET /api/files/history` + `POST /api/files/restore` maintain traversal-proof `.history/` snapshots with unified diff and reversible restore on every overwrite/delete; `GET/POST/PATCH/DELETE /api/routines`, `GET /api/routines/:id/history`, and `POST /api/routines/:id/run` run standing schedules automatically (opt-out via `ERGON_DISABLE_SCHEDULER=1`); `GET /api/audit/bundle?format=json|md` exports a tamper-evident Field-Book Audit Bundle combining the SHA-256 receipt chain, file digests, and routine history.
- **Sandbox** — every path is resolved through a component-walking
  resolver that re-checks containment at each step, follows at most 16
  symlink hops, and treats a symlink as an object in its own right
  (moving a link moves the link). Escapes — `..`, absolute paths,
  symlink chains, sibling-prefix tricks — are refused, and the test
  suite pins each one.
- **Learning** — each person's `memory.md` (including standing
  preferences) is injected into the model's context at the start of every
  task, and updated via `update_memory`.
- **Company library & personal suites** — shared library of workflows,
  subagents, and one-tap apps (`library.json`, seeded with twelve items).
  Published workflow scripts are compile-gated and ownership-checked, running
  under an instruction budget and memory limit.
- **Decisions & proactive offers** — `delete_file` parks an approval item
  in **Today**; only explicit approval deletes the exact path, rewriting the
  originating thread note. Repeating a task pattern or producing three
  spreadsheets earns a proactive one-tap app offer.
- **Office & PDF documents** (`src/docs.rs`) — the agent, every built-in
  workflow (`host_workspace_read`), `search_files`, the file viewer and the
  search index read **PDF** (page by page, including the CID fonts modern apps
  emit), **Word** `.docx` (as Markdown: headings, lists, tables), **Excel**
  `.xlsx` (each sheet as CSV with real dates, booleans and cached formula
  values; `book.xlsx#Sheet2` picks a sheet) and **PowerPoint** `.pptx` (slides
  and speaker notes). Long files are read window by window (`read_file` with
  `offset`) instead of being cut off. `export_file` (and **Save as Word / Save
  as Excel** in the viewer, `POST /api/files/export`) turns Markdown into a
  styled `.docx` (headings, lists, tables, links) and CSV into an `.xlsx` (typed
  numbers and dates, bold frozen header, autofilter, fitted widths). Uploads
  travel byte-exact up to 25 MB. Zip containers are read with per-entry and
  total decompression caps; PDF parsing runs isolated behind `catch_unwind` and
  a timeout.
- **Resilient model calls & tidy memory** — rate limits, overloads and gateway
  hiccups (HTTP 408/425/429/5xx, dropped connections to remote endpoints) are
  retried with backoff honouring `Retry-After`; a duplicate memory note is not
  added twice, and prompts carry a bounded slice of a long memory (identity +
  most recent learning).
- **Security tripwires** (`src/tripwire.rs`, `GET /api/security`) — honeytraps
  at every trust boundary, in the OWASP AppSensor tradition: decoy files and
  admin pages (`/.env`, `/.git/config`, `/wp-login.php`, `/api/login` — Ergon has
  no logins) that hand out unique, traceable `sk-ergon-…` keys; breadcrumbs in
  `robots.txt` and the page source; a decoy `ergon_role` cookie. Using a decoy
  key, or editing the cookie, is a confirmed attacker: refused, banned (remote
  addresses only — this computer is never locked out), and named in Today with
  where the key leaked from. Every agent run carries a bait key it is told it
  never needs, and tool output reaches the model fenced off as data between a
  per-run random boundary (Spotlighting). An egress guard vets every outbound
  fetch — the bait key and real secrets never leave (raw, percent, base64 or hex),
  cloud metadata and Ergon's own API are unreachable, and private-network
  addresses open only when your own words named them or you allowed them once.
  Reaching outside the workspace for credentials or Ergon's own data, sending the
  bait key, or a pile-up of weaker signals stops the run; suspicious memory notes
  and library skills that can read your data AND send it out wait for your OK;
  library skills cannot rewrite memory. Each trip is logged in a hash-chained
  `security_events.json` (tamper-evident, exported in the audit bundle) and turns
  into one plain Today card with a one-tap follow-up: quarantine the file, block
  the site, allow the address, keep the note, publish the skill. Pages carry a
  hash-pinned CSP, `X-Frame-Options: DENY` and friends. Design:
  `docs/superpowers/specs/2026-11-24-tripwire-honeytraps-design.md`.
- **Honesty machinery** — completion summaries are verified against files
  actually written in the workspace (`check_mentions`). Requests over 1 MB
  return HTTP 413; heads over 32 KiB return 431; cross-site or non-JSON
  writes are refused; corrupt JSON stores are backed up to `.bak` and
  reported in `store_recovered`.

## Verification

```sh
CARGO_BUILD_JOBS=2 cargo test          # 215 Rust unit, E2E, scale, scheduler/history, 12-workflow, tripwire & document tests
cd frontend && npm run check:ui        # Playwright E2E gate, graph probe, and worst-case replay
```

`tests/e2e.rs` drives a local stub chat-completions server over real HTTP to
verify messy notes -> `list_dir` -> `read_file` -> `write_document` ->
`update_memory` -> done, alongside sandbox containment, session
isolation, done-gate file verification, delete holds, proactive offers,
brief/today/graph assembly, self-updater verification, and crash-safe store
recovery.

## Status

Working software, verified by the automated Rust and browser suites above.

## Versions and updates

**Releasing:** bump `version` in `Cargo.toml` (and `frontend/package.json`),
add a `## X.Y.Z - date` entry to `CHANGELOG.md`, and merge to `main`.
`.github/workflows/release.yml` builds Linux (`x86_64`, `aarch64`), macOS
(`aarch64`), and Windows (`x86_64`) binaries and publishes the GitHub Release
`vX.Y.Z` with `SHA256SUMS`.

Self-update (`src/update.rs`): checks GitHub Releases, verifies SHA-256 and
`--version`, and swaps the binary only from loopback on released builds.

| Var | Default |
| --- | --- |
| `ERGON_UPDATE_MODE` | `notify` (check and tell); `auto` also downloads and installs (a restart is still yours to press); `off` never contacts GitHub |
| `ERGON_UPDATE_REPO` | `cayleb-james2008/ergon` |
| `ERGON_UPDATE_API` | `https://api.github.com` (plain http only for a loopback address) |
