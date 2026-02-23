use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Read models (database rows) ─────────────────────────────────────────────
//
// Each struct maps 1-to-1 to a database table row. `sqlx::FromRow` is derived
// so sqlx can hydrate them directly from query results without manual mapping.
// All timestamp columns are stored as TEXT in SQLite and mapped to DateTime<Utc>
// by sqlx via the chrono integration.

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub publish_at: Option<DateTime<Utc>>,
    pub author_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trashed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PageRevision {
    pub id: String,
    pub page_id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub short_text: String,
    pub content: String,
    pub status: String,
    pub publish_at: Option<DateTime<Utc>>,
    pub author_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trashed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ArticleRevision {
    pub id: String,
    pub article_id: String,
    pub title: String,
    pub short_text: String,
    pub content: String,
    pub author_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
}

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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct App {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_id: Option<String>,
    pub url: Option<String>,
    pub page_id: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SiteSetting {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

// ─── Write / input models ─────────────────────────────────────────────────────
//
// These are deserialized from request bodies and never sent to the client, so
// they only need `Deserialize`. Optional fields use `Option` so callers can
// omit them for partial updates (PATCH semantics).

#[derive(Debug, Deserialize)]
pub struct CreatePage {
    pub title: String,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<DateTime<Utc>>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePage {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<DateTime<Utc>>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateArticle {
    pub title: String,
    pub slug: Option<String>,
    pub short_text: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<DateTime<Utc>>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub short_text: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub publish_at: Option<DateTime<Utc>>,
    pub category_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub slug: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct UpdateMenu {
    pub items: Vec<MenuItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct MenuItemInput {
    /// Present when updating an existing item; absent when creating a new one.
    pub id: Option<String>,
    pub label: String,
    pub link_type: String,
    pub link_target: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

// ─── Utility types ────────────────────────────────────────────────────────────

/// A single hit returned by the cross-entity search endpoint. Only serialized,
/// never read from a request body or a DB row directly.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub result_type: String,
    pub id: String,
    pub title: String,
    pub slug: String,
    pub snippet: String,
}

/// Query parameters for paginated list endpoints.
///
/// Both fields are optional; callers that omit them get the defaults defined
/// in the accessor methods below. We keep the raw values private-ish via the
/// struct so that all clamping/defaulting logic lives in one place.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl PaginationParams {
    /// Returns the number of rows to skip, calculated as `(page - 1) * per_page`.
    /// Page numbers below 1 are treated as page 1, so the offset is never negative.
    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.per_page()
    }

    /// Returns the page size, clamped to the range `[1, 100]`. Defaults to 20
    /// when the caller does not specify `per_page`.
    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }
}

/// Standard envelope for any paginated list response.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
