# Linux Deploy Readiness Report (v0.40.0)

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/release/ergon` (`ergon 0.40.0`, release LTO build)
- Model: `qwen2.5:3b` via `http://127.0.0.1:11434/v1`
- Host systemd safety: `~/.config/systemd/user/` verified untouched (`--no-service`)

## Verified Checks
1. **Clean Install (`install.sh`)**: Installed into isolated prefix + data dir (`--no-service`); verified `ergon --version` (`ergon 0.40.0`), `ergon.env` (`0o600`), `ergon-serve` (`0o700`), and `tls/key.pem` (`0o600`).
2. **Boot, Live Run, Crash (`kill -9`) & Recovery**: Verified HTTPS `GET /api/health` (`200`), ran live `qwen2.5:3b` task, observed active run in `live_runs.json`, killed with `SIGKILL`, restarted, verified `interrupted` state in `/api/runs/status`, resumed via `POST /api/runs/resume` to `completed`, and confirmed thread/file/receipt persistence across clean restart.
3. **Backup -> Wipe -> Restore**: Created `backup.zip` (23 files, 36679 bytes), verified with `ergon verify` (`valid: true`), restored into fresh empty `DATA_DIR`, booted on port `18311`, and verified restored threads, files, company memory, and receipt chain (`GET /api/receipts/verify` `valid: true`).
4. **Upgrade Verification**: Ran `cargo test --release --locked --test installers_and_admin_updates` (all passed) and upgraded a `v0.39.0` prefix via `install.sh` while preserving `ergon.env` and user workspace files; verified `GET /api/version` & `GET /api/update` report `0.40.0`.
5. **Logs & Diagnostics Secret-Leak Scan**: Generated crash report (`--test-crash`) + diagnostic ZIP via both `ergon diagnostics` CLI and `GET /api/diagnostics/bundle`; verified valid rotating JSONL logs (`1` file(s)) and 0 leaked API keys, gate tokens, passwords, session tokens, or private keys across `12` diagnostic archive entries.

## Execution Log
```text
[10:41:30] 1. install.sh --no-service exited 0:
Installed: /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/prefix/bin/ergon
Data dir:  /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/data (people, accounts, TLS certs, library and work records live here)
TLS cert: /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/data/tls/cert.pem
TLS key:  /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/data/tls/key.pem (mode 0600)
SHA-256:  4cd86ca456f9217f22235f08455c7a59cb60a3a4c3ab9a5745f650f9d6e4ccde
Env file:  /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/data/ergon.env (mode 0600 — keep API keys and tokens here)
Launcher:  /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/prefix/bin/ergon-serve (mode 0700)

Next steps:
  1. Add the model key: edit ERGON_MODEL_API_KEY in /home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/data/ergon.env
     (mode 0600, owner-readable only). TLS cert paths (ERGON_TLS_CERT/ERGON_TLS_KEY)
     and OIDC settings (ERGON_OIDC_ISSUER/ERGON_OIDC_CLIENT_ID) go there too.
     Without a key Ergon runs deterministic workflows and refuses to fabricate
     LLM answers.
  2. Start it:  ergon-serve   (or: systemctl --user enable --now ergon)
  3. Open https://127.0.0.1:8787 to create the initial admin account (or run
     ERGON_DATA_DIR="/home/cayleb/Work/scratch/2026-11-27-ergon-full/wave4-linux/data" ergon user add admin --role admin --password ...).
     The field phone view is at https://127.0.0.1:8787/phone behind the same
     HTTPS/TLS and login gate.
[10:41:30]    Verified ~/.config/systemd/user/ untouched
[10:41:30]    Verified version='ergon 0.40.0', ergon.env=0o600, ergon-serve=0o700, tls/key.pem=0o600
[10:41:31] 2. HTTPS /api/health -> 200: {'auth_required': True, 'name': 'ergon', 'needs_setup': True, 'oidc_enabled': False, 'ok': True, 'tls': True, 'version': '0.40.0'}
[10:41:39]    Live Ollama qwen2.5:3b task completed (thread_id=t1795794091083-0)
[10:41:39]    Observed live_runs.json running run_id=run-1795794099192-1; sending SIGKILL (kill -9 pid=269479)
[10:41:39]    Boot recovery marked crashed run run-1795794099192-1 as 'interrupted'
[10:41:50]    POST /api/runs/resume completed recovered run run-1795794099192-1 -> status=completed
[10:41:50]    Clean restart persistence verified: threads=2, receipts={'head_hash': '7cc141b2c3a095514027b283d9639035d4e9bd4c9cb8111e48f10bbd378417dc', 'total': 2, 'valid': True}
[10:41:50] 3. ergon backup -> 23 files, 36679 bytes zip
[10:41:50]    ergon verify -> valid=True kind=backup_archive
[10:41:50]    ergon restore -> restored_files=23 into fresh-data
[10:41:50]    Restored fresh-data server verified: threads=2, company_memory=1, receipts={'head_hash': '7cc141b2c3a095514027b283d9639035d4e9bd4c9cb8111e48f10bbd378417dc', 'total': 2, 'valid': True}
[10:41:51] 4. cargo test --release --locked --test installers_and_admin_updates -> test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
[10:41:51]    Upgrade verified: ergon 0.39.0 -> ergon 0.40.0, ergon.env & workspace preserved, /api/update current=0.40.0
[10:41:52] 5. Secret-leak scan PASS: 1 JSONL log file(s), CLI diag.zip (logs=1, crashes=1), HTTP /api/diagnostics/bundle -> 0 secret leaks across 12 archive entries
```
