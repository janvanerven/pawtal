//! HTTP handlers for the categories resource.
//!
//! Route map (registered in main.rs):
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET    /api/admin/categories
//!     POST   /api/admin/categories
//!     PUT    /api/admin/categories/:id
//!     DELETE /api/admin/categories/:id

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::db::models::{Category, CreateCategory};
use crate::error::AppResult;
use crate::services::categories as svc;
use crate::AppState;

// ─── Input structs ────────────────────────────────────────────────────────────

/// Request body for `PUT /api/admin/categories/:id`.
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryInput {
    pub name: String,
    pub slug: Option<String>,
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/categories`
///
/// Returns all categories ordered alphabetically by name.
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Category>>> {
    let categories = svc::list_categories(&state.db).await?;
    Ok(Json(categories))
}

/// `POST /api/admin/categories`
///
/// Creates a new category. Slug is derived from the name if not supplied.
pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateCategory>,
) -> AppResult<Json<Category>> {
    let category = svc::create_category(&state.db, input).await?;
    Ok(Json(category))
}

/// `PUT /api/admin/categories/:id`
///
/// Updates the name and optionally the slug of an existing category.
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCategoryInput>,
) -> AppResult<Json<Category>> {
    let category = svc::update_category(&state.db, &id, input.name, input.slug).await?;
    Ok(Json(category))
}

/// `DELETE /api/admin/categories/:id`
///
/// Permanently deletes a category. Join-table entries are removed by the DB cascade.
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    svc::delete_category(&state.db, &id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
