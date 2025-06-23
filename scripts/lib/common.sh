#!/usr/bin/env bash
# shared logging helpers
set -euo pipefail

# ── color palette ──────────────────────────────────────────────────────────
c() { tput setaf "$1"; }     # fg
b() { tput bold; }
r() { tput sgr0; }

# ── log fns ────────────────────────────────────────────────────────────────
log()  { printf "%s[%s]%s %s\n" "$(b)$(c 2)✔$(r)" "$(date +%H:%M:%S)" "$(c 2)INFO$(r)"  "$*"; }
warn() { printf "%s[%s]%s %s\n" "$(b)$(c 3)!$(r)" "$(date +%H:%M:%S)" "$(c 3)WARN$(r)"  "$*"; }
err()  { printf "%s[%s]%s %s\n" "$(b)$(c 1)x$(r)" "$(date +%H:%M:%S)" "$(c 1)ERR $(r)" "$*"; }

# run_cmd "ls -la"
run_cmd() {
  printf "%s%s%s %s\n" "$(c 6)" "[CMD]" "$(r)" "$*"
  eval "$*"
}

