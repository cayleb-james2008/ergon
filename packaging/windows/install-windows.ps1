<#
.SYNOPSIS
    Installs Ergon on Windows, creates %ProgramData%\Ergon\data, registers and starts
    the ErgonService Windows Service with failure recovery, and creates Start Menu / Desktop shortcuts.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install-windows.ps1
    powershell -ExecutionPolicy Bypass -File .\install-windows.ps1 -Port 8787 -FirewallRule -DesktopShortcut
#>
[CmdletBinding()]
param(
    [string]$BinaryPath = "",
    [string]$InstallDir = "$env:ProgramFiles\Ergon",
    [string]$DataDir = "$env:ProgramData\Ergon\data",
    [string]$HostAddress = "127.0.0.1",
    [int]$Port = 8787,
    [string]$ServiceName = "ErgonService",
    [switch]$NoService,
    [switch]$FirewallRule,
    [switch]$DesktopShortcut
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not $NoService -and -not (Test-IsAdministrator)) {
    throw "Installing the Windows Service requires an elevated Administrator PowerShell session (or pass -NoService for user-local install)."
}

if ($Port -lt 1 -or $Port -gt 65535) {
    throw "Port must be between 1 and 65535."
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $candidates = @(
        (Join-Path $ScriptDir "ergon.exe"),
        (Join-Path $ScriptDir "ergon-x86_64-pc-windows-msvc.exe"),
        (Join-Path (Split-Path -Parent (Split-Path -Parent $ScriptDir)) "target\release\ergon.exe")
    )
    foreach ($cand in $candidates) {
        if (Test-Path -LiteralPath $cand) {
            $BinaryPath = $cand
            break
        }
    }
}

if ([string]::IsNullOrWhiteSpace($BinaryPath) -or -not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Could not find ergon.exe. Place ergon.exe next to install-windows.ps1 or pass -BinaryPath <path>."
}

$ResolvedBin = (Resolve-Path -LiteralPath $BinaryPath).Path
Write-Host "Installing Ergon from: $ResolvedBin"

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
New-Item -ItemType Directory -Path $DataDir -Force | Out-Null

$DestExe = Join-Path $InstallDir "ergon.exe"

# Stop running service before replacing binary if upgrading in place.
if (-not $NoService) {
    $existingSvc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($null -ne $existingSvc -and $existingSvc.Status -ne "Stopped") {
        Write-Host "Stopping existing $ServiceName..."
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
}

Copy-Item -LiteralPath $ResolvedBin -Destination $DestExe -Force

# Verify installed binary reports its version cleanly.
$verOut = & $DestExe --version
Write-Host "Installed binary: $DestExe ($verOut)"

# Write convenient launcher batch wrapper mirroring install.sh's ergon-serve conventions.
$ServeCmd = Join-Path $InstallDir "ergon-serve.cmd"
$ServeBody = @"
@echo off
setlocal
set "ERGON_DATA_DIR=$DataDir"
set "ERGON_HOST=$HostAddress"
set "ERGON_PORT=$Port"
set "ERGON_NO_OPEN=1"
REM Uncomment and set keys/tokens as needed:
REM set "ERGON_MODEL_API_KEY="
REM set "ERGON_GATE_TOKEN="
REM set "ERGON_ADMIN_TOKEN="
"$DestExe" --no-open --host %ERGON_HOST% --port %ERGON_PORT% %*
"@
Set-Content -LiteralPath $ServeCmd -Value $ServeBody -Encoding ASCII

if (-not $NoService) {
    Write-Host "Registering Windows Service '$ServiceName'..."
    & $DestExe service install --name $ServiceName --data $DataDir --host $HostAddress --port $Port
    if ($LASTEXITCODE -ne 0) {
        throw "ergon service install failed with exit code $LASTEXITCODE"
    }
    Write-Host "Service '$ServiceName' configured with auto-start and automatic restart recovery."
}

if ($FirewallRule) {
    $ruleName = "Ergon Server (TCP $Port)"
    $existingRule = Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
    if ($null -ne $existingRule) {
        Remove-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
    }
    New-NetFirewallRule `
        -DisplayName $ruleName `
        -Direction Inbound `
        -Action Allow `
        -Protocol TCP `
        -LocalPort $Port `
        -Program $DestExe `
        -Profile Private,Domain | Out-Null
    Write-Host "Added Windows Firewall inbound rule: $ruleName"
}

function New-ErgonShortcut {
    param(
        [string]$ShortcutPath,
        [string]$TargetExe,
        [string]$Arguments,
        [string]$WorkingDir,
        [string]$Description
    )
    $parent = Split-Path -Parent $ShortcutPath
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    $shell = New-Object -ComObject WScript.Shell
    $sc = $shell.CreateShortcut($ShortcutPath)
    $sc.TargetPath = $TargetExe
    $sc.Arguments = $Arguments
    $sc.WorkingDirectory = $WorkingDir
    $sc.Description = $Description
    $sc.Save()
}

$ConnectHost = if ($HostAddress -in @("0.0.0.0", "::", "[::]", "")) { "127.0.0.1" } else { $HostAddress }
$UiUrl = "http://${ConnectHost}:${Port}/"

$StartMenuDir = if (Test-IsAdministrator) {
    Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\Ergon"
} else {
    Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Ergon"
}
New-ErgonShortcut `
    -ShortcutPath (Join-Path $StartMenuDir "Ergon.lnk") `
    -TargetExe $DestExe `
    -Arguments "--app --host $HostAddress --port $Port" `
    -WorkingDir $InstallDir `
    -Description "Open Ergon ($UiUrl)"

if ($DesktopShortcut) {
    $DesktopDir = if (Test-IsAdministrator) {
        [Environment]::GetFolderPath("CommonDesktopDirectory")
    } else {
        [Environment]::GetFolderPath("Desktop")
    }
    New-ErgonShortcut `
        -ShortcutPath (Join-Path $DesktopDir "Ergon.lnk") `
        -TargetExe $DestExe `
        -Arguments "--app --host $HostAddress --port $Port" `
        -WorkingDir $InstallDir `
        -Description "Open Ergon ($UiUrl)"
}

Write-Host ""
Write-Host "Ergon installation complete."
Write-Host "  Binary:   $DestExe"
Write-Host "  Data dir: $DataDir"
Write-Host "  Launcher: $ServeCmd"
Write-Host "  Web UI:   $UiUrl"
if (-not $NoService) {
    Write-Host "  Service:  $ServiceName (check status: `"$DestExe`" service status --name $ServiceName)"
}
