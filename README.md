# Ergon

[![latest release](https://img.shields.io/github/v/release/cayleb-james2008/ergon)](https://github.com/cayleb-james2008/ergon/releases)

An AI agent desk for every employee — personal to each person, it learns how
they work and **finishes their digital work autonomously, end to end**.
The person states a task in plain words; the agent works it through real
sandboxed tools and Rune skills, and the finished result lands as a verified
Field-Book receipt. No approval ceremony in the core loop.

A single Rust binary serves the whole product: the embedded web desk (`/` and
`/phone`), the agent loop, the sandboxed tool layer, per-person standing
memory, and the Rune skill runtime. No external runtime or database server.

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
CARGO_BUILD_JOBS=2 cargo build
ERGON_MODEL_API_KEY=... ./target/debug/ergon   # http://127.0.0.1:8787
```

Without `ERGON_MODEL_API_KEY` the UI shows an honest "not connected to a
model yet" banner and `/api/chat` refuses (HTTP 503) — Ergon never
fabricates an answer. The model route is any OpenAI-compatible
chat-completions endpoint; the default is OpenCode Go with
`deepseek-v4.1-flash`.

## Config (env)

| Var | Default |
| --- | --- |
| `ERGON_HOST` / `ERGON_PORT` | `127.0.0.1` / `8787` |
| `ERGON_ALLOWED_HOSTS` | — (comma-separated host names, with or without ports, case-insensitive; extra Host names accepted when no gate token is set, e.g. a LAN name for `/phone`. Loopback is always accepted) |
| `ERGON_DATA_DIR` | `./data` (people, threads, per-person workspaces) |
| `ERGON_WORKSPACE` | `<data>/workspace` — the default person's sandbox root |
| `ERGON_MODEL_BASE_URL` | `https://opencode.ai/zen/go/v1` |
| `ERGON_MODEL_ID` | `deepseek-v4.1-flash` |
| `ERGON_MODEL_API_KEY` | — (bearer key; every model call also sends `x-opencode-session` + a browser User-Agent) |
| `ERGON_GATE_TOKEN` | — (when set, every request must carry the matching `X-Ergon-Gate` header; this is the hosting layer's gate — Ergon itself has no accounts or passwords) |

## How it works

- **Desk shell** — self-contained single-file bundles in `assets/`
  (`index.html`, `phone.html`) with self-hosted **Geist** and **Geist Mono**
  fonts, light-first House Frontend tokens, and full dark-mode parity. A
  256px left rail (a gesture-friendly Vaul drawer under 768px and on `/phone`)
  holds the active person desk, primary **New chat** action, `/` conversation
  search filter, gliding conversation list, **What my agent has learned** and
  **My suite and the library** dialogs with live counts, **Add a person**,
  Light/Dark segmented switch, and version/update controls. The top command
  bar switches across **Chat**, **Today** (with live pending-decision count),
  and **Work Graph**, plus `Cmd/Ctrl+K` to jump straight to the task box.
- **Chat & Field-Book entries** — empty chats present role-tailored starter
  task cards with skill/tool chips. Active runs display a live shimmering step
  ledger; completed runs settle into ruled Field-Book Result Cards with
  file-type glyphs, saved workspace file pills, `Geist Mono` step counts, and
  an expandable raw tool-step inspector.
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
- **Tools** — `list_dir`, `read_file`, `write_file`, `search_files`,
  `move_file` (all sandboxed to the person's workspace), `fetch_url`
  (HTML→text), and four Rune skills: `write_document`,
  `make_spreadsheet`, `research`, `update_memory`, plus `browse_library`,
  `install_item`, `publish_skill`, `delegate`, `delete_file` (which parks a
  decision instead of deleting directly), and `flag_decision`. One run draws
  from a single shared budget of 24 model calls.
- **Sandbox** — every path is resolved through a component-walking
  resolver that re-checks containment at each step, follows at most 16
  symlink hops, and treats a symlink as an object in its own right
  (moving a link moves the link). Escapes — `..`, absolute paths,
  symlink chains, sibling-prefix tricks — are refused, and the test
  suite pins each one.
- **Learning** — each person's `memory.md` (including standing
  preferences) is injected into the model's context at the start of every
  task, and updated via `update_memory`.
- **Company library & personal suites** — shared library of skills,
  subagents, and one-tap apps (`library.json`, seeded with twelve items).
  Published Rune skills are compile-gated and ownership-checked, running
  under an instruction budget and memory limit.
- **Decisions & proactive offers** — `delete_file` parks an approval item
  in **Today**; only explicit approval deletes the exact path, rewriting the
  originating thread note. Repeating a task pattern or producing three
  spreadsheets earns a proactive one-tap app offer.
- **Honesty machinery** — completion summaries are verified against files
  actually written in the workspace (`check_mentions`). Requests over 1 MB
  return HTTP 413; heads over 32 KiB return 431; cross-site or non-JSON
  writes are refused; corrupt JSON stores are backed up to `.bak` and
  reported in `store_recovered`.

## Verification

```sh
CARGO_BUILD_JOBS=2 cargo test          # 101 Rust unit + E2E tests
cd frontend && npm run check:ui        # Playwright E2E gate, graph probe, and worst-case replay
```

`tests/e2e.rs` drives a local stub chat-completions server over real HTTP to
verify messy notes -> `list_dir` -> `read_file` -> `write_document` (Rune) ->
`update_memory` (Rune) -> done, alongside sandbox containment, session
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
