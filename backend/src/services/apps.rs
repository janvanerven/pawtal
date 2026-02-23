//! Apps catalogue service.
//!
//! Apps are entries in a sortable catalogue displayed on the public-facing
//! app catalogue page. Sort order is explicit (stored as an integer) and can
//! be updated in bulk via `reorder_apps`.

use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{App, CreateApp, PaginatedResponse, PaginationParams, UpdateApp};
use crate::error::{AppError, AppResult};
use crate::services::audit;

// ─── Column list shared by all SELECT queries ─────────────────────────────────

const APP_COLS: &str =
    "a.id, a.name, a.description, a.icon_id, a.url, a.page_id, a.sort_order, \
     a.created_at, a.updated_at, m.filename AS icon_filename";

// ─── Public service functions ─────────────────────────────────────────────────

/// Returns a paginated list of apps ordered by `sort_order` ascending.
///
/// `per_page` defaults to the `apps_per_page` setting when the caller does not
/// supply one, falling back to 20 when the setting is absent or unparseable.
pub async fn list_apps(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<App>> {
    let per_page = params.per_page() as i64;
    let offset = params.offset() as i64;

    let rows = sqlx::query_as::<_, App>(&format!(
        "SELECT {APP_COLS} FROM apps a LEFT JOIN media m ON a.icon_id = m.id \
         ORDER BY a.sort_order ASC LIMIT ? OFFSET ?"
    ))
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM apps")
        .fetch_one(pool)
        .await?;

    Ok(PaginatedResponse {
        data: rows,
        total,
        page: params.page.unwrap_or(1).max(1),
        per_page: params.per_page(),
    })
}

/// Fetches a single app by primary key. Returns `NotFound` if absent.
pub async fn get_app(pool: &SqlitePool, id: &str) -> AppResult<App> {
    sqlx::query_as::<_, App>(&format!(
        "SELECT {APP_COLS} FROM apps a LEFT JOIN media m ON a.icon_id = m.id WHERE a.id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Creates a new app, placing it at the end of the sort order.
pub async fn create_app(
    pool: &SqlitePool,
    input: CreateApp,
    user_id: &str,
) -> AppResult<App> {
    let id = Uuid::new_v4().to_string();
    let description = input.description.unwrap_or_default();

    // Append to the end of the current list.
    let max_order = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(sort_order) FROM apps")
        .fetch_one(pool)
        .await?
        .unwrap_or(-1);
    let sort_order = (max_order + 1) as i32;

    sqlx::query(
        "INSERT INTO apps (id, name, description, icon_id, url, page_id, sort_order) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&description)
    .bind(&input.icon_id)
    .bind(&input.url)
    .bind(&input.page_id)
    .bind(sort_order)
    .execute(pool)
    .await?;

    audit::log_action(
        pool,
        user_id,
        "create",
        "app",
        &id,
        &json!({ "name": input.name }),
    )
    .await?;

    get_app(pool, &id).await
}

/// Applies a partial update to an existing app. Fields absent from `input`
/// keep their current values.
pub async fn update_app(
    pool: &SqlitePool,
    id: &str,
    input: UpdateApp,
    user_id: &str,
) -> AppResult<App> {
    let existing = get_app(pool, id).await?;

    let name = input.name.unwrap_or_else(|| existing.name.clone());
    let description = input.description.unwrap_or_else(|| existing.description.clone());
    let icon_id = if input.icon_id.is_some() { input.icon_id } else { existing.icon_id };
    let url = if input.url.is_some() { input.url } else { existing.url };
    let page_id = if input.page_id.is_some() { input.page_id } else { existing.page_id };

    sqlx::query(
        "UPDATE apps \
         SET name = ?, description = ?, icon_id = ?, url = ?, page_id = ?, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(&name)
    .bind(&description)
    .bind(&icon_id)
    .bind(&url)
    .bind(&page_id)
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(
        pool,
        user_id,
        "update",
        "app",
        id,
        &json!({ "name": name }),
    )
    .await?;

    get_app(pool, id).await
}

/// Deletes an app by ID.
pub async fn delete_app(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<()> {
    let app = get_app(pool, id).await?;

    sqlx::query("DELETE FROM apps WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    audit::log_action(
        pool,
        user_id,
        "delete",
        "app",
        id,
        &json!({ "name": app.name }),
    )
    .await?;

    Ok(())
}

/// Updates `sort_order` for a list of app IDs. The position in `ids` becomes
/// the new `sort_order` value (0-indexed).
///
/// Unknown IDs are silently skipped — they may have been deleted concurrently.
pub async fn reorder_apps(
    pool: &SqlitePool,
    ids: Vec<String>,
    user_id: &str,
) -> AppResult<()> {
    for (index, app_id) in ids.iter().enumerate() {
        sqlx::query(
            "UPDATE apps SET sort_order = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
             WHERE id = ?",
        )
        .bind(index as i32)
        .bind(app_id)
        .execute(pool)
        .await?;
    }

    audit::log_action(
        pool,
        user_id,
        "reorder",
        "apps",
        "apps",
        &json!({ "count": ids.len() }),
    )
    .await?;

    Ok(())
}
