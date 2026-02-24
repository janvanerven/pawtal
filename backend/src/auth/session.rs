//! Session lifecycle management backed by SQLite.
//!
//! Sessions are identified by a randomly generated UUID v4 token stored in an
//! HttpOnly cookie. The token is hashed with SHA-256 before storage so that a
//! database leak does not directly expose usable session credentials.
//!
//! Sessions expire after 7 days. Expired session cleanup can be triggered on a
//! schedule or opportunistically.

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::User;
use crate::error::{AppError, AppResult};

/// Returns the hex-encoded SHA-256 hash of a session token.
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// How long a session lives before it expires (7 days in seconds).
const SESSION_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Creates a new session for the given user and returns the session token.
///
/// The token is a UUID v4 string. It is stored directly in the `sessions` table
/// and returned to the caller to be set as a cookie value.
pub async fn create_session(pool: &SqlitePool, user_id: &str) -> AppResult<String> {
    let session_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&token);

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, token, expires_at, created_at)
        VALUES (
            ?,
            ?,
            ?,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ? || ' seconds'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        )
        "#,
    )
    .bind(&session_id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(SESSION_TTL_SECONDS.to_string())
    .execute(pool)
    .await?;

    // Return the raw token for the cookie; only the hash is stored.
    Ok(token)
}

/// Validates a session token and returns the associated `User`.
///
/// Returns `AppError::Unauthorized` if the token does not exist or has expired.
/// Expired sessions are left in the database; use `cleanup_expired_sessions` to
/// remove them in bulk.
pub async fn validate_session(pool: &SqlitePool, token: &str) -> AppResult<User> {
    let token_hash = hash_token(token);

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.external_id, u.email, u.display_name, u.role,
               u.created_at, u.last_login
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token = ?
          AND s.expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(user)
}

/// Deletes a session by token. Used during logout.
///
/// Silently succeeds if the token doesn't exist (already expired or never
/// created) — the end result is the same: no active session.
pub async fn delete_session(pool: &SqlitePool, token: &str) -> AppResult<()> {
    let token_hash = hash_token(token);

    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(&token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

