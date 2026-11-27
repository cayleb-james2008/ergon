# Release Signing & Verification (`docs/SIGNING.md`)

Ergon's release workflow (`.github/workflows/release.yml`) builds locked, tested binaries and native installers for Linux, Windows (`.msi` + `.zip`), and macOS (`.pkg` + `.dmg`), generates `SHA256SUMS`, and automatically signs and notarizes Windows and macOS artifacts whenever signing credentials are configured in **GitHub Actions Repository Secrets**.

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

### macOS Developer ID Signing & Notarization

Apple Gatekeeper requires **both** a **Developer ID Application** certificate (for `ergon`, `Ergon.app`, and `.dmg`) and a **Developer ID Installer** certificate (for `.pkg`), exported together into a single `.p12` bundle, plus an **App Store Connect API Key** (`.p8`) for `xcrun notarytool`.

| Secret | Required? | Description |
|---|---|---|
| `APPLE_CERT_P12_BASE64` | Required for macOS signing | Base64-encoded `.p12` containing both **Developer ID Application** and **Developer ID Installer** certificates + private keys. |
| `APPLE_CERT_PASSWORD` | Required with `.p12` | Export password for the `.p12` bundle. |
| `APPLE_SIGNING_IDENTITY` | Required for app/binary/`.dmg` signing | Exact signing identity string, e.g. `Developer ID Application: Example Corp (TEAMID1234)`. |
| `APPLE_INSTALLER_IDENTITY` | Required for `.pkg` signing | Exact installer identity string, e.g. `Developer ID Installer: Example Corp (TEAMID1234)`. |
| `APPLE_TEAM_ID` | Recommended | 10-character Apple Developer Team ID (`TEAMID1234`). |
| `APPLE_NOTARY_KEY_ID` | Required for notarization | App Store Connect API Key ID (10 characters). |
| `APPLE_NOTARY_ISSUER_ID` | Required for notarization | App Store Connect Issuer UUID. |
| `APPLE_NOTARY_KEY_P8` | Required for notarization | Full PEM contents of the `AuthKey_<KEY_ID>.p8` private key file. |

Encode the combined `.p12` on macOS:
```bash
base64 -i ergon-developer-id.p12 | pbcopy
```

---

## 2. How Artifacts Are Signed in CI

1. **Test Gate:** `cargo test --locked -- --test-threads=1` must pass before any platform binary is built.
2. **Windows (`windows-latest`):**
   - Signs `ergon-x86_64-pc-windows-msvc.exe` with `signtool.exe sign /fd SHA256 /tr <TSA> /td SHA256` and verifies with `signtool verify /pa /v`.
   - Builds `ergon-<ver>-x86_64.msi` via WiX v4 (`packaging/windows/build-msi.ps1` + `packaging/windows/ergon.wxs`) embedding the signed `ergon.exe`, signs and verifies the `.msi`, and packages `ergon-<ver>-x86_64-windows.zip`.
   - Deletes the temporary `.pfx` in a `finally` block even on failure.
3. **macOS (`macos-latest`):**
   - Imports `APPLE_CERT_P12_BASE64` into an ephemeral per-run keychain under `$RUNNER_TEMP` and removes it in an `if: always()` step.
   - Signs the standalone binary and `Ergon.app` with Hardened Runtime (`codesign --force --options runtime --timestamp`) and verifies with `codesign --verify --deep --strict`.
   - Builds `ergon-<ver>-aarch64-apple-darwin.pkg` (`packaging/macos/build-pkg.sh`) and `ergon-<ver>-aarch64-apple-darwin.dmg` (`packaging/macos/build-dmg.sh`), submits both to `xcrun notarytool submit --wait`, and staples the notarization ticket with `xcrun stapler staple`.
4. **Integrity Checksums:** The `release` job hashes every published file into `SHA256SUMS` (`sha256sum * > SHA256SUMS`). Ergon's built-in updater (`src/update.rs`) enforces HTTPS from GitHub hosts and verifies `SHA256SUMS` + `--version` before swapping binaries.

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

### macOS (Terminal)
```bash
# 1. Verify SHA-256 against SHA256SUMS
shasum -a 256 -c SHA256SUMS --ignore-missing

# 2. Verify .pkg installer signature & notarization
pkgutil --check-signature ergon-0.38.0-aarch64-apple-darwin.pkg
spctl --assess --type install -vv ergon-0.38.0-aarch64-apple-darwin.pkg

# 3. Verify .dmg and Ergon.app signature & notarization
codesign --verify --deep --strict --verbose=2 /Applications/Ergon.app
spctl --assess --type execute -vv /Applications/Ergon.app
xcrun stapler validate ergon-0.38.0-aarch64-apple-darwin.dmg
```

---

## 4. Current Status & What Remains Blocked

- **Ready in repository:** Windows WiX v4 `.wxs` MSI + PowerShell installer/uninstaller + `ErgonService` Windows Service (`ergon service`), macOS `.pkg`/`.dmg` build scripts + LaunchAgent + postinstall, authenticated admin update flow behind `ERGON_GATE_TOKEN`, and full CI signing/notarization wiring in `.github/workflows/release.yml`.
- **Blocked pending external provisioning:**
  1. **Windows Authenticode signing certificate** (OV/EV `.pfx` or Azure Trusted Signing account) uploaded to GitHub Secrets (`WINDOWS_SIGN_PFX_BASE64`, `WINDOWS_SIGN_PFX_PASSWORD`).
  2. **Apple Developer Program enrollment & certificates** (Developer ID Application + Developer ID Installer `.p12` + App Store Connect API `.p8` key) uploaded to GitHub Secrets (`APPLE_*`).
  3. **Native Windows 11 and macOS verification runs** on real hosts to confirm zero SmartScreen / Gatekeeper prompts on signed release artifacts and end-to-end MSI/PKG installation.
