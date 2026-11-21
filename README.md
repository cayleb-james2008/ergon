# Ergon

[![latest release](https://img.shields.io/github/v/release/cayleb-james2008/ergon)](https://github.com/cayleb-james2008/ergon/releases)


An AI agent for every employee — personal to each person, it learns how
they work and **finishes their digital work autonomously, end to end**.
The person states a task in plain words; the agent works it through real
tools and the finished result lands in the thread. No approval ceremony
in the core loop.

A single Rust binary serves the whole product: the web shell, the agent
loop, the tool layer, the per-person memory, and the Rune skill runtime.
No framework, no build step, no database server.

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

Without `ERGON_MODEL_API_KEY` the UI shows an honest "no model key
configured" banner and `/api/chat` refuses (HTTP 503) — Ergon never
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

- **Shell** — two embedded pages in `assets/` (`index.html`, `phone.html`),
  each one file of HTML, CSS and JS. The desktop face opens on **Chat**: a
  sidebar with the person switcher, New chat, the conversation list and
  Settings (appearance, what the agent has learned, the suite and library,
  add a person). The top bar switches to **Today** (the morning brief,
  what is waiting on you with approve or decline, what the agent learned,
  offers, today's stream) and **Work Graph** (a plain list under 768px).
  Agent steps appear in the thread as a ledger of what was actually done;
  the finished result is the final message. The phone face (`/phone`) has a
  bottom tab bar (Brief, Chat, Decisions, Suite) with the composer above it.
  Dark is the default; the light theme is under Settings, Appearance.
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
  `make_spreadsheet`, `research`, `update_memory`. On top of those,
  the completion build adds the library tools below, `delegate`,
  `delete_file` (which never deletes directly — see Decisions) and
  `flag_decision`. One run — the main agent plus everything it
  delegates — draws from a single shared budget of 24 model calls.
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
- **Company library & personal suites** — one shared library of
  skills, subagents and apps (`library.json` at the root data dir;
  seeded with twelve items, not a clean slate). Each person's agent
  carries a **suite** drawn from it, shaped by their role. Agents
  maintain the library themselves: `browse_library`, `install_item`
  and `publish_skill` (compile-gated, and ownership-checked — an agent
  can update its own person's skills, never silently overwrite
  another's). Installed library skills run through the same sandboxed
  Rune host layer, under an instruction budget and memory limit.
- **Subagents** — the main agent can `delegate` a focused sub-task to
  a specialist in the person's suite (Researcher, Editor, Bookkeeper
  seeds). The subagent's steps land in the same thread, labelled; its
  writes credit the run's ledger; its toolset restriction is enforced
  in the loop, not just written in its prompt — a subagent cannot
  publish, install, or park decisions.
- **Decisions** — work arrives as decisions only at consequential
  moments. `delete_file` parks a decision instead of deleting; the
  file is deleted (the exact path approved — a symlink approval
  deletes the link, never its target) only when the person approves
  in the inbox, and the outcome reports back into the thread it came
  from; the thread's "has NOT been deleted yet" note for that file is
  rewritten to say whether it was deleted or kept. Everything else
  finishes autonomously.
- **Proactive offers** — the agent watches its own work log: the same
  kind of finished task three times earns an offer to turn it into a
  one-tap app in the suite (accepting builds and installs it);
  spreadsheets produced by hand three times earn an offer of the
  expense-sheet app. A dismissed offer is never re-offered, unfinished
  runs earn nothing, and re-running an existing app doesn't count.
- **Morning brief** — yesterday's finished work, what's waiting on
  the person, and what the agent learned recently, assembled from the
  real stores on the Today face and the phone's Brief tab. "Today"
  is the host's local day, read from its zone file. A plain answer (no
  successful tool step and no written file), or a run whose tool steps
  all failed, is kept in the log but is not finished work: it is not in
  the brief, the graph's work nodes or the offers scan. A run that
  parked a decision is not finished in the brief until that decision
  is settled.
- **Honesty machinery** — a run's summary is checked against the files
  the run actually wrote: naming a file the run didn't write produces a
  factual ledger line in the thread, never a silent pass. Requests over
  1 MB get a real HTTP 413; request heads over 32 KiB get a 431; every
  request has a 30-second total deadline. A request whose Host is not
  localhost, 127.0.0.1, [::1] or `ERGON_ALLOWED_HOSTS` gets 403 (unless a
  gate token is set), a POST that a browser sends cross-site or with a
  foreign Origin gets 403, and a POST with a body that is not JSON gets
  415. `GET /api/status` includes `store_recovered`: the file names moved
  to a backup at load (empty when none), shown as a notice in the Chat face. Thread and people stores save atomically, and a corrupt
  store — invalid JSON *or* valid JSON of the wrong shape — is backed
  up (first free `.bak` suffix) instead of being silently overwritten.
- **Rune skills** (`src/skills/*.rn`, host layer in `src/skills.rs`) —
  document writing, spreadsheets, research and memory updates are Rune
  scripts that format their output and act only through registered host
  functions (workspace fs, memory, HTTP fetch).

## Proof

```sh
CARGO_BUILD_JOBS=2 cargo test   # 82 tests, all passing
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
(31 black-box checks — the original 19: people isolation, forged
cookies, sandbox escapes, learning reaching the prompt, the done-gate
both ways, 413 with a body in flight, corrupt-store backups; plus 12
for the completion build: library seeding and role-shaped suites,
cross-person skill publishing, delegation, the delete gate, offers,
brief/today/graph truth, app runs, faces and phone, the hosting gate,
installer) passed **31/31** against an earlier build of the binary; it was not re-run for this README.

The completion build additionally survived a fresh-context thermonuclear review whose reproduced findings (a symlink-delete that hit the wrong file, unsynchronised record stores, prompt-only subagent restrictions, a library lock held across skill runs) were each fixed and pinned by a regression test.

The live-model path is real code, exercised here against the stub
because no model key is available in the build environment; point
`ERGON_MODEL_BASE_URL` / `ERGON_MODEL_API_KEY` at any
OpenAI-compatible endpoint to run it live.

## Status

Working software, verified as above. Not yet installed at a real
company; no claim on this page goes past what the test record shows.

## Versions and updates

**Releasing:** bump `version` in `Cargo.toml`, add a `## X.Y.Z - date` entry to `CHANGELOG.md`, and merge to `main`. `.github/workflows/release.yml` sees the new version, builds every platform and publishes the GitHub Release `vX.Y.Z` with the binaries and `SHA256SUMS`; a push that does not change the version publishes nothing, and a new version without a changelog entry fails the workflow. Pushing a `vX.Y.Z` tag by hand does the same.

The running version is shown in the page footer, in `GET /api/version` and `GET /api/status`, and by `ergon --version`. Every release is tagged `vX.Y.Z` on GitHub, with a changelog entry in `CHANGELOG.md`, binaries for each platform and a `SHA256SUMS` file; merging a new version to `main` (or pushing a tag) runs `.github/workflows/release.yml`, which refuses a tag that does not match the version in `Cargo.toml`.

Self-update (`src/update.rs`): the app asks the newest GitHub release of its own repository whether a newer version exists (shortly after start and every 6 hours), and the page offers it. **Update now** downloads the binary for this platform, checks its SHA-256 against the release's `SHA256SUMS`, runs it with `--version` to confirm it is the build it claims to be, and only then swaps it in; **Restart** then starts the new build. Any failed step leaves the installed program untouched. Updating is refused for a cargo-built binary, through a shared (gated) address, from a non-local Host name, and while a task is running.

| Var | Default |
| --- | --- |
| `ERGON_UPDATE_MODE` | `notify` (check and tell); `auto` also downloads and installs (a restart is still yours to press); `off` never contacts GitHub |
| `ERGON_UPDATE_REPO` | `cayleb-james2008/ergon` |
| `ERGON_UPDATE_API` | `https://api.github.com` (plain http only for a loopback address) |

