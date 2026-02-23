//! Pages CRUD service.
//!
//! All database interactions for the `pages` domain live here. API handlers
//! call these functions and never touch the database directly — a deliberate
//! boundary that keeps handlers thin and business logic testable.

use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{Category, CreatePage, Page, PageRevision, PaginatedResponse,
    PaginationParams, UpdatePage};
use crate::error::{AppError, AppResult};
use crate::helpers::slugify;
use crate::services::audit;

// ─── Public service functions ─────────────────────────────────────────────────

/// Returns a paginated list of pages.
///
/// When `status_filter` is `Some`, only pages with that exact status are
/// returned. When it is `None`, trashed pages are excluded so the default
/// admin view shows all non-deleted content.
pub async fn list_pages(
    pool: &SqlitePool,
    params: &PaginationParams,
    status_filter: Option<&str>,
) -> AppResult<PaginatedResponse<Page>> {
    let per_page = params.per_page() as i64;
    let offset = params.offset() as i64;

    // Two almost-identical queries depending on whether a status is requested.
    // Keeping them as separate query! calls (rather than building SQL strings)
    // means sqlx can type-check them at compile time.
    let (rows, total) = if let Some(status) = status_filter {
        let rows = sqlx::query_as::<_, Page>(
            "SELECT id, title, slug, content, status, publish_at, author_id, \
                    created_at, updated_at, trashed_at \
             FROM pages \
             WHERE status = ? \
             ORDER BY updated_at DESC \
             LIMIT ? OFFSET ?",
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pages WHERE status = ?")
            .bind(status)
            .fetch_one(pool)
            .await?;

        (rows, total)
    } else {
        let rows = sqlx::query_as::<_, Page>(
            "SELECT id, title, slug, content, status, publish_at, author_id, \
                    created_at, updated_at, trashed_at \
             FROM pages \
             WHERE status != 'trashed' \
             ORDER BY updated_at DESC \
             LIMIT ? OFFSET ?",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pages WHERE status != 'trashed'")
                .fetch_one(pool)
                .await?;

        (rows, total)
    };

    Ok(PaginatedResponse {
        data: rows,
        total,
        page: params.page.unwrap_or(1).max(1),
        per_page: params.per_page(),
    })
}

/// Fetches a single page by primary key. Returns `NotFound` if absent.
pub async fn get_page(pool: &SqlitePool, id: &str) -> AppResult<Page> {
    sqlx::query_as::<_, Page>(
        "SELECT id, title, slug, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at \
         FROM pages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Fetches a published page by its slug. Used by the public API — only
/// `published` pages are visible without authentication.
pub async fn get_page_by_slug(pool: &SqlitePool, slug: &str) -> AppResult<Page> {
    sqlx::query_as::<_, Page>(
        "SELECT id, title, slug, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at \
         FROM pages WHERE slug = ? AND status = 'published'",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Creates a new page, including an initial revision and optional category
/// assignments.
///
/// Slug is derived from the title when not explicitly provided. Returns
/// `Conflict` if the derived or supplied slug is already taken.
pub async fn create_page(
    pool: &SqlitePool,
    input: CreatePage,
    author_id: &str,
) -> AppResult<Page> {
    // Derive or validate the slug.
    let slug = input
        .slug
        .as_deref()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| slugify(&input.title));

    ensure_slug_unique(pool, &slug, None).await?;

    let id = Uuid::new_v4().to_string();
    let content = input.content.unwrap_or_default();
    let status = input.status.unwrap_or_else(|| "draft".to_owned());

    sqlx::query(
        "INSERT INTO pages (id, title, slug, content, status, publish_at, author_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&slug)
    .bind(&content)
    .bind(&status)
    .bind(&input.publish_at)
    .bind(author_id)
    .execute(pool)
    .await?;

    // Attach categories when provided.
    if let Some(ref cat_ids) = input.category_ids {
        set_page_categories(pool, &id, cat_ids).await?;
    }

    // Record the initial revision so history starts from creation.
    create_revision(pool, &id, &input.title, &content, author_id).await?;

    audit::log_action(
        pool,
        author_id,
        "create",
        "page",
        &id,
        &json!({ "title": input.title, "slug": slug, "status": status }),
    )
    .await?;

    get_page(pool, &id).await
}

/// Applies a partial update to an existing page.
///
/// Fields absent from `input` keep their current values. A new revision is
/// created for every successful update so the full edit history is preserved.
pub async fn update_page(
    pool: &SqlitePool,
    id: &str,
    input: UpdatePage,
    user_id: &str,
) -> AppResult<Page> {
    let existing = get_page(pool, id).await?;

    // Merge supplied values with existing ones.
    let title = input.title.unwrap_or_else(|| existing.title.clone());
    let content = input.content.unwrap_or_else(|| existing.content.clone());
    let status = input.status.unwrap_or_else(|| existing.status.clone());
    let publish_at = if input.publish_at.is_some() {
        input.publish_at
    } else {
        existing.publish_at
    };

    // Compute the new slug and check uniqueness only when it changed.
    let slug = input.slug.unwrap_or_else(|| existing.slug.clone());
    if slug != existing.slug {
        ensure_slug_unique(pool, &slug, Some(id)).await?;
    }

    sqlx::query(
        "UPDATE pages \
         SET title = ?, slug = ?, content = ?, status = ?, publish_at = ?, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(&title)
    .bind(&slug)
    .bind(&content)
    .bind(&status)
    .bind(&publish_at)
    .bind(id)
    .execute(pool)
    .await?;

    // Update category assignments when explicitly provided.
    if let Some(ref cat_ids) = input.category_ids {
        set_page_categories(pool, id, cat_ids).await?;
    }

    create_revision(pool, id, &title, &content, user_id).await?;

    audit::log_action(
        pool,
        user_id,
        "update",
        "page",
        id,
        &json!({ "title": title, "slug": slug, "status": status }),
    )
    .await?;

    get_page(pool, id).await
}

/// Moves a page to the trash. Does not delete the row — it can be restored.
pub async fn trash_page(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<()> {
    // Verify the page exists before attempting the update.
    get_page(pool, id).await?;

    sqlx::query(
        "UPDATE pages \
         SET status = 'trashed', \
             trashed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(pool, user_id, "trash", "page", id, &json!({})).await?;

    Ok(())
}

/// Restores a trashed page back to `draft` status.
///
/// Returns `BadRequest` when the page is not currently trashed — restoring a
/// published page would silently demote it, which is almost certainly a mistake.
pub async fn restore_page(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<Page> {
    let existing = get_page(pool, id).await?;

    if existing.status != "trashed" {
        return Err(AppError::BadRequest(
            "Only trashed pages can be restored".to_owned(),
        ));
    }

    sqlx::query(
        "UPDATE pages \
         SET status = 'draft', trashed_at = NULL, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(pool, user_id, "restore", "page", id, &json!({})).await?;

    get_page(pool, id).await
}

/// Transitions a page to `published` status.
pub async fn publish_page(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<Page> {
    // Verify the page exists.
    get_page(pool, id).await?;

    sqlx::query(
        "UPDATE pages \
         SET status = 'published', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(pool, user_id, "publish", "page", id, &json!({})).await?;

    get_page(pool, id).await
}

/// Returns all revisions for a page, newest first.
pub async fn list_revisions(pool: &SqlitePool, page_id: &str) -> AppResult<Vec<PageRevision>> {
    // Confirm the page exists so we return a 404 rather than an empty list for
    // a nonexistent page_id.
    get_page(pool, page_id).await?;

    let revisions = sqlx::query_as::<_, PageRevision>(
        "SELECT id, page_id, title, content, author_id, created_at \
         FROM page_revisions \
         WHERE page_id = ? \
         ORDER BY created_at DESC",
    )
    .bind(page_id)
    .fetch_all(pool)
    .await?;

    Ok(revisions)
}

/// Restores a page to the state captured in a specific revision.
///
/// This creates a new revision recording the restore event, rather than
/// removing revisions, so the full history remains intact.
pub async fn restore_revision(
    pool: &SqlitePool,
    page_id: &str,
    revision_id: &str,
    user_id: &str,
) -> AppResult<Page> {
    let revision = sqlx::query_as::<_, PageRevision>(
        "SELECT id, page_id, title, content, author_id, created_at \
         FROM page_revisions \
         WHERE id = ? AND page_id = ?",
    )
    .bind(revision_id)
    .bind(page_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let input = UpdatePage {
        title: Some(revision.title),
        content: Some(revision.content),
        slug: None,
        status: None,
        publish_at: None,
        category_ids: None,
    };

    update_page(pool, page_id, input, user_id).await
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Inserts a revision row for the given page's current title and content.
async fn create_revision(
    pool: &SqlitePool,
    page_id: &str,
    title: &str,
    content: &str,
    author_id: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO page_revisions (id, page_id, title, content, author_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(page_id)
    .bind(title)
    .bind(content)
    .bind(author_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Replaces all category assignments for a page.
///
/// We delete-then-insert rather than diffing, which is simpler and correct for
/// the small cardinalities involved in a CMS category list.
async fn set_page_categories(
    pool: &SqlitePool,
    page_id: &str,
    category_ids: &[String],
) -> AppResult<()> {
    sqlx::query("DELETE FROM page_categories WHERE page_id = ?")
        .bind(page_id)
        .execute(pool)
        .await?;

    for cat_id in category_ids {
        sqlx::query(
            "INSERT INTO page_categories (page_id, category_id) VALUES (?, ?)",
        )
        .bind(page_id)
        .bind(cat_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Fetches the categories associated with a page via the join table.
#[allow(dead_code)] // Used by future article service; kept here for symmetry.
async fn get_page_categories(pool: &SqlitePool, page_id: &str) -> AppResult<Vec<Category>> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT c.id, c.name, c.slug \
         FROM categories c \
         INNER JOIN page_categories pc ON pc.category_id = c.id \
         WHERE pc.page_id = ?",
    )
    .bind(page_id)
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

/// Returns `Conflict` if `slug` is already used by a page other than
/// `exclude_id` (pass `None` when creating, `Some(id)` when updating).
async fn ensure_slug_unique(
    pool: &SqlitePool,
    slug: &str,
    exclude_id: Option<&str>,
) -> AppResult<()> {
    let existing_id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM pages WHERE slug = ? LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    match (existing_id, exclude_id) {
        // No existing row with this slug — unique.
        (None, _) => Ok(()),
        // Existing row belongs to the page we're updating — still unique.
        (Some(ref found_id), Some(excluded)) if found_id == excluded => Ok(()),
        // Slug is taken by a different page.
        _ => Err(AppError::Conflict(format!(
            "A page with slug '{}' already exists",
            slug
        ))),
    }
}
