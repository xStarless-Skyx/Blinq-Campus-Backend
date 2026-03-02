# Local HTTPS Setup

This repo now includes:
- `Caddyfile.local`
- `scripts/start-caddy-local.sh`
- `scripts/start-caddy-local.ps1`

## 1. Prerequisites

### macOS

```bash
brew install caddy
```

Ensure local DNS host exists:

```bash
<<<<<<< HEAD
echo "127.0.0.1 local.blinqcampus.chat" | sudo tee -a /etc/hosts
=======
echo "127.0.0.1 local.revolt.chat" | sudo tee -a /etc/hosts
>>>>>>> e99df03359637127adadf91224df853828eb0569
```

Trust Caddy's local CA once:

```bash
caddy trust
```

### Windows (PowerShell)

Install Caddy:

```powershell
winget install CaddyServer.Caddy
```

Add host entry (run as Administrator):

```powershell
<<<<<<< HEAD
Add-Content -Path "$env:SystemRoot\System32\drivers\etc\hosts" -Value "`n127.0.0.1 local.blinqcampus.chat"
=======
Add-Content -Path "$env:SystemRoot\System32\drivers\etc\hosts" -Value "`n127.0.0.1 local.revolt.chat"
>>>>>>> e99df03359637127adadf91224df853828eb0569
```

Trust Caddy's local CA once:

```powershell
caddy trust
```

## 2. Start backend + web as usual

Backend:

```bash
docker-compose up -d
./scripts/start.sh --skip-build
```

Web client (example):

```bash
<<<<<<< HEAD
echo "VITE_API_URL=https://local.blinqcampus.chat:24702" > .env.local
=======
echo "VITE_API_URL=https://local.revolt.chat:24702" > .env.local
>>>>>>> e99df03359637127adadf91224df853828eb0569
yarn dev --port 14701
```

## 3. Start HTTPS proxy

In another terminal:

macOS / Linux:

```bash
./scripts/start-caddy-local.sh
```

Windows (PowerShell):

```powershell
.\scripts\start-caddy-local.ps1
```

This proxies:
<<<<<<< HEAD
- `https://local.blinqcampus.chat:24701` -> `http://127.0.0.1:14701`
- `https://local.blinqcampus.chat:24702` -> `http://127.0.0.1:14702`
- `https://local.blinqcampus.chat:24703` -> `http://127.0.0.1:14703`
- `https://local.blinqcampus.chat:24704` -> `http://127.0.0.1:14704`
- `https://local.blinqcampus.chat:24705` -> `http://127.0.0.1:14705`
- `https://local.blinqcampus.chat:24706` -> `http://127.0.0.1:14706`
=======
- `https://local.revolt.chat:24701` -> `http://127.0.0.1:14701`
- `https://local.revolt.chat:24702` -> `http://127.0.0.1:14702`
- `https://local.revolt.chat:24703` -> `http://127.0.0.1:14703`
- `https://local.revolt.chat:24704` -> `http://127.0.0.1:14704`
- `https://local.revolt.chat:24705` -> `http://127.0.0.1:14705`
- `https://local.revolt.chat:24706` -> `http://127.0.0.1:14706`
>>>>>>> e99df03359637127adadf91224df853828eb0569

## 4. Google OAuth values

Google Cloud OAuth client:
<<<<<<< HEAD
- Authorized JavaScript origin: `https://local.blinqcampus.chat:24701`
- Authorized redirect URI: `https://local.blinqcampus.chat:24702/auth/session/oauth/google/callback`
=======
- Authorized JavaScript origin: `https://local.revolt.chat:24701`
- Authorized redirect URI: `https://local.revolt.chat:24702/auth/session/oauth/google/callback`
>>>>>>> e99df03359637127adadf91224df853828eb0569

Backend env vars:

macOS / Linux:

```bash
export GOOGLE_OAUTH_CLIENT_ID="..."
export GOOGLE_OAUTH_CLIENT_SECRET="..."
<<<<<<< HEAD
export GOOGLE_OAUTH_REDIRECT_URI="https://local.blinqcampus.chat:24702/auth/session/oauth/google/callback"
=======
export GOOGLE_OAUTH_REDIRECT_URI="https://local.revolt.chat:24702/auth/session/oauth/google/callback"
>>>>>>> e99df03359637127adadf91224df853828eb0569
```

Windows (PowerShell):

```powershell
$env:GOOGLE_OAUTH_CLIENT_ID="..."
$env:GOOGLE_OAUTH_CLIENT_SECRET="..."
<<<<<<< HEAD
$env:GOOGLE_OAUTH_REDIRECT_URI="https://local.blinqcampus.chat:24702/auth/session/oauth/google/callback"
=======
$env:GOOGLE_OAUTH_REDIRECT_URI="https://local.revolt.chat:24702/auth/session/oauth/google/callback"
>>>>>>> e99df03359637127adadf91224df853828eb0569
```
