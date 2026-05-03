#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

"$SCRIPT_DIR/migrate.sh" refresh
"$SCRIPT_DIR/generate_entity.sh"

echo "数据库已 refresh，entity 已重新生成"
