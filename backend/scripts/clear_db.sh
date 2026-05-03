#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url

run_psql "$DB_NAME" "DROP SCHEMA IF EXISTS public CASCADE;"
run_psql "$DB_NAME" "CREATE SCHEMA public;"
run_psql "$DB_NAME" "GRANT ALL ON SCHEMA public TO ${DB_USER};"
run_psql "$DB_NAME" "GRANT ALL ON SCHEMA public TO public;"

echo "已清空数据库：$DB_NAME"
