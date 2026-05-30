# Single-process local dev: build UI once, then serve API + static SPA on :47821
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

$env:ORCHESTRATOR_PROFILE = "dev"
$env:ORCHESTRATOR_BIND_ADDR = "127.0.0.1"
$env:ORCHESTRATOR_PORT = "47821"
$env:ORCHESTRATOR_DATA_DIR = Join-Path $Root ".data"

Write-Host "Building web UI..."
Push-Location (Join-Path $Root "web")
try {
    if (-not (Test-Path "node_modules")) {
        npm install
    }
    npm run build
} finally {
    Pop-Location
}

Write-Host "Starting orchestrator-server (API + UI) on http://127.0.0.1:47821 ..."
Set-Location $Root
cargo run -p orchestrator-server -- serve
