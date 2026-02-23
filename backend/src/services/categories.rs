//! Categories CRUD service.
//!
//! Categories are a flat taxonomy that can be attached to pages and articles
//! via join tables. Slug uniqueness is enforced here so all callers benefit
//! from the check regardless of which entry point is used.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{Category, CreateCategory};
use crate::error::{AppError, AppResult};
use crate::helpers::slugify;

// ─── Public service functions ─────────────────────────────────────────────────

/// Returns all categories ordered alphabetically by name.
pub async fn list_categories(pool: &SqlitePool) -> AppResult<Vec<Category>> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT id, name, slug FROM categories ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

/// Fetches a single category by primary key. Returns `NotFound` if absent.
pub async fn get_category(pool: &SqlitePool, id: &str) -> AppResult<Category> {
    sqlx::query_as::<_, Category>("SELECT id, name, slug FROM categories WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Creates a new category.
///
/// When `input.slug` is absent the slug is derived from the name. Returns
/// `Conflict` if the resulting slug is already taken.
pub async fn create_category(pool: &SqlitePool, input: CreateCategory) -> AppResult<Category> {
    let slug = input
        .slug
        .as_deref()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| slugify(&input.name));

    ensure_slug_unique(pool, &slug, None).await?;

    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO categories (id, name, slug) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&input.name)
        .bind(&slug)
        .execute(pool)
        .await?;

    get_category(pool, &id).await
}

/// Updates the name and/or slug of an existing category.
///
/// Slug uniqueness is checked only when the slug actually changes so that
/// an update that only touches the name does not trigger a spurious conflict.
pub async fn update_category(
    pool: &SqlitePool,
    id: &str,
    name: String,
    slug: Option<String>,
) -> AppResult<Category> {
    let existing = get_category(pool, id).await?;

    let new_slug = slug.unwrap_or_else(|| existing.slug.clone());
    if new_slug != existing.slug {
        ensure_slug_unique(pool, &new_slug, Some(id)).await?;
    }

    sqlx::query("UPDATE categories SET name = ?, slug = ? WHERE id = ?")
        .bind(&name)
        .bind(&new_slug)
        .bind(id)
        .execute(pool)
        .await?;

    get_category(pool, id).await
}

/// Deletes a category. The database schema cascades this to join-table rows
/// (page_categories, article_categories), so no explicit cleanup is needed here.
pub async fn delete_category(pool: &SqlitePool, id: &str) -> AppResult<()> {
    // Confirm the category exists before attempting deletion so we return a
    // proper 404 rather than silently deleting zero rows.
    get_category(pool, id).await?;

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Returns `Conflict` if `slug` is already used by a category other than
/// `exclude_id` (pass `None` when creating, `Some(id)` when updating).
async fn ensure_slug_unique(
    pool: &SqlitePool,
    slug: &str,
    exclude_id: Option<&str>,
) -> AppResult<()> {
    let existing_id =
        sqlx::query_scalar::<_, String>("SELECT id FROM categories WHERE slug = ? LIMIT 1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;

    match (existing_id, exclude_id) {
        (None, _) => Ok(()),
        (Some(ref found_id), Some(excluded)) if found_id == excluded => Ok(()),
        _ => Err(AppError::Conflict(format!(
            "A category with slug '{}' already exists",
            slug
        ))),
    }
}
