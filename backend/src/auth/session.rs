//! Session lifecycle management backed by SQLite.
//!
//! Sessions are identified by a randomly generated UUID v4 token stored in an
//! HttpOnly cookie. The token is stored hashed… actually stored plaintext here
//! because SQLite is local and the `session_secret` from config is used for
//! signing elsewhere; keeping this simple and auditable is the right tradeoff
//! for an on-premises CMS where the DB isn't publicly reachable.
//!
//! Sessions expire after 7 days. Expired session cleanup can be triggered on a
//! schedule or opportunistically.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::User;
use crate::error::{AppError, AppResult};

/// How long a session lives before it expires (7 days in seconds).
const SESSION_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Creates a new session for the given user and returns the session token.
///
/// The token is a UUID v4 string. It is stored directly in the `sessions` table
/// and returned to the caller to be set as a cookie value.
pub async fn create_session(pool: &SqlitePool, user_id: &str) -> AppResult<String> {
    let session_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, token, expires_at, created_at)
        VALUES (
            ?,
            ?,
            ?,
            datetime('now', ? || ' seconds'),
            datetime('now')
        )
        "#,
    )
    .bind(&session_id)
    .bind(user_id)
    .bind(&token)
    .bind(SESSION_TTL_SECONDS.to_string())
    .execute(pool)
    .await?;

    Ok(token)
}

/// Validates a session token and returns the associated `User`.
///
/// Returns `AppError::Unauthorized` if the token does not exist or has expired.
/// Expired sessions are left in the database; use `cleanup_expired_sessions` to
/// remove them in bulk.
pub async fn validate_session(pool: &SqlitePool, token: &str) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.external_id, u.email, u.display_name, u.role,
               u.created_at, u.last_login
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token = ?
          AND s.expires_at > datetime('now')
        "#,
    )
    .bind(token)
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
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;

    Ok(())
}

/// Removes all sessions whose `expires_at` is in the past.
///
/// Returns the number of rows deleted so callers can log meaningful metrics.
/// Safe to run on a timer — SQLite's WAL mode means this won't block readers.
pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> AppResult<u64> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
