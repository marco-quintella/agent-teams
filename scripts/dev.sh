#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

export ORCHESTRATOR_PROFILE=dev
export ORCHESTRATOR_BIND_ADDR=127.0.0.1
export ORCHESTRATOR_PORT=47821
export ORCHESTRATOR_DATA_DIR="${ROOT}/.data"

echo "Building web UI..."
cd "${ROOT}/web"
if [[ ! -d node_modules ]]; then
  npm install
fi
npm run build
cd "${ROOT}"

echo "Starting orchestrator-server (API + UI) on http://127.0.0.1:47821 ..."
cargo run -p orchestrator-server -- serve
