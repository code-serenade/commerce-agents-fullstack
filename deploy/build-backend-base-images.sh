#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

docker build \
  -f "$PROJECT_DIR/deploy/backend/builder-base.Dockerfile" \
  -t rust-builder:alpine \
  "$PROJECT_DIR"
