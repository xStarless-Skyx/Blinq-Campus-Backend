Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
$Caddyfile = Join-Path $RootDir "Caddyfile.local"
$HostsPath = Join-Path $env:SystemRoot "System32\drivers\etc\hosts"

if (-not (Get-Command caddy -ErrorAction SilentlyContinue)) {
    Write-Host "caddy is not installed."
    Write-Host "Install with:"
    Write-Host "  winget install CaddyServer.Caddy"
    exit 1
}

if (-not (Test-Path $HostsPath)) {
    Write-Host "Hosts file not found at: $HostsPath"
    exit 1
}

$hostsContent = Get-Content -Raw $HostsPath
if ($hostsContent -notmatch "(^|\s)local\.revolt\.chat(\s|$)") {
    Write-Host "Missing hosts entry for local.revolt.chat."
    Write-Host "Run PowerShell as Administrator and execute:"
    Write-Host '  Add-Content -Path "$env:SystemRoot\System32\drivers\etc\hosts" -Value "`n127.0.0.1 local.revolt.chat"'
    exit 1
}

Write-Host "If this is your first HTTPS run, trust Caddy's local CA:"
Write-Host "  caddy trust"
Write-Host ""
Write-Host "Starting Caddy with $Caddyfile"

caddy run --config $Caddyfile --adapter caddyfile
