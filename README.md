# Ergon (Rust + Rune restart)

An AI agent for every employee, personal to each person, that learns how
they work and **finishes their digital work autonomously** — the user
states a task in plain words, the agent works it end to end through real
tools, and the finished result lands in the thread. No approval ceremony
in the core loop.

Fresh codebase (17 Nov 2026 restart), built apart from the original app.

## Run

```sh
export PATH="$HOME/.local/rust/bin:$PATH"
CARGO_BUILD_JOBS=2 cargo build
ERGON_MODEL_API_KEY=... ./target/debug/ergon   # http://127.0.0.1:8787
```

Without `ERGON_MODEL_API_KEY` the UI shows an honest "no model key
configured" banner and `/api/chat` refuses (HTTP 503) — Ergon never
fabricates an answer.

## Config (env)

| Var | Default |
| --- | --- |
| `ERGON_HOST` / `ERGON_PORT` | `127.0.0.1` / `8787` |
| `ERGON_DATA_DIR` | `./data` (holds `memory.md`) |
| `ERGON_WORKSPACE` | `<data>/workspace` — all file tools are sandboxed here |
| `ERGON_MODEL_BASE_URL` | `https://opencode.ai/zen/go/v1` |
| `ERGON_MODEL_ID` | `deepseek-v4.1-flash` |
| `ERGON_MODEL_API_KEY` | — (bearer key; every model call also sends `x-opencode-session` + a browser User-Agent) |

## How it works

- **Shell**: embedded HTML/CSS/JS (no framework, no build step) —
  left sidebar with thread list + New chat, one scrolling thread,
  composer pinned at the bottom. Agent steps appear as thread messages;
  the finished result is the final message.
- **Agent loop** (`src/lib.rs`): `POST /api/chat {thread_id, message}`.
  The model replies with one JSON object per turn —
  `{"tool": name, "args": {...}}` or `{"done": true, "summary": "..."}` —
  the loop executes the tool for real, feeds the result back, and
  repeats (cap 24 steps). Tools: `list_dir`, `read_file`, `write_file`
  (sandboxed), `fetch_url` (HTML→text), `write_document` and
  `update_memory` (Rune skills).
- **Rune skills** (`src/skills/*.rn`, host layer in `src/skills.rs`):
  the document-writing and memory-update skills are Rune scripts that
  format their output and act only through registered host functions
  (workspace fs read/write/list, memory read/write, HTTP fetch).
- **Memory**: `memory.md` in the data dir is included in the model's
  context at the start of every task; the agent updates it via
  `update_memory` when it learns a preference or fact.

## Proof

```sh
CARGO_BUILD_JOBS=2 cargo test
```

`tests/e2e.rs` runs a stub model server (test fixture only) speaking
scripted chat-completions replies over the real ureq HTTP path, and
proves end to end: messy notes → `list_dir` → `read_file` ×2 →
`write_document` (Rune) → `update_memory` (Rune) → done, asserting the
merged `Summary.md`, the learned memory note, the transcript, and the
wire headers. Plus the no-key honesty test and the shell test.
