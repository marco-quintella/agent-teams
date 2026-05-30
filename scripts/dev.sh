#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

export ORCHESTRATOR_PROFILE=dev
export ORCHESTRATOR_BIND_ADDR=127.0.0.1
export ORCHESTRATOR_PORT=47821
export ORCHESTRATOR_DATA_DIR="${ROOT}/.data"

echo "Starting orchestrator-server on http://127.0.0.1:47821 ..."
cargo run -p orchestrator-server -- serve &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true' EXIT

sleep 2
cd "${ROOT}/web"
if [[ ! -d node_modules ]]; then
  npm install
fi
npm run dev
