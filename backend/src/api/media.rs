//! HTTP handlers for the media resource.
//!
//! Route map (registered in main.rs):
//!
//!   Admin (require_auth middleware applied at router level):
//!     GET    /api/admin/media         — paginated list with optional filter
//!     POST   /api/admin/media         — multipart file upload
//!     DELETE /api/admin/media/:id     — permanent delete
//!
//!   Static file serving (tower-http ServeDir, registered separately):
//!     GET    /uploads/{id}/{filename} — serves files from the uploads directory

use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::db::models::{Media, PaginatedResponse, PaginationParams, User};
use crate::error::{AppError, AppResult};
use crate::services::media as svc;
use crate::AppState;

// ─── Request types ────────────────────────────────────────────────────────────

/// Optional filter for the media list endpoint.
///
/// `filter` may be:
/// * `"icons"`  — return only icon media
/// * `"images"` — return only regular images
/// * absent     — return all media
#[derive(Debug, Deserialize)]
pub struct MediaFilter {
    pub filter: Option<String>,
}

// ─── Admin endpoints ──────────────────────────────────────────────────────────

/// `GET /api/admin/media`
///
/// Returns a paginated list of media records. An optional `filter` query
/// parameter narrows results to icons or images.
pub async fn admin_list(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(media_filter): Query<MediaFilter>,
) -> AppResult<Json<PaginatedResponse<Media>>> {
    let filter = media_filter.filter.as_deref();
    let result = svc::list_media(&state.db, &pagination, filter).await?;
    Ok(Json(result))
}

/// `POST /api/admin/media`
///
/// Accepts a multipart/form-data upload with the following fields:
/// * `file`    — the file to upload (required)
/// * `is_icon` — `"true"` or `"1"` to mark as an app icon (optional)
///
/// The service layer handles filesystem writes, image processing, and DB
/// insertion. Returns the newly-created media record as JSON.
pub async fn admin_upload(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> AppResult<Json<Media>> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut is_icon = false;

    // Consume all multipart fields before doing any work. Axum's Multipart
    // extractor is a streaming reader, so we must drain it completely before
    // we can use the collected values.
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?
                        .to_vec(),
                );
            }
            "is_icon" => {
                let val = field.text().await.unwrap_or_default();
                is_icon = val == "true" || val == "1";
            }
            // Silently ignore unknown fields — allows future extension without
            // breaking existing clients that send additional metadata.
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| AppError::BadRequest("Missing 'file' field".into()))?;
    let original_filename =
        filename.ok_or_else(|| AppError::BadRequest("Missing filename in upload".into()))?;

    // Fall back to octet-stream when the client does not supply a content type.
    let mime_type = content_type.unwrap_or_else(|| "application/octet-stream".into());

    // Reject empty uploads before hitting the filesystem.
    if data.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".into()));
    }

    let media = svc::upload_media(
        &state.db,
        &state.config.uploads_dir,
        &original_filename,
        &mime_type,
        &data,
        is_icon,
        &user.id,
    )
    .await?;

    Ok(Json(media))
}

/// `DELETE /api/admin/media/:id`
///
/// Permanently deletes the media record and all associated files on disk.
pub async fn admin_delete(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    svc::delete_media(&state.db, &state.config.uploads_dir, &id, &user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
