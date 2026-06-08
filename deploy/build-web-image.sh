#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

if [ -f "$PROJECT_DIR/.env" ]; then
  set -a
  . "$PROJECT_DIR/.env"
  set +a
fi

IMAGE_PREFIX="${IMAGE_PREFIX:-commerce-agents}"

echo "==> Building web frontend dist"
pnpm --dir frontend install --frozen-lockfile
pnpm --dir frontend --filter web build

echo "==> Building web runtime image"
docker compose build --no-cache web

echo "==> Verifying web runtime image"
docker image inspect "$IMAGE_PREFIX-web:local" >/dev/null
echo "found $IMAGE_PREFIX-web:local"
