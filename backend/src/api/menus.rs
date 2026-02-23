//! HTTP handlers for the menus resource.
//!
//! Route map (registered in main.rs):
//!
//!   Public:
//!     GET  /api/menus/:name
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET  /api/admin/menus/:name
//!     PUT  /api/admin/menus/:name

use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Serialize;

use crate::db::models::{Menu, MenuItem, UpdateMenu, User};
use crate::error::AppResult;
use crate::services::menus as svc;
use crate::AppState;

// ─── Response structs ─────────────────────────────────────────────────────────

/// Combined response envelope carrying the menu header and its items.
#[derive(Debug, Serialize)]
pub struct MenuResponse {
    pub menu: Menu,
    pub items: Vec<MenuItem>,
}

// ─── Public endpoints ─────────────────────────────────────────────────────────

/// `GET /api/menus/:name`
///
/// Returns the named menu and its items. Unauthenticated — menus drive
/// public site navigation.
pub async fn public_get(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Json<MenuResponse>> {
    let (menu, items) = svc::get_menu(&state.db, &name).await?;
    Ok(Json(MenuResponse { menu, items }))
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/menus/:name`
///
/// Same as the public endpoint but protected so editors can preview menus
/// that aren't yet surfaced publicly.
pub async fn admin_get(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Json<MenuResponse>> {
    let (menu, items) = svc::get_menu(&state.db, &name).await?;
    Ok(Json(MenuResponse { menu, items }))
}

/// `PUT /api/admin/menus/:name`
///
/// Replaces all items for the named menu. The menu is created automatically
/// if it does not yet exist.
pub async fn admin_update(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(name): Path<String>,
    Json(body): Json<UpdateMenu>,
) -> AppResult<Json<serde_json::Value>> {
    svc::update_menu(&state.db, &name, body.items, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
