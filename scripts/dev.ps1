# Runs cargo with an automatic local-proxy fallback.
#
# Direct connectivity is probed first; if it works, cargo runs untouched. If the
# network is unreachable, the local proxy is used instead so `cargo build/fetch`
# still works behind a firewall / offline uplink.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/dev.ps1 build --release
#   powershell -ExecutionPolicy Bypass -File scripts/dev.ps1 run
#   $env:VIDLY_PROXY="http://127.0.0.1:7890"; ...\dev.ps1 build
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"

$proxy = if ($env:VIDLY_PROXY) { $env:VIDLY_PROXY } else { "http://127.0.0.1:3128" }
$probe = "https://index.crates.io/config.json"
if ($CargoArgs.Count -eq 0) { $CargoArgs = @("build") }

function Test-Direct {
    try {
        Invoke-WebRequest -Uri $probe -TimeoutSec 5 -UseBasicParsing | Out-Null
        return $true
    } catch {
        return $false
    }
}

if (Test-Direct) {
    Write-Host "[vidly] direct connection OK" -ForegroundColor Green
} else {
    Write-Host "[vidly] direct connection failed, using local proxy $proxy" -ForegroundColor Yellow
    $env:CARGO_HTTP_PROXY = $proxy
    $env:HTTP_PROXY = $proxy
    $env:HTTPS_PROXY = $proxy
    $env:ALL_PROXY = $proxy
    $env:http_proxy = $proxy
    $env:https_proxy = $proxy
}

Write-Host "[vidly] cargo $($CargoArgs -join ' ')" -ForegroundColor Cyan
& cargo @CargoArgs
exit $LASTEXITCODE
