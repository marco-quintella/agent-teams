# Starts API (dev profile) and Vite UI for local development.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

$env:ORCHESTRATOR_PROFILE = "dev"
$env:ORCHESTRATOR_BIND_ADDR = "127.0.0.1"
$env:ORCHESTRATOR_PORT = "47821"
$env:ORCHESTRATOR_DATA_DIR = Join-Path $Root ".data"

Write-Host "Starting orchestrator-server on http://127.0.0.1:47821 ..."
$server = Start-Process -FilePath "cargo" -ArgumentList @("run", "-p", "orchestrator-server", "--", "serve") -WorkingDirectory $Root -PassThru -NoNewWindow

Start-Sleep -Seconds 2
Write-Host "Starting Vite on http://localhost:5173 ..."
Push-Location (Join-Path $Root "web")
try {
    if (-not (Test-Path "node_modules")) {
        npm install
    }
    npm run dev
} finally {
    Pop-Location
    if (-not $server.HasExited) {
        Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
    }
}
