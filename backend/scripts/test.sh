#!/usr/bin/env bash
set -euo pipefail

# directory that *this* file lives in
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
# repo root = parent of scripts/
ROOT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"

echo "▶ building workspace"
cargo build --quiet --bins --manifest-path "$ROOT_DIR/Cargo.toml"

IMPORT_BIN="$ROOT_DIR/target/debug/worker-import"
API_BIN="$ROOT_DIR/target/debug/api"
WORKER_BIN="$ROOT_DIR/target/debug/worker"

echo "▶ running e2e tests"
IMPORT_BIN="$IMPORT_BIN" \
API_BIN="$API_BIN" \
WORKER_BIN="$WORKER_BIN" \
cargo test -p e2e -- --nocapture --test-threads=1

