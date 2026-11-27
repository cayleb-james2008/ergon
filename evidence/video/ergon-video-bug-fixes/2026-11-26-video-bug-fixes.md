# Video Proof Sidecar — `ergon-video-bug-fixes`

- **Surface / App:** `ergon-video-bug-fixes` (Gap 5 — all 3 small bugs caught uncut in PR #10 videos: `HEAD /phone` returning `200 OK`, `PipelineBuilder` structured 2D `rows` + backend JSON/CSV parsing for `make_spreadsheet` / `write_document`, and `.csv`/`.tsv` tabular PDF 1.4 export + disabled `#file-save-pdf` tooltip on unsupported extensions)
- **Base Commit SHA:** `28c3a7354` (`v0.38.0`, branch `fix/mvp-video-bugs`)
- **Host OS:** `Linux 7.2.15-4-omarchy x86_64 GNU/Linux` (Omarchy / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-video-bug-fixes/2026-11-26-video-bug-fixes.mp4` (gitignored per large-file policy)
- **Deliverables Video Copy:** `ergon-video-evidence-2026-11-26/ergon-video-bug-fixes/2026-11-26-video-bug-fixes.mp4` (exact SHA-256 parity verified)
- **Video Metadata:** `34.35s` (`00:34`), `1,050,337` bytes (`1.00 MiB`), `1440x900`, `h264`, `yuv420p`, `20 fps`, uncut single Xvfb take (`DISPLAY=:97`, `GDK_SCALE=1`, `GDK_DPI_SCALE=1`, `XDG_SESSION_TYPE=x11`, Chromium `--ozone-platform=x11 --force-device-scale-factor=1`)
- **Video SHA-256:** `dc397348c092d74955e98e150fbd942a0bb4571906bfce1efcc9f0aedc27d15a`

## Exact Command & Inputs

- **Cold Launch Command:**
  `ERGON_DATA_DIR=./demo-data ERGON_PORT=18971 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 ERGON_UPDATE_MODE=off ergon --no-open`
- **HTTP Probe Command:**
  `curl -sI http://127.0.0.1:18971/phone`
- **Inputs Used:** Clean empty `./demo-data` workspace (`0` pre-existing files), `ERGON_NO_AUTO_MODEL=1`, and a typed `make_spreadsheet` pipeline authored directly in `PipelineBuilder`:
  - `title`: `Q4-Parts-Rollup`
  - `columns`: `["Part", "Supplier", "Qty", "Amount"]`
  - `rows`: `[["Conduit 10ft", "Acme, Inc.", "12", "$144.00"], ["Junction Box", "Northstar Supply", "8", "$96.00"], ["Panel Breaker", "Summit Tools", "4", "$220.00"]]` (includes embedded comma inside `"Acme, Inc."`)
  - Zero mocked HTTP routes, zero secrets, zero browser console/page errors (`console_errors: []`), and zero HTTP `>= 400` responses (`http_errors: []`).

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous uncut capture starts on `DISPLAY=:97` (`1440x900`); prints `ergon 0.38.0` and cold-launches `ergon` on loopback `127.0.0.1:18971` with clean empty `./demo-data` |
| `00:03` | Runs `curl -sI http://127.0.0.1:18971/phone` -> returns `HTTP/1.1 200 OK` with `Content-Type: text/html; charset=utf-8`, `Content-Length: 172815`, matching `ETag`, `Vary: Accept-Encoding`, `Cache-Control: no-cache`, and empty wire body |
| `00:05` | Opens headed Chromium (`1440x900`, 1:1 DPI) to `http://127.0.0.1:18971/` |
| `00:08` | Opens **Scheduled routines -> Manage -> New routine**, names routine `Q4 Parts Rollup`, opens **Typed pipeline composer**, selects `make_spreadsheet`, and enters `title`, JSON array `columns`, and 2D JSON `rows` (preserving `"Acme, Inc."` comma) inside the multiline structured editor |
| `00:11` | Clicks **Validate DAG** (`DAG valid · Stages: Stage 1: skill_1`) and clicks **Save routine** |
| `00:14` | Clicks **Run now** on `Q4 Parts Rollup` -> runs deterministic `make_spreadsheet` pipeline and writes `Q4-Parts-Rollup.csv` (`138 B`) into the sandboxed workspace |
| `00:16` | Opens Workspace Files inspector and clicks `Q4-Parts-Rollup.csv` in `FileViewerModal` -> confirms all `3 rows, 4 cols` render cleanly with `"Acme, Inc."` intact and `#file-save-pdf` enabled |
| `00:20` | Clicks **Save as PDF** (`#file-save-pdf`) on `Q4-Parts-Rollup.csv` -> `POST /api/files/export` returns `201 Created` (`0` HTTP 400 / `0` console errors), shows toast `Saved Q4-Parts-Rollup.pdf to your workspace.`, and downloads `Q4-Parts-Rollup.pdf` (`4,971` bytes) |
| `00:22` | Opens `Q4-Parts-Rollup.pdf` in `FileViewerModal` -> displays extracted PDF table content (`3 data rows · 4 columns`, all rows readable) and confirms `#file-save-pdf` is cleanly **disabled** with tooltip `PDF export is only available for .md, .txt, .csv, and .tsv files (not .pdf)` |
| `00:26` | Closes Chromium to reveal the terminal underneath in the same uncut take; runs `cat ./demo-data/workspace/Q4-Parts-Rollup.csv`, `file` + `head -c 9` (`PDF document, version 1.4, 1 page(s)`, `%PDF-1.4`), and `pdftotext -layout` showing clean right-aligned numeric columns and all 3 rows |
| `00:32` | Stops child `ergon` process cleanly via `./ergon.pid` |
| `00:33` | Continuous capture ends cleanly |

## Observed Notes

- Zero browser console/page errors (`console_errors: []`) and zero HTTP `>= 400` responses (`http_errors: []`).
- Downloaded artifact verified on disk inside the uncut take: `Q4-Parts-Rollup.pdf` (`4,971` bytes, valid `%PDF-1.4`, 1 page, readable via both `FileViewerModal` and `pdftotext -layout`).
- Additional backend edge cases (`.tsv` -> PDF 1.4, `.zip`/`.png`/`.pdf` HTTP `400` rejection, stringified JSON 2D `rows` & CSV newline `rows`, `write_document` native/stringified/plain `sections` preserving commas, and `HEAD /` + conditional `304` + gzip `HEAD /phone`) are verified by `tests/video_bug_fixes.rs` (`4/4 PASS`).
