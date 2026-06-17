// SPDX-License-Identifier: GPL-3.0-only
//! Asset serving — `RustEmbed` + content-type mapping (SL-072 PHASE-02).
//!
//! PHASE-02 scaffolding — types consumed in PHASE-03+.
#![allow(
    dead_code,
    clippy::same_name_method,
    reason = "PHASE-02 foundation + RustEmbed derive conflict"
)]

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[cfg_attr(debug_assertions, folder = "web/map/")]
#[cfg_attr(not(debug_assertions), folder = "web/map/dist/")]
pub(crate) struct Assets;

/// Map file extension → MIME content-type.
/// Covers the six asset types the map server ships.
pub(crate) fn content_type_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// Serve an embedded asset from the `RustEmbed` store.
/// Returns `AssetNotFound` for missing paths.
pub(crate) fn serve_embedded(
    path: &str,
) -> Result<(axum::http::HeaderMap, Vec<u8>), crate::map_server::error::MapServerError> {
    let ct = content_type_for(path);
    let asset = Assets::get(path)
        .ok_or_else(|| crate::map_server::error::MapServerError::AssetNotFound(path.to_owned()))?;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(ct),
    );
    Ok((headers, asset.data.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_known_extensions() {
        assert_eq!(
            content_type_for("app.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(content_type_for("style.css"), "text/css; charset=utf-8");
        assert_eq!(content_type_for("index.html"), "text/html; charset=utf-8");
        assert_eq!(content_type_for("graph.svg"), "image/svg+xml");
        assert_eq!(content_type_for("data.json"), "application/json");
        assert_eq!(content_type_for("font.woff2"), "font/woff2");
    }

    #[test]
    fn content_type_unknown_extension() {
        assert_eq!(content_type_for("file.bin"), "application/octet-stream");
    }

    #[test]
    fn content_type_no_extension() {
        assert_eq!(content_type_for("README"), "application/octet-stream");
    }

    #[test]
    fn assets_get_index_html() {
        // index.html is embedded as a placeholder
        let asset = Assets::get("index.html");
        assert!(asset.is_some(), "index.html should be embedded");
    }

    #[test]
    fn serve_embedded_index_returns_html() {
        let result = serve_embedded("index.html");
        assert!(result.is_ok(), "index.html should be found");
        let (headers, body) = result.unwrap();
        assert_eq!(
            headers.get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        assert!(!body.is_empty());
    }

    #[test]
    fn serve_embedded_missing_returns_asset_not_found() {
        let result = serve_embedded("nonexistent.js");
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::map_server::error::MapServerError::AssetNotFound(path) => {
                assert_eq!(path, "nonexistent.js");
            }
            other => panic!("expected AssetNotFound, got {:?}", other),
        }
    }
}
