//! Axum middleware extractors for authentication and authorization.
//!
//! Both middleware functions follow the same pattern:
//!   1. Extract the `pawtal_session` cookie from the `Cookie` header.
//!   2. Validate the token against the database.
//!   3. On success, inject the `User` into request extensions so downstream
//!      handlers can retrieve it via `Extension<User>`.
//!   4. On failure, short-circuit with the appropriate error response.
//!
//! Usage in router setup:
//! ```rust
//! Router::new()
//!     .route("/api/admin/me", get(me))
//!     .layer(from_fn_with_state(state.clone(), require_auth))
//! ```

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::auth::session::validate_session;
use crate::error::AppError;
use crate::AppState;

/// Extracts the `pawtal_session` cookie value from the raw `Cookie` header.
///
/// The Cookie header is a semicolon-separated list of `name=value` pairs.
/// We look for the specific cookie name and return its value if found.
fn extract_session_cookie(request: &Request) -> Option<String> {
    let cookie_header = request.headers().get("cookie")?.to_str().ok()?;

    // Parse "name=value; name2=value2; ..." pairs
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some((name, value)) = pair.split_once('=') {
            if name.trim() == "pawtal_session" {
                return Some(value.trim().to_string());
            }
        }
    }

    None
}

/// Middleware that requires a valid session.
///
/// On success the authenticated `User` is inserted into request extensions,
/// making it available to handlers via `Extension<User>`.
/// On failure returns a 401 Unauthorized response.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_session_cookie(&request).ok_or(AppError::Unauthorized)?;

    let user = validate_session(&state.db, &token).await?;

    // Insert the user into request extensions so handlers can retrieve it
    // with `Extension<User>` without hitting the database again.
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Middleware that requires a valid session *and* the `admin` role.
///
/// On success the authenticated `User` is inserted into request extensions.
/// On failure returns 401 if unauthenticated or 403 if authenticated but
/// lacking the admin role.
pub async fn require_admin(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_session_cookie(&request).ok_or(AppError::Unauthorized)?;

    let user = validate_session(&state.db, &token).await?;

    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}
