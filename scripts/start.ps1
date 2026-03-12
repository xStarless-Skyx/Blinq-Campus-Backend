Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function Write-Usage {
    @"
Usage: .\scripts\start.ps1 [options]

Builds selected backend binaries once, then starts them in parallel.

Options:
  --no-caddy           Do not start local Caddy reverse proxy
  --no-docs            Do not start docs site (npm)
  --with-pushd         Also start revolt-pushd
  --with-crond         Also start revolt-crond
  --with-voice-ingress Also start revolt-voice-ingress
  --skip-build         Skip cargo build step and run existing binaries
  --help, -h           Show this help text
"@ | Write-Host
}

# Load optional local env overrides for OAuth, etc.
$envFile = Join-Path $RootDir ".env.local"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -eq "" -or $line.StartsWith("#")) { return }
        $parts = $line.Split("=", 2)
        if ($parts.Count -eq 2) {
            $name = $parts[0].Trim()
            $value = $parts[1].Trim()
            [Environment]::SetEnvironmentVariable($name, $value)
            Set-Item -Path "Env:$name" -Value $value
        }
    }
}

$Services = New-Object System.Collections.Generic.List[string]
$Services.Add("revolt-delta")
$Services.Add("revolt-bonfire")
$Services.Add("revolt-autumn")
$Services.Add("revolt-january")
$Services.Add("revolt-gifbox")

$skipBuild = $false
$startCaddy = $true
$startDocs = $true

foreach ($arg in $args) {
    switch ($arg) {
        "--no-caddy" { $startCaddy = $false }
        "--no-docs" { $startDocs = $false }
        "--with-pushd" { $Services.Add("revolt-pushd") }
        "--with-crond" { $Services.Add("revolt-crond") }
        "--with-voice-ingress" { $Services.Add("revolt-voice-ingress") }
        "--skip-build" { $skipBuild = $true }
        "--help" { Write-Usage; exit 0 }
        "-h" { Write-Usage; exit 0 }
        default {
            Write-Host "Unknown option: $arg" -ForegroundColor Red
            Write-Usage
            exit 1
        }
    }
}

function Display-NameForBin([string]$bin) {
    switch ($bin) {
        "revolt-delta" { return "blinqcampus-api" }
        "revolt-bonfire" { return "blinqcampus-events" }
        "revolt-autumn" { return "blinqcampus-files" }
        "revolt-january" { return "blinqcampus-metadata" }
        "revolt-gifbox" { return "blinqcampus-gifbox" }
        "revolt-pushd" { return "blinqcampus-pushd" }
        "revolt-crond" { return "blinqcampus-crond" }
        "revolt-voice-ingress" { return "blinqcampus-voice-ingress" }
        default { return $bin }
    }
}

function Port-ForBin([string]$bin) {
    switch ($bin) {
        "revolt-delta" { return 14702 }
        "revolt-bonfire" { return 14703 }
        "revolt-autumn" { return 14704 }
        "revolt-january" { return 14705 }
        "revolt-gifbox" { return 14706 }
        default { return 0 }
    }
}

function Wait-ForPort([string]$host, [int]$port, [string]$label, [int]$timeoutSeconds = 60) {
    Write-Host "Waiting for $label on $host:$port..."
    $start = Get-Date
    while ($true) {
        $ok = $false
        try {
            $result = Test-NetConnection -ComputerName $host -Port $port -WarningAction SilentlyContinue
            $ok = $result.TcpTestSucceeded
        } catch {
            $ok = $false
        }

        if ($ok) {
            Write-Host "$label is reachable."
            return
        }

        if ((Get-Date) - $start -gt (New-TimeSpan -Seconds $timeoutSeconds)) {
            throw "Timed out waiting for $label on $host:$port."
        }

        Start-Sleep -Seconds 1
    }
}

foreach ($bin in $Services) {
    if (Get-Process -Name $bin -ErrorAction SilentlyContinue) {
        Write-Host "An existing $bin process is already running." -ForegroundColor Red
        Write-Host "Stop old backend processes first, e.g.:"
        Write-Host "  taskkill /F /IM revolt-*.exe"
        exit 1
    }

    $port = Port-ForBin $bin
    if ($port -ne 0) {
        $listeners = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue
        if ($listeners) {
            Write-Host "Port $port is already in use (needed by $bin)." -ForegroundColor Red
            Write-Host "Free the port or stop existing backend processes, then retry."
            exit 1
        }
    }
}

if ($startDocs) {
    $docsPort = 14701
    $docsListeners = Get-NetTCPConnection -LocalPort $docsPort -State Listen -ErrorAction SilentlyContinue
    if ($docsListeners) {
        Write-Host "Port $docsPort is already in use; leaving existing frontend/docs running."
        $startDocs = $false
    }
}

Wait-ForPort "127.0.0.1" 27017 "MongoDB"
Wait-ForPort "127.0.0.1" 6379 "Redis"
Wait-ForPort "127.0.0.1" 5672 "RabbitMQ"
Wait-ForPort "127.0.0.1" 14009 "MinIO"
Start-Sleep -Seconds 2

$processes = New-Object System.Collections.Generic.List[System.Diagnostics.Process]

if ($startCaddy) {
    Write-Host "Starting Caddy..."
    $caddyScript = Join-Path $RootDir "scripts/start-caddy-local.ps1"
    $caddyProc = Start-Process -FilePath "powershell" -ArgumentList @("-ExecutionPolicy", "Bypass", "-File", $caddyScript) -PassThru
    $processes.Add($caddyProc)
}

if ($startDocs) {
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Write-Host "npm is not installed. Start with --no-docs or install Node.js." -ForegroundColor Red
        exit 1
    }

    $docsDir = Join-Path $RootDir "docs"
    $nodeModules = Join-Path $docsDir "node_modules"
    if (-not (Test-Path $nodeModules)) {
        Write-Host "Installing docs dependencies..."
        Push-Location $docsDir
        try {
            & npm install
        } finally {
            Pop-Location
        }
    }

    Write-Host "Starting docs site (npm)..."
    $docsProc = Start-Process -FilePath "npm" -ArgumentList @("run", "start", "--", "--port", "14701") -WorkingDirectory $docsDir -PassThru
    $processes.Add($docsProc)
}

if (-not $skipBuild) {
    Write-Host "Building binaries..."
    $buildArgs = @()
    foreach ($bin in $Services) {
        $buildArgs += "--bin"
        $buildArgs += $bin
    }
    & cargo build @buildArgs
}

foreach ($bin in $Services) {
    $label = Display-NameForBin $bin
    Write-Host "Starting $label ($bin)"
    $exePath = Join-Path $RootDir ("target/debug/{0}.exe" -f $bin)
    $proc = Start-Process -FilePath $exePath -PassThru
    $processes.Add($proc)
}

Write-Host "All services started. Press Ctrl+C to stop."

try {
    while ($true) {
        foreach ($proc in $processes) {
            $proc.Refresh()
            if ($proc.HasExited) {
                $exitCode = $proc.ExitCode
                Write-Host "A service exited (code $exitCode). Stopping remaining services..."
                exit $exitCode
            }
        }
        Start-Sleep -Seconds 1
    }
} finally {
    if ($processes.Count -gt 0) {
        Write-Host "Shutting down services..."
        $ids = $processes | ForEach-Object { $_.Id }
        Stop-Process -Id $ids -ErrorAction SilentlyContinue
    }
}
