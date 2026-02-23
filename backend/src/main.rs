mod config;
mod db;
mod error;
mod helpers;

use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Shared application state passed to every handler via Axum's `State`
/// extractor. Must be `Clone` — Axum clones it once per request.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    // Load .env before anything else so variables are available to both the
    // tracing filter and Config::from_env().
    dotenvy::dotenv().ok(); // .ok() — missing file is fine in production

    // Structured tracing. Level controlled via RUST_LOG env var; defaults to
    // "info" if unset.
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env();

    // Ensure the directory that will contain the SQLite file exists.
    // The database_url looks like "sqlite:data/pawtal.db?mode=rwc"; we strip
    // the scheme prefix and any query string to get the bare file path.
    let db_path = config
        .database_url
        .strip_prefix("sqlite:")
        .unwrap_or(&config.database_url)
        .split('?')
        .next()
        .unwrap_or("data/pawtal.db");

    if let Some(parent) = std::path::Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            panic!("failed to create database directory '{parent:?}': {e}");
        });
    }

    let pool = db::create_pool(&config.database_url)
        .await
        .expect("failed to connect to database and run migrations");

    // Capture the port before `config` is moved into AppState, so we can use
    // it when binding the listener below.
    let port = config.port;

    let state = AppState { db: pool, config };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

/// `GET /api/health` — liveness probe for load balancers and Docker health
/// checks. Runs a trivial DB query so infrastructure can detect database
/// connectivity issues in addition to process liveness.
async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "db": db_ok
    }))
}
