#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUN_DIR="$ROOT_DIR/temp/run"
LOG_DIR="$ROOT_DIR/temp/logs"

mkdir -p "$RUN_DIR" "$LOG_DIR"

usage() {
  cat <<'EOF'
用法：
  ./manage.sh init_env [frontend|backend]
  ./manage.sh backend start|stop|restart|status
  ./manage.sh frontend install
  ./manage.sh frontend <app> start|stop|restart|status

示例：
  ./manage.sh init_env
  ./manage.sh init_env frontend
  ./manage.sh init_env backend
  ./manage.sh backend start
  ./manage.sh backend stop
  ./manage.sh frontend install
  ./manage.sh frontend admin start
  ./manage.sh frontend admin stop

日志：
  temp/logs/backend.log
  temp/logs/frontend-<app>.log
EOF
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "错误：缺少命令 '$1'"
    exit 1
  fi
}

run_frontend_init_env() {
  require_command pnpm
  echo "==> 安装前端依赖"
  pnpm -C "$ROOT_DIR/frontend" install
  echo "==> 初始化前端 env"
  pnpm -C "$ROOT_DIR/frontend" env:init
}

run_backend_init_env() {
  local script="$ROOT_DIR/backend/scripts/init.sh"

  if [ ! -f "$script" ]; then
    echo "错误：后端初始化脚本不存在：backend/scripts/init.sh"
    exit 1
  fi

  echo "==> 初始化后端 env"
  (
    cd "$ROOT_DIR/backend"
    bash scripts/init.sh
  )
  echo "后端 env 已初始化"
}

init_env() {
  local target="${1:-all}"

  case "$target" in
    all)
      run_backend_init_env
      run_frontend_init_env
      ;;
    backend)
      run_backend_init_env
      ;;
    frontend)
      run_frontend_init_env
      ;;
    *)
      echo "错误：未知 init_env 目标：$target"
      usage
      exit 1
      ;;
  esac
}

is_running() {
  local pid_file="$1"

  if [ ! -f "$pid_file" ]; then
    return 1
  fi

  local pid
  pid="$(cat "$pid_file")"

  if [ -z "$pid" ]; then
    return 1
  fi

  kill -0 "$pid" >/dev/null 2>&1
}

start_process() {
  local name="$1"
  local pid_file="$2"
  local log_file="$3"
  shift 3

  if is_running "$pid_file"; then
    echo "${name} 已在运行，PID: $(cat "$pid_file")"
    return 0
  fi

  echo "==> 启动 ${name}"
  echo "    日志：$log_file"

  : > "$log_file"
  nohup "$@" >"$log_file" 2>&1 &
  echo "$!" > "$pid_file"
}

print_log_tail() {
  local log_file="$1"

  if [ -f "$log_file" ]; then
    echo
    echo "最近日志："
    tail -n 80 "$log_file"
  fi
}

fail_start() {
  local name="$1"
  local pid_file="$2"
  local log_file="$3"

  rm -f "$pid_file"
  echo "错误：${name} 启动失败"
  echo "日志：$log_file"
  print_log_tail "$log_file"
  exit 1
}

wait_process_stable() {
  local name="$1"
  local pid_file="$2"
  local log_file="$3"
  local seconds="${4:-5}"
  local i

  for ((i = 1; i <= seconds; i++)); do
    if ! is_running "$pid_file"; then
      fail_start "$name" "$pid_file" "$log_file"
    fi
    sleep 1
  done

  echo "${name} 已启动，PID: $(cat "$pid_file")"
}

wait_http_url() {
  local name="$1"
  local pid_file="$2"
  local log_file="$3"
  local url="$4"
  local seconds="${5:-120}"
  local i

  echo "    等待访问地址：${url}"

  for ((i = 1; i <= seconds; i++)); do
    if ! is_running "$pid_file"; then
      fail_start "$name" "$pid_file" "$log_file"
    fi

    if command -v curl >/dev/null 2>&1 && curl -sS -o /dev/null --max-time 2 "$url" >/dev/null 2>&1; then
      echo "${name} 已启动，PID: $(cat "$pid_file")"
      echo "访问地址：${url}"
      return 0
    fi

    sleep 1
  done

  echo "错误：${name} 进程仍在运行，但没有在 ${seconds} 秒内连上访问地址"
  echo "日志：$log_file"
  print_log_tail "$log_file"
  exit 1
}

stop_process() {
  local name="$1"
  local pid_file="$2"

  if ! is_running "$pid_file"; then
    rm -f "$pid_file"
    echo "${name} 未运行"
    return 0
  fi

  local pid
  pid="$(cat "$pid_file")"

  echo "==> 停止 ${name}，PID: $pid"
  kill "$pid" >/dev/null 2>&1 || true

  local i
  for i in 1 2 3 4 5; do
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      rm -f "$pid_file"
      echo "${name} 已停止"
      return 0
    fi
    sleep 1
  done

  echo "==> 强制停止 ${name}"
  kill -9 "$pid" >/dev/null 2>&1 || true
  rm -f "$pid_file"
  echo "${name} 已停止"
}

status_process() {
  local name="$1"
  local pid_file="$2"

  if is_running "$pid_file"; then
    echo "${name} 运行中，PID: $(cat "$pid_file")"
  else
    rm -f "$pid_file"
    echo "${name} 未运行"
  fi
}

print_frontend_url() {
  local app="$1"
  local pid_file="$2"
  local log_file="$3"
  local url=""
  local i

  for i in 1 2 3 4 5 6 7 8 9 10; do
    if ! is_running "$pid_file"; then
      fail_start "frontend:${app}" "$pid_file" "$log_file"
    fi

    url="$(grep -E 'Local:' "$log_file" 2>/dev/null | grep -Eo 'https?://(localhost|127\.0\.0\.1|\[::1\])(:[0-9]+)?/?' | head -n 1 || true)"
    if [ -z "$url" ]; then
      url="$(grep -Eo 'https?://(localhost|127\.0\.0\.1|\[::1\])(:[0-9]+)?/?' "$log_file" 2>/dev/null | head -n 1 || true)"
    fi
    if [ -n "$url" ]; then
      echo "frontend:${app} 已启动，PID: $(cat "$pid_file")"
      echo "访问地址：$url"
      return 0
    fi
    sleep 1
  done

  echo "错误：frontend:${app} 未能在日志中找到访问地址"
  echo "日志：$log_file"
  print_log_tail "$log_file"
  exit 1
}

install_frontend_deps() {
  require_command pnpm
  echo "==> 安装前端依赖"
  pnpm -C "$ROOT_DIR/frontend" install
}

backend_url() {
  local config_file="$ROOT_DIR/backend/config/services.toml"
  local port=""

  if [ -f "$config_file" ]; then
    port="$(awk '
      /^\[http\]/ { in_http = 1; next }
      /^\[/ { in_http = 0 }
      in_http && /^[[:space:]]*port[[:space:]]*=/ {
        gsub(/[^0-9]/, "", $0)
        print $0
        exit
      }
    ' "$config_file")"
  fi

  if [ -z "$port" ]; then
    port="19878"
  fi

  echo "http://127.0.0.1:${port}/"
}

ensure_frontend_deps() {
  if [ -d "$ROOT_DIR/frontend/node_modules" ]; then
    return 0
  fi

  echo "错误：前端依赖还没有安装。"
  echo "请先运行：./manage.sh frontend install"
  exit 1
}

backend() {
  local action="${1:-}"
  local pid_file="$RUN_DIR/backend.pid"
  local log_file="$LOG_DIR/backend.log"

  case "$action" in
    start)
      require_command cargo
      start_process "backend" "$pid_file" "$log_file" \
        bash -c 'cd "$1" && shift && exec "$@"' _ "$ROOT_DIR/backend" cargo run -p web-server
      wait_http_url "backend" "$pid_file" "$log_file" "$(backend_url)" 120
      ;;
    stop)
      stop_process "backend" "$pid_file"
      ;;
    restart)
      stop_process "backend" "$pid_file"
      backend start
      ;;
    status)
      status_process "backend" "$pid_file"
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

frontend() {
  local app="${1:-}"
  local action="${2:-}"

  if [ "$app" = "install" ]; then
    install_frontend_deps
    return 0
  fi

  if [ -z "$app" ] || [ -z "$action" ]; then
    usage
    exit 1
  fi

  local app_dir="$ROOT_DIR/frontend/apps/${app}"
  if [ ! -d "$app_dir" ]; then
    echo "错误：前端 app 不存在：frontend/apps/${app}"
    exit 1
  fi

  local pid_file="$RUN_DIR/frontend-${app}.pid"
  local log_file="$LOG_DIR/frontend-${app}.log"

  case "$action" in
    start)
      require_command pnpm
      ensure_frontend_deps
      if is_running "$pid_file"; then
        echo "frontend:${app} 已在运行，PID: $(cat "$pid_file")"
      else
        start_process "frontend:${app}" "$pid_file" "$log_file" \
          pnpm -C "$app_dir" dev
      fi
      print_frontend_url "$app" "$pid_file" "$log_file"
      ;;
    stop)
      stop_process "frontend:${app}" "$pid_file"
      ;;
    restart)
      stop_process "frontend:${app}" "$pid_file"
      frontend "$app" start
      ;;
    status)
      status_process "frontend:${app}" "$pid_file"
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

case "${1:-}" in
  init_env)
    shift
    init_env "$@"
    ;;
  backend)
    shift
    backend "$@"
    ;;
  frontend)
    shift
    frontend "$@"
    ;;
  ""|help|--help|-h)
    usage
    ;;
  *)
    usage
    exit 1
    ;;
esac
