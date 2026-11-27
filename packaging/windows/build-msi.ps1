<#
.SYNOPSIS
    Builds the Windows WiX v4 MSI installer and portable ZIP bundle for Ergon,
    optionally signing ergon.exe and the .msi with Authenticode (signtool.exe).

.EXAMPLE
    pwsh -File packaging/windows/build-msi.ps1 -Version 0.38.0 -BinaryPath target/x86_64-pc-windows-msvc/release/ergon.exe -OutDir dist
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$BinaryPath,
    [string]$OutDir = "dist",
    [string]$Arch = "x86_64",
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WxsPath = Join-Path $ScriptDir "ergon.wxs"
$InstallScript = Join-Path $ScriptDir "install-windows.ps1"
$UninstallScript = Join-Path $ScriptDir "uninstall-windows.ps1"

if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Binary not found: $BinaryPath"
}
$ResolvedBinary = (Resolve-Path -LiteralPath $BinaryPath).Path

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
$ResolvedOutDir = (Resolve-Path -LiteralPath $OutDir).Path

# Verify binary executes and reports the expected version.
$verOut = & $ResolvedBinary --version
Write-Host "Binary version output: $verOut"
if ($verOut -notmatch [regex]::Escape($Version)) {
    throw "Binary version '$verOut' does not match expected version '$Version'."
}

function Find-SignTool {
    $cmd = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($null -ne $cmd) { return $cmd.Source }
    $kitsRoot = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    if (Test-Path -LiteralPath $kitsRoot) {
        $found = Get-ChildItem -Path $kitsRoot -Filter signtool.exe -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match "\\x64\\" } |
            Sort-Object FullName -Descending |
            Select-Object -First 1
        if ($null -ne $found) { return $found.FullName }
    }
    return $null
}

$PfxBase64 = $env:WINDOWS_SIGN_PFX_BASE64
$PfxPassword = $env:WINDOWS_SIGN_PFX_PASSWORD
if (-not [string]::IsNullOrWhiteSpace($env:WINDOWS_SIGN_TIMESTAMP_URL)) {
    $TimestampUrl = $env:WINDOWS_SIGN_TIMESTAMP_URL
}

$TempDir = Join-Path ([IO.Path]::GetTempPath()) ("ergon-win-pkg-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

$PfxFile = $null
$SignTool = $null

try {
    $StagedExe = Join-Path $TempDir "ergon.exe"
    Copy-Item -LiteralPath $ResolvedBinary -Destination $StagedExe -Force

    if (-not [string]::IsNullOrWhiteSpace($PfxBase64)) {
        $SignTool = Find-SignTool
        if ([string]::IsNullOrWhiteSpace($SignTool)) {
            throw "WINDOWS_SIGN_PFX_BASE64 is configured, but signtool.exe was not found on this runner."
        }
        $PfxFile = Join-Path $TempDir "codesign.pfx"
        [IO.File]::WriteAllBytes($PfxFile, [Convert]::FromBase64String($PfxBase64))

        Write-Host "Signing ergon.exe with Authenticode (SHA-256 + RFC 3161 timestamp)..."
        & $SignTool sign /fd SHA256 /f $PfxFile /p $PfxPassword /tr $TimestampUrl /td SHA256 /d "Ergon" $StagedExe
        if ($LASTEXITCODE -ne 0) { throw "signtool failed signing ergon.exe (exit $LASTEXITCODE)" }
        & $SignTool verify /pa /v $StagedExe | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "signtool verify failed on ergon.exe (exit $LASTEXITCODE)" }
    } else {
        Write-Warning "WINDOWS_SIGN_PFX_BASE64 is not set; building UNSIGNED Windows artifacts."
    }

    # Ensure WiX v4 CLI + Util extension are available.
    if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
        Write-Host "Installing WiX v4 CLI tool..."
        dotnet tool install --global wix --version 4.*
        $env:PATH = "$env:USERPROFILE\.dotnet\tools;$env:PATH"
    }
    wix extension add -g WixToolset.Util.wixext/4.0.5 | Out-Null

    $MsiOut = Join-Path $ResolvedOutDir "ergon-$Version-$Arch.msi"
    Write-Host "Building MSI: $MsiOut"
    wix build $WxsPath `
        -ext WixToolset.Util.wixext `
        -arch x64 `
        -d "ProductVersion=$Version" `
        -d "BinaryPath=$StagedExe" `
        -d "InstallScriptPath=$InstallScript" `
        -d "UninstallScriptPath=$UninstallScript" `
        -o $MsiOut
    if ($LASTEXITCODE -ne 0) {
        throw "wix build failed with exit code $LASTEXITCODE"
    }

    if ($null -ne $PfxFile -and $null -ne $SignTool) {
        Write-Host "Signing MSI with Authenticode..."
        & $SignTool sign /fd SHA256 /f $PfxFile /p $PfxPassword /tr $TimestampUrl /td SHA256 /d "Ergon Installer" $MsiOut
        if ($LASTEXITCODE -ne 0) { throw "signtool failed signing MSI (exit $LASTEXITCODE)" }
        & $SignTool verify /pa /v $MsiOut | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "signtool verify failed on MSI (exit $LASTEXITCODE)" }
    }

    # Also produce a clean Windows ZIP bundle containing the (signed) binary and installer scripts.
    $ZipStage = Join-Path $TempDir "zip-stage"
    New-Item -ItemType Directory -Path $ZipStage -Force | Out-Null
    Copy-Item -LiteralPath $StagedExe -Destination (Join-Path $ZipStage "ergon.exe") -Force
    Copy-Item -LiteralPath $InstallScript -Destination (Join-Path $ZipStage "install-windows.ps1") -Force
    Copy-Item -LiteralPath $UninstallScript -Destination (Join-Path $ZipStage "uninstall-windows.ps1") -Force
    $ReadmeTxt = @"
Ergon $Version for Windows ($Arch)
==================================

Option 1: Run the MSI installer (ergon-$Version-$Arch.msi) to install into
          %ProgramFiles%\Ergon, create %ProgramData%\Ergon\data, and start ErgonService.

Option 2: Run install-windows.ps1 from an Administrator PowerShell prompt:
          powershell -ExecutionPolicy Bypass -File .\install-windows.ps1 -DesktopShortcut

To uninstall:
          powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1

Web UI default: http://127.0.0.1:8787/
"@
    Set-Content -LiteralPath (Join-Path $ZipStage "README-WINDOWS.txt") -Value $ReadmeTxt -Encoding UTF8

    $ZipOut = Join-Path $ResolvedOutDir "ergon-$Version-$Arch-windows.zip"
    if (Test-Path -LiteralPath $ZipOut) { Remove-Item -LiteralPath $ZipOut -Force }
    Compress-Archive -Path (Join-Path $ZipStage "*") -DestinationPath $ZipOut -CompressionLevel Optimal

    Write-Host "Built Windows artifacts:"
    Write-Host "  MSI: $MsiOut"
    Write-Host "  ZIP: $ZipOut"
}
finally {
    if ($null -ne $PfxFile -and (Test-Path -LiteralPath $PfxFile)) {
        Remove-Item -LiteralPath $PfxFile -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $TempDir) {
        Remove-Item -LiteralPath $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
