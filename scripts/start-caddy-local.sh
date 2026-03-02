#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CADDYFILE="$ROOT_DIR/Caddyfile.local"

if ! command -v caddy >/dev/null 2>&1; then
  echo "caddy is not installed."
  echo "Install Caddy using your OS package manager."
  echo "macOS (Homebrew): brew install caddy"
  echo "Windows (winget): winget install CaddyServer.Caddy"
  exit 1
fi

<<<<<<< HEAD
if ! grep -Eq '(^|[[:space:]])local\.(blinqcampus|revolt)\.chat([[:space:]]|$)' /etc/hosts; then
  echo "Missing /etc/hosts entry for local.blinqcampus.chat."
  echo "Add this line:"
  echo "127.0.0.1 local.blinqcampus.chat"
=======
if ! grep -Eq '(^|[[:space:]])local\.revolt\.chat([[:space:]]|$)' /etc/hosts; then
  echo "Missing /etc/hosts entry for local.revolt.chat."
  echo "Add this line:"
  echo "127.0.0.1 local.revolt.chat"
>>>>>>> e99df03359637127adadf91224df853828eb0569
  exit 1
fi

echo "If this is your first HTTPS run, trust Caddy's local CA:"
echo "  caddy trust"
echo
echo "Starting Caddy with $CADDYFILE"
exec caddy run --config "$CADDYFILE" --adapter caddyfile
