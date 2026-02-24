mod api;
mod auth;
mod config;
mod db;
mod error;
mod helpers;
mod media;
mod services;
mod tasks;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, State},
    http::{Request, Response, StatusCode},
    middleware::{from_fn, from_fn_with_state},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use tower_http::services::ServeDir;
use serde_json::json;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Shared application state passed to every handler via Axum's `State`
/// extractor. Must be `Clone` — Axum clones it once per request.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: config::Config,
    /// Shared HTTP client for outbound requests (OAuth2, etc.). Reusing a
    /// single client avoids per-request connection pool and TLS overhead.
    pub http_client: reqwest::Client,
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

    // Ensure the uploads directory exists so media handlers can write files
    // immediately without additional setup.
    std::fs::create_dir_all(&config.uploads_dir).unwrap_or_else(|e| {
        panic!(
            "failed to create uploads directory '{}': {e}",
            config.uploads_dir
        );
    });

    let pool = db::create_pool(&config.database_url)
        .await
        .expect("failed to connect to database and run migrations");

    // Capture the port before `config` is moved into AppState, so we can use
    // it when binding the listener below.
    let port = config.port;

    // Address of the SvelteKit Node server. In Docker the entrypoint starts it
    // on port 3000. In local development the Vite dev server on port 5173
    // handles the frontend directly, so this path is only exercised in Docker.
    let frontend_origin = std::env::var("FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    let state = AppState {
        db: pool,
        config,
        http_client: reqwest::Client::new(),
    };

    // ── Route groups ──────────────────────────────────────────────────────────
    //
    // Splitting routes into groups keeps middleware application explicit and
    // makes it easy to see at a glance which routes are protected.

    // 1. Public routes — no authentication required.
    let public_routes = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/pages/{slug}", get(api::pages::public_get_by_slug))
        .route("/api/articles", get(api::articles::public_list))
        .route("/api/articles/{slug}", get(api::articles::public_get_by_slug))
        .route("/api/articles/{slug}/related", get(api::articles::public_related))
        .route("/feed.xml", get(api::feed::atom_feed))
        .route("/api/menus/{name}", get(api::menus::public_get))
        .route("/api/settings/public", get(api::settings::public_get))
        .route("/api/apps", get(api::apps::public_list))
        .route("/api/search", get(api::search::public_search));

    // 2. Auth routes — handle the OAuth2 flow; deliberately unprotected so
    //    unauthenticated users can reach them.
    let auth_routes = Router::new()
        .route("/api/auth/login", get(api::auth::login))
        .route("/api/auth/callback", get(api::auth::callback))
        .route("/api/auth/logout", post(api::auth::logout));

    // 3. Admin routes — protected by the require_auth middleware layer.
    //    Every route added here will require a valid session cookie.

    // Admin-only routes — require both authentication AND the "admin" role.
    let admin_only_routes = Router::new()
        .route("/api/admin/users", get(api::auth::list_users))
        .route(
            "/api/admin/users/{id}/role",
            put(api::auth::update_user_role),
        )
        .route(
            "/api/admin/settings",
            get(api::settings::admin_get).put(api::settings::admin_update),
        )
        .route("/api/admin/trash/empty", post(api::trash::empty))
        .layer(from_fn(auth::middleware::require_admin));

    let admin_routes = Router::new()
        .route("/api/admin/me", get(api::auth::me))
        // Articles CRUD
        .route(
            "/api/admin/articles",
            get(api::articles::admin_list).post(api::articles::admin_create),
        )
        .route(
            "/api/admin/articles/{id}",
            get(api::articles::admin_get)
                .put(api::articles::admin_update)
                .delete(api::articles::admin_delete),
        )
        .route(
            "/api/admin/articles/{id}/publish",
            post(api::articles::admin_publish),
        )
        .route(
            "/api/admin/articles/{id}/restore",
            post(api::articles::admin_restore),
        )
        .route(
            "/api/admin/articles/{id}/revisions",
            get(api::articles::admin_revisions),
        )
        .route(
            "/api/admin/articles/{id}/revisions/{rev_id}/restore",
            post(api::articles::admin_restore_revision),
        )
        // Pages CRUD
        .route(
            "/api/admin/pages",
            get(api::pages::admin_list).post(api::pages::admin_create),
        )
        .route(
            "/api/admin/pages/{id}",
            get(api::pages::admin_get)
                .put(api::pages::admin_update)
                .delete(api::pages::admin_delete),
        )
        .route(
            "/api/admin/pages/{id}/publish",
            post(api::pages::admin_publish),
        )
        .route(
            "/api/admin/pages/{id}/restore",
            post(api::pages::admin_restore),
        )
        .route(
            "/api/admin/pages/{id}/revisions",
            get(api::pages::admin_revisions),
        )
        .route(
            "/api/admin/pages/{id}/revisions/{rev_id}/restore",
            post(api::pages::admin_restore_revision),
        )
        // Categories
        .route(
            "/api/admin/categories",
            get(api::categories::list).post(api::categories::create),
        )
        .route(
            "/api/admin/categories/{id}",
            put(api::categories::update).delete(api::categories::delete),
        )
        // Menus
        .route(
            "/api/admin/menus/{name}",
            get(api::menus::admin_get).put(api::menus::admin_update),
        )
        // Apps — reorder MUST be registered before /{id} to avoid routing ambiguity
        .route(
            "/api/admin/apps/reorder",
            put(api::apps::admin_reorder),
        )
        .route(
            "/api/admin/apps",
            get(api::apps::admin_list).post(api::apps::admin_create),
        )
        .route(
            "/api/admin/apps/{id}",
            get(api::apps::admin_get)
                .put(api::apps::admin_update)
                .delete(api::apps::admin_delete),
        )
        // Trash
        .route("/api/admin/trash", get(api::trash::list))
        // Audit log
        .route("/api/admin/audit-log", get(api::audit::list))
        // Search (admin — includes unpublished content)
        .route("/api/admin/search", get(api::search::admin_search))
        // Media — upload has a 50 MB body size limit
        .route(
            "/api/admin/media",
            get(api::media::admin_list).post(api::media::admin_upload),
        )
        .route(
            "/api/admin/media/{id}",
            delete(api::media::admin_delete),
        )
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50 MB
        .merge(admin_only_routes)
        .layer(from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    // Clone the pool before `state` is moved into the router.
    tasks::spawn_background_tasks(state.db.clone());

    // ServeDir must be nested before `.with_state()` so it is part of the same
    // router tree. We clone `uploads_dir` here because `state` is moved below.
    let uploads_dir = state.config.uploads_dir.clone();

    // Reuse the shared reqwest client from AppState for the SvelteKit proxy.
    let http_client = state.http_client.clone();

    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(admin_routes)
        // Serve uploaded files directly from the filesystem. The path segment
        // after `/uploads/` maps to `{uploads_dir}/{rest}`, so a URL like
        // `/uploads/{id}/thumbnail.webp` maps to `uploads/{id}/thumbnail.webp`.
        .nest_service("/uploads", ServeDir::new(&uploads_dir))
        // All remaining requests (i.e. the SvelteKit frontend) are reverse-
        // proxied to the Node server. This is only active in Docker where
        // FRONTEND_ORIGIN points at the running SvelteKit process on port 3000.
        // In local development Vite's dev server handles the frontend directly.
        .fallback(move |req: Request<Body>| {
            proxy_to_frontend(req, http_client.clone(), frontend_origin.clone())
        })
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

/// Reverse-proxy fallback: forwards every request that didn't match an API or
/// `/uploads` route to the SvelteKit Node server running on `frontend_origin`.
///
/// This is the glue that lets a single container serve both the Rust API and
/// the SvelteKit SSR frontend without an external reverse proxy in front.
///
/// Header forwarding is intentionally selective:
/// - `Host` is NOT forwarded so SvelteKit sees `127.0.0.1:3000` as its host
///   rather than the external hostname (which could trigger CSRF checks).
/// - `X-Forwarded-*` headers are added so SvelteKit can reconstruct the
///   original request URL when building absolute links or redirect targets.
async fn proxy_to_frontend(
    req: Request<Body>,
    client: reqwest::Client,
    frontend_origin: String,
) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let req_headers = req.headers().clone();

    // Reconstruct the target URL: origin + path + optional query string.
    let path_and_query = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let target_url = format!("{frontend_origin}{path_and_query}");

    // Collect the incoming body bytes. For typical frontend requests (HTML,
    // JS, CSS) there is no body, so this is a near-zero-cost no-op.
    // Cap at 10 MB to prevent memory exhaustion via the proxy path.
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            warn!("proxy: failed to read request body: {e}");
            return (StatusCode::BAD_GATEWAY, "bad gateway").into_response();
        }
    };

    let mut proxy_req = client
        .request(
            reqwest::Method::from_bytes(method.as_str().as_bytes())
                .unwrap_or(reqwest::Method::GET),
            &target_url,
        )
        .body(body_bytes);

    // Forward selected headers from the original request so SvelteKit can
    // access cookies (for session-aware SSR) and content negotiation headers.
    let headers_to_forward = ["cookie", "accept", "accept-language", "content-type"];
    for (name, value) in req_headers.iter() {
        if headers_to_forward.contains(&name.as_str()) {
            if let Ok(v) = value.to_str() {
                proxy_req = proxy_req.header(name.as_str(), v);
            }
        }
    }

    match proxy_req.send().await {
        Ok(upstream) => {
            let status = StatusCode::from_u16(upstream.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let mut builder = Response::builder().status(status);

            // Forward upstream response headers (e.g. Content-Type, Set-Cookie,
            // Cache-Control) back to the browser.
            for (name, value) in upstream.headers() {
                // Skip transfer-encoding — hyper will set it correctly for the
                // outgoing response and a duplicate causes decoding errors.
                if name == reqwest::header::TRANSFER_ENCODING {
                    continue;
                }
                builder = builder.header(name, value);
            }

            match upstream.bytes().await {
                Ok(body) => builder
                    .body(Body::from(body))
                    .unwrap_or_else(|_| {
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::empty())
                            .unwrap()
                    })
                    .into_response(),
                Err(e) => {
                    warn!("proxy: failed to read upstream body: {e}");
                    (StatusCode::BAD_GATEWAY, "bad gateway").into_response()
                }
            }
        }
        Err(e) => {
            warn!("proxy: upstream request to {target_url} failed: {e}");
            (StatusCode::BAD_GATEWAY, "frontend unavailable").into_response()
        }
    }
}
