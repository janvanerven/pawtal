//! Menus service.
//!
//! Menus are identified by a name (e.g. "main", "footer"). Each menu has an
//! ordered list of items. Updates replace the entire item list atomically —
//! delete-then-insert is simpler and correct for the small cardinalities
//! typical of navigation menus.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{Menu, MenuItem, MenuItemInput};
use crate::error::{AppError, AppResult};

// ─── Public service functions ─────────────────────────────────────────────────

/// Fetches a menu by name together with its items ordered by `sort_order`.
///
/// Returns `NotFound` when no menu with the given name exists.
pub async fn get_menu(pool: &SqlitePool, name: &str) -> AppResult<(Menu, Vec<MenuItem>)> {
    let menu = sqlx::query_as::<_, Menu>("SELECT id, name FROM menus WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let items = sqlx::query_as::<_, MenuItem>(
        "SELECT id, menu_id, label, link_type, link_target, parent_id, sort_order \
         FROM menu_items \
         WHERE menu_id = ? \
         ORDER BY sort_order ASC",
    )
    .bind(&menu.id)
    .fetch_all(pool)
    .await?;

    Ok((menu, items))
}

/// Replaces all items for the named menu.
///
/// When the menu does not yet exist it is created automatically, so callers
/// can bootstrap menus by writing to them without a separate creation step.
/// Existing item IDs from `MenuItemInput.id` are preserved so the frontend
/// can correlate items across saves; new items receive fresh UUIDs.
pub async fn update_menu(
    pool: &SqlitePool,
    name: &str,
    items: Vec<MenuItemInput>,
    user_id: &str,
) -> AppResult<()> {
    // Retrieve or create the menu row.
    let menu_id = match sqlx::query_scalar::<_, String>("SELECT id FROM menus WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        Some(id) => id,
        None => {
            let new_id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO menus (id, name) VALUES (?, ?)")
                .bind(&new_id)
                .bind(name)
                .execute(pool)
                .await?;
            new_id
        }
    };

    // Delete all existing items for this menu before inserting the new set.
    sqlx::query("DELETE FROM menu_items WHERE menu_id = ?")
        .bind(&menu_id)
        .execute(pool)
        .await?;

    for item in items {
        let item_id = item.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        sqlx::query(
            "INSERT INTO menu_items (id, menu_id, label, link_type, link_target, parent_id, sort_order) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&item_id)
        .bind(&menu_id)
        .bind(&item.label)
        .bind(&item.link_type)
        .bind(&item.link_target)
        .bind(&item.parent_id)
        .bind(item.sort_order)
        .execute(pool)
        .await?;
    }

    crate::services::audit::log_action(
        pool,
        user_id,
        "update",
        "menu",
        &menu_id,
        &serde_json::json!({ "name": name }),
    )
    .await?;

    Ok(())
}
