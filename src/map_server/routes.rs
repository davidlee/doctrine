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
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::concept_map;
use crate::map_server::assets;
use crate::map_server::error::MapServerError;
use crate::map_server::markdown;
use crate::map_server::shell::DOT_BODY_LIMIT;
use crate::map_server::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// A single mutation against a concept map's DSL.
#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum MutationAction {
    #[serde(rename = "add_edge")]
    AddEdge {
        source: String,
        rel: String,
        target: String,
    },
    #[serde(rename = "remove_edge")]
    RemoveEdge {
        source: String,
        rel: String,
        target: String,
    },
    #[serde(rename = "rename_node")]
    RenameNode {
        #[serde(alias = "old")]
        old_label: String,
        #[serde(alias = "new")]
        new_label: String,
    },
}

/// A pending concept-map mutation with optional optimistic concurrency hash.
#[derive(Debug, Deserialize)]
struct ConceptMapMutation {
    #[serde(flatten)]
    action: MutationAction,
    #[serde(default)]
    base_hash: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Construct the axum Router with all routes.
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
        .route(
            "/api/concept-map/{id}",
            get(get_concept_map).post(mutate_concept_map),
        )
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
    let dot_result = dot_version().await;
    let dot_ok = dot_result.is_ok();
    let dot_version = dot_result.ok();
    let graph_ok = !state.graph.read().await.nodes.is_empty();
    Json(json!({
        "ok": true,
        "dot": { "ok": dot_ok, "version": dot_version },
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
// Concept-map handlers
// ---------------------------------------------------------------------------

/// `GET /api/concept-map/:id` — return the concept map's nodes, edges, and
/// diagnostics as JSON.
async fn get_concept_map(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Result<impl IntoResponse, MapServerError> {
    let id = concept_map::parse_ref(&id_str)
        .map_err(|_e| MapServerError::BadConceptMapId(id_str.clone()))?;
    let cm_root = state.root.join(concept_map::CONCEPT_MAP_DIR);
    let (doc, toml_text, _body) = concept_map::read_concept_map(&cm_root, id)
        .map_err(|_e| MapServerError::ConceptMapNotFound(id))?;

    // get_dsl errors if the `dsl` key is absent — treat as empty CM
    let (parsed, diagnostics, dsl_hash) = match concept_map::get_dsl(&toml_text) {
        Ok(dsl) => {
            let hash = hex::encode(Sha256::digest(dsl.as_bytes()));
            let parsed = concept_map::parse_dsl(&dsl);
            let mut diagnostics = concept_map::check(&parsed);
            // Merge parse-time diagnostics that check() doesn't carry forward
            // (MalformedLine, EmptyLabel, DuplicateEdge). The CLI run_check
            // does the same merge; keep both in sync.
            for d in &parsed.diagnostics {
                match d {
                    concept_map::ConceptMapDiagnostic::CanonicalNodeCollision { .. }
                    | concept_map::ConceptMapDiagnostic::SelfEdge { .. } => {
                        // Already included by check().
                    }
                    _ => diagnostics.push(d.clone()),
                }
            }
            diagnostics.sort_by_key(concept_map::line_of_diagnostic);
            (parsed, diagnostics, hash)
        }
        Err(_) => {
            return Ok(Json(json!({
                "id": format!("CM-{id:03}"),
                "title": doc.title,
                "status": doc.status,
                "description": doc.description,
                "dsl_hash": "",
                "nodes": [],
                "edges": [],
                "diagnostics": []
            })));
        }
    };

    let nodes: Vec<serde_json::Value> = parsed
        .nodes
        .iter()
        .map(|n| json!({"key": n.key, "label": n.label}))
        .collect();

    let edges: Vec<serde_json::Value> = parsed
        .edges
        .iter()
        .map(|e| {
            json!({
                "from_key": e.from_key,
                "from_label": e.from_label,
                "rel": e.rel,
                "to_key": e.to_key,
                "to_label": e.to_label,
                "line": e.line,
            })
        })
        .collect();

    let diag_list: Vec<serde_json::Value> = diagnostics
        .iter()
        .map(|d| serde_json::to_value(d).unwrap_or(json!({})))
        .collect();

    Ok(Json(json!({
        "id": format!("CM-{id:03}"),
        "title": doc.title,
        "status": doc.status,
        "description": doc.description,
        "dsl_hash": dsl_hash,
        "nodes": nodes,
        "edges": edges,
        "diagnostics": diag_list,
    })))
}

/// `POST /api/concept-map/:id` — apply a mutation (`add_edge`, `remove_edge`,
/// `rename_node`) to the concept map's DSL.
async fn mutate_concept_map(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
    Json(body): Json<ConceptMapMutation>,
) -> Result<impl IntoResponse, MapServerError> {
    let id = concept_map::parse_ref(&id_str)
        .map_err(|_e| MapServerError::BadConceptMapId(id_str.clone()))?;
    let cm_root = state.root.join(concept_map::CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = concept_map::read_concept_map(&cm_root, id)
        .map_err(|_e| MapServerError::ConceptMapNotFound(id))?;
    let old_dsl = concept_map::get_dsl(&toml_text)
        .map_err(|_e| MapServerError::ConceptMapParseError("TOML is missing a `dsl` key".into()))?;

    // Stale-write guard
    if let Some(ref base_hash) = body.base_hash {
        let current_hash = hex::encode(Sha256::digest(old_dsl.as_bytes()));
        if current_hash != *base_hash {
            return Err(MapServerError::StaleConceptMap);
        }
    }

    // Mutate — collect (new_dsl_text, optional rename_occurrences)
    let (new_dsl_text, rename_occurrences) = match &body.action {
        MutationAction::AddEdge {
            source,
            rel,
            target,
        } => {
            let dsl = concept_map::add_edge_to_dsl(&old_dsl, source, rel, target)
                .map_err(MapServerError::from)?;
            (dsl, None)
        }
        MutationAction::RemoveEdge {
            source,
            rel,
            target,
        } => {
            let dsl = concept_map::remove_edge_from_dsl(&old_dsl, source, rel, target)
                .map_err(MapServerError::from)?;
            (dsl, None)
        }
        MutationAction::RenameNode {
            old_label,
            new_label,
        } => {
            let (dsl, count) = concept_map::rename_node_in_dsl(&old_dsl, old_label, new_label)
                .map_err(MapServerError::from)?;
            (dsl, Some(count))
        }
    };

    // Write back via set_dsl
    let updated_toml = concept_map::set_dsl(&toml_text, &new_dsl_text)
        .map_err(|e| MapServerError::ConceptMapParseError(e.to_string()))?;
    let name = format!("{id:03}");
    let stem = format!("concept-map-{name}");
    let toml_path = cm_root.join(&name).join(format!("{stem}.toml"));
    std::fs::write(&toml_path, &updated_toml)
        .map_err(|e| MapServerError::ConceptMapIoError(e.to_string()))?;

    // Re-parse for response — parse the DSL directly (we already have it)
    let fresh_hash = hex::encode(Sha256::digest(new_dsl_text.as_bytes()));
    let parsed = concept_map::parse_dsl(&new_dsl_text);

    let nodes: Vec<serde_json::Value> = parsed
        .nodes
        .iter()
        .map(|n| json!({"key": n.key, "label": n.label}))
        .collect();
    let edges: Vec<serde_json::Value> = parsed
        .edges
        .iter()
        .map(|e| {
            json!({
                "from_key": e.from_key,
                "from_label": e.from_label,
                "rel": e.rel,
                "to_key": e.to_key,
                "to_label": e.to_label,
                "line": e.line,
            })
        })
        .collect();

    let mut resp = json!({
        "ok": true,
        "nodes": nodes,
        "edges": edges,
        "dsl_hash": fresh_hash,
    });
    if let Some(occurrences) = rename_occurrences
        && let Some(obj) = resp.as_object_mut()
    {
        obj.insert("occurrences".into(), json!(occurrences));
    }

    Ok(Json(resp))
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

    // -----------------------------------------------------------------------
    // Concept-map route tests
    // -----------------------------------------------------------------------

    /// Seed a concept map entity for route tests.
    fn seed_concept_map(root: &std::path::Path, id: u32, dsl: &str) {
        use crate::catalog::test_helpers::write;
        let name = format!("{id:03}");
        let stem = format!("concept-map-{name}");
        let toml = format!(
            "id = {id}\nslug = \"cm{id}\"\ntitle = \"Test Map {id}\"\nstatus = \"draft\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ndescription = \"\"\ndsl = '''\n{dsl}
'''\n",
        );
        let cm_root = std::path::Path::new(".doctrine/concept-map");
        write(
            root,
            &format!("{}/{}/{}.toml", cm_root.display(), name, stem),
            &toml,
        );
        write(
            root,
            &format!("{}/{}/{}.md", cm_root.display(), name, stem),
            "# Concept Map\n",
        );
        // Create slug symlink
        let link = root.join(cm_root).join(format!("{name}-cm{id}"));
        let _ = std::os::unix::fs::symlink(&name, &link);
    }

    /// Create a temp dir + seeded CM app.
    async fn seeded_cm_app(dsl: &str) -> (tempfile::TempDir, Router) {
        let root = crate::catalog::test_helpers::tmp();
        let root_path = root.path().to_path_buf();
        // Seed supporting entities for the catalog graph
        crate::catalog::test_helpers::seed_slice(&root_path, 1, &[]);
        crate::catalog::test_helpers::seed_adr(&root_path, 1, &[]);
        crate::catalog::test_helpers::seed_requirement(&root_path, 1);
        // Seed CM-001
        seed_concept_map(&root_path, 1, dsl);
        let app = super::super::tests::test_app(&root_path).await;
        (root, app)
    }

    // -- GET concept map --

    #[tokio::test]
    async fn get_concept_map_existing_returns_200() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let (status, headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["id"], "CM-001");
        assert_eq!(parsed["title"], "Test Map 1");
        assert_eq!(parsed["status"], "draft");
        assert!(!parsed["dsl_hash"].as_str().unwrap().is_empty());
        // Nodes
        let nodes = parsed["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0]["key"], "user");
        assert_eq!(nodes[1]["key"], "document");
        // Edges
        let edges = parsed["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0]["from_key"], "user");
        assert_eq!(edges[0]["rel"], "creates");
        assert_eq!(edges[0]["to_key"], "document");
        // Diagnostics
        let diags = parsed["diagnostics"].as_array().unwrap();
        assert!(
            diags.is_empty(),
            "clean map should have no diagnostics, got: {diags:?}"
        );
    }

    #[tokio::test]
    async fn get_concept_map_nonexistent_returns_404() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-999", None)).await;
        assert_eq!(status, 404);
        assert!(body.contains("not_found"));
        assert!(body.contains("CM-999"));
    }

    #[tokio::test]
    async fn get_concept_map_bad_id_returns_400() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/garbage", None)).await;
        assert_eq!(status, 400);
        assert!(body.contains("bad_concept_map_id"));
    }

    #[tokio::test]
    async fn get_concept_map_no_dsl_returns_200_empty() {
        // Seed without a `dsl` key — that means empty concept map.
        let root = crate::catalog::test_helpers::tmp();
        let root_path = root.path().to_path_buf();
        crate::catalog::test_helpers::seed_slice(&root_path, 1, &[]);
        // Manually seed CM-001 without a `dsl` key
        {
            let cm_dir = root_path.join(".doctrine/concept-map/001");
            std::fs::create_dir_all(&cm_dir).unwrap();
            std::fs::write(
                cm_dir.join("concept-map-001.toml"),
                "id = 1\nslug = \"cm1\"\ntitle = \"Empty Map\"\nstatus = \"draft\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ndescription = \"\"\n",
            )
            .unwrap();
            std::fs::write(cm_dir.join("concept-map-001.md"), "# Empty\n").unwrap();
            let _ =
                std::os::unix::fs::symlink("001", root_path.join(".doctrine/concept-map/001-cm1"));
        }
        let app = super::super::tests::test_app(&root_path).await;
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        assert_eq!(status, 200);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["dsl_hash"], "");
        assert!(parsed["nodes"].as_array().unwrap().is_empty());
        assert!(parsed["edges"].as_array().unwrap().is_empty());
    }

    // -- POST mutate concept map --

    #[tokio::test]
    async fn mutate_add_edge_returns_200() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(
            r#"{"action":"add_edge","source":"Document","rel":"belongs to","target":"Workspace"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200, "body: {body_str}");
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["ok"], true);
        let edges = parsed["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 2);
    }

    #[tokio::test]
    async fn mutate_add_edge_duplicate_returns_409() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(
            r#"{"action":"add_edge","source":"User","rel":"creates","target":"Document"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 409, "body: {body_str}");
        assert!(body_str.contains("duplicate_edge"));
    }

    #[tokio::test]
    async fn mutate_add_edge_empty_field_returns_400() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body =
            Body::from(r#"{"action":"add_edge","source":"","rel":"creates","target":"Document"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 400, "body: {body_str}");
        assert!(body_str.contains("empty_field"));
    }

    #[tokio::test]
    async fn mutate_remove_edge_returns_200() {
        let (_root, app) = seeded_cm_app("User > creates > Document\nDoc > relates > Note").await;
        let body = Body::from(
            r#"{"action":"remove_edge","source":"Doc","rel":"relates","target":"Note"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200, "body: {body_str}");
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["edges"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn mutate_remove_edge_not_found_returns_404() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(
            r#"{"action":"remove_edge","source":"Ghost","rel":"haunts","target":"House"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 404, "body: {body_str}");
        assert!(body_str.contains("edge_not_found"));
    }

    #[tokio::test]
    async fn mutate_rename_node_returns_200() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(r#"{"action":"rename_node","old_label":"User","new_label":"Actor"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200, "body: {body_str}");
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["ok"], true);
        // Verify the response contains the renamed node
        let nodes = parsed["nodes"].as_array().unwrap();
        assert!(nodes.iter().any(|n| n["label"] == "Actor"));
        assert!(!nodes.iter().any(|n| n["label"] == "User"));
        let edges = parsed["edges"].as_array().unwrap();
        assert!(edges.iter().any(|e| e["from_label"] == "Actor"));
    }

    #[tokio::test]
    async fn mutate_rename_node_persists_to_disk() {
        let (root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(r#"{"action":"rename_node","old_label":"User","new_label":"Actor"}"#);
        let (status, _headers, _body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200);
        // Read the TOML file directly
        let toml_content = std::fs::read_to_string(
            root.path()
                .join(".doctrine/concept-map/001/concept-map-001.toml"),
        )
        .unwrap();
        assert!(
            toml_content.contains("Actor > creates > Document"),
            "TOML should contain renamed node, got:\n{toml_content}"
        );
    }

    #[tokio::test]
    async fn mutate_rename_node_collision_returns_409() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        // Rename "User" to "Document" — same name but should collide on keys
        let body =
            Body::from(r#"{"action":"rename_node","old_label":"User","new_label":"Document"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 409, "body: {body_str}");
        assert!(body_str.contains("node_collision"));
    }

    #[tokio::test]
    async fn mutate_stale_write_returns_409() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(
            r#"{"action":"add_edge","source":"Doc","rel":"uses","target":"Note","base_hash":"deadbeef"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 409, "body: {body_str}");
        assert!(body_str.contains("stale_concept_map"));
    }

    #[tokio::test]
    async fn mutate_stale_write_correct_hash_succeeds() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        // First, get the current hash
        let (_, _, get_body) = send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        let parsed: serde_json::Value = serde_json::from_str(&get_body).unwrap();
        let current_hash = parsed["dsl_hash"].as_str().unwrap();

        // Now POST with the correct hash
        let body = Body::from(format!(
            r#"{{"action":"add_edge","source":"Doc","rel":"uses","target":"Note","base_hash":"{}"}}"#,
            current_hash
        ));
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200, "body: {body_str}");
        let p: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(p["ok"], true);
        assert_eq!(p["edges"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn mutate_unknown_action_returns_422() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(r#"{"action":"fly_to_moon"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        // axum returns 422 Unprocessable Entity for deserialization failures
        assert_eq!(status, 422, "body: {body_str}");
        assert!(body_str.contains("fly_to_moon"));
    }

    #[tokio::test]
    async fn entity_markdown_cm001_returns_200() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let (status, headers, body) =
            send(&app, json_req("GET", "/api/entity/CM-001/markdown", None)).await;
        assert_eq!(status, 200);
        assert!(
            headers["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/markdown")
        );
        assert_eq!(body, "# Concept Map\n");
    }

    // -----------------------------------------------------------------------
    // Adversarial: labels with special chars in GET/POST
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_concept_map_with_quoted_labels() {
        let (_root, app) = seeded_cm_app("\"Hello\" > relates to > \"World\"").await;
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        assert_eq!(status, 200);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let edges = parsed["edges"].as_array().unwrap();
        assert_eq!(edges[0]["from_label"], "\"Hello\"");
        assert_eq!(edges[0]["to_label"], "\"World\"");
    }

    #[tokio::test]
    async fn post_add_edge_with_special_chars() {
        let (_root, app) = seeded_cm_app("A > rel > B").await;
        let body = Body::from(
            r#"{"action":"add_edge","source":"\"quoted\"","rel":"uses","target":"target"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200, "body: {body_str}");
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        let edges = parsed["edges"].as_array().unwrap();
        assert!(edges.iter().any(|e| e["from_label"] == "\"quoted\""));
    }

    // -----------------------------------------------------------------------
    // Malformed DSL returns diagnostics, not 500
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_concept_map_malformed_dsl_returns_200_no_panic() {
        // Malformed DSL lines produce parse-time diagnostics (MalformedLine,
        // EmptyLabel). These are now merged with check() output in the handler
        // (the same merge the CLI run_check performs).
        let (_root, app) = seeded_cm_app("User creates Document").await;
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        assert_eq!(status, 200);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        // Nodes/edges are empty for malformed lines (no valid edges parsed)
        assert!(parsed["nodes"].as_array().unwrap().is_empty());
        assert!(parsed["edges"].as_array().unwrap().is_empty());
        // Parse-time diagnostics are now merged into the response
        let diags = parsed["diagnostics"].as_array().unwrap();
        assert!(
            !diags.is_empty(),
            "malformed DSL should produce diagnostics"
        );
        // Externally tagged enum: {"MalformedLine": {"line": 1, "text": "..."}}
        let has_malformed = diags.iter().any(|d| d.get("MalformedLine").is_some());
        assert!(has_malformed, "should include MalformedLine diagnostic");
        // no crash — server handled it gracefully
    }

    #[tokio::test]
    async fn get_concept_map_malformed_dsl_empty_source_returns_200() {
        // Empty source label produces parse-time EmptyLabel diagnostic.
        // These are now merged with check() output.
        let (_root, app) = seeded_cm_app(" > rel > Target").await;
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        assert_eq!(status, 200);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(parsed["edges"].as_array().unwrap().is_empty());
        let diags = parsed["diagnostics"].as_array().unwrap();
        assert!(!diags.is_empty(), "empty label should produce diagnostics");
        let has_empty = diags.iter().any(|d| d.get("EmptyLabel").is_some());
        assert!(has_empty, "should include EmptyLabel diagnostic");
    }

    // -----------------------------------------------------------------------
    // POST with unknown action / garbage
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn mutate_garbage_body_returns_400() {
        // Axum's Json extractor rejects non-JSON bodies before our handler runs.
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from("not-even-json");
        let (status, _headers, _body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 400, "garbage body should be rejected gracefully");
    }

    // -----------------------------------------------------------------------
    // File I/O errors: invalid TOML does not panic
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_concept_map_invalid_toml_does_not_panic() {
        // Create a valid CM first so scan_catalog succeeds, then corrupt the TOML.
        // The handler maps read_concept_map errors to 404 (ConceptMapNotFound).
        // verify the server survives the request without panicking.
        let root = crate::catalog::test_helpers::tmp();
        let root_path = root.path().to_path_buf();
        crate::catalog::test_helpers::seed_slice(&root_path, 1, &[]);
        crate::catalog::test_helpers::seed_adr(&root_path, 1, &[]);
        crate::catalog::test_helpers::seed_requirement(&root_path, 1);
        seed_concept_map(&root_path, 1, "A > rel > B");
        let app = super::super::tests::test_app(&root_path).await;
        // Now corrupt the TOML on disk — invalidate after app creation
        std::fs::write(
            root_path.join(".doctrine/concept-map/001/concept-map-001.toml"),
            "id = 1\nslug = \"cm1\"\ntitle = \"Bad\"\nstatus = \"draft\"\ndescription = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ndsl = '''\nunclosed\n",
        ).unwrap();
        // Primary assertion: server must not panic
        let (status, _headers, body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        // read_concept_map maps all errors to 404; verify server survived.
        assert!(status == 404, "should not crash; body: {body}");
    }

    #[tokio::test]
    async fn server_serves_after_internal_error() {
        // Create valid CM first, start app, then corrupt → first request errors,
        // second request (healthy endpoint) must still work.
        let root = crate::catalog::test_helpers::tmp();
        let root_path = root.path().to_path_buf();
        crate::catalog::test_helpers::seed_slice(&root_path, 1, &[]);
        crate::catalog::test_helpers::seed_adr(&root_path, 1, &[]);
        crate::catalog::test_helpers::seed_requirement(&root_path, 1);
        seed_concept_map(&root_path, 1, "A > rel > B");
        let app = super::super::tests::test_app(&root_path).await;
        std::fs::write(
            root_path.join(".doctrine/concept-map/001/concept-map-001.toml"),
            "id = 1\nslug = \"cm1\"\ntitle = \"Bad\"\nstatus = \"draft\"\ndescription = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ndsl = '''\nunclosed\n",
        ).unwrap();

        let (status, _headers, _body) =
            send(&app, json_req("GET", "/api/concept-map/CM-001", None)).await;
        assert_eq!(status, 404);
        // Second request: health check must still work
        let (status2, _headers, _body2) = send(&app, json_req("GET", "/api/health", None)).await;
        assert_eq!(status2, 200);
    }

    // -----------------------------------------------------------------------
    // Edge case: whitespace-only fields
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn mutate_add_edge_whitespace_only_source_returns_400() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(
            r#"{"action":"add_edge","source":"   ","rel":"creates","target":"Document"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 400, "body: {body_str}");
        assert!(body_str.contains("empty_field"));
    }

    #[tokio::test]
    async fn mutate_add_edge_whitespace_only_rel_returns_400() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body =
            Body::from(r#"{"action":"add_edge","source":"User","rel":"\t","target":"Document"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 400, "body: {body_str}");
        assert!(body_str.contains("empty_field"));
    }

    #[tokio::test]
    async fn mutate_remove_edge_whitespace_only_source_returns_400() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(
            r#"{"action":"remove_edge","source":"   ","rel":"creates","target":"Document"}"#,
        );
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 400, "body: {body_str}");
        assert!(body_str.contains("empty_field"));
    }

    #[tokio::test]
    async fn mutate_rename_node_whitespace_only_old_label_returns_400() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(r#"{"action":"rename_node","old_label":"   ","new_label":"Actor"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 400, "body: {body_str}");
        assert!(body_str.contains("empty_field"));
    }

    // -----------------------------------------------------------------------
    // POST rename_node to same label
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn mutate_rename_node_text_identical_returns_200() {
        let (_root, app) = seeded_cm_app("User > creates > Document").await;
        let body = Body::from(r#"{"action":"rename_node","old_label":"User","new_label":"User"}"#);
        let (status, _headers, body_str) = send(
            &app,
            json_req("POST", "/api/concept-map/CM-001", Some(body)),
        )
        .await;
        assert_eq!(status, 200, "body: {body_str}");
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["ok"], true);
        let occ = parsed["occurrences"].as_u64();
        assert!(occ.is_some(), "occurrences field should be present");
        assert_eq!(
            occ.unwrap(),
            0,
            "text-identical rename should have 0 occurrences"
        );
    }
}
