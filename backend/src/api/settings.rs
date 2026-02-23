//! HTTP handlers for the site settings resource.
//!
//! Route map (registered in main.rs):
//!
//!   Public:
//!     GET  /api/settings/public
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET  /api/admin/settings
//!     PUT  /api/admin/settings

use std::collections::HashMap;

use axum::{
    extract::{Extension, State},
    Json,
};

use crate::db::models::User;
use crate::error::AppResult;
use crate::services::settings as svc;
use crate::AppState;

// ─── Public endpoints ─────────────────────────────────────────────────────────

/// `GET /api/settings/public`
///
/// Returns the subset of settings that are safe to expose without
/// authentication (site_title, front_page_type, etc.).
pub async fn public_get(
    State(state): State<AppState>,
) -> AppResult<Json<HashMap<String, String>>> {
    let settings = svc::get_public_settings(&state.db).await?;
    Ok(Json(settings))
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/settings`
///
/// Returns all key-value pairs in site_settings. Admin-only.
pub async fn admin_get(
    State(state): State<AppState>,
) -> AppResult<Json<HashMap<String, String>>> {
    let settings = svc::get_all_settings(&state.db).await?;
    Ok(Json(settings))
}

/// `PUT /api/admin/settings`
///
/// Upserts a batch of key-value pairs. Admin-only.
pub async fn admin_update(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(updates): Json<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    svc::update_settings(&state.db, updates, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
