// SPDX-License-Identifier: GPL-3.0-only
pub(crate) mod assets;
pub(crate) mod error;
pub(crate) mod markdown;
pub(crate) mod routes;
pub(crate) mod shell;
pub(crate) mod state;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use crate::map_server::shell::{FakeDotMode, FakeDotRenderer};

    pub(crate) async fn test_app(root: &std::path::Path) -> axum::Router {
        let catalog = crate::catalog::hydrate::scan_catalog(root).expect("scan");
        let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
        let state = super::state::AppState {
            root: root.to_path_buf(),
            graph: Arc::new(RwLock::new(graph)),
            dot_renderer: Arc::new(FakeDotRenderer {
                mode: FakeDotMode::Success(b"<svg></svg>".to_vec()),
            }),
        };
        super::routes::router(state)
    }
}
