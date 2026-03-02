#!/usr/bin/env bash

set -euo pipefail

# Core services typically needed for local development.
SERVICES=(
  revolt-delta
  revolt-bonfire
  revolt-autumn
  revolt-january
  revolt-gifbox
)

<<<<<<< HEAD
display_name_for_bin() {
  case "$1" in
    revolt-delta) echo "blinqcampus-api" ;;
    revolt-bonfire) echo "blinqcampus-events" ;;
    revolt-autumn) echo "blinqcampus-files" ;;
    revolt-january) echo "blinqcampus-metadata" ;;
    revolt-gifbox) echo "blinqcampus-gifbox" ;;
    revolt-pushd) echo "blinqcampus-pushd" ;;
    revolt-crond) echo "blinqcampus-crond" ;;
    revolt-voice-ingress) echo "blinqcampus-voice-ingress" ;;
    *) echo "$1" ;;
  esac
}

=======
>>>>>>> e99df03359637127adadf91224df853828eb0569
usage() {
  cat <<'USAGE'
Usage: ./scripts/start.sh [options]

Builds selected backend binaries once, then starts them in parallel.

Options:
  --with-pushd          Also start revolt-pushd
  --with-crond          Also start revolt-crond
  --with-voice-ingress  Also start revolt-voice-ingress
  --skip-build          Skip cargo build step and run existing binaries
  --help, -h            Show this help text
USAGE
}

skip_build=0

for arg in "$@"; do
  case "$arg" in
    --with-pushd)
      SERVICES+=(revolt-pushd)
      ;;
    --with-crond)
      SERVICES+=(revolt-crond)
      ;;
    --with-voice-ingress)
      SERVICES+=(revolt-voice-ingress)
      ;;
    --skip-build)
      skip_build=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $arg" >&2
      usage >&2
      exit 1
      ;;
  esac
done

pids=()

wait_for_port() {
  host="$1"
  port="$2"
  label="$3"
  timeout_seconds="${4:-60}"

  echo "Waiting for $label on $host:$port..."
  start_ts="$(date +%s)"
  while true; do
    if nc -z "$host" "$port" >/dev/null 2>&1; then
      echo "$label is reachable."
      return 0
    fi

    now_ts="$(date +%s)"
    if [ $((now_ts - start_ts)) -ge "$timeout_seconds" ]; then
      echo "Timed out waiting for $label on $host:$port." >&2
      return 1
    fi

    sleep 1
  done
}

port_for_bin() {
  case "$1" in
    revolt-delta) echo "14702" ;;
    revolt-bonfire) echo "14703" ;;
    revolt-autumn) echo "14704" ;;
    revolt-january) echo "14705" ;;
    revolt-gifbox) echo "14706" ;;
    *) echo "" ;;
  esac
}

cleanup() {
  trap - INT TERM EXIT

  if [ "${#pids[@]}" -gt 0 ]; then
    echo "Shutting down services..."
    kill "${pids[@]}" 2>/dev/null || true
    wait "${pids[@]}" 2>/dev/null || true
  fi
}

trap cleanup INT TERM EXIT

for bin in "${SERVICES[@]}"; do
  if pgrep -f "target/debug/$bin" >/dev/null 2>&1; then
    echo "An existing $bin process is already running." >&2
    echo "Stop old backend processes first, e.g.:" >&2
    echo "  pkill -f 'target/debug/revolt-'" >&2
    exit 1
  fi

  port="$(port_for_bin "$bin")"
  if [ -n "$port" ] && lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Port $port is already in use (needed by $bin)." >&2
    echo "Free the port or stop existing backend processes, then retry." >&2
    exit 1
  fi
done

wait_for_port "127.0.0.1" "27017" "MongoDB"
wait_for_port "127.0.0.1" "6379" "Redis"
wait_for_port "127.0.0.1" "5672" "RabbitMQ"
wait_for_port "127.0.0.1" "14009" "MinIO"
# Give RabbitMQ a brief grace period after socket open.
sleep 2

if [ "$skip_build" -eq 0 ]; then
  echo "Building binaries..."
  build_args=()
  for bin in "${SERVICES[@]}"; do
    build_args+=(--bin "$bin")
  done
  cargo build "${build_args[@]}"
fi

for bin in "${SERVICES[@]}"; do
<<<<<<< HEAD
  echo "Starting $(display_name_for_bin "$bin") ($bin)"
=======
  echo "Starting $bin"
>>>>>>> e99df03359637127adadf91224df853828eb0569
  "target/debug/$bin" &
  pids+=("$!")
done

echo "All services started. Press Ctrl+C to stop."

# Wait until any service exits; cleanup trap will stop the rest.
exit_code=0
while true; do
  for pid in "${pids[@]}"; do
    if ! kill -0 "$pid" 2>/dev/null; then
      wait "$pid" || exit_code=$?
      echo "A service exited (code $exit_code). Stopping remaining services..."
      exit "$exit_code"
    fi
  done
  sleep 1
done
