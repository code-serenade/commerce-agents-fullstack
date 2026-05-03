#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

require_command curl
require_command sed
require_command awk
require_command tr
require_command base64
require_command python3

load_env
parse_database_url

CONFIG_PATH="${CONFIG_PATH:-$PROJECT_ROOT/config/services.toml}"

if [ ! -f "$CONFIG_PATH" ]; then
  echo "错误：找不到配置文件 $CONFIG_PATH"
  exit 1
fi

read_jwt_verify_url() {
  awk '
    /^\[jwt_verify\]/ { in_section=1; next }
    /^\[/ && in_section { exit }
    in_section && $0 ~ /^[[:space:]]*url[[:space:]]*=/ {
      sub(/^[[:space:]]*url[[:space:]]*=[[:space:]]*"/, "", $0)
      sub(/"[[:space:]]*$/, "", $0)
      print
      exit
    }
  ' "$CONFIG_PATH"
}

decode_sub_from_jwt() {
  local token="$1"
  local payload
  payload="$(printf '%s' "$token" | cut -d '.' -f 2)"

  if [ -z "$payload" ] || [ "$payload" = "$token" ]; then
    echo "错误：非法 JWT 格式" >&2
    return 1
  fi

  payload="${payload//-/+}"
  payload="${payload//_/\/}"

  case $(( ${#payload} % 4 )) in
    2) payload="${payload}==" ;;
    3) payload="${payload}=" ;;
    1) echo "错误：非法 JWT payload" >&2; return 1 ;;
  esac

  local decoded
  if ! decoded="$(printf '%s' "$payload" | base64 -d 2>/dev/null)"; then
    decoded="$(printf '%s' "$payload" | base64 -D 2>/dev/null)" || {
      echo "错误：JWT payload base64 解码失败" >&2
      return 1
    }
  fi

  printf '%s' "$decoded" \
    | sed -n 's/.*"sub"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p'
}

extract_user_profile() {
  python3 -c '
import json, sys

payload = json.load(sys.stdin)
data = payload.get("data")
if isinstance(data, dict):
    payload = data

user_id = payload.get("id", "")
display_id = payload.get("display_user_id", "")
display_name = payload.get("display_name", "")

print(user_id)
print(display_id)
print(display_name)
'
}

extract_access_token() {
  sed -n 's/.*"accessToken"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p'
}

json_escape() {
  python3 -c 'import json, sys; print(json.dumps(sys.argv[1])[1:-1])' "$1"
}

sql_escape() {
  printf '%s' "$1" | sed "s/'/''/g"
}

JWT_VERIFY_URL="$(read_jwt_verify_url)"
if [ -z "$JWT_VERIFY_URL" ]; then
  echo "错误：config/services.toml 中缺少 jwt_verify.url"
  exit 1
fi

AUTH_BASE_URL="${JWT_VERIFY_URL%/internal/jwt_verify_config}"
LOGIN_URL="${AUTH_BASE_URL}/auth/session/login"
ME_URL="${AUTH_BASE_URL}/auth/user/me"

printf "请输入 root 账号（用户名或邮箱）: "
read -r IDENTIFIER

if [ -z "$IDENTIFIER" ]; then
  echo "错误：账号不能为空"
  exit 1
fi

printf "请输入 root 密码: "
stty -echo
read -r PASSWORD
stty echo
printf "\n"

if [ -z "$PASSWORD" ]; then
  echo "错误：密码不能为空"
  exit 1
fi

LOGIN_RESPONSE="$(curl -fsS \
  -H 'Content-Type: application/json' \
  -X POST "$LOGIN_URL" \
  -d "{\"identifier\":\"$(json_escape "$IDENTIFIER")\",\"password\":\"$(json_escape "$PASSWORD")\"}"
)"

ACCESS_TOKEN="$(printf '%s' "$LOGIN_RESPONSE" | extract_access_token)"
if [ -z "$ACCESS_TOKEN" ]; then
  echo "错误：登录响应中缺少 accessToken"
  exit 1
fi

ME_RESPONSE="$(curl -fsS \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  "$ME_URL"
)"

USER_PROFILE="$(printf '%s' "$ME_RESPONSE" | extract_user_profile)"
USER_ID="$(printf '%s\n' "$USER_PROFILE" | sed -n '1p')"
DISPLAY_ID="$(printf '%s\n' "$USER_PROFILE" | sed -n '2p')"
DISPLAY_NAME="$(printf '%s\n' "$USER_PROFILE" | sed -n '3p')"

if [ -z "$USER_ID" ]; then
  USER_ID="$(decode_sub_from_jwt "$ACCESS_TOKEN")"
fi

if [ -z "$USER_ID" ]; then
  echo "错误：无法从 auth access token 中解析 user_id"
  exit 1
fi

if [ -z "$DISPLAY_ID" ]; then
  DISPLAY_ID="$USER_ID"
fi

if [ -z "$DISPLAY_NAME" ]; then
  DISPLAY_NAME="$IDENTIFIER"
fi

ROLE_ID="$(
  run_psql "$DB_NAME" "
    WITH upserted AS (
      INSERT INTO admin_roles (name, code)
      VALUES ('Root', 'root')
      ON CONFLICT (code) DO NOTHING
      RETURNING id
    )
    SELECT id FROM upserted
    UNION ALL
    SELECT id FROM admin_roles WHERE code = 'root'
    LIMIT 1;
  " | awk 'NF { print $1 }' | tail -n 1 | tr -d '[:space:]'
)"

if [ -z "$ROLE_ID" ]; then
  echo "错误：初始化 root 角色失败"
  exit 1
fi

run_psql "$DB_NAME" "
  INSERT INTO admin_users (user_id, display_id, display_name, remark, status)
  VALUES (
    '${USER_ID}',
    '$(sql_escape "$DISPLAY_ID")',
    '$(sql_escape "$DISPLAY_NAME")',
    NULL,
    'enabled'
  )
  ON CONFLICT (user_id) DO UPDATE
  SET display_id = EXCLUDED.display_id,
      display_name = EXCLUDED.display_name,
      remark = EXCLUDED.remark,
      status = EXCLUDED.status;
" >/dev/null

run_psql "$DB_NAME" "
  INSERT INTO admin_user_roles (user_id, role_id)
  SELECT '${USER_ID}'::uuid, id
  FROM admin_roles
  WHERE code = 'root'
  ON CONFLICT (user_id, role_id) DO NOTHING;
" >/dev/null

run_psql "$DB_NAME" "
  DELETE FROM admin_role_permissions
  WHERE role_id = ${ROLE_ID};
" >/dev/null

USER_ROLE_COUNT="$(
  run_psql "$DB_NAME" "
    SELECT COUNT(*)
    FROM admin_user_roles
    WHERE user_id = '${USER_ID}'::uuid
      AND role_id = ${ROLE_ID};
  " | awk 'NF { print $1 }' | tail -n 1 | tr -d '[:space:]'
)"

if [ "${USER_ROLE_COUNT:-0}" != "1" ]; then
  echo "错误：root 用户角色绑定失败"
  exit 1
fi

echo "root 初始化完成"
echo "auth user_id: ${USER_ID}"
echo "display_id: ${DISPLAY_ID}"
echo "display_name: ${DISPLAY_NAME}"
echo "role_id: ${ROLE_ID}"
echo "auth login url: ${LOGIN_URL}"
