//! HTTP handlers for the trash management endpoints.
//!
//! There is no separate service module for trash — the queries are simple
//! enough to sit directly in the handlers. Both pages and articles share the
//! same `status = 'trashed'` convention, so a single pair of endpoints covers
//! both resource types.
//!
//! Route map (registered in main.rs):
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET  /api/admin/trash
//!     POST /api/admin/trash/empty

use axum::{extract::State, Json};
use serde::Serialize;

use crate::db::models::{Article, Page};
use crate::error::AppResult;
use crate::AppState;

// ─── Response structs ─────────────────────────────────────────────────────────

/// Combined trash listing — both trashed pages and trashed articles.
#[derive(Debug, Serialize)]
pub struct TrashResponse {
    pub pages: Vec<Page>,
    pub articles: Vec<Article>,
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/trash`
///
/// Returns all pages and articles that are currently in the trash.
pub async fn list(State(state): State<AppState>) -> AppResult<Json<TrashResponse>> {
    let pages = sqlx::query_as::<_, Page>(
        "SELECT id, title, slug, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at, template \
         FROM pages \
         WHERE status = 'trashed' \
         ORDER BY trashed_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, slug, short_text, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at, cover_image_id, reading_time_minutes \
         FROM articles \
         WHERE status = 'trashed' \
         ORDER BY trashed_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(TrashResponse { pages, articles }))
}

/// `POST /api/admin/trash/empty`
///
/// Permanently deletes trashed items older than 30 days. Items trashed more
/// recently are retained so users have a grace period to recover mistakes.
pub async fn empty(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let pages_deleted = sqlx::query(
        "DELETE FROM pages \
         WHERE status = 'trashed' \
           AND trashed_at < datetime('now', '-30 days')",
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    let articles_deleted = sqlx::query(
        "DELETE FROM articles \
         WHERE status = 'trashed' \
           AND trashed_at < datetime('now', '-30 days')",
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    Ok(Json(serde_json::json!({
        "ok": true,
        "pages_deleted": pages_deleted,
        "articles_deleted": articles_deleted,
    })))
}
