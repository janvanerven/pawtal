use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub mod models;

/// Creates a SQLite connection pool, runs pending migrations, and configures
/// per-connection pragmas (WAL mode + foreign key enforcement).
///
/// `database_url` should be a SQLite connection string such as
/// `sqlite:data/pawtal.db?mode=rwc`.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Apply all pending migrations from the ./migrations directory.
    // The macro embeds migration files at compile time so the binary is
    // self-contained — no migrations directory needed at runtime.
    sqlx::migrate!("./migrations").run(&pool).await?;

    // WAL mode gives significantly better read/write concurrency for SQLite.
    // This is a database-level setting that persists across connections, but
    // we set it here to ensure it's always active.
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    // Foreign key constraints are opt-in per connection in SQLite.
    // We enforce them unconditionally so the schema's FK definitions are
    // actually respected at runtime.
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}
