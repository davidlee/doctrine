// SPDX-License-Identifier: GPL-3.0-only
pub(crate) mod assets;
pub(crate) mod error;
pub(crate) mod markdown;
pub(crate) mod open;
pub(crate) mod routes;
pub(crate) mod shell;
pub(crate) mod state;

use crate::catalog::scan::ScanMode;

pub(crate) async fn serve(config: state::Config) -> anyhow::Result<()> {
    use std::io::Write;
    use std::sync::Arc;

    use tokio::sync::RwLock;

    let catalog = crate::catalog::hydrate::scan_catalog(&config.root, ScanMode::default())?;
    let priority_graph = crate::priority::graph::build(&config.root)?;
    let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
    let stores = state::DataStores {
        catalog,
        priority_graph,
        graph,
    };
    let state = state::AppState {
        root: config.root.clone(),
        stores: Arc::new(RwLock::new(stores)),
        dot_renderer: Arc::new(state::RealDotRenderer),
    };
    let app = routes::router(state);

    // Loopback-only — security property of this slice
    let listener =
        tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, config.port)).await?;
    let addr = listener.local_addr()?;
    writeln!(std::io::stdout(), "Serving Doctrine map at http://{addr}/")?;

    if config.open {
        let url = open::map_url(addr, config.focus.as_deref(), config.depth);
        if let Err(err) = open::open_browser(&url) {
            writeln!(std::io::stderr(), "Could not open browser: {err}")?;
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use crate::catalog::scan::ScanMode;
    use crate::map_server::shell::{FakeDotMode, FakeDotRenderer};

    pub(crate) async fn test_app(root: &std::path::Path) -> axum::Router {
        let catalog = crate::catalog::hydrate::scan_catalog(root, ScanMode::default()).expect("scan");
        let priority_graph = crate::priority::graph::build(root).expect("priority graph");
        let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
        let stores = super::state::DataStores {
            catalog,
            priority_graph,
            graph,
        };
        let state = super::state::AppState {
            root: root.to_path_buf(),
            stores: Arc::new(RwLock::new(stores)),
            dot_renderer: Arc::new(FakeDotRenderer {
                mode: FakeDotMode::Success(b"<svg></svg>".to_vec()),
            }),
        };
        super::routes::router(state)
    }
}
