//! HTTP handlers for the articles resource.
//!
//! Handlers are intentionally thin: they extract parameters, delegate to the
//! service layer, and wrap the result in the appropriate JSON envelope. No
//! business logic lives here.
//!
//! Route map (registered in main.rs):
//!
//!   Public:
//!     GET  /api/articles
//!     GET  /api/articles/:slug
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET    /api/admin/articles
//!     POST   /api/admin/articles
//!     GET    /api/admin/articles/:id
//!     PUT    /api/admin/articles/:id
//!     DELETE /api/admin/articles/:id
//!     POST   /api/admin/articles/:id/publish
//!     POST   /api/admin/articles/:id/restore
//!     GET    /api/admin/articles/:id/revisions
//!     POST   /api/admin/articles/:id/revisions/:rev_id/restore

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::db::models::{Article, ArticleRevision, CreateArticle, PaginatedResponse,
    PaginationParams, UpdateArticle, User};
use crate::error::AppResult;
use crate::services::articles as svc;
use crate::AppState;

// ─── Query parameter structs ──────────────────────────────────────────────────

/// Optional `?status=` filter used by the admin list endpoint.
#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

// ─── Public endpoints ─────────────────────────────────────────────────────────

/// `GET /api/articles`
///
/// Returns a paginated list of published articles, ordered newest first.
/// Unauthenticated — only published content is visible.
pub async fn public_list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Article>>> {
    let result = svc::list_published_articles(&state.db, &pagination).await?;
    Ok(Json(result))
}

/// `GET /api/articles/:slug`
///
/// Returns a published article by its URL slug. Unauthenticated — only
/// published content is visible.
pub async fn public_get_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Article>> {
    let article = svc::get_article_by_slug(&state.db, &slug).await?;
    Ok(Json(article))
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/articles`
///
/// Lists articles with optional status filtering and pagination.
pub async fn admin_list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<StatusFilter>,
) -> AppResult<Json<PaginatedResponse<Article>>> {
    let result = svc::list_articles(&state.db, &pagination, filter.status.as_deref()).await?;
    Ok(Json(result))
}

/// `GET /api/admin/articles/:id`
///
/// Fetches a single article by its ID.
pub async fn admin_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Article>> {
    let article = svc::get_article(&state.db, &id).await?;
    Ok(Json(article))
}

/// `POST /api/admin/articles`
///
/// Creates a new article. The authenticated user becomes the author.
pub async fn admin_create(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(input): Json<CreateArticle>,
) -> AppResult<Json<Article>> {
    let article = svc::create_article(&state.db, input, &user.id).await?;
    Ok(Json(article))
}

/// `PUT /api/admin/articles/:id`
///
/// Applies a partial update to an existing article.
pub async fn admin_update(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    Json(input): Json<UpdateArticle>,
) -> AppResult<Json<Article>> {
    let article = svc::update_article(&state.db, &id, input, &user.id).await?;
    Ok(Json(article))
}

/// `DELETE /api/admin/articles/:id`
///
/// Moves the article to the trash. The row is retained and can be restored.
pub async fn admin_delete(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    svc::trash_article(&state.db, &id, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// `POST /api/admin/articles/:id/publish`
///
/// Transitions the article status to `published`.
pub async fn admin_publish(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<Article>> {
    let article = svc::publish_article(&state.db, &id, &user.id).await?;
    Ok(Json(article))
}

/// `POST /api/admin/articles/:id/restore`
///
/// Restores a trashed article to `draft` status.
pub async fn admin_restore(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<Article>> {
    let article = svc::restore_article(&state.db, &id, &user.id).await?;
    Ok(Json(article))
}

/// `GET /api/admin/articles/:id/revisions`
///
/// Lists all revisions for an article, newest first.
pub async fn admin_revisions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<ArticleRevision>>> {
    let revisions = svc::list_revisions(&state.db, &id).await?;
    Ok(Json(revisions))
}

/// `POST /api/admin/articles/:id/revisions/:rev_id/restore`
///
/// Restores an article to a previously captured revision state. Creates a new
/// revision recording the restore rather than destroying history.
pub async fn admin_restore_revision(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((id, rev_id)): Path<(String, String)>,
) -> AppResult<Json<Article>> {
    let article = svc::restore_revision(&state.db, &id, &rev_id, &user.id).await?;
    Ok(Json(article))
}
