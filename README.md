# Pawtal

A self-hosted CMS built with Rust (Axum) and SvelteKit. Manages pages, articles, media, app catalogues, and menus — all behind OAuth2 authentication.

## Features

- Page & article management with rich text editor (TipTap)
- Revision history and scheduled publishing
- Media library with automatic image optimization (WebP, thumbnails)
- App catalogue with custom icons
- Menu structure editor
- Full-text search (SQLite FTS5)
- Trash / soft-delete with auto-cleanup
- Audit log
- Dark / light mode
- Role-based access (admin / editor)

## Quick Start

### 1. Prerequisites

- Docker and Docker Compose
- An OAuth2 / OIDC provider (e.g. [Authentik](https://goauthentik.io/))

### 2. Configure

Copy the example environment file and fill in your values:

```bash
cp .env.example .env
```

| Variable | Description |
|---|---|
| `OAUTH2_CLIENT_ID` | Client ID from your OIDC provider |
| `OAUTH2_CLIENT_SECRET` | Client secret from your OIDC provider |
| `OAUTH2_ISSUER_URL` | OIDC issuer URL (e.g. `https://auth.example.com/application/o/pawtal`) |
| `SESSION_SECRET` | A random string used to sign session cookies. Generate one with `openssl rand -hex 32` |
| `BASE_URL` | The public URL where Pawtal will be accessible (e.g. `https://pawtal.example.com`) |

### 3. Run

```bash
docker compose up -d
```

Pawtal will be available on port **8080**. Point your reverse proxy (Nginx, Caddy, Traefik, etc.) at `http://localhost:8080`.

### 4. OIDC Provider Setup

In your OAuth2 provider, create an application with:

- **Redirect URI:** `https://your-domain.com/api/auth/callback`
- **Scopes:** `openid profile email`
- **Grant type:** Authorization Code

The first user to log in is automatically assigned the **admin** role. Subsequent users get the **editor** role (admins can change roles from the admin panel).

## Architecture

```
┌─────────────────────────────────────┐
│           Docker Container          │
│                                     │
│  Axum (port 8080)                   │
│  ├── /api/*      → REST API         │
│  ├── /uploads/*  → static files     │
│  └── /*          → reverse proxy ──→ SvelteKit (port 3000)
│                                     │
│  SQLite database    /app/data/      │
│  Uploaded files     /app/uploads/   │
└─────────────────────────────────────┘
```

- **Axum** serves the API and proxies all other requests to SvelteKit
- **SvelteKit** (adapter-node) handles SSR for the admin panel and public site
- **SQLite** stores all data with WAL mode enabled
- Only port 8080 is exposed

## Volumes

| Path | Purpose |
|---|---|
| `/app/data` | SQLite database |
| `/app/uploads` | Uploaded media files |

Both are configured as named Docker volumes by default. Back them up regularly.

## Using the GHCR Image

Pre-built images are published to GitHub Container Registry on every push to `main`:

```bash
docker pull ghcr.io/janvanerven/pawtal:latest
```

To use the pre-built image instead of building locally, update `docker-compose.yml`:

```yaml
services:
  pawtal:
    image: ghcr.io/janvanerven/pawtal:latest
    # remove the "build: ." line
```

## Development

### Backend

```bash
cd backend
cargo run
```

Requires Rust 1.85+ (edition 2024).

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The Vite dev server proxies `/api` and `/uploads` to `localhost:8080`.

## License

MIT
