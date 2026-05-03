#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url
assert_safe_identifier "$DB_NAME" "数据库名"

if run_psql postgres "SELECT 1 FROM pg_database WHERE datname = '${DB_NAME}'" | grep -q 1; then
  echo "数据库已存在：$DB_NAME"
  exit 0
fi

run_psql postgres "CREATE DATABASE ${DB_NAME};"
echo "已创建数据库：$DB_NAME"
