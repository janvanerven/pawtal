# Pawtal CMS Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a self-hosted CMS called Pawtal with a Rust/Axum backend, SvelteKit frontend, and SQLite database.

**Architecture:** Rust REST API backend with SvelteKit SPA frontend, both served from a single Docker container. OAuth2 auth via Authentik, local file storage for media, SQLite FTS5 for search.

**Tech Stack:** Rust (Axum, sqlx, image, webp, tower-http), SvelteKit (adapter-node, TipTap), SQLite, Docker

**Design doc:** `docs/plans/2026-02-23-pawtal-cms-design.md`

---

## Phase 1: Backend Foundation

### Task 1: Scaffold Rust Backend

**Files:**
- Create: `backend/Cargo.toml`
- Create: `backend/src/main.rs`
- Create: `backend/src/config.rs`

**Step 1: Create Cargo.toml with dependencies**

```toml
[package]
name = "pawtal"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
image = "0.25"
webp = "0.3"
reqwest = { version = "0.12", features = ["json"] }
tower = "0.5"
thiserror = "2"
```

**Step 2: Create config.rs**

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    pub uploads_dir: String,
    pub oauth2_client_id: String,
    pub oauth2_client_secret: String,
    pub oauth2_issuer_url: String,
    pub session_secret: String,
    pub base_url: String,
}

fn default_port() -> u16 {
    8080
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }
}
```

Add `envy = "0.4"` to Cargo.toml dependencies.

**Step 3: Create main.rs with health check**

```rust
mod config;

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .route("/api/health", get(health_check));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
```

**Step 4: Verify it compiles**

Run: `cd backend && cargo check`
Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add backend/
git commit -m "feat: scaffold Rust backend with Axum and health check"
```

---

### Task 2: Database Setup and Migration Runner

**Files:**
- Create: `backend/src/db/mod.rs`
- Create: `backend/migrations/001_initial_schema.sql`
- Modify: `backend/src/main.rs`

**Step 1: Write the initial migration**

Create `backend/migrations/001_initial_schema.sql`:

```sql
-- Users (synced from OAuth2)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    external_id TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'editor' CHECK (role IN ('admin', 'editor')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_login TEXT
);

-- Sessions
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

-- Site Settings (key-value)
CREATE TABLE IF NOT EXISTS site_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Default settings
INSERT OR IGNORE INTO site_settings (key, value) VALUES
    ('site_title', 'Pawtal'),
    ('front_page_type', 'articles'),
    ('front_page_id', ''),
    ('apps_per_page', '12'),
    ('app_catalogue_intro', ''),
    ('dark_mode_default', 'false');

-- Categories
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE
);

-- Pages
CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'scheduled', 'trashed')),
    publish_at TEXT,
    author_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    trashed_at TEXT
);

CREATE INDEX idx_pages_slug ON pages(slug);
CREATE INDEX idx_pages_status ON pages(status);

-- Page Revisions
CREATE TABLE IF NOT EXISTS page_revisions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_page_revisions_page ON page_revisions(page_id);

-- Page Categories (join table)
CREATE TABLE IF NOT EXISTS page_categories (
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (page_id, category_id)
);

-- Articles
CREATE TABLE IF NOT EXISTS articles (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    short_text TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'scheduled', 'trashed')),
    publish_at TEXT,
    author_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    trashed_at TEXT
);

CREATE INDEX idx_articles_slug ON articles(slug);
CREATE INDEX idx_articles_status ON articles(status);

-- Article Revisions
CREATE TABLE IF NOT EXISTS article_revisions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    short_text TEXT NOT NULL,
    content TEXT NOT NULL,
    author_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_article_revisions_article ON article_revisions(article_id);

-- Article Categories (join table)
CREATE TABLE IF NOT EXISTS article_categories (
    article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, category_id)
);

-- Media
CREATE TABLE IF NOT EXISTS media (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    alt_text TEXT NOT NULL DEFAULT '',
    is_icon INTEGER NOT NULL DEFAULT 0,
    uploaded_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- App Catalogue Items
CREATE TABLE IF NOT EXISTS apps (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon_id TEXT REFERENCES media(id) ON DELETE SET NULL,
    url TEXT,
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Menus
CREATE TABLE IF NOT EXISTS menus (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL UNIQUE
);

INSERT OR IGNORE INTO menus (id, name) VALUES ('main', 'main'), ('footer', 'footer');

-- Menu Items
CREATE TABLE IF NOT EXISTS menu_items (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    menu_id TEXT NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    link_type TEXT NOT NULL CHECK (link_type IN ('page', 'article', 'url', 'app_catalogue')),
    link_target TEXT NOT NULL,
    parent_id TEXT REFERENCES menu_items(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_menu_items_menu ON menu_items(menu_id);

-- Audit Log
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_audit_log_created ON audit_log(created_at);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);

-- Full-Text Search (FTS5)
CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(title, content, content=pages, content_rowid=rowid);
CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(title, short_text, content, content=articles, content_rowid=rowid);
CREATE VIRTUAL TABLE IF NOT EXISTS apps_fts USING fts5(name, description, content=apps, content_rowid=rowid);

-- FTS triggers for pages
CREATE TRIGGER pages_ai AFTER INSERT ON pages BEGIN
    INSERT INTO pages_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;
CREATE TRIGGER pages_ad AFTER DELETE ON pages BEGIN
    INSERT INTO pages_fts(pages_fts, rowid, title, content) VALUES ('delete', old.rowid, old.title, old.content);
END;
CREATE TRIGGER pages_au AFTER UPDATE ON pages BEGIN
    INSERT INTO pages_fts(pages_fts, rowid, title, content) VALUES ('delete', old.rowid, old.title, old.content);
    INSERT INTO pages_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;

-- FTS triggers for articles
CREATE TRIGGER articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, short_text, content) VALUES (new.rowid, new.title, new.short_text, new.content);
END;
CREATE TRIGGER articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, short_text, content) VALUES ('delete', old.rowid, old.title, old.short_text, old.content);
END;
CREATE TRIGGER articles_au AFTER UPDATE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, short_text, content) VALUES ('delete', old.rowid, old.title, old.short_text, old.content);
    INSERT INTO articles_fts(rowid, title, short_text, content) VALUES (new.rowid, new.title, new.short_text, new.content);
END;

-- FTS triggers for apps
CREATE TRIGGER apps_ai AFTER INSERT ON apps BEGIN
    INSERT INTO apps_fts(rowid, name, description) VALUES (new.rowid, new.name, new.description);
END;
CREATE TRIGGER apps_ad AFTER DELETE ON apps BEGIN
    INSERT INTO apps_fts(apps_fts, rowid, name, description) VALUES ('delete', old.rowid, old.name, old.description);
END;
CREATE TRIGGER apps_au AFTER UPDATE ON apps BEGIN
    INSERT INTO apps_fts(apps_fts, rowid, name, description) VALUES ('delete', old.rowid, old.name, old.description);
    INSERT INTO apps_fts(rowid, name, description) VALUES (new.rowid, new.name, new.description);
END;
```

**Step 2: Create db module**

Create `backend/src/db/mod.rs`:

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // Enable WAL mode for better concurrent read performance
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}
```

**Step 3: Update main.rs to use database**

```rust
mod config;
mod db;

use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let config = config::Config::from_env().expect("Failed to load config");
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await;
    let db_ok = result.is_ok();
    Json(json!({ "status": if db_ok { "ok" } else { "degraded" }, "db": db_ok }))
}
```

**Step 4: Create .env for development**

Create `backend/.env`:
```
DATABASE_URL=sqlite:data/pawtal.db?mode=rwc
UPLOADS_DIR=uploads
OAUTH2_CLIENT_ID=dev
OAUTH2_CLIENT_SECRET=dev
OAUTH2_ISSUER_URL=http://localhost:9000
SESSION_SECRET=dev-secret-change-in-production
BASE_URL=http://localhost:8080
```

**Step 5: Verify it compiles and migrations run**

Run: `cd backend && cargo check`
Expected: Compiles without errors.

**Step 6: Commit**

```bash
git add backend/
git commit -m "feat: add SQLite database setup with full schema migration"
```

---

### Task 3: Backend Data Models

**Files:**
- Create: `backend/src/db/models.rs`

**Step 1: Define all data models**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Users ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

// --- Pages ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub publish_at: Option<String>,
    pub author_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub trashed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePage {
    pub title: String,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<String>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePage {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<String>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PageRevision {
    pub id: String,
    pub page_id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: String,
}

// --- Articles ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub short_text: String,
    pub content: String,
    pub status: String,
    pub publish_at: Option<String>,
    pub author_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub trashed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateArticle {
    pub title: String,
    pub slug: Option<String>,
    pub short_text: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<String>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub short_text: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<String>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ArticleRevision {
    pub id: String,
    pub article_id: String,
    pub title: String,
    pub short_text: String,
    pub content: String,
    pub author_id: String,
    pub created_at: String,
}

// --- Categories ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub slug: Option<String>,
}

// --- Media ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Media {
    pub id: String,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: String,
    pub is_icon: bool,
    pub uploaded_by: String,
    pub created_at: String,
}

// --- Apps ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct App {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_id: Option<String>,
    pub url: Option<String>,
    pub page_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApp {
    pub name: String,
    pub description: Option<String>,
    pub icon_id: Option<String>,
    pub url: Option<String>,
    pub page_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApp {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_id: Option<String>,
    pub url: Option<String>,
    pub page_id: Option<String>,
}

// --- Menus ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Menu {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuItem {
    pub id: String,
    pub menu_id: String,
    pub label: String,
    pub link_type: String,
    pub link_target: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMenu {
    pub items: Vec<MenuItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct MenuItemInput {
    pub id: Option<String>,
    pub label: String,
    pub link_type: String,
    pub link_target: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

// --- Site Settings ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SiteSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

// --- Audit Log ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: String,
    pub created_at: String,
}

// --- Search ---

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub result_type: String,
    pub id: String,
    pub title: String,
    pub slug: String,
    pub snippet: String,
}

// --- Pagination ---

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page();
        (page - 1) * per_page
    }

    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).min(100)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
```

**Step 2: Add models to db module**

Update `backend/src/db/mod.rs` to add: `pub mod models;`

**Step 3: Verify it compiles**

Run: `cd backend && cargo check`
Expected: Compiles without errors.

**Step 4: Commit**

```bash
git add backend/src/db/models.rs backend/src/db/mod.rs
git commit -m "feat: add all data models for Pawtal CMS"
```

---

### Task 4: Shared Error Handling and Helpers

**Files:**
- Create: `backend/src/error.rs`
- Create: `backend/src/helpers.rs`

**Step 1: Create error types**

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

**Step 2: Create helpers (slug generation)**

```rust
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Special!@#Characters"), "special-characters");
        assert_eq!(slugify("Already-slugified"), "already-slugified");
    }
}
```

**Step 3: Run tests**

Run: `cd backend && cargo test`
Expected: All tests pass.

**Step 4: Add modules to main.rs**

Add to main.rs: `mod error;` and `mod helpers;`

**Step 5: Commit**

```bash
git add backend/src/error.rs backend/src/helpers.rs backend/src/main.rs
git commit -m "feat: add error handling and slug helper"
```

---

## Phase 2: Authentication

### Task 5: OAuth2 Login and Session Management

**Files:**
- Create: `backend/src/auth/mod.rs`
- Create: `backend/src/auth/oauth2.rs`
- Create: `backend/src/auth/session.rs`
- Create: `backend/src/auth/middleware.rs`
- Create: `backend/src/api/mod.rs`
- Create: `backend/src/api/auth.rs`
- Modify: `backend/src/main.rs`

**Step 1: Create OAuth2 client**

`backend/src/auth/oauth2.rs`:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct OidcDiscovery {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub name: Option<String>,
}

pub async fn discover_oidc(issuer_url: &str) -> AppResult<OidcDiscovery> {
    let url = format!("{}/.well-known/openid-configuration", issuer_url.trim_end_matches('/'));
    let client = Client::new();
    client.get(&url)
        .send().await
        .map_err(|e| AppError::Internal(format!("OIDC discovery failed: {}", e)))?
        .json::<OidcDiscovery>().await
        .map_err(|e| AppError::Internal(format!("OIDC discovery parse failed: {}", e)))
}

pub fn build_auth_url(discovery: &OidcDiscovery, config: &Config, state: &str) -> String {
    let redirect_uri = format!("{}/api/auth/callback", config.base_url);
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope=openid+email+profile&state={}",
        discovery.authorization_endpoint,
        urlencoding::encode(&config.oauth2_client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(state),
    )
}

pub async fn exchange_code(
    discovery: &OidcDiscovery,
    config: &Config,
    code: &str,
) -> AppResult<TokenResponse> {
    let redirect_uri = format!("{}/api/auth/callback", config.base_url);
    let client = Client::new();
    client.post(&discovery.token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", &config.oauth2_client_id),
            ("client_secret", &config.oauth2_client_secret),
            ("code", code),
            ("redirect_uri", &redirect_uri),
        ])
        .send().await
        .map_err(|e| AppError::Internal(format!("Token exchange failed: {}", e)))?
        .json::<TokenResponse>().await
        .map_err(|e| AppError::Internal(format!("Token parse failed: {}", e)))
}

pub async fn fetch_userinfo(
    discovery: &OidcDiscovery,
    access_token: &str,
) -> AppResult<UserInfo> {
    let client = Client::new();
    client.get(&discovery.userinfo_endpoint)
        .bearer_auth(access_token)
        .send().await
        .map_err(|e| AppError::Internal(format!("Userinfo fetch failed: {}", e)))?
        .json::<UserInfo>().await
        .map_err(|e| AppError::Internal(format!("Userinfo parse failed: {}", e)))
}
```

Add `urlencoding = "2"` to Cargo.toml.

**Step 2: Create session management**

`backend/src/auth/session.rs`:

```rust
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::db::models::User;
use crate::error::{AppError, AppResult};

pub async fn create_session(pool: &SqlitePool, user_id: &str) -> AppResult<String> {
    let token = Uuid::new_v4().to_string();
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, token, expires_at) VALUES (?, ?, ?, datetime('now', '+7 days'))"
    )
    .bind(&id)
    .bind(user_id)
    .bind(&token)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn validate_session(pool: &SqlitePool, token: &str) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u
         JOIN sessions s ON s.user_id = u.id
         WHERE s.token = ? AND s.expires_at > datetime('now')"
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(user)
}

pub async fn delete_session(pool: &SqlitePool, token: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> AppResult<u64> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < datetime('now')")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
```

**Step 3: Create auth middleware**

`backend/src/auth/middleware.rs`:

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::AppState;
use crate::db::models::User;
use crate::error::AppError;

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req.headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .find_map(|c| {
                    let c = c.trim();
                    c.strip_prefix("pawtal_session=")
                })
        })
        .ok_or(AppError::Unauthorized)?;

    let user = super::session::validate_session(&state.db, token).await?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub async fn require_admin(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req.headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .find_map(|c| {
                    let c = c.trim();
                    c.strip_prefix("pawtal_session=")
                })
        })
        .ok_or(AppError::Unauthorized)?;

    let user = super::session::validate_session(&state.db, token).await?;
    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
```

**Step 4: Create auth module root**

`backend/src/auth/mod.rs`:

```rust
pub mod middleware;
pub mod oauth2;
pub mod session;
```

**Step 5: Create auth API handlers**

`backend/src/api/auth.rs`:

```rust
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::AppState;
use crate::auth::{oauth2, session};
use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn login(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let discovery = oauth2::discover_oidc(&state.config.oauth2_issuer_url).await?;
    let csrf_state = Uuid::new_v4().to_string();
    let auth_url = oauth2::build_auth_url(&discovery, &state.config, &csrf_state);
    Ok(Redirect::temporary(&auth_url))
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> AppResult<impl IntoResponse> {
    let discovery = oauth2::discover_oidc(&state.config.oauth2_issuer_url).await?;
    let tokens = oauth2::exchange_code(&discovery, &state.config, &params.code).await?;
    let userinfo = oauth2::fetch_userinfo(&discovery, &tokens.access_token).await?;

    // Upsert user
    let user_id = sqlx::query_scalar::<_, String>(
        "INSERT INTO users (id, external_id, email, display_name, last_login)
         VALUES (?, ?, ?, ?, datetime('now'))
         ON CONFLICT(external_id) DO UPDATE SET
             email = excluded.email,
             display_name = excluded.display_name,
             last_login = datetime('now')
         RETURNING id"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&userinfo.sub)
    .bind(userinfo.email.as_deref().unwrap_or(""))
    .bind(userinfo.name.as_deref().unwrap_or(&userinfo.sub))
    .fetch_one(&state.db)
    .await?;

    let token = session::create_session(&state.db, &user_id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "set-cookie",
        format!(
            "pawtal_session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800",
            token
        ).parse().unwrap(),
    );

    Ok((headers, Redirect::temporary("/admin")))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<impl IntoResponse> {
    if let Some(cookie) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        if let Some(token) = cookie.split(';').find_map(|c| c.trim().strip_prefix("pawtal_session=")) {
            session::delete_session(&state.db, token).await?;
        }
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        "set-cookie",
        "pawtal_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );

    Ok((resp_headers, Redirect::temporary("/")))
}

pub async fn me(Extension(user): Extension<crate::db::models::User>) -> Json<crate::db::models::User> {
    Json(user)
}
```

**Step 6: Create api module root**

`backend/src/api/mod.rs`:

```rust
pub mod auth;
```

**Step 7: Wire up routes in main.rs**

Update main.rs to include auth routes:

```rust
mod config;
mod db;
mod error;
mod helpers;
mod auth;
mod api;

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let config = config::Config::from_env().expect("Failed to load config");

    // Create uploads directory
    std::fs::create_dir_all(&config.uploads_dir).expect("Failed to create uploads dir");

    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    // Public routes
    let public_routes = Router::new()
        .route("/api/health", get(health_check));

    // Auth routes
    let auth_routes = Router::new()
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/callback", get(api::auth::callback))
        .route("/api/auth/logout", post(api::auth::logout));

    // Admin routes (require authentication)
    let admin_routes = Router::new()
        .route("/api/admin/me", get(api::auth::me))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(admin_routes)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<serde_json::Value> {
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await;
    let db_ok = result.is_ok();
    Json(json!({ "status": if db_ok { "ok" } else { "degraded" }, "db": db_ok }))
}
```

**Step 8: Verify it compiles**

Run: `cd backend && cargo check`
Expected: Compiles without errors.

**Step 9: Commit**

```bash
git add backend/
git commit -m "feat: add OAuth2 authentication and session management"
```

---

## Phase 3: Core CRUD Services and API

### Task 6: Audit Log Service

**Files:**
- Create: `backend/src/services/mod.rs`
- Create: `backend/src/services/audit.rs`

**Step 1: Create audit log service**

This is used by all other services, so build it first.

`backend/src/services/audit.rs`:

```rust
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::error::AppResult;

pub async fn log_action(
    pool: &SqlitePool,
    user_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: &serde_json::Value,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO audit_log (id, user_id, action, entity_type, entity_id, details)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details.to_string())
    .execute(pool)
    .await?;
    Ok(())
}
```

`backend/src/services/mod.rs`:

```rust
pub mod audit;
```

Add `mod services;` to main.rs.

**Step 2: Commit**

```bash
git add backend/src/services/
git commit -m "feat: add audit log service"
```

---

### Task 7: Pages Service and Admin API

**Files:**
- Create: `backend/src/services/pages.rs`
- Create: `backend/src/api/pages.rs`
- Modify: `backend/src/services/mod.rs`
- Modify: `backend/src/api/mod.rs`
- Modify: `backend/src/main.rs`

**Step 1: Create pages service**

`backend/src/services/pages.rs`:

```rust
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::db::models::*;
use crate::error::{AppError, AppResult};
use crate::helpers::slugify;
use crate::services::audit;

pub async fn list_pages(pool: &SqlitePool, params: &PaginationParams, status_filter: Option<&str>) -> AppResult<PaginatedResponse<Page>> {
    let per_page = params.per_page();
    let offset = params.offset();

    let (pages, total) = if let Some(status) = status_filter {
        let pages = sqlx::query_as::<_, Page>(
            "SELECT * FROM pages WHERE status = ? ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool).await?;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pages WHERE status = ?")
            .bind(status)
            .fetch_one(pool).await?;

        (pages, total)
    } else {
        let pages = sqlx::query_as::<_, Page>(
            "SELECT * FROM pages WHERE status != 'trashed' ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool).await?;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pages WHERE status != 'trashed'")
            .fetch_one(pool).await?;

        (pages, total)
    };

    Ok(PaginatedResponse {
        data: pages,
        total,
        page: params.page.unwrap_or(1).max(1),
        per_page,
    })
}

pub async fn get_page(pool: &SqlitePool, id: &str) -> AppResult<Page> {
    sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE id = ?")
        .bind(id)
        .fetch_optional(pool).await?
        .ok_or(AppError::NotFound)
}

pub async fn get_page_by_slug(pool: &SqlitePool, slug: &str) -> AppResult<Page> {
    sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE slug = ? AND status = 'published'")
        .bind(slug)
        .fetch_optional(pool).await?
        .ok_or(AppError::NotFound)
}

pub async fn create_page(pool: &SqlitePool, input: CreatePage, author_id: &str) -> AppResult<Page> {
    let id = Uuid::new_v4().to_string();
    let slug = input.slug.unwrap_or_else(|| slugify(&input.title));
    let content = input.content.unwrap_or_default();
    let status = input.status.unwrap_or_else(|| "draft".into());

    // Check slug uniqueness
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pages WHERE slug = ?")
        .bind(&slug)
        .fetch_one(pool).await?;
    if exists > 0 {
        return Err(AppError::Conflict(format!("Slug '{}' already exists", slug)));
    }

    sqlx::query(
        "INSERT INTO pages (id, title, slug, content, status, publish_at, author_id)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&slug)
    .bind(&content)
    .bind(&status)
    .bind(&input.publish_at)
    .bind(author_id)
    .execute(pool).await?;

    // Set categories if provided
    if let Some(cat_ids) = &input.category_ids {
        set_page_categories(pool, &id, cat_ids).await?;
    }

    // Create initial revision
    create_revision(pool, &id, &input.title, &content, author_id).await?;

    // Audit
    audit::log_action(pool, author_id, "create", "page", &id, &serde_json::json!({"title": input.title})).await?;

    get_page(pool, &id).await
}

pub async fn update_page(pool: &SqlitePool, id: &str, input: UpdatePage, user_id: &str) -> AppResult<Page> {
    let existing = get_page(pool, id).await?;

    let title = input.title.unwrap_or(existing.title);
    let slug = input.slug.unwrap_or(existing.slug);
    let content = input.content.unwrap_or(existing.content);
    let status = input.status.unwrap_or(existing.status);
    let publish_at = input.publish_at.or(existing.publish_at);

    // Check slug uniqueness if changed
    let slug_conflict = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pages WHERE slug = ? AND id != ?"
    )
    .bind(&slug)
    .bind(id)
    .fetch_one(pool).await?;
    if slug_conflict > 0 {
        return Err(AppError::Conflict(format!("Slug '{}' already exists", slug)));
    }

    sqlx::query(
        "UPDATE pages SET title = ?, slug = ?, content = ?, status = ?, publish_at = ?, updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(&title)
    .bind(&slug)
    .bind(&content)
    .bind(&status)
    .bind(&publish_at)
    .bind(id)
    .execute(pool).await?;

    if let Some(cat_ids) = &input.category_ids {
        set_page_categories(pool, id, cat_ids).await?;
    }

    // Create revision
    create_revision(pool, id, &title, &content, user_id).await?;

    audit::log_action(pool, user_id, "update", "page", id, &serde_json::json!({"title": title})).await?;

    get_page(pool, id).await
}

pub async fn trash_page(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE pages SET status = 'trashed', trashed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?"
    )
    .bind(id)
    .execute(pool).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    audit::log_action(pool, user_id, "trash", "page", id, &serde_json::json!({})).await?;
    Ok(())
}

pub async fn restore_page(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<Page> {
    sqlx::query(
        "UPDATE pages SET status = 'draft', trashed_at = NULL, updated_at = datetime('now') WHERE id = ? AND status = 'trashed'"
    )
    .bind(id)
    .execute(pool).await?;

    audit::log_action(pool, user_id, "restore", "page", id, &serde_json::json!({})).await?;
    get_page(pool, id).await
}

pub async fn publish_page(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<Page> {
    sqlx::query(
        "UPDATE pages SET status = 'published', updated_at = datetime('now') WHERE id = ?"
    )
    .bind(id)
    .execute(pool).await?;

    audit::log_action(pool, user_id, "publish", "page", id, &serde_json::json!({})).await?;
    get_page(pool, id).await
}

// --- Revisions ---

async fn create_revision(pool: &SqlitePool, page_id: &str, title: &str, content: &str, author_id: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO page_revisions (id, page_id, title, content, author_id)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(page_id)
    .bind(title)
    .bind(content)
    .bind(author_id)
    .execute(pool).await?;
    Ok(())
}

pub async fn list_revisions(pool: &SqlitePool, page_id: &str) -> AppResult<Vec<PageRevision>> {
    let revisions = sqlx::query_as::<_, PageRevision>(
        "SELECT * FROM page_revisions WHERE page_id = ? ORDER BY created_at DESC"
    )
    .bind(page_id)
    .fetch_all(pool).await?;
    Ok(revisions)
}

pub async fn restore_revision(pool: &SqlitePool, page_id: &str, revision_id: &str, user_id: &str) -> AppResult<Page> {
    let revision = sqlx::query_as::<_, PageRevision>(
        "SELECT * FROM page_revisions WHERE id = ? AND page_id = ?"
    )
    .bind(revision_id)
    .bind(page_id)
    .fetch_optional(pool).await?
    .ok_or(AppError::NotFound)?;

    let input = UpdatePage {
        title: Some(revision.title),
        slug: None,
        content: Some(revision.content),
        status: None,
        publish_at: None,
        category_ids: None,
    };

    update_page(pool, page_id, input, user_id).await
}

// --- Categories ---

async fn set_page_categories(pool: &SqlitePool, page_id: &str, category_ids: &[String]) -> AppResult<()> {
    sqlx::query("DELETE FROM page_categories WHERE page_id = ?")
        .bind(page_id)
        .execute(pool).await?;

    for cat_id in category_ids {
        sqlx::query("INSERT INTO page_categories (page_id, category_id) VALUES (?, ?)")
            .bind(page_id)
            .bind(cat_id)
            .execute(pool).await?;
    }
    Ok(())
}

pub async fn get_page_categories(pool: &SqlitePool, page_id: &str) -> AppResult<Vec<Category>> {
    let cats = sqlx::query_as::<_, Category>(
        "SELECT c.* FROM categories c
         JOIN page_categories pc ON pc.category_id = c.id
         WHERE pc.page_id = ?"
    )
    .bind(page_id)
    .fetch_all(pool).await?;
    Ok(cats)
}
```

**Step 2: Create pages API handlers**

`backend/src/api/pages.rs`:

```rust
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use crate::AppState;
use crate::db::models::*;
use crate::error::AppResult;
use crate::services::pages as page_svc;

// --- Admin endpoints ---

pub async fn admin_list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
    Query(filter): Query<StatusFilter>,
) -> AppResult<Json<PaginatedResponse<Page>>> {
    let result = page_svc::list_pages(&state.db, &params, filter.status.as_deref()).await?;
    Ok(Json(result))
}

#[derive(Debug, serde::Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

pub async fn admin_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Page>> {
    let page = page_svc::get_page(&state.db, &id).await?;
    Ok(Json(page))
}

pub async fn admin_create(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(input): Json<CreatePage>,
) -> AppResult<Json<Page>> {
    let page = page_svc::create_page(&state.db, input, &user.id).await?;
    Ok(Json(page))
}

pub async fn admin_update(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePage>,
) -> AppResult<Json<Page>> {
    let page = page_svc::update_page(&state.db, &id, input, &user.id).await?;
    Ok(Json(page))
}

pub async fn admin_delete(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    page_svc::trash_page(&state.db, &id, &user.id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn admin_publish(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<Page>> {
    let page = page_svc::publish_page(&state.db, &id, &user.id).await?;
    Ok(Json(page))
}

pub async fn admin_restore(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<Page>> {
    let page = page_svc::restore_page(&state.db, &id, &user.id).await?;
    Ok(Json(page))
}

pub async fn admin_revisions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<PageRevision>>> {
    let revisions = page_svc::list_revisions(&state.db, &id).await?;
    Ok(Json(revisions))
}

pub async fn admin_restore_revision(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((id, rev_id)): Path<(String, String)>,
) -> AppResult<Json<Page>> {
    let page = page_svc::restore_revision(&state.db, &id, &rev_id, &user.id).await?;
    Ok(Json(page))
}

// --- Public endpoints ---

pub async fn public_get_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Page>> {
    let page = page_svc::get_page_by_slug(&state.db, &slug).await?;
    Ok(Json(page))
}
```

**Step 3: Update modules and routes**

Add `pub mod pages;` to `backend/src/services/mod.rs` and `backend/src/api/mod.rs`.

Update main.rs admin_routes and public_routes to include page routes:

```rust
// In main.rs, update the route setup:

use axum::routing::{delete, get, post, put};

// Public routes
let public_routes = Router::new()
    .route("/api/health", get(health_check))
    .route("/api/pages/{slug}", get(api::pages::public_get_by_slug));

// Admin routes
let admin_routes = Router::new()
    .route("/api/admin/me", get(api::auth::me))
    .route("/api/admin/pages", get(api::pages::admin_list).post(api::pages::admin_create))
    .route("/api/admin/pages/{id}", get(api::pages::admin_get).put(api::pages::admin_update).delete(api::pages::admin_delete))
    .route("/api/admin/pages/{id}/publish", post(api::pages::admin_publish))
    .route("/api/admin/pages/{id}/restore", post(api::pages::admin_restore))
    .route("/api/admin/pages/{id}/revisions", get(api::pages::admin_revisions))
    .route("/api/admin/pages/{id}/revisions/{rev_id}/restore", post(api::pages::admin_restore_revision))
    .layer(middleware::from_fn_with_state(state.clone(), auth::middleware::require_auth));
```

**Step 4: Verify it compiles**

Run: `cd backend && cargo check`
Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add backend/
git commit -m "feat: add pages CRUD service and API endpoints"
```

---

### Task 8: Articles Service and API

**Files:**
- Create: `backend/src/services/articles.rs`
- Create: `backend/src/api/articles.rs`

Mirrors the pages service/API pattern exactly, with these differences:
- Article model has `short_text` field
- Article revisions include `short_text`
- Public list endpoint returns paginated published articles ordered by `created_at DESC`

**Step 1: Create articles service**

Follow the exact same pattern as `backend/src/services/pages.rs` but for articles. Key differences:
- Uses `articles` table instead of `pages`
- `CreateArticle` / `UpdateArticle` have `short_text` field
- `article_revisions` includes `short_text`
- `article_categories` join table
- Public list: `SELECT * FROM articles WHERE status = 'published' ORDER BY created_at DESC LIMIT ? OFFSET ?`

**Step 2: Create articles API handlers**

Same pattern as `backend/src/api/pages.rs` plus:
- `public_list`: paginated published articles
- `public_get_by_slug`: single published article

**Step 3: Wire up routes in main.rs**

Public:
```rust
.route("/api/articles", get(api::articles::public_list))
.route("/api/articles/{slug}", get(api::articles::public_get_by_slug))
```

Admin:
```rust
.route("/api/admin/articles", get(api::articles::admin_list).post(api::articles::admin_create))
.route("/api/admin/articles/{id}", get(api::articles::admin_get).put(api::articles::admin_update).delete(api::articles::admin_delete))
.route("/api/admin/articles/{id}/publish", post(api::articles::admin_publish))
.route("/api/admin/articles/{id}/restore", post(api::articles::admin_restore))
.route("/api/admin/articles/{id}/revisions", get(api::articles::admin_revisions))
.route("/api/admin/articles/{id}/revisions/{rev_id}/restore", post(api::articles::admin_restore_revision))
```

**Step 4: Verify and commit**

```bash
cd backend && cargo check
git add backend/ && git commit -m "feat: add articles CRUD service and API endpoints"
```

---

### Task 9: Categories, Settings, Menus, Apps, Trash, and Audit Log APIs

**Files:**
- Create: `backend/src/services/categories.rs`
- Create: `backend/src/services/settings.rs`
- Create: `backend/src/services/menus.rs`
- Create: `backend/src/services/apps.rs`
- Create: `backend/src/api/categories.rs`
- Create: `backend/src/api/settings.rs`
- Create: `backend/src/api/menus.rs`
- Create: `backend/src/api/apps.rs`
- Create: `backend/src/api/trash.rs`
- Create: `backend/src/api/audit.rs`

These are all simpler CRUD services following the same patterns established by pages/articles. Build them one at a time.

**Step 1: Categories service + API**

Service: CRUD on `categories` table. `create_category` auto-generates slug from name if not provided.

API:
- `GET /api/admin/categories` — list all
- `POST /api/admin/categories` — create
- `PUT /api/admin/categories/{id}` — update
- `DELETE /api/admin/categories/{id}` — delete

**Step 2: Settings service + API**

Service:
- `get_all_settings(pool) -> HashMap<String, String>` — SELECT all from site_settings
- `get_public_settings(pool)` — returns only public-safe keys
- `update_settings(pool, updates: HashMap<String, String>, user_id)` — upsert each key

API:
- `GET /api/settings/public` — public
- `GET /api/admin/settings` — admin (all settings)
- `PUT /api/admin/settings` — admin (update, requires admin role)

**Step 3: Menus service + API**

Service:
- `get_menu(pool, name) -> Menu + Vec<MenuItem>` — returns menu with nested items
- `update_menu(pool, name, items: Vec<MenuItemInput>, user_id)` — replace all items for menu

API:
- `GET /api/menus/{name}` — public
- `GET /api/admin/menus/{name}` — admin
- `PUT /api/admin/menus/{name}` — admin (update)

**Step 4: Apps service + API**

Service:
- CRUD on `apps` table
- `reorder_apps(pool, ids: Vec<String>)` — update sort_order
- Public list respects `apps_per_page` setting

API:
- `GET /api/apps` — public (paginated)
- `GET /api/admin/apps` — admin list
- `POST /api/admin/apps` — create
- `GET/PUT/DELETE /api/admin/apps/{id}` — CRUD
- `PUT /api/admin/apps/reorder` — reorder

**Step 5: Trash API**

`backend/src/api/trash.rs`:
- `GET /api/admin/trash` — list all trashed pages and articles
- `POST /api/admin/trash/empty` — permanently delete all items trashed > 30 days ago

**Step 6: Audit log API**

`backend/src/api/audit.rs`:
- `GET /api/admin/audit-log?page=&per_page=` — paginated, admin only

**Step 7: Users API (admin only)**

Add to `backend/src/api/auth.rs`:
- `GET /api/admin/users` — list all users
- `PUT /api/admin/users/{id}/role` — change user role

**Step 8: Wire all routes, verify compilation, commit**

```bash
cd backend && cargo check
git add backend/ && git commit -m "feat: add categories, settings, menus, apps, trash, and audit log APIs"
```

---

### Task 10: Media Upload and Image Processing

**Files:**
- Create: `backend/src/services/media.rs`
- Create: `backend/src/media/mod.rs`
- Create: `backend/src/media/processing.rs`
- Create: `backend/src/api/media.rs`

**Step 1: Create image processing module**

`backend/src/media/processing.rs`:

```rust
use image::{DynamicImage, ImageFormat};
use std::path::Path;
use crate::error::{AppError, AppResult};

pub struct ImageVariant {
    pub suffix: String,
    pub max_width: u32,
    pub max_height: u32,
    pub crop_square: bool,
}

pub fn get_standard_variants() -> Vec<ImageVariant> {
    vec![
        ImageVariant { suffix: "thumbnail".into(), max_width: 200, max_height: 200, crop_square: true },
        ImageVariant { suffix: "medium".into(), max_width: 800, max_height: 800, crop_square: false },
        ImageVariant { suffix: "large".into(), max_width: 1600, max_height: 1600, crop_square: false },
    ]
}

pub fn get_icon_variants() -> Vec<ImageVariant> {
    let mut variants = get_standard_variants();
    variants.push(ImageVariant { suffix: "icon".into(), max_width: 128, max_height: 128, crop_square: true });
    variants
}

pub fn process_image(
    input_path: &Path,
    output_dir: &Path,
    variants: &[ImageVariant],
) -> AppResult<(u32, u32)> {
    let img = image::open(input_path)
        .map_err(|e| AppError::Internal(format!("Failed to open image: {}", e)))?;

    let (orig_w, orig_h) = (img.width(), img.height());

    for variant in variants {
        let resized = if variant.crop_square {
            let size = img.width().min(img.height());
            let cropped = img.crop_imm(
                (img.width() - size) / 2,
                (img.height() - size) / 2,
                size,
                size,
            );
            cropped.resize(variant.max_width, variant.max_height, image::imageops::FilterType::Lanczos3)
        } else {
            img.resize(variant.max_width, variant.max_height, image::imageops::FilterType::Lanczos3)
        };

        // Save as original format
        let ext = input_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let output_path = output_dir.join(format!("{}.{}", variant.suffix, ext));
        resized.save(&output_path)
            .map_err(|e| AppError::Internal(format!("Failed to save variant: {}", e)))?;

        // Save as WebP
        let webp_path = output_dir.join(format!("{}.webp", variant.suffix));
        let rgba = resized.to_rgba8();
        let encoder = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
        let webp_data = encoder.encode(80.0);
        std::fs::write(&webp_path, &*webp_data)
            .map_err(|e| AppError::Internal(format!("Failed to save WebP: {}", e)))?;
    }

    Ok((orig_w, orig_h))
}
```

`backend/src/media/mod.rs`:
```rust
pub mod processing;
```

**Step 2: Create media service**

`backend/src/services/media.rs`:

```rust
use sqlx::SqlitePool;
use uuid::Uuid;
use std::path::PathBuf;
use crate::db::models::*;
use crate::error::{AppError, AppResult};
use crate::media::processing;
use crate::services::audit;

pub async fn list_media(
    pool: &SqlitePool,
    params: &PaginationParams,
    filter: Option<&str>, // "images", "icons", or None for all
) -> AppResult<PaginatedResponse<Media>> {
    let per_page = params.per_page();
    let offset = params.offset();

    let (items, total) = match filter {
        Some("icons") => {
            let items = sqlx::query_as::<_, Media>(
                "SELECT * FROM media WHERE is_icon = 1 ORDER BY created_at DESC LIMIT ? OFFSET ?"
            ).bind(per_page).bind(offset).fetch_all(pool).await?;
            let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media WHERE is_icon = 1")
                .fetch_one(pool).await?;
            (items, total)
        }
        Some("images") => {
            let items = sqlx::query_as::<_, Media>(
                "SELECT * FROM media WHERE is_icon = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?"
            ).bind(per_page).bind(offset).fetch_all(pool).await?;
            let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media WHERE is_icon = 0")
                .fetch_one(pool).await?;
            (items, total)
        }
        _ => {
            let items = sqlx::query_as::<_, Media>(
                "SELECT * FROM media ORDER BY created_at DESC LIMIT ? OFFSET ?"
            ).bind(per_page).bind(offset).fetch_all(pool).await?;
            let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media")
                .fetch_one(pool).await?;
            (items, total)
        }
    };

    Ok(PaginatedResponse { data: items, total, page: params.page.unwrap_or(1).max(1), per_page })
}

pub async fn upload_media(
    pool: &SqlitePool,
    uploads_dir: &str,
    original_filename: &str,
    mime_type: &str,
    data: &[u8],
    is_icon: bool,
    user_id: &str,
) -> AppResult<Media> {
    let id = Uuid::new_v4().to_string();
    let ext = original_filename.rsplit('.').next().unwrap_or("bin");
    let filename = format!("{}.{}", id, ext);

    // Create upload directory
    let upload_dir = PathBuf::from(uploads_dir).join(&id);
    std::fs::create_dir_all(&upload_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;

    // Save original
    let original_path = upload_dir.join(format!("original.{}", ext));
    std::fs::write(&original_path, data)
        .map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;

    let size_bytes = data.len() as i64;

    // Process image variants (async)
    let (width, height) = if mime_type.starts_with("image/") {
        let variants = if is_icon {
            processing::get_icon_variants()
        } else {
            processing::get_standard_variants()
        };
        let (w, h) = processing::process_image(&original_path, &upload_dir, &variants)?;
        (Some(w as i32), Some(h as i32))
    } else {
        (None, None)
    };

    sqlx::query(
        "INSERT INTO media (id, filename, original_filename, mime_type, size_bytes, width, height, is_icon, uploaded_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&filename)
    .bind(original_filename)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(width)
    .bind(height)
    .bind(is_icon)
    .bind(user_id)
    .execute(pool).await?;

    audit::log_action(pool, user_id, "upload", "media", &id, &serde_json::json!({"filename": original_filename})).await?;

    sqlx::query_as::<_, Media>("SELECT * FROM media WHERE id = ?")
        .bind(&id)
        .fetch_one(pool).await
        .map_err(Into::into)
}

pub async fn delete_media(pool: &SqlitePool, uploads_dir: &str, id: &str, user_id: &str) -> AppResult<()> {
    let media = sqlx::query_as::<_, Media>("SELECT * FROM media WHERE id = ?")
        .bind(id)
        .fetch_optional(pool).await?
        .ok_or(AppError::NotFound)?;

    // Delete files
    let upload_dir = PathBuf::from(uploads_dir).join(id);
    if upload_dir.exists() {
        std::fs::remove_dir_all(&upload_dir)
            .map_err(|e| AppError::Internal(format!("Failed to delete files: {}", e)))?;
    }

    sqlx::query("DELETE FROM media WHERE id = ?")
        .bind(id)
        .execute(pool).await?;

    audit::log_action(pool, user_id, "delete", "media", id, &serde_json::json!({"filename": media.original_filename})).await?;
    Ok(())
}
```

**Step 3: Create media API handlers**

`backend/src/api/media.rs` — handles multipart upload, list, delete, and serves media files.

Add a static file serving route for `/uploads/**` using `tower_http::services::ServeDir`.

**Step 4: Wire routes, verify, commit**

```bash
cd backend && cargo check
git add backend/ && git commit -m "feat: add media upload with image processing pipeline"
```

---

### Task 11: Search API

**Files:**
- Create: `backend/src/services/search.rs`
- Create: `backend/src/api/search.rs`

**Step 1: Create search service**

```rust
use sqlx::SqlitePool;
use crate::db::models::SearchResult;
use crate::error::AppResult;

pub async fn search(
    pool: &SqlitePool,
    query: &str,
    search_type: Option<&str>,
    include_unpublished: bool,
) -> AppResult<Vec<SearchResult>> {
    let mut results = Vec::new();
    let fts_query = format!("{}*", query); // Prefix matching

    let search_pages = search_type.is_none() || search_type == Some("pages");
    let search_articles = search_type.is_none() || search_type == Some("articles");
    let search_apps = search_type.is_none() || search_type == Some("apps");

    if search_pages {
        let status_filter = if include_unpublished { "" } else { "AND p.status = 'published'" };
        let sql = format!(
            "SELECT p.id, p.title, p.slug, snippet(pages_fts, 1, '<mark>', '</mark>', '...', 32) as snippet
             FROM pages_fts
             JOIN pages p ON p.rowid = pages_fts.rowid
             WHERE pages_fts MATCH ?1 {status_filter}
             ORDER BY bm25(pages_fts)
             LIMIT 20"
        );
        let rows = sqlx::query_as::<_, (String, String, String, String)>(&sql)
            .bind(&fts_query)
            .fetch_all(pool).await?;

        for (id, title, slug, snippet) in rows {
            results.push(SearchResult {
                result_type: "page".into(),
                id, title, slug, snippet,
            });
        }
    }

    if search_articles {
        let status_filter = if include_unpublished { "" } else { "AND a.status = 'published'" };
        let sql = format!(
            "SELECT a.id, a.title, a.slug, snippet(articles_fts, 2, '<mark>', '</mark>', '...', 32) as snippet
             FROM articles_fts
             JOIN articles a ON a.rowid = articles_fts.rowid
             WHERE articles_fts MATCH ?1 {status_filter}
             ORDER BY bm25(articles_fts)
             LIMIT 20"
        );
        let rows = sqlx::query_as::<_, (String, String, String, String)>(&sql)
            .bind(&fts_query)
            .fetch_all(pool).await?;

        for (id, title, slug, snippet) in rows {
            results.push(SearchResult {
                result_type: "article".into(),
                id, title, slug, snippet,
            });
        }
    }

    if search_apps {
        let sql =
            "SELECT a.id, a.name, '' as slug, snippet(apps_fts, 1, '<mark>', '</mark>', '...', 32) as snippet
             FROM apps_fts
             JOIN apps a ON a.rowid = apps_fts.rowid
             WHERE apps_fts MATCH ?1
             ORDER BY bm25(apps_fts)
             LIMIT 20";
        let rows = sqlx::query_as::<_, (String, String, String, String)>(sql)
            .bind(&fts_query)
            .fetch_all(pool).await?;

        for (id, name, _, snippet) in rows {
            results.push(SearchResult {
                result_type: "app".into(),
                id,
                title: name,
                slug: String::new(),
                snippet,
            });
        }
    }

    Ok(results)
}
```

**Step 2: Create search API handler**

```rust
#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(rename = "type")]
    pub search_type: Option<String>,
}

pub async fn public_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<Vec<SearchResult>>> {
    let results = search_svc::search(&state.db, &params.q, params.search_type.as_deref(), false).await?;
    Ok(Json(results))
}
```

**Step 3: Wire route, verify, commit**

```bash
cd backend && cargo check
git add backend/ && git commit -m "feat: add full-text search API with SQLite FTS5"
```

---

### Task 12: Background Tasks (Scheduled Publishing + Trash Cleanup)

**Files:**
- Create: `backend/src/tasks.rs`
- Modify: `backend/src/main.rs`

**Step 1: Create background task runner**

`backend/src/tasks.rs`:

```rust
use sqlx::SqlitePool;
use std::time::Duration;

pub fn spawn_background_tasks(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = run_scheduled_tasks(&pool).await {
                tracing::error!("Background task error: {:?}", e);
            }
        }
    });
}

async fn run_scheduled_tasks(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Publish scheduled content
    let published = sqlx::query(
        "UPDATE pages SET status = 'published', updated_at = datetime('now')
         WHERE status = 'scheduled' AND publish_at <= datetime('now')"
    )
    .execute(pool).await?;

    if published.rows_affected() > 0 {
        tracing::info!("Published {} scheduled pages", published.rows_affected());
    }

    let published = sqlx::query(
        "UPDATE articles SET status = 'published', updated_at = datetime('now')
         WHERE status = 'scheduled' AND publish_at <= datetime('now')"
    )
    .execute(pool).await?;

    if published.rows_affected() > 0 {
        tracing::info!("Published {} scheduled articles", published.rows_affected());
    }

    // Permanently delete trashed items older than 30 days
    let deleted_pages = sqlx::query(
        "DELETE FROM pages WHERE status = 'trashed' AND trashed_at < datetime('now', '-30 days')"
    )
    .execute(pool).await?;

    let deleted_articles = sqlx::query(
        "DELETE FROM articles WHERE status = 'trashed' AND trashed_at < datetime('now', '-30 days')"
    )
    .execute(pool).await?;

    if deleted_pages.rows_affected() + deleted_articles.rows_affected() > 0 {
        tracing::info!(
            "Cleaned up {} pages and {} articles from trash",
            deleted_pages.rows_affected(),
            deleted_articles.rows_affected()
        );
    }

    // Clean up expired sessions
    sqlx::query("DELETE FROM sessions WHERE expires_at < datetime('now')")
        .execute(pool).await?;

    Ok(())
}
```

**Step 2: Spawn tasks in main.rs**

Add to main.rs before starting the server:
```rust
mod tasks;
// ... in main():
tasks::spawn_background_tasks(state.db.clone());
```

**Step 3: Verify, commit**

```bash
cd backend && cargo check
git add backend/ && git commit -m "feat: add background tasks for scheduled publishing and trash cleanup"
```

---

## Phase 4: Frontend Foundation

### Task 13: Scaffold SvelteKit Project

**Files:**
- Create: `frontend/` (entire SvelteKit project)

**Step 1: Initialize SvelteKit**

```bash
cd /path/to/pawtal
npm create svelte@latest frontend -- --template skeleton --types typescript
cd frontend
npm install
npm install -D @sveltejs/adapter-node
```

**Step 2: Configure adapter-node**

Update `frontend/svelte.config.js`:

```javascript
import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ out: 'build' }),
  },
};
```

**Step 3: Install core dependencies**

```bash
cd frontend
npm install @tiptap/core @tiptap/starter-kit @tiptap/extension-image @tiptap/extension-link @tiptap/extension-table @tiptap/extension-placeholder svelte-tiptap
npm install -D @fontsource/nunito @fontsource/inter
```

**Step 4: Verify it builds**

```bash
cd frontend && npm run build
```

**Step 5: Commit**

```bash
git add frontend/
git commit -m "feat: scaffold SvelteKit frontend with adapter-node"
```

---

### Task 14: Design Tokens and Global Styles

**Files:**
- Create: `frontend/src/app.css`
- Modify: `frontend/src/app.html`

**Step 1: Create global styles with design tokens**

`frontend/src/app.css`:

```css
@import '@fontsource/nunito/400.css';
@import '@fontsource/nunito/600.css';
@import '@fontsource/nunito/700.css';
@import '@fontsource/inter/400.css';
@import '@fontsource/inter/500.css';

:root {
  --color-primary: #E8924A;
  --color-primary-hover: #D47E3A;
  --color-secondary: #7BA68C;
  --color-secondary-hover: #6B967C;
  --color-accent: #E85D5D;
  --color-accent-hover: #D54D4D;
  --color-bg: #FFF8F0;
  --color-surface: #FFFFFF;
  --color-text: #3D3229;
  --color-text-muted: #8A7D72;
  --color-border: #E8DED4;

  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-full: 9999px;

  --shadow-sm: 0 1px 3px rgba(61,50,41,0.08);
  --shadow-md: 0 4px 12px rgba(61,50,41,0.12);
  --shadow-lg: 0 8px 24px rgba(61,50,41,0.16);

  --font-heading: 'Nunito', sans-serif;
  --font-body: 'Inter', sans-serif;

  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
}

[data-theme="dark"] {
  --color-bg: #1A1614;
  --color-surface: #2A2420;
  --color-text: #F0E8E0;
  --color-text-muted: #A89888;
  --color-border: #3D3229;
}

*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  font-family: var(--font-body);
  color: var(--color-text);
  background-color: var(--color-bg);
  -webkit-font-smoothing: antialiased;
}

h1, h2, h3, h4, h5, h6 {
  font-family: var(--font-heading);
  font-weight: 700;
}

a {
  color: var(--color-primary);
  text-decoration: none;
}
a:hover {
  color: var(--color-primary-hover);
}

button {
  font-family: var(--font-body);
  cursor: pointer;
}
```

**Step 2: Commit**

```bash
git add frontend/src/app.css
git commit -m "feat: add Pawtal design tokens and global styles"
```

---

### Task 15: API Client Library

**Files:**
- Create: `frontend/src/lib/api/client.ts`
- Create: `frontend/src/lib/api/types.ts`

**Step 1: Create TypeScript types matching backend models**

`frontend/src/lib/api/types.ts` — define interfaces for Page, Article, Media, App, MenuItem, Category, SearchResult, PaginatedResponse, SiteSettings, AuditLogEntry, User.

**Step 2: Create API client**

`frontend/src/lib/api/client.ts`:

```typescript
const BASE = '/api';

async function fetchApi<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    ...options,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || res.statusText);
  }
  return res.json();
}

export const api = {
  // Public
  getPage: (slug: string) => fetchApi<Page>(`/pages/${slug}`),
  listArticles: (page = 1) => fetchApi<PaginatedResponse<Article>>(`/articles?page=${page}`),
  getArticle: (slug: string) => fetchApi<Article>(`/articles/${slug}`),
  listApps: (page = 1) => fetchApi<PaginatedResponse<App>>(`/apps?page=${page}`),
  getMenu: (name: string) => fetchApi<{ menu: Menu; items: MenuItem[] }>(`/menus/${name}`),
  getPublicSettings: () => fetchApi<Record<string, string>>(`/settings/public`),
  search: (q: string, type?: string) => fetchApi<SearchResult[]>(`/search?q=${encodeURIComponent(q)}${type ? `&type=${type}` : ''}`),

  // Admin
  admin: {
    me: () => fetchApi<User>('/admin/me'),
    // Pages
    listPages: (page = 1, status?: string) => fetchApi<PaginatedResponse<Page>>(`/admin/pages?page=${page}${status ? `&status=${status}` : ''}`),
    getPage: (id: string) => fetchApi<Page>(`/admin/pages/${id}`),
    createPage: (data: Partial<Page>) => fetchApi<Page>('/admin/pages', { method: 'POST', body: JSON.stringify(data) }),
    updatePage: (id: string, data: Partial<Page>) => fetchApi<Page>(`/admin/pages/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
    deletePage: (id: string) => fetchApi(`/admin/pages/${id}`, { method: 'DELETE' }),
    publishPage: (id: string) => fetchApi<Page>(`/admin/pages/${id}/publish`, { method: 'POST' }),
    restorePage: (id: string) => fetchApi<Page>(`/admin/pages/${id}/restore`, { method: 'POST' }),
    // ... same pattern for articles, media, apps, categories, menus, settings, audit, trash, users
  },
};
```

**Step 3: Commit**

```bash
git add frontend/src/lib/api/
git commit -m "feat: add TypeScript API client library"
```

---

## Phase 5: Frontend — Admin Panel

### Task 16: Admin Layout and Auth Guard

**Files:**
- Create: `frontend/src/routes/admin/+layout.svelte`
- Create: `frontend/src/routes/admin/+layout.server.ts`
- Create: `frontend/src/routes/admin/+page.svelte` (dashboard)

**Step 1: Create admin layout with sidebar**

The admin layout should:
- Check auth via `+layout.server.ts` (redirect to `/api/auth/login` if no session)
- Render sidebar with nav links: Dashboard, Pages, Articles, Media, Apps, Menus, Settings, Trash, Audit Log
- Sidebar collapses to bottom nav on mobile
- Show current user in sidebar footer
- Use Pawtal design tokens (warm colors, rounded corners)

**Step 2: Create dashboard page**

Shows:
- Welcome message with Koda mascot
- Quick stats cards (draft count, published count, recent activity)
- Quick action buttons (New Page, New Article)

**Step 3: Commit**

```bash
git add frontend/src/routes/admin/
git commit -m "feat: add admin layout with sidebar navigation and dashboard"
```

---

### Task 17: Admin Pages List and Editor

**Files:**
- Create: `frontend/src/routes/admin/pages/+page.svelte` (list)
- Create: `frontend/src/routes/admin/pages/new/+page.svelte` (create)
- Create: `frontend/src/routes/admin/pages/[id]/+page.svelte` (edit)
- Create: `frontend/src/lib/components/RichTextEditor.svelte`

**Step 1: Create rich text editor component**

Wrap TipTap with Svelte:
- Toolbar: headings (H1-H3), bold, italic, bullet list, ordered list, link, blockquote, code block, image (opens media library modal), table
- Style toolbar with Pawtal design tokens
- Emit `on:change` with HTML content

**Step 2: Create pages list**

- Table with columns: Title, Status (badge), Author, Updated
- Status filter tabs: All, Draft, Published, Scheduled
- "New Page" button
- Click row to edit
- Delete button (trash)

**Step 3: Create page editor**

- Title input (auto-generates slug)
- Slug input (editable)
- Rich text editor (TipTap component)
- Category picker (multi-select)
- Status selector (Draft / Published / Scheduled)
- If Scheduled: date/time picker for `publish_at`
- Revision history sidebar (collapsible)
- Save / Publish / Preview buttons

**Step 4: Commit**

```bash
git add frontend/src/routes/admin/pages/ frontend/src/lib/components/RichTextEditor.svelte
git commit -m "feat: add admin pages list and TipTap rich text editor"
```

---

### Task 18: Admin Articles List and Editor

**Files:**
- Create: `frontend/src/routes/admin/articles/+page.svelte`
- Create: `frontend/src/routes/admin/articles/new/+page.svelte`
- Create: `frontend/src/routes/admin/articles/[id]/+page.svelte`

Mirrors Task 17 exactly but with:
- `short_text` textarea field added above the rich text editor
- Otherwise identical pattern

**Commit:**
```bash
git add frontend/src/routes/admin/articles/
git commit -m "feat: add admin articles list and editor"
```

---

### Task 19: Admin Media Library

**Files:**
- Create: `frontend/src/routes/admin/media/+page.svelte`
- Create: `frontend/src/lib/components/MediaLibrary.svelte` (reusable modal version)
- Create: `frontend/src/lib/components/MediaUpload.svelte`

**Step 1: Create media upload component**

- Drag-and-drop zone + file input
- Progress indicator
- Multipart upload to `/api/admin/media`

**Step 2: Create media library page**

- Grid of image thumbnails
- Filter tabs: All, Images, Icons
- Click to view details (filename, dimensions, alt text edit)
- Delete button
- Upload button

**Step 3: Create reusable media picker modal**

- Same grid but in a modal
- Used by rich text editor for image insert
- Used by app catalogue for icon selection
- Returns selected media item

**Commit:**
```bash
git add frontend/src/routes/admin/media/ frontend/src/lib/components/Media*.svelte
git commit -m "feat: add admin media library with drag-and-drop upload"
```

---

### Task 20: Admin App Catalogue, Menu Editor, Settings, Trash, Audit

**Files:**
- Create: `frontend/src/routes/admin/apps/+page.svelte`
- Create: `frontend/src/routes/admin/menus/+page.svelte`
- Create: `frontend/src/routes/admin/settings/+page.svelte`
- Create: `frontend/src/routes/admin/trash/+page.svelte`
- Create: `frontend/src/routes/admin/audit/+page.svelte`

**Step 1: App catalogue manager**

- List of apps with drag-and-drop reorder (use HTML5 drag or a Svelte DnD library)
- Add/edit app form: name, description, icon (media picker), link type (URL or page), URL/page selector
- Delete button

**Step 2: Menu editor**

- Tree view of menu items with drag-and-drop reorder and nesting
- Add item form: label, link type (page/article/URL/app catalogue), link target
- Delete item
- Tab switching between "main" and "footer" menus

**Step 3: Settings page**

- Form fields for: site title, front page type (dropdown: page/articles/app catalogue), front page selector, apps per page, app catalogue intro text, dark mode default
- Save button
- Admin only (editor role sees 403)

**Step 4: Trash page**

- List of trashed pages and articles
- Restore button per item
- "Empty trash" button (permanent delete of items > 30 days)

**Step 5: Audit log page**

- Paginated table: timestamp, user, action, entity type, entity
- Admin only

**Commit:**
```bash
git add frontend/src/routes/admin/
git commit -m "feat: add admin apps, menus, settings, trash, and audit log pages"
```

---

## Phase 6: Frontend — Public Site

### Task 21: Public Layout and Navigation

**Files:**
- Create: `frontend/src/routes/(public)/+layout.svelte`
- Create: `frontend/src/routes/(public)/+layout.server.ts`
- Create: `frontend/src/lib/components/Navigation.svelte`
- Create: `frontend/src/lib/components/DarkModeToggle.svelte`
- Create: `frontend/src/lib/components/SearchBar.svelte`

**Step 1: Create public layout**

- Load menu and site settings in `+layout.server.ts`
- Header with site title, navigation from menu API, search bar, dark mode toggle
- Sidebar on desktop with navigation links
- Bottom nav on mobile
- Footer with footer menu
- Warm Pawtal styling

**Step 2: Dark mode toggle**

- Toggle between light/dark themes
- Store preference in localStorage
- Respect `dark_mode_default` from settings on first visit
- Sets `data-theme="dark"` on `<html>`

**Step 3: Search bar**

- Input with search icon
- On submit, navigate to `/search?q=...`
- Optional: live search with debounced API calls

**Commit:**
```bash
git add frontend/src/routes/\(public\)/ frontend/src/lib/components/
git commit -m "feat: add public layout with navigation, dark mode, and search"
```

---

### Task 22: Public Front Page, Pages, Articles, Apps, Search

**Files:**
- Create: `frontend/src/routes/(public)/+page.svelte`
- Create: `frontend/src/routes/(public)/+page.server.ts`
- Create: `frontend/src/routes/(public)/[slug]/+page.svelte`
- Create: `frontend/src/routes/(public)/[slug]/+page.server.ts`
- Create: `frontend/src/routes/(public)/articles/+page.svelte`
- Create: `frontend/src/routes/(public)/articles/[slug]/+page.svelte`
- Create: `frontend/src/routes/(public)/apps/+page.svelte`
- Create: `frontend/src/routes/(public)/search/+page.svelte`

**Step 1: Front page**

- In `+page.server.ts`, load settings to determine front page type
- If `page`: load the configured page and render its content
- If `articles`: render paginated article list
- If `app_catalogue`: render app catalogue grid
- Show Milo mascot in empty states

**Step 2: Page view**

- `/[slug]` — loads page by slug from public API
- Renders HTML content
- Category tags shown at bottom
- 404 with Koda mascot if not found

**Step 3: Articles**

- `/articles` — paginated card list (title, short text, date, categories)
- `/articles/[slug]` — full article view
- Card-based layout with warm shadows

**Step 4: App catalogue**

- `/apps` — grid of app cards
- Each card: icon, name, description
- Click links to URL or dedicated page
- Pagination respects `apps_per_page` setting
- Optional intro text from settings above the grid

**Step 5: Search results**

- `/search?q=...` — unified results with type badges (Page, Article, App)
- Highlighted snippets from FTS5
- Links to respective content

**Commit:**
```bash
git add frontend/src/routes/\(public\)/
git commit -m "feat: add public pages, articles, app catalogue, and search"
```

---

## Phase 7: Docker and Deployment

### Task 23: Docker Setup

**Files:**
- Create: `Dockerfile`
- Create: `docker-compose.yml`
- Create: `.dockerignore`

**Step 1: Create multi-stage Dockerfile**

```dockerfile
# Stage 1: Build Rust backend
FROM rust:1.84-bookworm AS backend-builder
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release

# Stage 2: Build SvelteKit frontend
FROM node:22-bookworm AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=backend-builder /app/backend/target/release/pawtal ./pawtal
COPY --from=frontend-builder /app/frontend/build ./frontend/build

RUN mkdir -p data uploads

EXPOSE 8080
CMD ["./pawtal"]
```

**Step 2: Create docker-compose.yml**

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
      - DATABASE_URL=sqlite:/app/data/pawtal.db?mode=rwc
      - UPLOADS_DIR=/app/uploads
      - OAUTH2_CLIENT_ID=${OAUTH2_CLIENT_ID}
      - OAUTH2_CLIENT_SECRET=${OAUTH2_CLIENT_SECRET}
      - OAUTH2_ISSUER_URL=${OAUTH2_ISSUER_URL}
      - SESSION_SECRET=${SESSION_SECRET}
      - BASE_URL=${BASE_URL}
    restart: unless-stopped

volumes:
  pawtal-data:
  pawtal-uploads:
```

**Step 3: Create .dockerignore**

```
target/
node_modules/
.git/
*.md
.env
```

**Step 4: Update Rust backend to serve SvelteKit build**

In `main.rs`, add a fallback route that serves the SvelteKit build:
```rust
use tower_http::services::ServeDir;

// After all API routes, add:
let app = app
    .nest_service("/uploads", ServeDir::new(&config.uploads_dir))
    .fallback_service(ServeDir::new("frontend/build").append_index_html_on_directories(true));
```

**Step 5: Build and verify Docker image**

```bash
docker compose build
docker compose up -d
# Test: curl http://localhost:8080/api/health
```

**Step 6: Commit**

```bash
git add Dockerfile docker-compose.yml .dockerignore
git commit -m "feat: add Docker multi-stage build and compose configuration"
```

---

## Phase 8: Content Preview

### Task 24: Content Preview Feature

**Files:**
- Modify: `frontend/src/routes/admin/pages/[id]/+page.svelte`
- Modify: `frontend/src/routes/admin/articles/[id]/+page.svelte`

**Step 1: Add preview mode**

- Add "Preview" button in page/article editor
- Opens a modal or new tab showing the content rendered as it would appear on the public site
- Uses the same public layout components but with draft content
- Backend endpoint `GET /api/admin/pages/{id}/preview` returns page regardless of status

**Commit:**
```bash
git add frontend/
git commit -m "feat: add content preview for pages and articles"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-4 | Backend foundation: scaffold, DB, models, errors |
| 2 | 5 | Authentication: OAuth2, sessions, middleware |
| 3 | 6-12 | Backend CRUD: all services and API endpoints |
| 4 | 13-15 | Frontend foundation: SvelteKit, styles, API client |
| 5 | 16-20 | Frontend admin: layout, editors, media, all admin pages |
| 6 | 21-22 | Frontend public: layout, pages, articles, apps, search |
| 7 | 23 | Docker deployment |
| 8 | 24 | Content preview |

**Total: 24 tasks across 8 phases.**

Phases 1-3 (backend) can be built and tested independently via API calls.
Phases 4-6 (frontend) depend on the backend being functional.
Phase 7 (Docker) can be done any time after Phase 1.
Phase 8 is a refinement pass.
