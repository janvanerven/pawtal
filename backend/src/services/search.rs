//! Full-text search service backed by SQLite FTS5.
//!
//! Three virtual tables — `pages_fts`, `articles_fts`, and `apps_fts` — are
//! content tables that mirror their respective source tables. Queries use the
//! FTS5 `MATCH` operator with a trailing `*` for prefix matching, `snippet()`
//! for highlighted excerpts, and `bm25()` for relevance ranking.
//!
//! The caller controls which entity types to search and whether unpublished
//! content should be included (admin vs public use cases).

use sqlx::{Row, SqlitePool};

use crate::db::models::SearchResult;
use crate::error::AppResult;

/// Searches published (and optionally unpublished) content across all entity
/// types or a specific subset.
///
/// `query` is the raw text entered by the user. A trailing `*` is appended
/// internally so partial words match (e.g. "rust" matches "rustacean").
///
/// `search_type` filters results to a single entity kind:
///   - `"pages"`    — static pages only
///   - `"articles"` — blog articles only
///   - `"apps"`     — app directory entries only
///   - `None`       — all three kinds
///
/// `include_unpublished` exposes draft/archived content; set to `true` only
/// for authenticated admin callers.
pub async fn search(
    pool: &SqlitePool,
    query: &str,
    search_type: Option<&str>,
    include_unpublished: bool,
) -> AppResult<Vec<SearchResult>> {
    // Guard against empty queries — FTS5 treats an empty string as a syntax
    // error, and returning nothing is the clearest UX response.
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut results: Vec<SearchResult> = Vec::new();

    // Append `*` for prefix matching so users get results while still typing.
    let fts_query = format!("{}*", query.trim());

    // ── Pages ─────────────────────────────────────────────────────────────────

    if search_type.is_none() || search_type == Some("pages") {
        // The snippet() call highlights the matched term in the second indexed
        // column (index 1 = content) using HTML <mark> tags. The FTS5 rowid
        // equals the rowid of the source `pages` row, so a direct JOIN works.
        let sql = if include_unpublished {
            "SELECT p.id, p.title, p.slug,
                    snippet(pages_fts, 1, '<mark>', '</mark>', '...', 32) AS snippet
             FROM pages_fts
             JOIN pages p ON p.rowid = pages_fts.rowid
             WHERE pages_fts MATCH ?
               AND p.trashed_at IS NULL
             ORDER BY bm25(pages_fts)
             LIMIT 20"
        } else {
            "SELECT p.id, p.title, p.slug,
                    snippet(pages_fts, 1, '<mark>', '</mark>', '...', 32) AS snippet
             FROM pages_fts
             JOIN pages p ON p.rowid = pages_fts.rowid
             WHERE pages_fts MATCH ?
               AND p.status = 'published'
               AND p.trashed_at IS NULL
             ORDER BY bm25(pages_fts)
             LIMIT 20"
        };

        let rows = sqlx::query(sql)
            .bind(&fts_query)
            .fetch_all(pool)
            .await?;

        for row in rows {
            results.push(SearchResult {
                result_type: "page".to_string(),
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                snippet: row.get("snippet"),
            });
        }
    }

    // ── Articles ──────────────────────────────────────────────────────────────

    if search_type.is_none() || search_type == Some("articles") {
        // articles_fts indexes: 0 = title, 1 = short_text, 2 = content.
        // Snippet is drawn from column 2 (content) for richer context.
        let sql = if include_unpublished {
            "SELECT a.id, a.title, a.slug,
                    snippet(articles_fts, 2, '<mark>', '</mark>', '...', 32) AS snippet
             FROM articles_fts
             JOIN articles a ON a.rowid = articles_fts.rowid
             WHERE articles_fts MATCH ?
               AND a.trashed_at IS NULL
             ORDER BY bm25(articles_fts)
             LIMIT 20"
        } else {
            "SELECT a.id, a.title, a.slug,
                    snippet(articles_fts, 2, '<mark>', '</mark>', '...', 32) AS snippet
             FROM articles_fts
             JOIN articles a ON a.rowid = articles_fts.rowid
             WHERE articles_fts MATCH ?
               AND a.status = 'published'
               AND a.trashed_at IS NULL
             ORDER BY bm25(articles_fts)
             LIMIT 20"
        };

        let rows = sqlx::query(sql)
            .bind(&fts_query)
            .fetch_all(pool)
            .await?;

        for row in rows {
            results.push(SearchResult {
                result_type: "article".to_string(),
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                snippet: row.get("snippet"),
            });
        }
    }

    // ── Apps ──────────────────────────────────────────────────────────────────

    if search_type.is_none() || search_type == Some("apps") {
        // Apps have no publish/draft lifecycle, so no status filter is applied.
        // apps_fts indexes: 0 = name, 1 = description. Snippet from description.
        let rows = sqlx::query(
            "SELECT a.id, a.name AS title, '' AS slug,
                    snippet(apps_fts, 1, '<mark>', '</mark>', '...', 32) AS snippet
             FROM apps_fts
             JOIN apps a ON a.rowid = apps_fts.rowid
             WHERE apps_fts MATCH ?
             ORDER BY bm25(apps_fts)
             LIMIT 20",
        )
        .bind(&fts_query)
        .fetch_all(pool)
        .await?;

        for row in rows {
            results.push(SearchResult {
                result_type: "app".to_string(),
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                snippet: row.get("snippet"),
            });
        }
    }

    Ok(results)
}
