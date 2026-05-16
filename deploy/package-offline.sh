#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_DIR/output/commerce-agents-offline}"
ARCHIVE_PATH="${ARCHIVE_PATH:-$PROJECT_DIR/output/commerce-agents-offline.tar.gz}"

IMAGES=(
  commerce-agents-postgres:local
  commerce-agents-backend:local
  commerce-agents-web:local
  commerce-agents-admin:local
)

mkdir -p "$PROJECT_DIR/output"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

for image in "${IMAGES[@]}"; do
  if ! docker image inspect "$image" >/dev/null 2>&1; then
    echo "缺少镜像：$image"
    echo "请先在打包机执行 docker compose build"
    exit 1
  fi
done

docker save "${IMAGES[@]}" -o "$OUTPUT_DIR/commerce-agents-images.tar"

cp "$PROJECT_DIR/deploy/runtime/docker-compose.yml" "$OUTPUT_DIR/docker-compose.yml"
cp "$PROJECT_DIR/deploy/runtime/.env.example" "$OUTPUT_DIR/.env.example"
cp "$PROJECT_DIR/deploy/runtime/install.sh" "$OUTPUT_DIR/install.sh"
cp "$PROJECT_DIR/deploy/runtime/README.md" "$OUTPUT_DIR/README.md"
chmod +x "$OUTPUT_DIR/install.sh"

tar -czf "$ARCHIVE_PATH" -C "$(dirname "$OUTPUT_DIR")" "$(basename "$OUTPUT_DIR")"

echo "离线安装包已生成：$ARCHIVE_PATH"
