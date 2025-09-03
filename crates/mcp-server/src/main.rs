// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ Binary: mcp-server                                                  ┃
// ┃ Purpose: Minimal MCP server with health endpoint                    ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                           ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use axum::{Json, Router};
use common::{init_logging, load_config};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing::{info, warn};
use index::{IndexPaths, SearchIndex};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Serialize)]
struct Health { status: &'static str }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let cfg = load_config().unwrap_or_else(|e| {
        warn!(error=?e, "Config not found; using defaults");
        // default fallback
        common::AppConfig { server: common::ServerConfig { host: "127.0.0.1".into(), port: 8080, auto_start_watchers: true } }
    });

    // Initialize shared index state
    let workspace_root = PathBuf::from(".");
    let data_dir = PathBuf::from(".index_data");
    let index_paths = IndexPaths { root: workspace_root.clone(), data_dir };
    let mut idx = SearchIndex::open(&index_paths)?;
    // Perform an initial scan if index is empty; cheap no-op otherwise
    let _ = idx.scan_and_index(&workspace_root);
    let shared_index: Arc<Mutex<SearchIndex>> = Arc::new(Mutex::new(idx));
    // Watcher task handle managed in state
    let watcher_handle: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    let watcher_shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    // Auto-start the index watcher on server startup (toggle via config)
    if cfg.server.auto_start_watchers {
        let mut handle_guard = watcher_handle.lock().await;
        if handle_guard.is_none() {
            watcher_shutdown.store(false, Ordering::Relaxed);
            let shared_for_thread = Arc::clone(&shared_index);
            let root = workspace_root.clone();
            let shutdown = Arc::clone(&watcher_shutdown);
            let handle = tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async move {
                    let mut idx = shared_for_thread.lock().await;
                    let _ = idx.watch_with_shutdown(&root, shutdown);
                });
            });
            *handle_guard = Some(handle);
            info!("Index watcher auto-started");
        }
    }

    // Build routes via lib factory
    let app_routes = mcp_server::build_router(shared_index.clone(), watcher_handle.clone(), watcher_shutdown.clone(), workspace_root.clone());
    let app = Router::new()
        .route("/health", axum::routing::get(|| async { Json(Health { status: "ok" }) }))
        .merge(app_routes);

    let addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!(%addr, "Starting MCP server");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

//EOF