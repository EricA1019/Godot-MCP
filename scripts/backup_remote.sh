#!/usr/bin/env bash
set -euo pipefail

# ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
# ┃ File: scripts/backup_remote.sh                                      ┃
# ┃ Purpose: Ensure remote is configured and push to GitHub             ┃
# ┃ Author: EricA1019                                                   ┃
# ┃ Last Updated: 2025-09-02                                           ┃
# ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

REMOTE_URL="git@github.com:EricA1019/Godot-MCP.git"

if ! git rev-parse --git-dir > /dev/null 2>&1; then
  echo "Initializing git repository..."
  git init
  git add .
  git commit -m "chore: initial commit"
fi

if ! git remote get-url origin > /dev/null 2>&1; then
  git remote add origin "$REMOTE_URL"
else
  git remote set-url origin "$REMOTE_URL"
fi

BRANCH="main"
git checkout -B "$BRANCH"
git add -A
git commit -m "chore: backup $(date +%Y-%m-%dT%H:%M:%S)" || true
git push -u origin "$BRANCH"

echo "Remote backup pushed to $REMOTE_URL ($BRANCH)"

#EOF