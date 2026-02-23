//! HTTP handlers for the audit log resource.
//!
//! The audit log is append-only and admin-only. Entries are returned newest
//! first so the most recent activity is immediately visible.
//!
//! Route map (registered in main.rs):
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET  /api/admin/audit-log

use axum::{
    extract::{Query, State},
    Json,
};

use crate::db::models::{AuditLogEntry, PaginatedResponse, PaginationParams};
use crate::error::AppResult;
use crate::AppState;

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/audit-log`
///
/// Returns a paginated list of audit log entries, newest first.
pub async fn list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<AuditLogEntry>>> {
    let per_page = pagination.per_page() as i64;
    let offset = pagination.offset() as i64;

    let rows = sqlx::query_as::<_, AuditLogEntry>(
        "SELECT id, user_id, action, entity_type, entity_id, details, created_at \
         FROM audit_log \
         ORDER BY created_at DESC \
         LIMIT ? OFFSET ?",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: rows,
        total,
        page: pagination.page.unwrap_or(1).max(1),
        per_page: pagination.per_page(),
    }))
}
