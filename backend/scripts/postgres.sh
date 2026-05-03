#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
. "$SCRIPT_DIR/common.sh"

load_env
parse_database_url

require_command docker

ACTION="${1:-up}"

case "$ACTION" in
  up)
    mkdir -p "$PG_DATA_DIR"
    chmod 700 "$PG_DATA_DIR"

    if docker_container_running; then
      echo "PostgreSQL 容器已在运行：$DB_CONTAINER_NAME"
    elif docker_container_exists; then
      echo "启动已有容器：$DB_CONTAINER_NAME"
      docker start "$DB_CONTAINER_NAME" >/dev/null
    else
      echo "创建并启动 PostgreSQL 容器：$DB_CONTAINER_NAME"
      docker run -d \
        --name "$DB_CONTAINER_NAME" \
        --restart unless-stopped \
        -e POSTGRES_USER="$DB_USER" \
        -e POSTGRES_PASSWORD="$DB_PASSWORD" \
        -e POSTGRES_DB=postgres \
        -v "$PG_DATA_DIR":/var/lib/postgresql/data \
        -p "$DB_PORT":5432 \
        "$PG_IMAGE" >/dev/null
    fi

    wait_for_postgres
    "$SCRIPT_DIR/init_db.sh"
    echo "PostgreSQL 已就绪"
    echo "容器名: $DB_CONTAINER_NAME"
    echo "数据库: $DB_NAME"
    echo "地址: $DATABASE_URL"
    ;;
  status)
    if docker_container_running; then
      docker ps --filter "name=^${DB_CONTAINER_NAME}$" \
        --format '容器 {{.Names}} 正在运行，端口：{{.Ports}}'
    elif docker_container_exists; then
      docker ps -a --filter "name=^${DB_CONTAINER_NAME}$" \
        --format '容器 {{.Names}} 已存在但未运行，状态：{{.Status}}'
    else
      echo "容器不存在：$DB_CONTAINER_NAME"
    fi
    ;;
  stop)
    if docker_container_running; then
      docker stop "$DB_CONTAINER_NAME" >/dev/null
      echo "已停止容器：$DB_CONTAINER_NAME"
    else
      echo "容器未运行：$DB_CONTAINER_NAME"
    fi
    ;;
  rm)
    if docker_container_exists; then
      docker rm -f "$DB_CONTAINER_NAME" >/dev/null
      echo "已删除容器：$DB_CONTAINER_NAME"
    else
      echo "容器不存在：$DB_CONTAINER_NAME"
    fi
    ;;
  *)
    echo "用法: ./scripts/postgres.sh [up|status|stop|rm]"
    exit 1
    ;;
esac
