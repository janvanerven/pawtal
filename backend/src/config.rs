use serde::Deserialize;

/// Application configuration loaded from environment variables via `envy`.
///
/// All fields map directly to environment variable names (uppercased by envy).
/// Provide defaults via `.env` for local development; override via real env
/// vars in production / Docker.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// TCP port the HTTP server listens on.
    #[serde(default = "default_port")]
    pub port: u16,

    /// SQLite connection string, e.g. `sqlite:data/pawtal.db?mode=rwc`.
    pub database_url: String,

    /// Filesystem path where uploaded media files are stored.
    pub uploads_dir: String,

    /// OAuth2 client ID registered with the identity provider (Authentik).
    pub oauth2_client_id: String,

    /// OAuth2 client secret.
    pub oauth2_client_secret: String,

    /// Base URL of the OAuth2 issuer (Authentik instance), used for OIDC
    /// discovery and token exchange.
    pub oauth2_issuer_url: String,

    /// Secret used to sign/verify session tokens. Must be long and random in
    /// production — treat it like a password.
    pub session_secret: String,

    /// Public base URL of this application, e.g. `https://your-domain.com`.
    /// Used when constructing OAuth2 redirect URIs.
    pub base_url: String,
}

fn default_port() -> u16 {
    8080
}

impl Config {
    /// Load configuration from the current process environment.
    ///
    /// Panics with a clear message if any required variable is missing.
    pub fn from_env() -> Self {
        envy::from_env::<Config>().unwrap_or_else(|err| {
            panic!("configuration error: {err}\nCheck that all required environment variables are set (see .env.example).");
        })
    }
}
