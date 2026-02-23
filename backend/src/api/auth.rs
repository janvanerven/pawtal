//! HTTP handlers for the OAuth2 / OIDC authentication flow and user management.
//!
//! Flow:
//!   GET  /api/auth/login    → redirect browser to IdP
//!   GET  /api/auth/callback → exchange code, upsert user, set cookie, redirect to /admin
//!   POST /api/auth/logout   → clear cookie, delete session, redirect to /
//!   GET  /api/admin/me      → return current user (protected by require_auth middleware)
//!
//! User management (admin-only):
//!   GET  /api/admin/users
//!   PUT  /api/admin/users/:id/role

use axum::{
    extract::{Extension, Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::oauth2::{build_auth_url, discover_oidc, exchange_code, fetch_userinfo};
use crate::auth::session::{create_session, delete_session};
use crate::db::models::User;
use crate::error::{AppError, AppResult};
use crate::AppState;

/// Query parameters sent back by the IdP to our callback endpoint.
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// `GET /api/auth/login`
///
/// Discovers the OIDC provider endpoints, builds the authorization URL, and
/// redirects the browser to the IdP's login page. The CSRF `state` parameter
/// is stored in a short-lived HttpOnly cookie and verified in the callback.
pub async fn login(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let client = reqwest::Client::new();

    let discovery = discover_oidc(&client, &state.config.oauth2_issuer_url).await?;

    let csrf_state = Uuid::new_v4().to_string();
    let auth_url = build_auth_url(&discovery, &state.config, &csrf_state);

    // Store the CSRF state in a short-lived HttpOnly cookie so we can verify
    // it when the IdP redirects back to our callback endpoint.
    let state_cookie = format!(
        "pawtal_oauth_state={}; HttpOnly; SameSite=Lax; Path=/api/auth/callback; Max-Age=600",
        csrf_state
    );

    let response = Response::builder()
        .status(axum::http::StatusCode::FOUND)
        .header(header::LOCATION, auth_url)
        .header(header::SET_COOKIE, state_cookie)
        .body(axum::body::Body::empty())
        .map_err(|e| AppError::Internal(format!("Failed to build redirect response: {e}")))?;

    Ok(response)
}

/// `GET /api/auth/callback`
///
/// Receives the authorization code from the IdP, exchanges it for tokens,
/// fetches user info, upserts the user record, creates a session, and sets
/// an HttpOnly session cookie before redirecting to the admin area.
pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
    headers: axum::http::HeaderMap,
) -> AppResult<impl IntoResponse> {
    // Verify the CSRF state parameter matches the cookie we set during login.
    let stored_state = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|pair| {
                let pair = pair.trim();
                pair.split_once('=')
                    .filter(|(name, _)| name.trim() == "pawtal_oauth_state")
                    .map(|(_, value)| value.trim().to_string())
            })
        })
        .ok_or_else(|| AppError::BadRequest("Missing OAuth state cookie".into()))?;

    if params.state != stored_state {
        return Err(AppError::BadRequest("OAuth state mismatch".into()));
    }
    let client = reqwest::Client::new();

    let discovery = discover_oidc(&client, &state.config.oauth2_issuer_url).await?;

    // Exchange the authorization code for tokens.
    let tokens = exchange_code(&client, &discovery, &state.config, &params.code).await?;

    // Fetch the user's identity claims.
    let userinfo = fetch_userinfo(&client, &discovery, &tokens.access_token).await?;

    // Derive a display name: prefer `name`, fall back to `preferred_username`,
    // then to the `sub` identifier so the field is never empty.
    let display_name = userinfo
        .name
        .or(userinfo.preferred_username)
        .unwrap_or_else(|| userinfo.sub.clone());

    let email = userinfo.email.unwrap_or_default();

    // Determine the role: the very first user gets "admin", everyone after gets "editor".
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;
    let role = if user_count == 0 { "admin" } else { "editor" };

    // Upsert the user. On conflict (same external_id) we update mutable fields
    // so changes on the IdP side (new email, display name) are reflected here.
    // We use RETURNING to get the internal UUID without a second query.
    let user_id = sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO users (id, external_id, email, display_name, role, last_login)
        VALUES (?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        ON CONFLICT(external_id) DO UPDATE SET
            email        = excluded.email,
            display_name = excluded.display_name,
            last_login   = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        RETURNING id
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&userinfo.sub)
    .bind(&email)
    .bind(&display_name)
    .bind(role)
    .fetch_one(&state.db)
    .await?;

    // Create a session and get the token that will become the cookie value.
    let session_token = create_session(&state.db, &user_id).await?;

    // Build an HttpOnly, SameSite=Strict cookie.
    // Max-Age=604800 matches the 7-day TTL used in session creation.
    let cookie = format!(
        "pawtal_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=604800",
        session_token
    );

    // Clear the OAuth state cookie now that it has been verified.
    let clear_state_cookie =
        "pawtal_oauth_state=; HttpOnly; SameSite=Lax; Path=/api/auth/callback; Max-Age=0";

    // Redirect to the admin UI. The Set-Cookie headers are carried alongside the
    // redirect response — browsers apply cookies before following the redirect.
    let response = Response::builder()
        .status(axum::http::StatusCode::FOUND)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .header(header::SET_COOKIE, clear_state_cookie)
        .body(axum::body::Body::empty())
        .map_err(|e| AppError::Internal(format!("Failed to build redirect response: {e}")))?;

    Ok(response)
}

/// `POST /api/auth/logout`
///
/// Deletes the server-side session and clears the cookie by sending an expired
/// replacement, then redirects to the home page.
pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> AppResult<impl IntoResponse> {
    // Extract the session cookie if present so we can delete the server-side
    // record. If the cookie is absent (already logged out), we still redirect
    // cleanly — there's nothing to clean up.
    if let Some(cookie_header) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for pair in cookie_header.split(';') {
            let pair = pair.trim();
            if let Some(("pawtal_session", token)) = pair.split_once('=') {
                // Best-effort — if this fails (e.g. session already gone) we
                // still want to clear the cookie and redirect.
                let _ = delete_session(&state.db, token.trim()).await;
                break;
            }
        }
    }

    // Overwrite the cookie with an expired one to force the browser to delete it.
    let clear_cookie = "pawtal_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0";

    let response = Response::builder()
        .status(axum::http::StatusCode::FOUND)
        .header(header::LOCATION, "/")
        .header(header::SET_COOKIE, clear_cookie)
        .body(axum::body::Body::empty())
        .map_err(|e| AppError::Internal(format!("Failed to build redirect response: {e}")))?;

    Ok(response)
}

/// `GET /api/admin/me`
///
/// Returns the currently authenticated user's profile.
/// Protected by `require_auth` middleware which injects `User` into extensions.
pub async fn me(Extension(user): Extension<User>) -> Json<User> {
    Json(user)
}

// ─── User management endpoints ────────────────────────────────────────────────

/// Request body for `PUT /api/admin/users/:id/role`.
#[derive(Debug, Deserialize)]
pub struct RoleUpdate {
    pub role: String,
}

/// `GET /api/admin/users`
///
/// Returns all registered users. Admin-only.
pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<User>>> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, external_id, email, display_name, role, created_at, last_login \
         FROM users \
         ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}

/// `PUT /api/admin/users/:id/role`
///
/// Changes the role of a user. Only "admin" and "editor" are valid roles.
/// The authenticated admin cannot demote themselves to prevent lockout.
pub async fn update_user_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Path(id): Path<String>,
    Json(body): Json<RoleUpdate>,
) -> AppResult<Json<User>> {
    // Validate the role before touching the database.
    if body.role != "admin" && body.role != "editor" {
        return Err(AppError::BadRequest(format!(
            "Invalid role '{}'. Must be 'admin' or 'editor'.",
            body.role
        )));
    }

    // Prevent an admin from demoting themselves — they'd be locked out of the
    // admin area immediately and would need direct database access to recover.
    if id == current_user.id && body.role != current_user.role {
        return Err(AppError::BadRequest(
            "You cannot change your own role.".to_owned(),
        ));
    }

    let updated = sqlx::query_as::<_, User>(
        "UPDATE users SET role = ? WHERE id = ? \
         RETURNING id, external_id, email, display_name, role, created_at, last_login",
    )
    .bind(&body.role)
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(updated))
}
