#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CADDYFILE="$ROOT_DIR/Caddyfile.local"

check_prereqs() {
  if ! command -v caddy >/dev/null 2>&1; then
    echo "caddy is not installed."
    echo "Install Caddy using your OS package manager."
    echo "macOS (Homebrew): brew install caddy"
    echo "Windows (winget): winget install CaddyServer.Caddy"
    return 1
  fi

  if ! grep -Eq '(^|[[:space:]])local\.blinqcampus\.chat([[:space:]]|$)' /etc/hosts; then
    echo "Missing /etc/hosts entry for local.blinqcampus.chat."
    echo "Add this line:"
    echo "127.0.0.1 local.blinqcampus.chat"
    return 1
  fi
}

if [ "${1:-}" = "--check" ]; then
  check_prereqs
  exit $?
fi

check_prereqs

echo "If this is your first HTTPS run, trust Caddy's local CA:"
echo "  caddy trust"
echo
echo "Starting Caddy with $CADDYFILE"
exec caddy run --config "$CADDYFILE" --adapter caddyfile
