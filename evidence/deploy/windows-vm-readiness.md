# Windows 11 Home VM Deploy-Readiness Proof (Wave 4)

- **Date:** 2026-11-27
- **Repository HEAD:** `df3857ea4f04031a7718add6b8c90788413febe3` (`ergon-rs v0.40.0`)
- **Guest OS:** Microsoft Windows 11 Home (`Microsoft Windows NT 10.0.26300.0`, hostname `ERGON-W11`, user `ergon`, elevated `True`)
- **Hypervisor:** QEMU `11.0.1` (`-accel kvm -cpu host -m 6144 -smp 4 -machine pc`)
- **Full Raw Log:** [windows-vm-readiness.log](file:///home/cayleb/Work/projects/ergon/evidence/deploy/windows-vm-readiness.log)

## 1. Cross-Compilation (`x86_64-pc-windows-msvc`, Static CRT)

Built from clean `main` with static MSVC runtime configured in `.cargo/config.toml` (`rustflags = ["-C", "target-feature=+crt-static"]`):

```bash
CARGO_TARGET_DIR=/home/cayleb/Work/scratch/2026-11-27-ergon-full/target \
CARGO_BUILD_JOBS=2 \
cargo xwin build --release --locked --target x86_64-pc-windows-msvc --bin ergon
```

- **Result:** Exit `0` (`Finished 'release' profile [optimized] target(s) in 7m 30s`)
- **Binary:** `ergon.exe` (`14,220,800` bytes, SHA-256 `5a69e632ac691e0a91e04443941414fb286bad5289c4606f79709d666b1770ac`)
- **Staged to VM share (`/home/cayleb/.winvm-scratch/win11/share/`):**
  - `ergon.exe`
  - `packaging/windows/install-windows.ps1`
  - `packaging/windows/uninstall-windows.ps1`
  - `packaging/windows/vm-smoke-test.ps1`
  - `go.ps1` (SHA-256 `CDF82EBDA152FEE13919E69A3AAC46C072820518A4B3E23A75B4939CB5315B26`)

## 2. Authenticode & Windows Service Smoke Test (`vm-smoke-test.ps1`)

Inside the Windows 11 Home VM, `go.ps1` generated a throwaway self-signed code-signing certificate (`CN=Ergon Test Signing (self-signed, NOT FOR RELEASE)`, Thumbprint `FC415C2BFA54E49E887DCD5C112594A85CAF6F3A`), signed `C:\ergon-test\ergon.exe` with `Set-AuthenticodeSignature` + `http://timestamp.digicert.com` (`Timestamper=CN=DigiCert SHA256 RSA4096 Timestamp Responder 2026 1, O="DigiCert, Inc.", C=US`), deleted the `.pfx` (`G02 pfx deleted from guest: True`), and executed `vm-smoke-test.ps1`:

| Check | Result | Observed Detail |
| --- | --- | --- |
| Binary staged | **PASS** | `C:\ergon-test\ergon.exe` (`14220800` bytes) |
| Authenticode embedded | **PASS** | `exe carries an Authenticode signature` (`UnknownError` before trusting test root) |
| Authenticode chain with trusted test root | **PASS** | `signature Valid once test root is trusted status=Valid` |
| `install-windows.ps1` | **PASS** | `exit=0`; installed `C:\Program Files\Ergon\ergon.exe (ergon 0.40.0)` |
| Service registration | **PASS** | `ErgonService` registered (`data dir: C:\ProgramData\Ergon\data`) |
| Service running | **PASS** | `status=Running`; `ergon service status` reported `state=running installed=true running=true` |
| Service start mode | **PASS** | `mode=Auto` |
| Failure recovery config (`sc qfailure`) | **PASS** | `RESET_PERIOD: 86400`, `FAILURE_ACTIONS: RESTART` at `5000ms`, `15000ms`, `30000ms` |
| Live HTTP API probe | **PASS** | `GET http://127.0.0.1:8787/api/status` returned `200` |
| Crash auto-recovery | **PASS** | Killed `ErgonService` PID; SCM restarted service with new PID (`S26 [PASS]`) |
| `uninstall-windows.ps1` | **PASS** | `ErgonService` removed (`S31 [PASS]`), `C:\Program Files\Ergon\ergon.exe` removed (`S32 [PASS]`) |
| `vm-smoke-test.ps1` summary | **PASS** | `S33 ==== END: 0 failure(s) ====` |

## 3. CLI & Data Lifecycle Checks on Windows

Executed against an isolated temporary `ERGON_DATA_DIR` (`C:\ergon-test\cli-tmp\data`) containing `workspace\marker.txt`, `config.json` (with canary `api_key`), and `logs\ergon.log` (with canary secret):

| Check | Result | Observed Detail |
| --- | --- | --- |
| `ergon.exe --version` | **PASS** | `exit=0 out=ergon 0.40.0` |
| `ergon.exe backup` | **PASS** | `exit=0` (`3 files, 225 bytes raw, 1123 bytes zip`, sha256 `63b8d0eea7a5343a6805d8b6dec4fec1210bb8cfe1e2e87b30fb5899e670e5ec`) |
| `ergon.exe verify` | **PASS** | `exit=0 PASS [backup_archive] Backup archive passed offline SHA-256 verification (3 files, 225 bytes, Ergon 0.40.0)` |
| `ergon.exe restore` | **PASS** | `exit=0 marker_match=True` (`restored 3 files (225 bytes) into C:\ergon-test\cli-tmp\restored`) |
| `ergon.exe diagnostics` build | **PASS** | `exit=0` (`1580 bytes, sha256:5525cc514ae1, logs:1, crashes:0, security_events:0`) |
| `ergon.exe diagnostics` secret-free check | **PASS** | Inspected all 5 ZIP entries (`README.txt,system.json,security_events.json,connectors.json,logs/ergon.log`); `leaked=0` |

## 4. Cleanup & VM Teardown

- **Guest certificate store cleanup:** Removed test root thumbprint `FC415C2BFA54E49E887DCD5C112594A85CAF6F3A` from `Cert:\LocalMachine\Root` and `Cert:\LocalMachine\TrustedPublisher` (`Root remaining=0 TrustedPublisher remaining=0 ; ergon.exe signature now=UnknownError`, `cli_failures=0 cleanup_complete=True`).
- **Guest script exit:** `ERGON-W11 go.ps1 done sha256=CDF82EBDA152FEE13919E69A3AAC46C072820518A4B3E23A75B4939CB5315B26 exit=0`.
- **Host teardown:** Issued `system_powerdown` via `/home/cayleb/.winvm-scratch/win11/monitor.sock`; verified QEMU PID `257169` exited cleanly, stopped `serve3.py` PID `257022`, and verified `127.0.0.1:8899` is clear with no QEMU or `serve3.py` processes remaining.
