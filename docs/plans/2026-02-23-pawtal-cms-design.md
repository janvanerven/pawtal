# Pawtal CMS Design Document

**Date:** 2026-02-23
**Status:** Approved

## Overview

Pawtal ("portal + paw") is a self-hosted CMS for managing a website themed around applications. It features a warm, playful design language with mascot characters, card-based UI, and a full-featured content management system.

## Tech Stack

- **Backend:** Rust + Axum
- **Frontend:** SvelteKit (single app: public site + admin panel)
- **Database:** SQLite (via sqlx)
- **Auth:** OAuth2 via Authentik
- **File storage:** Local filesystem (Docker volume)
- **Deployment:** Docker container behind reverse proxy (HTTP only)

## Design Language

### Mascots
- **Koda the Red Panda** — helpful guide (onboarding, empty states, tips)
- **Milo the Koala** — chill companion (loading states, success messages, quiet moments)
- Soft hand-drawn watercolor illustration style, used as accents

### Color Palette
| Role       | Color              | Hex     |
|------------|--------------------|---------|
| Primary    | Warm amber/orange  | #E8924A |
| Secondary  | Soft sage green    | #7BA68C |
| Accent     | Coral red          | #E85D5D |
| Background | Warm cream         | #FFF8F0 |
| Surface    | Soft white         | #FFFFFF |
| Text       | Warm charcoal      | #3D3229 |

### Typography
- Headers: Nunito (rounded sans-serif)
- Body: Inter (clean sans)

### UI Characteristics
- Rounded corners (8px/12px/16px), warm shadows
- Mobile-first responsive design
- Sidebar collapses to bottom nav on mobile
- Card-based layout

## Project Structure

```
pawtal/
├── backend/                    # Rust Axum API
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs           # App configuration
│   │   ├── db/                 # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── migrations/     # SQL migrations
│   │   │   └── models/         # Data models
│   │   ├── api/                # REST API handlers
│   │   │   ├── mod.rs
│   │   │   ├── pages.rs
│   │   │   ├── articles.rs
│   │   │   ├── media.rs
│   │   │   ├── apps.rs
│   │   │   ├── menus.rs
│   │   │   ├── settings.rs
│   │   │   ├── auth.rs
│   │   │   └── search.rs
│   │   ├── auth/               # OAuth2 + session management
│   │   ├── media/              # Image processing (resize, WebP)
│   │   └── services/           # Business logic layer
│   └── tests/
├── frontend/                   # SvelteKit app
│   ├── package.json
│   ├── src/
│   │   ├── routes/
│   │   │   ├── (public)/       # Public-facing site
│   │   │   │   ├── +layout.svelte
│   │   │   │   ├── +page.svelte
│   │   │   │   ├── [slug]/+page.svelte
│   │   │   │   ├── articles/
│   │   │   │   └── apps/
│   │   │   └── admin/          # Admin panel
│   │   │       ├── +layout.svelte
│   │   │       ├── pages/
│   │   │       ├── articles/
│   │   │       ├── media/
│   │   │       ├── apps/
│   │   │       ├── menus/
│   │   │       └── settings/
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   ├── api/
│   │   │   └── stores/
│   │   └── app.css
│   └── static/
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
└── docs/
    └── plans/
```

## Data Model

### Users (synced from OAuth2)
- `id`, `external_id` (from Authentik), `email`, `display_name`, `role` (admin/editor), `created_at`, `last_login`

### Pages
- `id`, `title`, `slug` (unique), `content` (HTML from rich text editor), `status` (draft/published/scheduled/trashed), `publish_at` (nullable), `author_id`, `created_at`, `updated_at`, `trashed_at`

### Page Revisions
- `id`, `page_id`, `title`, `content`, `author_id`, `created_at`
- Created automatically on each save. Restoring = creating new revision from old one.

### Articles
- `id`, `title`, `slug`, `short_text`, `content`, `status`, `publish_at`, `author_id`, `created_at`, `updated_at`, `trashed_at`

### Article Revisions
- Same pattern as page revisions

### Categories
- `id`, `name`, `slug`
- Many-to-many with articles (`article_categories`) and pages (`page_categories`)

### Media
- `id`, `filename`, `original_filename`, `mime_type`, `size_bytes`, `width`, `height`, `alt_text`, `is_icon`, `uploaded_by`, `created_at`
- Physical files: `uploads/{id}/original.{ext}`, `thumbnail.{ext}`, `medium.{ext}`, `large.{ext}` + WebP variants

### App Catalogue Items
- `id`, `name`, `description`, `icon_id` (references media), `url` (external link), `page_id` (nullable, for dedicated page), `sort_order`, `created_at`, `updated_at`

### Menus
- `id`, `name` (e.g., "main", "footer")
- Menu Items: `id`, `menu_id`, `label`, `link_type` (page/article/url/app_catalogue), `link_target`, `parent_id` (nullable), `sort_order`

### Site Settings (key-value)
- `key`, `value`, `updated_at`
- Keys: `site_title`, `front_page_type`, `front_page_id`, `apps_per_page`, `app_catalogue_intro`, `dark_mode_default`

### Audit Log
- `id`, `user_id`, `action`, `entity_type`, `entity_id`, `details` (JSON), `created_at`

### Sessions
- `id`, `user_id`, `token`, `expires_at`, `created_at`

### Status Flow
Draft -> Published / Scheduled -> Published (auto) / any -> Trashed -> permanent delete after 30 days

## Authentication & Authorization

### OAuth2 Flow
1. User visits `/admin` -> redirected to Authentik login
2. Authentik authenticates -> redirects back with authorization code
3. Backend exchanges code for tokens, extracts user info
4. Backend creates/updates local user record, issues session cookie
5. Session stored in SQLite with expiry

### Roles
- **Admin:** Full access (all CRUD, settings, user management, audit log)
- **Editor:** Create/edit/publish pages and articles, manage media. No settings or audit log access.

### Security
- All `/api/admin/*` endpoints require valid session + role check
- Public endpoints are unauthenticated
- CSRF protection via `SameSite=Strict` cookies + origin check on mutations

## API Endpoints

### Public API (`/api/`)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/pages/:slug` | Get published page |
| GET | `/api/articles` | List published articles (paginated) |
| GET | `/api/articles/:slug` | Get published article |
| GET | `/api/apps` | List apps (paginated) |
| GET | `/api/menus/:name` | Get menu structure |
| GET | `/api/settings/public` | Public settings |
| GET | `/api/search?q=&type=` | Full-text search |

### Admin API (`/api/admin/`) — requires auth
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET/POST | `/api/admin/pages` | List / create pages |
| GET/PUT/DELETE | `/api/admin/pages/:id` | Get / update / trash page |
| POST | `/api/admin/pages/:id/publish` | Publish page |
| POST | `/api/admin/pages/:id/restore` | Restore from trash |
| GET | `/api/admin/pages/:id/revisions` | List revisions |
| POST | `/api/admin/pages/:id/revisions/:rev_id/restore` | Restore revision |
| GET | `/api/admin/pages/:id/preview` | Preview as visitor |
| — | *Same pattern for articles* | — |
| GET/POST | `/api/admin/media` | List / upload media |
| DELETE | `/api/admin/media/:id` | Delete media |
| GET/POST | `/api/admin/apps` | List / create apps |
| GET/PUT/DELETE | `/api/admin/apps/:id` | CRUD apps |
| PUT | `/api/admin/apps/reorder` | Reorder apps |
| GET/PUT | `/api/admin/menus/:name` | Get / update menu |
| GET/PUT | `/api/admin/settings` | Get / update settings |
| GET/POST/PUT/DELETE | `/api/admin/categories` | CRUD categories |
| GET | `/api/admin/audit-log` | View audit log (admin only) |
| GET/POST | `/api/admin/trash` | List / empty trash |
| GET | `/api/admin/users` | List users (admin only) |
| PUT | `/api/admin/users/:id/role` | Change role (admin only) |
| POST | `/api/auth/login` | Start OAuth2 flow |
| GET | `/api/auth/callback` | OAuth2 callback |
| POST | `/api/auth/logout` | End session |

## Frontend Architecture

### Public Site
- **Front page** — configurable: page, article list, or app catalogue
- **Page view** — `/[slug]`
- **Articles** — `/articles` (paginated list), `/articles/[slug]` (full)
- **App catalogue** — `/apps` (grid with icons, paginated)
- **Search** — `/search?q=`
- **Navigation** — sidebar on desktop, bottom nav on mobile
- **Dark/light mode** — toggle in header, stored in localStorage

### Admin Panel (`/admin`)
- **Dashboard** — recent activity, draft counts, Koda greeting
- **Pages/Articles** — sortable table, status filters, rich text editor (TipTap)
- **Media library** — grid view, drag-and-drop upload, alt text editing
- **App catalogue manager** — drag-and-drop reorder, icon upload
- **Menu editor** — drag-and-drop tree structure
- **Settings** — site title, front page selection, apps per page
- **Trash** — restore / permanent delete
- **Audit log** — filterable activity feed (admin only)

### Rich Text Editor: TipTap
- ProseMirror-based with Svelte wrapper
- Features: headings, bold/italic, lists, links, blockquotes, code blocks, image embed (from media library), tables
- Stores content as HTML

### Design Tokens
```css
:root {
  --color-primary: #E8924A;
  --color-secondary: #7BA68C;
  --color-accent: #E85D5D;
  --color-bg: #FFF8F0;
  --color-surface: #FFFFFF;
  --color-text: #3D3229;
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --shadow-sm: 0 1px 3px rgba(61,50,41,0.08);
  --shadow-md: 0 4px 12px rgba(61,50,41,0.12);
  --font-heading: 'Nunito', sans-serif;
  --font-body: 'Inter', sans-serif;
}
```

## Image Processing Pipeline

### Upload Flow
1. User uploads via media library or inline editor
2. Original saved to `uploads/{id}/original.{ext}`
3. Async processing generates variants:
   - Thumbnail: 200x200, cropped square
   - Medium: max 800px wide
   - Large: max 1600px wide
   - WebP versions of each
4. App icons get additional 128x128 square variant

### Libraries
- `image` crate for resize/crop
- `webp` crate for WebP conversion

## Search (SQLite FTS5)

- FTS5 virtual tables for pages, articles, and apps
- Triggers keep FTS in sync with source tables
- Only indexes published content
- `GET /api/search?q=term&type=all|pages|articles|apps`
- Returns unified results with type, title, slug, highlighted snippet
- Ranked by BM25 relevance
- Admin search includes drafts and trashed items

## Deployment

### Docker Compose
```yaml
services:
  pawtal:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - pawtal-data:/app/data
      - pawtal-uploads:/app/uploads
    environment:
      - OAUTH2_CLIENT_ID=...
      - OAUTH2_CLIENT_SECRET=...
      - OAUTH2_ISSUER_URL=...
      - SESSION_SECRET=...
      - BASE_URL=https://your-domain.com

volumes:
  pawtal-data:
  pawtal-uploads:
```

### Dockerfile (multi-stage)
1. Build Rust: `cargo build --release`
2. Build SvelteKit: `npm run build` (adapter-node)
3. Runtime: debian-slim with Rust binary + SvelteKit build

### Background Tasks (tokio)
- Runs every minute:
  - Publish scheduled content whose `publish_at` has passed
  - Permanently delete trashed items older than 30 days
