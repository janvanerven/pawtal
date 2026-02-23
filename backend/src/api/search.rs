//! HTTP handlers for the search resource.
//!
//! Handlers delegate entirely to the search service and add no business logic
//! of their own. The public endpoint excludes unpublished content; the admin
//! endpoint exposes it so editors can find drafts.
//!
//! Route map (registered in main.rs):
//!
//!   Public:
//!     GET  /api/search?q=<query>[&type=pages|articles|apps]
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET  /api/admin/search?q=<query>[&type=pages|articles|apps]

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::db::models::SearchResult;
use crate::error::AppResult;
use crate::services::search as search_svc;
use crate::AppState;

/// Query parameters for both search endpoints.
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// The search query string. An empty value returns an empty result set.
    pub q: String,
    /// Optional filter to restrict results to a single entity kind.
    /// Accepted values: `"pages"`, `"articles"`, `"apps"`.
    /// Omit to search all entity types.
    #[serde(rename = "type")]
    pub search_type: Option<String>,
}

/// `GET /api/search` — full-text search over published content only.
///
/// Suitable for use by unauthenticated visitors. Drafts and archived records
/// are never included in the response.
pub async fn public_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<Vec<SearchResult>>> {
    let results = search_svc::search(
        &state.db,
        &params.q,
        params.search_type.as_deref(),
        false, // exclude unpublished content
    )
    .await?;

    Ok(Json(results))
}

/// `GET /api/admin/search` — full-text search including unpublished content.
///
/// Intended for authenticated admin users who need to locate drafts, archived
/// records, and other non-public content via the CMS UI.
pub async fn admin_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<Vec<SearchResult>>> {
    let results = search_svc::search(
        &state.db,
        &params.q,
        params.search_type.as_deref(),
        true, // include unpublished content
    )
    .await?;

    Ok(Json(results))
}
