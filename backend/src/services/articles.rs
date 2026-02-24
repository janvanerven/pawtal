//! Articles CRUD service.
//!
//! All database interactions for the `articles` domain live here. API handlers
//! call these functions and never touch the database directly — a deliberate
//! boundary that keeps handlers thin and business logic testable.
//!
//! Articles mirror pages structurally but carry an additional `short_text`
//! field that is included in every revision snapshot.

use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{Article, ArticleRevision, Category, CreateArticle, PaginatedResponse,
    PaginationParams, UpdateArticle};
use crate::error::{AppError, AppResult};
use crate::helpers::slugify;
use crate::services::audit;

// ─── Public service functions ─────────────────────────────────────────────────

/// Returns a paginated list of articles.
///
/// When `status_filter` is `Some`, only articles with that exact status are
/// returned. When it is `None`, trashed articles are excluded so the default
/// admin view shows all non-deleted content.
pub async fn list_articles(
    pool: &SqlitePool,
    params: &PaginationParams,
    status_filter: Option<&str>,
) -> AppResult<PaginatedResponse<Article>> {
    let per_page = params.per_page() as i64;
    let offset = params.offset() as i64;

    // Two almost-identical queries depending on whether a status is requested.
    // Keeping them as separate query! calls (rather than building SQL strings)
    // means sqlx can type-check them at compile time.
    let (rows, total) = if let Some(status) = status_filter {
        let rows = sqlx::query_as::<_, Article>(
            "SELECT id, title, slug, short_text, content, status, publish_at, author_id, \
                    created_at, updated_at, trashed_at, cover_image_id, reading_time_minutes \
             FROM articles \
             WHERE status = ? \
             ORDER BY updated_at DESC \
             LIMIT ? OFFSET ?",
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM articles WHERE status = ?")
                .bind(status)
                .fetch_one(pool)
                .await?;

        (rows, total)
    } else {
        let rows = sqlx::query_as::<_, Article>(
            "SELECT id, title, slug, short_text, content, status, publish_at, author_id, \
                    created_at, updated_at, trashed_at, cover_image_id, reading_time_minutes \
             FROM articles \
             WHERE status != 'trashed' \
             ORDER BY updated_at DESC \
             LIMIT ? OFFSET ?",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM articles WHERE status != 'trashed'",
        )
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

/// Returns a paginated list of published articles, ordered newest first.
///
/// Intended for the public-facing API — no authentication required. Only
/// `published` articles are included; draft and trashed rows are hidden.
pub async fn list_published_articles(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<Article>> {
    let per_page = params.per_page() as i64;
    let offset = params.offset() as i64;

    let rows = sqlx::query_as::<_, Article>(
        "SELECT id, title, slug, short_text, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at, cover_image_id, reading_time_minutes \
         FROM articles \
         WHERE status = 'published' \
         ORDER BY created_at DESC \
         LIMIT ? OFFSET ?",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM articles WHERE status = 'published'",
    )
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse {
        data: rows,
        total,
        page: params.page.unwrap_or(1).max(1),
        per_page: params.per_page(),
    })
}

/// Fetches a single article by primary key. Returns `NotFound` if absent.
pub async fn get_article(pool: &SqlitePool, id: &str) -> AppResult<Article> {
    sqlx::query_as::<_, Article>(
        "SELECT id, title, slug, short_text, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at, cover_image_id, reading_time_minutes \
         FROM articles WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Fetches a published article by its slug. Used by the public API — only
/// `published` articles are visible without authentication.
pub async fn get_article_by_slug(pool: &SqlitePool, slug: &str) -> AppResult<Article> {
    sqlx::query_as::<_, Article>(
        "SELECT id, title, slug, short_text, content, status, publish_at, author_id, \
                created_at, updated_at, trashed_at, cover_image_id, reading_time_minutes \
         FROM articles WHERE slug = ? AND status = 'published'",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Creates a new article, including an initial revision and optional category
/// assignments.
///
/// Slug is derived from the title when not explicitly provided. Returns
/// `Conflict` if the derived or supplied slug is already taken.
pub async fn create_article(
    pool: &SqlitePool,
    input: CreateArticle,
    author_id: &str,
) -> AppResult<Article> {
    // Derive or validate the slug.
    let slug = input
        .slug
        .as_deref()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| slugify(&input.title));

    ensure_slug_unique(pool, &slug, None).await?;

    let id = Uuid::new_v4().to_string();
    let short_text = input.short_text.unwrap_or_default();
    let content = input.content.unwrap_or_default();
    let status = input.status.unwrap_or_else(|| "draft".to_owned());
    let reading_time = estimate_reading_time(&content);

    sqlx::query(
        "INSERT INTO articles \
             (id, title, slug, short_text, content, status, publish_at, author_id, \
              cover_image_id, reading_time_minutes) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&slug)
    .bind(&short_text)
    .bind(&content)
    .bind(&status)
    .bind(&input.publish_at)
    .bind(author_id)
    .bind(&input.cover_image_id)
    .bind(reading_time)
    .execute(pool)
    .await?;

    // Attach categories when provided.
    if let Some(ref cat_ids) = input.category_ids {
        set_article_categories(pool, &id, cat_ids).await?;
    }

    // Record the initial revision so history starts from creation.
    create_revision(pool, &id, &input.title, &short_text, &content, author_id).await?;

    audit::log_action(
        pool,
        author_id,
        "create",
        "article",
        &id,
        &json!({ "title": input.title, "slug": slug, "status": status }),
    )
    .await?;

    get_article(pool, &id).await
}

/// Applies a partial update to an existing article.
///
/// Fields absent from `input` keep their current values. A new revision is
/// created for every successful update so the full edit history is preserved.
pub async fn update_article(
    pool: &SqlitePool,
    id: &str,
    input: UpdateArticle,
    user_id: &str,
) -> AppResult<Article> {
    let existing = get_article(pool, id).await?;

    // Merge supplied values with existing ones.
    let title = input.title.unwrap_or_else(|| existing.title.clone());
    let short_text = input.short_text.unwrap_or_else(|| existing.short_text.clone());
    let content = input.content.unwrap_or_else(|| existing.content.clone());
    let status = input.status.unwrap_or_else(|| existing.status.clone());
    let publish_at = if input.publish_at.is_some() {
        input.publish_at
    } else {
        existing.publish_at
    };
    // Preserve the existing cover image if a new one was not supplied.
    let cover_image_id = if input.cover_image_id.is_some() {
        input.cover_image_id
    } else {
        existing.cover_image_id.clone()
    };
    let reading_time = estimate_reading_time(&content);

    // Compute the new slug and check uniqueness only when it changed.
    let slug = input.slug.unwrap_or_else(|| existing.slug.clone());
    if slug != existing.slug {
        ensure_slug_unique(pool, &slug, Some(id)).await?;
    }

    sqlx::query(
        "UPDATE articles \
         SET title = ?, slug = ?, short_text = ?, content = ?, status = ?, publish_at = ?, \
             cover_image_id = ?, reading_time_minutes = ?, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(&title)
    .bind(&slug)
    .bind(&short_text)
    .bind(&content)
    .bind(&status)
    .bind(&publish_at)
    .bind(&cover_image_id)
    .bind(reading_time)
    .bind(id)
    .execute(pool)
    .await?;

    // Update category assignments when explicitly provided.
    if let Some(ref cat_ids) = input.category_ids {
        set_article_categories(pool, id, cat_ids).await?;
    }

    create_revision(pool, id, &title, &short_text, &content, user_id).await?;

    audit::log_action(
        pool,
        user_id,
        "update",
        "article",
        id,
        &json!({ "title": title, "slug": slug, "status": status }),
    )
    .await?;

    get_article(pool, id).await
}

/// Moves an article to the trash. Does not delete the row — it can be restored.
pub async fn trash_article(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<()> {
    // Verify the article exists before attempting the update.
    get_article(pool, id).await?;

    sqlx::query(
        "UPDATE articles \
         SET status = 'trashed', \
             trashed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(pool, user_id, "trash", "article", id, &json!({})).await?;

    Ok(())
}

/// Restores a trashed article back to `draft` status.
///
/// Returns `BadRequest` when the article is not currently trashed — restoring a
/// published article would silently demote it, which is almost certainly a mistake.
pub async fn restore_article(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<Article> {
    let existing = get_article(pool, id).await?;

    if existing.status != "trashed" {
        return Err(AppError::BadRequest(
            "Only trashed articles can be restored".to_owned(),
        ));
    }

    sqlx::query(
        "UPDATE articles \
         SET status = 'draft', trashed_at = NULL, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(pool, user_id, "restore", "article", id, &json!({})).await?;

    get_article(pool, id).await
}

/// Transitions an article to `published` status.
pub async fn publish_article(pool: &SqlitePool, id: &str, user_id: &str) -> AppResult<Article> {
    // Verify the article exists.
    get_article(pool, id).await?;

    sqlx::query(
        "UPDATE articles \
         SET status = 'published', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;

    audit::log_action(pool, user_id, "publish", "article", id, &json!({})).await?;

    get_article(pool, id).await
}

/// Returns all revisions for an article, newest first.
pub async fn list_revisions(
    pool: &SqlitePool,
    article_id: &str,
) -> AppResult<Vec<ArticleRevision>> {
    // Confirm the article exists so we return a 404 rather than an empty list
    // for a nonexistent article_id.
    get_article(pool, article_id).await?;

    let revisions = sqlx::query_as::<_, ArticleRevision>(
        "SELECT id, article_id, title, short_text, content, author_id, created_at \
         FROM article_revisions \
         WHERE article_id = ? \
         ORDER BY created_at DESC",
    )
    .bind(article_id)
    .fetch_all(pool)
    .await?;

    Ok(revisions)
}

/// Restores an article to the state captured in a specific revision.
///
/// This creates a new revision recording the restore event, rather than
/// removing revisions, so the full history remains intact.
pub async fn restore_revision(
    pool: &SqlitePool,
    article_id: &str,
    revision_id: &str,
    user_id: &str,
) -> AppResult<Article> {
    let revision = sqlx::query_as::<_, ArticleRevision>(
        "SELECT id, article_id, title, short_text, content, author_id, created_at \
         FROM article_revisions \
         WHERE id = ? AND article_id = ?",
    )
    .bind(revision_id)
    .bind(article_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let input = UpdateArticle {
        title: Some(revision.title),
        short_text: Some(revision.short_text),
        content: Some(revision.content),
        slug: None,
        status: None,
        publish_at: None,
        category_ids: None,
        cover_image_id: None,
    };

    update_article(pool, article_id, input, user_id).await
}

/// Returns published articles that share at least one category with the given
/// article. Used to populate a "related articles" section on article detail pages.
pub async fn get_related_articles(
    pool: &SqlitePool,
    article_id: &str,
    limit: i64,
) -> AppResult<Vec<Article>> {
    let articles = sqlx::query_as::<_, Article>(
        "SELECT DISTINCT a.id, a.title, a.slug, a.short_text, a.content, a.status, \
                a.publish_at, a.author_id, a.created_at, a.updated_at, a.trashed_at, \
                a.cover_image_id, a.reading_time_minutes \
         FROM articles a \
         INNER JOIN article_categories ac ON ac.article_id = a.id \
         WHERE ac.category_id IN (SELECT category_id FROM article_categories WHERE article_id = ?) \
           AND a.id != ? \
           AND a.status = 'published' \
         ORDER BY a.created_at DESC \
         LIMIT ?",
    )
    .bind(article_id)
    .bind(article_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(articles)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Estimates reading time from HTML content at ~200 words per minute.
///
/// Strips HTML tags by walking the characters, then counts whitespace-delimited
/// words. The minimum returned value is 1 minute to avoid showing "0 min read".
fn estimate_reading_time(html: &str) -> i32 {
    let mut in_tag = false;
    let mut text = String::new();
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let word_count = text.split_whitespace().count();
    ((word_count as f64) / 200.0).ceil().max(1.0) as i32
}

/// Inserts a revision row for the given article's current title, short_text,
/// and content.
async fn create_revision(
    pool: &SqlitePool,
    article_id: &str,
    title: &str,
    short_text: &str,
    content: &str,
    author_id: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO article_revisions \
             (id, article_id, title, short_text, content, author_id) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(article_id)
    .bind(title)
    .bind(short_text)
    .bind(content)
    .bind(author_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Replaces all category assignments for an article.
///
/// We delete-then-insert rather than diffing, which is simpler and correct for
/// the small cardinalities involved in a CMS category list.
async fn set_article_categories(
    pool: &SqlitePool,
    article_id: &str,
    category_ids: &[String],
) -> AppResult<()> {
    sqlx::query("DELETE FROM article_categories WHERE article_id = ?")
        .bind(article_id)
        .execute(pool)
        .await?;

    for cat_id in category_ids {
        sqlx::query(
            "INSERT INTO article_categories (article_id, category_id) VALUES (?, ?)",
        )
        .bind(article_id)
        .bind(cat_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Fetches the categories associated with an article via the join table.
#[allow(dead_code)]
async fn get_article_categories(
    pool: &SqlitePool,
    article_id: &str,
) -> AppResult<Vec<Category>> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT c.id, c.name, c.slug \
         FROM categories c \
         INNER JOIN article_categories ac ON ac.category_id = c.id \
         WHERE ac.article_id = ?",
    )
    .bind(article_id)
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

/// Returns `Conflict` if `slug` is already used by an article other than
/// `exclude_id` (pass `None` when creating, `Some(id)` when updating).
async fn ensure_slug_unique(
    pool: &SqlitePool,
    slug: &str,
    exclude_id: Option<&str>,
) -> AppResult<()> {
    let existing_id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM articles WHERE slug = ? LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    match (existing_id, exclude_id) {
        // No existing row with this slug — unique.
        (None, _) => Ok(()),
        // Existing row belongs to the article we're updating — still unique.
        (Some(ref found_id), Some(excluded)) if found_id == excluded => Ok(()),
        // Slug is taken by a different article.
        _ => Err(AppError::Conflict(format!(
            "An article with slug '{}' already exists",
            slug
        ))),
    }
}
