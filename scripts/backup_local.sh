#!/usr/bin/env bash
set -euo pipefail

# ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
# ┃ File: scripts/backup_local.sh                                       ┃
# ┃ Purpose: Create date-stamped local archive of repo                  ┃
# ┃ Author: EricA1019                                                   ┃
# ┃ Last Updated: 2025-09-02                                           ┃
# ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/.backups"
STAMP="$(date +%Y%m%d-%H%M%S)"
NAME="godot-mcp-$STAMP"

mkdir -p "$OUT_DIR"

tar --exclude-vcs --exclude=".backups" -czf "$OUT_DIR/$NAME.tgz" -C "$ROOT_DIR" .
echo "Local backup created: $OUT_DIR/$NAME.tgz"

#EOF