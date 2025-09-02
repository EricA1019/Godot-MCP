# Development Log

## 2025-09-02 — Hop 1: Bootstrap MCP Server Skeleton

Decisions
- Rust workspace with crates: common (logging/config), mcp-server (HTTP server)
- Axum for HTTP, Tokio runtime
- Tracing for logs; config via YAML with env overrides

What changed
- Added Cargo workspace with two crates
- Health endpoint at GET /health
- VS Code tasks for build/test/run
- Default config in config/default.yaml

Verification
- Built workspace (cargo build)
- Ran tests (cargo test) — green
- Health endpoint returns {"status":"ok"}

Next
- Hop 2: Master Index full-text core (Tantivy + FS watcher)

#EOF

## 2025-09-02 — Hop 2: Master Index core

What changed
- Added `crates/index` with Tantivy-based index, CLI (scan/query/watch), and FS watcher.
- Schema: path (stored), content (indexed), kind/hash (stored).
- Ignored heavy/ephemeral directories in scanner: .git, target, .backups, .import, .godot, .index_data, node_modules.
- VS Code tasks to run scan/query.
 - MCP server integration: endpoints for /index/scan, /index/query, /index/query/advanced, /index/health.
 - Watcher endpoints: /index/watch/start and /index/watch/stop; auto-start on boot gated by config.
 - Two-phase update (delete-then-add) to ensure query freshness; fresh reader per query.

Verification
- Built workspace successfully.
- Ran `index-cli scan` and `index-cli query godot` — returned hits from docs.
- Added a small smoke test (`crates/index/tests/smoke.rs`).
 - Added update/delete test (`crates/index/tests/update_delete.rs`) — PASS.
 - `cargo test` green across workspace.

Next
- Debounce watcher events; add delete tombstones instead of full rescan.
- Add performance tests and integrate with MCP server.
 - Document endpoints and config; measure latency targets.

