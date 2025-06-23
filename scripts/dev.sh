#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

export DATABASE_URL="postgres://setlist:setlist@localhost/setlist"
export AMQP_URL="amqp://setlist:setlist@localhost:5672/%2f"

ROOT_DIR="$(git -C "$SCRIPT_DIR/../.." rev-parse --show-toplevel 2>/dev/null || pwd)"
BACKEND="$ROOT_DIR/backend"

cleanup() {
  log "Shutting down dev stack"
  run_cmd "docker compose -f infra/docker-compose.yml down -v"
  pkill -P $$ || true
  r
}
trap cleanup INT TERM

log "Starting Docker services"
run_cmd "docker compose -f infra/docker-compose.yml up -d"

log "Waiting for Postgres"
until pg_isready -h localhost -p 5432 -U setlist &>/dev/null; do sleep 0.5; done
log "Postgres is ready"

log "Waiting for RabbitMQ"
until nc -z localhost 5672 2>/dev/null; do sleep 0.5; done
log "RabbitMQ is ready"

log "Applying SQL migrations"
run_cmd "(cd $BACKEND && sqlx migrate run)"

log "Checking Rust workspace"
run_cmd "cargo check --workspace --manifest-path $BACKEND/Cargo.toml"

run_comp () (
  name="$1"; color="$2"; shift 2
  { printf "%s%s%s started\n" "$(c "$color")" "$name" "$(r)"; "$@"; } |
  while IFS= read -r line; do
    printf "%s%s%s %s\n" "$(c "$color")" "$name" "$(r)" "$line"
  done
)

run_comp API    4 cargo run -q -p api    --manifest-path "$BACKEND/Cargo.toml" &
run_comp WORKER 5 cargo run -q -p worker --manifest-path "$BACKEND/Cargo.toml" &

wait

