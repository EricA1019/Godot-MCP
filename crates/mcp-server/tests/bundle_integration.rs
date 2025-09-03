use std::{sync::{Arc, atomic::AtomicBool}};

use axum::{Router, body::{Body, to_bytes}};
use index::{IndexPaths, SearchIndex};
use tokio::task::JoinHandle;
use tower::ServiceExt; // for oneshot
use hyper::{Request, StatusCode};

#[tokio::test]
async fn context_bundle_endpoint_smoke() {
    // temp index
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("root");
    let data = tmp.path().join("data");
    std::fs::create_dir_all(&root).unwrap();
    // seed some files
    std::fs::write(root.join("a.gd"), "func _ready():\n\tprint(\"godot banana\")").unwrap();
    std::fs::write(root.join("b.rs"), "fn main(){ println!(\"banana\"); }").unwrap();
    std::fs::write(root.join("doc.txt"), "banana in docs").unwrap();

    let paths = IndexPaths { root: root.clone(), data_dir: data.clone() };
    let mut idx = SearchIndex::open(&paths).unwrap();
    let _ = idx.scan_and_index(&root).unwrap();
    let shared_index: Arc<tokio::sync::Mutex<SearchIndex>> = Arc::new(tokio::sync::Mutex::new(idx));
    let watcher_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>> = Arc::new(tokio::sync::Mutex::new(None));
    let watcher_shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

    let app: Router = mcp_server::build_router(shared_index.clone(), watcher_handle, watcher_shutdown, root.clone());

    // call endpoint directly against the router
    let body = serde_json::to_vec(&serde_json::json!({"q":"banana","limit":10, "cap_bytes": 4096})).unwrap();
    let req = Request::post("/context/bundle")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["query"], "banana");
    let items = v["items"].as_array().unwrap();
    assert!(!items.is_empty());
    let size = v["size_bytes"].as_u64().unwrap();
    assert!(size > 0);
    assert!(size <= 4096, "bundle size should respect cap");
    // Deterministic ordering: non-increasing score, then path asc
    for w in items.windows(2) {
        let a = &w[0];
        let b = &w[1];
        let sa = a["score"].as_i64().unwrap();
        let sb = b["score"].as_i64().unwrap();
        if sa == sb {
            let pa = a["path"].as_str().unwrap();
            let pb = b["path"].as_str().unwrap();
            assert!(pa <= pb, "paths must be ascending on tie");
        } else {
            assert!(sa >= sb, "scores must be non-increasing");
        }
    }
}

#[tokio::test]
async fn context_bundle_kind_filter() {
    // temp index
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("root");
    let data = tmp.path().join("data");
    std::fs::create_dir_all(&root).unwrap();
    // seed files across kinds
    std::fs::write(root.join("a.gd"), "func _ready():\n\tprint(\"apple banana\")").unwrap();
    std::fs::write(root.join("b.rs"), "fn main(){ println!(\"banana\"); }").unwrap();
    std::fs::write(root.join("doc.txt"), "banana in docs").unwrap();

    let paths = IndexPaths { root: root.clone(), data_dir: data.clone() };
    let mut idx = SearchIndex::open(&paths).unwrap();
    let _ = idx.scan_and_index(&root).unwrap();
    let shared_index: Arc<tokio::sync::Mutex<SearchIndex>> = Arc::new(tokio::sync::Mutex::new(idx));
    let watcher_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>> = Arc::new(tokio::sync::Mutex::new(None));
    let watcher_shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

    let app: Router = mcp_server::build_router(shared_index.clone(), watcher_handle, watcher_shutdown, root.clone());

    // request with kind filter = gdscript
    let body = serde_json::to_vec(&serde_json::json!({"q":"banana","limit":10, "cap_bytes": 4096, "kind": "gdscript"})).unwrap();
    let req = Request::post("/context/bundle")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let items = v["items"].as_array().unwrap();
    assert!(!items.is_empty());
    for it in items {
        assert_eq!(it["kind"].as_str().unwrap(), "gdscript");
    }
}

#[tokio::test]
async fn context_bundle_enforces_small_cap() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("root");
    let data = tmp.path().join("data");
    std::fs::create_dir_all(&root).unwrap();
    // Write a large file to pressure the cap
    let big = "banana ".repeat(10_000);
    std::fs::write(root.join("big.txt"), big).unwrap();

    let paths = IndexPaths { root: root.clone(), data_dir: data.clone() };
    let mut idx = SearchIndex::open(&paths).unwrap();
    let _ = idx.scan_and_index(&root).unwrap();
    let shared_index: Arc<tokio::sync::Mutex<SearchIndex>> = Arc::new(tokio::sync::Mutex::new(idx));
    let watcher_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>> = Arc::new(tokio::sync::Mutex::new(None));
    let watcher_shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

    let app: Router = mcp_server::build_router(shared_index.clone(), watcher_handle, watcher_shutdown, root.clone());

    let cap = 512u64;
    let body = serde_json::to_vec(&serde_json::json!({"q":"banana","limit":10, "cap_bytes": cap})).unwrap();
    let req = Request::post("/context/bundle")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let size = v["size_bytes"].as_u64().unwrap();
    assert!(size <= cap, "bundle size {} should be <= cap {}", size, cap);
}
