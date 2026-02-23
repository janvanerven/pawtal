//! HTTP handlers for the pages resource.
//!
//! Handlers are intentionally thin: they extract parameters, delegate to the
//! service layer, and wrap the result in the appropriate JSON envelope. No
//! business logic lives here.
//!
//! Route map (registered in main.rs):
//!
//!   Public:
//!     GET  /api/pages/:slug
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET    /api/admin/pages
//!     POST   /api/admin/pages
//!     GET    /api/admin/pages/:id
//!     PUT    /api/admin/pages/:id
//!     DELETE /api/admin/pages/:id
//!     POST   /api/admin/pages/:id/publish
//!     POST   /api/admin/pages/:id/restore
//!     GET    /api/admin/pages/:id/revisions
//!     POST   /api/admin/pages/:id/revisions/:rev_id/restore

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::db::models::{CreatePage, Page, PageRevision, PaginatedResponse, PaginationParams,
    UpdatePage, User};
use crate::error::AppResult;
use crate::services::pages as svc;
use crate::AppState;

// ─── Query parameter structs ──────────────────────────────────────────────────

/// Optional `?status=` filter used by the admin list endpoint.
#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

// ─── Public endpoints ─────────────────────────────────────────────────────────

/// `GET /api/pages/:slug`
///
/// Returns a published page by its URL slug. Unauthenticated — only published
/// content is visible.
pub async fn public_get_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Page>> {
    let page = svc::get_page_by_slug(&state.db, &slug).await?;
    Ok(Json(page))
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/pages`
///
/// Lists pages with optional status filtering and pagination.
pub async fn admin_list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<StatusFilter>,
) -> AppResult<Json<PaginatedResponse<Page>>> {
    let result = svc::list_pages(&state.db, &pagination, filter.status.as_deref()).await?;
    Ok(Json(result))
}

/// `GET /api/admin/pages/:id`
///
/// Fetches a single page by its ID.
pub async fn admin_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Page>> {
    let page = svc::get_page(&state.db, &id).await?;
    Ok(Json(page))
}

/// `POST /api/admin/pages`
///
/// Creates a new page. The authenticated user becomes the author.
pub async fn admin_create(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(input): Json<CreatePage>,
) -> AppResult<Json<Page>> {
    let page = svc::create_page(&state.db, input, &user.id).await?;
    Ok(Json(page))
}

/// `PUT /api/admin/pages/:id`
///
/// Applies a partial update to an existing page.
pub async fn admin_update(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePage>,
) -> AppResult<Json<Page>> {
    let page = svc::update_page(&state.db, &id, input, &user.id).await?;
    Ok(Json(page))
}

/// `DELETE /api/admin/pages/:id`
///
/// Moves the page to the trash. The row is retained and can be restored.
pub async fn admin_delete(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    svc::trash_page(&state.db, &id, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// `POST /api/admin/pages/:id/publish`
///
/// Transitions the page status to `published`.
pub async fn admin_publish(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<Page>> {
    let page = svc::publish_page(&state.db, &id, &user.id).await?;
    Ok(Json(page))
}

/// `POST /api/admin/pages/:id/restore`
///
/// Restores a trashed page to `draft` status.
pub async fn admin_restore(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<Page>> {
    let page = svc::restore_page(&state.db, &id, &user.id).await?;
    Ok(Json(page))
}

/// `GET /api/admin/pages/:id/revisions`
///
/// Lists all revisions for a page, newest first.
pub async fn admin_revisions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<PageRevision>>> {
    let revisions = svc::list_revisions(&state.db, &id).await?;
    Ok(Json(revisions))
}

/// `POST /api/admin/pages/:id/revisions/:rev_id/restore`
///
/// Restores a page to a previously captured revision state. Creates a new
/// revision recording the restore rather than destroying history.
pub async fn admin_restore_revision(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((id, rev_id)): Path<(String, String)>,
) -> AppResult<Json<Page>> {
    let page = svc::restore_revision(&state.db, &id, &rev_id, &user.id).await?;
    Ok(Json(page))
}
