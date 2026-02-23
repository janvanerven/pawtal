//! Media upload and management service.
//!
//! Uploaded files land in `{uploads_dir}/{media_id}/` — one directory per
//! media record. That structure makes deletion trivially safe (`remove_dir_all`)
//! and keeps variants co-located with the original.
//!
//! Image MIME types trigger the processing pipeline, which produces resized
//! variants in both the original format and WebP. Non-image files (e.g. PDF,
//! video) are stored as-is with no variant generation.

use std::path::Path;

use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{Media, PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult};
use crate::media::processing;
use crate::services::audit;

// ─── Column list shared by all SELECT queries ─────────────────────────────────

const MEDIA_COLS: &str =
    "id, filename, original_filename, mime_type, size_bytes, width, height, \
     alt_text, is_icon, uploaded_by, created_at";

// ─── Public service functions ─────────────────────────────────────────────────

/// Returns a paginated list of media records ordered by `created_at DESC`.
///
/// `filter` controls which subset is returned:
/// * `Some("icons")`  — only icon media (`is_icon = 1`)
/// * `Some("images")` — only regular images (`is_icon = 0`)
/// * `None`           — all records
pub async fn list_media(
    pool: &SqlitePool,
    params: &PaginationParams,
    filter: Option<&str>,
) -> AppResult<PaginatedResponse<Media>> {
    let per_page = params.per_page() as i64;
    let offset = params.offset() as i64;

    // Build the WHERE clause from the filter parameter. We keep this explicit
    // rather than constructing arbitrary SQL so the query planner can still
    // cache the prepared statement shape.
    let (where_clause, count_query) = match filter {
        Some("icons") => (
            format!("SELECT {MEDIA_COLS} FROM media WHERE is_icon = 1 ORDER BY created_at DESC LIMIT ? OFFSET ?"),
            "SELECT COUNT(*) FROM media WHERE is_icon = 1".to_string(),
        ),
        Some("images") => (
            format!("SELECT {MEDIA_COLS} FROM media WHERE is_icon = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?"),
            "SELECT COUNT(*) FROM media WHERE is_icon = 0".to_string(),
        ),
        _ => (
            format!("SELECT {MEDIA_COLS} FROM media ORDER BY created_at DESC LIMIT ? OFFSET ?"),
            "SELECT COUNT(*) FROM media".to_string(),
        ),
    };

    let rows = sqlx::query_as::<_, Media>(&where_clause)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(&count_query)
        .fetch_one(pool)
        .await?;

    Ok(PaginatedResponse {
        data: rows,
        total,
        page: params.page.unwrap_or(1).max(1),
        per_page: params.per_page(),
    })
}

/// Stores an uploaded file and, for images, generates resized variants.
///
/// Steps:
/// 1. Allocate a UUID as the record ID.
/// 2. Create `{uploads_dir}/{id}/` to hold the original and all variants.
/// 3. Write the raw bytes to disk as the original file.
/// 4. If the MIME type is an image, run the processing pipeline.
/// 5. Insert the media row into the database.
/// 6. Write an audit log entry.
/// 7. Return the freshly-inserted record.
pub async fn upload_media(
    pool: &SqlitePool,
    uploads_dir: &str,
    original_filename: &str,
    mime_type: &str,
    data: &[u8],
    is_icon: bool,
    user_id: &str,
) -> AppResult<Media> {
    let id = Uuid::new_v4().to_string();

    // Derive a safe filename from the original. Reject empty names up-front.
    let safe_name = sanitize_filename(original_filename);
    if safe_name.is_empty() {
        return Err(AppError::BadRequest("Invalid filename".into()));
    }

    // Create the per-record directory.
    let record_dir = Path::new(uploads_dir).join(&id);
    std::fs::create_dir_all(&record_dir).map_err(|e| {
        AppError::Internal(format!("Failed to create media directory: {}", e))
    })?;

    // Write the original file.
    let original_path = record_dir.join(&safe_name);
    std::fs::write(&original_path, data).map_err(|e| {
        AppError::Internal(format!("Failed to write uploaded file: {}", e))
    })?;

    // Process image variants when the MIME type indicates an image.
    let (width, height) = if is_image_mime(mime_type) {
        let variants = if is_icon {
            processing::get_icon_variants()
        } else {
            processing::get_standard_variants()
        };

        // Image processing is CPU-bound; spawn_blocking keeps the async executor
        // free for other requests while the heavy lifting runs on a thread pool.
        let original_path_clone = original_path.clone();
        let record_dir_clone = record_dir.clone();
        let (w, h) = tokio::task::spawn_blocking(move || {
            processing::process_image(&original_path_clone, &record_dir_clone, &variants)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Image processing task panicked: {}", e)))??;

        (Some(w as i32), Some(h as i32))
    } else {
        (None, None)
    };

    let size_bytes = data.len() as i64;

    sqlx::query(
        "INSERT INTO media \
         (id, filename, original_filename, mime_type, size_bytes, width, height, \
          alt_text, is_icon, uploaded_by) \
         VALUES (?, ?, ?, ?, ?, ?, ?, '', ?, ?)",
    )
    .bind(&id)
    .bind(&safe_name)
    .bind(original_filename)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(width)
    .bind(height)
    .bind(is_icon)
    .bind(user_id)
    .execute(pool)
    .await?;

    audit::log_action(
        pool,
        user_id,
        "upload",
        "media",
        &id,
        &json!({
            "filename": safe_name,
            "mime_type": mime_type,
            "size_bytes": size_bytes,
            "is_icon": is_icon,
        }),
    )
    .await?;

    get_media(pool, &id).await
}

/// Permanently deletes a media record and all associated files on disk.
///
/// Steps:
/// 1. Verify the record exists (returns `NotFound` if absent).
/// 2. Remove the entire per-record directory from disk.
/// 3. Delete the database row.
/// 4. Write an audit log entry.
pub async fn delete_media(
    pool: &SqlitePool,
    uploads_dir: &str,
    id: &str,
    user_id: &str,
) -> AppResult<()> {
    let media = get_media(pool, id).await?;

    // Best-effort file removal. If the directory is already gone (manual cleanup,
    // previous partial delete) we log a warning but do not fail the request — the
    // database row deletion below is what matters for consistency.
    let record_dir = Path::new(uploads_dir).join(id);
    if record_dir.exists() {
        std::fs::remove_dir_all(&record_dir).map_err(|e| {
            AppError::Internal(format!("Failed to delete media files: {}", e))
        })?;
    } else {
        tracing::warn!(media_id = %id, "Media directory not found during delete; skipping filesystem removal");
    }

    sqlx::query("DELETE FROM media WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    audit::log_action(
        pool,
        user_id,
        "delete",
        "media",
        id,
        &json!({ "filename": media.filename }),
    )
    .await?;

    Ok(())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Fetches a single media record by ID. Returns `NotFound` if absent.
async fn get_media(pool: &SqlitePool, id: &str) -> AppResult<Media> {
    sqlx::query_as::<_, Media>(&format!(
        "SELECT {MEDIA_COLS} FROM media WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Returns `true` for MIME types that the image processing pipeline supports.
fn is_image_mime(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg"
            | "image/jpg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/bmp"
            | "image/tiff"
    )
}

/// Strips path components and characters that would be unsafe in a filename,
/// preserving the original extension. The result is always lowercase for
/// consistent filesystem behaviour on case-sensitive and case-insensitive
/// mounts alike.
fn sanitize_filename(name: &str) -> String {
    // Take only the last component — prevents directory traversal if the client
    // sends something like `../../etc/passwd`.
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(name);

    // Keep alphanumerics, dots, hyphens, and underscores; replace anything
    // else with an underscore. This covers common problem chars: spaces,
    // colons, angle brackets, etc.
    base.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}
