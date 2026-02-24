//! Site settings service.
//!
//! Settings are stored as key-value pairs in the `site_settings` table. The
//! public subset is small and well-known; everything else is admin-only. All
//! writes are upserts so missing keys are created on first write rather than
//! requiring a seed script.

use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::services::audit;

/// Keys that are safe to expose without authentication.
const PUBLIC_KEYS: &[&str] = &[
    "site_title",
    "site_description",
    "front_page_type",
    "front_page_slug",
    "apps_per_page",
    "app_catalogue_intro",
    "dark_mode_default",
];

/// All known settings keys. Updates with unrecognised keys are rejected.
const ALLOWED_KEYS: &[&str] = &[
    "site_title",
    "site_description",
    "front_page_type",
    "front_page_slug",
    "apps_per_page",
    "app_catalogue_intro",
    "dark_mode_default",
];

// ─── Public service functions ─────────────────────────────────────────────────

/// Returns every key-value pair in `site_settings`. Admin-only.
pub async fn get_all_settings(pool: &SqlitePool) -> AppResult<HashMap<String, String>> {
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM site_settings")
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().collect())
}

/// Returns only the settings that are safe to expose publicly.
///
/// Keys that have no stored value are omitted rather than returned as empty
/// strings, so callers can distinguish "not configured" from "set to empty".
pub async fn get_public_settings(pool: &SqlitePool) -> AppResult<HashMap<String, String>> {
    let placeholders = PUBLIC_KEYS
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "SELECT key, value FROM site_settings WHERE key IN ({})",
        placeholders
    );

    let mut query = sqlx::query_as::<_, (String, String)>(&sql);
    for key in PUBLIC_KEYS {
        query = query.bind(*key);
    }

    let rows = query.fetch_all(pool).await?;

    Ok(rows.into_iter().collect())
}

/// Upserts a batch of key-value pairs.
///
/// Each entry is written individually in a loop. SQLite is local so the
/// round-trip cost is negligible, and this keeps the code simple and
/// easy to extend with per-key validation later.
pub async fn update_settings(
    pool: &SqlitePool,
    updates: HashMap<String, String>,
    user_id: &str,
) -> AppResult<()> {
    // Reject unrecognised keys to prevent settings table pollution.
    for key in updates.keys() {
        if !ALLOWED_KEYS.contains(&key.as_str()) {
            return Err(crate::error::AppError::BadRequest(format!(
                "Unknown setting key: '{}'",
                key
            )));
        }
    }

    for (key, value) in &updates {
        sqlx::query(
            "INSERT INTO site_settings (key, value, updated_at) \
             VALUES (?, ?, datetime('now')) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    }

    audit::log_action(
        pool,
        user_id,
        "update",
        "settings",
        "site_settings",
        &serde_json::json!({ "keys": updates.keys().collect::<Vec<_>>() }),
    )
    .await?;

    Ok(())
}
