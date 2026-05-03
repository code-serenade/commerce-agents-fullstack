#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url
require_sea_orm_cli

ACTION="${1:-}"
shift || true

case "$ACTION" in
  up|down|fresh|refresh|reset|status)
    sea-orm-cli migrate "$ACTION" -d "$MIGRATION_DIR" "$@"
    ;;
  generate)
    NAME="${1:-}"
    if [ -z "$NAME" ]; then
      echo "错误：需要迁移名称。用法：./scripts/migrate.sh generate create_users"
      exit 1
    fi
    sea-orm-cli migrate generate "$NAME" -d "$MIGRATION_DIR"
    ;;
  *)
    echo "用法: ./scripts/migrate.sh [up|down|fresh|refresh|reset|status|generate NAME]"
    exit 1
    ;;
esac
