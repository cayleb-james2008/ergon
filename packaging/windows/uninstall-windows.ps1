<#
.SYNOPSIS
    Stops and removes the ErgonService Windows Service, removes shortcuts and installed
    binaries, and optionally purges %ProgramData%\Ergon\data when -PurgeData is passed.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1
    powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1 -PurgeData
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "$env:ProgramFiles\Ergon",
    [string]$DataDir = "$env:ProgramData\Ergon\data",
    [string]$ServiceName = "ErgonService",
    [int]$Port = 8787,
    [switch]$PurgeData
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($null -ne $svc) {
    Write-Host "Stopping and removing Windows Service '$ServiceName'..."
    if ($svc.Status -ne "Stopped") {
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
    $exe = Join-Path $InstallDir "ergon.exe"
    if (Test-Path -LiteralPath $exe) {
        & $exe service uninstall --name $ServiceName | Out-Null
    } else {
        & sc.exe delete $ServiceName | Out-Null
    }
}

$ruleName = "Ergon Server (TCP $Port)"
$rule = Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
if ($null -ne $rule) {
    Write-Host "Removing Windows Firewall rule '$ruleName'..."
    Remove-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
}

$shortcutLocations = @(
    (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\Ergon"),
    (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Ergon"),
    (Join-Path ([Environment]::GetFolderPath("CommonDesktopDirectory")) "Ergon.lnk"),
    (Join-Path ([Environment]::GetFolderPath("Desktop")) "Ergon.lnk")
)
foreach ($item in $shortcutLocations) {
    if (Test-Path -LiteralPath $item) {
        Remove-Item -LiteralPath $item -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if (Test-Path -LiteralPath $InstallDir) {
    Write-Host "Removing installation directory: $InstallDir"
    Remove-Item -LiteralPath $InstallDir -Recurse -Force
}

if ($PurgeData) {
    if (Test-Path -LiteralPath $DataDir) {
        Write-Host "Purging data directory: $DataDir"
        Remove-Item -LiteralPath $DataDir -Recurse -Force
    }
} else {
    Write-Host "Preserved data directory: $DataDir (pass -PurgeData to delete workspace data)."
}

Write-Host "Ergon uninstall complete."
