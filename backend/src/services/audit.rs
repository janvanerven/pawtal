//! Audit log service — records every significant CMS action to the `audit_log`
//! table for accountability and debugging.
//!
//! Callers should fire-and-forget where possible. We deliberately do not return
//! a domain error if the audit write fails: losing an audit entry is bad, but
//! it should never roll back the user-visible operation that triggered it.
//! However, since we do return `AppResult`, callers can choose to propagate the
//! error when auditability is a hard requirement.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::AppResult;

/// Appends one row to `audit_log`.
///
/// * `user_id`     — the authenticated user who performed the action
/// * `action`      — verb describing the operation (e.g. "create", "update", "publish", "trash")
/// * `entity_type` — the affected resource type (e.g. "page", "article", "media")
/// * `entity_id`   — primary key of the affected row
/// * `details`     — arbitrary JSON payload (before/after snapshots, extra context)
pub async fn log_action(
    pool: &SqlitePool,
    user_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: &serde_json::Value,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO audit_log (id, user_id, action, entity_type, entity_id, details) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details.to_string())
    .execute(pool)
    .await?;

    Ok(())
}
