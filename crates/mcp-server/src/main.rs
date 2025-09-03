// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ Binary: mcp-server                                                  ┃
// ┃ Purpose: Minimal MCP server with health endpoint                    ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                           ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use axum::{routing::{get, post}, extract::{Query, State}, Json, Router};
use common::{init_logging, load_config};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing::{info, warn};
use index::{IndexPaths, SearchIndex};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use serde::Deserialize;
use context as ctx;
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

    // HTTP models
    #[derive(Deserialize)]
    struct QueryRequest { q: String, limit: Option<usize> }
    #[derive(Serialize)]
    struct Hit { score: f32, path: String }
    #[derive(Serialize)]
    struct QueryResponse { hits: Vec<Hit> }
    #[derive(Deserialize)]
    struct ScanRequest { path: Option<String> }
    #[derive(Deserialize)]
    struct QueryAdvancedRequest { q: String, kind: Option<String>, limit: Option<usize>, snippet: Option<bool> }
    #[derive(Serialize)]
    struct HitAdv { score: f32, path: String, kind: String, snippet: Option<String> }
    #[derive(Serialize)]
    struct HealthResponse { docs: u64, segments: usize }
    #[derive(Serialize)]
    struct ScanResponse { indexed: usize }
    #[derive(Serialize)]
    struct WatchResponse { status: &'static str }
    #[derive(Deserialize)]
    struct BundleRequest { q: String, limit: Option<usize>, cap_bytes: Option<usize>, kind: Option<String> }
    #[derive(Serialize)]
    struct BundleItemDto { path: String, kind: String, score: i32, content: String }
    #[derive(Serialize)]
    struct BundleResponse { query: String, items: Vec<BundleItemDto>, size_bytes: usize }

    // Build routes
    let app = Router::new()
        .route("/health", get(|| async { Json(Health { status: "ok" }) }))
        .route("/index/query", post({
            let shared_index = shared_index.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>, Json(req): Json<QueryRequest>| {
                let shared_index = shared_index.clone();
                async move {
                    let guard = shared_index.lock().await;
                    let limit = req.limit.unwrap_or(10).min(100).max(1);
                    let hits = guard.query(&req.q, limit).unwrap_or_default()
                        .into_iter()
                        .map(|(score, path)| Hit { score, path })
                        .collect();
                    Json(QueryResponse { hits })
                }
            }
        }))
        .route("/index/query", get({
            let shared_index = shared_index.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>, Query(req): Query<QueryRequest>| {
                let shared_index = shared_index.clone();
                async move {
                    let guard = shared_index.lock().await;
                    let limit = req.limit.unwrap_or(10).min(100).max(1);
                    let hits = guard.query(&req.q, limit).unwrap_or_default()
                        .into_iter()
                        .map(|(score, path)| Hit { score, path })
                        .collect();
                    Json(QueryResponse { hits })
                }
            }
        }))
        .route("/index/scan", post({
            let shared_index = shared_index.clone();
            let workspace_root = workspace_root.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>, Json(req): Json<ScanRequest>| {
                let shared_index = shared_index.clone();
                let root_override = req.path.map(PathBuf::from).unwrap_or(workspace_root.clone());
                async move {
                    let mut guard = shared_index.lock().await;
                    let n = guard.scan_and_index(&root_override).unwrap_or(0);
                    Json(ScanResponse { indexed: n })
                }
            }
        }))
        .route("/index/query/advanced", post({
            let shared_index = shared_index.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>, Json(req): Json<QueryAdvancedRequest>| {
                let shared_index = shared_index.clone();
                async move {
                    let guard = shared_index.lock().await;
                    let limit = req.limit.unwrap_or(10).min(100).max(1);
                    let with_snippet = req.snippet.unwrap_or(false);
                    let hits = guard
                        .query_filtered(&req.q, req.kind.as_deref(), limit, with_snippet)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(score, path, kind, snippet)| HitAdv { score, path, kind, snippet })
                        .collect::<Vec<_>>();
                    Json(hits)
                }
            }
        }))
        .route("/index/watch/start", post({
            let shared_index = shared_index.clone();
            let watcher_handle = watcher_handle.clone();
            let watcher_shutdown = watcher_shutdown.clone();
            let workspace_root = workspace_root.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>| {
                let shared_index = shared_index.clone();
                let watcher_handle = watcher_handle.clone();
                let watcher_shutdown = watcher_shutdown.clone();
                let workspace_root = workspace_root.clone();
                async move {
                    let mut handle_guard = watcher_handle.lock().await;
                    if handle_guard.is_some() {
                        return Json(WatchResponse { status: "already_running" });
                    }
                    watcher_shutdown.store(false, Ordering::Relaxed);
                    // Spawn a background task that runs the blocking watch loop
                    let shared_for_thread = shared_index.clone();
                    let root = workspace_root.clone();
                    let shutdown = watcher_shutdown.clone();
                    let handle = tokio::task::spawn_blocking(move || {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async move {
                            let mut idx = shared_for_thread.lock().await;
                            let _ = idx.watch_with_shutdown(&root, shutdown);
                        });
                    });
                    *handle_guard = Some(handle);
                    Json(WatchResponse { status: "started" })
                }
            }
        }))
        .route("/index/watch/stop", post({
            let watcher_handle = watcher_handle.clone();
            let watcher_shutdown = watcher_shutdown.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>| {
                let watcher_handle = watcher_handle.clone();
                let watcher_shutdown = watcher_shutdown.clone();
                async move {
                    let mut handle_guard = watcher_handle.lock().await;
                    if let Some(handle) = handle_guard.take() {
                        watcher_shutdown.store(true, Ordering::Relaxed);
                        // Wait for watcher to stop cleanly
                        let _ = handle.await;
                        return Json(WatchResponse { status: "stopped" });
                    }
                    Json(WatchResponse { status: "not_running" })
                }
            }
        }))
        .route("/index/health", get({
            let shared_index = shared_index.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>| {
                let shared_index = shared_index.clone();
                async move {
                    let guard = shared_index.lock().await;
                    let (docs, segments) = guard.health().unwrap_or((0,0));
                    Json(HealthResponse { docs, segments })
                }
            }
        }))
        .route("/context/bundle", post({
            let shared_index = shared_index.clone();
            move |State(_): State<Arc<Mutex<SearchIndex>>>, Json(req): Json<BundleRequest>| {
                let shared_index = shared_index.clone();
                async move {
                    let guard = shared_index.lock().await;
                    let limit = req.limit.unwrap_or(10).min(100).max(1);
                    let cap = req.cap_bytes.or(Some(ctx::DEFAULT_BUNDLE_CAP));
                    let b = ctx::bundle_query(&*guard, &req.q, limit, cap, req.kind.as_deref()).unwrap_or_else(|_| ctx::Bundle { query: req.q, items: vec![], size_bytes: 0 });
                    let items = b.items.into_iter().map(|it| BundleItemDto { path: it.path, kind: it.kind, score: it.score, content: it.content }).collect();
                    Json(BundleResponse { query: b.query, items, size_bytes: b.size_bytes })
                }
            }
        }))
        .with_state(shared_index.clone());

    let addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!(%addr, "Starting MCP server");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

//EOF