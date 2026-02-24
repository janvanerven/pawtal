//! Atom feed handler.
//!
//! Generates a standards-compliant Atom 1.0 feed from the 20 most recently
//! published articles. Atom is preferred over RSS 2.0 because it has a stable
//! specification with unambiguous date and ID semantics.
//!
//! Route: `GET /feed.xml`

use axum::{
    extract::State,
    http::header,
    response::IntoResponse,
};

use crate::db::models::PaginationParams;
use crate::services::articles as article_svc;
use crate::services::settings;
use crate::AppState;

/// `GET /feed.xml`
///
/// Returns an Atom 1.0 feed of the 20 most recent published articles.
/// The response `Content-Type` is `application/atom+xml; charset=utf-8`.
pub async fn atom_feed(State(state): State<AppState>) -> impl IntoResponse {
    let params = PaginationParams {
        page: Some(1),
        per_page: Some(20),
    };

    let articles = match article_svc::list_published_articles(&state.db, &params).await {
        Ok(r) => r.data,
        Err(_) => vec![],
    };

    let site_settings = settings::get_public_settings(&state.db)
        .await
        .unwrap_or_default();

    let site_title = site_settings
        .get("site_title")
        .cloned()
        .unwrap_or_else(|| "Pawtal".to_string());

    let base_url = state.config.base_url.trim_end_matches('/');

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", escape_xml(&site_title)));
    xml.push_str(&format!(
        "  <link href=\"{}/feed.xml\" rel=\"self\"/>\n",
        base_url
    ));
    xml.push_str(&format!(
        "  <link href=\"{}\" rel=\"alternate\"/>\n",
        base_url
    ));
    xml.push_str(&format!("  <id>{}/</id>\n", base_url));

    if let Some(first) = articles.first() {
        let updated = first.publish_at.unwrap_or(first.created_at);
        xml.push_str(&format!(
            "  <updated>{}</updated>\n",
            updated.to_rfc3339()
        ));
    }

    for article in &articles {
        let published = article.publish_at.unwrap_or(article.created_at);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!(
            "    <title>{}</title>\n",
            escape_xml(&article.title)
        ));
        xml.push_str(&format!(
            "    <link href=\"{}/articles/{}\"/>\n",
            base_url, article.slug
        ));
        xml.push_str(&format!(
            "    <id>{}/articles/{}</id>\n",
            base_url, article.slug
        ));
        xml.push_str(&format!(
            "    <published>{}</published>\n",
            published.to_rfc3339()
        ));
        xml.push_str(&format!(
            "    <updated>{}</updated>\n",
            article.updated_at.to_rfc3339()
        ));
        if !article.short_text.is_empty() {
            xml.push_str(&format!(
                "    <summary>{}</summary>\n",
                escape_xml(&article.short_text)
            ));
        }
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");

    (
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        xml,
    )
}

/// Escapes the five XML special characters so text content is safe to embed
/// directly in element bodies and attribute values.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
