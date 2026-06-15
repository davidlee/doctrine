# SL-072 Design: Doctrine Map Server

## 1. Architecture & Module Layout

### Tier placement (ADR-001)

```
Command tier:
  src/main.rs           → Map { command: MapCommand } variant
  src/commands/map.rs   → clap Args, root detection, calls map_server::serve

Engine tier:
  src/map_server/mod.rs     → pub async fn serve(Config) — bind, print URL, serve
  src/map_server/state.rs   → AppState { root, graph, dot_renderer }
  src/map_server/routes.rs  → axum Router + thin handlers
  src/map_server/assets.rs  → #[derive(RustEmbed)] + content-type mapping
  src/map_server/shell.rs   → DotRenderer trait + RealDotRenderer impl
  src/map_server/error.rs   → MapServerError enum + axum IntoResponse
  src/map_server/open.rs    → pure URL construction + browser-invoke helper
  src/map_server/markdown.rs → entity body path lookup (isolated; temporary duplication of catalog conventions)

  src/catalog/              → CatalogGraph, CatalogEntity (read-only, existing)
  src/root.rs               → project root detection (existing)

web/map/                    → embedded browser app placeholder (not Rust)
  index.html, app.js, style.css
  vendor/markdown-it.min.js, vendor/purify.min.js, vendor/github-markdown.css
```

All dependencies point downward per ADR-001. `map_server` depends on `catalog` (engine),
`root` (leaf), and axum/tokio/webbrowser (external). No upward edges.

### Browser scope

The browser app (`web/map/`) is a **placeholder shell** in SL-072. The Rust server is the
deliverable. The placeholder `index.html` + `app.js` is enough to send requests and verify
end-to-end format/behaviour manually. Full interactive map UX is out of scope.

### Data flow

```
Browser                    axum router              Catalog / FS
  │                            │                        │
  ├─ GET /api/graph ──────────►│                        │
  │                            ├─ graph.read() → clone  │
  │                            ├─ Json(snapshot)        │
  │◄── 200 application/json ───┤                        │
  │                            │                        │
  ├─ POST /api/refresh ───────►│                        │
  │                            ├─ scan_catalog(root) ───►│
  │                            ├─ CatalogGraph::from_catalog()
  │                            ├─ graph.write()          │
  │◄── 200 {"ok":true} ───────┤                        │
  │                            │                        │
  ├─ GET /api/entity/SL-001/markdown ─►                  │
  │                            ├─ validate_entity_id()   │
  │                            ├─ lookup in graph        │
  │                            ├─ read .doctrine/slice/001/slice-001.md
  │◄── 200 text/markdown ─────┤                        │
  │                            │                        │
  ├─ POST /api/dot/svg ───────►│                        │
  │  (DOT body)                ├─ body limit (middleware)│
  │                            ├─ dot_renderer.render_svg()
  │◄── 200 image/svg+xml ─────┤                        │
```

## 2. Core Types

### AppState

```rust
pub(crate) struct AppState {
    pub(crate) root: PathBuf,
    pub(crate) graph: Arc<tokio::sync::RwLock<catalog::graph::CatalogGraph>>,
    pub(crate) dot_renderer: Arc<dyn DotRenderer>,
}
```

Catalog loaded at startup; replaced on refresh behind `RwLock` inside `Arc`.
No background polling. No TTL.

### Config

```rust
pub(crate) struct Config {
    pub(crate) root: PathBuf,
    pub(crate) graph: catalog::graph::CatalogGraph,
    pub(crate) port: u16,
    pub(crate) open: bool,
    pub(crate) focus: Option<String>,
    pub(crate) depth: u8,
}
```

No `host` field — the server binds **loopback-only** (`127.0.0.1`). This is a
security property of the slice; if non-loopback binding is needed, it must be
explicitly designed and risk-assessed in a follow-up slice.

### DotRenderer trait

```rust
#[async_trait]
pub(crate) trait DotRenderer: Send + Sync {
    async fn render_svg(&self, dot: &[u8]) -> Result<Vec<u8>, MapServerError>;
}

pub(crate) struct RealDotRenderer;
```

The sole abstraction — isolates the graphviz process seam for testability.
Production impl spawns `dot -Tsvg`. Tests inject a fake.

### MapServerError

```rust
#[derive(Debug, thiserror::Error)]
pub(crate) enum MapServerError {
    #[error("bad entity id: {0}")]
    BadEntityId(String),
    #[error("entity not found: {0}")]
    EntityNotFound(String),
    #[error("asset not found: {0}")]
    AssetNotFound(String),
    #[error("entity markdown not implemented for kind: {0}")]
    MarkdownNotImplemented(&'static str),
    #[error("request body too large")]
    BodyTooLarge,
    #[error("{tool} is unavailable")]
    ToolUnavailable { tool: &'static str },
    #[error("{command} failed with status {status:?}")]
    CommandFailed {
        command: &'static str,
        status: Option<i32>,
        stderr: String,
    },
    #[error("{command} timed out")]
    Timeout { command: &'static str },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

Implements `axum::response::IntoResponse`. Response body: `{"error":"snake_case_variant","message":"..."}`
(JSON), with `stderr` included for `CommandFailed`. Status codes: 400 BadEntityId, 404
EntityNotFound/AssetNotFound, 413 BodyTooLarge, 422 CommandFailed,
501 MarkdownNotImplemented, 503 ToolUnavailable, 504 Timeout, 500 Other.

**Stderr cap:** `CommandFailed.stderr` is truncated to 8 KiB before serialization into the
JSON response to prevent unbounded payload leakage. The full stderr is available in any
future server-side log layer but never in the HTTP body.

### Entity markdown lookup (isolated in map_server::markdown)

```rust
// src/map_server/markdown.rs

/// Return the Markdown body for a known entity key.
/// Path derivation is isolated here; it temporarily duplicates catalog
/// path conventions for non-REQ, non-memory kinds. Promotion into `catalog`
/// is required before additional path exceptions are added.
pub(crate) async fn read_entity_markdown(
    root: &Path,
    key: &EntityKey,
) -> Result<String, MapServerError> {
    let path = entity_md_path(root, key)?;
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MapServerError::EntityNotFound(key.canonical()),
            _ => MapServerError::Other(e.into()),
        })
}

/// Derive the .md file path for an entity key.
/// Known kinds use the catalog convention: `<kind.dir>/<nnn>/<stem>.md`.
/// Memory kinds (ASM, DEC, QUE, CON) read the entity's record TOML to extract
/// the UID, then delegate to `memory::read_body`.
/// Requirements (REQ) return 501 — unresolved in SL-072 (needs parent spec lookup).
fn entity_md_path(root: &Path, key: &EntityKey) -> Result<PathBuf, MapServerError> {
    match key.prefix {
        "REQ" => Err(MapServerError::MarkdownNotImplemented("REQ")),
        // Knowledge records (ASM/DEC/QUE/CON): read the record TOML to get the uid,
        // then the body lives at `{kind.dir}/{id:03}/record-{id:03}.md`.
        // The uid is in the TOML for memory::read_body; for path-only lookup we use
        // the entity directory's record-NNN.md directly.
        "ASM" | "DEC" | "QUE" | "CON" => {
            let kind = integrity::kind_by_prefix(key.prefix)
                .ok_or(MapServerError::BadEntityId(key.canonical()))?;
            let dir = root.join(kind.kind.dir).join(format!("{:03}", key.id));
            Ok(dir.join(format!("{}-{:03}.md", kind.stem, key.id)))
        }
        _ => {
            let kind = integrity::kind_by_prefix(key.prefix)
                .ok_or(MapServerError::BadEntityId(key.canonical()))?;
            let dir = root.join(kind.kind.dir).join(format!("{:03}", key.id));
            Ok(dir.join(format!("{}-{:03}.md", kind.stem, key.id)))
        }
    }
}
```

This is the single function in the map server that knows entity kind structure.
It validates the prefix against `integrity::KINDS` (returns 400 for unknown
prefixes — no panics). **Temporary duplication:** path derivation for
slice/adr/spec/etc. kinds repeats catalog conventions. Promotion into
`catalog::hydrate::entity_body_path(root, key)` is required before additional
path exceptions are added beyond SL-072.

**REQ explicit decision:** requirement markdown returns HTTP 501 for SL-072.
The parent-spec lookup needed for REQ path resolution is a non-trivial catalog
walk; it belongs in a follow-up slice or a catalog-owned helper, not in the
map server. The route handler and tests encode 501 explicitly.

## 3. HTTP Routes

```rust
pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .route("/vendor/{*path}", get(vendor_asset))
        .route("/api/health", get(health))
        .route("/api/graph", get(graph))
        .route("/api/refresh", post(refresh))
        .route("/api/dot/svg", post(dot_svg).layer(DefaultBodyLimit::max(DOT_BODY_LIMIT)))
        .route("/api/entity/{id}/markdown", get(entity_markdown))
        .with_state(state)
}
```

### `GET /` — index

Serve embedded `index.html`. Hash routing in the browser — no SPA fallback needed.

```rust
async fn index() -> impl IntoResponse {
    let asset = Assets::get("index.html").expect("index.html is embedded");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], asset.data)
}
```

### `GET /assets/{*path}` — static assets

```rust
async fn asset(Path(path): Path<String>) -> Result<impl IntoResponse, MapServerError> {
    serve_embedded(&path)
}
```

### `GET /vendor/{*path}` — vendor assets

```rust
async fn vendor_asset(Path(path): Path<String>) -> Result<impl IntoResponse, MapServerError> {
    let full_path = format!("vendor/{path}");
    serve_embedded(&full_path)
}
```

Both delegate to:

```rust
fn serve_embedded(path: &str) -> Result<impl IntoResponse, MapServerError> {
    let ct = content_type_for(path);
    let asset = Assets::get(path).ok_or(MapServerError::AssetNotFound(path.to_owned()))?;
    Ok(([(header::CONTENT_TYPE, ct)], asset.data))
}
```

Content-type mapping: local match on extension — `.html`/`.css`/`.js`/`.svg`/`.json`/`.woff2`.
No `mime_guess` dep.

### `GET /api/health` — liveness + capabilities

Uses `tokio::process::Command` (non-blocking) with a 2-second timeout. Checks
`dot` exit status. Graph availability is `!graph.nodes.is_empty()`.

```rust
const HEALTH_DOT_TIMEOUT: Duration = Duration::from_secs(2);

async fn health(State(state): State<AppState>) -> impl IntoResponse {
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
    // kill_on_drop(true) ensures child cleanup on timeout
    let output = tokio::time::timeout(HEALTH_DOT_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| MapServerError::Timeout { command: "dot" })??;

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
```

### `GET /api/graph` — canonical graph JSON

Returns an owned snapshot — clones the catalog graph under the read lock
so the response does not hold the lock during serialization.

```rust
async fn graph(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.graph.read().await.clone();
    Json(snapshot)
}
```

`CatalogGraph` already derives `Clone`. The browser receives
raw `CatalogGraph` JSON (`{ nodes: {...}, edges: [...] }`).
This is an **internal format** — the browser normalizes as needed and the shape
may change with catalog evolution. Diagnostics live on `Catalog`, not
`CatalogGraph`; if diagnostics are needed in the browser later, a separate
`/api/diagnostics` endpoint is the right seam. A minimum contract test verifies
required top-level keys (`nodes`, `edges`).

### `POST /api/refresh` — re-scan corpus

```rust
async fn refresh(State(state): State<AppState>) -> Result<impl IntoResponse, MapServerError> {
    let catalog = catalog::hydrate::scan_catalog(&state.root)?;
    let g = catalog::graph::CatalogGraph::from_catalog(&catalog);
    *state.graph.write().await = g;
    Ok(Json(json!({"ok": true})))
}
```

### `POST /api/dot/svg` — browser DOT → SVG

Body size limited by `DefaultBodyLimit` layer on the route (defense-in-depth)
plus an in-handler check. On timeout/cancellation, the child is configured with
`kill_on_drop(true)` so it is killed on drop; explicit reap is not guaranteed
by this function.

```rust
async fn dot_svg(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse, MapServerError> {
    // Defense-in-depth: the route layer also enforces a limit, but re-check
    if body.len() > DOT_BODY_LIMIT {
        return Err(MapServerError::BodyTooLarge);
    }
    let svg = state.dot_renderer.render_svg(&body).await?;
    Ok(([(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")], svg))
}
```

### `GET /api/entity/{id}/markdown` — entity Markdown body

```rust
async fn entity_markdown(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MapServerError> {
    let (kind_ref, num) = integrity::parse_canonical_ref(&id)
        .map_err(|_| MapServerError::BadEntityId(id.clone()))?;
    let key = EntityKey { prefix: kind_ref.kind.prefix, id: num };
    // Prefix + numeric id validated by parse_canonical_ref — no separate guards.
    let graph = state.graph.read().await;
    let _node = graph.nodes.get(&NodeKey::Entity(key))
        .ok_or(MapServerError::EntityNotFound(id.clone()))?;
    drop(graph);

    let body = read_entity_markdown(&state.root, &key).await?;
    Ok(([(header::CONTENT_TYPE, "text/markdown; charset=utf-8")], body))
}
```

Entity ID validation reuses `integrity::parse_canonical_ref` — the single
corpus-wide ID parser — rather than a parallel regex. No digit-count guard
is applied: `parse_canonical_ref` accepts any valid prefix + `u32` id, and
`EntityKey::canonical()` zero-pads to 3 digits for display regardless of
actual magnitude. Loose forms are browser concerns. Unknown prefixes are
rejected by `parse_canonical_ref` itself (returns 400, never panics).

## 4. Graphviz Process Bridge

```rust
// src/map_server/shell.rs

const DOT_BODY_LIMIT: usize = 1_048_576; // 1 MiB
const DOT_TIMEOUT: Duration = Duration::from_secs(10);

#[async_trait]
impl DotRenderer for RealDotRenderer {
    async fn render_svg(&self, dot: &[u8]) -> Result<Vec<u8>, MapServerError> {
        let mut child = tokio::process::Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)   // Ensure child is killed if the future is dropped
            .spawn()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => MapServerError::ToolUnavailable { tool: "dot" },
                _ => MapServerError::Other(e.into()),
            })?;

        // Write stdin (owned); drop to signal EOF
        let mut stdin = child.stdin.take().expect("stdin piped");
        let dot_owned = dot.to_vec();
        tokio::time::timeout(DOT_TIMEOUT, stdin.write_all(&dot_owned))
            .await
            .map_err(|_| MapServerError::Timeout { command: "dot" })?
            .map_err(|e| MapServerError::Other(e.into()))?;
        drop(stdin);

        // Wait for child to finish with timeout
        let output = tokio::time::timeout(DOT_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_| MapServerError::Timeout { command: "dot" })??;

        if !output.status.success() {
            return Err(MapServerError::CommandFailed {
                command: "dot",
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        Ok(output.stdout)
    }
}
```

Invariants:
- No shell — `.arg("-Tsvg")` only, never `sh -c`
- No temp files — piped stdin/stdout/stderr
- `kill_on_drop(true)` — child is killed even if the future is cancelled
- Timeout enforced on both stdin write and process wait
- Structured error mapping: NotFound → ToolUnavailable, non-zero exit → CommandFailed, timeout → Timeout
- Stderr collected via pipe (bounded by OS buffer); response payload capped at 8 KiB

### DOT semantics

`POST /api/dot/svg` is a **rendering utility**, not part of Doctrine's graph
semantics. The browser generates DOT for the currently visible projection;
the server renders it mechanically. Canonical graph semantics remain in
`CatalogGraph` (and future `cordage` projections). DOT is disposable presentation.

## 5. Asset Serving

```rust
// src/map_server/assets.rs

#[derive(RustEmbed)]
#[folder = "web/map/"]
pub(crate) struct Assets;

fn content_type_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html")  => "text/html; charset=utf-8",
        Some("css")   => "text/css; charset=utf-8",
        Some("js")    => "application/javascript; charset=utf-8",
        Some("svg")   => "image/svg+xml",
        Some("json")  => "application/json",
        Some("woff2") => "font/woff2",
        _             => "application/octet-stream",
    }
}
```

## 6. CLI Entry

```rust
// src/commands/map.rs (command tier)

#[derive(clap::Args)]
pub(crate) struct MapServeArgs {
    #[arg(long, default_value = "0")]
    port: u16,
    #[arg(long)]
    open: bool,
    #[arg(long, value_parser = validate_focus)]
    focus: Option<String>,
    #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=3))]
    depth: u8,
}

fn validate_focus(s: &str) -> Result<String, String> {
    integrity::parse_canonical_ref(s)
        .map(|_| s.to_owned())
        .map_err(|e| format!("focus must be a canonical entity id (e.g. SL-001), got '{s}': {e}"))
}

pub(crate) async fn run_serve(path: Option<PathBuf>, args: MapServeArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let catalog = crate::catalog::hydrate::scan_catalog(&root)?;
    let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);

    map_server::serve(map_server::Config {
        root,
        graph,
        port: args.port,
        open: args.open,
        focus: args.focus,
        depth: args.depth,
    }).await
}
```

In `main.rs`:
```rust
// Command enum
Map {
    #[command(subcommand)]
    command: MapCommand,
},

// Subcommand enum
#[derive(Subcommand)]
enum MapCommand {
    /// Start the local map explorer web server (loopback only)
    Serve(MapServeArgs),
}

// Dispatch
Command::Map { command } => match command {
    MapCommand::Serve(args) => map::run_serve(path, args).await,
},
```

### Serve entrypoint

```rust
// src/map_server/mod.rs

pub(crate) async fn serve(config: Config) -> anyhow::Result<()> {
    let state = AppState {
        root: config.root,
        graph: Arc::new(RwLock::new(config.graph)),
        dot_renderer: Arc::new(RealDotRenderer),
    };
    let app = routes::router(state);

    // Loopback-only — security property of this slice
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.port)).await?;
    let addr = listener.local_addr()?;
    println!("Serving Doctrine map at http://{addr}/");

    if config.open {
        let url = map_url(addr, config.focus.as_deref(), config.depth);
        if let Err(err) = open_browser(&url) {
            eprintln!("Could not open browser: {err}");
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}
```

### Browser open + pure URL construction

```rust
// src/map_server/open.rs

/// Construct the browser URL. Pure — testable without binding a socket.
fn map_url(addr: std::net::SocketAddr, focus: Option<&str>, depth: u8) -> String {
    let base = format!("http://{addr}/");
    let Some(focus_id) = focus else { return base; };
    if depth == 1 {
        format!("{base}#/focus/{focus_id}")
    } else {
        format!("{base}#/focus/{focus_id}?depth={depth}")
    }
}

fn open_browser(url: &str) -> anyhow::Result<()> {
    webbrowser::open(url)?;
    Ok(())
}
```

Browser-open failure is non-fatal: the printed URL still works.

## 7. Browser-Side Security Policy

The browser app handles untrusted Markdown and SVG content. The Rust server
only delivers bytes; the browser must enforce content security:

- **Markdown:** `markdown-it` configured with `html: false` (no raw HTML in
  source). `DOMPurify.sanitize()` applied to rendered output before
  `innerHTML` insertion. No inline `<script>` or event-handler attributes
  survive sanitization.
- **SVG:** Graphviz output is served as `image/svg+xml`. The browser renders
  it via `<img src="blob:...">` or `<object>` — **not** inline SVG injection.
  This prevents SVG-based XSS (scripts, event attributes, external references)
  without additional client-side SVG sanitization.

These rules are specified here as design constraints; they are exercised
manually during acceptance, not via automated Rust tests.

## 8. Test Strategy

### 8.1 Test fixture helper

```rust
// src/map_server/mod.rs #[cfg(test)]
async fn test_app(root: &Path) -> (axum::Router, AppState) {
    let catalog = catalog::hydrate::scan_catalog(root).expect("scan");
    let graph = catalog::graph::CatalogGraph::from_catalog(&catalog);
    let state = AppState {
        root: root.to_path_buf(),
        graph: Arc::new(RwLock::new(graph)),
        dot_renderer: Arc::new(FakeDotRenderer::new()),
    };
    (routes::router(state), state)
}
```

Reuses `catalog::test_helpers::*` (`seed_slice`, `seed_requirement`, `seed_adr`,
`seed_memory`, `tmp`, `write`) — no new fixture infrastructure.

### 8.2 Route integration tests

| Test | Behaviour proved | Fixture |
|---|---|---|
| `GET /` → 200, HTML content-type | Index served from embedded assets | None |
| `GET /assets/nope.js` → 404 | Missing asset returns not-found | None |
| `GET /api/graph` → 200, valid JSON, correct entity/edge counts | Catalog→JSON round-trip | 1 slice, 1 ADR, 1 edge |
| `GET /api/graph` → top-level keys `nodes`/`edges` present | Minimum contract shape | 1 entity |
| `POST /api/refresh` → 200, then `GET /api/graph` shows added entity | Refresh re-scans from disk | Mutate fixture between calls |
| `GET /api/entity/SL-001/markdown` → 200, `text/markdown`, body matches `.md` | MD retrieval from correct path | Slice with non-empty `.md` |
| `GET /api/entity/SL-999/markdown` → 404 | Missing entity → not found | No SL-999 in fixture |
| `GET /api/entity/sl-001/markdown` → 400 | Malformed ID rejected | None |
| `GET /api/entity/BOGUS-001/markdown` → 400 | Unknown prefix rejected | None |
| `GET /api/entity/REQ-001/markdown` → 501, `{"error":"markdown_not_implemented"}` | REQ explicitly unimplemented via dedicated variant | Requirement in fixture |
| `GET /api/entity/ASM-001/markdown` → 200 | Memory entity path derived via memory module conventions | Memory with body |
| `GET /api/health` → 200, correct JSON shape | Health reports capability status | Fixture with entities |

### 8.3 DotRenderer tests (fake injected)

| Test | Behaviour proved |
|---|---|
| Valid DOT → 200 SVG response | Happy path |
| Body > 1 MiB → 413, renderer not called | Size gate fires |
| Fake returns `ToolUnavailable` → 503 | Tool-unavailable propagation |
| Fake returns `CommandFailed{stderr:"err"}` → 422, stderr in payload | Command-failed propagation |
| Fake returns `Timeout` → 504 | Timeout propagation |

### 8.4 Process seam tests (RealDotRenderer, conditional)

| Test | Behaviour proved |
|---|---|
| `"digraph { a -> b }"` → SVG bytes | Real process spawn, stdin/stdout |
| `"garbage"` → `CommandFailed` with stderr | Invalid input → structured error |
| Both skipped if `which dot` fails | CI resilience |

### 8.5 Error mapping tests (pure unit)

| `MapServerError` variant | Expected status | Expected JSON keys |
|---|---|---|
| `BadEntityId("x")` | 400 | `error`, `message` |
| `EntityNotFound("SL-999")` | 404 | `error`, `message` |
| `AssetNotFound("x.js")` | 404 | `error`, `message` |
| `BodyTooLarge` | 413 | `error`, `message` |
| `CommandFailed{command:"dot",status:Some(1),stderr:"err"}` | 422 | `error`, `message`, `stderr` |
| `ToolUnavailable{tool:"dot"}` | 503 | `error`, `message` |
| `Timeout{command:"dot"}` | 504 | `error`, `message` |
| `MarkdownNotImplemented("REQ")` | 501 | `error`, `message` |
| `Other(anyhow!("internal"))` | 500 | `error`, `message` (no raw debug) |

### 8.6 Entity ID validation tests (pure unit)

| Input | Expected |
|---|---|
| `"SL-001"` | valid (`parse_canonical_ref` succeeds) |
| `"SL-001"` → `("SL", 1)` | parsed correctly via `parse_canonical_ref` |
| `"ADR-012"` → `("ADR", 12)` | parsed correctly via `parse_canonical_ref` |
| `"sl-001"` | invalid (`parse_canonical_ref` rejects lowercase prefix) |
| `"SL-1"` | valid (any `u32` id is accepted; no digit-count guard) |
| `"SL-1000"` | valid (`parse_canonical_ref` succeeds) |
| `"SL-001-trailing"` | invalid (`parse_canonical_ref` rejects) |
| `""` | invalid |

### 8.7 URL construction tests (pure)

| `map_url(addr, focus, depth)` | Expected |
|---|---|
| `("127.0.0.1:8080", None, 1)` | `"http://127.0.0.1:8080/"` |
| `("127.0.0.1:8080", Some("SL-001"), 1)` | `"http://127.0.0.1:8080/#/focus/SL-001"` |
| `("127.0.0.1:8080", Some("SL-001"), 2)` | `"http://127.0.0.1:8080/#/focus/SL-001?depth=2"` |
| `("[::1]:8080", None, 1)` | `"http://[::1]:8080/"` |

### 8.8 What is NOT tested

- Browser app behaviour (JS out of scope — placeholder only)
- Real network binding (`oneshot` covers route logic; `tokio::TcpListener::bind` is std)
- Concurrent refresh + markdown-read races (explicitly accepted: eventual consistency)
- `--open` browser invocation (webbrowser crate responsibility; URL construction is pure-tested)
- Graceful shutdown (CLI-only; `Ctrl-C` is the shutdown mechanism)

## 9. Cargo.toml Changes

```diff
-#tokio.workspace = true
-#axum.workspace = true
+tokio.workspace = true
+axum.workspace = true
+webbrowser = "1"
```

`tower` and `http-body-util` are already in `[dev-dependencies]`. `rust-embed` is in `[dependencies]`.
`webbrowser` is the sole new crate — invokes the platform browser opener; failure is non-fatal.

## 10. Design Decisions

| Decision | Rationale |
|---|---|
| Loopback-only binding (no `--host`) | Security property; non-loopback needs explicit risk assessment |
| `MarkdownNotImplemented` dedicated variant | Named error with correct 501 status, not `Other`→500 |
| DotRenderer as sole trait | Process seam is the only hard-to-test surface |
| CatalogGraph cloned for response | Avoids holding read lock during serialization |
| Refresh via POST, not polling | Explicit, simple, no cache invalidation |
| Entity markdown in isolated `markdown.rs` module | Confines per-kind path logic; no entity semantics leak into routes |
| REQ markdown → 501 for SL-072 | Parent-spec lookup needs catalog work; not map server's concern |
| `AssetNotFound` separate from `EntityNotFound` | Distinct error contracts for assets vs entities |
| `DefaultBodyLimit` route-layer + in-handler check | Defense-in-depth for DOT body size |
| `kill_on_drop(true)` on dot child | Guaranteed cleanup even under cancellation |
| Stderr capped at 8 KiB in response | Prevents unbounded process output in HTTP body |
| `focus` validated at CLI parse, URL constructed purely | No raw string injection into URL; URL logic is testable |
| Depth clamped 1..=3 at CLI parse | Prevents unbounded graph expansion |
| Browser placeholder only in SL-072 | Rust server is the deliverable; full UX is follow-up |
| DOT is rendering utility, not graph semantics | Canonical graph remains in catalog/cordage |
| No `tokio::spawn` for stdin write | `dot` is owned before write — no lifetime issue |
| No `mime_guess` dep | 6 extensions cover all embedded assets |
| Entity ID parsed via `integrity::parse_canonical_ref` | Reuses the single corpus-wide ID parser — no parallel regex; digit-count minimum enforced in route handler as a separate guard |
| Entity ID validated against `integrity::KINDS` for prefix lookup | No second kind registry; `kind_by_prefix` is the authority |

## 11. Concurrency Model

- **Read path:** `graph.read().await` then `drop(graph)` before any async IO.
  The markdown handler releases the lock before the filesystem read.
- **Refresh path:** `scan_catalog` (pure, no lock held) then `graph.write().await`
  to swap. Two concurrent refreshes may both scan; the later write wins.
  This is accepted eventual consistency — the graph is always a valid snapshot,
  just possibly not the very latest one.
- **Refresh + markdown race:** If refresh removes an entity between the graph
  lookup and the filesystem read, the filesystem read fails → 404. This is
  correct behaviour — the entity was valid when the request started but no
  longer exists. Test: `entity_in_graph_but_missing_markdown_returns_404` (entity present in
graph snapshot but `.md` file absent on disk). A true "deleted after lookup"
race requires an injection seam between graph read and file read, which is
overkill for SL-072.

## 12. Open Questions / Risks

- **axum 0.8 API stability.** Workspace pins `axum = "0.8"`. Compilation gate catches drift.
- **Graphviz not installed.** Health reports it; `/api/dot/svg` returns structured error.
  Browser can detect and hide the graphviz panel.
- **CatalogGraph `Clone` cost.** The graph is cloned on every `/api/graph` request.
  For a Doctrine corpus (hundreds of entities, low thousands of edges), this is
  negligible. Profile before optimizing to an `Arc`-swap pattern.
- **Memory entity path lookup (PHASE-03 scoping).** Knowledge records (ASM/DEC/QUE/CON)
  live under `.doctrine/knowledge/{assumption,decision,question,constraint}/NNN/`
  with `record-NNN.{toml,md}` files. Resolving `ASM-001` to a `.md` path requires
  reading the entity's TOML to extract the UID, then delegating to
  `memory::read_body(root, uid)`. The helper functions `memory_uid_for_key` and
  `memory_body_path` do not exist yet — they are implemented in PHASE-03 as part
  of the `map_server::markdown` module, not as a prerequisite in the memory module.
  If the path derivation grows complex, it should move into `memory::body_path(root, key)`
  in a follow-up.
- **Entity ID parsing.** The map server reuses `integrity::parse_canonical_ref` —
  the single corpus-wide ID parser — rather than introducing a parallel regex.
  A separate digit-count guard (`id >= 100` for ≥3 digits in canonical form) is
  applied in the route handler for URL-discipline, not at parse time.
- **`catalog::test_helpers` visibility.** Helper functions (`seed_slice`, `seed_adr`,
  `seed_requirement`, `seed_memory`) are currently `pub(in crate::catalog)` —
  invisible to `src/map_server/` tests. Their visibility is widened to `pub(crate)`
  during PHASE-05 (the first phase that needs them for route integration tests).
- **`webbrowser` crate platform support.** Shells out to platform opener; well-tested
  on Linux/macOS/Windows. Edge case systems (headless, minimal containers): the
  URL is printed regardless, and open failure is logged but does not abort
  server startup.
