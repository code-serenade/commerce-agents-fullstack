#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url
require_sea_orm_cli

TABLE_COUNT="$(run_psql "$DB_NAME" "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';" | tr -d '[:space:]')"

if [ "${TABLE_COUNT:-0}" = "0" ]; then
  echo "错误：当前数据库没有任何表结构，无法生成 entity"
  echo "请先执行迁移或先创建表，再运行此脚本"
  exit 1
fi

sea-orm-cli generate entity \
  -o "$ENTITY_DIR" \
  --with-serde both \
  --date-time-crate time

echo "entity 已生成到：$ENTITY_DIR"
echo "此操作不会修改数据库结构"
