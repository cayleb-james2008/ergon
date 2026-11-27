<#
.SYNOPSIS
    Throwaway-VM smoke test for the Windows installer + ErgonService.

.DESCRIPTION
    Meant for a disposable test VM only. It:
      1. Optionally trusts a self-signed TEST code-signing cert (LocalMachine Root +
         TrustedPublisher) so the Authenticode chain can be checked end to end.
      2. Checks Get-AuthenticodeSignature on ergon.exe.
      3. Runs install-windows.ps1, makes sure ErgonService is running and the HTTP
         API answers on 127.0.0.1:<Port>.
      4. Kills the service process and checks Windows restarts it (failure recovery).
      5. Runs uninstall-windows.ps1 and checks the service and binary are gone.
    Every result line is printed and, when -ReportUrl is set, sent to that URL as ?d=<line>.

    Never run step 1 on a real machine: trusting a test root is only OK in a throwaway VM.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\vm-smoke-test.ps1 -Dir C:\ergon-test -TestCertPath C:\ergon-test\ergon-test-cert.cer -ReportUrl http://10.0.2.2:8899/report
#>
[CmdletBinding()]
param(
    [string]$Dir = $PSScriptRoot,
    [string]$TestCertPath = "",
    [string]$ReportUrl = "",
    [int]$Port = 8787,
    [string]$ServiceName = "ErgonService"
)

$ErrorActionPreference = "Continue"
$script:n = 0
$script:fails = 0

function Say([string]$line) {
    $script:n++
    $msg = "S{0:D2} {1}" -f $script:n, $line
    Write-Output $msg
    Add-Content -Path (Join-Path $Dir "smoke-result.txt") -Value $msg
    if ($ReportUrl) {
        try { Invoke-WebRequest -UseBasicParsing -Uri ($ReportUrl + "?d=" + [uri]::EscapeDataString($msg)) -TimeoutSec 5 | Out-Null } catch {}
    }
}
function Check([string]$name, [bool]$ok, [string]$detail = "") {
    if (-not $ok) { $script:fails++ }
    Say ("[{0}] {1} {2}" -f ($(if ($ok) { "PASS" } else { "FAIL" })), $name, $detail)
}

Remove-Item -Path (Join-Path $Dir "smoke-result.txt") -ErrorAction SilentlyContinue
Say "==== ERGON WINDOWS VM SMOKE TEST ===="
Say ((Get-CimInstance Win32_OperatingSystem).Caption + " " + [Environment]::OSVersion.VersionString)

$exe = Join-Path $Dir "ergon.exe"
Check "ergon.exe present" (Test-Path -LiteralPath $exe) $exe

# 1-2. Signature
$sig0 = Get-AuthenticodeSignature -FilePath $exe
Say ("signature before trusting test root: " + $sig0.Status + " / signer=" + $sig0.SignerCertificate.Subject)
Check "exe carries an Authenticode signature" ($null -ne $sig0.SignerCertificate)
if ($TestCertPath -and (Test-Path -LiteralPath $TestCertPath)) {
    Import-Certificate -FilePath $TestCertPath -CertStoreLocation Cert:\LocalMachine\Root | Out-Null
    Import-Certificate -FilePath $TestCertPath -CertStoreLocation Cert:\LocalMachine\TrustedPublisher | Out-Null
    $sig1 = Get-AuthenticodeSignature -FilePath $exe
    Check "signature Valid once test root is trusted" ($sig1.Status -eq "Valid") ("status=" + $sig1.Status)
}

# 3. Install + service
$inst = Join-Path $Dir "install-windows.ps1"
$out = & powershell -NoProfile -ExecutionPolicy Bypass -File $inst -BinaryPath $exe -Port $Port -DesktopShortcut 2>&1
$out | ForEach-Object { Say ("  install> " + $_) }
Check "install-windows.ps1 exit code 0" ($LASTEXITCODE -eq 0) ("exit=" + $LASTEXITCODE)

$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
Check "service registered" ($null -ne $svc)
if ($null -ne $svc -and $svc.Status -ne "Running") { Start-Service -Name $ServiceName -ErrorAction SilentlyContinue; Start-Sleep 5; $svc.Refresh() }
Check "service running" ($null -ne $svc -and $svc.Status -eq "Running") ("status=" + $(if ($svc) { $svc.Status }))
$cim = Get-CimInstance Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue
Check "service start mode Auto" ($null -ne $cim -and $cim.StartMode -eq "Auto") ("mode=" + $(if ($cim) { $cim.StartMode }))
Say ("sc qfailure: " + ((& sc.exe qfailure $ServiceName) -join " | "))

$api = $null
for ($i = 0; $i -lt 15 -and -not $api; $i++) {
    try { $api = Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:$Port/api/status" -TimeoutSec 5 } catch { Start-Sleep 2 }
}
Check "GET /api/status answers" ($null -ne $api -and $api.StatusCode -eq 200) ("code=" + $(if ($api) { $api.StatusCode }))
$st = & "$env:ProgramFiles\Ergon\ergon.exe" service status --name $ServiceName 2>&1
Say ("ergon service status> " + ($st -join " | "))

# 4. Failure recovery: kill the process, expect SCM to bring it back.
if ($cim -and $cim.ProcessId) {
    $pid0 = $cim.ProcessId
    Stop-Process -Id $pid0 -Force -ErrorAction SilentlyContinue
    $back = $false
    for ($i = 0; $i -lt 20 -and -not $back; $i++) {
        Start-Sleep 3
        $c2 = Get-CimInstance Win32_Service -Filter "Name='$ServiceName'"
        if ($c2.State -eq "Running" -and $c2.ProcessId -ne $pid0 -and $c2.ProcessId -ne 0) { $back = $true }
    }
    Check "service auto-restarts after crash" $back
}

# 5. Uninstall
$un = Join-Path $Dir "uninstall-windows.ps1"
$out = & powershell -NoProfile -ExecutionPolicy Bypass -File $un -Port $Port 2>&1
$out | ForEach-Object { Say ("  uninstall> " + $_) }
Start-Sleep 3
Check "service removed" ($null -eq (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue))
Check "binary removed" (-not (Test-Path "$env:ProgramFiles\Ergon\ergon.exe"))

Say ("==== END: " + $script:fails + " failure(s) ====")
