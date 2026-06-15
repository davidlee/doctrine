// SPDX-License-Identifier: GPL-3.0-only
//! Map-server HTTP routes — axum `Router` + all 7 handlers (SL-072 PHASE-05).
//!
//! Engine-tier (ADR-001): thin wrappers over `catalog`/`assets`/`markdown`/`shell`.
//! No duplicated graph policy or entity semantics in route handlers.

use std::sync::Arc;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::header,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;

use crate::map_server::assets;
use crate::map_server::error::MapServerError;
use crate::map_server::markdown;
use crate::map_server::shell::DOT_BODY_LIMIT;
use crate::map_server::state::AppState;

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Construct the axum Router with all routes.
#[cfg_attr(not(test), expect(dead_code, reason = "consumed in PHASE-06 (serve)"))]
pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .route("/vendor/{*path}", get(vendor_asset))
        .route("/api/health", get(health))
        .route("/api/graph", get(graph))
        .route("/api/refresh", post(refresh))
        .route(
            "/api/dot/svg",
            post(dot_svg).layer(DefaultBodyLimit::max(DOT_BODY_LIMIT)),
        )
        .route("/api/entity/{id}/markdown", get(entity_markdown))
        .with_state(Arc::new(state))
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

async fn index() -> impl IntoResponse {
    #[expect(clippy::expect_used, reason = "index.html is embedded at build time")]
    let asset = assets::Assets::get("index.html").expect("index.html is embedded");
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        asset.data.to_vec(),
    )
}

async fn asset(Path(path): Path<String>) -> Result<impl IntoResponse, MapServerError> {
    assets::serve_embedded(&path)
}

async fn vendor_asset(Path(path): Path<String>) -> Result<impl IntoResponse, MapServerError> {
    let full_path = format!("vendor/{path}");
    assets::serve_embedded(&full_path)
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let dot_ok = dot_version().await.is_ok();
    let graph_ok = !state.graph.read().await.nodes.is_empty();
    Json(json!({
        "ok": true,
        "dot": { "ok": dot_ok },
        "graph": { "ok": graph_ok }
    }))
}

async fn dot_version() -> Result<String, MapServerError> {
    use std::process::Stdio;
    let child = tokio::process::Command::new("dot")
        .arg("-V")
        .stdout(Stdio::null())
        .stderr(Stdio::piped()) // graphviz prints version to stderr
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MapServerError::ToolUnavailable { tool: "dot" },
            _ => MapServerError::Other(e.into()),
        })?;
    let output = tokio::time::timeout(std::time::Duration::from_secs(2), child.wait_with_output())
        .await
        .map_err(|_elapsed| MapServerError::Timeout { command: "dot" })?
        .map_err(|e| MapServerError::Other(e.into()))?;

    if !output.status.success() {
        return Err(MapServerError::CommandFailed {
            command: "dot",
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    String::from_utf8(output.stderr)
        .map(|s| s.trim().to_owned())
        .map_err(|e| MapServerError::Other(e.into()))
}

async fn graph(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snapshot = state.graph.read().await.clone();
    Json(snapshot)
}

async fn refresh(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, MapServerError> {
    let catalog =
        crate::catalog::hydrate::scan_catalog(&state.root).map_err(MapServerError::Other)?;
    let g = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
    *state.graph.write().await = g;
    Ok(Json(json!({"ok": true})))
}

async fn dot_svg(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<impl IntoResponse, MapServerError> {
    if body.len() > DOT_BODY_LIMIT {
        return Err(MapServerError::BodyTooLarge);
    }
    let svg = state.dot_renderer.render_svg(&body).await?;
    Ok((
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        svg,
    ))
}

async fn entity_markdown(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MapServerError> {
    let (kind_ref, num) = crate::integrity::parse_canonical_ref(&id)
        .map_err(|_e| MapServerError::BadEntityId(id.clone()))?;
    let key = crate::catalog::scan::EntityKey {
        prefix: kind_ref.kind.prefix,
        id: num,
    };
    let graph = state.graph.read().await;
    let node_exists = graph
        .nodes
        .contains_key(&crate::catalog::graph::NodeKey::Entity(key));
    drop(graph);
    if !node_exists {
        return Err(MapServerError::EntityNotFound(id));
    }
    let body = markdown::read_entity_markdown(&state.root, &key).await?;
    Ok((
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        body,
    ))
}

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// Helper: send a request to the test app and collect the response.
    async fn send(
        app: &Router,
        req: axum::http::Request<Body>,
    ) -> (axum::http::StatusCode, axum::http::HeaderMap, String) {
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        (status, headers, String::from_utf8_lossy(&body).to_string())
    }

    fn json_req(method: &str, uri: &str, body: Option<Body>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder().method(method).uri(uri);
        if let Some(b) = body {
            builder = builder.header("content-type", "application/json");
            builder.body(b).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
    }

    /// Create a test app from the given root path.
    async fn fixture_app(root_path: &std::path::Path) -> Router {
        // Seed entities for the graph
        crate::catalog::test_helpers::seed_slice(root_path, 1, &[]);
        crate::catalog::test_helpers::seed_adr(root_path, 1, &[]);
        // Add a requirement for REQ-001 → 501 test
        crate::catalog::test_helpers::seed_requirement(root_path, 1);
        // Add ASM-001 for memory kind test
        crate::catalog::test_helpers::seed_knowledge(
            root_path,
            "ASM",
            1,
            "Test Assumption",
            "active",
        );
        super::super::tests::test_app(root_path).await
    }

    /// Convenience: create a temp dir + seeded app, returning both so the
    /// `TempDir` lives as long as the `Router` needs the files on disk.
    async fn seeded_app() -> (tempfile::TempDir, Router) {
        let root = crate::catalog::test_helpers::tmp();
        let app = fixture_app(root.path()).await;
        (root, app)
    }

    #[tokio::test]
    async fn index_returns_200_html() {
        let (status, headers, _body) =
            send(&seeded_app().await.1, json_req("GET", "/", None)).await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
    }

    #[tokio::test]
    async fn missing_asset_returns_404() {
        let (status, _headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/assets/nonexistent.js", None),
        )
        .await;
        assert_eq!(status, 404);
        assert!(body.contains("asset_not_found"));
    }

    #[tokio::test]
    async fn graph_returns_200_valid_json() {
        let (status, headers, body) =
            send(&seeded_app().await.1, json_req("GET", "/api/graph", None)).await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(parsed.get("nodes").is_some(), "missing nodes key");
        assert!(parsed.get("edges").is_some(), "missing edges key");
    }

    #[tokio::test]
    async fn refresh_returns_200_ok() {
        let app = seeded_app().await.1;
        let (status, _headers, body) =
            send(&app, json_req("POST", "/api/refresh", Some(Body::empty()))).await;
        assert_eq!(status, 200);
        assert!(body.contains("\"ok\":true"));

        // Graph is still accessible after refresh.
        let (status2, _, _) = send(&app, json_req("GET", "/api/graph", None)).await;
        assert_eq!(status2, 200);
    }

    #[tokio::test]
    async fn entity_markdown_sl001_returns_200() {
        let (status, headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/api/entity/SL-001/markdown", None),
        )
        .await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/markdown")
        );
        assert_eq!(body, "scope\n");
    }

    #[tokio::test]
    async fn entity_markdown_not_in_graph_returns_404() {
        let (status, _headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/api/entity/SL-999/markdown", None),
        )
        .await;
        assert_eq!(status, 404);
        assert!(body.contains("entity_not_found"));
        assert!(body.contains("SL-999"));
    }

    #[tokio::test]
    async fn entity_markdown_lowercase_prefix_returns_400() {
        let (status, _headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/api/entity/sl-001/markdown", None),
        )
        .await;
        assert_eq!(status, 400);
        assert!(body.contains("bad_entity_id"));
    }

    #[tokio::test]
    async fn entity_markdown_bogus_prefix_returns_400() {
        let (status, _headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/api/entity/BOGUS-001/markdown", None),
        )
        .await;
        assert_eq!(status, 400);
        assert!(body.contains("bad_entity_id"));
    }

    #[tokio::test]
    async fn entity_markdown_req001_returns_501() {
        let (status, _headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/api/entity/REQ-001/markdown", None),
        )
        .await;
        assert_eq!(status, 501);
        assert!(body.contains("markdown_not_implemented"));
    }

    #[tokio::test]
    async fn entity_markdown_asm001_returns_200() {
        let (status, headers, body) = send(
            &seeded_app().await.1,
            json_req("GET", "/api/entity/ASM-001/markdown", None),
        )
        .await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/markdown")
        );
        assert_eq!(body, "body\n");
    }

    #[tokio::test]
    async fn health_returns_200() {
        let (status, headers, body) =
            send(&seeded_app().await.1, json_req("GET", "/api/health", None)).await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["ok"], json!(true));
        assert!(parsed.get("dot").is_some());
        assert!(parsed.get("graph").is_some());
    }

    #[tokio::test]
    async fn dot_svg_valid_input_returns_200() {
        let body = Body::from("digraph { a -> b }");
        let (status, headers, body_str) = send(
            &seeded_app().await.1,
            json_req("POST", "/api/dot/svg", Some(body)),
        )
        .await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("image/svg+xml")
        );
        assert_eq!(body_str, "<svg></svg>");
    }

    #[tokio::test]
    async fn dot_svg_body_too_large_returns_413() {
        // 1 MiB + 1 byte exceeds DOT_BODY_LIMIT
        let big = vec![b'x'; DOT_BODY_LIMIT + 1];
        let body = Body::from(big);
        let (status, _headers, _body_str) = send(
            &seeded_app().await.1,
            json_req("POST", "/api/dot/svg", Some(body)),
        )
        .await;
        // DefaultBodyLimit on the route rejects oversized bodies before
        // the handler's check fires, so axum returns its own 413 response.
        assert_eq!(status, 413);
    }

    #[tokio::test]
    async fn dot_svg_tool_unavailable_returns_503() {
        use std::sync::Arc;

        use tokio::sync::RwLock;

        use crate::map_server::shell::{FakeDotMode, FakeDotRenderer};

        let root = crate::catalog::test_helpers::tmp();
        let root_path = root.path().to_path_buf();
        crate::catalog::test_helpers::seed_slice(&root_path, 1, &[]);

        let catalog = crate::catalog::hydrate::scan_catalog(&root_path).expect("scan");
        let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
        let state = AppState {
            root: root_path,
            graph: Arc::new(RwLock::new(graph)),
            dot_renderer: Arc::new(FakeDotRenderer {
                mode: FakeDotMode::ToolUnavailable,
            }),
        };
        let app = router(state);

        let body = Body::from("digraph { a -> b }");
        let (status, _headers, body_str) =
            send(&app, json_req("POST", "/api/dot/svg", Some(body))).await;
        assert_eq!(status, 503);
        assert!(body_str.contains("tool_unavailable"));
    }

    #[tokio::test]
    async fn entity_in_graph_but_md_missing_returns_404() {
        let root = crate::catalog::test_helpers::tmp();
        let root_path = root.path().to_path_buf();
        // Seed both files so scan succeeds (read_slice requires the .md).
        crate::catalog::test_helpers::seed_slice(&root_path, 1, &[]);

        // Build the app while the .md exists.
        let app = super::super::tests::test_app(&root_path).await;

        // Now remove the .md — entity is still in the in-memory graph,
        // but the file read returns EntityNotFound.
        std::fs::remove_file(root_path.join(".doctrine/slice/001/slice-001.md")).unwrap();

        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/entity/SL-001/markdown", None)).await;
        assert_eq!(status, 404);
        assert!(body.contains("entity_not_found"));
        assert!(body.contains("SL-001"));
    }
}
