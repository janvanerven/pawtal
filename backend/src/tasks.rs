use sqlx::SqlitePool;
use std::time::Duration;

/// Spawns a long-lived tokio task that wakes up every 60 seconds and runs
/// all scheduled maintenance work. Errors are logged but never fatal — a
/// transient DB hiccup should not take the server down.
pub fn spawn_background_tasks(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = run_scheduled_tasks(&pool).await {
                tracing::error!("Background task error: {:?}", e);
            }
        }
    });
}

/// Runs all periodic maintenance queries against the database.
///
/// Each step is intentionally independent: a failure in one query returns
/// early, but the next invocation (60 s later) will retry cleanly.
async fn run_scheduled_tasks(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 1. Publish scheduled pages whose publish_at time has passed.
    let published_pages = sqlx::query(
        "UPDATE pages SET status = 'published', updated_at = datetime('now')
         WHERE status = 'scheduled' AND publish_at <= datetime('now')",
    )
    .execute(pool)
    .await?;
    if published_pages.rows_affected() > 0 {
        tracing::info!("Published {} scheduled pages", published_pages.rows_affected());
    }

    // 2. Publish scheduled articles whose publish_at time has passed.
    let published_articles = sqlx::query(
        "UPDATE articles SET status = 'published', updated_at = datetime('now')
         WHERE status = 'scheduled' AND publish_at <= datetime('now')",
    )
    .execute(pool)
    .await?;
    if published_articles.rows_affected() > 0 {
        tracing::info!(
            "Published {} scheduled articles",
            published_articles.rows_affected()
        );
    }

    // 3. Permanently delete trashed content older than 30 days.
    //    The 30-day window gives admins a reasonable recovery window without
    //    letting the database grow unbounded.
    let deleted_pages = sqlx::query(
        "DELETE FROM pages WHERE status = 'trashed' AND trashed_at < datetime('now', '-30 days')",
    )
    .execute(pool)
    .await?;
    let deleted_articles = sqlx::query(
        "DELETE FROM articles WHERE status = 'trashed' AND trashed_at < datetime('now', '-30 days')",
    )
    .execute(pool)
    .await?;
    if deleted_pages.rows_affected() + deleted_articles.rows_affected() > 0 {
        tracing::info!(
            "Cleaned up {} pages and {} articles from trash",
            deleted_pages.rows_affected(),
            deleted_articles.rows_affected()
        );
    }

    // 4. Remove expired sessions so the sessions table stays lean.
    sqlx::query("DELETE FROM sessions WHERE expires_at < datetime('now')")
        .execute(pool)
        .await?;

    Ok(())
}
