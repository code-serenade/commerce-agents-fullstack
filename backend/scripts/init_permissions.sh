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
    INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
    VALUES ('${code}', '${name}', ${parent_sql}, ${sort}, '${kind}')
    ON CONFLICT (code) DO UPDATE
    SET
      name = EXCLUDED.name,
      parent_code = EXCLUDED.parent_code,
      sort = EXCLUDED.sort,
      kind = EXCLUDED.kind;
  " >/dev/null
}

upsert_menu() {
  local name="$1"
  local code="$2"
  local parent_code="$3"
  local permission_code="$4"
  local sort_hint="$5"

  local parent_id_sql="NULL"
  if [ -n "$parent_code" ]; then
    parent_id_sql="(SELECT id FROM admin_menus WHERE permission_code = '${parent_code}' LIMIT 1)"
  fi

  run_psql "$DB_NAME" "
    UPDATE admin_menus
    SET
      name = '${name}',
      parent_id = ${parent_id_sql},
      permission_code = '${permission_code}'
    WHERE permission_code = '${code}'
       OR (
         permission_code IS NULL
         AND name = '${name}'
         AND parent_id IS NOT DISTINCT FROM ${parent_id_sql}
       );

    INSERT INTO admin_menus (name, parent_id, permission_code)
    SELECT '${name}', ${parent_id_sql}, '${permission_code}'
    WHERE NOT EXISTS (
      SELECT 1 FROM admin_menus WHERE permission_code = '${code}'
    );
  " >/dev/null

  # Force a deterministic order when listing by id during development.
  if [ -n "$sort_hint" ]; then
    :
  fi
}

upsert_role() {
  local code="$1"
  local name="$2"

  run_psql "$DB_NAME" "
    INSERT INTO admin_roles (code, name)
    VALUES ('${code}', '${name}')
    ON CONFLICT (code) DO NOTHING;
  " >/dev/null
}

grant_role_permissions() {
  local role_code="$1"
  shift

  local permission_values=""
  local permission_code
  for permission_code in "$@"; do
    if [ -n "$permission_values" ]; then
      permission_values="${permission_values},"
    fi
    permission_values="${permission_values}('${permission_code}')"
  done

  run_psql "$DB_NAME" "
    WITH role_row AS (
      SELECT id
      FROM admin_roles
      WHERE code = '${role_code}'
    ),
    permission_rows AS (
      SELECT id
      FROM admin_permissions
      WHERE code IN (
        SELECT code
        FROM (VALUES ${permission_values}) AS permission_codes(code)
      )
    )
    INSERT INTO admin_role_permissions (role_id, permission_id)
    SELECT role_row.id, permission_rows.id
    FROM role_row
    CROSS JOIN permission_rows
    ON CONFLICT (role_id, permission_id) DO NOTHING;
  " >/dev/null
}

echo "初始化基础权限节点..."

run_psql "$DB_NAME" "
  TRUNCATE admin_menus, admin_role_permissions, admin_permissions RESTART IDENTITY CASCADE;
" >/dev/null

upsert_permission "dashboard" "Dashboard" "" 100 "group"

upsert_permission "accounts" "Accounts" "" 200 "group"
upsert_permission "accounts:admin_users" "Admin Users" "accounts" 210 "group"
upsert_permission "accounts:app_users" "App Users" "accounts" 220 "group"

upsert_permission "access_control" "Access Control" "" 300 "group"
upsert_permission "access_control:roles" "Roles" "access_control" 310 "group"
upsert_permission "access_control:role_permissions" "Role Permissions" "access_control" 320 "group"
upsert_permission "access_control:app_roles" "App Roles" "access_control" 330 "group"
upsert_permission "access_control:app_role_permissions" "App Role Permissions" "access_control" 340 "group"

echo "初始化基础菜单..."

upsert_menu "Dashboard" "dashboard" "" "dashboard" "100"

upsert_menu "Accounts" "accounts" "" "accounts" "200"
upsert_menu "Admin Users" "accounts:admin_users" "accounts" "accounts:admin_users" "210"
upsert_menu "App Users" "accounts:app_users" "accounts" "accounts:app_users" "220"

upsert_menu "Access Control" "access_control" "" "access_control" "300"
upsert_menu "Roles" "access_control:roles" "access_control" "access_control:roles" "310"
upsert_menu "Role Permissions" "access_control:role_permissions" "access_control" "access_control:role_permissions" "320"
upsert_menu "App Roles" "access_control:app_roles" "access_control" "access_control:app_roles" "330"
upsert_menu "App Role Permissions" "access_control:app_role_permissions" "access_control" "access_control:app_role_permissions" "340"

echo "初始化后台基础角色..."

upsert_role "admin" "超级管理员"
upsert_role "support" "客服"

grant_role_permissions "admin" \
  "dashboard" \
  "accounts" \
  "accounts:admin_users" \
  "accounts:app_users" \
  "access_control" \
  "access_control:roles" \
  "access_control:role_permissions" \
  "access_control:app_roles" \
  "access_control:app_role_permissions"

grant_role_permissions "support" \
  "dashboard" \
  "accounts:app_users" \
  "access_control:app_roles" \
  "access_control:app_role_permissions"

PERMISSION_COUNT="$(
  run_psql "$DB_NAME" "
    SELECT COUNT(*)
    FROM admin_permissions
    WHERE code IN (
      'dashboard',
      'accounts',
      'accounts:admin_users',
      'accounts:app_users',
      'access_control',
      'access_control:roles',
      'access_control:role_permissions',
      'access_control:app_roles',
      'access_control:app_role_permissions'
    );
  " | tr -d '[:space:]'
)"

MENU_COUNT="$(
  run_psql "$DB_NAME" "
    SELECT COUNT(*)
    FROM admin_menus
    WHERE permission_code IN (
      'dashboard',
      'accounts',
      'accounts:admin_users',
      'accounts:app_users',
      'access_control',
      'access_control:roles',
      'access_control:role_permissions',
      'access_control:app_roles',
      'access_control:app_role_permissions'
    );
  " | tr -d '[:space:]'
)"

ROLE_COUNT="$(
  run_psql "$DB_NAME" "
    SELECT COUNT(*)
    FROM admin_roles
    WHERE code IN ('admin', 'support');
  " | tr -d '[:space:]'
)"

ROLE_PERMISSION_COUNT="$(
  run_psql "$DB_NAME" "
    SELECT COUNT(*)
    FROM admin_role_permissions rp
    JOIN admin_roles r ON r.id = rp.role_id
    WHERE r.code IN ('admin', 'support');
  " | tr -d '[:space:]'
)"

echo "基础权限初始化完成"
echo "admin_permissions: ${PERMISSION_COUNT}"
echo "admin_menus: ${MENU_COUNT}"
echo "admin_roles: ${ROLE_COUNT}"
echo "admin_role_permissions: ${ROLE_PERMISSION_COUNT}"
