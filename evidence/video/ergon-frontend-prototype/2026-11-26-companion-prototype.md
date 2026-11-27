# Video Proof Sidecar — `ergon-frontend-prototype` (Companion Out-of-Repo Prototype)

- **Surface / App:** `ergon-frontend-prototype` (Sibling standalone prototype at `/home/cayleb/Work/projects/ergon-frontend`, NOT part of `cayleb-james2008/ergon` git repo; last modified 2026-11-17/18, serves built `dist/` via Node `server.mjs` and proxies `/api/agent` to local Ollama `qwen2.5:3b` on `127.0.0.1:11434`)
- **Commit SHA (main Ergon repo reference):** `28c3a7354` (`v0.38.0`; sibling directory is not a git repository)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy 4.5.0 / Arch Linux)
- **Result:** `PARTIAL` (Static React shell serves and renders cleanly at `HTTP 200`; live `/api/agent` chat is `BLOCKED` and honestly degrades with `HTTP 503 model_unavailable` because local Ollama is not running on `127.0.0.1:11434`)
- **Worktree Video Path:** `evidence/video/ergon-frontend-prototype/2026-11-26-companion-prototype.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-frontend-prototype/2026-11-26-companion-prototype.mp4`
- **Video Metadata:** `11.55s` (`00:11`), `227,020` bytes (`221.7 KiB`), `1440x900`, `h264`, `20 fps`, uncut single Xvfb take (`DISPLAY=:91`)
- **Video SHA-256:** `86e6050f4ffe57df8548ecba7be79f3480bb6c25b71293d0558ab077a13dea67`

## Exact Command & Inputs

- **Cold Launch Command:** `node /home/cayleb/Work/projects/ergon-frontend/server.mjs --port 18915`
- **Inputs Used:** Built-in static demo workspace bundled in `ergon-frontend/dist/` (seeded prototype UI state) + real HTTP POST to `/api/agent` (`{"view":"manager","messages":[{"role":"user","content":"What is waiting at the gate?"}]}`). Zero mocked Ollama responses.

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous capture starts on `DISPLAY=:91`; inspects `server.mjs` and `dist/index.html`, cold-launches Node server on port `18915` (`ERGON_SERVER_READY`), and confirms `HTTP/1.1 200 OK` |
| `00:03` | Opens headed Chromium (`1440x900`, 1:1 DPI) to `http://127.0.0.1:18915/`, rendering the companion Work Graph / Gate prototype shell cleanly with zero console errors |
| `00:07` | Closes Chromium and invokes `POST http://127.0.0.1:18915/api/agent` from terminal -> receives honest `HTTP_STATUS:503` `{"ok":false,"code":"model_unavailable","message":"The local Ollama runtime could not be reached."}`, then cleanly stops the Node process |
| `00:10` | Capture ends cleanly |

## Observed Notes

- This sibling directory predates the unified Rust-embedded SPA inside `cayleb-james2008/ergon` (`assets/index.html`) and is not packaged with `ergon v0.38.0`.
- Live agent replies require an active Ollama daemon on `127.0.0.1:11434` with `qwen2.5:3b` pulled, which is unavailable on this host (`BLOCKED`).
