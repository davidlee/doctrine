// SPDX-License-Identifier: GPL-3.0-only
//! Map-server shared state and configuration (SL-072 PHASE-01).
//!
//! PHASE-01 scaffolding — types consumed in PHASE-02+.
#![allow(
    dead_code,
    reason = "PHASE-01 foundation — types consumed in PHASE-02+"
)]

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::catalog::graph::CatalogGraph;
use crate::catalog::hydrate::Catalog;
use crate::map_server::error::MapServerError;
use crate::priority::graph::PriorityGraph;

/// All three priority data stores — built and replaced atomically (SL-089 D9).
/// A single [`RwLock<DataStores>`] guarantees a reader never sees a fresh
/// `catalog` but a stale `priority_graph`.
#[expect(
    dead_code,
    reason = "PHASE-01 Task 4: consumed in PHASE-02 AppState revision"
)]
pub(crate) struct DataStores {
    pub(crate) catalog: Catalog,
    pub(crate) priority_graph: PriorityGraph,
    pub(crate) graph: CatalogGraph,
}

/// Shared application state available to every request handler.
pub(crate) struct AppState {
    /// The project root — the directory containing `.doctrine/`.
    pub(crate) root: PathBuf,

    /// The live catalog graph, behind a read-write lock so the serve thread
    /// can refresh it on every request (a write is cheap — it replaces the
    /// single allocation). The `Arc` is the single-owner copy; `axum::State`
    /// clones the `Arc` cheaply.
    pub(crate) graph: Arc<RwLock<CatalogGraph>>,

    /// A Graphviz `dot → SVG` renderer. Injected as a trait object so unit
    /// tests can supply a stub without shelling out.
    pub(crate) dot_renderer: Arc<dyn DotRenderer>,
}

/// Start-up configuration assembled from CLI flags + catalog hydration.
pub(crate) struct Config {
    /// The project root.
    pub(crate) root: PathBuf,

    /// The catalog graph hydrated at start-up.
    pub(crate) graph: CatalogGraph,

    /// The TCP port the HTTP server binds to.
    pub(crate) port: u16,

    /// If true, launch the default browser after binding.
    pub(crate) open: bool,

    /// An optional entity id to focus the graph on (e.g. `SL-005`).
    pub(crate) focus: Option<String>,

    /// Graph neighbourhood depth for the focus entity.
    pub(crate) depth: u8,
}

/// The Graphviz rendering seam.
///
/// Separates the pure HTTP layer from the impure `dot` process invocation.
/// A real implementation lives in `shell.rs` (PHASE-04); a test stub can
/// return canned SVG without a system dependency.
#[async_trait]
pub(crate) trait DotRenderer: Send + Sync {
    /// Render a DOT source buffer to SVG bytes.
    async fn render_svg(&self, _dot: &[u8]) -> Result<Vec<u8>, MapServerError>;
}

/// The production renderer — implemented in [`shell`](super::shell).
pub(crate) struct RealDotRenderer;
