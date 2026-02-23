use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

pub mod models;

/// Creates a SQLite connection pool, runs pending migrations, and configures
/// per-connection pragmas (WAL mode + foreign key enforcement).
///
/// `database_url` should be a SQLite connection string such as
/// `sqlite:data/pawtal.db?mode=rwc`.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Configure connection options so that WAL mode and foreign keys are set
    // on every connection in the pool, not just the first one.
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Apply all pending migrations from the ./migrations directory.
    // The macro embeds migration files at compile time so the binary is
    // self-contained — no migrations directory needed at runtime.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
