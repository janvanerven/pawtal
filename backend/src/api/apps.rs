//! HTTP handlers for the apps catalogue resource.
//!
//! Route map (registered in main.rs):
//!
//!   Public:
//!     GET  /api/apps
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET    /api/admin/apps
//!     POST   /api/admin/apps
//!     PUT    /api/admin/apps/reorder   ← must come BEFORE /:id route
//!     GET    /api/admin/apps/:id
//!     PUT    /api/admin/apps/:id
//!     DELETE /api/admin/apps/:id

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};

use crate::db::models::{App, CreateApp, PaginatedResponse, PaginationParams, UpdateApp, User};
use crate::error::AppResult;
use crate::services::apps as svc;
use crate::AppState;

// ─── Public endpoints ─────────────────────────────────────────────────────────

/// `GET /api/apps`
///
/// Returns a paginated list of apps ordered by sort_order. The default
/// page size is sourced from the `apps_per_page` site setting when available.
pub async fn public_list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<App>>> {
    let result = svc::list_apps(&state.db, &pagination).await?;
    Ok(Json(result))
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/apps`
///
/// Returns a paginated list of all apps.
pub async fn admin_list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<App>>> {
    let result = svc::list_apps(&state.db, &pagination).await?;
    Ok(Json(result))
}

/// `POST /api/admin/apps`
///
/// Creates a new app, appended at the end of the sort order.
pub async fn admin_create(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(input): Json<CreateApp>,
) -> AppResult<Json<App>> {
    let app = svc::create_app(&state.db, input, &user.id).await?;
    Ok(Json(app))
}

/// `GET /api/admin/apps/:id`
///
/// Fetches a single app by its ID.
pub async fn admin_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<App>> {
    let app = svc::get_app(&state.db, &id).await?;
    Ok(Json(app))
}

/// `PUT /api/admin/apps/:id`
///
/// Applies a partial update to an existing app.
pub async fn admin_update(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    Json(input): Json<UpdateApp>,
) -> AppResult<Json<App>> {
    let app = svc::update_app(&state.db, &id, input, &user.id).await?;
    Ok(Json(app))
}

/// `DELETE /api/admin/apps/:id`
///
/// Permanently deletes an app.
pub async fn admin_delete(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    svc::delete_app(&state.db, &id, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// `PUT /api/admin/apps/reorder`
///
/// Receives an ordered list of app IDs and updates sort_order accordingly.
/// This route must be registered before `/api/admin/apps/:id` in main.rs
/// to prevent the literal string "reorder" being treated as an ID parameter.
pub async fn admin_reorder(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(ids): Json<Vec<String>>,
) -> AppResult<Json<serde_json::Value>> {
    svc::reorder_apps(&state.db, ids, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
