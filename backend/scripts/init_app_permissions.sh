#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url

upsert_permission() {
  local code="$1"
  local name="$2"
  local parent_code="$3"
  local sort="$4"
  local kind="$5"

  local parent_sql="NULL"
  if [ -n "$parent_code" ]; then
    parent_sql="'${parent_code}'"
  fi

  run_psql "$DB_NAME" "
    INSERT INTO app_permissions (code, name, parent_code, sort, kind)
    VALUES ('${code}', '${name}', ${parent_sql}, ${sort}, '${kind}')
    ON CONFLICT (code) DO UPDATE
    SET
      name = EXCLUDED.name,
      parent_code = EXCLUDED.parent_code,
      sort = EXCLUDED.sort,
      kind = EXCLUDED.kind;
  " >/dev/null
}

echo "初始化普通用户基础权限节点..."

run_psql "$DB_NAME" "
  INSERT INTO app_roles (code, name)
  VALUES ('free', '免费')
  ON CONFLICT (code) DO NOTHING;

  DELETE FROM app_role_permissions;
  DELETE FROM app_permissions;
" >/dev/null

upsert_permission "profile" "个人中心" "" 100 "group"
upsert_permission "profile:view" "查看个人资料" "profile" 110 "action"
upsert_permission "profile:update" "更新个人资料" "profile" 120 "action"

PERMISSION_COUNT="$(
  run_psql "$DB_NAME" "
    SELECT COUNT(*)
    FROM app_permissions
    WHERE code IN (
      'profile',
      'profile:view',
      'profile:update'
    );
  " | tr -d '[:space:]'
)"

echo "普通用户基础权限初始化完成"
echo "app_roles: free"
echo "app_permissions: ${PERMISSION_COUNT}"
