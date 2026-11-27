# Release Signing & Verification (`docs/SIGNING.md`)

Ergon's release workflow (`.github/workflows/release.yml`) builds locked, tested binaries for Linux and native installers for Windows (`.msi` + `.zip`), generates `SHA256SUMS`, and automatically Authenticode-signs the Windows artifacts whenever signing credentials are configured in **GitHub Actions Repository Secrets**. Ergon ships for Windows and Linux only; there is no macOS build.

When signing secrets are not provisioned, the release workflow still builds and publishes unsigned artifacts with an explicit CI warning so local/staging releases are never blocked.

---

## 1. Required GitHub Actions Secrets

Add these under **Repository Settings → Secrets and variables → Actions**.

### Windows Authenticode Signing

| Secret | Required? | Description |
|---|---|---|
| `WINDOWS_SIGN_PFX_BASE64` | Required for Windows signing | Base64-encoded PKCS#12 (`.pfx`) code-signing certificate (OV or EV Authenticode certificate issued by a trusted CA). |
| `WINDOWS_SIGN_PFX_PASSWORD` | Required with PFX | Password protecting the `.pfx` archive. |
| `WINDOWS_SIGN_TIMESTAMP_URL` | Optional | RFC 3161 SHA-256 timestamp server URL. Defaults to `http://timestamp.digicert.com`. |

> **Alternative (Azure Trusted Signing):** Organizations using Microsoft **Azure Trusted Signing** (recommended for instant Windows SmartScreen reputation without an HSM token) can replace the `signtool` step with `azure/trusted-signing-action@v0` using `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, endpoint URL, account name, and certificate profile name.

Encode a `.pfx` safely without writing secrets into git:
```bash
base64 -w0 ergon-authenticode.pfx | pbcopy   # or xclip / Set-Clipboard on Windows
```

---

## 2. How Artifacts Are Signed in CI

1. **Test Gate:** `cargo test --locked -- --test-threads=1` must pass before any platform binary is built.
2. **Windows (`windows-latest`):**
   - Signs `ergon-x86_64-pc-windows-msvc.exe` with `signtool.exe sign /fd SHA256 /tr <TSA> /td SHA256` and verifies with `signtool verify /pa /v`.
   - Builds `ergon-<ver>-x86_64.msi` via WiX v4 (`packaging/windows/build-msi.ps1` + `packaging/windows/ergon.wxs`) embedding the signed `ergon.exe`, signs and verifies the `.msi`, and packages `ergon-<ver>-x86_64-windows.zip`.
   - Deletes the temporary `.pfx` in a `finally` block even on failure.
3. **Integrity Checksums:** The `release` job hashes every published file into `SHA256SUMS` (`sha256sum * > SHA256SUMS`). Ergon's built-in updater (`src/update.rs`) enforces HTTPS from GitHub hosts and verifies `SHA256SUMS` + `--version` before swapping binaries.

---

## 3. Verifying Release Signatures

### Linux (`SHA256SUMS`)
```bash
sha256sum -c SHA256SUMS --ignore-missing
```

### Windows (PowerShell)
```powershell
# 1. Verify SHA-256 against SHA256SUMS
Get-FileHash .\ergon-0.38.0-x86_64.msi -Algorithm SHA256

# 2. Verify Authenticode signature & certificate chain
Get-AuthenticodeSignature .\ergon-0.38.0-x86_64.msi | Format-List Status, SignerCertificate, TimeStamperCertificate
signtool verify /pa /v .\ergon-0.38.0-x86_64.msi
```

---

## 4. Current Status & What Remains Blocked

- **Ready in repository:** Windows WiX v4 `.wxs` MSI + PowerShell installer/uninstaller + `ErgonService` Windows Service (`ergon service`), authenticated admin update flow behind `ERGON_GATE_TOKEN`, and full CI Windows signing wiring in `.github/workflows/release.yml`.
- **Blocked pending external provisioning:**
  1. **Windows Authenticode signing certificate** (OV/EV `.pfx` or Azure Trusted Signing account) uploaded to GitHub Secrets (`WINDOWS_SIGN_PFX_BASE64`, `WINDOWS_SIGN_PFX_PASSWORD`).
  2. ~~Native Windows 11 verification run~~ Done 2026-11-27 on a clean Windows 11 Home VM with a self-signed test cert (`packaging/windows/vm-smoke-test.ps1`): install, ErgonService auto-start, restart-on-crash, `/api/status` and uninstall all pass. A real-cert run still needs item 1.
