SHELL := /bin/bash

.PHONY: help init \
	db-up db-status db-stop db-rm db-init db-clear db-truncate \
	migrate-up migrate-down migrate-fresh migrate-refresh migrate-reset migrate-status migrate-gen \
	entity-generate fresh-db \
	build test clean \
	postgres generate-entity

help:
	@echo "AI 数据库工具："
	@echo "  make init                    - 初始化 .env"
	@echo "  make db-up                   - 用 Docker 启动 PostgreSQL，并确保目标数据库存在"
	@echo "  make db-status               - 查看数据库容器状态"
	@echo "  make db-stop                 - 停止数据库容器"
	@echo "  make db-rm                   - 删除数据库容器"
	@echo "  make db-init                 - 初始化 app 数据库"
	@echo "  make db-clear                - 清空整个数据库中的 public schema"
	@echo "  make db-truncate TABLE=xxx   - 清空指定表并重置自增"
	@echo ""
	@echo "SeaORM 工具："
	@echo "  make migrate-up             - 执行待处理迁移"
	@echo "  make migrate-down           - 回滚一次迁移"
	@echo "  make migrate-fresh          - 删除全部表后重新执行所有迁移"
	@echo "  make migrate-refresh        - 回滚全部迁移后重新执行"
	@echo "  make migrate-reset          - 回滚全部迁移"
	@echo "  make migrate-status         - 查看迁移状态"
	@echo "  make migrate-gen NAME=xxx   - 生成新的迁移文件"
	@echo "  make entity-generate        - 仅根据当前数据库表结构生成 entity，不修改数据库"
	@echo "  make fresh-db               - refresh 数据库后重新生成 entity"
	@echo ""
	@echo "其它："
	@echo "  make build"
	@echo "  make test"
	@echo "  make clean"

init:
	@./scripts/init.sh

db-up:
	@./scripts/postgres.sh up

db-status:
	@./scripts/postgres.sh status

db-stop:
	@./scripts/postgres.sh stop

db-rm:
	@./scripts/postgres.sh rm

db-init:
	@./scripts/init_db.sh

db-clear:
	@./scripts/clear_db.sh

db-truncate:
	@if [ -z "$(TABLE)" ]; then \
		echo "错误：需要提供 TABLE。用法：make db-truncate TABLE=your_table"; \
		exit 1; \
	fi
	@./scripts/truncate_table.sh "$(TABLE)"

migrate-up:
	@./scripts/migrate.sh up

migrate-down:
	@./scripts/migrate.sh down

migrate-fresh:
	@./scripts/migrate.sh fresh

migrate-refresh:
	@./scripts/migrate.sh refresh

migrate-reset:
	@./scripts/migrate.sh reset

migrate-status:
	@./scripts/migrate.sh status

migrate-gen:
	@if [ -z "$(NAME)" ]; then \
		echo "错误：需要提供 NAME。用法：make migrate-gen NAME=create_users"; \
		exit 1; \
	fi
	@./scripts/migrate.sh generate "$(NAME)"

entity-generate:
	@./scripts/generate_entity.sh

fresh-db:
	@./scripts/fresh_db.sh

build:
	@cargo build

test:
	@cargo test

clean:
	@cargo clean

# 兼容旧命令
postgres: db-up
generate-entity: entity-generate
