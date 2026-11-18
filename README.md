# Ergon

An AI agent for every employee — personal to each person, it learns how
they work and **finishes their digital work autonomously, end to end**.
The person states a task in plain words; the agent works it through real
tools and the finished result lands in the thread. No approval ceremony
in the core loop.

A single Rust binary serves the whole product: the web shell, the agent
loop, the tool layer, the per-person memory, and the Rune skill runtime.
No framework, no build step, no database server.

## Run

```sh
export PATH="$HOME/.local/rust/bin:$PATH"
CARGO_BUILD_JOBS=2 cargo build
ERGON_MODEL_API_KEY=... ./target/debug/ergon   # http://127.0.0.1:8787
```

Without `ERGON_MODEL_API_KEY` the UI shows an honest "no model key
configured" banner and `/api/chat` refuses (HTTP 503) — Ergon never
fabricates an answer. The model route is any OpenAI-compatible
chat-completions endpoint; the default is OpenCode Go with
`deepseek-v4.1-flash`.

## Config (env)

| Var | Default |
| --- | --- |
| `ERGON_HOST` / `ERGON_PORT` | `127.0.0.1` / `8787` |
| `ERGON_DATA_DIR` | `./data` (people, threads, per-person workspaces) |
| `ERGON_WORKSPACE` | `<data>/workspace` — the default person's sandbox root |
| `ERGON_MODEL_BASE_URL` | `https://opencode.ai/zen/go/v1` |
| `ERGON_MODEL_ID` | `deepseek-v4.1-flash` |
| `ERGON_MODEL_API_KEY` | — (bearer key; every model call also sends `x-opencode-session` + a browser User-Agent) |

## How it works

- **Shell** — embedded HTML/CSS/JS: left sidebar with the person
  switcher and thread list, one scrolling thread, composer pinned at
  the bottom. Agent steps appear in the thread as a plain ledger of what
  was actually done; the finished result is the final message. Light and
  dark themes, toggle under Settings → Appearance, dark by default.
- **People** — every employee gets their own workspace, memory and
  threads (`people.json` + one data directory per person). People can be
  created in the UI; a new person's agent starts from a seeded memory.
  Browser sessions are pinned to a person and never leak across people;
  session ids are 16 random bytes and a forged or unknown cookie is
  replaced, never adopted.
- **Agent loop** (`src/lib.rs`) — `POST /api/chat {thread_id, message}`.
  The model replies with one JSON object per turn —
  `{"tool": name, "args": {...}}` or `{"done": true, "summary": "..."}` —
  the loop executes the tool for real, feeds the result back, and
  repeats (cap 24 steps).
- **Tools** — `list_dir`, `read_file`, `write_file`, `search_files`,
  `move_file` (all sandboxed to the person's workspace), `fetch_url`
  (HTML→text), and four Rune skills: `write_document`,
  `make_spreadsheet`, `research`, `update_memory`.
- **Sandbox** — every path is resolved through a component-walking
  resolver that re-checks containment at each step, follows at most 16
  symlink hops, and treats a symlink as an object in its own right
  (moving a link moves the link). Escapes — `..`, absolute paths,
  symlink chains, sibling-prefix tricks — are refused, and the test
  suite pins each one.
- **Learning** — each person's `memory.md` (including a standing-
  preferences section) is injected into the model's context at the start
  of every task, and the agent updates it through `update_memory` when
  it learns a fact or preference — including for people created minutes
  earlier. What it learned is visible in the UI's memory panel.
- **Honesty machinery** — a run's summary is checked against the files
  the run actually wrote: naming a file the run didn't write produces a
  factual ledger line in the thread, never a silent pass. Requests over
  1 MB get a real HTTP 413; every request has a 30-second total
  deadline. Thread and people stores save atomically, and a corrupt
  store — invalid JSON *or* valid JSON of the wrong shape — is backed
  up (first free `.bak` suffix) instead of being silently overwritten.
- **Rune skills** (`src/skills/*.rn`, host layer in `src/skills.rs`) —
  document writing, spreadsheets, research and memory updates are Rune
  scripts that format their output and act only through registered host
  functions (workspace fs, memory, HTTP fetch).

## Proof

```sh
CARGO_BUILD_JOBS=2 cargo test   # 44 tests, all passing
```

`tests/e2e.rs` runs a stub model server (test fixture only) speaking
scripted chat-completions replies over the real HTTP path, and proves
end to end: messy notes → `list_dir` → `read_file` ×2 →
`write_document` (Rune) → `update_memory` (Rune) → done — asserting the
merged document, the learned memory note, the transcript and the wire
headers. The suite also pins the sandbox escapes, the session rules,
the done-gate in both directions (including filenames with spaces),
persistence across restarts, and the no-key honesty path.

Beyond the suite, an independently written sealed acceptance exam
(19 black-box checks — people isolation, forged cookies, sandbox
escapes, learning reaching the prompt, the done-gate both ways, 413
with a body in flight, corrupt-store backups) passes **19/19** against
the unmodified binary.

The live-model path is real code, exercised here against the stub
because no model key is available in the build environment; point
`ERGON_MODEL_BASE_URL` / `ERGON_MODEL_API_KEY` at any
OpenAI-compatible endpoint to run it live.

## Status

Working software, verified as above. Not yet installed at a real
company; no claim on this page goes past what the test record shows.
