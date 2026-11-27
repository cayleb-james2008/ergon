# Video Proof Sidecar — `ergon-phone-and-web-standalone`

- **Surface / App:** `ergon-phone-and-web-standalone` (Responsive `/phone` web app route at `390x844` + standalone offline single-file HTML desk `frontend/dist/ergon-web.html` via `file://` with zero running server)
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy 4.5.0 / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-phone-and-web-standalone/2026-11-26-phone-and-standalone.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-phone-and-web-standalone/2026-11-26-phone-and-standalone.mp4`
- **Video Metadata:** `25.55s` (`00:25`), `706,268` bytes (`689.7 KiB`), `1440x900`, `h264`, `20 fps`, uncut single Xvfb take (`DISPLAY=:91`)
- **Video SHA-256:** `545b1979ca8ea8f264d07979e434b00d1eb89e2384eb0d32ace4c96c431ee430`

## Exact Command & Inputs

- **Part A (`/phone`):** `ERGON_DATA_DIR=/home/cayleb/Work/scratch/ultron-ergon/run-core/phone-standalone-env/data ERGON_PORT=18914 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 /home/cayleb/Work/scratch/ultron-ergon/cargo-target/release/ergon` opened at `http://127.0.0.1:18914/phone` in a real `390x844` touch-enabled Chromium window.
- **Part B (`ergon-web.html`):** Server explicitly stopped on screen (`00:13`), then `file:///home/cayleb/Work/projects/ergon/frontend/dist/ergon-web.html` (`1.5 MB` standalone bundle built via `npm run build:web`) opened directly in Chromium with zero backend running, executing built-in `reconcile_ledger` fixture test locally via `webEngine.ts`.
- **Inputs Used:** Clean empty `DATA_DIR` for `/phone`; built-in catalog fixture (`reconcile_ledger`, explicitly labeled seeded fixture) inside offline `ergon-web.html`. Zero browser console or page errors.

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous capture starts on `DISPLAY=:91`; inspects `frontend/dist/ergon-web.html`, cold-launches `ergon v0.38.0` on port `18914`, and runs `curl -sI /phone` (returns `404 Not Found` because `src/server.rs` matches `GET /phone` only, not `HEAD`) |
| `00:03` | Opens headed Chromium (`GET /phone`) at `390x844` mobile touch viewport to `http://127.0.0.1:18914/phone` (`200 OK`) |
| `00:06` | Opens mobile Vaul navigation drawer (`Open menu`), opens Settings, toggles Dark -> Light theme cleanly on mobile, and switches to Today view |
| `00:13` | Closes mobile window and explicitly stops the `ergon` server process in terminal (proving zero server dependency for the next step) |
| `00:14` | Opens `file:///home/cayleb/Work/projects/ergon/frontend/dist/ergon-web.html` directly in Chromium (`1440x900`) |
| `00:16` | Opens Workflows dialog and runs built-in `reconcile_ledger` workflow completely offline in-browser (`webEngine.ts`), then switches to Today |
| `00:22` | Closes browser and prints `sha256sum` of `frontend/dist/ergon-web.html` in terminal |
| `00:24` | Capture ends cleanly |

## Observed Notes

- Zero page/console errors (`console_errors: []`). Both `/phone` mobile responsive parity (`GET /phone`) and offline zero-server `ergon-web.html` execution passed cleanly.
- Minor detail observed on screen: `HEAD /phone` (`curl -sI`) returns `404 Not Found` because `src/server.rs` only dispatches `("GET", "/phone")`; `GET /phone` renders the full responsive SPA.
- Prebuilt local untracked artifact `/home/cayleb/Work/projects/ergon/frontend/dist/ergon-web.html` (mtime `2026-11-26 16:56`, SHA-256 `82ba0652b098...`) displays footer `Ergon 0.36.0` (built prior to the `v0.38.0` version bump; `npm run build:web` rebuilds it from `frontend/`).
