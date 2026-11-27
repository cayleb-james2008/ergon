# Video Proof Sidecar — `ergon-installer-cli`

- **Surface / App:** `ergon-installer-cli` (`install.sh` + `ergon` CLI + `ergon-serve` + `export-handoff` + `verify`)
- **Commit SHA:** `28c3a7354` (`v0.38.0`)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy 4.5.0 / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-installer-cli/2026-11-26-install-and-cli.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `/home/cayleb/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-installer-cli/2026-11-26-install-and-cli.mp4`
- **Video Metadata:** `28.05s` (`00:28`), `1,308,864` bytes (`1.25 MiB`), `1440x900`, `h264`, `20 fps`, uncut single Xvfb take (`DISPLAY=:91`)
- **Video SHA-256:** `f4d4ee6d7671ed88a0a9cd3a19dd229e1f6ae4180eefd1385bc4d3d9fd8fe4d3`

## Exact Commands & Inputs

- **Input Type:** Real compiled `v0.38.0` release binary (from commit `28c3a7354`) placed alongside an exact copy of repo `install.sh` in an isolated staging directory (exercising `install.sh`'s documented prebuilt-binary installation path) + realistic non-sensitive test checklist file `q4-site-readiness.md` (seeded during the recording). Zero secrets, zero mocked HTTP responses.
- **Commands Executed Live on Screen:**
  1. `HOME=.../cli-env/home .../stage/install.sh --prefix .../cli-env/prefix --data .../cli-env/data --no-service`
  2. `ls -lh .../prefix/bin && cat .../prefix/bin/ergon-serve`
  3. `.../prefix/bin/ergon --version && .../prefix/bin/ergon --help`
  4. `ERGON_PORT=18910 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 .../prefix/bin/ergon-serve`
  5. `curl -s http://127.0.0.1:18910/api/version`, `/api/status`, `/api/skills`, `/api/update`
  6. `ERGON_DATA_DIR=.../data .../prefix/bin/ergon export-handoff --out .../handoff.zip --title 'Q4 Site Readiness Handoff' --json`
  7. `unzip -l .../handoff.zip && .../prefix/bin/ergon verify .../unzipped/audit_bundle.json --workspace .../data/workspace`

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous capture starts on `DISPLAY=:91`; checks commit SHA (`28c3a7354`), OS, and empty isolated target directories |
| `00:02` | Runs `install.sh --prefix ... --data ... --no-service` into clean isolated `HOME`/`PREFIX`/`DATA_DIR` |
| `00:04` | Lists installed `ergon` and generated `ergon-serve` launcher script |
| `00:07` | Verifies `ergon --version` (`ergon 0.38.0`) and `ergon --help` usage output |
| `00:10` | Cold-launches installed `ergon-serve` on loopback port `18910` (`ERGON_NO_AUTO_MODEL=1`) |
| `00:13` | Queries live `/api/version`, `/api/status` (showing `0.38.0`, `configured: false`, full capabilities), `/api/skills` (12 built-in workflows), and `/api/update` |
| `00:18` | Creates realistic `q4-site-readiness.md` in workspace and runs `ergon export-handoff --json` (`PASS`, 0 leaks) |
| `00:22` | Lists `handoff.zip` (`CLIENT_SUMMARY.md`, `audit_bundle.json`, `VERIFY.txt`, `SHA256SUMS`), extracts `audit_bundle.json`, and runs offline `ergon verify` (`PASS [audit_bundle] ... sha256: 321362fd99bf...`) |
| `00:25` | Cleanly stops child `ergon-serve` process and confirms exit |
| `00:27` | Capture ends cleanly |

## Observed Notes

- No failures or workarounds occurred. `install.sh` cleanly quoted paths into `ergon-serve`, `ergon` booted cold on port `18910`, `export-handoff` generated a clean 4-file ZIP archive, and offline `ergon verify` returned `PASS`.
