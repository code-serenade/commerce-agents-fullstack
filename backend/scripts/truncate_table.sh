#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url

TABLE_NAME="${1:-}"
if [ -z "$TABLE_NAME" ]; then
  echo "错误：需要表名。用法：./scripts/truncate_table.sh your_table"
  exit 1
fi

assert_safe_identifier "$TABLE_NAME" "表名"

if ! run_psql "$DB_NAME" "SELECT to_regclass('public.${TABLE_NAME}') IS NOT NULL;" | grep -q t; then
  echo "错误：表不存在：$TABLE_NAME"
  exit 1
fi

run_psql "$DB_NAME" "TRUNCATE TABLE ${TABLE_NAME} RESTART IDENTITY CASCADE;"
echo "已清空表：$TABLE_NAME"
