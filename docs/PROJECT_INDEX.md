# Project Index

## Systems
- MCP Server (Rust): `crates/mcp-server` — Axum HTTP with /health
- Common Library (Rust): `crates/common` — logging + config
- Configuration: `config/default.yaml`
- CI: `.github/workflows/backup.yml` (nightly archive)
- Tooling: `.pre-commit-config.yaml`, `.vscode/tasks.json`
- Scripts: `scripts/backup_local.sh`, `scripts/backup_remote.sh`

## Endpoints
- GET /health — returns { status: "ok" }

## Docs
- GODOT_MCP_SPECIFICATION.md — overall spec
- ROADMAP_DETAILED.md — hop-by-hop plan
- CLOSE_TO_SHORE.md — methodology
- STYLE_GUIDE.md — commenting/style requirements
- DEV_LOG.md — changes per hop (this file)

## Cleanup / Deprecated
- (Auto-updated by meta-tagging tool later)

#EOF
