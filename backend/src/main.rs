mod config;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

    let app = Router::new().route("/api/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

/// `GET /api/health` — liveness probe for load balancers and Docker health
/// checks. Returns 200 OK with a static JSON body.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
