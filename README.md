# Blinq Campus Backend

# Still in development, everything currently runs locally

Backend services for Blinq Campus (API, events, media, and proxy services).

This repository is a customized fork for local development and deployment. Some internal crate and binary names still use `revolt-*` for compatibility.

## What This Runs

Core local services:
- `revolt-delta` (API) on `14702`
- `revolt-bonfire` (events / websocket) on `14703`
- `revolt-autumn` (files) on `14704`
- `revolt-january` (metadata proxy) on `14705`
- `revolt-gifbox` (gif/search proxy) on `14706`

Docker dependencies:
- MongoDB
- Redis
- RabbitMQ
- MinIO
- Maildev
- LiveKit

## Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- Docker Desktop (running)
- `docker compose`

Optional but recommended:
- `mise` (if you use the project toolchain bootstrap)

## Quick Start (Backend Only)

From the repo root:

```bash
docker compose up -d
./scripts/start.sh
```

Expected startup lines include:
<<<<<<< HEAD
- `Starting blinqcampus-api (revolt-delta)`
- `Starting blinqcampus-events (revolt-bonfire)`
- `Starting blinqcampus-files (revolt-autumn)`
- `Starting blinqcampus-metadata (revolt-january)`
- `Starting blinqcampus-gifbox (revolt-gifbox)`
=======
- `Starting revolt-delta`
- `Starting revolt-bonfire`
- `Starting revolt-autumn`
- `Starting revolt-january`
- `Starting revolt-gifbox`
>>>>>>> e99df03359637127adadf91224df853828eb0569

## Verify It Is Running

```bash
docker compose ps
lsof -nP -iTCP:14702 -sTCP:LISTEN
lsof -nP -iTCP:14703 -sTCP:LISTEN
```

Health checks:
- API: `http://localhost:14702`
- Events WS: `ws://localhost:14703`

## Stop Services

Stop Rust services started by `start.sh`:

```bash
pkill -f 'target/debug/revolt-'
```

Stop Docker dependencies:

```bash
docker compose down
```

## HTTPS Local Development (OAuth-Compatible)

For Google OAuth and secure-cookie flows, use HTTPS local endpoints via reverse proxy:
<<<<<<< HEAD
- `https://local.blinqcampus.chat:24701` -> web
- `https://local.blinqcampus.chat:24702` -> API
- `https://local.blinqcampus.chat:24703` -> events
=======
- `https://local.revolt.chat:24701` -> web
- `https://local.revolt.chat:24702` -> API
- `https://local.revolt.chat:24703` -> events
>>>>>>> e99df03359637127adadf91224df853828eb0569

Use the helper docs and scripts in `docs/local-https.md` and `scripts/start-caddy-local.*`.

## Common Issues

### Docker daemon not running

If you see:
`Cannot connect to the Docker daemon ... Is the docker daemon running?`

Start Docker Desktop, then run:

```bash
docker compose up -d
```

### Address already in use

If startup fails with `AddrInUse` on one of the backend ports:

```bash
lsof -nP -iTCP:14702 -sTCP:LISTEN
lsof -nP -iTCP:14703 -sTCP:LISTEN
pkill -f 'target/debug/revolt-'
```

Then restart with `./scripts/start.sh`.

### Websocket 502 through reverse proxy

If Caddy logs show `connect: connection refused` for `14703`, `revolt-bonfire` is not running.

Start services again:

```bash
./scripts/start.sh
```

### Cargo lock/build lock waits

`Blocking waiting for file lock` means another cargo process is active.
<<<<<<< HEAD

Check and stop old processes:

```bash
ps aux | rg cargo
pkill -f cargo
```

Then restart.

## OAuth Notes (Current Local Setup)

This fork uses local OAuth callback handling in backend route code and local HTTPS endpoints.

If OAuth fails, check:
1. Backend is running from this repo copy.
2. Google OAuth app has exact redirect URI configured.
3. Local HTTPS host/ports match your runtime (`24701`/`24702`).

## Repository Layout

- `crates/delta` - API service
- `crates/bonfire` - events/websocket service
- `crates/services/autumn` - file service
- `crates/services/january` - metadata/embed service
- `crates/services/gifbox` - gif/search proxy
- `scripts/start.sh` - run all backend services together
- `compose.yml` - local dependency stack
- `Revolt.toml` - runtime config

## Development Notes

- This repository may contain active customizations for Blinq Campus policy and admin tooling.
- Keep route names and internal types stable unless frontend and SDK consumers are updated in lockstep.
=======

Check and stop old processes:

```bash
ps aux | rg cargo
pkill -f cargo
```

Then restart.

## OAuth Notes (Current Local Setup)

This fork uses local OAuth callback handling in backend route code and local HTTPS endpoints.

If OAuth fails, check:
1. Backend is running from this repo copy.
2. Google OAuth app has exact redirect URI configured.
3. Local HTTPS host/ports match your runtime (`24701`/`24702`).

## Repository Layout

- `crates/delta` - API service
- `crates/bonfire` - events/websocket service
- `crates/services/autumn` - file service
- `crates/services/january` - metadata/embed service
- `crates/services/gifbox` - gif/search proxy
- `scripts/start.sh` - run all backend services together
- `compose.yml` - local dependency stack
- `Revolt.toml` - runtime config

## Development Notes

- This repository may contain active customizations for Blinq Campus policy and admin tooling.
- Keep route names and internal types stable unless frontend and SDK consumers are updated in lockstep.

>>>>>>> e99df03359637127adadf91224df853828eb0569
