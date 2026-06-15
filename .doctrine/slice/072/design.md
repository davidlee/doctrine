# SL-072 Design: Doctrine Map Server

## 1. Architecture & Module Layout

### Tier placement (ADR-001)

```
Command tier:
  src/main.rs           → Map { command: MapCommand } variant
  src/commands/map.rs   → clap Args, root detection, calls map_server::serve

Engine tier:
  src/map_server/mod.rs     → pub async fn serve(Config) — bind, print URL, serve
  src/map_server/state.rs   → AppState { root, graph: RwLock<CatalogGraph>, dot_renderer }
  src/map_server/routes.rs  → axum Router + thin handlers
  src/map_server/assets.rs  → #[derive(RustEmbed)] + content-type mapping
  src/map_server/shell.rs   → DotRenderer trait + RealDotRenderer impl
  src/map_server/error.rs   → MapServerError enum + axum IntoResponse
  src/map_server/open.rs    → browser-open helper

  src/catalog/              → CatalogGraph, CatalogEntity (read-only, existing)
  src/root.rs               → project root detection (existing)

web/map/                    → embedded browser app (not Rust)
  index.html, app.js, style.css
  vendor/markdown-it.min.js, vendor/purify.min.js, vendor/github-markdown.css
```

All dependencies point downward: command → engine → leaf. `map_server` depends on `catalog` (engine), `root` (leaf), and axum/tokio (external). No upward edges.

### Data flow

```
Browser                    axum router              Catalog / FS
  │                            │                        │
  ├─ GET /api/graph ──────────►│                        │
  │                            ├─ AppState.graph.read() │
  │                            ├─ Json(&*g)             │
  │◄── 200 application/json ───┤                        │
  │                            │                        │
  ├─ POST /api/refresh ───────►│                        │
  │                            ├─ scan_catalog(root) ───►│
  │                            ├─ CatalogGraph::from_catalog()
  │                            ├─ AppState.graph.write() │
  │◄── 200 {"ok":true} ───────┤                        │
  │                            │                        │
  ├─ GET /api/entity/SL-001/markdown ─►                  │
  │                            ├─ validate_entity_id()   │
  │                            ├─ lookup in graph        │
  │                            ├─ read .doctrine/slice/001/slice-001.md
  │◄── 200 text/markdown ─────┤                        │
  │                            │                        │
  ├─ POST /api/dot/svg ───────►│                        │
  │  (DOT body)                ├─ size check             │
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

Catalog loaded at startup; replaced on refresh behind RwLock. No background polling. No TTL.

### Config

```rust
pub(crate) struct Config {
    pub(crate) root: PathBuf,
    pub(crate) graph: catalog::graph::CatalogGraph,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) open: bool,
    pub(crate) focus: Option<String>,
    pub(crate) depth: u8,
}
```

### DotRenderer trait

```rust
#[async_trait]
pub(crate) trait DotRenderer: Send + Sync {
    async fn render_svg(&self, dot: &[u8]) -> Result<Vec<u8>, MapServerError>;
}

pub(crate) struct RealDotRenderer;
```

The only abstraction in the map server — isolates the graphviz process seam for testability. Production impl spawns `dot -Tsvg`. Tests inject a fake with canned responses.

### MapServerError

```rust
#[derive(Debug, thiserror::Error)]
pub(crate) enum MapServerError {
    #[error("bad entity id: {0}")]
    BadEntityId(String),
    #[error("entity not found: {0}")]
    EntityNotFound(String),
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

Implements `axum::response::IntoResponse`. HTTP payload shape: `{"error": "snake_case_variant", "message": "..."}`. Status codes: 400 BadEntityId, 404 EntityNotFound, 413 BodyTooLarge, 422 CommandFailed, 503 ToolUnavailable, 504 Timeout, 500 Other.

`CommandFailed` and `Timeout` include `command: &'static str` and `stderr: String` in the JSON payload. `Other` does not expose raw debug formatting.

## 3. HTTP Routes

```rust
pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .route("/vendor/{*path}", get(asset))
        .route("/api/health", get(health))
        .route("/api/graph", get(graph))
        .route("/api/refresh", post(refresh))
        .route("/api/dot/svg", post(dot_svg))
        .route("/api/entity/{id}/markdown", get(entity_markdown))
        .with_state(state)
}
```

### `GET /` — index

Serve embedded `index.html`. Hash routing in the browser means no SPA fallback needed.

```rust
async fn index() -> impl IntoResponse {
    let asset = Assets::get("index.html").expect("index.html is embedded");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], asset.data)
}
```

### `GET /assets/{*path}` / `GET /vendor/{*path}` — static assets

```rust
async fn asset(Path(path): Path<String>) -> Result<impl IntoResponse, MapServerError> {
    let ct = content_type_for(&path);
    let asset = Assets::get(&path).ok_or(MapServerError::EntityNotFound(path))?;
    Ok(([(header::CONTENT_TYPE, ct)], asset.data))
}
```

Content-type mapping: local match on extension — `.html`/`.css`/`.js`/`.svg`/`.json`/`.woff2`. No `mime_guess` dep.

### `GET /api/health` — liveness + capabilities

`check_dot_version` runs `dot -V` with a short timeout, returns the version string
or an error. Graph availability is `!graph.nodes.is_empty()`.

```rust
async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let dot_result = dot_version();
    let dot_ok = dot_result.is_ok();
    let dot_version = dot_result.ok();
    let graph_ok = !state.graph.read().await.nodes.is_empty();
    Json(json!({
        "ok": true,
        "dot": { "ok": dot_ok, "version": dot_version },
        "graph": { "ok": graph_ok }
    }))
}

fn dot_version() -> Result<String, MapServerError> {
    let output = std::process::Command::new("dot")
        .arg("-V")
        .stderr(Stdio::piped())  // graphviz prints version to stderr
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MapServerError::ToolUnavailable { tool: "dot" },
            _ => MapServerError::Other(e.into()),
        })?;
    String::from_utf8(output.stderr).map_err(|e| MapServerError::Other(e.into()))
}
```

### `GET /api/graph` — canonical graph JSON

```rust
async fn graph(State(state): State<AppState>) -> impl IntoResponse {
    Json(&*state.graph.read().await)
}
```

`CatalogGraph` derives `Serialize`. The browser receives `{ nodes: {...}, edges: [...], diagnostics: [...] }` and normalizes as needed.

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

```rust
async fn dot_svg(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse, MapServerError> {
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
    validate_entity_id(&id)?;
    let key = parse_entity_key(&id)?;
    let graph = state.graph.read().await;
    let _node = graph.nodes.get(&NodeKey::Entity(key))
        .ok_or(MapServerError::EntityNotFound(id.clone()))?;

    let body = read_entity_markdown(&state.root, &key).await
        .map_err(|_| MapServerError::EntityNotFound(id))?;
    Ok(([(header::CONTENT_TYPE, "text/markdown; charset=utf-8")], body))
}
```

`read_entity_markdown` matches on `key.prefix` to construct the `.md` path:

```rust
fn entity_md_path(root: &Path, key: &EntityKey) -> PathBuf {
    let kind = integrity::kind_by_prefix(key.prefix)
        .expect("validated entity prefix");
    let dir = root.join(kind.dir).join(format!("{:03}", key.id));
    let stem = format!("{}-{:03}", kind.prefix.to_lowercase(), key.id);
    dir.join(format!("{stem}.md"))
}
```

For memory kinds (ASM/DEC/QUE/CON), delegates to `memory::read_body` which handles
the `items/` vs `shipped/` fallback. For requirements (REQ), the `.md` path derivation
needs the parent spec — deferred: requirements return 501 for now, or the handler walks
up via `CatalogEdge` to find the parent spec. The handler stays thin — per-kind dispatch
is confined to this single helper function.

Entity ID validation: regex `^[A-Z][A-Z0-9]*-[0-9]{3,}$`. Loose forms (`SL71`, title search) are browser concerns.

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
            .spawn()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => MapServerError::ToolUnavailable { tool: "dot" },
                _ => MapServerError::Other(e.into()),
            })?;

        child.stdin.take().expect("stdin piped").write_all(dot).await?;

        let output = tokio::time::timeout(DOT_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_| {
                // Best-effort kill; error on kill means already dead, ignore
                MapServerError::Timeout { command: "dot" }
            })??;

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
- Timeout enforced, child killed on timeout
- Stderr collected via pipe (bounded by OS buffer)
- Structured error mapping: NotFound → ToolUnavailable, non-zero exit → CommandFailed, timeout → Timeout

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
        Some("js")    => "text/javascript; charset=utf-8",
        Some("svg")   => "image/svg+xml",
        Some("json")  => "application/json",
        Some("woff2") => "font/woff2",
        _             => "application/octet-stream",
    }
}
```

No `mime_guess` dependency. The `.woff2` entry covers the github-markdown.css webfont.

## 6. CLI Entry

```rust
// src/commands/map.rs (command tier)

#[derive(clap::Args)]
pub(crate) struct MapServeArgs {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value = "0")]
    port: u16,
    #[arg(long)]
    open: bool,
    #[arg(long)]
    focus: Option<String>,
    #[arg(long, default_value = "1")]
    depth: u8,
}

pub(crate) async fn run_serve(path: Option<PathBuf>, args: MapServeArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let catalog = crate::catalog::hydrate::scan_catalog(&root)?;
    let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);

    map_server::serve(map_server::Config {
        root,
        graph,
        host: args.host,
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
    /// Start the local map explorer web server
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

    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port)).await?;
    let addr = listener.local_addr()?;
    println!("Serving Doctrine map at http://{addr}/");

    if config.open {
        open_browser(addr, &config)?;
    }

    axum::serve(listener, app).await?;
    Ok(())
}
```

With `--port 0`, the OS assigns a free port. `listener.local_addr()` gives the resolved address for the printed URL and `--open` URL construction.

### Browser open

Uses the `webbrowser` crate (zero-dependency on Linux — shells out to `xdg-open`).
`addr` is the resolved `local_addr()` after binding, so `--port 0` always produces
a correct URL with the OS-assigned port.

```rust
// src/map_server/open.rs

fn open_browser(addr: std::net::SocketAddr, config: &Config) -> anyhow::Result<()> {
    let mut url = format!("http://{addr}/");
    if let Some(ref focus) = config.focus {
        url.push_str(&format!("#/focus/{focus}"));
        if config.depth != 1 {
            url.push_str(&format!("?depth={}", config.depth));
        }
    }
    webbrowser::open(&url)?;
    Ok(())
}
```

## 7. Test Strategy

### 7.1 Test fixture helper

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

Reuses `catalog::test_helpers::*` (`seed_slice`, `seed_requirement`, `seed_adr`, `tmp`, `write`) — no new fixture infrastructure.

### 7.2 Route integration tests

Using `tower::ServiceExt::oneshot` (no bound socket — repo already has `tower` + `http-body-util` in dev-deps).

| Test | Behaviour proved | Fixture |
|---|---|---|
| `GET /` → 200, `text/html` content-type | Asset embedding, index served | None needed |
| `GET /assets/nope.js` → 404 | Missing asset returns not-found | None |
| `GET /api/graph` → 200, valid JSON, correct entity/edge counts | Catalog→JSON round-trip | 1 slice, 1 ADR, 1 edge |
| `POST /api/refresh` → 200, then `GET /api/graph` shows added entity | Refresh re-scans from disk | Mutate fixture between calls |
| `GET /api/entity/SL-001/markdown` → 200, `text/markdown`, body matches `.md` file | MD retrieval from correct path | Slice with non-empty `.md` |
| `GET /api/entity/SL-999/markdown` → 404 | Missing entity → not found | No SL-999 in fixture |
| `GET /api/entity/sl-001/markdown` → 400 | Malformed ID rejected before any lookup | None |
| `GET /api/entity/MEM-001/markdown` → 200 | Memory entity delegates to `memory::read_body` | Memory with body |
| `GET /api/health` → 200, `{"ok":true,"dot":{...},"graph":{...}}` | Health reports capability status | Fixture with entities |

### 7.3 DotRenderer tests (fake injected)

`FakeDotRenderer` — a struct holding a closure or canned response. Tests use it to verify error propagation without `dot` on PATH.

| Test | Behaviour proved |
|---|---|
| Valid DOT body → forwarded to renderer, 200 SVG response | Happy path |
| Body > 1 MiB → 413, renderer never called | Size gate fires first |
| Renderer returns `ToolUnavailable` → 503, JSON error payload | Tool-unavailable propagation |
| Renderer returns `CommandFailed{stderr:"..."}` → 422, stderr in payload | Command-failed propagation |
| Renderer returns `Timeout` → 504 | Timeout propagation |

### 7.4 Process seam tests (RealDotRenderer, conditional)

| Test | Behaviour proved |
|---|---|
| `"digraph { a -> b }"` → SVG bytes | Real process spawn, stdin/stdout |
| `"garbage"` → `CommandFailed` with stderr | Invalid input → structured error |
| Both skipped if `which dot` fails | CI resilience |

### 7.5 Error mapping tests (pure unit)

No HTTP — test `MapServerError` → `IntoResponse` directly.

| Test | Behaviour proved |
|---|---|
| `BadEntityId("x").into_response()` → 400, `{"error":"bad_entity_id","message":"bad entity id: x"}` | Client error shape |
| `EntityNotFound("SL-999").into_response()` → 404 | Not-found shape |
| `CommandFailed{command:"dot",status:Some(1),stderr:"..."}.into_response()` → 422, stderr included | Server error with detail |
| `Other(anyhow!("db")).into_response()` → 500, no raw debug in message | Opaque internal errors |

### 7.6 Entity ID validation tests (pure unit)

| Input | Expected |
|---|---|
| `"SL-001"` | valid |
| `"SL-001"` → `("SL", 1)` | parsed correctly |
| `"ADR-012"` → `("ADR", 12)` | parsed correctly |
| `"sl-001"` | invalid (lowercase) |
| `"SL-1"` | invalid (not zero-padded 3+) |
| `"SL-1000"` | valid (4+ digits) |
| `"SL-001-trailing"` | invalid |
| `""` | invalid |

### 7.7 What is NOT tested

- Browser app behaviour (JS tests out of scope — Rust slice only)
- Concurrent refresh + graph read races (RwLock guarantees safety; no async IO during read)
- Real network binding (oneshot tests cover route logic; binding is tokio's responsibility)
- `--open` flag (integration concern — tested manually or in a future e2e suite)
- CatalogGraph serialization format (already covered by SL-071 catalog tests)

## 8. Cargo.toml Changes

Uncomment two workspace deps, add `webbrowser`:

```diff
-#tokio.workspace = true
-#axum.workspace = true
+tokio.workspace = true
+axum.workspace = true
+webbrowser = "1"
```

`tower` and `http-body-util` are already in `[dev-dependencies]`. `rust-embed` is already in `[dependencies]`. `webbrowser` is the sole new crate (zero Linux deps — shells out to `xdg-open`).

## 9. Implementation Order (phases)

1. **PHASE-01:** Module scaffolding — `src/commands/map.rs`, `src/map_server/*.rs` stubs, `web/map/index.html` placeholder, Cargo.toml uncomments, `main.rs` enum wiring. Clippy-clean skeleton compiles.

2. **PHASE-02:** Asset serving — `assets.rs`, `index.html`, `style.css`, vendor files. `GET /` and `GET /assets/*` routes. Test: `GET /` returns HTML.

3. **PHASE-03:** Graph endpoint — `GET /api/graph` with real `CatalogGraph`. Test: round-trip with fixture.

4. **PHASE-04:** Graphviz bridge — `shell.rs` (DotRenderer trait + RealDotRenderer), `POST /api/dot/svg`. Tests: fake renderer (error propagation, size cap), real process (conditional).

5. **PHASE-05:** Entity Markdown — `GET /api/entity/{id}/markdown`, ID validation, path derivation. Tests: valid MD, missing entity, malformed ID.

6. **PHASE-06:** Health + refresh + open — `GET /api/health`, `POST /api/refresh`, `--open` flag, browser-open helper. Tests: health JSON shape, refresh re-scan.

## 10. Design Decisions

| Decision | Rationale |
|---|---|
| DotRenderer as sole trait | Process seam is the only hard-to-test surface; graph + markdown endpoints test naturally with temp fixtures |
| CatalogGraph serialized directly | Browser normalizes shape client-side; no Rust-side transform layer needed |
| Refresh via POST, not background polling | Simple, explicit, no cache invalidation complexity |
| Entity markdown via path derivation, not per-kind dispatch | CatalogEntity.path is the SSoT; handler stays thin |
| No `mime_guess` dep | 6 extensions cover all embedded assets; local match is trivial |
| Hash routing in browser | No SPA fallback route needed on server |
| `--port 0` for OS-assigned port | Standard pattern; `local_addr()` gives resolved address for UI |

## 11. Open Questions / Risks

- **axum 0.8 API stability.** The workspace pins `axum = "0.8"`. Any API drift between 0.8.x patch versions should be caught by the compilation gate.
- **Graphviz not installed.** Handled gracefully — health reports it, `/api/dot/svg` returns structured error. Browser can detect and hide the graphviz panel or show a "dot not installed" message.
- **Browser app complexity.** The JS side (graph projection, DOT generation, SVG injection, Markdown rendering) is the bulk of the user-facing work. This design keeps the Rust side thin so the browser can iterate independently.
- **Memory entity markdown.** Uses `memory::read_body` which already handles the `items/` vs `shipped/` fallback. No new path logic.
